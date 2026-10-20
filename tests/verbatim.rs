//! verbatim 环境保护测试，基准取自 tests/cases/verbatim。
//!
//! T1 搬运：覆盖 `latexindent.pl-main/test-cases/verbatim/verbatim-test-cases.sh`
//! 的全部 **136** 次调用。脚本第 3 行设 `verbatimTest=1`，`common.sh` 据此把
//! `loopmin` 改写为 `-1`，因此脚本第 17 行的 `for (( i=$loopmin ; i <= $loopmax ; i++ ))`
//! 展开为 **i = -1, 0, 1, 2, 3, 4 共 6 轮**（每轮 18 次调用 = 108 条）。
//! 分段：循环前静态 5 条 + 循环 108 条 + 循环后静态 23 条 = 136 条。
mod common;
use common::{check, check_owned, check_overwrite, report};

/// 一条待补用例：输入文件名、参数、产物（基准）文件名。
type Case = (String, Vec<String>, String);

fn case(input: &str, args: &[&str], expected: &str) -> Case {
    (
        input.to_string(),
        args.iter().map(|a| a.to_string()).collect(),
        expected.to_string(),
    )
}

/// 脚本中尚未搬成断言的 **134** 条调用（136 条扣掉本文件原有的 2 条）。
fn ported_cases() -> Vec<Case> {
    let mut v: Vec<Case> = Vec::new();

    // ---- 循环前静态：L9 / L12 / L13（L8 与 L10 已由下方原有两个断言覆盖）----
    v.push(case(
        "verbatim-preamble.tex",
        &["-m", "-s", "-o=+-mod2", "-l=verb-begin2"],
        "verbatim-preamble-mod2.tex",
    ));
    v.push(case(
        "verbatim-starred1.tex",
        &["-s", "-l=verb-equation-star", "-o=+-mod1"],
        "verbatim-starred1-mod1.tex",
    ));
    v.push(case(
        "verbatim-starred2.tex",
        &["-s", "-l=no-indent-star", "-o=+-mod1"],
        "verbatim-starred2-mod1.tex",
    ));

    // ---- 循环体：i = -1, 0, 1, 2, 3, 4（6 轮 × 18 条 = 108 条）----
    for i in -1..=4i32 {
        let n = i.to_string();
        // L21–L23：verbatim 环境，begin 版
        let lb = format!("-l=verb-begin{n}");
        let ob = format!("-o=+-mod{n}");
        for base in ["verbatim1", "verbatim2", "verbatim3"] {
            v.push(case(
                &format!("{base}.tex"),
                &["-s", "-m", &lb, &ob],
                &format!("{base}-mod{n}.tex"),
            ));
        }
        // L24–L26：verbatim 环境，end 版
        let le = format!("-l=verb-end{n}");
        let oe = format!("-o=+-end-mod{n}");
        for base in ["verbatim1", "verbatim2", "verbatim3"] {
            v.push(case(
                &format!("{base}.tex"),
                &["-s", "-m", &le, &oe],
                &format!("{base}-end-mod{n}.tex"),
            ));
        }
        // L29–L31：verbatim special，spec 版
        let ls = format!("-l=verb-special,verb-spec{n}");
        for base in ["verbatim-special1", "verbatim-special2", "verbatim-special3"] {
            v.push(case(
                &format!("{base}.tex"),
                &["-s", "-m", &ls, &ob],
                &format!("{base}-mod{n}.tex"),
            ));
        }
        // L32–L34：verbatim special，end-spec 版
        let lx = format!("-l=verb-special,verb-end-spec{n}");
        for base in ["verbatim-special1", "verbatim-special4", "verbatim-special5"] {
            v.push(case(
                &format!("{base}.tex"),
                &["-s", "-m", &lx, &oe],
                &format!("{base}-end-mod{n}.tex"),
            ));
        }
        // L37–L39：verbatim commands，begin 版
        for base in [
            "verbatim-commands1",
            "verbatim-commands2",
            "verbatim-commands3",
        ] {
            v.push(case(
                &format!("{base}.tex"),
                &["-s", "-m", &lb, &ob],
                &format!("{base}-mod{n}.tex"),
            ));
        }
        // L41–L43：verbatim commands，end 版
        for base in [
            "verbatim-commands1",
            "verbatim-commands4",
            "verbatim-commands5",
        ] {
            v.push(case(
                &format!("{base}.tex"),
                &["-s", "-m", &le, &oe],
                &format!("{base}-end-mod{n}.tex"),
            ));
        }
    }

    // ---- 循环后静态：L48 – L86 共 23 条 ----
    v.push(case(
        "verbatim6.tex",
        &["-s", "-m", "-l=verb-env-1-named", "-o=+-mod-1"],
        "verbatim6-mod-1.tex",
    ));
    v.push(case(
        "verbatim7.tex",
        &["-s", "-m", "-o=+-mod-1-default"],
        "verbatim7-mod-1-default.tex",
    ));
    v.push(case(
        "verbatim7.tex",
        &["-s", "-m", "-l=nameAsRegex", "-o=+-mod-1"],
        "verbatim7-mod-1.tex",
    ));
    v.push(case(
        "verbatim7.tex",
        &["-s", "-m", "-l=nameAsRegex2", "-o=+-mod-2"],
        "verbatim7-mod-2.tex",
    ));
    v.push(case(
        "verbatim7a.tex",
        &["-s", "-m", "-l=nameAsRegex", "-o=+-mod-1"],
        "verbatim7a-mod-1.tex",
    ));
    v.push(case(
        "verbatim5.tex",
        &["-s", "-m", "-l=verb-env-1-named", "-o=+-mod-1-named"],
        "verbatim5-mod-1-named.tex",
    ));
    v.push(case(
        "verbatim5.tex",
        &[
            "-s",
            "-m",
            "-l=verb-env-1-named,verb-begin-1",
            "-o=+-mod-1-named-mk2",
        ],
        "verbatim5-mod-1-named-mk2.tex",
    ));
    v.push(case(
        "verbatim-special3.tex",
        &["-s", "-m", "-l=verb-special,verb-spec2-named", "-o=+-mod2-named"],
        "verbatim-special3-mod2-named.tex",
    ));
    v.push(case(
        "verbatim-trailing-comments.tex",
        &["-s", "-m", "-l=verb-begin2", "-o=+-mod2"],
        "verbatim-trailing-comments-mod2.tex",
    ));
    v.push(case(
        "verbatim-commands.tex",
        &["-s", "-l=nameAsRegex", "-o=+-mod1"],
        "verbatim-commands-mod1.tex",
    ));
    v.push(case(
        "verbatim-trailing-comments1.tex",
        &["-s", "-o=+-default"],
        "verbatim-trailing-comments1-default.tex",
    ));
    for base in ["issue-266", "issue-266a", "issue-266b"] {
        v.push(case(
            &format!("{base}.tex"),
            &["-s", "-o=+-default"],
            &format!("{base}-default.tex"),
        ));
    }
    v.push(case(
        "noindentblock1.tex",
        &["-s", "-o=+-default"],
        "noindentblock1-default.tex",
    ));
    v.push(case(
        "noindentblock1.tex",
        &["-s", "-l=noindent1", "-o=+-mod1"],
        "noindentblock1-mod1.tex",
    ));
    v.push(case(
        "href.tex",
        &["-s", "-m", "-l=href1", "-o=+-mod1"],
        "href-mod1.tex",
    ));
    v.push(case(
        "href.tex",
        &["-s", "-m", "-l=href3", "-o=+-mod3", "-g=one.log"],
        "href-mod3.tex",
    ));
    v.push(case(
        "href.tex",
        &["-s", "-m", "-l=href4", "-o=+-mod4", "-g=two.log"],
        "href-mod4.tex",
    ));
    v.push(case(
        "href.tex",
        &["-s", "-m", "-l=href5", "-o=+-mod5"],
        "href-mod5.tex",
    ));
    v.push(case(
        "href.tex",
        &["-s", "-m", "-l=href6", "-o=+-mod6"],
        "href-mod6.tex",
    ));
    v.push(case(
        "issue-616.tex",
        &["-s", "-r", "-l=issue-616.yaml", "-o=+-mod1"],
        "issue-616-mod1.tex",
    ));
    v.push(case(
        "issue-602.tex",
        &["-s", "-m", "-t", "-l=issue-602.yaml", "-o=+-mod1"],
        "issue-602-mod1.tex",
    ));

    v
}

/// 已无待修用例：33 条 `-end-mod*` / `-mod4` 缺口已全部修好
/// （`VerbatimEndFinishesWithLineBreak`、`replacementText` 后缀重复拼接、
/// `horizontalTrailingSpace` 残留、`noIndentBlock` 的 name-as-regex 等，
/// 详见 doc/pending-work.md §5A.4）。
const PENDING: &[&str] = &[];

fn run_ported_cases(pending: bool) {
    for (input, args, expected) in ported_cases() {
        if PENDING.contains(&expected.as_str()) == pending {
            check_owned("verbatim", &input, &args, &expected);
        }
    }
    report();
}

#[test]
fn verbatim_ported() {
    run_ported_cases(false);
}

#[test]
fn verbatim_pending() {
    run_ported_cases(true);
}

// ---------------------------------------------------------------------------
// 本轮搬运前已有的两条断言（对应脚本 L8 与 L10）
// ---------------------------------------------------------------------------

// verbatim 内容不被改动（不动点）
#[test]
fn preamble_overwrite() {
    check_overwrite("verbatim", "verbatim-preamble.tex", &["-s"]);
}

#[test]
fn preamble1_mod1() {
    check(
        "verbatim",
        "verbatim-preamble1.tex",
        &["-s", "-o=+-mod1"],
        "verbatim-preamble1-mod1.tex",
    );
}

/// L87：`latexindent.pl -s -m -t -l issue-602.yaml issue-602.tex -o=+-mod1`
/// 的 `egrep -i 'found:' indent.log > issue-602.txt`。
#[test]
fn verbatim_logs() {
    common::check_log_lines(
        "verbatim",
        "issue-602.tex",
        &["-s", "-m", "-t", "-l=issue-602.yaml", "-o=+-mod1"],
        "found:",
        "issue-602.txt",
    );
}
