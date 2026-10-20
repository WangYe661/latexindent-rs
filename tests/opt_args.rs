//! 环境可选参数（opt-args）测试，覆盖 tests/cases/opt-args/opt-args-test-cases.sh。
mod common;
use common::{check, check_overwrite, check_owned, report};

const OA: &str = "opt-args-remove-all,../environments/env-all-on";

#[test]
fn before_loop() {
    check(
        "opt-args",
        "env-first-opt-args.tex",
        &["-m", "-s", "-o=+-mod0", &format!("-l={OA}")],
        "env-first-opt-args-mod0.tex",
    );
    check(
        "opt-args",
        "env-first-opt-args-rm-lb1.tex",
        &["-m", "-s", "-o=+-mod0", &format!("-l={OA}")],
        "env-first-opt-args-rm-lb1-mod0.tex",
    );
    check(
        "opt-args",
        "env-simple-opt-args.tex",
        &["-m", "-s", "-o=+-out", &format!("-l={OA}")],
        "env-simple-opt-args-out.tex",
    );
}

#[test]
fn loop_opt_args() {
    for i in 1..=16 {
        let om = format!("{OA},opt-args-mod{i}");
        check_owned(
            "opt-args",
            "env-first-opt-args.tex",
            &[
                "-m".into(),
                "-s".into(),
                format!("-o=+-mod{i}"),
                format!("-l={om}"),
            ],
            &format!("env-first-opt-args-mod{i}.tex"),
        );
        check_owned(
            "opt-args",
            "env-first-opt-args.tex",
            &[
                "-m".into(),
                "-s".into(),
                format!("-o=+-mod-supp{i}"),
                format!("-l={om},opt-args-supp"),
            ],
            &format!("env-first-opt-args-mod-supp{i}.tex"),
        );
        check_owned(
            "opt-args",
            "env-first-opt-args-rm-lb1.tex",
            &[
                "-m".into(),
                "-s".into(),
                format!("-o=+-mod{i}"),
                format!("-l={om}"),
            ],
            &format!("env-first-opt-args-rm-lb1-mod{i}.tex"),
        );
        check_owned(
            "opt-args",
            "env-first-opt-args-rm-lb1.tex",
            &[
                "-m".into(),
                "-s".into(),
                format!("-o=+-mod-supp{i}"),
                format!("-l={om},opt-args-supp"),
            ],
            &format!("env-first-opt-args-rm-lb1-mod-supp{i}.tex"),
        );
        check_owned(
            "opt-args",
            "env-first-opt-args-rm-lb2.tex",
            &[
                "-m".into(),
                "-s".into(),
                format!("-o=+-mod-supp{i}"),
                format!("-l={om},unprotect-blank-lines"),
            ],
            &format!("env-first-opt-args-rm-lb2-mod-supp{i}.tex"),
        );
        check_owned(
            "opt-args",
            "env-second-opt-args.tex",
            &[
                "-m".into(),
                format!("-l={om}"),
                "-s".into(),
                format!("-o=+-mod{i}"),
            ],
            &format!("env-second-opt-args-mod{i}.tex"),
        );
        check_owned(
            "opt-args",
            "env-second-opt-args-rm-lb1.tex",
            &[
                "-m".into(),
                format!("-l={om}"),
                "-s".into(),
                format!("-o=+-mod{i}"),
            ],
            &format!("env-second-opt-args-rm-lb1-mod{i}.tex"),
        );
        check_owned(
            "opt-args",
            "env-second-opt-args-rm-lb1.tex",
            &[
                "-m".into(),
                format!("-l={om},unprotect-blank-lines"),
                "-s".into(),
                format!("-o=+-mod-unprotect{i}"),
            ],
            &format!("env-second-opt-args-rm-lb1-mod-unprotect{i}.tex"),
        );
        check_owned(
            "opt-args",
            "env-third-opt-args-rm-lb1-tc.tex",
            &[
                "-m".into(),
                format!("-l={om}"),
                "-s".into(),
                format!("-o=+-mod{i}"),
            ],
            &format!("env-third-opt-args-rm-lb1-tc-mod{i}.tex"),
        );
        check_owned(
            "opt-args",
            "env-third-opt-args.tex",
            &[
                "-m".into(),
                format!("-l={om},addPercentAfterBegin"),
                "-s".into(),
                format!("-o=+-mod{i}"),
                "-g=other.log".into(),
            ],
            &format!("env-third-opt-args-mod{i}.tex"),
        );
    }
    report();
}

#[test]
fn loop_opt_args_condense() {
    for i in 1..=16 {
        let om = format!("{OA},opt-args-mod{i}");
        check_owned("opt-args", "env-second-opt-args-rm-lb1.tex", &["-m".into(), format!("-l={om},unprotect-blank-lines,condense-blank-lines,../ifelsefi/removeTWS-before"), "-s".into(), format!("-o=+-mod-unprotect-condense{i}")], &format!("env-second-opt-args-rm-lb1-mod-unprotect-condense{i}.tex"));
    }
    report();
}

#[test]
fn after_loop() {
    check(
        "opt-args",
        "env-third-opt-args-rm-lb1-tc.tex",
        &[
            "-m",
            "-l=opt-args-remove-all,../environments/env-all-on,opt-args-mod1,addPercentAfterBegin",
            "-s",
            "-o=+-mod1-addPercentAfterBegin",
        ],
        "env-third-opt-args-rm-lb1-tc-mod1-addPercentAfterBegin.tex",
    );
    check(
        "opt-args",
        "env-third-opt-args.tex",
        &[
            "-l=opt-args-remove-all,../environments/env-all-on,opt-args-mod1,addPercentBeforeBegin",
            "-m",
            "-s",
            "-o=+-percent-before-begin",
        ],
        "env-third-opt-args-percent-before-begin.tex",
    );
    check(
        "opt-args",
        "env-third-opt-args.tex",
        &[
            "-l=opt-args-remove-all,../environments/env-all-on,addPercentAfterEnd",
            "-m",
            "-s",
            "-o=+-percent-after-end",
        ],
        "env-third-opt-args-percent-after-end.tex",
    );
    check(
        "opt-args",
        "env-third-opt-args.tex",
        &[
            "-l=opt-args-remove-all,../environments/env-all-on,addPercentAfterBody",
            "-m",
            "-s",
            "-o=+-percent-after-body",
        ],
        "env-third-opt-args-percent-after-body.tex",
    );
    check(
        "opt-args",
        "env-first-opt-args-mod-supp1.tex",
        &[
            "-s",
            "-m",
            "-w",
            "-l=opt-args-remove-all,../environments/env-all-on,addPercentAfterBody",
            "-o=+-mod1",
        ],
        "env-first-opt-args-mod-supp1-mod1.tex",
    );
    check(
        "opt-args",
        "env-first-opt-args-more-comments.tex",
        &[
            "-s",
            "-m",
            "-l=opt-args-remove-all,../environments/env-all-on,addPercentAfterBegin",
            "-o=+-addPercentAfterBegin",
        ],
        "env-first-opt-args-more-comments-addPercentAfterBegin.tex",
    );
    check(
        "opt-args",
        "env-third-opt-args.tex",
        &[
            "-l=opt-args-remove-all,../environments/env-all-on,addPercentAll",
            "-m",
            "-s",
            "-o=+-percent-after-all",
        ],
        "env-third-opt-args-percent-after-all.tex",
    );
    check(
        "opt-args",
        "env-third-opt-args.tex",
        &[
            "-l=opt-args-remove-all,../environments/env-all-on,addPercent-noAdditionalIndent",
            "-m",
            "-s",
            "-o=+-noAddtionalIndentScalar",
        ],
        "env-third-opt-args-noAddtionalIndentScalar.tex",
    );
    check("opt-args", "env-third-opt-args.tex", &["-l=opt-args-remove-all,../environments/env-all-on,addPercent-noAdditionalIndent-opt-args-mod1", "-m", "-s", "-o=+-noAddtionalIndentHash-mod1"], "env-third-opt-args-noAddtionalIndentHash-mod1.tex");
    check("opt-args", "env-third-opt-args.tex", &["-l=opt-args-remove-all,../environments/env-all-on,addPercent-noAdditionalIndent-opt-args-mod2", "-m", "-s", "-o=+-noAddtionalIndentHash-mod2"], "env-third-opt-args-noAddtionalIndentHash-mod2.tex");
    check("opt-args", "env-third-opt-args.tex", &["-l=opt-args-remove-all,../environments/env-all-on,addPercent-noAdditionalIndent-opt-args-mod3", "-m", "-s", "-o=+-noAddtionalIndentHash-mod3"], "env-third-opt-args-noAddtionalIndentHash-mod3.tex");
    check("opt-args", "env-third-opt-args.tex", &["-l=opt-args-remove-all,../environments/env-all-on,addPercent-noAdditionalIndent-opt-args-mod4", "-m", "-s", "-o=+-noAddtionalIndentHash-mod4"], "env-third-opt-args-noAddtionalIndentHash-mod4.tex");
}

/// T1 搬运补齐：上游 opt-args-test-cases.sh 尾部 12 条非循环调用（sh:57-60、66、69、72、75-79）。
/// 第 63 行 `-w` 不动点用例单独由 `opt_args_w_gap` 覆盖。
const GAP_CASES: &[(&str, &[&str], &str)] = &[
    (
        "env-third-opt-args.tex",
        &[
            "-l=opt-args-remove-all,../environments/env-all-on,addPercent-noAdditionalIndent-opt-args-indent-rules1",
            "-m",
            "-s",
            "-o=env-third-opt-args-indent-rules1",
        ],
        "env-third-opt-args-indent-rules1.tex",
    ),
    (
        "env-third-opt-args.tex",
        &[
            "-l=opt-args-remove-all,../environments/env-all-on,addPercent-noAdditionalIndent-opt-args-indent-rules2",
            "-m",
            "-s",
            "-o=env-third-opt-args-indent-rules2",
        ],
        "env-third-opt-args-indent-rules2.tex",
    ),
    (
        "env-third-opt-args.tex",
        &[
            "-l=opt-args-remove-all,../environments/env-all-on,addPercent-noAdditionalIndent-opt-args-indent-rules3",
            "-m",
            "-s",
            "-o=env-third-opt-args-indent-rules3",
        ],
        "env-third-opt-args-indent-rules3.tex",
    ),
    (
        "env-third-opt-args.tex",
        &[
            "-l=opt-args-remove-all,../environments/env-all-on,addPercent-noAdditionalIndent-opt-args-indent-rules4",
            "-m",
            "-s",
            "-o=env-third-opt-args-indent-rules4",
        ],
        "env-third-opt-args-indent-rules4.tex",
    ),
    (
        "env-third-opt-args-mod1.tex",
        &[
            "-l=opt-args-remove-all,../environments/env-all-on,noAdditionalIndentGlobal",
            "-s",
            "-o=env-third-opt-args-mod1-global",
            "-tt",
        ],
        "env-third-opt-args-mod1-global.tex",
    ),
    (
        "env-third-opt-args-mod1.tex",
        &[
            "-l=opt-args-remove-all,../environments/env-all-on,indentRulesGlobal",
            "-s",
            "-o=env-third-opt-args-mod1-indent-rules-global",
            "-tt",
        ],
        "env-third-opt-args-mod1-indent-rules-global.tex",
    ),
    (
        "forrest.tex",
        &["-s", "-o=+-mod1", "-y=defaultIndent:' '"],
        "forrest-mod1.tex",
    ),
    (
        "issue-445.tex",
        &["-s", "-l", "issue-445", "-m", "-o=+-mod1"],
        "issue-445-mod1.tex",
    ),
    (
        "issue-445.tex",
        &["-s", "-l", "issue-445a", "-m", "-o=+-mod2"],
        "issue-445-mod2.tex",
    ),
    (
        "issue-445.tex",
        &["-s", "-l", "issue-445b", "-m", "-o=+-mod3"],
        "issue-445-mod3.tex",
    ),
    (
        "issue-445.tex",
        &["-s", "-l", "issue-445c", "-m", "-o=+-mod4"],
        "issue-445-mod4.tex",
    ),
    (
        "issue-445a.tex",
        &["-s", "-l", "issue-445", "-m", "-o=+-mod1"],
        "issue-445a-mod1.tex",
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
            "opt-args",
            input,
            &args.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
            expected,
        );
    }
    report();
}

#[test]
fn opt_args_gaps() {
    run_gap_cases(false);
}

/// 已无待修项（原缺口全部修复并并入 `GAP_CASES`）。
#[test]
fn opt_args_gaps_pending() {
    run_gap_cases(true);
}

/// sh:63 `latexindent.pl env-third-opt-args-multiple-lines -w -s`（`-w` 不动点，无 `-o`）。
#[test]
fn opt_args_w_gap() {
    check_overwrite("opt-args", "env-third-opt-args-multiple-lines.tex", &["-w", "-s"]);
}
