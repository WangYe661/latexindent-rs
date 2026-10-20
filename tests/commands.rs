//! 命令（command）及其 可选/必选参数 的缩进/换行测试，覆盖 tests/cases/commands/commands-test-cases.sh。
//! 基准取自 tests/cases/commands 已提交的 latexindent.pl 结果。
mod common;
use common::{check, check_log_lines, check_overwrite, check_owned, report};

// ---- 原地覆盖（不动点）----
#[test]
fn fixed_points() {
    for f in [
        "commands-simple.tex",
        "commands-simple1.tex",
        "commands-nested.tex",
        "commands-nested-opt-nested.tex",
        "commands-nested-harder.tex",
        "commands-nested-harder1.tex",
        "commands-four-nested.tex",
        "commands-four-nested-mk1.tex",
        "commands-five-nested.tex",
        "commands-five-nested-mk1.tex",
        "commands-six-nested.tex",
        "commands-six-nested-mk1.tex",
        "trailingComments.tex",
        "bracketTest.tex",
    ] {
        check_overwrite("commands", f, &["-s"]);
    }
}

// ---- -y=defaultIndent 两空格缩进 ----
#[test]
fn indent_two_spaces() {
    for base in [
        "commands-simple",
        "commands-simple1",
        "commands-nested",
        "commands-nested-opt-nested",
        "commands-nested-harder",
        "commands-nested-harder1",
        "commands-four-nested",
        "commands-five-nested",
        "commands-five-nested-mk1",
    ] {
        check(
            "commands",
            &format!("{base}.tex"),
            &["-s", "-o=+-mod1", "-y=defaultIndent:'  '"],
            &format!("{base}-mod1.tex"),
        );
    }
}

#[test]
fn at_end_of_file() {
    check(
        "commands",
        "command-at-end-of-file.tex",
        &["-s", "-o=+-mod1"],
        "command-at-end-of-file-mod1.tex",
    );
    check(
        "commands",
        "command-at-end-of-file.tex",
        &["-s", "-m", "-o=+-mod2"],
        "command-at-end-of-file-mod2.tex",
    );
}

// ---- noAdditionalIndent / 全局缩进规则 ----
#[test]
fn no_additional_indent() {
    let base = "commands-six-nested.tex";
    check(
        "commands",
        base,
        &[
            "-s",
            "-l=../opt-args/opt-args-remove-all.yaml,noAdditionalIndent1.yaml",
            "-o=+-NAD1.tex",
        ],
        "commands-six-nested-NAD1.tex",
    );
    check(
        "commands",
        base,
        &[
            "-s",
            "-l=../opt-args/opt-args-remove-all.yaml,noAdditionalIndent2.yaml",
            "-o=+-NAD2.tex",
        ],
        "commands-six-nested-NAD2.tex",
    );
    check(
        "commands",
        base,
        &[
            "-s",
            "-l=../opt-args/opt-args-remove-all.yaml,noAdditionalIndent3.yaml",
            "-o=+-NAD3.tex",
        ],
        "commands-six-nested-NAD3.tex",
    );
    check("commands", base, &["-s", "-l=../opt-args/opt-args-remove-all.yaml,noAdditionalIndent1.yaml,noAdditionalIndent2.yaml", "-o=+-NAD4.tex"], "commands-six-nested-NAD4.tex");
    check(
        "commands",
        base,
        &["-s", "-o=+-global.tex", "-l=noAdditionalIndentGlobal.yaml"],
        "commands-six-nested-global.tex",
    );
    check(
        "commands",
        "commands-simple-more-text.tex",
        &["-s", "-o=+-not-global.tex"],
        "commands-simple-more-text-not-global.tex",
    );
    check(
        "commands",
        "commands-simple-more-text.tex",
        &["-s", "-o=+-global.tex", "-l=noAdditionalIndentGlobal.yaml"],
        "commands-simple-more-text-global.tex",
    );
    check(
        "commands",
        "commands-simple-more-text.tex",
        &[
            "-s",
            "-o=+-indent-rules-global.tex",
            "-l=../opt-args/opt-args-remove-all.yaml,indentRulesGlobal.yaml",
        ],
        "commands-simple-more-text-indent-rules-global.tex",
    );
}

#[test]
fn just_one_command_mod1() {
    check(
        "commands",
        "just-one-command.tex",
        &["-m", "-s", "-o=+-mod1", "-l=command-name-not-finishes-with-line-break,../opt-args/opt-args-remove-all,mand-args-mod1"],
        "just-one-command-mod1.tex",
    );
}

// ---- 循环1：i=1..32，加/删换行 ----
#[test]
fn loop_add_remove_line_breaks() {
    for i in 1..=32 {
        let rm = format!("-l=../opt-args/opt-args-remove-all,mand-args-mod{i}");
        check_owned(
            "commands",
            "just-one-command.tex",
            &["-m".into(), "-s".into(), format!("-o=+-mod{i}"), rm.clone()],
            &format!("just-one-command-mod{i}.tex"),
        );
        check_owned(
            "commands",
            "commands-one-line.tex",
            &["-m".into(), "-s".into(), format!("-o=+-mod{i}"), rm.clone()],
            &format!("commands-one-line-mod{i}.tex"),
        );
        check_owned(
            "commands",
            "commands-one-line.tex",
            &[
                "-m".into(),
                "-s".into(),
                format!("-o=+-noAdditionalIndentGlobal-mod{i}"),
                format!("{rm},noAdditionalIndentGlobal"),
            ],
            &format!("commands-one-line-noAdditionalIndentGlobal-mod{i}.tex"),
        );
        check_owned(
            "commands",
            "commands-one-line-nested-simple.tex",
            &["-m".into(), "-s".into(), format!("-o=+-mod{i}"), rm.clone()],
            &format!("commands-one-line-nested-simple-mod{i}.tex"),
        );
        check_owned(
            "commands",
            "commands-one-line-nested.tex",
            &["-m".into(), "-s".into(), format!("-o=+-mod{i}"), rm.clone()],
            &format!("commands-one-line-nested-mod{i}.tex"),
        );
        check_owned(
            "commands",
            "commands-one-line-nested.tex",
            &[
                "-m".into(),
                "-s".into(),
                format!("-o=+-noAdditionalIndentGlobal-mod{i}"),
                format!("{rm},noAdditionalIndentGlobal"),
            ],
            &format!("commands-one-line-nested-noAdditionalIndentGlobal-mod{i}.tex"),
        );
        check_owned(
            "commands",
            "com-rm-lb.tex",
            &["-s".into(), "-m".into(), format!("-o=+-mod{i}"), rm.clone()],
            &format!("com-rm-lb-mod{i}.tex"),
        );
        check_owned(
            "commands",
            "com-rm-lb.tex",
            &[
                "-s".into(),
                "-m".into(),
                format!("-o=+-unprotect-mod{i}"),
                format!("{rm},unprotect-blank-lines,noChangeCommandBody"),
            ],
            &format!("com-rm-lb-unprotect-mod{i}.tex"),
        );
        check_owned(
            "commands",
            "com-rm-lb.tex",
            &[
                "-s".into(),
                "-m".into(),
                format!("-o=+-unprotect-no-condense-mod{i}"),
                format!("{rm},unprotect-blank-lines,noCondenseMultipleLines,noChangeCommandBody"),
            ],
            &format!("com-rm-lb-unprotect-no-condense-mod{i}.tex"),
        );
        check_owned(
            "commands",
            "com-rm-lb.tex",
            &[
                "-s".into(),
                "-m".into(),
                format!("-o=+-noAdditionalGlobal-mod{i}"),
                format!("{rm},noAdditionalIndentGlobal,unprotect-blank-lines,noChangeCommandBody"),
            ],
            &format!("com-rm-lb-noAdditionalGlobal-mod{i}.tex"),
        );
        check_owned(
            "commands",
            "com-rm-lb.tex",
            &[
                "-s".into(),
                "-m".into(),
                format!("-o=+-noAdditionalGlobal-changeCommandBody-mod{i}"),
                format!("{rm},noAdditionalIndentGlobal,unprotect-blank-lines,ChangeCommandBody"),
            ],
            &format!("com-rm-lb-noAdditionalGlobal-changeCommandBody-mod{i}.tex"),
        );
        check_owned(
            "commands",
            "commands-nested-multiple.tex",
            &["-m".into(), "-s".into(), format!("-o=+-mod{i}"), rm.clone()],
            &format!("commands-nested-multiple-mod{i}.tex"),
        );
        check_owned(
            "commands",
            "commands-nested-multiple.tex",
            &[
                "-m".into(),
                "-s".into(),
                format!("-o=+-textbf-mod{i}"),
                format!("{rm},textbf"),
            ],
            &format!("commands-nested-multiple-textbf-mod{i}.tex"),
        );
        check_owned(
            "commands",
            "commands-nested-multiple.tex",
            &[
                "-m".into(),
                "-s".into(),
                format!("-o=+-textbf-noAdditionalIndentGlobal-mod{i}"),
                format!("{rm},textbf,noAdditionalIndentGlobal"),
            ],
            &format!("commands-nested-multiple-textbf-noAdditionalIndentGlobal-mod{i}.tex"),
        );
        check_owned(
            "commands",
            "commands-nested-multiple.tex",
            &[
                "-m".into(),
                "-s".into(),
                format!("-o=+-textbf-mand-args-noAdditionalIndentGlobal-mod{i}"),
                format!("{rm},textbf-mand-args,noAdditionalIndentGlobal"),
            ],
            &format!(
                "commands-nested-multiple-textbf-mand-args-noAdditionalIndentGlobal-mod{i}.tex"
            ),
        );
        let fv = "command-name-not-finishes-with-line-break,../opt-args/opt-args-remove-all,../environments/env-all-on";
        check_owned(
            "commands",
            "figureValign.tex",
            &[
                "-m".into(),
                "-s".into(),
                format!("-o=+-mod{i}"),
                format!(
                    "{fv},mand-args-mod{i},figValign-yaml,../filecontents/indentPreambleYes,SAS"
                ),
            ],
            &format!("figureValign-mod{i}.tex"),
        );
        check_owned("commands", "figureValign.tex", &["-m".into(), "-s".into(), format!("-o=+-opt-mod{i}"), format!("{fv},opt-args-mod{i},figValign-yaml,makebox,../filecontents/indentPreambleYes,SAS")], &format!("figureValign-opt-mod{i}.tex"));
    }
}

// ---- 循环前后的直接调用 ----
#[test]
fn command_begin_and_braces() {
    check(
        "commands",
        "commands-nested-multiple.tex",
        &[
            "-m",
            "-s",
            "-o=+-command-mod1",
            "-l=../opt-args/opt-args-remove-all,command-begin-mod1",
        ],
        "commands-nested-multiple-command-mod1.tex",
    );
    check(
        "commands",
        "commands-nested-multiple.tex",
        &[
            "-m",
            "-s",
            "-o=+-command-mod2",
            "-l=../opt-args/opt-args-remove-all,command-begin-mod2",
        ],
        "commands-nested-multiple-command-mod2.tex",
    );
    check(
        "commands",
        "commands-nested-multiple-remove-line-breaks.tex",
        &[
            "-m",
            "-s",
            "-o=+-command-mod3",
            "-l=../opt-args/opt-args-remove-all,command-begin-mod3",
        ],
        "commands-nested-multiple-remove-line-breaks-command-mod3.tex",
    );
    check(
        "commands",
        "commands-nested-multiple-remove-line-breaks.tex",
        &[
            "-m",
            "-s",
            "-o=+-command-unprotect-mod3",
            "-l=../opt-args/opt-args-remove-all,command-begin-mod3,unprotect-blank-lines",
        ],
        "commands-nested-multiple-remove-line-breaks-command-unprotect-mod3.tex",
    );
    check(
        "commands",
        "commands-four-special-characters.tex",
        &["-s", "-o=+-default"],
        "commands-four-special-characters-default.tex",
    );
    check(
        "commands",
        "multipleBraces.tex",
        &["-s", "-o=+-mod0"],
        "multipleBraces-mod0.tex",
    );
    check(
        "commands",
        "multipleBraces.tex",
        &[
            "-s",
            "-m",
            "-o=+-mod1",
            "-l=../opt-args/opt-args-remove-all,mand-args-mod1,no-end",
        ],
        "multipleBraces-mod1.tex",
    );
    check(
        "commands",
        "multipleBraces.tex",
        &[
            "-s",
            "-m",
            "-o=+-xapptocmd-none-mod1",
            "-l=../opt-args/opt-args-remove-all,mand-args-mod1,xapptocmd-none,no-end",
        ],
        "multipleBraces-xapptocmd-none-mod1.tex",
    );
    check(
        "commands",
        "multipleBraces.tex",
        &[
            "-s",
            "-m",
            "-o=+-xapptocmd-none-pagestyle-comments-mod1",
            "-l=../opt-args/opt-args-remove-all,mand-args-mod1,xapptocmd-none,pagestyle,no-end",
        ],
        "multipleBraces-xapptocmd-none-pagestyle-comments-mod1.tex",
    );
    check(
        "commands",
        "just-one-command-multiple-trailing-comments.tex",
        &[
            "-m",
            "-s",
            "-o=+-mod17",
            "-l=../opt-args/opt-args-remove-all,mand-args-mod17",
        ],
        "just-one-command-multiple-trailing-comments-mod17.tex",
    );
    check(
        "commands",
        "com-rm-lb-tc.tex",
        &[
            "-m",
            "-s",
            "-o=+-mod5",
            "-l=../opt-args/opt-args-remove-all,mand-args-mod5",
        ],
        "com-rm-lb-tc-mod5.tex",
    );
    check(
        "commands",
        "command-with-numeric-args.tex",
        &["-s", "-o=+-default"],
        "command-with-numeric-args-default.tex",
    );
    check(
        "commands",
        "issue-239.tex",
        &["-s", "-o=+-default.tex"],
        "issue-239-default.tex",
    );
    check_overwrite("commands", "command-nest-env.tex", &["-s"]);
    check(
        "commands",
        "numbers-between-args.tex",
        &["-s", "-o=+-mod0", "-l=numbers-allowed"],
        "numbers-between-args-mod0.tex",
    );
    check(
        "commands",
        "numbers-between-args.tex",
        &[
            "-s",
            "-o=+-mod1",
            "-m",
            "-l=numbers-allowed,mand-args-mod1,../opt-args/opt-args-mod1",
        ],
        "numbers-between-args-mod1.tex",
    );
    check(
        "commands",
        "brackets1.tex",
        &["-s", "-o=+-default"],
        "brackets1-default.tex",
    );
    check(
        "commands",
        "foreach.tex",
        &["-s", "-o=+-mod1", "-l=foreach"],
        "foreach-mod1.tex",
    );
}

// ---- 循环2：i=33..48，poly-switch=3 加换行 ----
#[test]
fn loop_poly_switch3() {
    for i in 33..=48 {
        check_owned(
            "commands",
            "commands-nested-multiple.tex",
            &[
                "-m".into(),
                "-s".into(),
                format!("-o=+-mod{i}"),
                format!("-l=../opt-args/opt-args-mod{i},mand-args-mod{i}"),
            ],
            &format!("commands-nested-multiple-mod{i}.tex"),
        );
        check_owned(
            "commands",
            "commands-nested-multiple.tex",
            &[
                "-m".into(),
                "-s".into(),
                format!("-o=+-un-protect-mod{i}"),
                format!("-l=../opt-args/opt-args-mod{i},mand-args-mod{i},unprotect-blank-lines"),
            ],
            &format!("commands-nested-multiple-un-protect-mod{i}.tex"),
        );
        check_owned(
            "commands",
            "com-rm-lb.tex",
            &[
                "-s".into(),
                "-m".into(),
                format!("-o=+-mod{i}"),
                format!("-l=../opt-args/opt-args-mod{i},mand-args-mod{i},unprotect-blank-lines"),
            ],
            &format!("com-rm-lb-mod{i}.tex"),
        );
    }
}

/// 一条上游 shell 调用：`latexindent.pl <args...> <input>` → 产物 `expected`。
/// `overwrite` 表示 `-w` 不动点（产物即输入文件自身）。
struct CCase {
    input: &'static str,
    args: &'static [&'static str],
    expected: &'static str,
    overwrite: bool,
}

/// 脚本尾部（第 165–224 行）尚未搬入的 26 条非循环调用；两个循环已 100% 覆盖。
const TAIL_CASES: &[CCase] = &[
    // issue 123
    CCase { input: "issue-123.tex", args: &["-s", "-o=+-default.tex"], expected: "issue-123-default.tex", overwrite: false },
    // ifnextchar
    CCase { input: "ifnextchar.tex", args: &["-s", "-o=+-default"], expected: "ifnextchar-default.tex", overwrite: false },
    CCase { input: "ifnextchar.tex", args: &["-s", "-o=+-mod1", "-l=com-name-special1.yaml"], expected: "ifnextchar-mod1.tex", overwrite: false },
    // issue 379（另有日志基准 issue-379-mod1.txt，见 `tail_log_pending`）
    CCase { input: "issue-379.tex", args: &["-s", "-o=+-default.tex"], expected: "issue-379-default.tex", overwrite: false },
    CCase { input: "issue-379.tex", args: &["-s", "-t", "-l", "issue-379.yaml", "-o=+-mod1.tex"], expected: "issue-379-mod1.tex", overwrite: false },
    CCase { input: "commands-keys.tex", args: &["-s", "-l", "commands-keys", "-o=+-mod1"], expected: "commands-keys-mod1.tex", overwrite: false },
    CCase { input: "stars-from-documentation.tex", args: &["-s", "-o=+-default"], expected: "stars-from-documentation-default.tex", overwrite: false },
    // testcls-small.cls（另有日志基准 testcls.txt，见 `tail_log_pending`）
    CCase { input: "testcls-small.cls", args: &["-s", "-t", "-o=+-default.cls"], expected: "testcls-small-default.cls", overwrite: false },
    // ifelsefiONE.tex（另有日志基准 ifelsefiONE.txt，见 `tail_log_pending`）
    CCase { input: "ifelsefiONE.tex", args: &["-s", "-w", "-t"], expected: "ifelsefiONE.tex", overwrite: true },
    CCase { input: "testcls.cls", args: &["-s", "-w"], expected: "testcls.cls", overwrite: true },
    CCase { input: "unnamed-small.tex", args: &["-s", "-o=+-mod1"], expected: "unnamed-small-mod1.tex", overwrite: false },
    CCase { input: "bigTest.tex", args: &["-s", "-o=+-default.tex"], expected: "bigTest-default.tex", overwrite: false },
    // github-issue-35（-no-at / -no-at3 另有日志基准，见 `tail_log_pending`）
    CCase { input: "github-issue-35.tex", args: &["-t", "-s", "-o=+-default"], expected: "github-issue-35-default.tex", overwrite: false },
    CCase { input: "github-issue-35.tex", args: &["-t", "-s", "-o=+-no-at", "-l", "no-at-between-args.yaml"], expected: "github-issue-35-no-at.tex", overwrite: false },
    CCase { input: "github-issue-35.tex", args: &["-s", "-o=+-no-at1", "-l", "no-at-between-args1.yaml"], expected: "github-issue-35-no-at1.tex", overwrite: false },
    CCase { input: "github-issue-35.tex", args: &["-s", "-o=+-no-at2", "-l", "no-at-between-args2.yaml"], expected: "github-issue-35-no-at2.tex", overwrite: false },
    CCase { input: "github-issue-35.tex", args: &["-t", "-s", "-o=+-no-at3", "-l", "no-at-between-args3.yaml"], expected: "github-issue-35-no-at3.tex", overwrite: false },
    CCase { input: "figureValign.tex", args: &["-w", "-s"], expected: "figureValign.tex", overwrite: true },
    CCase { input: "pstricks1.tex", args: &["-s", "-o=+-default", "-l=../texexchange/indentPreamble"], expected: "pstricks1-default.tex", overwrite: false },
    // sub-super-scripts（另有日志基准 sub-super-scripts.txt，见 `tail_log_pending`）
    CCase { input: "sub-super-scripts.tex", args: &["-s", "-t", "-o=+-default"], expected: "sub-super-scripts-default.tex", overwrite: false },
    CCase { input: "sub-super-scripts.tex", args: &["-s", "-o=+-mod5", "-m", "-l=../specials/special-mod5"], expected: "sub-super-scripts-mod5.tex", overwrite: false },
    CCase { input: "sub-super-scripts.tex", args: &["-s", "-o=+-mod6", "-m", "-l=../specials/special-mod5,mand-args-mod1"], expected: "sub-super-scripts-mod6.tex", overwrite: false },
    CCase { input: "sub-super-scripts.tex", args: &["-s", "-o=+-mod55", "-m", "-l=../mand-args/mand-args-mod5,../specials/special-mod5"], expected: "sub-super-scripts-mod55.tex", overwrite: false },
    CCase { input: "commands-four-special-characters.tex", args: &["-o=+-mod1", "-s", "-l=fine-tuning-args1"], expected: "commands-four-special-characters-mod1.tex", overwrite: false },
    CCase { input: "issue-605.tex", args: &["-s", "-l", "issue-605.yaml", "-o=+-mod1.tex"], expected: "issue-605-mod1.tex", overwrite: false },
    // issue-620（另有日志基准 issue-620.txt，见 `tail_log_pending`）
    CCase { input: "issue-620.tex", args: &["-s", "-t", "-o=+-mod1"], expected: "issue-620-mod1.tex", overwrite: false },
];

/// 与基准不一致、暂挂起的产物名（只搬运未修码）。从 `TAIL_CASES` 中过滤出来单独 ignore。
/// 已无待修用例：`ifelsefiONE.tex` 的 pcre2 match limit 早已修好
/// （参数前缀 trivia 括号组前置 + 所有格星号，见 patterns.rs），归入常规测试。
const PENDING: &[&str] = &[];

fn run_tail_case(c: &CCase) {
    let mut args = c.args.iter().map(|s| s.to_string()).collect::<Vec<_>>();
    if c.overwrite && !args.iter().any(|a| a == "-w" || a == "-wd") {
        args.push("-w".into());
    }
    check_owned("commands", c.input, &args, c.expected);
}

/// 补齐的脚本尾部调用（26 条）。收集全部失败后一次性 report。
#[test]
fn tail_cases() {
    for c in TAIL_CASES {
        if PENDING.contains(&c.expected) {
            continue;
        }
        run_tail_case(c);
    }
    report();
}

/// 脚本尾部 8 处 `egrep 'found:' indent.log > *.txt` 日志基准中的 7 条（实测全部通过）。
#[test]
fn tail_logs() {
    check_log_lines(
        "commands",
        "issue-379.tex",
        &["-s", "-t", "-l", "issue-379.yaml", "-o=+-mod1.tex"],
        "found:",
        "issue-379-mod1.txt",
    );
    check_log_lines(
        "commands",
        "testcls-small.cls",
        &["-s", "-t", "-o=+-default.cls"],
        "found:",
        "testcls.txt",
    );
    check_log_lines(
        "commands",
        "github-issue-35.tex",
        &["-t", "-s", "-o=+-default"],
        "found:",
        "github-issue-35-default.txt",
    );
    check_log_lines(
        "commands",
        "github-issue-35.tex",
        &["-t", "-s", "-o=+-no-at", "-l", "no-at-between-args.yaml"],
        "found:",
        "github-issue-35-no-at.txt",
    );
    check_log_lines(
        "commands",
        "github-issue-35.tex",
        &["-t", "-s", "-o=+-no-at3", "-l", "no-at-between-args3.yaml"],
        "found:",
        "github-issue-35-no-at3.txt",
    );
    check_log_lines(
        "commands",
        "sub-super-scripts.tex",
        &["-s", "-t", "-o=+-default"],
        "found:",
        "sub-super-scripts.txt",
    );
    check_log_lines(
        "commands",
        "issue-620.tex",
        &["-s", "-t", "-o=+-mod1"],
        "found:",
        "issue-620.txt",
    );
    // pcre2 match limit 修好后（§5A.2）ifelsefiONE 可以正常产出日志。
    check_log_lines(
        "commands",
        "ifelsefiONE.tex",
        &["-s", "-w", "-t"],
        "found:",
        "ifelsefiONE.txt",
    );
}
