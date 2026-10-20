//! filecontents / preamble 测试，覆盖 tests/cases/filecontents/filecontents-test-cases.sh 全部直接调用。
mod common;
use common::{check, check_owned, report};

#[test]
fn filecontents_basic() {
    check("filecontents", "simplest.tex", &["-s", "-o=simplest-out"], "simplest-out.tex");
    check("filecontents", "filecontents1.tex", &["-s", "-o=+-default"], "filecontents1-default.tex");
    check("filecontents", "filecontents1.tex", &["-s", "-o=+-indent-preamble", "-l=indentPreambleYes"], "filecontents1-indent-preamble.tex");
    check("filecontents", "filecontents2.tex", &["-s", "-o=filecontents2-default"], "filecontents2-default.tex");
    check("filecontents", "filecontents3.tex", &["-s", "-o=+-default", "-y=noAdditionalIndentGlobal:namedGroupingBracesBrackets:1"], "filecontents3-default.tex");
}

#[test]
fn preamble_and_theorem() {
    check("filecontents", "preamble1.tex", &["-s", "-o=preamble1-default", "-g=one.log"], "preamble1-default.tex");
    check("filecontents", "preamble1.tex", &["-s", "-o=preamble1-indent-preamble", "-l=indentPreambleYes,preambleCommandsBeforeEnvironments"], "preamble1-indent-preamble.tex");
    check("filecontents", "theorem-small.tex", &["-s", "-o=+-mod1", "-l=specialLR"], "theorem-small-mod1.tex");
    check("filecontents", "theorem.tex", &["-s", "-o=+-default", "-l=specialLR"], "theorem-default.tex");
}

#[test]
fn filecontents_indent_rules() {
    check("filecontents", "filecontents1.tex", &["-s", "-o=+-noAdditionalIndent", "-l=noAdditionalIndentGlobal,indentPreambleYes"], "filecontents1-noAdditionalIndent.tex");
    check("filecontents", "filecontents1.tex", &["-s", "-o=+-indentRulesGlobal", "-l=indentRulesGlobal,indentPreambleYes"], "filecontents1-indentRulesGlobal.tex");
}

/// T1 搬运补齐：上游 filecontents-test-cases.sh 中此前未搬成断言的 12 条调用
/// （filecontents2/3/theorem 的 indent-preamble 变体 + filecontents4/5/6 + headings-preamble-verbatim 三组）。
const GAP_CASES: &[(&str, &[&str], &str)] = &[
    (
        "filecontents2.tex",
        &["-s", "-o", "filecontents2-indent-preamble", "-l=indentPreambleYes"],
        "filecontents2-indent-preamble.tex",
    ),
    (
        "filecontents3.tex",
        &[
            "-s",
            "-o=+-indent-preamble",
            "-l=indentPreambleYes",
            "-y=noAdditionalIndentGlobal:namedGroupingBracesBrackets:1",
        ],
        "filecontents3-indent-preamble.tex",
    ),
    (
        "theorem.tex",
        &[
            "-s",
            "-o=+-indent-preamble",
            "-l=indentPreambleYes,preambleCommandsBeforeEnvironments,specialLR",
        ],
        "theorem-indent-preamble.tex",
    ),
    ("filecontents4.tex", &["-s", "-o=+-default"], "filecontents4-default.tex"),
    (
        "filecontents4.tex",
        &["-s", "-o=+-indent-preamble", "-l=indentPreambleYes"],
        "filecontents4-indent-preamble.tex",
    ),
    ("filecontents5.tex", &["-s", "-o=+-default"], "filecontents5-default.tex"),
    (
        "filecontents5.tex",
        &["-s", "-o=+-indent-preamble", "-l=indentPreambleYes"],
        "filecontents5-indent-preamble.tex",
    ),
    ("filecontents6.tex", &["-s", "-o=+-default"], "filecontents6-default.tex"),
    (
        "filecontents6.tex",
        &["-s", "-o=+-indent-preamble", "-l=indentPreambleYes"],
        "filecontents6-indent-preamble.tex",
    ),
    (
        "headings-preamble-verbatim.tex",
        &["-s", "-o=+-default"],
        "headings-preamble-verbatim-default.tex",
    ),
    (
        "headings-preamble-verbatim.tex",
        &["-s", "-o=+-indent-preamble", "-l=indentPreambleYes"],
        "headings-preamble-verbatim-indent-preamble.tex",
    ),
    (
        "headings-preamble-verbatim.tex",
        &[
            "-s",
            "-o=+-indent-preamble-with-headings",
            "-l=indentPreambleYes,levels1",
        ],
        "headings-preamble-verbatim-indent-preamble-with-headings.tex",
    ),
];

/// 与基准不一致、待修的产物名（只搬运未修码，故标 #[ignore]）。
/// 已无待修用例：`theorem-indent-preamble` 随「参数体循环照抄 Perl」修好（§5A.9）。
const GAP_PENDING: &[&str] = &[];

fn run_gap_cases(pending_only: bool) {
    for (input, args, expected) in GAP_CASES {
        if GAP_PENDING.contains(expected) != pending_only {
            continue;
        }
        check_owned(
            "filecontents",
            input,
            &args.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
            expected,
        );
    }
    report();
}

#[test]
fn filecontents_gaps() {
    run_gap_cases(false);
}

