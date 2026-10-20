//! text wrap（对应 LatexIndent/Wrap.pm）：textWrapOptions.columns 折行。

use pcre2::bytes::Regex;

use crate::patterns::{gn, replace_all_with};
use crate::settings::{as_bool, as_i64, as_str};
use crate::state;
use crate::tokens::tok;

fn re(src: &str) -> Regex {
    crate::patterns::re_new(src).expect("wrap regex")
}

fn tw<T>(f: impl FnOnce(&serde_yaml::Mapping) -> T) -> T {
    state::settings(|s| {
        let m = crate::settings::get_path(&s.root, &["modifyLineBreaks", "textWrapOptions"])
            .and_then(|v| v.as_mapping())
            .cloned()
            .unwrap_or_default();
        f(&m)
    })
}

/// `Text::Wrap::wrap('', '', text)` 的精确移植（Perl 5.40 自带 Text::Wrap 2009.001）。
///
/// 旧实现是「空白串+词」贪心累积的近似，在 `huge=wrap` 下与 Text::Wrap 分歧：
/// Text::Wrap 每轮优先用 `\G(UNIT{0,$ll})($break|\n+|\z)` 尝试「最多 `columns-1` 个字素，
/// 且恰好停在 break / 换行串 / 串尾」，**只有这条整体失败**才退回「硬切 `$ll` 个字素」。
/// 所以末片正好收在 `\z` 时不补 separator —— 旧实现会给长行末尾多补一个分隔符
/// （`long-lines1-mod-5` 末尾多一个 `|`）。
///
/// 另注 `$Text::Wrap::break` 默认值 `(?>\n|\r\n|\s\pM*)`：先试 `\n`，故换行是硬断点；
/// `$separator2` 为 undef，所以 `$nl` 恒等于 `$separator`。
pub fn wrap_text(text: &str, columns: usize) -> String {
    if columns == 0 {
        return text.to_string();
    }
    let sep = tw(|m| as_str(get_opt(m, "separator")).unwrap_or("").to_string());
    let sep = if sep.is_empty() {
        "\n".to_string()
    } else {
        sep
    };
    // latexindent defaultSettings.yaml 的默认值是 overflow（不是 Text::Wrap 自己的 wrap）
    let huge = tw(|m| as_str(get_opt(m, "huge")).unwrap_or("overflow").to_string());

    // `textWrapOptions.break`（Perl 仅在其为真值时覆写，否则用 Text::Wrap 默认值）
    let brk = tw(|m| as_str(get_opt(m, "break")).unwrap_or("").to_string());
    let brk = if brk.is_empty() {
        r"(?>\n|\r\n|\s\pM*)".to_string()
    } else {
        brk
    };
    // 一个「字素」单元：非换行的非组合字符 + 其后的组合标记；或行首/换行后的孤立组合标记
    let unit = r"(?>(?!\n)\PM\pM*|(?<![^\n])\pM+)";

    // `textWrapOptions.tabstop`（Perl 仅在其有值时覆写 Text::Wrap 的默认 8）
    let tabstop = tw(|m| as_i64(get_opt(m, "tabstop")).unwrap_or(8)).max(1) as usize;
    // `textWrapOptions.unexpand`：输出时把成串空格折回制表符（Text::Tabs::unexpand）
    let unexpand = tw(|m| as_bool(get_opt(m, "unexpand")));
    let t = expand_tabs(text, tabstop);
    let subj = t.as_bytes();
    let nll = columns - 1;
    let mut ll = nll;

    let re_stop = re(&format!(r"\G(?:{brk})*\z"));
    let re_a = re(&format!(r"\G({unit}{{0,{ll}}})({brk}|\n+|\z)"));
    let re_b = re(&format!(r"\G({unit}{{{ll}}})"));
    let re_c = re(&format!(r"\G([^\n]*?)(?!(?<![^\n])\pM)({brk}|\n+|\z)"));
    let mut la = re_a.capture_locations();
    let mut lb = re_b.capture_locations();
    let mut lc = re_c.capture_locations();

    let mut out = String::new();
    let mut nl = String::new();
    let mut remainder = String::new();
    let mut pos = 0usize;
    while !re_stop.is_match_at(subj, pos).unwrap_or(false) {
        if re_a.captures_read_at(&mut la, subj, pos).unwrap_or(None).is_some() {
            let (s1, e1) = la.get(1).unwrap_or((pos, pos));
            let (s2, e2) = la.get(2).unwrap_or((e1, e1));
            push_piece(&mut out, &nl, &t[s1..e1], unexpand, tabstop);
            remainder = t[s2..e2].to_string();
            pos = e2;
        } else if huge == "wrap" && re_b.captures_read_at(&mut lb, subj, pos).unwrap_or(None).is_some() {
            let (s1, e1) = lb.get(1).unwrap_or((pos, pos));
            push_piece(&mut out, &nl, &t[s1..e1], unexpand, tabstop);
            remainder = sep.clone();
            pos = e1;
        } else if huge == "overflow"
            && re_c.captures_read_at(&mut lc, subj, pos).unwrap_or(None).is_some()
        {
            let (s1, e1) = lc.get(1).unwrap_or((pos, pos));
            let (s2, e2) = lc.get(2).unwrap_or((e1, e1));
            out.push_str(&nl);
            out.push_str(&t[s1..e1]);
            remainder = t[s2..e2].to_string();
            pos = e2;
        } else if huge == "die" {
            // Perl 这里 die；本工程不因折行失败中止，退回原文
            return text.to_string();
        } else if columns < 2 {
            // Perl `return @_`（标量上下文下其实是元素个数，属上游 quirk），退回原文
            return text.to_string();
        } else {
            break;
        }
        ll = nll;
        nl = sep.clone();
    }
    out.push_str(&remainder);
    if pos < t.len() {
        out.push_str(&t[pos..]);
    }
    out
}

/// 输出一个折行片段（`$nl . $lead . $1`）。`unexpand` 打开时按 Text::Tabs 的语义折回制表符。
fn push_piece(out: &mut String, nl: &str, chunk: &str, unexpand: bool, tabstop: usize) {
    if unexpand {
        out.push_str(&unexpand_tabs(&format!("{nl}{chunk}"), tabstop));
    } else {
        out.push_str(nl);
        out.push_str(chunk);
    }
}

/// `Text::Tabs::unexpand`：先 expand，再按 `tabstop` 个字素切块，
/// 块内「两个及以上行尾空格」折成一个 `\t`；末尾残块若恰好是 `tabstop` 个空格也折成 `\t`。
fn unexpand_tabs(s: &str, tabstop: usize) -> String {
    use unicode_segmentation::UnicodeSegmentation;
    let expanded = expand_tabs(s, tabstop);
    let graphemes: Vec<&str> = expanded.graphemes(true).collect();
    let mut out = String::with_capacity(expanded.len());
    let mut i = 0usize;
    while i < graphemes.len() {
        let end = (i + tabstop).min(graphemes.len());
        let chunk: String = graphemes[i..end].concat();
        if end == graphemes.len() {
            if chunk == " ".repeat(tabstop) {
                out.push('\t');
            } else {
                out.push_str(&chunk);
            }
        } else {
            let trimmed = chunk.trim_end_matches(' ');
            out.push_str(trimmed);
            let removed = chunk.len() - trimmed.len();
            if removed >= 2 {
                out.push('\t');
            } else {
                out.push_str(&chunk[trimmed.len()..]);
            }
        }
        i = end;
    }
    out
}

/// `Text::Tabs::expand`：按字符位置把 `\t` 展开为空格，`\n` 复位列号。
fn expand_tabs(s: &str, tabstop: usize) -> String {
    let mut out = String::with_capacity(s.len());
    let mut col = 0usize;
    for ch in s.chars() {
        match ch {
            '\t' => {
                let pad = tabstop - col % tabstop;
                for _ in 0..pad {
                    out.push(' ');
                }
                col += pad;
            }
            '\n' => {
                out.push('\n');
                col = 0;
            }
            c => {
                out.push(c);
                col += 1;
            }
        }
    }
    out
}

fn width(s: &str) -> usize {
    use unicode_segmentation::UnicodeSegmentation;
    s.graphemes(true).count()
}

fn huge_is_overflow() -> bool {
    tw(|m| as_str(get_opt(m, "huge")).unwrap_or("overflow") == "overflow")
}

/// 见 `text_wrap` 内的调用点：把单行 verbatim token 换成等宽占位串。
/// 返回 `(原 token, 占位串)`，折行后按序 `replacen(.., 1)` 换回（Perl 的 `s///s` 非全局）。
fn install_verbatim_width_shim(text: &mut String) -> Vec<(String, String)> {
    // Perl 遍历 `%verbatimStorage`（哈希序不确定）；这里对 id 排序只为可复现。
    let mut items: Vec<(String, String)> = state::storage(|s| {
        let mut v: Vec<(String, String)> = s
            .verbatim
            .iter()
            .map(|(id, o)| (id.clone(), format!("{}{}{}", o.begin, o.body, o.end)))
            .filter(|(id, thing)| {
                // 含换行的对象不测量（Perl `next if $verbatimThing =~ m/\R/s`）
                !thing.contains('\n') && !thing.contains('\r') && text.contains(id.as_str())
            })
            .collect();
        v.sort();
        v
    });
    let mut out: Vec<(String, String)> = Vec::new();
    for (i, (id, thing)) in items.drain(..).enumerate() {
        let len = width(&thing);
        if len == 0 {
            continue;
        }
        let tmp = fresh_placeholder(len, text, i);
        // Perl 是 `s/$id/$tmp/s`（只换第一处）
        *text = text.replacen(&id, &tmp, 1);
        out.push((id, tmp));
    }
    out
}

/// 生成一个长度恰为 `len` 的字母数字串，且保证当前正文里没有同串（Perl 的碰撞重试）。
fn fresh_placeholder(len: usize, avoid: &str, salt: usize) -> String {
    const ALPHABET: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";
    let mut seed = 0x9e37_79b9_7f4a_7c15u64 ^ (salt as u64).wrapping_mul(0x100_0000_01b3);
    loop {
        let mut s = String::with_capacity(len);
        for _ in 0..len {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            s.push(ALPHABET[(seed % ALPHABET.len() as u64) as usize] as char);
        }
        if !avoid.contains(&s) {
            return s;
        }
    }
}

/// 句子折行（one-sentence-per-line 的 textWrapSentences）。
///
/// Perl Wrap.pm L336-345：`when: after` 时先把 `follows` **最后一行**的宽度折算成
/// `subsequentSpace`（tab 逐组换回 4 空格），再用 `columns - length(subsequentSpace)`
/// 作为实际列宽。少了这一步，缩进深的句子会折得过晚（issue-608-mod1）。
pub fn text_wrap_sentence(value: &str, follows: &str) -> String {
    let columns = tw(|m| as_i64(get_opt(m, "columns")).unwrap_or(0));
    if columns == 0 {
        return value.to_string();
    }
    // Perl 的句子折行走的也是 `text_wrap`（Wrap.pm），所以尾注释 token 一样要
    // 「先摘出、折完再挂到末尾」。少了这一步，句子中间（例如 `\begin{equation*}%`）
    // 的注释会留在原位，与 Perl 的输出分叉（opl/kiryph-mod1）。
    let (text, trailing_comments) = take_trailing_comments(value);
    // ⚠️ 这段缩进折算只在 `when: after` 生效（Perl 把它放在 `if (when eq 'after')` 里）。
    // 默认 `when: before` 时 `subsequentSpace` 为空，列宽就是 columns 本身；
    // 无条件扣减会让 before 模式的句子提前折行（opl_after_loop 之外的用例全崩）。
    let wrapped = if !tw(|m| as_str(get_opt(m, "when")).unwrap_or("before") == "after") {
        wrap_text(&text, columns.max(0) as usize)
    } else {
        let last = follows.split(['\n', '\r']).next_back().unwrap_or("");
        let n_tabs = last.matches('\t').count();
        let mut space = " ".repeat(width(last));
        for _ in 0..n_tabs {
            space = space.replacen(' ', "    ", 1);
        }
        let sub = space.chars().count() as i64;
        wrap_text(&text, (columns - sub).max(0) as usize)
    };
    if trailing_comments.is_empty() {
        return wrapped;
    }
    // Perl：`s/\R?$/$trailingComments\n/s`
    let r = re(r"(?s)\R?$");
    replace_all_with(&wrapped, &r, |_| format!("{trailing_comments}\n"))
}

/// 把块内所有尾注释 token 摘出来（Perl Wrap.pm「now we put all of the trailing
/// comments together」）。返回 (去掉注释的正文, 注释 token 串)。
fn take_trailing_comments(text: &str) -> (String, String) {
    let tc = state::patterns(|p| p.trailing_comment.clone());
    let mut text = text.to_string();
    // 注释前不是横空白、注释后紧跟换行 → 把那个换行去掉（Perl「无空格连接两行」语义）
    loop {
        let r = re(&format!(r"(?s)\H{tc}\h*\R"));
        if !r.is_match(text.as_bytes()).unwrap_or(false) {
            break;
        }
        let r2 = re(&format!(r"(?s)(\H)({tc})\h*\R"));
        text = replace_all_with(&text, &r2, |c| format!("{}{}", gn(c, 1), gn(c, 2)));
    }
    let mut comments = String::new();
    loop {
        let r = re(&format!(r"(?s)({tc})"));
        match r.captures(text.as_bytes()).unwrap() {
            Some(c) => {
                let comment = gn(&c, 1).to_string();
                text = text.replacen(&comment, "", 1);
                comments.push_str(&comment);
            }
            None => break,
        }
    }
    (text, comments)
}

fn get_opt<'a>(m: &'a serde_yaml::Mapping, key: &str) -> Option<&'a serde_yaml::Value> {
    crate::settings::get(m, key)
}

/// text_wrap 主入口：块切分 → 截尾 → 尾注释抽取 → 折行 → 拼回。
pub fn text_wrap(body: &str) -> String {
    let t = tok(|t| t.clone());
    let tc = state::patterns(|p| p.trailing_comment.clone());
    let columns_i = tw(|m| as_i64(get_opt(m, "columns")).unwrap_or(0));
    if columns_i == 0 {
        return body.to_string();
    }
    let columns = columns_i as usize;
    let when_after = tw(|m| as_str(get_opt(m, "when")).unwrap_or("before") == "after");
    let multi = tw(|m| as_bool(get_opt(m, "multipleSpacesToSingle")));
    let remove_block_lb = tw(|m| as_bool(get_opt(m, "removeBlockLineBreaks")));
    // `comments` 是映射（`{wrap: 0/1, inheritLeadingSpace: 0/1}`），
    // 旧代码对整个映射取 as_bool → 恒为 false，`comments.wrap` 从未生效。
    let comments_wrap = tw(|m| {
        as_bool(
            get_opt(m, "comments")
                .and_then(|v| v.as_mapping())
                .and_then(|mm| crate::settings::get(mm, "wrap")),
        )
    });
    // blocksFollow：分支顺序照 Perl `sort keys`（blankLine、commentOnPreviousLine、
    // headings、other、par、verbatim），每个分支自带 (?:...) 分组——Perl 的 qr// 插值
    // 即分组，裸拼 alternation 会把分支内部的 | 提升到顶层并改写优先级。
    // 整体再包一层 (?:...) 使 (?:\h|\R)* 尾缀作用于全部分支。
    //
    // ⚠️ Perl 原版里的 `\G` 分支一律改写成 `\A`。实测（perl 5.40）：
    //   split(/(\Gx)/, "xax") 与 split(/(\Ax)/, "xax") 结果完全相同（"|x|ax"），
    //   即 **split 内部的 `\G` 等价于 `\A`**（只在 subject 起点成立）。
    // 若沿用 `\G`，pcre2 的 captures_iter 会让它在每个扫描起点都成立，
    // 于是 `\G(?!blanklines)(?:\h|\R)*` 会在每个换行处吃掉 `\n` 当分隔符 →
    // 段落被拼成一行（text-wrap/verbatim4-mod1：`\end{commandshell} after`）。
    let (follow_parts, follow_comment_tail, headings_re) = tw(|m| {
        let bf = get_opt(m, "blocksFollow")
            .and_then(|v| v.as_mapping())
            .cloned()
            .unwrap_or_default();
        let blank_line = as_bool(crate::settings::get(&bf, "blankLine"));
        let comment_prev = as_bool(crate::settings::get(&bf, "commentOnPreviousLine"));
        let mut parts: Vec<String> = Vec::new();
        if blank_line {
            let bl = &t.blanklines;
            parts.push(format!(
                r"(?:\A(?:{bl}\R)+)|(?:\A(?:{bl}\R)+)|(?:(?:{bl}\h*\R)+)|\R{{2,}}"
            ));
        }
        if comment_prev {
            parts.push(format!(r"(?m)(?:^\h*{tc}\R?)+"));
        }
        let mut headings_re = String::new();
        if as_bool(crate::settings::get(&bf, "headings")) {
            // Perl 不看 lookForThis：indentAfterHeadings 全部条目都进 blocksFollow；
            // 名字含 * 的放交替串前部（Perl 语义）
            let (plain, starred): (Vec<String>, Vec<String>) = state::settings(|s| {
                let (mut plain, mut starred) = (Vec::new(), Vec::new());
                if let Some(m) = crate::settings::get(&s.root, "indentAfterHeadings")
                    .and_then(|x| x.as_mapping())
                {
                    for (k, _) in m {
                        let name = k.as_str().unwrap_or_default();
                        if name.contains('*') {
                            starred.push(name.replace('*', r"\*"));
                        } else {
                            plain.push(name.to_string());
                        }
                    }
                }
                (plain, starred)
            });
            for h in starred.into_iter().chain(plain.into_iter()) {
                let one = format!(
                    r"\\{h}\h*(?:\[[^]]*?\])?\h*\{{[^}}]*?\}}\h*(?:\\label\{{[^}}]*?\}})?"
                );
                // Perl 的 `$headingsRegEx` 累积的是同一个 pattern（供 when:after 从
                // 前一个分隔符里剥掉标题行）
                headings_re.push_str(&one);
                headings_re.push('|');
                parts.push(one);
            }
            headings_re = headings_re.trim_end_matches('|').to_string();
        }
        if let Some(o) = other_regex(&bf) {
            // Perl `$blocksFollowEachPart = qr/$yesNo/x;` —— blocksFollow.other 是以 **/x** 编译的
            // （blocksBeginWith/blocksEndBefore 则不带 /x）。默认配置里这段就带 `# regex` 注释
            // 和多行 `|`，不按 /x 解析会要求字面空格与 `#`，导致 `\end{equation*}` 断不开
            // （issue-553：方程后那一大段整行未被折行）。
            //
            // 末尾补换行：用户片段常以 `#` 行尾注释收尾（如 tw-bf-myenv.yaml 的
            // `\\end\{myenv\}   # <--- new bit`），否则注释会吞掉紧随的 `)` →
            // `missing closing parenthesis` 直接 panic。补在 `(?x:...)` **内部**，
            // 换行在 x-mode 下被忽略；补在外部会变成必须匹配的字面换行（见下方注释）。
            parts.push(format!("(?x:{o}\n)"));
        }
        if as_bool(crate::settings::get(&bf, "par")) {
            parts.push(r"\R?\\par(?![a-zA-Z])".to_string());
        }
        if as_bool(crate::settings::get(&bf, "verbatim")) {
            parts.push(format!(r"{}\d+{}", t.verbatim, t.end_of_token));
        }
        if !blank_line {
            // Perl 的 `\G` 在 split 里等价 `\A`（见上方注释）：起点零宽匹配会被跳过
            parts.push(format!(r"\A(?!{})", t.blanklines));
        }
        (parts, comment_prev, headings_re)
    });
    // 注：`(?x:...)` 自带作用域，其后的 `x_guard` 补换行会落在 /x 作用域**之外**，
    // 变成一个必须匹配的字面换行 → 整条交替串失效。这类片段原样使用。
    let branches: Vec<String> = follow_parts
        .iter()
        .map(|p| {
            if p.starts_with("(?x:") {
                format!("(?:{p})")
            } else {
                format!("(?:{})", crate::patterns::x_guard(p))
            }
        })
        .collect();
    let blocks_follow = if follow_comment_tail {
        format!(
            r"(?s:(?:{})(?:\h|\R)*(?:{}\R\h*)?)",
            branches.join("|"),
            tc
        )
    } else {
        format!(r"(?s:(?:{})(?:\h|\R)*)", branches.join("|"))
    };
    let blocks_begin_with = tw(|m| {
        let b = get_opt(m, "blocksBeginWith")
            .and_then(|v| v.as_mapping())
            .cloned()
            .unwrap_or_default();
        let mut parts: Vec<String> = Vec::new();
        if as_bool(crate::settings::get(&b, "A-Z")) {
            parts.push(format!(
                r"(?!(?:{}|{}|{}))[A-Z]",
                t.blanklines, t.verbatim, t.preamble
            ));
        }
        if as_bool(crate::settings::get(&b, "a-z")) {
            parts.push("[a-z]".to_string());
        }
        if as_bool(crate::settings::get(&b, "0-9")) {
            parts.push("[0-9]".to_string());
        }
        if let Some(o) = other_regex(&b) {
            parts.push(o);
        }
        parts.join("|")
    });
    let blocks_end_before = tw(|m| {
        let b = get_opt(m, "blocksEndBefore")
            .and_then(|v| v.as_mapping())
            .cloned()
            .unwrap_or_default();
        let mut parts: Vec<String> = Vec::new();
        if as_bool(crate::settings::get(&b, "commentOnOwnLine")) {
            parts.push(format!(r"(?m)^\h*{tc}"));
        }
        if as_bool(crate::settings::get(&b, "verbatim")) {
            parts.push(format!(r"{}\d", t.verbatim));
        }
        if let Some(o) = other_regex(&b) {
            parts.push(format!(r"\R?{o}"));
        }
        parts.join("|")
    });

    // 块切分（Perl split：空匹配不作分隔符；分隔符整体进奇数位）
    let split_re = if when_after {
        re(&format!(r"(?s)(\s*{blocks_follow}+)"))
    } else {
        re(&format!(r"({blocks_follow})"))
    };
    let mut raw: Vec<String> = Vec::new();
    let mut last = 0usize;
    for m in split_re.captures_iter(body.as_bytes()) {
        let c = m.expect("match");
        let whole = c.get(0).unwrap();
        if whole.start() == whole.end() {
            // Perl split 跳过零宽匹配（如 `\A(?!blanklines)` 的起点空匹配）
            continue;
        }
        raw.push(body[last..whole.start()].to_string());
        raw.push(gn(&c, 1).to_string());
        last = whole.end();
    }
    raw.push(body[last..].to_string());

    let begin_re = re(&format!(r"(?s)^\s*(?:{blocks_begin_with})"));
    let mut out = String::new();
    let mut wrap_count: i64 = -1;
    // Perl：piece 以换行开头且累积正文以换行结尾时去掉一个开头换行（防空行叠加）
    fn push_piece(out: &mut String, mut piece: String) {
        if piece.starts_with('\n') && out.ends_with('\n') {
            piece.remove(0);
        }
        out.push_str(&piece);
    }
    for chunk in raw.iter() {
        wrap_count += 1;
        let is_follow = wrap_count % 2 == 1;
        if is_follow {
            push_piece(&mut out, chunk.clone());
            continue;
        }
        if !begin_re.is_match(chunk.as_bytes()).unwrap_or(false) {
            push_piece(&mut out, chunk.clone());
            continue;
        }
        let mut text = chunk.clone();

        // `when: after` 的后续行缩进（Perl Wrap.pm L288-401）。
        //
        // 折行发生在缩进之后，块首行已带缩进，但换出来的新行还没有。Perl 从
        // **前一个分隔符**（`@textWrapBlockStorage[$count-1]`，即 blocksFollow
        // 匹配到的 `\n \end{enumerate}\n <blankline>` 之类）的最后一行量出宽度，
        // 作为 `subsequentSpace`：既从列宽里扣掉，折完再逐行回填。
        // 少了这一步，`-m` + `textWrapOptions.when: after` 下被缩进的段落
        // 第二行起会顶格（documentation/manual 的 s*.tex 全部不是不动点）。
        let mut subsequent_space = String::new();
        let mut first_line_columns = 0usize;
        if when_after {
            let prev = if wrap_count >= 1 {
                raw[(wrap_count - 1) as usize].as_str()
            } else {
                ""
            };
            let prev_lines: Vec<&str> = prev.split('\n').collect();
            let only_comment = prev_lines.len() == 1 && tc_is_comment(prev);
            if only_comment {
                subsequent_space = String::new();
            } else {
                let mut thing = prev_lines.last().copied().unwrap_or("").to_string();
                // Perl 的这几处 `s///`（无 /g）都只替换首次出现
                thing = thing.replacen(&t.blanklines, "", 1);
                let vb_re = re(&format!(
                    r"{}\d+{}",
                    crate::patterns::esc_lit(&t.verbatim),
                    crate::patterns::esc_lit(&t.end_of_token)
                ));
                thing = remove_first_match(&thing, &vb_re);
                let tc_re = re(&tc);
                thing = remove_first_match(&thing, &tc_re);
                if !headings_re.is_empty() {
                    let hr = re(&headings_re);
                    thing = remove_first_match(&thing, &hr);
                }
                let end_re = re(r"\\end\{[^}]+?\}");
                thing = remove_first_match(&thing, &end_re);

                subsequent_space = if wrap_count == 0 {
                    String::new()
                } else {
                    " ".repeat(width(&thing))
                };
                let n_tabs = thing.matches('\t').count();
                for _ in 0..n_tabs {
                    subsequent_space = subsequent_space.replacen(' ', "    ", 1);
                }

                // 自定义 indentRules（Perl issue-444 段）
                let mut name = thing.replace("\\begin", "");
                name.retain(|c| c.is_ascii_alphanumeric() || c == '@');
                if wrap_count > 0 && !name.trim().is_empty() {
                    if let Some(rule) = text_wrap_after_indent(&name) {
                        if is_hspace_or_empty(&rule) {
                            first_line_columns = columns.saturating_sub(subsequent_space.len());
                            let t2 = thing.trim_start_matches([' ', '\t']).to_string();
                            for _ in 0..width(&t2) {
                                subsequent_space = subsequent_space.replacen(' ', "", 1);
                            }
                            subsequent_space.push_str(&rule);
                        }
                    }
                }
                let n_tabs = thing.matches('\t').count();
                for _ in 0..n_tabs {
                    subsequent_space = subsequent_space.replacen("    ", "\t", 1);
                }
            }
        }
        let wrap_columns = columns.saturating_sub(subsequent_space.len());

        // 截尾（blocksEndBefore）
        let mut tail = String::new();
        if !blocks_end_before.is_empty() {
            let re_beb = if when_after {
                re(&format!(r"(?s)(\s*{blocks_end_before})"))
            } else {
                re(&format!(r"(?s)({blocks_end_before})"))
            };
            if let Some(c) = re_beb.captures(text.as_bytes()).unwrap() {
                let whole = c.get(0).unwrap();
                tail = text[whole.start()..].to_string();
                text = text[..whole.start()].to_string();
            }
        }

        // 尾注释抽取
        let (without_comments, mut trailing_comments) = take_trailing_comments(&text);
        text = without_comments;
        if multi {
            let r = re(r"\h{2,}");
            trailing_comments = replace_all_with(&trailing_comments, &r, |_| " ".to_string());
        }

        // 折行前整理
        if remove_block_lb {
            let r = re(r"(?s)\R(?!\Z)");
            text = replace_all_with(&text, &r, |_| " ".to_string());
        }
        if multi {
            let r = re(r"\h{2,}");
            text = replace_all_with(&text, &r, |_| " ".to_string());
        }

        // 执行折行（Perl：仅 columns > 0 才调 Text::Wrap；-1 之类只合行不折行）
        if columns_i > 0 {
            // Perl Wrap.pm L490-524：`huge=overflow` 时，把**单行** verbatim 对象换成一个
            // 等宽（`get_column_width`，即字素数）的占位串再折行，最后原样换回。
            // 理由：body 里 `\verb!x+y!` 已被换成 `LTXIN-TK-VERBATIM<N>-END`（22 字符），
            // 若按 token 文本量宽度会多算十几列 → 提前断行（verbatim3-mod1）。
            let vb_shim = if huge_is_overflow() {
                install_verbatim_width_shim(&mut text)
            } else {
                Vec::new()
            };
            if first_line_columns > 0 {
                // Perl：带自定义 indentRules 时首行与后续行用不同列宽
                // （首行 = columns - 原 subsequentSpace，后续行 = 上述 wrap_columns）。
                let first_wrapped = wrap_text(&text, first_line_columns);
                let mut it = first_wrapped.split('\n');
                let first = it.next().unwrap_or("").to_string();
                let rest = it.collect::<Vec<_>>().join(" ");
                let rest_wrapped = wrap_text(&rest, wrap_columns);
                text = format!("{first}\n{rest_wrapped}");
            } else {
                text = wrap_text(&text, wrap_columns);
            }
            for (orig, tmp) in vb_shim {
                text = text.replacen(&tmp, &orig, 1);
            }
        }

        // Perl Wrap.pm L578-585：`when: after` 折完把 subsequentSpace 回填到每行行首，
        // 再把块首行自身的缩进去掉（首行的缩进已由 blocksFollow 段提供）。
        if when_after {
            text = replace_all_with(&text, &re(r"(?m)^"), |_| subsequent_space.clone());
            text = replace_all_with(&text, &re(r"(?s)^\h*"), |_| String::new());
        }

        // `comments.wrap`：把本块抽出的尾注释先合成一段、按列宽折行，
        // 再重新登记成**两行** token（首行一个，其余用空格拼成第二个）。
        // 随后 `text_wrap_comment_blocks` 会把这段 token 块再折一次。
        // Perl Wrap.pm L591-627。
        if comments_wrap && !trailing_comments.is_empty() {
            let mut comment_value = String::new();
            let id_re = re(&format!(r"([0-9]+){}", crate::patterns::esc_lit(&t.end_of_token)));
            for c in id_re.captures_iter(trailing_comments.as_bytes()).flatten() {
                let n: usize = gn(&c, 1).parse().unwrap_or(0);
                if n == 0 {
                    continue;
                }
                let v = state::storage(|st| {
                    st.trailing_comments
                        .get(n - 1)
                        .map(|x| x.value.clone())
                        .unwrap_or_default()
                });
                comment_value.push_str(&v);
            }
            let lead: String = comment_value
                .chars()
                .take_while(|c| *c == ' ' || *c == '\t')
                .collect();
            let leading_comment_space = if lead.is_empty() {
                " ".to_string()
            } else {
                lead
            };
            // 列宽扣掉块尾最后一行的宽度（Perl 用 get_column_width）
            let last_w = width(text.split('\n').next_back().unwrap_or(""));
            let ccols = (columns_i - last_w as i64).max(0) as usize;
            let wrapped = wrap_text(&comment_value, ccols);
            let mut it = wrapped.split('\n');
            let first = it.next().unwrap_or("").to_string();
            let mut rest = it.collect::<Vec<_>>().join(" ");
            if multi {
                let r = re(r"\h{2,}");
                rest = replace_all_with(&rest, &r, |_| " ".to_string());
            }
            let id1 = crate::trailing_comments::add_comment_symbol(&first);
            let id2 = crate::trailing_comments::add_comment_symbol(&format!(
                "{leading_comment_space}{rest}"
            ));
            trailing_comments = format!("%{id1}\n%{id2}");
        }

        if !trailing_comments.is_empty() {
            let r = re(r"(?s)\R?$");
            text = replace_all_with(&text, &r, |_| format!("{trailing_comments}\n"));
        }
        if !tail.is_empty() {
            // Perl quirk：块尾以换行起头时，去掉结尾注释与其之间的换行
            if !trailing_comments.is_empty()
                && re(r"(?s)^\h*\R").is_match(tail.as_bytes()).unwrap_or(false)
            {
                let ending = format!("{trailing_comments}\n");
                if text.ends_with(&ending) {
                    text.truncate(text.len() - 1);
                }
            }
            text.push_str(&tail);
        }
        // 空行 token 归位
        let r = re(&format!(r"(?m)^\H.*?{}", t.blanklines));
        text = replace_all_with(&text, &r, |c| {
            format!(
                "{}\n{}",
                gn(c, 0).trim_end_matches(&t.blanklines),
                t.blanklines
            )
        });
        let r = re(&format!(r"(?m)^\h*{}\h*", t.blanklines));
        text = replace_all_with(&text, &r, |_| t.blanklines.clone());
        push_piece(&mut out, text);
    }
    out
}

fn other_regex(m: &serde_yaml::Mapping) -> Option<String> {
    match crate::settings::get(m, "other") {
        Some(serde_yaml::Value::String(s)) if s != "0" && !s.is_empty() => Some(s.clone()),
        _ => None,
    }
}

/// Perl 无 /g 的 `s///`：只删除首个匹配。
fn remove_first_match(text: &str, re: &Regex) -> String {
    match re.find(text.as_bytes()) {
        Ok(Some(m)) => {
            let mut s = text.to_string();
            s.replace_range(m.start()..m.end(), "");
            s
        }
        _ => text.to_string(),
    }
}

/// 该行（trim 后）是否整体是一条尾注释。
fn tc_is_comment(line: &str) -> bool {
    let tc = state::patterns(|p| p.trailing_comment.clone());
    re(&format!(r"(?s)^\h*{}\h*$", tc))
        .is_match(line.as_bytes())
        .unwrap_or(false)
}

fn is_hspace_or_empty(s: &str) -> bool {
    s.chars().all(|c| c == ' ' || c == '\t')
}

/// Perl `previouslyFoundSetting{textWrapAfter}{name}`：`when: after` 时按对象名
/// 缓存的自定义缩进（只对纯横向空白的 indentRules 记录）。
fn text_wrap_after_indent(name: &str) -> Option<String> {
    state::settings(|s| {
        if crate::settings::as_str(crate::settings::get_path(
            &s.root,
            &["modifyLineBreaks", "textWrapOptions", "when"],
        )) != Some("after")
        {
            return None;
        }
        s.previously_found
            .get(&format!("textWrapAfter{name}"))
            .map(|o| o.indentation.clone())
    })
}

/// text_wrap_comment_blocks：注释块折行（comments.wrap）。
pub fn text_wrap_comment_blocks(body: &str) -> String {
    let t = tok(|t| t.clone());
    let tc = state::patterns(|p| p.trailing_comment.clone());
    let columns = tw(|m| as_i64(get_opt(m, "columns")).unwrap_or(0));
    if columns == 0 {
        logger_warn_columns();
        return body.to_string();
    }
    let inherit = tw(|m| {
        as_bool(get_opt(m, "comments").and_then(|v| {
            v.as_mapping()
                .and_then(|mm| crate::settings::get(mm, "inheritLeadingSpace"))
        }))
    });
    let multi = tw(|m| as_bool(get_opt(m, "multipleSpacesToSingle")));
    // 拆开连体注释：`% fifth% fourth` → 两行（Perl Wrap.pm L695）
    let mut body = replace_all_with(body, &re(&format!(r"(?m)({tc})\h*({tc})")), |c| {
        format!("{}\n{}", gn(c, 1), gn(c, 2))
    });
    // 注释块折行（Perl Wrap.pm L712-760）
    //
    // 关键：此时注释正文已被替换成 token，**块里的可见文本是 token 而非内容**。
    // Perl 是按 token 里的数字 id 回到 `@trailingComments` 取 `value` 拼成一整段，
    // 折行后再把每一行**重新登记成新 token**。旧实现直接拿 token 文本当内容，
    // 结果写出 `% tc1-END`（issue-562-mod1）。
    let id_re = re(&format!(r"([0-9]+){}", crate::patterns::esc_lit(&t.end_of_token)));
    let block_re = re(&format!(r"(?m)((?:^\h*{tc}\R)+)"));
    body = replace_all_with(&body, &block_re, |caps| {
        let block = gn(caps, 1);
        let hs_raw: String = block
            .chars()
            .take_while(|c| *c == ' ' || *c == '\t')
            .collect();
        let n_tabs = hs_raw.matches('\t').count();
        let mut leading_hs = hs_raw.replace('\t', "    ");

        // 按 token 顺序取回注释正文
        let mut value = String::new();
        for c in id_re.captures_iter(block.as_bytes()).flatten() {
            let n: usize = gn(&c, 1).parse().unwrap_or(0);
            if n == 0 {
                continue;
            }
            let v = state::storage(|s| {
                s.trailing_comments
                    .get(n - 1)
                    .map(|c| c.value.clone())
                    .unwrap_or_default()
            });
            value.push(' ');
            value.push_str(&v);
        }
        // 去掉最前面一个空白，再取其余前导空白
        let value = value.strip_prefix(|c: char| c == ' ' || c == '\t').unwrap_or(&value);
        let (lead_ws, rest) = split_leading_hs(value);
        let leading_space = if inherit {
            lead_ws.to_string()
        } else {
            " ".to_string()
        };
        let mut value = rest.to_string();
        if multi {
            let r = re(r"\h{2,}");
            value = replace_all_with(&value, &r, |_| " ".to_string());
        }
        // 列宽要扣掉行首缩进与 `%` 后那个空格（Perl 用 length，不是字素宽）
        let wrap_cols = columns - leading_hs.chars().count() as i64 - leading_space.chars().count() as i64;
        let wrapped = wrap_text(&value, wrap_cols.max(0) as usize);
        // 还原制表符（Perl 只把前 $numberOfTABS 组 4 空格换回 tab）
        for _ in 0..n_tabs {
            leading_hs = leading_hs.replacen("    ", "\t", 1);
        }
        let mut out = String::new();
        for line in wrapped.split('\n') {
            let id = crate::trailing_comments::add_comment_symbol(&format!("{leading_space}{line}"));
            out.push_str(&leading_hs);
            out.push('%');
            out.push_str(&id);
            out.push('\n');
        }
        out
    });
    body
}

fn split_leading_hs(s: &str) -> (&str, &str) {
    let n = s
        .find(|c: char| c != ' ' && c != '\t')
        .unwrap_or(s.len());
    s.split_at(n)
}

fn logger_warn_columns() {
    crate::logger::warn("*textWrapOptions/comments/wrap: no value for columns");
}
