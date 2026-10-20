//! 正则拼装（对应 LatexIndent/Blocks.pm 的 _construct_code_blocks_regex 与
//! _construct_args_with_between）。
//!
//! PCRE2 与 Perl 的差异及补偿：
//! - 递归正则 `(??{...})` → `(?(DEFINE)...)` 定义块 + `(?&dXXX)` 子程序调用；
//! - 调用内捕获返回后丢弃 → 需要读取的捕获全部内联在主流正则中；
//! - 不允许重复命名组 → 同一子模式的多份内联拷贝用后缀（_env/_if/_sp/_spn）消歧。

use pcre2::bytes::{Captures, Match, Regex, RegexBuilder};
use serde_yaml::Value;

use crate::logger;
use crate::settings::{as_bool, as_str, get, get_path};
use crate::tokens::tok;

/// 统一正则编译入口：UCP+UTF（对齐 Perl 字符语义，匹配边界必落在字符边界，
/// 避免字节模式下 `.`/类匹配半个 UTF-8 字符导致切片 panic）。
/// Perl 与 PCRE2 对「未定义转义」的处理不同：Perl 视 `\<`、`\>` 为**字面字符**，
/// PCRE2 则把它们解释成词首 / 词尾断言（`\b` 的单侧变体）。
///
/// 用户的 yaml（以及 latexindent 自带默认值）都是按 Perl 语法书写的，因此凡是要拼进
/// pcre2 的用户正则片段都应先过一遍这里，否则 `begin: \<\<` 这类写法会静默失效
/// （issue241-mod2：`<<...>>` 的块边界错乱，连带外层命令的参数整段丢缩进）。
pub(crate) fn perl_literal_escapes(src: &str) -> String {
    let mut out = String::with_capacity(src.len());
    let mut it = src.chars().peekable();
    while let Some(c) = it.next() {
        if c == '\\' {
            match it.peek().copied() {
                Some(n) if n == '<' || n == '>' => {
                    out.push(n);
                    it.next();
                    continue;
                }
                Some(n) => {
                    out.push('\\');
                    out.push(n);
                    it.next();
                    continue;
                }
                None => {
                    out.push('\\');
                    continue;
                }
            }
        }
        out.push(c);
    }
    out
}

/// 把「精确版」正则源码里的参数体循环换成「所有格版」。
///
/// 精确版（`(?:(?:head)|(?:arg_body)|(?&dCODE))*`）照抄 Perl：循环可回溯，
/// 嵌套块（`\if...\fi` / special / 环境）绝不会越过参数的闭括号。
/// 但病态输入会让它指数回溯（`broken.tex` 直接 match limit）。
/// 兜底版把顺序换成 `dCODE` 优先并加所有格，永不回溯、绝不 panic。
fn fast_variant(src: &str, loop_pairs: &[(String, String)]) -> String {
    let mut out = src.to_string();
    for (accurate, fast) in loop_pairs {
        out = out.replace(accurate.as_str(), fast.as_str());
    }
    out
}

/// `replace_all_with` 的可失败版本：正则超限时返回 Err 而不是 panic，
/// 供「精确版失败 → 所有格版重跑」的回退路径使用。
pub fn try_replace_all_with(
    body: &str,
    re: &Regex,
    mut f: impl FnMut(&Captures) -> String,
) -> Result<String, pcre2::Error> {
    let bytes = body.as_bytes();
    let mut out = String::new();
    let mut last = 0usize;
    for cap in re.captures_iter(bytes) {
        let cap = cap?;
        let m = cap.get(0).unwrap();
        out.push_str(&body[last..m.start()]);
        out.push_str(&f(&cap));
        last = m.end();
    }
    out.push_str(&body[last..]);
    Ok(out)
}

pub fn re_new(src: &str) -> Result<Regex, pcre2::Error> {
    // `(?J)`：允许重名捕获组（等价于 Perl 的默认行为）。用户自定义
    // fineTuning.environments.begin/end 常自带命名组（如 issue-619 的 `(?<ENVNAME>`），
    // 同一段片段会被同时内联到顶层 pattern 与 DEFINE 块里 → 不开启会因
    // PCRE2_DUPNAMES not set 编译失败。
    let mut src = src.to_string();
    if !src.starts_with("(?J)") {
        src.insert_str(0, "(?J)");
    }
    RegexBuilder::new().ucp(true).build(&src)
}

pub struct PatternSet {
    pub all_code_blocks: Regex,
    /// `all_code_blocks` 的「所有格」变体：参数体循环用 `dCODE` 优先 + `*+`。
    /// 仅在精确版撞 pcre2 match limit（如 broken.tex 的病态嵌套）时整段回退使用。
    pub all_code_blocks_fast: Regex,
    pub code_block_look_for: Regex,
    pub one_arg: Regex,
    /// `one_arg` 的所有格变体，用途同 `all_code_blocks_fast`。
    pub one_arg_fast: Regex,
    pub one_arg_names: std::collections::HashMap<String, usize>,
    pub trailing_comment: String,
    pub item_regexp: String,
    pub special_end: Vec<(String, String)>, // (begin, end) 参与 specialBody 截止判断
    pub re_arg_only_comment: Regex,
    pub re_arg_only_space: Regex,
    pub re_body_linebreak: Regex,
    pub re_key_split: Regex,
    pub re_item_no_space: Regex,
    pub re_item: Regex,
    pub re_middle: Option<(String, Regex)>, // specialLookUpMiddle 逐名构建，另行处理
    pub defs: String,
    /// Perl $specialAllRegEx：完整 special 块（SPECIAL | SPECIALNESTED），
    /// 供 _find_special_middle 的 SPECIALALL 分支整块吞掉已渲染的嵌套 special。
    pub special_all: String,
    /// 是否存在参与识别的 specialBeginEnd（用于 `-t` 的构建期 trace）。
    pub special_present: bool,
}

fn frag(v: Option<&Value>, default: &str) -> String {
    as_str(v).unwrap_or(default).to_string()
}

/// Perl Regexp 字符串化 quirk：模式若以 x-mode 收尾，拼接时在其后补 "\n"，
/// 否则片段行尾的 `#` 注释会吞掉外层闭括号（如 issue-308 的
/// fineTuning:arguments:between 带 `#` 注释行）。
pub fn x_guard(src: &str) -> String {
    let mut x = false;
    let mut rest = src;
    while let Some(p) = rest.find("(?") {
        rest = &rest[p + 2..];
        let flags: String = rest
            .chars()
            .take_while(|c| c.is_ascii_alphabetic() || *c == '-')
            .collect();
        if !flags.is_empty()
            && rest[flags.len()..]
                .chars()
                .next()
                .map(|c| c == ')' || c == ':')
                .unwrap_or(false)
        {
            let neg = flags.starts_with('-');
            if flags.contains('x') {
                x = !neg;
            }
        }
    }
    if x {
        format!("{src}\n")
    } else {
        src.to_string()
    }
}

pub(crate) fn esc_lit(s: &str) -> String {
    let mut out = String::new();
    for c in s.chars() {
        if r"\.^$|?*+()[]{}".contains(c) {
            out.push('\\');
        }
        out.push(c);
    }
    out
}

/// 逻辑组名 → 变体后缀列表（查询捕获时逐个尝试）。
const SUFFIXES: &[&str] = &["", "_env", "_if", "_sp", "_spn", "_mid"];

pub fn g<'t>(caps: &'t Captures, logical: &str) -> Option<Match<'t>> {
    for suf in SUFFIXES {
        let name = if suf.is_empty() {
            logical.to_string()
        } else {
            format!("{logical}{suf}")
        };
        if let Some(m) = caps.name(&name) {
            return Some(m);
        }
    }
    None
}

pub fn gs<'t>(caps: &'t Captures, logical: &str) -> Option<&'t str> {
    g(caps, logical).and_then(|m| std::str::from_utf8(m.as_bytes()).ok())
}

/// 按组序号取捕获文本（无名组用）。
pub fn gn<'t>(caps: &'t Captures, i: usize) -> &'t str {
    caps.get(i)
        .and_then(|m| std::str::from_utf8(m.as_bytes()).ok())
        .unwrap_or("")
}

/// s///eg 的等价物：对每个非重叠匹配应用 f 生成替换文本。
pub fn replace_all_with(body: &str, re: &Regex, mut f: impl FnMut(&Captures) -> String) -> String {
    let bytes = body.as_bytes();
    let mut out = String::new();
    let mut last = 0usize;
    for cap in re.captures_iter(bytes) {
        let cap = cap.expect("regex match failed");
        let m = cap.get(0).unwrap();
        out.push_str(&body[last..m.start()]);
        out.push_str(&f(&cap));
        last = m.end();
    }
    out.push_str(&body[last..]);
    out
}

/// 从设置提取各 fineTuning 片段并拼装全部正则（is_m 对应 $mSwitchOnlyTrailing）。
pub fn build(root: &serde_yaml::Mapping, is_m: bool) -> PatternSet {
    let t = tok(crate::tokens::Tokens::clone);
    let env_name = frag(
        get_path(root, &["fineTuning", "environments", "name"]),
        r"[a-zA-Z@\*0-9_\\]+",
    );
    let cmd_name = frag(
        get_path(root, &["fineTuning", "commands", "name"]),
        r"[+a-zA-Z@*0-9_:]+",
    );
    let named_name = frag(
        get_path(root, &["fineTuning", "namedGroupingBracesBrackets", "name"]),
        r"[0-9\.a-zA-Z@\*><]+",
    );
    let key_name = frag(
        get_path(
            root,
            &["fineTuning", "keyEqualsValuesBracesBrackets", "name"],
        ),
        r"[a-zA-Z@\*0-9_\/.:\#-]+[a-zA-Z@\*0-9_\/.\h:\#-]*",
    );
    // 用户提供的片段若以 x-mode 收尾（如 issue-378 的 `#` 注释结尾），
    // 拼接时必须补 "\n"，否则行尾 `#` 会吞掉外层闭括号 → 编译失败。
    let key_x = x_guard(&key_name);
    let if_name = frag(
        get_path(root, &["fineTuning", "ifElseFi", "name"]),
        r"(?!@?if[a-zA-Z@]*?\{)@?if[a-zA-Z@]*",
    );
    let between = frag(
        get_path(root, &["fineTuning", "arguments", "between"]),
        r"(?x)(?:[_^*#0-9(),+-]|(?:node|at|to|decoration)|(?:\([^)]*\)))",
    );
    let not_preceded_by = frag(
        get_path(root, &["fineTuning", "trailingComments", "notPrecededBy"]),
        r"(?<!\\)",
    );
    let trailing_comment = format!(
        "{}%{}\\d+{}",
        not_preceded_by, t.trailing_comment, t.end_of_token
    );
    let item_regexp = frag(
        get_path(root, &["fineTuning", "items", "itemRegEx"]),
        r"\\(?:item|myitem)",
    );
    let trailing_comment_x = format!("(?-xs:{trailing_comment})");
    let between_x = format!("(?-xs:{})", x_guard(&between));
    let blank_lit = esc_lit(&t.blanklines);

    // 可选 fineTuning.environments.begin/end 覆盖
    let env_custom = get_path(root, &["fineTuning", "environments"]);
    // 这两段会原样内联进 `/x` 模式的正则。若覆写值以 `#` 注释收尾（issue-556 的
    // `fineTuning.environments.begin` 就是），拼接在其后的 `)` 会被注释吞掉 →
    // `missing closing parenthesis`。统一过 `x_guard`（在含 `(?x)` 时补一个换行）收口。
    let env_begin_custom = env_custom
        .and_then(|m| m.as_mapping())
        .and_then(|m| get(m, "begin"))
        .and_then(|v| v.as_str())
        .map(x_guard);
    let env_end_custom = env_custom
        .and_then(|m| m.as_mapping())
        .and_then(|m| get(m, "end"))
        .and_then(|v| v.as_str())
        .map(x_guard);

    // ---- 片段（内部递归用定义块，捕获不可读，前缀 d） ----
    let arg_body = format!(
        r"(?>\\(?:begin|end)\{{{env_name}\}}|(?:\\\\\h*\[[^]]*\])|(?:\\[{{}}\[\]])|[^{{}}\[\]])"
    );
    // 花括号参数专用：允许正文里出现**落单的 `]`**。
    //
    // Perl 的 argumentBodyRegEx 把 `]` 也排除在外（因为方括号参数要靠 `\]` 收尾），
    // 但 `{...}` 里的 `]` 永远不会终止参数，排除它只会让「`[...]` 已被 dCODE 吃掉、
    // 只剩一个孤儿 `]`」的正文彻底失配。而正文循环对所有格（`*+`），一旦失配无法回溯，
    // 结果就是整条命令的后续参数全丢（issue241-mod2：`\ifnumequal` 第 3/4 个参数整段
    // 不缩进，`<< ... >>` 里的内容也跟着掉一级）。
    // 方括号参数仍用严格版 arg_body：它必须让 `]` 留给 OPTARGEND。
    let arg_body_curly = format!(
        r"(?>\\(?:begin|end)\{{{env_name}\}}|(?:\\\\\h*\[[^]]*\])|(?:\\[{{}}\[\]])|[^{{}}\[])"
    );
    // Perl argumentBodyRegEx 的第一分支：\begin{..}/\end{..} 在参数体里恒为字面原子，
    // 必须先于 dCODE 消费。否则当 fineTuning.commands.name 排除了 begin|end 时，
    // dCODE 会把它当成环境跨越到后面的 \end{..}，吞掉外层闭括号，
    // 导致外层命令整体失配（issue: filecontents/theorem-indent-preamble）。
    let arg_body_head = format!(r"(?:\\(?:begin|end)\{{{env_name}\}})");
    let m_trail_nc = if is_m {
        format!("(?:\\h*)?(?:{})?(?:\\R\\s*)?", trailing_comment_x)
    } else {
        String::new()
    };

    // 参数前缀 trivia（对应 Perl 的 (?:\s|tc|blank|between)*）：
    //
    // between 的字符类与括号组分支在 `(` 上歧义（字符类匹配单字符
    // `(`，括号组匹配整组 `(-2,-1)`），普通星号在 \{/\[ 匹配失败后
    // 回溯会在两个分支间指数组合——N 个坐标组即 2^N 条路径
    // （ifelsefiONE.tex 的 tikz 坐标链触发 pcre2 match limit exceeded）。
    //
    // 若 between 含括号组分支：括号组前置 + 确定性断言（组内无 `{}[]` 才整组吃）
    // + 所有格星号。`(` 处成为确定性的二选一（组可达则整组，否则字符类单字符），
    // 所有格禁止回溯重试迭代 → 线性；同时保留 Perl 依赖的"把 (...) 整组吞为
    // trivia 以暴露后续 {" 行为（github-issue-35 的 tikz
    // `at (...)frame.north west){...}`：括号组不可达时 `west` 会被
    // 单独匹配成 namedGroupingBracesBrackets 块）。详见下方注释的 ①② 对照。
    // 所有格化不影响括号匹配结果：星号吞掉 {/[ 的唯一途径是括号组
    // （字符类不含括号），丢弃该迭代暴露的是 `(` 而非 {/[，故 Perl
    // 星号任何可靠"丢弃迭代"成功的路径在所有格下同样成功。
    //
    // between 无括号组时（如 no-at-between-args3.yaml 的纯字符类），
    // 各分支两两不相交（字符类无字母、单词分支无类中字符、tc/blank
    // 为字面量），回溯不爆炸——保持普通星号与 Perl 逐字一致，
    // 且不引入用户配置里没有的括号组吞并行为。
    let arg_trivia = if between.contains(r"\([^)]") {
        // 括号组分支要同时满足两个互相冲突的要求：
        //
        // ① **性能**（ifelsefiONE.tex）：`(` 处存在 `组 / 字符类单字符` 两种读法，
        //    普通星号在后续 `\{`/`\[` 失配后会回溯枚举全部 2^N 组合 —— 坐标链
        //    `(-3,-3)--(-3,-2)--…` 直接撞 pcre2 match limit。故必须「组优先 + 所有格」。
        // ② **语义**（texchange/348）：`\n({a});` 里括号组会把 `({a});` 整段吃成
        //    以 `)` 结尾的一次迭代，`\{` 随即无处可匹配；Perl 靠回溯退回 0 次，
        //    改由 between 字符类吃掉 `(`、`{a}` 成为参数体（命令尾部 body 缩进）。
        //    所有格把这次迭代锁死 → 参数整体失配，少 2 空格。
        //
        // 解法：把括号组改成**确定性**的 —— 仅当组内不含 `{` 时才整组消费（前置
        // 断言，零回溯），否则该迭代不可达，自然落到字符类单字符读法。
        // 这正是 Perl 回溯后的那一步，且不再产生分支枚举 → ① ② 同时成立。
        // 注意只禁 `{` 不禁 `[`：`([xshift=-1cm,yshift=-1cm]frame.north west)` 这类
        // tikz 坐标组内部本来就含 `[`（github-issue-35），禁掉会让 `at (...)` 断链。
        // 坐标组 `(-3,-3)` 内无 `{` → 仍整组吞掉。
        let paren_group = r"(?!\([^)]*\{)\([^)]*\)";
        format!(
            "(?:(?:{})|\\s|{}|{}|{})*+",
            paren_group, trailing_comment_x, blank_lit, between_x
        )
    } else {
        format!(
            "(?:\\s|{}|{}|{})*",
            trailing_comment_x, blank_lit, between_x
        )
    };

    // dARG：allArgumentRegEx 的定义块版本
    //
    // 分支顺序必须与 Perl 的 `(?: $argumentBodyRegEx | (??{ $allCodeBlocks}) )*` 一致：
    // 先单字符 arg_body（含优先父节点 head 的 \begin{}/\end{} 字面），递归 dCODE 兜底。
    // arg_body 的兜底分支 `[^{}\[\]]` 在普通字符上必定成功 ⇒ 昂贵的 dCODE 递归
    // 只在 `{`/`[` 等 arg_body 失配处尝试（Perl 的代价分布）。
    // 若把 dCODE 前置，参数体内每个字符位置都会发起一次完整代码块匹配，
    // 嵌套越深代价越乘性放大（issue-378 的 tblr 参数触发 pcre2 match limit）。
    // 参数体用所有格量词：确定性单遍，无划分回溯。
    let d_arg = format!(
        r"(?<dARG>(?:(?:(?:{trivia})(?<!\\)\{{(?:(?:{head})|(?:{arg_body_curly})|(?&dCODE))*\}})|(?:(?:{trivia})(?<!\\)\[(?:(?:{head})|(?:{arg_body})|(?&dCODE))*\])))",
        trivia = arg_trivia,
        head = arg_body_head,
        arg_body = arg_body
    );

    // dENV：environmentRegEx 的定义块版本
    let d_env_body = if let (Some(b), Some(e)) = (env_begin_custom.clone(), env_end_custom.clone()) {
        format!(
            r"(?<dENV>(?xs:(?:{begin})(?:(?&dARG))*{trail}(?<dEBODY>(?:(?&dENV)|(?!{end}).)*)(?:{end}){mtrail}))",
            begin = b,
            end = e,
            trail = m_trail_nc,
            mtrail = m_trail_nc
        )
    } else {
        format!(
            r"(?<dENV>(?xs:\\begin\{{(?<dEN>{env_name})\}}(?:(?&dARG))*{trail}(?<dEBODY>(?:(?&dENV)|(?!\\end\{{).)*)\\end\{{\k<dEN>\}}{mtrail}))",
            trail = m_trail_nc,
            mtrail = m_trail_nc
        )
    };

    // dIF：ifElseFiRegEx 的定义块版本
    let d_if = format!(
        r"(?<dIF>(?xs:(?<!\\newif)\\(?:{if_name})(?<dIBODY>(?>(?:(?&dIF)|(?!\\fi(?![a-zA-Z])).)*))\\fi(?![a-zA-Z]){mtrail}))",
        if_name = if_name,
        mtrail = m_trail_nc
    );

    // specials 分组（非嵌套 / 嵌套）
    let mut sp_begin = Vec::new();
    let mut sp_end = Vec::new();
    let mut spn_begin = Vec::new();
    let mut spn_end = Vec::new();
    if let Some(Value::Sequence(seq)) = get(root, "specialBeginEnd") {
        for item in seq {
            let m = match item.as_mapping() {
                Some(m) => m,
                None => continue,
            };
            if !as_bool(get(m, "lookForThis")) {
                continue;
            }
            if as_str(get(m, "lookForThis")) == Some("verbatim") {
                continue;
            }
            // 用户 yaml 的正则按 Perl 语法写：Perl 里未定义的转义（`\<`、`\>`）等于字面字符，
            // 而 PCRE2 把它们当成词首/词尾断言 → `begin: \<\<`、`end: \>\>` 在两边语义不同
            // （issue241-mod2：`<<...>>` 被当成零宽断言，块边界错乱，进而让外层命令的
            //  `(?:dARG)*+` 无法回溯，`\ifnumequal` 的第 3/4 个参数整段丢失缩进）。
            let b_owned = perl_literal_escapes(as_str(get(m, "begin")).unwrap_or_default());
            let e_owned = perl_literal_escapes(as_str(get(m, "end")).unwrap_or_default());
            let b = b_owned.as_str();
            let e = e_owned.as_str();
            // Perl 允许 end 为空串（如 begin 即完整块的 special），仅 begin 为空才跳过
            if b.is_empty() {
                continue;
            }
            if as_bool(get(m, "nested")) {
                spn_begin.push(b.to_string());
                spn_end.push(e.to_string());
            } else {
                sp_begin.push(b.to_string());
                sp_end.push(e.to_string());
            }
        }
    }
    // Perl 语义：逐项拼接交替（累积为空时不加分隔符；空 end 也占一位）
    let join_alt = |v: &[String]| -> String {
        let mut s = String::new();
        for x in v {
            if s.is_empty() {
                s.push_str(x);
            } else {
                s.push('|');
                s.push_str(x);
            }
        }
        s
    };
    let sp_begin_re = join_alt(&sp_begin);
    let sp_end_re = join_alt(&sp_end);
    let spn_begin_re = join_alt(&spn_begin);
    let spn_end_re = join_alt(&spn_end);
    // trace 由 document.rs 在 operate_on_file 开头发出（Perl `_construct_code_blocks_regex`
    // 的调用时机），不在这里——否则会插在 YAML 读取与 -o 开关日志之间。
    let special_present = !(sp_begin_re.is_empty() && spn_begin_re.is_empty());

    // dSPN：嵌套 special 的定义块版本。
    // 注意 1：{b} 必须包进 (?:...) —— 否则 (?xs:...) 组内的交替会把
    // `\left\(` / `\left\[` 变成"只匹配 begin、不要求 body/end"的分支，
    // 嵌套递归随之失效（参见 specialLeftBracket 用例）。
    // 注意 2：body 星号必须是原子组 (?>A|B)*（对应 Perl 的
    // `(?>(??{ $specialRegExNested })|(?!end).)*`）。没有原子组时，
    // 整体匹配失败后引擎会回溯进递归调用、给嵌套块换一个更长的 body，
    // 导致 `\[ ... \]` 一路吞到文档末尾最后一个 `\]`（special-nested-mod1）。
    let d_spn = if spn_begin.is_empty() {
        String::new()
    } else {
        format!(
            r"(?<dSPN>(?xs:(?:{b})(?<dSBody>(?>(?&dSPN)|(?!(?:{e})).)*)(?:{e}){mtrail}))",
            b = spn_begin_re,
            e = spn_end_re,
            mtrail = m_trail_nc
        )
    };

    // dCODE 内联的 specials 分支（非嵌套平铺 + 嵌套走 dSPN）
    // 非嵌套分支对应 Perl $specialRegEx，body 同为原子组 (?>(?!end).)*
    let d_specials_alt = {
        let mut s = String::new();
        if !sp_begin_re.is_empty() {
            s.push_str(&format!(
                r"|(?xs:(?:{b})(?>(?!(?:{e})).)*(?:{e}){mtrail})",
                b = sp_begin_re,
                e = sp_end_re,
                mtrail = m_trail_nc
            ));
        }
        if !spn_begin_re.is_empty() {
            s.push_str("|(?&dSPN)");
        }
        if s.is_empty() {
            s.push_str("|(?!x)x"); // 永不匹配的占位分支
        }
        s
    };

    // dCODE：allCodeBlocks 的定义块版本
    //
    // 命令分支前置 `(?!(?&dENV))`，对应 Perl `(??{ $allCodeBlocks })` 里
    // COMMAND 的 `(?!(?:$environmentRegEx))` 守卫：`\begin{..}` 必须留给
    // dENV 分支整体消费。否则命令分支会先把 `\begin{pspicture}` 吃成
    // 「命令 + 参数」（begin 是合法命令名），紧跟的 `(*PRUNE)` 又锁死
    // 这次选择；参数体随后卡在 `[`（如 `\begin{pspicture}[showgrid}` 这种
    // 上游演示里的「不闭合可选参数」）无法回退到环境分支，
    // 整个外层命令/无名块失配（documentation/pstricks 演示用例）。
    let d_code = format!(
        r"(?<dCODE>\s*(?:(?:(?:(?!(?&dENV))\\(?:{cmd})|(?:{key})(?:\s|{tc}|{bl})*+=|(?<!\\)(?<![a-zA-Z])(?:{named})|(?<![a-zA-Z]))(?:(?&dARG))+{mtrail})(*PRUNE)|(?&dENV)(*PRUNE)|(?&dIF)(*PRUNE){d_specials_alt}))",
        cmd = cmd_name,
        key = key_x,
        tc = trailing_comment_x,
        bl = blank_lit,
        named = named_name,
        mtrail = m_trail_nc,
        d_specials_alt = d_specials_alt
    );

    let defs = format!(
        "(?(DEFINE){d_arg}{d_env}{d_if}{d_spn}{d_code})",
        d_arg = d_arg,
        d_env = d_env_body,
        d_if = d_if,
        d_spn = d_spn,
        d_code = d_code
    );

    // ---- 顶层 allCodeBlocks（可读捕获内联） ----
    // Perl $keyEqVal：name 后跟普通星号 (?:\s|tc|blank)*=（Blocks.pm:65-66），逐字移植。
    let key_eqval = format!("{}(?:\\s|{}|{})*+=", key_x, trailing_comment_x, blank_lit);

    // 后缀工具：给子模式内的逻辑组名加后缀
    let suff = |s: &str, suf: &str| -> String { format!("{s}{suf}") };

    let all_arg = |suf: &str| -> String {
        format!(
            r"(?:(?:(?<{mb}>(?:{trivia})(?<!\\)\{{)(?<{mby}>(?:(?:{head})|(?:{arg_body_curly})|(?&dCODE))*)(?<{me}>(?<!\\)\}}))|(?:(?<{ob}>(?:{trivia})(?<!\\)\[)(?<{oby}>(?:(?:{head})|(?:{arg_body})|(?&dCODE))*)(?<{oe}>(?<!\\)\])))",
            mb = suff("MANDARGBEGIN", suf),
            mby = suff("MANDARGBODY", suf),
            me = suff("MANDARGEND", suf),
            ob = suff("OPTARGBEGIN", suf),
            oby = suff("OPTARGBODY", suf),
            oe = suff("OPTARGEND", suf),
            trivia = arg_trivia,
            arg_body = arg_body,
            arg_body_curly = arg_body_curly,
            head = arg_body_head
        )
    };

    let m_trail = |suf: &str| -> String {
        if !is_m {
            return String::new();
        }
        format!(
            r"(?:(?<{a}>\h*)?(?<{c}>{tc})?(?<{l}>\R\s*)?)",
            a = suff("TRAILINGHSPACE", suf),
            c = suff("TRAILINGCOMMENT", suf),
            l = suff("TRAILINGLINEBREAK", suf),
            tc = trailing_comment_x
        )
    };

    // 环境（顶层可读版）
    let env_pattern = if let (Some(b), Some(e)) = (env_begin_custom.clone(), env_end_custom.clone()) {
        format!(
            r"(?xs:(?<ENVBEGIN>{begin})(?<ENVARGS>(?:(?&dARG))*{mtrail_nc})(?<ENVBODY>(?:(?&dENV)|(?!{end}).)*)(?<ENVEND>{end}){trail})",
            begin = b,
            end = e,
            mtrail_nc = m_trail_nc,
            trail = m_trail("_env")
        )
    } else {
        format!(
            r"(?xs:(?<ENVBEGIN>\\begin\{{(?<ENVNAME>{env_name})\}})(?<ENVARGS>(?:(?&dARG))*{mtrail_nc})(?<ENVBODY>(?:(?&dENV)|(?!\\end\{{).)*)(?<ENVEND>\\end\{{\k<ENVNAME>\}}){trail})",
            mtrail_nc = m_trail_nc,
            trail = m_trail("_env")
        )
    };

    let if_pattern = format!(
        r"(?xs:(?<IFELSEFIBEGIN>(?<!\\newif)\\(?<IFELSEFINAME>{if_name}))(?<IFELSEFIBODY>(?>(?:(?&dIF)|(?!\\fi(?![a-zA-Z])).)*))(?<IFELSEFIEND>\\fi(?![a-zA-Z])){trail})",
        trail = m_trail("_if")
    );

    let sp_pattern = if sp_begin.is_empty() {
        String::new()
    } else {
        // Perl $specialRegEx：body 为原子组 (?>(?!end).)*
        format!(
            r"(?xs:(?<SPECIALBEGIN>{b})(?<SPECIALBODY>(?>(?!(?:{e})).)*)(?<SPECIALEND>(?:{e})){trail})",
            b = sp_begin_re,
            e = sp_end_re,
            trail = m_trail("_sp")
        )
    };
    let spn_pattern = if spn_begin.is_empty() {
        String::new()
    } else {
        // Perl $specialRegExNested：body 为原子组 (?>(??{nested})|(?!end).)*
        format!(
            r"(?xs:(?<{b0}>{b})(?<{b1}>(?>(?&dSPN)|(?!(?:{e})).)*)(?<{b2}>(?:{e})){trail})",
            b0 = "SPECIALBEGIN_spn",
            b1 = "SPECIALBODY_spn",
            b2 = "SPECIALEND_spn",
            b = spn_begin_re,
            e = spn_end_re,
            trail = m_trail("_spn")
        )
    };

    // COMMAND 的 \begin 保护（Perl 无条件 (?!(?:$environmentRegEx))，与 specials 无关）
    let cmd_guard = "(?!(?&dENV))".to_string();

    let thing = format!(
        r"(?<THINGWITHARGS>(?>(?:(?<COMMAND>{guard}\\(?<COMMANDNAME>{cmd}))|(?<KEYEQVAL>{key})|(?<NAMEDBRACESBRACKETS>(?<!\\)(?<![a-zA-Z])(?:{named}))|(?<UNNAMEDBRACESBRACKETS>(?<![a-zA-Z]))))(?<ARGUMENTS>(?:(?&dARG))+{trail}))",
        guard = cmd_guard,
        cmd = cmd_name,
        key = key_eqval,
        named = named_name,
        trail = m_trail("")
    );

    let mut alt_specials = String::new();
    if !sp_pattern.is_empty() {
        alt_specials.push_str("|(?<SPECIAL>");
        alt_specials.push_str(&sp_pattern);
        alt_specials.push(')');
    }
    if !spn_pattern.is_empty() {
        alt_specials.push_str("|(?<SPECIALNESTED>");
        alt_specials.push_str(&spn_pattern);
        alt_specials.push(')');
    }

    let all_code_blocks_src = format!(
        r"{defs}(?<LEADINGSPACE>\s*)(?:{thing}(*PRUNE)|(?<ENVIRONMENT>{env})(*PRUNE)|(?<IFELSEFI>{ifp})(*PRUNE){specials})",
        defs = defs,
        thing = thing,
        env = env_pattern,
        ifp = if_pattern,
        specials = alt_specials
    );

    let all_code_blocks = crate::patterns::re_new(&all_code_blocks_src).unwrap_or_else(|e| {
        panic!("allCodeBlocks 拼装失败: {e}\n{all_code_blocks_src}");
    });
    // 所有格兜底：把参数体循环换成「dCODE 优先 + *+」。
    // 精确版（body 优先 + `*`）照抄 Perl，能靠回溯让嵌套块不越过参数闭括号；
    // 但病态输入会指数回溯（broken.tex 的 match limit），此时回退到所有格版，
    // 它不会回溯、绝不 panic，代价只是极端嵌套下块边界可能与 Perl 略有出入。
    // 参数体循环的「精确 / 所有格」两种形态（同一段文本在下面的多处拼接中出现）
    let loop_pairs: Vec<(String, String)> = [&arg_body_curly, &arg_body]
        .iter()
        .map(|b| {
            (
                format!(r"(?:(?:{head})|(?:{b})|(?&dCODE))*", head = arg_body_head, b = b),
                format!(r"(?:(?:{head})|(?&dCODE)|(?:{b}))*+", head = arg_body_head, b = b),
            )
        })
        .collect();
    let all_code_blocks_fast =
        crate::patterns::re_new(&fast_variant(&all_code_blocks_src, &loop_pairs))
            .unwrap_or_else(|e| panic!("allCodeBlocks(所有格) 拼装失败: {e}"));

    // codeBlockLookFor：快速预检
    let mut look = String::from(r"[{[]|\\if");
    if !sp_begin_re.is_empty() {
        look.push('|');
        look.push_str(&sp_begin_re);
    }
    if !spn_begin_re.is_empty() {
        look.push('|');
        look.push_str(&spn_begin_re);
    }
    let code_block_look_for = crate::patterns::re_new(&format!("(?s:{look})")).expect("codeBlockLookFor");

    // _indent_all_args 用：\G 逐个参数（捕获可读）
    let one_arg_src = format!(r"{defs}(?x)\G(?:{}){}", all_arg(""), m_trail(""));
    let one_arg = crate::patterns::re_new(&one_arg_src).expect("one_arg");
    let one_arg_fast =
        crate::patterns::re_new(&fast_variant(&one_arg_src, &loop_pairs)).expect("one_arg_fast");

    let special_end = sp_end
        .into_iter()
        .chain(spn_end.into_iter())
        .map(|e| (String::new(), e))
        .collect();

    let re_arg_only_comment =
        crate::patterns::re_new(&format!(r"(?s)^\s*{}\s*$", trailing_comment_x)).expect("re_arg_only_comment");
    let re_arg_only_space = crate::patterns::re_new(r"(?s)^\s*$").expect("re_arg_only_space");
    let re_body_linebreak = crate::patterns::re_new(r"^\h*\R").expect("re_body_linebreak");
    let re_key_split = crate::patterns::re_new(&format!(
        r"(?:(?:\s|{}|{})*)=",
        trailing_comment_x, blank_lit
    ))
    .expect("re_key_split");

    // items 正则（_find_env_items 用）：body 遇到下一个 item 截止，内部可嵌套环境
    let defs = defs_pre(&d_arg, &d_env_body, &d_if, &d_spn, &d_code);
    let re_item_no_space = crate::patterns::re_new(&format!(
        r"(?s){defs}(?:(?<ITEMBEGIN>{item})(?<ITEMBODY>(?:(?&dENV)|(?!(?:{item})).)*))",
        item = item_regexp
    ))
    .expect("re_item_no_space");
    let re_item = crate::patterns::re_new(&format!(
        r"(?s){defs}(?<WS>\s*)(?:(?<ITEMBEGIN>{item})(?<ITEMBODY>(?:(?&dENV)|(?!(?:{item})).)*))",
        item = item_regexp
    ))
    .expect("re_item");

    let one_arg_names: std::collections::HashMap<String, usize> = one_arg
        .capture_names()
        .iter()
        .enumerate()
        .filter_map(|(i, n)| n.clone().map(|n| (n, i)))
        .collect();

    PatternSet {
        all_code_blocks,
        all_code_blocks_fast,
        code_block_look_for,
        one_arg,
        one_arg_fast,
        one_arg_names,
        trailing_comment,
        item_regexp,
        special_end,
        re_arg_only_comment,
        re_arg_only_space,
        re_body_linebreak,
        re_key_split,
        re_item_no_space,
        re_item,
        re_middle: None,
        defs,
        special_all: alt_specials.trim_start_matches('|').to_string(),
        special_present,
    }
}

pub fn defs_pre(d_arg: &str, d_env: &str, d_if: &str, d_spn: &str, d_code: &str) -> String {
    format!("(?(DEFINE){d_arg}{d_env}{d_if}{d_spn}{d_code})")
}
