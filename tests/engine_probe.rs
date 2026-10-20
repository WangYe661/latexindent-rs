//! 探针：验证 pcre2 是否具备移植 latexindent 所需的语义（递归正则、原子组、
//! 递归调用中的捕获隔离、重复命名组、\h \R \X）。任何断言失败都意味着
//! 移植时需要用手工扫描器或改写正则来补偿。

use pcre2::bytes::Regex;

fn s(m: pcre2::bytes::Match) -> String {
    String::from_utf8(m.as_bytes().to_vec()).unwrap()
}

const ENV_PATTERN: &str = concat!(
    r"(?<ENVALL>\\begin\{(?<ENVNAME>[a-z*]+)\}",
    r"(?<ENVBODY>(?:(?&ENVALL)|(?!\\end\{).)*)",
    r"(?<ENVEND>\\end\{\k<ENVNAME>\}))"
);

#[test]
fn atomic_group_supported() {
    let re = Regex::new(r"(?>ab|a)b").unwrap();
    assert!(re.find(br"ab").unwrap().is_none(), "(?>ab|a)b 对 ab 不应匹配（原子组不回溯）");
    assert!(re.find(br"xab").unwrap().is_none());
}

#[test]
fn self_recursion_and_backref() {
    let re = Regex::new(ENV_PATTERN).unwrap();
    let text = r"\begin{a}x\begin{b}y\end{b}z\end{a}";
    let caps = re.captures(text.as_bytes()).unwrap().unwrap();
    assert_eq!(s(caps.get(0).unwrap()), text, "应匹配整个嵌套结构");
    assert_eq!(s(caps.name("ENVNAME").unwrap()), "a", "递归调用中的捕获不应覆盖外层 ENVNAME");
}

#[test]
fn recursion_capture_isolation() {
    // 外层 ARG 的 BODY 不应被内层递归捕获覆盖（Perl (??{}) 语义）
    let re = Regex::new(r"(?<ARG>\{(?<BODY>(?:(?&ARG)|[^{}])*)\})").unwrap();
    let caps = re.captures(br"{a{b}c}").unwrap().unwrap();
    assert_eq!(s(caps.name("BODY").unwrap()), "a{b}c");
}

#[test]
fn mismatched_nesting_semantics() {
    let re = Regex::new(ENV_PATTERN).unwrap();
    let text = r"\begin{a}\begin{b}x\end{a}\end{b}";
    let caps = re.captures(text.as_bytes()).unwrap().unwrap();
    assert_eq!(s(caps.get(0).unwrap()), r"\begin{a}\begin{b}x\end{a}");
    assert_eq!(s(caps.name("ENVNAME").unwrap()), "a");
}

#[test]
fn duplicate_names_not_allowed() {
    // PCRE2 默认拒绝重复命名组 → 组合正则时需对重复出现的组名加后缀
    assert!(Regex::new(r"(?<X>a)(?<X>b)").is_err());
}

#[test]
fn subroutine_call_captures_discarded() {
    // (?&DEF) 调用内的捕获在返回后被丢弃 → 可读捕获必须内联在主流正则中
    let re = Regex::new(r"a(?&DEF)z(?:0(?<DEF>(?<DVAL>\d+)))?").unwrap();
    let caps = re.captures(br"a42z").unwrap().unwrap();
    assert_eq!(caps.name("DVAL").map(s), None);
}

#[test]
fn classes_supported() {
    for pat in [r"\h", r"\R", r"\X", r"\v", r"(?<!\\)%", r"(*PRUNE)"] {
        assert!(Regex::new(pat).is_ok(), "应支持 {pat}");
    }
}
