//! key=value 括号（keyEqualsValueBraces）测试，覆盖 tests/cases/keyEqualsValueBraces/key-equals-values-test-cases.sh。
//!
//! 上游脚本口径：`loopmax=48`、`loopmin=1`（`verbatimTest` 未设置，common.sh 的 `-1` 分支不生效）
//! ⇒ 循环体 `i = 1..=48`；第 53 行 `[[ $i -lt 33 ]]` 条件 ⇒ `contributors.bib` 族只有 `i = 1..=32`。
//! 上游调用总数 = S1(14) + S2(11×48 + 32 = 560) + S3(25) = 599。
//!
//! 本文件结构：前 4 个函数为原有搬运（14 条断言）；其后为本次 T1 补齐的 585 条缺口，
//! 每条缺口都有一条断言——与基准一致的正常执行，不一致的进 `key_equals_pending`（`#[ignore]`）。
mod common;
use common::{check, check_owned, check_overwrite, report};

const DIR: &str = "keyEqualsValueBraces";

/// 与上游 Perl 基准不一致、待修的产物清单（只搬运、未修码）。
/// 这些项在下面的搬运函数里被跳过，改由 `key_equals_pending` 以 `#[ignore]` 承载。
/// 已无待修用例：`issue-378-mod1` 的 pcre2 match limit 随「精确 / 所有格双正则 +
/// 失败回退」修好（§5A.9）。
const PENDING: &[&str] = &[];

/// 某项是否属于待修集合。
fn is_pending(name: &str) -> bool {
    PENDING.contains(&name)
}

fn owned(args: &[&str]) -> Vec<String> {
    args.iter().map(|s| s.to_string()).collect()
}

// ---------------------------------------------------------------------------
// 原有搬运（14 条断言）
// ---------------------------------------------------------------------------

#[test]
fn key_equals_default() {
    check("keyEqualsValueBraces", "keyEqualsValueFirst.tex", &["-s", "-o=+-default"], "keyEqualsValueFirst-default.tex");
    check("keyEqualsValueBraces", "braceTestsmall.tex", &["-s", "-o=+-default"], "braceTestsmall-default.tex");
    check("keyEqualsValueBraces", "braceTest.tex", &["-s", "-o=+-default"], "braceTest-default.tex");
}

#[test]
fn key_no_additional_indent() {
    check("keyEqualsValueBraces", "keyEqualsValueFirst.tex", &["-s", "-l=noAdditionalKey", "-o=+-noAdditional-key-global"], "keyEqualsValueFirst-noAdditional-key-global.tex");
    check("keyEqualsValueBraces", "braceTest.tex", &["-s", "-l=noAdditionalIndent-pdfstartview", "-o=+-noAdditional-pdfstartview"], "braceTest-noAdditional-pdfstartview.tex");
    check("keyEqualsValueBraces", "braceTest.tex", &["-s", "-l=noAdditionalIndent-pdfstartview,noAdditionalKey", "-o=+-noAdditional-pdfstartview-and-global"], "braceTest-noAdditional-pdfstartview-and-global.tex");
}

#[test]
fn pgfkeys() {
    check_overwrite("keyEqualsValueBraces", "pgfkeys-first.tex", &["-s"]);
    check_overwrite("keyEqualsValueBraces", "pgfkeys-nested.tex", &["-s"]);
    check("keyEqualsValueBraces", "pgfkeys-multiple-nested.tex", &["-s", "-m", "-l=../commands/mand-args-mod1", "-o=+-mod1"], "pgfkeys-multiple-nested-mod1.tex");
    check("keyEqualsValueBraces", "pgfkeys-multiple-nested.tex", &["-s", "-m", "-l=../commands/mand-args-mod1,noAdditionalIndentGlobal", "-o=+-global-mod1"], "pgfkeys-multiple-nested-global-mod1.tex");
}

#[test]
fn pgfkeys_equals_linebreaks() {
    check("keyEqualsValueBraces", "pgfkeys-multiple-keys.tex", &["-s", "-l=equals-not-finishes-with-line-break,keyStartsOnOwnLine", "-m", "-o=+-remove"], "pgfkeys-multiple-keys-remove.tex");
    check("keyEqualsValueBraces", "pgfkeys-multiple-keys-single-line.tex", &["-s", "-l=equals-not-finishes-with-line-break,keyStartsOnOwnLineYes", "-m", "-o=+-to-multiple"], "pgfkeys-multiple-keys-single-line-to-multiple.tex");
    check("keyEqualsValueBraces", "pgfkeys-add-equals.tex", &["-s", "-m", "-l=equals-not-finishes-with-line-break,EqualsStartsOnOwnLine", "-o=+-mod1"], "pgfkeys-add-equals-mod1.tex");
    check("keyEqualsValueBraces", "pgfkeys-add-equals.tex", &["-s", "-m", "-l=EqualsStartsOnOwnLine2", "-o=+-mod2"], "pgfkeys-add-equals-mod2.tex");
}

// ---------------------------------------------------------------------------
// T1 补齐 S1：循环前直接调用的 4 条缺口（脚本第 13、14、17、18 行）
// ---------------------------------------------------------------------------

/// L13/L14（`-l=noAdditionalKey`）＋ L17/L18（`-l=noAdditionalIndent-pdfstartview`）。
fn gaps_no_additional(only_pending: bool) {
    for input in ["braceTestsmall.tex", "braceTest.tex"] {
        let base = input.strip_suffix(".tex").unwrap();
        let expected = format!("{base}-noAdditional-key-global.tex");
        if is_pending(&expected) != only_pending {
            continue;
        }
        check_owned(
            DIR,
            input,
            &owned(&["-s", "-l=noAdditionalKey", "-o=+-noAdditional-key-global"]),
            &expected,
        );
    }
    for input in ["keyEqualsValueFirst.tex", "braceTestsmall.tex"] {
        let base = input.strip_suffix(".tex").unwrap();
        let expected = format!("{base}-noAdditional-pdfstartview.tex");
        if is_pending(&expected) != only_pending {
            continue;
        }
        check_owned(
            DIR,
            input,
            &owned(&[
                "-s",
                "-l=noAdditionalIndent-pdfstartview",
                "-o=+-noAdditional-pdfstartview",
            ]),
            &expected,
        );
    }
}

#[test]
fn key_no_additional_indent_remaining() {
    gaps_no_additional(false);
    report();
}

// ---------------------------------------------------------------------------
// T1 补齐 S2：循环体（脚本第 32–63 行）— 11×48 + 32 = 560 条
// ---------------------------------------------------------------------------

/// L37/L40/L43：`pgfkeys-nested` 三族，各 48 条（共 144）。
fn gaps_pgfkeys_nested_loops(only_pending: bool) {
    for i in 1..=48 {
        let variants: Vec<(String, String, String)> = vec![
            (
                format!("-l=mand-args-mod{i}"),
                format!("-o=+-mod{i}"),
                format!("pgfkeys-nested-mod{i}.tex"),
            ),
            (
                format!("-l=mand-args-mod{i},noAdditionalIndentGlobal"),
                format!("-o=+-noAdditional-Global-mod{i}"),
                format!("pgfkeys-nested-noAdditional-Global-mod{i}.tex"),
            ),
            (
                format!("-l=mand-args-mod{i},noAdditionalIndent-start"),
                format!("-o=+-noAdditional-start-mod{i}"),
                format!("pgfkeys-nested-noAdditional-start-mod{i}.tex"),
            ),
        ];
        for (yaml, out, expected) in &variants {
            if is_pending(expected) != only_pending {
                continue;
            }
            check_owned(DIR, "pgfkeys-nested.tex", &owned(&["-s", "-m", yaml, out]), expected);
        }
    }
}

#[test]
fn pgfkeys_nested_mod_loops() {
    gaps_pgfkeys_nested_loops(false);
    report();
}

/// L46–L49：`pgfkeys-remove-line-breaks-first` 四族，各 48 条（共 192）。
fn gaps_remove_line_breaks_first_loops(only_pending: bool) {
    for i in 1..=48 {
        let variants: Vec<(String, String, String)> = vec![
            (
                format!("-l=equals-not-finishes-with-line-break,mand-args-mod{i}"),
                format!("-o=+-mod{i}"),
                format!("pgfkeys-remove-line-breaks-first-mod{i}.tex"),
            ),
            (
                format!("-l=equals-not-finishes-with-line-break,mand-args-mod{i},unprotect-blank-lines,equalsKeyOff"),
                format!("-o=+-unprotect-equalsKeyOff-mod{i}"),
                format!("pgfkeys-remove-line-breaks-first-unprotect-equalsKeyOff-mod{i}.tex"),
            ),
            (
                format!("-l=equals-not-finishes-with-line-break,mand-args-mod{i},unprotect-blank-lines"),
                format!("-o=+-unprotect-mod{i}"),
                format!("pgfkeys-remove-line-breaks-first-unprotect-mod{i}.tex"),
            ),
            (
                format!("-l=equals-not-finishes-with-line-break,mand-args-mod{i},unprotect-blank-lines,noCondenseMultipleLines"),
                format!("-o=+-unprotect-nocondense-mod{i}"),
                format!("pgfkeys-remove-line-breaks-first-unprotect-nocondense-mod{i}.tex"),
            ),
        ];
        for (yaml, out, expected) in &variants {
            if is_pending(expected) != only_pending {
                continue;
            }
            check_owned(
                DIR,
                "pgfkeys-remove-line-breaks-first.tex",
                &owned(&["-s", "-m", yaml, out]),
                expected,
            );
        }
    }
}

#[test]
fn pgfkeys_remove_line_breaks_first_loops() {
    gaps_remove_line_breaks_first_loops(false);
    report();
}

/// L53：`[[ $i -lt 33 ]]` 守卫 ⇒ 仅 i = 1..=32；输入/产物均为 `.bib`（非 `.tex` 扩展名）。
fn gaps_contributors_bib(only_pending: bool) {
    for i in 1..=32 {
        let expected = format!("contributors-mod{i}.bib");
        if is_pending(&expected) != only_pending {
            continue;
        }
        let yaml = format!("-l=equals-not-finishes-with-line-break,online,mand-args-keywords{i}");
        let out = format!("-o=+-mod{i}.bib");
        check_owned(DIR, "contributors.bib", &owned(&["-s", "-m", &yaml, &out]), &expected);
    }
}

#[test]
fn contributors_bib_loop() {
    gaps_contributors_bib(false);
    report();
}

/// L58（尾注释版，48 条）＋ L59–L61（`pgfkeys-remove-line-breaks` 三族，144 条）= 192 条。
fn gaps_remove_line_breaks_loops(only_pending: bool) {
    for i in 1..=48 {
        let expected = format!("pgfkeys-remove-line-breaks-first-tc-mod{i}.tex");
        if is_pending(&expected) == only_pending {
            let yaml = format!("-l=equals-not-finishes-with-line-break,unprotect-blank-lines,mand-args-mod{i}");
            let out = format!("-o=+-mod{i}");
            check_owned(
                DIR,
                "pgfkeys-remove-line-breaks-first-tc.tex",
                &owned(&["-s", "-m", &yaml, &out]),
                &expected,
            );
        }
    }
    for i in 1..=48 {
        let variants: Vec<(String, String, String)> = vec![
            (
                format!("-l=equals-not-finishes-with-line-break,mand-args-mod{i}"),
                format!("-o=+-mod{i}"),
                format!("pgfkeys-remove-line-breaks-mod{i}.tex"),
            ),
            (
                format!("-l=equals-not-finishes-with-line-break,mand-args-mod{i},unprotect-blank-lines"),
                format!("-o=+-unprotect-mod{i}"),
                format!("pgfkeys-remove-line-breaks-unprotect-mod{i}.tex"),
            ),
            (
                format!("-l=equals-not-finishes-with-line-break,mand-args-mod{i},unprotect-blank-lines,noCondenseMultipleLines"),
                format!("-o=+-unprotect-nocondense-mod{i}"),
                format!("pgfkeys-remove-line-breaks-unprotect-nocondense-mod{i}.tex"),
            ),
        ];
        for (yaml, out, expected) in &variants {
            if is_pending(expected) != only_pending {
                continue;
            }
            check_owned(
                DIR,
                "pgfkeys-remove-line-breaks.tex",
                &owned(&["-s", "-m", yaml, out]),
                expected,
            );
        }
    }
}

#[test]
fn pgfkeys_remove_line_breaks_loops() {
    gaps_remove_line_breaks_loops(false);
    report();
}

// ---------------------------------------------------------------------------
// T1 补齐 S3：循环后直接调用的 21 条缺口（脚本第 68–105 行）
// ---------------------------------------------------------------------------

/// L69/L71/L76–L79：keyStartsOnOwnLine / EqualsStartsOnOwnLine 系列（6 条）。
fn gaps_keys_on_own_line(only_pending: bool) {
    let cases: &[(&str, &[&str], &str)] = &[
        (
            "pgfkeys-multiple-keys.tex",
            &[
                "-s",
                "-l=equals-not-finishes-with-line-break,keyStartsOnOwnLine,unprotect-blank-lines",
                "-m",
                "-o=+-unprotect-remove",
            ],
            "pgfkeys-multiple-keys-unprotect-remove.tex",
        ),
        (
            "pgfkeys-multiple-keys-single-line.tex",
            &[
                "-s",
                "-l=equals-not-finishes-with-line-break,keyStartsOnOwnLineWithComment",
                "-m",
                "-o=+-to-multiple-comment",
            ],
            "pgfkeys-multiple-keys-single-line-to-multiple-comment.tex",
        ),
        (
            "pgfkeys-add-equals.tex",
            &[
                "-s",
                "-m",
                "-l=equals-not-finishes-with-line-break,EqualsStartsOnOwnLine-comment",
                "-o=+-comments",
            ],
            "pgfkeys-add-equals-comments.tex",
        ),
        (
            "pgfkeys-add-equals.tex",
            &[
                "-s",
                "-m",
                "-l=equals-not-finishes-with-line-break,EqualsStartsOnOwnLine-comment,mand-args-mod17",
                "-o=+-comments-everywhere",
            ],
            "pgfkeys-add-equals-comments-everywhere.tex",
        ),
        (
            "pgfkeys-equals-remove-line-breaks.tex",
            &[
                "-s",
                "-m",
                "-l=equals-not-finishes-with-line-break,EqualsStartsOnOwnLine-remove",
                "-o=+-mod0",
            ],
            "pgfkeys-equals-remove-line-breaks-mod0.tex",
        ),
        (
            "pgfkeys-equals-remove-line-breaks.tex",
            &[
                "-s",
                "-m",
                "-l=equals-not-finishes-with-line-break,EqualsStartsOnOwnLine-remove,unprotect-blank-lines",
                "-o=+-unprotect-mod0",
            ],
            "pgfkeys-equals-remove-line-breaks-unprotect-mod0.tex",
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
fn pgfkeys_keys_on_own_line_gaps() {
    gaps_keys_on_own_line(false);
    report();
}

/// L82–L105：outn.cls、环境可选参数、混合对象、hea 系列、反斜杠规避、issue-378（15 条）。
fn gaps_misc(only_pending: bool) {
    let cases: &[(&str, &[&str], &str)] = &[
        // L82/L83：非 .tex 输入，产物扩展名为 .cls
        (
            "outn-tiny.cls",
            &["-s", "-o=+-default.cls"],
            "outn-tiny-default.cls",
        ),
        (
            "outn-small.cls",
            &["-s", "-o=+-default.cls"],
            "outn-small-default.cls",
        ),
        // L86
        (
            "environment-opt-arg-with-commands1.tex",
            &["-s", "-o=+-default"],
            "environment-opt-arg-with-commands1-default.tex",
        ),
        // L89/L90：输入本身带 default-small，产物 -out
        (
            "mixed-ifelsefi-commands-keys-braces1-default-small.tex",
            &["-s", "-o=+-out"],
            "mixed-ifelsefi-commands-keys-braces1-default-small-out.tex",
        ),
        (
            "environment-opt-arg-with-commands1-default-small.tex",
            &["-s", "-o=+-out"],
            "environment-opt-arg-with-commands1-default-small-out.tex",
        ),
        // L91–L94
        (
            "mixed-ifelsefi-commands-keys-braces1.tex",
            &["-s", "-o=+-default", "-l=../filecontents/indentPreambleYes"],
            "mixed-ifelsefi-commands-keys-braces1-default.tex",
        ),
        (
            "glossary1.tex",
            &["-s", "-o=+-default"],
            "glossary1-default.tex",
        ),
        (
            "pgfplotstable1.tex",
            &["-s", "-o=+-default", "-l=../filecontents/indentPreambleYes"],
            "pgfplotstable1-default.tex",
        ),
        (
            "tikz2.tex",
            &["-s", "-o=+-default", "-l", "tikz2"],
            "tikz2-default.tex",
        ),
        // L97：非 .tex 输入，产物扩展名为 .bib
        (
            "heabib.bib",
            &["-s", "-o=+-default.bib", "-l", "heabib"],
            "heabib-default.bib",
        ),
        // L98–L100
        (
            "hea-senior-fellowship-application.tex",
            &["-s", "-o=+-default", "-l=../filecontents/indentPreambleYes"],
            "hea-senior-fellowship-application-default.tex",
        ),
        (
            "hea-senior-fellowship-application.tex",
            &[
                "-s",
                "-o=+-max-indentation",
                "-l=../filecontents/indentPreambleYes,../environments/max-indentation1",
            ],
            "hea-senior-fellowship-application-max-indentation.tex",
        ),
        (
            "hea-senior-fellowship-application.tex",
            &[
                "-s",
                "-o=+-max-indentation5",
                "-l=../filecontents/indentPreambleYes,../environments/max-indentation5",
            ],
            "hea-senior-fellowship-application-max-indentation5.tex",
        ),
        // L103：-o 不带 `+`，整名指定输出
        (
            "dodge-double-back-slash.tex",
            &["-s", "-o", "dodge-double-back-slash-default"],
            "dodge-double-back-slash-default.tex",
        ),
        // L105
        (
            "issue-378.tex",
            &["-s", "-o=+-mod1", "-l", "issue-378"],
            "issue-378-mod1.tex",
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
fn key_equals_misc_gaps() {
    gaps_misc(false);
    report();
}

// ---------------------------------------------------------------------------
