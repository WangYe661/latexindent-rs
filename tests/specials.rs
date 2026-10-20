//! specials / ifelsefi / items 代码块测试，基准取自 tests/cases 下各目录。
mod common;
use common::{check, check_overwrite, check_owned, report};

// ---- specials ----
#[test]
fn special_default() {
    check("specials", "special1.tex", &["-s", "-o=special1-default"], "special1-default.tex");
}

#[test]
fn special_mod_line_breaks() {
    check(
        "specials",
        "special2.tex",
        &["-s", "-m", "-l=special-mod1", "-o=+-mod1"],
        "special2-mod1.tex",
    );
}

#[test]
fn special_mod17() {
    check(
        "specials",
        "special3.tex",
        &["-s", "-m", "-l=special-mod17", "-o=+-mod17"],
        "special3-mod17.tex",
    );
}

// ---------------------------------------------------------------------------
// specials 缺口批量补齐（T1：只搬运，不修码）
//
// 上游 latexindent.pl-main/test-cases/specials/specials-test-cases.sh：
//   调用 210 条（第一个循环 i=1..48；第二个循环因脚本 L74-75
//   `[[ $loopmax -gt 10 ]] && loopmax=10` 被截断，只有 i=1..10）
//   已搬 3 条（special1-default / special2-mod1 / special3-mod17）
//   → 缺口 207 条，全部以下面的断言形式落地。
// ---------------------------------------------------------------------------

/// 与基准不一致、暂时挂起的产物名（只搬运未修码，实现问题留待修码阶段）。
/// 共 41 条；断言仍然存在，只是被 `#[ignore]` 的 `specials_pending` 接管。
const PENDING: &[&str] = &[
    // L7 / L8 / L10 / L11
    "env1-mod1.tex",
    "env2-mod1.tex",
    "commands1-mod1.tex",
    "spec1-mod1.tex",
    // L23 循环：unprotect 组合（i = 1,2,3,4,13,14,15,16,33,34,35,36,45,46,47,48）
    "special1-remove-line-breaks-unprotect-mod1.tex",
    "special1-remove-line-breaks-unprotect-mod2.tex",
    "special1-remove-line-breaks-unprotect-mod3.tex",
    "special1-remove-line-breaks-unprotect-mod4.tex",
    "special1-remove-line-breaks-unprotect-mod13.tex",
    "special1-remove-line-breaks-unprotect-mod14.tex",
    "special1-remove-line-breaks-unprotect-mod15.tex",
    "special1-remove-line-breaks-unprotect-mod16.tex",
    "special1-remove-line-breaks-unprotect-mod33.tex",
    "special1-remove-line-breaks-unprotect-mod34.tex",
    "special1-remove-line-breaks-unprotect-mod35.tex",
    "special1-remove-line-breaks-unprotect-mod36.tex",
    "special1-remove-line-breaks-unprotect-mod45.tex",
    "special1-remove-line-breaks-unprotect-mod46.tex",
    "special1-remove-line-breaks-unprotect-mod47.tex",
    "special1-remove-line-breaks-unprotect-mod48.tex",
    // L44
    "special3-no-inline-math-mod17.tex",
    // L57
    "nested-specials-default.tex",
    // L60–L62
    "specialLeftBracket-commands-first.tex",
    "specialLeftBracket-commands-first-no-paren.tex",
    "specialLeftBracket-specials-first.tex",
    // L65–L70
    "algpseudocode-mod1.tex",
    "algpseudocode-mk2-mod0.tex",
    "algpseudocode-mk2-mod-array.tex",
    "algpseudocode-mod3.tex",
    // L79 循环（i = 1,2,5,6,8,9）
    "algpseudocode-mk2-mod1.tex",
    "algpseudocode-mk2-mod2.tex",
    "algpseudocode-mk2-mod5.tex",
    "algpseudocode-mk2-mod6.tex",
    "algpseudocode-mk2-mod8.tex",
    "algpseudocode-mk2-mod9.tex",
    // L84
    "algpseudocode-mk3-mod1.tex",
    // L87–L89
    "special4-mod1.tex",
    "special4-mod2.tex",
    "special4-mod3.tex",
    // L92 / L112（后者退出码非 0）
    "issue-511-mod1.tex",
    "issue-619-mod1.tex",
];

/// 一条上游调用：输入文件 + 参数（原样照抄 shell）+ 产物基准名。
struct Spec {
    group: &'static str,
    input: &'static str,
    args: Vec<String>,
    out: String,
}

fn spec(group: &'static str, input: &'static str, args: &[&str], out: &str) -> Spec {
    Spec {
        group,
        input,
        args: args.iter().map(|s| s.to_string()).collect(),
        out: out.to_string(),
    }
}

/// 全部 207 条缺口，按上游脚本顺序分组。
fn all_specs() -> Vec<Spec> {
    let mut v: Vec<Spec> = Vec::new();

    // ---- L7 / L8 / L10 / L11 ----
    v.push(spec(
        "basic",
        "env1.tex",
        &[
            "-s",
            "-m",
            "-l=../environments/env-mod1,../commands/unprotect-blank-lines",
            "-o=+-mod1",
        ],
        "env1-mod1.tex",
    ));
    v.push(spec(
        "basic",
        "env2.tex",
        &[
            "-s",
            "-m",
            "-l=../environments/env-mod1,../commands/unprotect-blank-lines",
            "-o=+-mod1",
        ],
        "env2-mod1.tex",
    ));
    v.push(spec(
        "basic",
        "commands1.tex",
        &[
            "-s",
            "-m",
            "-o=+-mod1",
            "-l=../mand-args/mand-args-mod1,../commands/unprotect-blank-lines",
        ],
        "commands1-mod1.tex",
    ));
    v.push(spec(
        "basic",
        "spec1.tex",
        &[
            "-s",
            "-m",
            "-o=+-mod1",
            "-l=special-mod1,../commands/unprotect-blank-lines",
        ],
        "spec1-mod1.tex",
    ));

    // ---- L21 / L22 / L23：modifyLineBreaks 主循环 i = 1..48 ----
    for i in 1..=48 {
        let l = format!("-l=special-mod{i}");
        v.push(spec(
            "loop1",
            "special1.tex",
            &["-s", &format!("-o=+-mod{i}"), "-m", &l],
            &format!("special1-mod{i}.tex"),
        ));
        v.push(spec(
            "loop1",
            "special1-remove-line-breaks.tex",
            &["-s", &format!("-o=+-mod{i}"), "-m", &l],
            &format!("special1-remove-line-breaks-mod{i}.tex"),
        ));
        v.push(spec(
            "loop1",
            "special1-remove-line-breaks.tex",
            &[
                "-s",
                &format!("-o=+-unprotect-mod{i}"),
                "-m",
                &format!("-l=special-mod{i},../commands/unprotect-blank-lines"),
            ],
            &format!("special1-remove-line-breaks-unprotect-mod{i}.tex"),
        ));
    }

    // ---- L29 / L30 ----
    v.push(spec(
        "mod1mod17",
        "special3.tex",
        &["-s", "-o=+-mod1", "-m", "-l=special-mod1"],
        "special3-mod1.tex",
    ));
    v.push(spec(
        "mod1mod17",
        "special2.tex",
        &["-s", "-o=+-mod17", "-m", "-l=special-mod17"],
        "special2-mod17.tex",
    ));

    // ---- L34–L37：noAdditionalIndent ----
    v.push(spec(
        "noadd",
        "special3.tex",
        &[
            "-s",
            "-o=+-noAddIndent-global-mod17",
            "-m",
            "-l=no-add-global,special-mod17",
        ],
        "special3-noAddIndent-global-mod17.tex",
    ));
    v.push(spec(
        "noadd",
        "special3.tex",
        &[
            "-s",
            "-o=+-noAddIndent-displayMath-mod17",
            "-m",
            "-l=no-add-displayMath,special-mod17",
        ],
        "special3-noAddIndent-displayMath-mod17.tex",
    ));
    v.push(spec(
        "noadd",
        "special3.tex",
        &[
            "-s",
            "-o=+-noAddIndent-displayMath-body-mod17",
            "-m",
            "-l=no-add-displayMath-body,special-mod17",
        ],
        "special3-noAddIndent-displayMath-body-mod17.tex",
    ));
    v.push(spec(
        "noadd",
        "special3.tex",
        &[
            "-s",
            "-o=+-noAddIndent-displayMath-body-no-mod17",
            "-m",
            "-l=no-add-displayMath-body-no,special-mod17",
        ],
        "special3-noAddIndent-displayMath-body-no-mod17.tex",
    ));

    // ---- L40 / L41：indentRules ----
    v.push(spec(
        "indentrules",
        "special3.tex",
        &[
            "-s",
            "-o=+-indent-rules-global-mod17",
            "-m",
            "-l=indent-rules-global,special-mod17",
        ],
        "special3-indent-rules-global-mod17.tex",
    ));
    v.push(spec(
        "indentrules",
        "special3.tex",
        &[
            "-s",
            "-o=+-indent-rules-displayMath-mod17",
            "-m",
            "-l=indent-rule-displayMath,special-mod17",
            "-g=one.log",
        ],
        "special3-indent-rules-displayMath-mod17.tex",
    ));

    // ---- L44：no-inline-math ----
    v.push(spec(
        "noinline",
        "special3.tex",
        &[
            "-s",
            "-o=+-no-inline-math-mod17",
            "-m",
            "-l=no-inline-math,special-mod17",
        ],
        "special3-no-inline-math-mod17.tex",
    ));

    // ---- L47–L51：issues from github ----
    v.push(spec(
        "github",
        "Focus.tex",
        &["-s", "-o=Focus-default"],
        "Focus-default.tex",
    ));
    v.push(spec(
        "github",
        "heptadecagon1.tex",
        &["-s", "-o=+-default"],
        "heptadecagon1-default.tex",
    ));
    v.push(spec(
        "github",
        "heptadecagon2.tex",
        &["-s", "-o=+-default"],
        "heptadecagon2-default.tex",
    ));
    v.push(spec(
        "github",
        "heptadecagon3.tex",
        &["-s", "-o=+-mod1", "-m", "-l=special-mod1,env-mod1"],
        "heptadecagon3-mod1.tex",
    ));
    v.push(spec(
        "github",
        "heptadecagon3.tex",
        &["-s", "-o=+-mod17", "-m", "-l=special-mod17,env-mod1"],
        "heptadecagon3-mod17.tex",
    ));

    // ---- L54–L57：nested special ----
    v.push(spec(
        "nested",
        "special-nested-simple.tex",
        &["-s", "-l", "dm-nested", "-o=+-default"],
        "special-nested-simple-default.tex",
    ));
    v.push(spec(
        "nested",
        "special-nested.tex",
        &["-s", "-o=+-default"],
        "special-nested-default.tex",
    ));
    v.push(spec(
        "nested",
        "special-nested.tex",
        &["-s", "-o=+-mod1", "-m", "-l", "special-mod1,dm-nested"],
        "special-nested-mod1.tex",
    ));
    v.push(spec(
        "nested",
        "nested-specials.tex",
        &["-s", "-l", "dm-nested", "-o=+-default"],
        "nested-specials-default.tex",
    ));

    // ---- L60–L62：specialBeforeCommand ----
    v.push(spec(
        "beforecmd",
        "specialLeftBracket.tex",
        &["-s", "-o=+-commands-first", "-l=specialsLeftRight"],
        "specialLeftBracket-commands-first.tex",
    ));
    v.push(spec(
        "beforecmd",
        "specialLeftBracket.tex",
        &["-s", "-o=+-commands-first-no-paren", "-l=specialsLeftRight"],
        "specialLeftBracket-commands-first-no-paren.tex",
    ));
    v.push(spec(
        "beforecmd",
        "specialLeftBracket.tex",
        &[
            "-s",
            "-o=+-specials-first",
            "-l=specialsLeftRight,specialBeforeCommand",
        ],
        "specialLeftBracket-specials-first.tex",
    ));

    // ---- L65–L70：specialMiddle ----
    v.push(spec(
        "middle",
        "algpseudocode.tex",
        &["-s", "-o=+-mod1", "-l=koppor"],
        "algpseudocode-mod1.tex",
    ));
    v.push(spec(
        "middle",
        "algpseudocode-mk1.tex",
        &["-s", "-o=+-mod1", "-l=koppor"],
        "algpseudocode-mk1-mod1.tex",
    ));
    v.push(spec(
        "middle",
        "algpseudocode-mk2.tex",
        &["-s", "-o=+-mod0", "-l=koppor"],
        "algpseudocode-mk2-mod0.tex",
    ));
    v.push(spec(
        "middle",
        "algpseudocode-mk2.tex",
        &["-s", "-o=+-mod-array", "-l=koppor2"],
        "algpseudocode-mk2-mod-array.tex",
    ));
    v.push(spec(
        "middle",
        "algpseudocode.tex",
        &["-s", "-o=+-mod3", "-l=koppor3"],
        "algpseudocode-mod3.tex",
    ));
    v.push(spec(
        "middle",
        "algpseudocode.tex",
        &["-s", "-o=+-mod4", "-l=koppor4"],
        "algpseudocode-mod4.tex",
    ));

    // ---- L79：specialMiddle polyswitch，i = 1..10（loopmax 被截断为 10）----
    for i in 1..=10 {
        v.push(spec(
            "loop2",
            "algpseudocode-mk2.tex",
            &[
                "-m",
                "-s",
                &format!("-l=koppor,special-middle{i}"),
                "-t",
                &format!("-o=+-mod{i}"),
            ],
            &format!("algpseudocode-mk2-mod{i}.tex"),
        ));
    }

    // ---- L84 ----
    v.push(spec(
        "mk3",
        "algpseudocode-mk3.tex",
        &["-s", "-o=+-mod1", "-l=koppor,koppor1"],
        "algpseudocode-mk3-mod1.tex",
    ));

    // ---- L87–L89：special verbatim ----
    v.push(spec(
        "verb",
        "special4.tex",
        &["-s", "-o=+-mod1", "-l=special-verb1"],
        "special4-mod1.tex",
    ));
    v.push(spec(
        "verb",
        "special4.tex",
        &["-s", "-o=+-mod2", "-l=special-verb2"],
        "special4-mod2.tex",
    ));
    v.push(spec(
        "verb",
        "special4.tex",
        &["-s", "-o=+-mod3", "-l=special-verb1,special-verb2"],
        "special4-mod3.tex",
    ));

    // ---- L91–L124 ----
    v.push(spec(
        "issues",
        "issue-448.tex",
        &["-s", "-l", "issue-448b", "-o=+-mod2"],
        "issue-448-mod2.tex",
    ));
    v.push(spec(
        "issues",
        "issue-511.tex",
        &["-s", "-l", "issue-511", "-o=+-mod1"],
        "issue-511-mod1.tex",
    ));
    v.push(spec(
        "issues",
        "issue-511.tex",
        &["-s", "-l", "issue-511a", "-o=+-mod2"],
        "issue-511-mod2.tex",
    ));
    v.push(spec(
        "issues",
        "issue-554.tex",
        &["-s", "-l", "issue-554", "-o=+-mod1"],
        "issue-554-mod1.tex",
    ));
    v.push(spec(
        "issues",
        "issue-572.tex",
        &["-s", "-l", "issue-572", "-m", "-r", "-o=+-mod1"],
        "issue-572-mod1.tex",
    ));
    v.push(spec(
        "issues",
        "issue-477.tex",
        &["-s", "-l", "issue-477", "-o=+-mod1"],
        "issue-477-mod1.tex",
    ));
    v.push(spec(
        "issues",
        "issue-480.tex",
        &["-s", "-l", "issue-480", "-o=+-mod1"],
        "issue-480-mod1.tex",
    ));
    v.push(spec(
        "issues",
        "issue-597.tex",
        &["-s", "-l", "issue-597.yaml", "-o=+-mod1"],
        "issue-597-mod1.tex",
    ));
    v.push(spec(
        "issues",
        "issue-614.tex",
        &["-s", "-t", "-o=+-mod1"],
        "issue-614-mod1.tex",
    ));
    v.push(spec(
        "issues",
        "commands2.tex",
        &["-s", "-l", "no-specials.yaml", "-o=+-mod1"],
        "commands2-mod1.tex",
    ));
    v.push(spec(
        "issues",
        "issue-615.tex",
        &["-s", "-l", "issue-615", "-t", "-o=+-mod1"],
        "issue-615-mod1.tex",
    ));
    v.push(spec(
        "issues",
        "spec-preamble1.tex",
        &["-s", "-l", "preamble1", "-t", "-o=+-mod1"],
        "spec-preamble1-mod1.tex",
    ));
    v.push(spec(
        "issues",
        "issue-619.tex",
        &["-s", "-l", "issue-619", "-t", "-o=+-mod1"],
        "issue-619-mod1.tex",
    ));
    v.push(spec(
        "issues",
        "issue-621.tex",
        &["-s", "-l", "issue-621", "-t", "-o=+-mod1"],
        "issue-621-mod1.tex",
    ));
    v.push(spec(
        "issues",
        "issue-638.tex",
        &["-s", "-o=+-mod1"],
        "issue-638-mod1.tex",
    ));
    v.push(spec(
        "issues",
        "issue-638.tex",
        &["-s", "-l", "issue-638", "-t", "-o=+-mod2"],
        "issue-638-mod2.tex",
    ));
    v.push(spec(
        "issues",
        "issue-647.tex",
        &["-s", "-o=+-mod1"],
        "issue-647-mod1.tex",
    ));
    v.push(spec(
        "issues",
        "issue-647.tex",
        &["-s", "-l", "issue-647", "-t", "-o=+-mod2"],
        "issue-647-mod2.tex",
    ));

    v
}

fn is_pending(out: &str) -> bool {
    PENDING.contains(&out)
}

/// 跑某一组；`pending` 决定是否只跑 PENDING 中的条目。
fn run_group(group: &str, pending: bool) {
    for s in all_specs() {
        if s.group != group || is_pending(&s.out) != pending {
            continue;
        }
        check_owned("specials", s.input, &s.args, &s.out);
    }
    report();
}

#[test]
fn specials_basic_mods() {
    run_group("basic", false);
}

#[test]
fn specials_mod_line_breaks_loop() {
    run_group("loop1", false);
}

#[test]
fn specials_mod1_mod17() {
    run_group("mod1mod17", false);
}

#[test]
fn specials_no_additional_indent() {
    run_group("noadd", false);
}

#[test]
fn specials_indent_rules() {
    run_group("indentrules", false);
}

#[test]
fn specials_no_inline_math() {
    run_group("noinline", false);
}

#[test]
fn specials_github_issues() {
    run_group("github", false);
}

#[test]
fn specials_nested() {
    run_group("nested", false);
}

#[test]
fn specials_before_command() {
    run_group("beforecmd", false);
}

#[test]
fn specials_middle() {
    run_group("middle", false);
}

#[test]
fn specials_middle_polyswitch() {
    run_group("loop2", false);
}

#[test]
fn specials_mk3() {
    run_group("mk3", false);
}

#[test]
fn specials_verbatim() {
    run_group("verb", false);
}

#[test]
fn specials_issues_tail() {
    run_group("issues", false);
}

/// T1 搬运时与基准不一致的 41 个缺口；修码阶段已全部修复，
/// 并入常规测试（PENDING 列表仅用于把这些条目归入本组）。
#[test]
fn specials_pending() {
    for s in all_specs() {
        if !is_pending(&s.out) {
            continue;
        }
        check_owned("specials", s.input, &s.args, &s.out);
    }
    report();
}

// ---- ifelsefi ----
#[test]
fn ifelsefi_first() {
    check("ifelsefi", "ifelsefi-first.tex", &["-s", "-o=+-mod0"], "ifelsefi-first-mod0.tex");
}

#[test]
fn ifelsefi_simple_nested() {
    check(
        "ifelsefi",
        "ifelsefi-simple-nested.tex",
        &["-s", "-o=+-mod1"],
        "ifelsefi-simple-nested-mod1.tex",
    );
}

#[test]
fn ifelsefi_small_overwrite() {
    check_overwrite("ifelsefi", "ifelsefiSmall.tex", &["-s"]);
}

// ---- items ----
#[test]
fn items_default() {
    check("items", "items1.tex", &["-s", "-o=+-default"], "items1-default.tex");
}

#[test]
fn items_mod1() {
    check("items", "items1-5.tex", &["-s", "-o=+-mod1"], "items1-5-mod1.tex");
}

#[test]
fn items_overwrite() {
    check_overwrite("items", "items4.tex", &["-s"]);
}

// ---------------------------------------------------------------------------
// 日志断言（上游脚本 L102/L107/L110/L113/L116/L121/L125 的 7 处 `egrep -i 'found:'`）
// ---------------------------------------------------------------------------

/// 已实测通过：issue-614 / issue-619 / issue-621 / issue-638 / spec-preamble1，
/// 以及 issue-615 的 special MIDDLE 命中行、issue-647 的 alignment hidden child 行。
#[test]
fn specials_logs() {
    // L102
    common::check_log_lines(
        "specials",
        "issue-614.tex",
        &["-s", "-t", "-o=+-mod1"],
        "found:",
        "issue-614.txt",
    );
    // L110
    common::check_log_lines(
        "specials",
        "spec-preamble1.tex",
        &["-s", "-l", "preamble1", "-t", "-o=+-mod1"],
        "found:",
        "spec-preamble1.txt",
    );
    // L113
    common::check_log_lines(
        "specials",
        "issue-619.tex",
        &["-s", "-l", "issue-619", "-t", "-o=+-mod1"],
        "found:",
        "issue-619.txt",
    );
    // L116
    common::check_log_lines(
        "specials",
        "issue-621.tex",
        &["-s", "-l", "issue-621", "-t", "-o=+-mod1"],
        "found:",
        "issue-621.txt",
    );
    // L121
    common::check_log_lines(
        "specials",
        "issue-638.tex",
        &["-s", "-l", "issue-638", "-t", "-o=+-mod2"],
        "found:",
        "issue-638.txt",
    );
    // L107：issue-615 的 special MIDDLE 命中行
    common::check_log_lines(
        "specials",
        "issue-615.tex",
        &["-s", "-l", "issue-615", "-t", "-o=+-mod1"],
        "found:",
        "issue-615.txt",
    );
    // L125：issue-647 的 alignment hidden child 行
    common::check_log_lines(
        "specials",
        "issue-647.tex",
        &["-s", "-l", "issue-647", "-t", "-o=+-mod2"],
        "found:",
        "issue-647.txt",
    );
}
