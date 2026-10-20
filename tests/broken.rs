//! broken：残缺/病态输入的解析与日志摘录，覆盖 tests/cases/broken/broken-test-cases.sh 的 8 个调用。
//! 基准取自 tests/cases/broken 已提交的 latexindent.pl 结果。
mod common;
use common::run;

const DIR: &str = "broken";

/// `latexindent.pl -s broken -o=+-mod1`：未闭合参数的嵌套键值/条件块。
#[test]
fn broken_default() {
    let c = run(DIR, "broken.tex", &["-s".into(), "-o=+-mod1".into()]);
    c.assert_success();
    c.assert_matches("broken-mod1.tex", "broken-mod1.tex");
}

/// no-specials.yaml 全部 lookForThis: 0 → 日志报 specialBeginEnd empty。
#[test]
fn broken_no_specials() {
    let c = run(
        DIR,
        "broken.tex",
        &[
            "-s".into(),
            "-l".into(),
            "no-specials.yaml".into(),
            "-o=+-mod2".into(),
            "-t".into(),
        ],
    );
    c.assert_success();
    c.assert_matches("broken-mod2.tex", "broken-mod2.tex");
    c.assert_log_grep("indent.log", "specialBeginEnd empty", "no-specials.txt");
}

/// lookForThis: verbatim 的 special 走 verbatim 保护（found: 摘录含 VERBATIM special found）。
#[test]
fn ifnextchar() {
    let c = run(
        DIR,
        "ifnextchar.tex",
        &[
            "-s".into(),
            "-o=+-mod1".into(),
            "-l".into(),
            "com-name-special1.yaml".into(),
            "-t".into(),
        ],
    );
    c.assert_success();
    c.assert_matches("ifnextchar-mod1.tex", "ifnextchar-mod1.tex");
    c.assert_log_grep("indent.log", "found:", "ifnextchar.txt");
}

/// 残缺环境：\begin 无 \end 时按命令解析。
#[test]
fn broken1() {
    let c = run(
        DIR,
        "broken1.tex",
        &["-s".into(), "-o=+-mod1".into(), "-t".into()],
    );
    c.assert_success();
    c.assert_matches("broken1-mod1.tex", "broken1-mod1.tex");
    c.assert_log_grep("indent.log", "found:", "broken1.txt");
}

/// 未闭合 \if 的 parbox 参数。
#[test]
fn broken2() {
    let c = run(
        DIR,
        "broken2.tex",
        &["-s".into(), "-o=+-mod1".into(), "-t".into()],
    );
    c.assert_success();
    c.assert_matches("broken2-mod1.tex", "broken2-mod1.tex");
    c.assert_log_grep("indent.log", "found:", "broken2.txt");
}

/// 未闭合 \[ 的 parbox 参数。
#[test]
fn broken3() {
    let c = run(
        DIR,
        "broken3.tex",
        &["-s".into(), "-o=+-mod1".into(), "-t".into()],
    );
    c.assert_success();
    c.assert_matches("broken3-mod1.tex", "broken3-mod1.tex");
    c.assert_log_grep("indent.log", "found:", "broken3.txt");
}

/// 多层嵌套且括号不配平的 tcbraster 用例。
#[test]
fn broken4() {
    let c = run(
        DIR,
        "broken4.tex",
        &["-s".into(), "-o=+-mod1".into(), "-t".into()],
    );
    c.assert_success();
    c.assert_matches("broken4-mod1.tex", "broken4-mod1.tex");
}

/// broken4 的 found: 轨迹：Perl 引擎 quirk——`(?!(?:$environmentRegEx))` 负向断言内部
/// 评估触发的 (*PRUNE) 泄漏到外层，令 `\begin{...}` 命令分支整体失效（本例 pos 0 的
/// `\begin{tcbraster}` 被跳过、`[...]` 记为无名组）。pcre2 无此泄漏，trace 必然不同；
/// 产物不受影响。属上游 Perl 引擎自身 quirk，按用户决定放弃不修。
#[test]
#[ignore = "Perl 引擎 PRUNE 泄漏 quirk，pcre2 无法复现（产物一致，仅 found: trace 不同）"]
fn broken4_found_trace() {
    let c = run(
        DIR,
        "broken4.tex",
        &["-s".into(), "-o=+-mod1".into(), "-t".into()],
    );
    c.assert_success();
    c.assert_log_grep("indent.log", "found:", "broken4.txt");
}

/// obsolete.yaml 的废弃字段警告（perl -p0i 同款截取：WARN:  Obsolete 到 INFO: 之前）。
#[test]
fn obsolete_warnings() {
    let c = run(
        DIR,
        "obsolete.tex",
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
