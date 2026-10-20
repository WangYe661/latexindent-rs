//! poly-switch 空行（poly-switch-blank-line）测试，覆盖 tests/cases/poly-switch-blank-line 全部直接调用。
//! `-m` + `-y` poly=4（空行变体）。
mod common;
use common::{check, check_owned, report};

#[test]
fn blank_line_body() {
    check("poly-switch-blank-line", "ifelsefi-simple-nested.tex", &["-s", "-m", "-y=modifyLineBreaks:ifElseFi:BodyStartsOnOwnLine:4", "-o=+-bl-mod1"], "ifelsefi-simple-nested-bl-mod1.tex");
    check("poly-switch-blank-line", "ifelsefi-simple-nested.tex", &["-s", "-m", "-y=modifyLineBreaks:ifElseFi:FiStartsOnOwnLine:4", "-o=+-bl-fi-mod1"], "ifelsefi-simple-nested-bl-fi-mod1.tex");
    check("poly-switch-blank-line", "ifelsefi-simple-nested.tex", &["-s", "-m", "-y=modifyLineBreaks:ifElseFi:FiFinishesWithLineBreak:4", "-o=+-bl-fi-fin-mod1"], "ifelsefi-simple-nested-bl-fi-fin-mod1.tex");
}

#[test]
fn blank_line_mand_args() {
    check("poly-switch-blank-line", "mycommand.tex", &["-s", "-m", "-y=modifyLineBreaks:mandatoryArguments:LCuBStartsOnOwnLine:4", "-o=+-mod4-LCuB"], "mycommand-mod4-LCuB.tex");
    check("poly-switch-blank-line", "mycommand.tex", &["-s", "-m", "-y=modifyLineBreaks:mandatoryArguments:RCuBStartsOnOwnLine:4", "-o=+-mod4-RCuB"], "mycommand-mod4-RCuB.tex");
    check("poly-switch-blank-line", "mycommand.tex", &["-s", "-m", "-y=modifyLineBreaks:optionalArguments:LSqBStartsOnOwnLine:4", "-o=+-mod4-LSqB"], "mycommand-mod4-LSqB.tex");
}

#[test]
fn blank_line_commands() {
    check("poly-switch-blank-line", "section1.tex", &["-s", "-m", "-y=modifyLineBreaks:mandatoryArguments:RCuBFinishesWithLineBreak:4", "-o=+-bl-beg-mod1"], "section1-bl-beg-mod1.tex");
    check("poly-switch-blank-line", "section1.tex", &["-s", "-m", "-y=modifyLineBreaks:commands:CommandStartsOnOwnLine:4", "-o=+-bl-beg-head-mod1"], "section1-bl-beg-head-mod1.tex");
    check("poly-switch-blank-line", "defn1.tex", &["-s", "-m", "-y=modifyLineBreaks:environments:defn:EndFinishesWithLineBreak:4", "-o=+-bl-beg-mod1"], "defn1-bl-beg-mod1.tex");
}

/// T1 搬运补齐：上游 poly-switch-blank-line-test-cases.sh 中此前未搬成断言的 16 条调用
/// （7 条以 `../environments/`、`../specials/` 为输入、产物落在本目录）。
const GAP_CASES: &[(&str, &[&str], &str)] = &[
    (
        "../environments/environments-nested-fourth.tex",
        &[
            "-s",
            "-m",
            "-y=modifyLineBreaks:environments:BodyStartsOnOwnLine:4",
            "-o=+-bl-mod1",
        ],
        "environments-nested-fourth-bl-mod1.tex",
    ),
    (
        "../specials/special1.tex",
        &[
            "-s",
            "-m",
            "-y=modifyLineBreaks:specialBeginEnd:SpecialBodyStartsOnOwnLine:4",
            "-o=+-bl-mod1",
        ],
        "special1-bl-mod1.tex",
    ),
    (
        "../environments/environments-simple.tex",
        &[
            "-s",
            "-m",
            "-y=modifyLineBreaks:environments:EndStartsOnOwnLine:4",
            "-o=+-bl-mod1",
        ],
        "environments-simple-bl-mod1.tex",
    ),
    (
        "../specials/special1.tex",
        &[
            "-s",
            "-m",
            "-y=modifyLineBreaks:specialBeginEnd:SpecialEndStartsOnOwnLine:4",
            "-o=+-bl-end-mod1",
        ],
        "special1-bl-end-mod1.tex",
    ),
    (
        "../environments/environments-simple.tex",
        &[
            "-s",
            "-m",
            "-y=modifyLineBreaks:environments:EndFinishesWithLineBreak:4",
            "-o=+-bl-fin-mod1",
        ],
        "environments-simple-bl-fin-mod1.tex",
    ),
    (
        "../specials/special1.tex",
        &[
            "-s",
            "-m",
            "-y=modifyLineBreaks:specialBeginEnd:SpecialEndFinishesWithLineBreak:4",
            "-o=+-bl-fin-mod1",
        ],
        "special1-bl-fin-mod1.tex",
    ),
    (
        "../environments/environments-simple.tex",
        &[
            "-s",
            "-m",
            "-y=modifyLineBreaks:environments:BeginStartsOnOwnLine:4",
            "-o=+-bl-beg-mod1",
        ],
        "environments-simple-bl-beg-mod1.tex",
    ),
    (
        "mycommand.tex",
        &[
            "-s",
            "-m",
            "-y=modifyLineBreaks:optionalArguments:RSqBStartsOnOwnLine:4",
            "-o=+-mod4-RSqB",
        ],
        "mycommand-mod4-RSqB.tex",
    ),
    (
        "mycommand1.tex",
        &[
            "-s",
            "-m",
            "-y=modifyLineBreaks:mandatoryArguments:LCuBStartsOnOwnLine:4",
            "-o=+-mod4-LCuB",
        ],
        "mycommand1-mod4-LCuB.tex",
    ),
    (
        "mycommand1.tex",
        &[
            "-s",
            "-m",
            "-y=modifyLineBreaks:mandatoryArguments:RCuBStartsOnOwnLine:4",
            "-o=+-mod4-RCuB",
        ],
        "mycommand1-mod4-RCuB.tex",
    ),
    (
        "mycommand1.tex",
        &[
            "-s",
            "-m",
            "-y=modifyLineBreaks:optionalArguments:LSqBStartsOnOwnLine:4",
            "-o=+-mod4-LSqB",
        ],
        "mycommand1-mod4-LSqB.tex",
    ),
    (
        "mycommand1.tex",
        &[
            "-s",
            "-m",
            "-y=modifyLineBreaks:optionalArguments:RSqBStartsOnOwnLine:4",
            "-o=+-mod4-RSqB",
        ],
        "mycommand1-mod4-RSqB.tex",
    ),
    (
        "xaltsc.tex",
        &["-s", "-m", "-l=xaltsc.yaml", "-o=+-mod1"],
        "xaltsc-mod1.tex",
    ),
    (
        "xaltsc.tex",
        &["-s", "-m", "-r", "-l=xaltsc2.yaml", "-o=+-mod2"],
        "xaltsc-mod2.tex",
    ),
    (
        "issue-492.tex",
        &["-s", "-m", "-l", "issue-492", "-o", "+-mod1"],
        "issue-492-mod1.tex",
    ),
    (
        "issue-649.tex",
        &["-s", "-m", "-l", "issue-649", "-o", "+-mod1"],
        "issue-649-mod1.tex",
    ),
];

/// 与基准不一致、待修的产物名（只搬运未修码，故标 #[ignore]）。
const GAP_PENDING: &[&str] = &[];

fn run_gap_cases(pending_only: bool) {
    for (input, args, expected) in GAP_CASES {
        if GAP_PENDING.contains(expected) != pending_only {
            continue;
        }
        check_owned(
            "poly-switch-blank-line",
            input,
            &args.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
            expected,
        );
    }
    report();
}

#[test]
fn blank_line_gaps() {
    run_gap_cases(false);
}

/// 已无待修项（原缺口全部修复并并入 `GAP_CASES`）。
#[test]
fn blank_line_gaps_pending() {
    run_gap_cases(true);
}
