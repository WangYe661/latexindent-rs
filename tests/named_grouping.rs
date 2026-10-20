//! 命名分组括号（namedGroupingBracesBrackets）测试。
//!
//! 上游脚本：`tests/cases/namedGroupingBracesBrackets/named-grouping-test-cases.sh`（唯一 .sh）。
//! 口径：`loopmax=32`、未设 `verbatimTest` ⇒ `loopmin=1` ⇒ 循环 `i = 1..=32`，
//! 循环体 2 次调用（脚本第 22、23 行）⇒ 上游调用总数 = 静态 25 + 循环 32×2 = **89**。
//!
//! 注意：本组输入含 `.bib`（`contributors.bib` / `contributors-remove-line-breaks.bib`），
//! `-o=+-xxx` 的产物**保留输入的扩展名**，即 `<输入基名>-xxx.bib`（不是 `.bib.tex`）。
//!
//! 本文件结构：前 4 个函数为原有搬运（15 条断言）；其后为本次 T1 补齐的 74 条缺口，
//! 每条缺口都有一条断言——与基准一致的正常执行，不一致的进 `named_grouping_pending`（`#[ignore]`）。
mod common;
use common::{check, check_owned, report};

const DIR: &str = "namedGroupingBracesBrackets";

/// 已无待修用例（2026-10-18 全部转正，见 doc/pending-work.md §5A.8）。
const PENDING: &[&str] = &[];

/// 某项是否属于待修集合。
fn is_pending(name: &str) -> bool {
    PENDING.contains(&name)
}

fn owned(args: &[&str]) -> Vec<String> {
    args.iter().map(|s| s.to_string()).collect()
}

// ---------------------------------------------------------------------------
// 原有搬运（15 条断言）
// ---------------------------------------------------------------------------

#[test]
fn contributors_default() {
    check(
        DIR,
        "contributors.bib",
        &["-s", "-l=online.yaml", "-o=+-default"],
        "contributors-default.bib",
    );
}

#[test]
fn name_start_finish_line_breaks() {
    check(
        DIR,
        "contributors-remove-line-breaks.bib",
        &[
            "-m",
            "-s",
            "-t",
            "-o=contributors-vo-nameStartsOnOwnLine",
            "-l=NameStartsOnOwnLineYes,online",
        ],
        "contributors-vo-nameStartsOnOwnLine.bib",
    );
    check(
        DIR,
        "contributors-remove-line-breaks.bib",
        &[
            "-m",
            "-s",
            "-o=contributors-vo-nameStartsOnOwnLine-no",
            "-l=NameStartsOnOwnLineNo,../commands/unprotect-blank-lines,online",
        ],
        "contributors-vo-nameStartsOnOwnLine-no.bib",
    );
    check(
        DIR,
        "contributors-remove-line-breaks.bib",
        &[
            "-m",
            "-s",
            "-o=+-vo-name-finishes-lb-mod1",
            "-l=mand-args-Vo-mod1,../commands/unprotect-blank-lines,NameFinishesWithLineBreakYes,online",
        ],
        "contributors-remove-line-breaks-vo-name-finishes-lb-mod1.bib",
    );
    check(
        DIR,
        "contributors-remove-line-breaks.bib",
        &[
            "-m",
            "-s",
            "-o=+-vo-name-finishes-lb-mod2",
            "-l=mand-args-Vo-mod2,../commands/unprotect-blank-lines,NameFinishesWithLineBreakNo,online",
        ],
        "contributors-remove-line-breaks-vo-name-finishes-lb-mod2.bib",
    );
}

#[test]
fn special_characters() {
    check(
        DIR,
        "special-characters.tex",
        &[
            "-s",
            "-m",
            "-l=mand-args-mod1,../filecontents/indentPreambleYes",
            "-y=lookForAlignDelims:array:0",
            "-o=+-mod1",
        ],
        "special-characters-mod1.tex",
    );
    check(
        DIR,
        "special-characters-minimal.tex",
        &["-s", "-o=+-default"],
        "special-characters-minimal-default.tex",
    );
    check(
        DIR,
        "special-characters-minimal-blank-lines.tex",
        &["-s", "-o=+-default", "-g=one.log"],
        "special-characters-minimal-blank-lines-default.tex",
    );
}

#[test]
fn tikz_and_issues() {
    check(
        DIR,
        "tikz3.tex",
        &["-s", "-o=+-default"],
        "tikz3-default.tex",
    );
    check(
        DIR,
        "tikz4.tex",
        &["-s", "-o=+-default", "-l=../texexchange/indentPreamble"],
        "tikz4-default.tex",
    );
    check(
        DIR,
        "issue-501.tex",
        &["-s", "-o=+-mod1"],
        "issue-501-mod1.tex",
    );
    check(
        DIR,
        "issue-501.tex",
        &["-s", "-l=issue-501", "-o=+-mod2"],
        "issue-501-mod2.tex",
    );
    check(
        DIR,
        "issue-544.tex",
        &["-s", "-l=issue-544", "-o=+-mod1"],
        "issue-544-mod1.tex",
    );
    check(
        DIR,
        "issue-565.tex",
        &["-s", "-l=issue-565", "-o=+-mod1"],
        "issue-565-mod1.tex",
    );
    check(
        DIR,
        "issue-653.tex",
        &["-s", "-t", "-o=+-mod1"],
        "issue-653-mod1.tex",
    );
}

// ---------------------------------------------------------------------------
// T1 补齐（74 条缺口）
// ---------------------------------------------------------------------------

/// 循环前/后的静态缺口 10 条：脚本第 12、13、28、36、38、43、46、47、52、60 行。
fn gaps_static(only_pending: bool) {
    let cases: &[(&str, &[&str], &str)] = &[
        // L12
        (
            "contributors-remove-line-breaks.bib",
            &[
                "-m",
                "-s",
                "-o=+-vo-name-finishes-lb-mod3",
                "-l=mand-args-Vo-mod1,../commands/unprotect-blank-lines,NameFinishesWithLineBreakNo,online",
            ],
            "contributors-remove-line-breaks-vo-name-finishes-lb-mod3.bib",
        ),
        // L13
        (
            "contributors-remove-line-breaks.bib",
            &[
                "-m",
                "-s",
                "-o=+-vo-name-finishes-lb-mod4",
                "-l=mand-args-Vo-mod2,../commands/unprotect-blank-lines,NameFinishesWithLineBreakYes,online",
            ],
            "contributors-remove-line-breaks-vo-name-finishes-lb-mod4.bib",
        ),
        // L28（`-y` 值内冒号后的空格照抄）
        (
            "contributors-remove-line-breaks.bib",
            &[
                "-m",
                "-s",
                "-o=+-vo-mod5-no-remove",
                "-l=mand-args-Vo-mod5,../commands/unprotect-blank-lines,online",
                "-y=removeTrailingWhitespace:beforeProcessing: 0",
            ],
            "contributors-remove-line-breaks-vo-mod5-no-remove.bib",
        ),
        // L36
        (
            "special-characters-minimal-blank-lines.tex",
            &["-t", "-s", "-m", "-o=+-m-switch", "-l=noCondenseBlankLines"],
            "special-characters-minimal-blank-lines-m-switch.tex",
        ),
        // L38
        (
            "special-characters-minimal-blank-lines.tex",
            &["-s", "-m", "-o=+-m-switch-condense"],
            "special-characters-minimal-blank-lines-m-switch-condense.tex",
        ),
        // L43
        (
            "tikz4.tex",
            &[
                "-s",
                "-o=+-no-add-global",
                "-l=../texexchange/indentPreamble,noAddGlobNamed",
            ],
            "tikz4-no-add-global.tex",
        ),
        // L46–L47
        (
            "issue241.tex",
            &["-s", "-l=issue241", "-o=+-mod1"],
            "issue241-mod1.tex",
        ),
        (
            "issue241.tex",
            &["-s", "-l=issue241a", "-o=+-mod2"],
            "issue241-mod2.tex",
        ),
        // L52（`-y` 内 `(?:\\\\` 是 shell 双引号转义后的两个字面反斜杠）
        (
            "issue-501a.tex",
            &[
                "-s",
                "-m",
                r#"-y=modifyLineBreaks:textWrapOptions:columns:25,modifyLineBreaks:textWrapOptions:blocksBeginWith:other:(?:\\),noAdditionalIndentGlobal:namedGroupingBracesBrackets:1"#,
                "-o=+-mod2",
            ],
            "issue-501a-mod2.tex",
        ),
        // L60
        (
            "issue-653a.tex",
            &["-s", "-t", "-o=+-mod1"],
            "issue-653a-mod1.tex",
        ),
    ];
    for (input, args, expected) in cases {
        if is_pending(expected) != only_pending {
            continue;
        }
        check_owned(DIR, input, &owned(args), expected);
    }
}

#[test]
fn named_grouping_static() {
    gaps_static(false);
    report();
}

/// L22：`contributors.bib`，i = 1..=32（32 条；产物扩展名为 `.bib`）。
fn gaps_contributors_vo(only_pending: bool) {
    for i in 1..=32 {
        let expected = format!("contributors-vo-mod{i}.bib");
        if is_pending(&expected) != only_pending {
            continue;
        }
        check_owned(
            DIR,
            "contributors.bib",
            &owned(&[
                "-m",
                "-s",
                &format!("-o=+-vo-mod{i}"),
                &format!("-l=mand-args-Vo-mod{i},online"),
            ]),
            &expected,
        );
    }
}

#[test]
fn named_grouping_contributors_vo() {
    gaps_contributors_vo(false);
    report();
}

/// L23：`contributors-remove-line-breaks.bib`（`-t`），i = 1..=32（32 条）。
fn gaps_remove_line_breaks_vo(only_pending: bool) {
    for i in 1..=32 {
        let expected = format!("contributors-remove-line-breaks-vo-mod{i}.bib");
        if is_pending(&expected) != only_pending {
            continue;
        }
        check_owned(
            DIR,
            "contributors-remove-line-breaks.bib",
            &owned(&[
                "-t",
                "-m",
                "-s",
                &format!("-o=+-vo-mod{i}"),
                &format!("-l=mand-args-Vo-mod{i},../commands/unprotect-blank-lines,online"),
            ]),
            &expected,
        );
    }
}

#[test]
fn named_grouping_remove_line_breaks_vo() {
    gaps_remove_line_breaks_vo(false);
    report();
}

// ---------------------------------------------------------------------------
// 日志断言（上游脚本 L37 / L58 / L61 的三处 `egrep 'found:' indent.log > *.txt`）
// ---------------------------------------------------------------------------

#[test]
fn named_grouping_logs() {
    // L37：`latexindent.pl -t -s -m special-characters-minimal-blank-lines -o=+-m-switch -l=noCondenseBlankLines`
    common::check_log_lines(
        DIR,
        "special-characters-minimal-blank-lines.tex",
        &["-t", "-s", "-m", "-o=+-m-switch", "-l=noCondenseBlankLines"],
        "found:",
        "special-characters-minimal-blank-lines-m-swith.txt",
    );
    // L58
    common::check_log_lines(
        DIR,
        "issue-653.tex",
        &["-s", "-t", "-o=+-mod1"],
        "found:",
        "issue-653.txt",
    );
    // L61
    common::check_log_lines(
        DIR,
        "issue-653a.tex",
        &["-s", "-t", "-o=+-mod1"],
        "found:",
        "issue-653a.txt",
    );
}

