//! `-r` 替换里 Perl `/e`（替换串是 Perl 代码）的原生等价实现。
//!
//! 上游把 `substitution` 直接交给 Perl：`eval("\$body =~ $substitution")`
//! （`Replacement.pm:82-84`）。Rust 侧没有 Perl 解释器，因此这里为**已知的**
//! eval 代码块提供逐字节等价的原生实现（方案 B：模板 handler）。
//!
//! 识别方式是「结构指纹」而不是整段哈希：只检查若干稳定的特征串，这样上游
//! 调整缩进/注释/变量名之外的写法时仍能命中。未命中的片段由调用方警告并保持
//! 原文不变——与上游「eval 失败则静默不改」的行为一致，但保留可诊断性。
//!
//! 每个 handler 都严格复刻 Perl 语义，尤其是：
//! - `split("\n", $s)` 丢弃尾部空字段；
//! - `" " x 负数` 得到空串（Rust 用 `saturating_sub`）；
//! - `foreach (@lines)` 的 `$_` 是元素别名（这里直接改写 `Vec<String>`）；
//! - `length` 计字符数（码点），而 `get_column_width` / `\X` 计字素簇。

use crate::patterns::{gn, gs, re_new, replace_all_with};
use pcre2::bytes::{Captures, Regex};

/// 一次替换的回调：由捕获组算出替换文本（对应 Perl `/e` 的求值结果）。
pub type EvalFn = Box<dyn Fn(&Captures) -> String>;

/// 识别 `substitution` 中的 Perl 代码，返回原生等价实现。
pub fn eval_replacement(code: &str) -> Option<EvalFn> {
    if code.contains("$rulerMax") && code.contains("----|") {
        return ruler_handler(code);
    }
    if code.contains("cmhlistingsfromfile") && code.contains(r"\R@@sg") {
        return cmh_handler();
    }
    if code.contains("Insert comment describing function here") {
        return replace26_handler();
    }
    if code.contains("$leadingSpace") && code.contains("s|^$leadingSpace||mg") {
        return issue513_handler();
    }
    if code.contains("$preamble") && code.contains(r"\norm{") {
        return issue517a_handler();
    }
    if code.contains("comma space unification") {
        return issue524_handler();
    }
    if code.contains("$beforeJustLengths") {
        return te_752552a_handler();
    }
    if code.contains("$maxLengthBeforeNode") {
        // issue-382 用模块函数量宽（受 --GCString 影响），issue-382a 换成 `/\X/g` 计数。
        let gcstring = code.contains("get_column_width");
        return issue382_handler(gcstring);
    }
    None
}

// ---------------------------------------------------------------------------
// 小工具：Perl 语义
// ---------------------------------------------------------------------------

/// Perl 真值：空串与 "0" 为假，其余为真；假值按 `q()` 取空串。
fn perl_truthy(s: &str) -> &str {
    if s.is_empty() || s == "0" {
        ""
    } else {
        s
    }
}

/// Perl `split("\n", $s)`：丢弃尾部空字段。
fn split_perl_lines(s: &str) -> Vec<String> {
    let mut v: Vec<String> = s.split('\n').map(str::to_string).collect();
    while v.last().is_some_and(|last| last.is_empty()) {
        v.pop();
    }
    v
}

/// Perl `length`：字符（码点）数，不是字节数，也不是字素簇数。
fn char_len(s: &str) -> usize {
    s.chars().count()
}

/// PCRE2 `\X`：字素簇数。
fn grapheme_len(s: &str) -> usize {
    use unicode_segmentation::UnicodeSegmentation;
    s.graphemes(true).count()
}

/// Perl `\h`（水平空白）。
fn is_h(c: char) -> bool {
    matches!(
        c,
        ' ' | '\t' | '\u{a0}' | '\u{1680}' | '\u{2000}'
            ..='\u{200a}' | '\u{202f}' | '\u{205f}' | '\u{3000}'
    )
}

fn del_all(re: &Regex, s: &str) -> String {
    replace_all_with(s, re, |_| String::new())
}

// ---------------------------------------------------------------------------
// 1. ruler 生成器（addruler1..3 / issue-359* / issue-444* / issue-450* /
//    text-wrap-4 / text-wrap-6）
// ---------------------------------------------------------------------------

fn ruler_handler(code: &str) -> Option<EvalFn> {
    let re = re_new(r"int\(\s*(\d+)\s*/\s*10\s*\+\s*1\.5\s*\)\s*\*\s*10").ok()?;
    let caps = re.captures(code.as_bytes()).ok()??;
    let n: u32 = gn(&caps, 1).parse().ok()?;
    // int($n/10 + 1.5) * 10，Perl 的 int 向零截断。
    let ruler_max = ((n as f64 / 10.0 + 1.5).trunc() * 10.0) as u32;
    // 末尾是 `$ruler."\n"` 而不是裸 `$ruler` 时，多输出一个换行。
    let trailing_newline = code.contains(r#"$ruler."\n""#);
    let text = ruler_text(ruler_max, trailing_newline);
    Some(Box::new(move |_caps: &Captures| text.clone()))
}

fn ruler_text(ruler_max: u32, trailing_newline: bool) -> String {
    let n = (ruler_max / 5) as usize;
    let mut out = "----|".repeat(n);
    out.push('\n');
    // $ruler .= "   ".$i*5  →  `.` 优先级低于 `*`，即 "   ".( $i*5 )
    for i in 1..=n {
        out.push_str("   ");
        out.push_str(&(i * 5).to_string());
    }
    if trailing_newline {
        out.push('\n');
    }
    out
}

// ---------------------------------------------------------------------------
// 2. cmhlistings 展平（manual/latexindent.yaml、batch-tests/documentation.yaml）
// ---------------------------------------------------------------------------

fn cmh_handler() -> Option<EvalFn> {
    let re_linebreak = re_new(r"\R").ok()?;
    Some(Box::new(move |caps: &Captures| {
        let body = gn(caps, 1);
        let flat = del_all(&re_linebreak, body);
        format!("{flat}\n")
    }))
}

// ---------------------------------------------------------------------------
// 3. 数学公式规范化
// ---------------------------------------------------------------------------

/// replace26：`$$...$$` → `%` 注释块 + 公式体。
fn replace26_handler() -> Option<EvalFn> {
    let re_sup = re_new(r"(?s)(\^|_)([a-zA-Z0-9]+)").ok()?;
    let re_ops = re_new(r"(?s)\h*([+\-=])\h*").ok()?;
    Some(Box::new(move |caps: &Captures| {
        let body = gn(caps, 1);
        let body = replace_all_with(body, &re_sup, |c| format!("{}{{{}}}", gn(c, 1), gn(c, 2)));
        let body = replace_all_with(&body, &re_ops, |c| format!(" {} ", gn(c, 1)));
        format!("%%\n%% Insert comment describing function here\n%%\n$ {body} $")
    }))
}

/// issue-513：把多行 `%` 注释块首行的缩进去掉。
fn issue513_handler() -> Option<EvalFn> {
    let re_lead = re_new(r"^\h*").ok()?;
    Some(Box::new(move |caps: &Captures| {
        let begin = gn(caps, 1);
        let middle = gn(caps, 2);
        let end = gn(caps, 3);
        // $middle =~ s|^(\h*)||s; my $leadingSpace = $1
        let lead_end = re_lead
            .find(middle.as_bytes())
            .ok()
            .flatten()
            .map_or(0, |m| m.end());
        let leading = &middle[..lead_end];
        let mut stripped = middle[lead_end..].to_string();
        // $middle =~ s|^$leadingSpace||mg
        if !leading.is_empty() {
            stripped = stripped
                .split_inclusive('\n')
                .map(|line| line.strip_prefix(leading).unwrap_or(line))
                .collect();
        }
        format!("{begin}{stripped}{end}")
    }))
}

/// issue-517a：只在 body 部分把 `\Vert ... \rVert` 换成 `\norm{...}`。
fn issue517a_handler() -> Option<EvalFn> {
    let re = re_new(r"(?sx)(?<!\\) \\l?Vert\s*(?P<content>.+?)\s*(?<!\\) \\r?Vert").ok()?;
    Some(Box::new(move |caps: &Captures| {
        let preamble = gn(caps, 1);
        let body = gn(caps, 2);
        let body = replace_all_with(body, &re, |c| {
            format!("\\norm{{{}}}", gs(c, "content").unwrap_or(""))
        });
        format!("{preamble}{body}")
    }))
}

/// issue-524：公式体里的空格/括号/运算符规范化。
fn issue524_handler() -> Option<EvalFn> {
    let re_begin = re_new(r"^(\\\(|\$|\\\[|\\begin\{[^}]+?\})").ok()?;
    let re_end = re_new(r"(\\\)|\$|\\\]|\\end\{[^}]+?\})$").ok()?;
    let re_comma = re_new(r"(?s)([a-zA-Z0-9])\h*,\h*([a-zA-Z0-9])").ok()?;
    let re_lparen = re_new(r"(?s)\(\h*([a-zA-Z0-9])").ok()?;
    let re_rparen = re_new(r"(?s)([a-zA-Z0-9])\h*\)").ok()?;
    let re_ops = re_new(r"(?s)([a-zA-Z0-9])\h*(=|\*|\+|\-)\h*([a-zA-Z0-9])").ok()?;
    let re_times = re_new(r"(?s)([a-zA-Z0-9])\h*(\\times)\h*([a-zA-Z0-9])").ok()?;
    Some(Box::new(move |caps: &Captures| {
        let mut middle = gn(caps, 1).to_string();
        let (begin, cut) = match re_begin.captures(middle.as_bytes()) {
            Ok(Some(c)) => (gn(&c, 1).to_string(), c.get(0).map_or(0, |m| m.end())),
            _ => (String::new(), 0),
        };
        middle = middle[cut..].to_string();
        let (end, keep) = match re_end.captures(middle.as_bytes()) {
            Ok(Some(c)) => (
                gn(&c, 1).to_string(),
                c.get(0).map_or(middle.len(), |m| m.start()),
            ),
            _ => (String::new(), middle.len()),
        };
        middle.truncate(keep);
        let mid = replace_all_with(&middle, &re_comma, |c| {
            format!("{}, {}", gn(c, 1), gn(c, 2))
        });
        let mid = replace_all_with(&mid, &re_lparen, |c| format!("({}", gn(c, 1)));
        let mid = replace_all_with(&mid, &re_rparen, |c| format!("{})", gn(c, 1)));
        let mid = replace_all_with(&mid, &re_ops, |c| {
            format!("{} {} {}", gn(c, 1), gn(c, 2), gn(c, 3))
        });
        let mid = replace_all_with(&mid, &re_times, |c| {
            format!("{} {} {}", gn(c, 1), gn(c, 2), gn(c, 3))
        });
        format!("{begin}{mid}{end}")
    }))
}

// ---------------------------------------------------------------------------
// 4. 对齐填充
// ---------------------------------------------------------------------------

/// te-752552a：把 prooftree 里 `just` 之前的内容按最大宽度右侧补空格对齐。
fn te_752552a_handler() -> Option<EvalFn> {
    Some(Box::new(move |caps: &Captures| {
        let begin = gn(caps, 1);
        let end = gn(caps, 3);
        let mut lines = split_perl_lines(gn(caps, 2));

        // loop 1：测量 `just` 之前（去掉 "just..." 后）的最大长度
        let mut max_len = 0usize;
        for line in &lines {
            if let Some(idx) = line.find("just") {
                max_len = max_len.max(char_len(&line[..idx]));
            }
        }

        // loop 2：把 `just` 之前的水平空白去掉，再按差值补空格
        for line in &mut lines {
            let Some(idx) = line.find("just") else {
                continue;
            };
            let mut start = idx;
            while start > 0 {
                let prev = line[..start].chars().next_back().unwrap_or('\0');
                if is_h(prev) {
                    start -= prev.len_utf8();
                } else {
                    break;
                }
            }
            let prefix = line[..start].to_string();
            let rest = line[idx..].to_string();
            let pad = max_len.saturating_sub(char_len(&prefix));
            *line = format!("{prefix}{}{rest}", " ".repeat(pad));
        }

        format!("{begin}{}\n{end}", lines.join("\n"))
    }))
}

/// issue-382 / issue-382a：`\draw ...;` 内按 `node` 与行末 `(a)` 对齐。
fn issue382_handler(gcstring: bool) -> Option<EvalFn> {
    let re_row_tail = re_new(r"(?s)\h*\([a-z]+\)\h*$").ok()?;
    let re_node_head = re_new(r"^(\h*)(.*?)node").ok()?;
    let re_piece = re_new(r"(?s)\h*(\([a-z]+\))$").ok()?;
    Some(Box::new(move |caps: &Captures| {
        let body = gn(caps, 1);
        let end = gn(caps, 2);
        let mut lines = split_perl_lines(body);
        let measure = |s: &str| {
            if gcstring {
                crate::alignment::width(s)
            } else {
                grapheme_len(s)
            }
        };

        // step 1：量出最大行宽与 `node` 之前的最大宽度
        let mut max_row = 0usize;
        let mut max_before = 0usize;
        for line in &lines {
            let row_for_measure = del_all(&re_row_tail, line);
            max_row = max_row.max(measure(&row_for_measure));
            if !line.contains("node") {
                continue;
            }
            if let Ok(Some(c)) = re_node_head.captures(line.as_bytes()) {
                let before = format!("{}{}", perl_truthy(gn(&c, 1)), perl_truthy(gn(&c, 2)));
                max_before = max_before.max(measure(&before));
            }
        }

        // step 2：按最大值补齐；`(a)` 等尾部先摘下，最后再挂回去
        for line in &mut lines {
            let original = line.clone();
            let mut cur = original.clone();
            let mut final_piece = String::new();
            let piece = match re_piece.captures(cur.as_bytes()) {
                Ok(Some(c)) => Some((
                    c.get(0).map_or(cur.len(), |m| m.start()),
                    gn(&c, 1).to_string(),
                )),
                _ => None,
            };
            if let Some((start, piece)) = piece {
                cur.truncate(start);
                final_piece = piece;
            }

            if !original.contains("node") {
                let pad = (max_row + 1).saturating_sub(measure(&cur));
                *line = format!("{cur}{}{final_piece}", " ".repeat(pad));
                continue;
            }

            let (leading, before, node_end) = match re_node_head.captures(cur.as_bytes()) {
                Ok(Some(c)) => (
                    perl_truthy(gn(&c, 1)).to_string(),
                    perl_truthy(gn(&c, 2)).to_string(),
                    c.get(0).map_or(0, |m| m.end()),
                ),
                _ => (String::new(), String::new(), 0),
            };
            let rebuilt = format!("node{}", &cur[node_end..]);
            let pad = max_before.saturating_sub(measure(&format!("{leading}{before}")));
            cur = format!("{leading}{before}{}{rebuilt}", " ".repeat(pad));
            let pad = (max_row + 1).saturating_sub(measure(&cur));
            *line = format!("{cur}{}{final_piece}", " ".repeat(pad));
        }

        format!("{}{end}", lines.join("\n"))
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ruler_text_matches_perl_output() {
        // int(25/10 + 1.5) * 10 = 40 → 8 段 "----|"，再逐行列出 5 的倍数
        let got = ruler_text(40, false);
        assert_eq!(
            got,
            "----|----|----|----|----|----|----|----|\n   5   10   15   20   25   30   35   40"
        );
        assert!(ruler_text(40, true).ends_with("\n   5   10   15   20   25   30   35   40\n"));
    }

    #[test]
    fn ruler_max_rounding_follows_perl_int() {
        // 各夹具里的常量：int(N/10 + 1.5) * 10，Perl 的 int 向零截断
        for (n, want) in [(25, 40), (45, 60), (70, 80), (75, 90), (80, 90), (100, 110)] {
            let got = ((n as f64 / 10.0 + 1.5).trunc() * 10.0) as u32;
            assert_eq!(got, want, "N={n}");
        }
    }

    #[test]
    fn recognises_each_supported_family() {
        let cases = [
            ("my $rulerMax = int(25/10 +1.5 )*10; \"----|\"", "ruler"),
            ("cmhlistingsfromfile $body =~ s@\\R@@sg", "cmh"),
            ("Insert comment describing function here", "replace26"),
            ("$leadingSpace = 1; s|^$leadingSpace||mg", "issue-513"),
            ("$preamble . \\norm{$+{content}}", "issue-517a"),
            ("comma space unification", "issue-524"),
            ("$beforeJustLengths", "te-752552a"),
            ("$maxLengthBeforeNode; get_column_width(x)", "issue-382"),
            (
                "$maxLengthBeforeNode; while ($s =~ /\\X/g) {}",
                "issue-382a",
            ),
        ];
        for (code, name) in cases {
            assert!(
                eval_replacement(code).is_some(),
                "{name} should be recognised"
            );
        }
    }

    #[test]
    fn unrecognised_snippet_returns_none() {
        assert!(eval_replacement("my $x = $1; uc($x);").is_none());
        assert!(eval_replacement("").is_none());
    }

    #[test]
    fn perl_split_drops_trailing_empty_fields() {
        assert_eq!(split_perl_lines("a\nb\n"), vec!["a", "b"]);
        assert_eq!(split_perl_lines("a\n\nb\n\n"), vec!["a", "", "b"]);
        assert!(split_perl_lines("").is_empty());
    }
}
