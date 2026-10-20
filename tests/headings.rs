//! 标题（heading）层级缩进测试，基准取自 tests/cases/headings。
//! 覆盖 latexindent.pl-main/test-cases/headings/headings-test-cases.sh 的全部调用。
mod common;
use common::{check, check_log_lines, check_owned, report, run};

/// 一条上游 shell 调用：`latexindent.pl <args...> <input>` → 产物 `expected`。
struct HCase {
    input: &'static str,
    args: &'static [&'static str],
    expected: &'static str,
}

/// 上游脚本 39 次调用中尚未搬入的 36 条（L7/L18/L19 此前已搬运）。
const CASES: &[HCase] = &[
    // 第一组：headings-first
    HCase { input: "headings-first.tex", args: &["-s", "-o", "+1", "-l=indentRules1,levels1"], expected: "headings-first1.tex" },
    HCase { input: "headings-first.tex", args: &["-s", "-o", "+2", "-l=levels2,resetChapter"], expected: "headings-first2.tex" },
    // 第二组：headings-second
    HCase { input: "headings-second.tex", args: &["-s", "-o", "+-default", "-local", "resetChapter"], expected: "headings-second-default.tex" },
    HCase { input: "headings-second.tex", args: &["-s", "-o", "+1", "-l=indentRules1,levels1"], expected: "headings-second1.tex" },
    HCase { input: "headings-second.tex", args: &["-s", "-o", "+2", "-l=levels2,resetChapter", "-t"], expected: "headings-second2.tex" },
    HCase { input: "headings-second.tex", args: &["-s", "-o", "+2-indentRules", "-l=indentRules1,levels2", "-tt"], expected: "headings-second2-indentRules.tex" },
    // 大（legacy）用例
    HCase { input: "testHeadings.tex", args: &["-s", "-o", "+2", "-l=levels2"], expected: "testHeadings2.tex" },
    HCase { input: "testHeadings.tex", args: &["-s", "-o", "+3", "-l=levels3"], expected: "testHeadings3.tex" },
    HCase { input: "testHeadings.tex", args: &["-s", "-o", "+4", "-l=levels4"], expected: "testHeadings4.tex" },
    // 自定义标题
    HCase { input: "customHeadings.tex", args: &["-s", "-o", "+1", "-l", "custom1"], expected: "customHeadings1.tex" },
    HCase { input: "customHeadings.tex", args: &["-s", "-o", "+2", "-l", "custom2"], expected: "customHeadings2.tex" },
    HCase { input: "customHeadings.tex", args: &["-s", "-o", "+3", "-l", "custom3"], expected: "customHeadings3.tex" },
    // modifyLineBreaks 实验：levels1
    HCase { input: "headings-single-line.tex", args: &["-s", "-o", "+-mod1", "-l=levels1,headingStartOnOwnLine", "-m"], expected: "headings-single-line-mod1.tex" },
    HCase { input: "headings-single-line-simple.tex", args: &["-s", "-o", "+-mod1", "-l=levels1,headingStartOnOwnLine", "-m"], expected: "headings-single-line-simple-mod1.tex" },
    HCase { input: "headings-single-line.tex", args: &["-s", "-o", "+-comment-mod1", "-l=levels1,headingStartOwnLineComment", "-m"], expected: "headings-single-line-comment-mod1.tex" },
    HCase { input: "headings-blank-line.tex", args: &["-s", "-o", "+-default", "-l", "resetChapter"], expected: "headings-blank-line-default.tex" },
    HCase { input: "headings-blank-line.tex", args: &["-s", "-o", "+-mod1", "-l=levels1,headingStartOnOwnLine,resetChapter", "-m"], expected: "headings-blank-line-mod1.tex" },
    HCase { input: "headings-blank-line.tex", args: &["-s", "-o", "headings-remove-blank-line-mod1", "-l=levels1,headingNotStartOwnLine,resetChapter", "-m"], expected: "headings-remove-blank-line-mod1.tex" },
    // levels2
    HCase { input: "headings-single-line.tex", args: &["-s", "-o", "+-mod2", "-l=levels2,headingStartOnOwnLine", "-m", "-g=one.log"], expected: "headings-single-line-mod2.tex" },
    HCase { input: "headings-single-line.tex", args: &["-s", "-o", "+-comment-mod2", "-l=levels2,headingStartOwnLineComment", "-m"], expected: "headings-single-line-comment-mod2.tex" },
    // 单行多标题
    HCase { input: "headings-single-line-many.tex", args: &["-s", "-o", "+-mod1", "-l=levels1,headingStartOnOwnLine,../specials/special-mod1", "-m"], expected: "headings-single-line-many-mod1.tex" },
    HCase { input: "headings-single-line-many.tex", args: &["-s", "-o", "+-mod2", "-l=levels2,headingStartOnOwnLine,../specials/special-mod1", "-m"], expected: "headings-single-line-many-mod2.tex" },
    HCase { input: "headings-single-line-many.tex", args: &["-s", "-o", "+-mod3", "-l=levels3,headingStartOnOwnLine,../specials/special-mod1", "-m"], expected: "headings-single-line-many-mod3.tex" },
    // 带星号的标题
    HCase { input: "headings-names-with-stars.tex", args: &["-s", "-o", "+-mod5", "-l=levels5,resetChapter"], expected: "headings-names-with-stars-mod5.tex" },
    HCase { input: "headings-names-with-stars.tex", args: &["-s", "-o", "+-mod6", "-l=levels6,resetChapter"], expected: "headings-names-with-stars-mod6.tex" },
    // poly-switch = 3（空行模式）
    HCase { input: "headings-single-line-many.tex", args: &["-s", "-o", "+-mod7", "-l=levels3,headingStartOnOwnLine,../specials/special-mod1,chapterBlankLines", "-m"], expected: "headings-single-line-many-mod7.tex" },
    HCase { input: "headings-single-line-many.tex", args: &["-s", "-o", "+-mod8", "-l=levels3,headingStartOnOwnLine,../specials/special-mod33,headingsBlankLines", "-m"], expected: "headings-single-line-many-mod8.tex" },
    HCase { input: "headings-single-line-many.tex", args: &["-s", "-o", "+-mod9", "-l=aronovgj", "-m"], expected: "headings-single-line-many-mod9.tex" },
    HCase { input: "headings-blank-line.tex", args: &["-s", "-o", "+-mod2", "-l=levels3,headingStartOnOwnLine,../specials/special-mod33,headingsBlankLines", "-m"], expected: "headings-blank-line-mod2.tex" },
    // -y 开关与 issue 用例
    HCase { input: "headings1.tex", args: &["-s", "-o=+-mod1", "-y= indentAfterHeadings:paragraph:indentAfterThisHeading:1;level:1 "], expected: "headings1-mod1.tex" },
    HCase { input: "issue-519.tex", args: &["-s", "-l", "issue-519", "-o=+-mod1"], expected: "issue-519-mod1.tex" },
    HCase { input: "issue-448.tex", args: &["-s", "-l", "issue-448a", "-o=+-mod1"], expected: "issue-448-mod1.tex" },
    // preamble 内含标题命令（另有 egrep 'found:' 日志基准，见 `sec_in_preamble_log`）
    HCase { input: "sec-in-preamble.tex", args: &["-s", "-t", "-l=sec-in-preamble1.yaml", "-o=+-mod1"], expected: "sec-in-preamble-mod1.tex" },
    HCase { input: "sample-large.tex", args: &["-s", "-o=+-mod1", "-l", "indentHeadings"], expected: "sample-large-mod1.tex" },
    // 注：上游 `-s testHeadings -l obsolete.yaml`（L68）不产文件（实测 Perl 亦只写 stdout），
    // 该条语义由下方 `obsolete_warnings` 的日志基准承担，故此处不建 HCase。
    HCase { input: "issue-644.tex", args: &["-s", "-l", "issue-644", "-o=+-mod1"], expected: "issue-644-mod1.tex" },
];

/// 已无待修项（原 3 条均已修复并转正）。
const PENDING: &[&str] = &[];

fn run_case(c: &HCase) {
    check_owned(
        "headings",
        c.input,
        &c.args.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
        c.expected,
    );
}

#[test]
fn headings_first_reset_chapter() {
    check(
        "headings",
        "headings-first.tex",
        &["-s", "-l=resetChapter", "-o=+-default"],
        "headings-first-default.tex",
    );
}

#[test]
fn test_headings_default() {
    check("headings", "testHeadings.tex", &["-s", "-o=+-default"], "testHeadings-default.tex");
}

// levels1 控制各层级缩进
#[test]
fn test_headings_levels1() {
    check("headings", "testHeadings.tex", &["-s", "-l=levels1", "-o=+1"], "testHeadings1.tex");
}

/// 补齐的标题用例（36 条）。收集全部失败后一次性 report。
#[test]
fn heading_cases() {
    for c in CASES {
        if PENDING.contains(&c.expected) {
            continue;
        }
        run_case(c);
    }
    report();
}

/// 曾经与基准不一致的用例：现已全部修复，保留函数以锁住回归。
#[test]
fn headings_pending() {
    for c in CASES {
        if !PENDING.contains(&c.expected) {
            continue;
        }
        run_case(c);
    }
    // 上游 L63 的日志摘录：`egrep -i 'found:' indent.log > headings-info1.txt`。
    // 差异：Rust 侧 -t 追踪缺少 preamble / heading 三类 found 行。
    check_log_lines(
        "headings",
        "sec-in-preamble.tex",
        &["-s", "-t", "-l=sec-in-preamble1.yaml", "-o=+-mod1"],
        "found:",
        "headings-info1.txt",
    );
    // 上游 L68 的日志摘录：obsolete.yaml 的废弃字段警告（perl -p0i 同款截取）。
    // 差异：Rust 侧仅输出 `*Obsolete YAML`，没有上游那整段 YAML 转储。
    let c = run(
        "headings",
        "testHeadings.tex",
        &["-s".into(), "-l".into(), "obsolete.yaml".into()],
    );
    c.assert_success();
    c.assert_log_section(
        "indent.log",
        "WARN:  Obsolete",
        Some("INFO:"),
        "obsolete.txt",
    );
}
