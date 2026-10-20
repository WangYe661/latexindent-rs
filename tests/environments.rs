//! 环境（environment）缩进与换行控制测试，基准取自 tests/cases/environments。
mod common;
use common::{check, check_owned, check_overwrite, report};

// 基础缩进：默认设置下环境体缩进一个层级
#[test]
fn simple_indent() {
    check_overwrite("environments", "environments-simple.tex", &["-s"]);
}

#[test]
fn nested_indent() {
    check_overwrite("environments", "environments-simple-nested.tex", &["-s"]);
}

#[test]
fn deeply_nested() {
    check_overwrite("environments", "environments-nested-nested.tex", &["-s"]);
}

#[test]
fn one_line() {
    check_overwrite("environments", "environments-one-line.tex", &["-s"]);
}

#[test]
fn challenging() {
    check_overwrite("environments", "environments-challenging.tex", &["-s"]);
}

#[test]
fn trailing_comments() {
    check_overwrite(
        "environments",
        "environments-trailing-comments.tex",
        &["-s"],
    );
}

// -o=+<后缀>：把结果写到 <stem><后缀>.tex，与基准同名比对
#[test]
fn opt_and_mand_args() {
    check(
        "environments",
        "env-diff-opt-mand-args.tex",
        &["-s", "-o=+-mod1"],
        "env-diff-opt-mand-args-mod1.tex",
    );
}

// -m + modifyLineBreaks：单行环境展开为多行
#[test]
fn single_line_to_multiline() {
    check(
        "environments",
        "env-single-line.tex",
        &["-s", "-m", "-l=env-mod1", "-o=+-mod1"],
        "env-single-line-mod1.tex",
    );
}

#[test]
fn single_line_to_multiline_mod5() {
    check(
        "environments",
        "env-single-line.tex",
        &["-s", "-m", "-l=env-mod5", "-o=+-mod5"],
        "env-single-line-mod5.tex",
    );
}

// 多行换行保护/取消保护
#[test]
fn modify_line_breaks_protect() {
    check(
        "environments",
        "environments-modify-multiple-line-breaks.tex",
        &["-s", "-m", "-l=env-mod1", "-o=+-protect-mod1"],
        "environments-modify-multiple-line-breaks-protect-mod1.tex",
    );
}

// -y 开关：内联 YAML 覆盖 defaultIndent
#[test]
fn yaml_switch_default_indent() {
    check(
        "environments",
        "environments-nested-fourth.tex",
        &["-s", "-o=+-yaml-switch1", "-yaml=defaultIndent: ' '"],
        "environments-nested-fourth-yaml-switch1.tex",
    );
}

// 空 body 环境
#[test]
fn empty_body() {
    check(
        "environments",
        "env-no-body.tex",
        &["-s", "-m", "-l=env-mod1", "-o=+-mod1"],
        "env-no-body-mod1.tex",
    );
}

// env-mod-lines poly-switch
#[test]
fn env_mod_lines() {
    check(
        "environments",
        "environments-one-line.tex",
        &[
            "-s",
            "-l=env-all-on,env-mod-lines1",
            "-m",
            "-o=+-mod-lines1",
        ],
        "environments-one-line-mod-lines1.tex",
    );
    check(
        "environments",
        "environments-one-line.tex",
        &[
            "-s",
            "-l=env-all-on,env-mod-lines2",
            "-m",
            "-o=+-mod-lines2",
        ],
        "environments-one-line-mod-lines2.tex",
    );
    check(
        "environments",
        "environments-one-line-simple.tex",
        &["-s", "-l=env-all-on,env-mod-lines5", "-m", "-o=+-mod1"],
        "environments-one-line-simple-mod1.tex",
    );
}

#[test]
fn remove_line_breaks() {
    check(
        "environments",
        "environments-remove-line-breaks.tex",
        &["-s", "-m", "-l=env-all-on,env-mod-lines6", "-o=+-mod1"],
        "environments-remove-line-breaks-mod1.tex",
    );
    check(
        "environments",
        "environments-remove-line-breaks-mk1.tex",
        &["-s", "-m", "-l=env-all-on,env-mod-lines7", "-o=+-mod1"],
        "environments-remove-line-breaks-mk1-mod1.tex",
    );
    check(
        "environments",
        "environments-modify-multiple-line-breaks.tex",
        &["-s", "-m", "-l=env-all-on,env-mod-lines9", "-o=+-mod1"],
        "environments-modify-multiple-line-breaks-mod1.tex",
    );
}

#[test]
fn line_break_conflict() {
    check(
        "environments",
        "environments-line-break-conflict.tex",
        &["-s", "-m", "-l=env-all-on,env-conflicts-mod1", "-o=+-mod1"],
        "environments-line-break-conflict-mod1.tex",
    );
    check(
        "environments",
        "environments-line-break-conflict-nested.tex",
        &["-s", "-m", "-l=env-all-on,env-conflicts-mod2", "-o=+-mod-2"],
        "environments-line-break-conflict-nested-mod-2.tex",
    );
    check(
        "environments",
        "environments-nested-fourth.tex",
        &["-s", "-l=max-indentation1", "-o=+-max-indentation1"],
        "environments-nested-fourth-max-indentation1.tex",
    );
}

// ---------------------------------------------------------------------------
// 以下为 T1 搬运：把 environments-test-cases.sh 中尚未搬成断言的调用补齐。
// 判据 = tests/*.rs 有无断言（不用 tests/cases 有无基准文件）。
// 循环完整展开：主循环 i = 1..48（脚本 loopmax=48，common.sh loopmin=1）。
//
// 说明：产物流与基准不一致的条目列入 PENDING，从正式函数中过滤出去，
// 单独放到 #[ignore] 的 environments_pending()，待修码阶段处理。
// ---------------------------------------------------------------------------

/// 已无待修用例（2026-10-18 全部转正 / 剔陈旧基准，见 doc/pending-work.md §5A.7）。
const PENDING: &[&str] = &[];

/// 一条待补的调用（输入 / 参数 / 产物基准名）。
struct CaseSpec {
    input: String,
    args: Vec<String>,
    out: String,
}

fn c(input: &str, args: Vec<String>, out: &str) -> CaseSpec {
    CaseSpec {
        input: input.to_string(),
        args,
        out: out.to_string(),
    }
}

fn s(args: &[&str]) -> Vec<String> {
    args.iter().map(|a| a.to_string()).collect()
}

const DIR: &str = "environments";

fn run_group(group: &[CaseSpec], pending: bool) {
    for cs in group
        .iter()
        .filter(|cs| PENDING.contains(&cs.out.as_str()) == pending)
    {
        check_owned(DIR, &cs.input, &cs.args, &cs.out);
    }
}


fn all_groups() -> Vec<Vec<CaseSpec>> {
    vec![
        group_overwrite(),
        group_challenging_output(),
        group_mod_lines(),
        group_line_breaks(),
        group_percent_and_global(),
        group_max_indentation(),
        group_no_body_aronvgi(),
        group_yaml_switch(),
        group_yswitch_subfields(),
        group_issues(),
        group_main_loop(),
    ]
}

// 段 1：脚本 L13–L19，`-w` 不动点（产物 = 输入文件本身）
fn group_overwrite() -> Vec<CaseSpec> {
    [
        "environments-verbatim-simple.tex",
        "environments-verbatim-harder.tex",
        "environments-noindent-block.tex",
        "environments-noindent-block-mk1.tex",
        "noIndent-maetra.tex",
        "no-environments.tex",
        "environments-repeated.tex",
    ]
    .iter()
    .map(|f| c(f, s(&["-w", "-s"]), f))
    .collect()
}

// 段 1：脚本 L21，产物与 L22/L23 同名（environments-challenging-output.tex），
// 但只有本条与基准不一致，无法用产物名单独过滤，故只在 environments_pending 里登记。
/// 段 1（脚本 L21）已剔除：`environments-challenging-output.tex` 是**陈旧基准**。
///
/// 上游脚本第 12 行 `latexindent.pl -w -s environments-challenging` 会**就地覆盖输入**，
/// 第 21 行再对该文件跑一次并产出这个基准。但用当前 Perl（5.40 + 本仓 vendored 版本）
/// 实测：单跑一次、连跑两次，输出都与**原始输入完全一致**（已有缩进被保留，幂等）；
/// 而基准里 `\begin{one}` 的 body 是 0 缩进、`three` 的 body 是 `\t` + 1 空格，
/// 任何一遍都复现不出来 ⇒ 该基准是旧版本产物，不能作为判据。
/// 本工程输出与当前 Perl 逐字节一致，故该条不再断言。
// 段 1：脚本 L22–L23（带 -l=env-all-on,env-chall），产物同上，两条均通过
fn group_challenging_output() -> Vec<CaseSpec> {
    vec![
        c(
            "environments-challenging.tex",
            s(&["-w", "-l=env-all-on,env-chall", "-s", "-o=+-output"]),
            "environments-challenging-output.tex",
        ),
        c(
            "environments-challenging.tex",
            s(&[
                "-w",
                "-l=env-all-on,env-chall",
                "-s",
                "-o=environments-challenging-output",
            ]),
            "environments-challenging-output.tex",
        ),
    ]
}

// 段 2：脚本 L28–L32
fn group_mod_lines() -> Vec<CaseSpec> {
    vec![
        c(
            "environments-one-line.tex",
            s(&["-s", "-l=env-all-on,env-mod-lines3", "-m", "-o=+-mod-lines3"]),
            "environments-one-line-mod-lines3.tex",
        ),
        c(
            "environments-one-line.tex",
            s(&["-s", "-l=env-all-on,env-mod-lines4", "-m", "-o=+-mod-lines4"]),
            "environments-one-line-mod-lines4.tex",
        ),
        c(
            "environments-one-line.tex",
            s(&["-s", "-l=env-all-on,env-mod-lines5", "-m", "-o=+-mod-lines5"]),
            "environments-one-line-mod-lines5.tex",
        ),
        // L32：`-w` + `-o`，-o 优先（LogFile.pm：-w 被置 0）
        c(
            "environments-nested-fourth.tex",
            s(&["-s", "-l=env-all-on", "-w", "-o=+-mod1", "-m"]),
            "environments-nested-fourth-mod1.tex",
        ),
    ]
}

// 段 3：脚本 L39–L59
fn group_line_breaks() -> Vec<CaseSpec> {
    vec![
        c(
            "environments-remove-line-breaks.tex",
            s(&["-s", "-m", "-l=env-all-on,env-mod-lines7", "-o=+-mod2"]),
            "environments-remove-line-breaks-mod2.tex",
        ),
        c(
            "environments-remove-line-breaks-trailing-comments.tex",
            s(&["-s", "-m", "-l=env-all-on,env-mod-lines7", "-o=+-mod1"]),
            "environments-remove-line-breaks-trailing-comments-mod1.tex",
        ),
        c(
            "environments-remove-line-breaks-trailing-comments-mk1.tex",
            s(&["-s", "-m", "-l=env-all-on,env-mod-lines8", "-o=+-mod1"]),
            "environments-remove-line-breaks-trailing-comments-mk1-mod1.tex",
        ),
        c(
            "environments-modify-multiple-line-breaks.tex",
            s(&["-s", "-o=+-mod2", "-m", "-l=env-all-on,env-mod-lines10"]),
            "environments-modify-multiple-line-breaks-mod2.tex",
        ),
        c(
            "environments-modify-multiple-line-breaks.tex",
            s(&["-s", "-o=+-mod3", "-m", "-l=env-all-on,env-mod-lines11"]),
            "environments-modify-multiple-line-breaks-mod3.tex",
        ),
        c(
            "environments-line-break-conflict.tex",
            s(&["-s", "-m", "-o=+-mod4", "-l=env-all-on,env-conflicts-mod4"]),
            "environments-line-break-conflict-mod4.tex",
        ),
        c(
            "environments-line-break-conflict-nested.tex",
            s(&["-s", "-m", "-o=+-mod-3", "-l=env-all-on,env-conflicts-mod3"]),
            "environments-line-break-conflict-nested-mod-3.tex",
        ),
        c(
            "environments-modify-multiple-line-breaks.tex",
            s(&["-s", "-o=+-mod4", "-m", "-l=env-all-on,env-mod-lines12"]),
            "environments-modify-multiple-line-breaks-mod4.tex",
        ),
        c(
            "environments-modify-multiple-line-breaks.tex",
            s(&["-s", "-o=+-mod5", "-m", "-l=env-all-on,env-mod-lines13"]),
            "environments-modify-multiple-line-breaks-mod5.tex",
        ),
        c(
            "environments-modify-multiple-line-breaks.tex",
            s(&["-s", "-o=+-mod6", "-m", "-l=env-all-on,env-mod-lines14"]),
            "environments-modify-multiple-line-breaks-mod6.tex",
        ),
        c(
            "environments-modify-multiple-line-breaks-verbatim.tex",
            s(&["-s", "-m", "-o=+-mod1", "-l=env-all-on,env-mod-lines12"]),
            "environments-modify-multiple-line-breaks-verbatim-mod1.tex",
        ),
    ]
}

// 段 3：脚本 L62–L78（trailing comment 百分号、全局 noAdditionalIndent / indentRules、特殊字符）
fn group_percent_and_global() -> Vec<CaseSpec> {
    vec![
        c(
            "environments-simple-trailing-comments.tex",
            s(&[
                "-s",
                "-l=env-all-on,env-addPercentAfterBegin",
                "-m",
                "-o=+-percent-after-begin",
            ]),
            "environments-simple-trailing-comments-percent-after-begin.tex",
        ),
        c(
            "environments-simple-trailing-comments.tex",
            s(&[
                "-s",
                "-l=env-all-on,env-addPercentAfterBody",
                "-m",
                "-o=+-percent-after-Body",
            ]),
            "environments-simple-trailing-comments-percent-after-Body.tex",
        ),
        c(
            "environments-simple-trailing-comments.tex",
            s(&[
                "-s",
                "-l=env-all-on,env-addPercentAfterEnd",
                "-m",
                "-o=+-percent-after-End",
            ]),
            "environments-simple-trailing-comments-percent-after-End.tex",
        ),
        c(
            "environments-simple.tex",
            s(&["-s", "-l=env-all-on,noAdditionalIndentGlobal", "-o=+-global"]),
            "environments-simple-global.tex",
        ),
        c(
            "environments-ifelsefi.tex",
            s(&["-s", "-l=env-all-on,noAdditionalIndentGlobal", "-o=+-global"]),
            "environments-ifelsefi-global.tex",
        ),
        c(
            "environments-simple.tex",
            s(&[
                "-s",
                "-l=env-all-on,indentRulesGlobal",
                "-o=+-indent-rules-global",
            ]),
            "environments-simple-indent-rules-global.tex",
        ),
        c(
            "environments-ifelsefi.tex",
            s(&[
                "-s",
                "-l=env-all-on,indentRulesGlobal",
                "-o=+-indent-rules-global",
            ]),
            "environments-ifelsefi-indent-rules-global.tex",
        ),
        c(
            "environments-simple.tex",
            s(&[
                "-s",
                "-l=env-all-on,indentRulesGlobal,noAdditionalIndentGlobal",
                "-o=+-indent-rules-global-conflict",
            ]),
            "environments-simple-indent-rules-global-conflict.tex",
        ),
        c(
            "environments-special-characters.tex",
            s(&["-s", "-o=+-default"]),
            "environments-special-characters-default.tex",
        ),
    ]
}

// 段 5：脚本 L97–L102，maximumIndentation
fn group_max_indentation() -> Vec<CaseSpec> {
    (2..=5)
        .map(|i| {
            c(
                "environments-nested-fourth.tex",
                vec![
                    "-s".to_string(),
                    format!("-o=+-max-indentation{i}"),
                    format!("-l=max-indentation{i}"),
                ],
                &format!("environments-nested-fourth-max-indentation{i}.tex"),
            )
        })
        .chain(std::iter::once(c(
            "environments-nested-fourth.tex",
            s(&[
                "-s",
                "-o=+-max-indentation-mod1",
                "-l=max-indentation1,env-mod-lines1",
                "-m",
            ]),
            "environments-nested-fourth-max-indentation-mod1.tex",
        )))
        .chain(std::iter::once(c(
            "environments-verbatim-harder.tex",
            s(&["-s", "-o=+-max-ind1", "-l=max-indentation1"]),
            "environments-verbatim-harder-max-ind1.tex",
        )))
        .collect()
}

// 段 5：脚本 L106（空 body + mod17）、L109（-o=+mod 无分隔符）
fn group_no_body_aronvgi() -> Vec<CaseSpec> {
    vec![
        c(
            "env-no-body.tex",
            s(&["-s", "-o=+-mod17", "-l=env-mod17", "-m"]),
            "env-no-body-mod17.tex",
        ),
        c(
            "aronvgi.tex",
            s(&["-s", "-m", "-l=aronvgi", "-o=+mod"]),
            "aronvgimod.tex",
        ),
    ]
}

// 段 5：脚本 L112、L114–L120（-yaml / -y 内联开关）
fn group_yaml_switch() -> Vec<CaseSpec> {
    vec![
        c(
            "environments-nested-fourth.tex",
            s(&["-s", "-o=+-yaml-switch0", "-yaml=defaultIndent: ' '", "-d"]),
            "environments-nested-fourth-yaml-switch0.tex",
        ),
        c(
            "environments-nested-fourth.tex",
            s(&[
                "-s",
                "-o=+-yaml-switch1A",
                "-y=defaultIndent: ' ',maximumIndentation:' '",
            ]),
            "environments-nested-fourth-yaml-switch1A.tex",
        ),
        c(
            "environments-nested-fourth.tex",
            s(&[
                "-s",
                "-o=+-yaml-switch2",
                r#"-y=defaultIndent: '\t\t\t\t'"#,
            ]),
            "environments-nested-fourth-yaml-switch2.tex",
        ),
        c(
            "environments-nested-fourth.tex",
            s(&[
                "-s",
                "-o=+-yaml-switch3",
                r#"-y=indentRules:one:"\t\t\t\t""#,
            ]),
            "environments-nested-fourth-yaml-switch3.tex",
        ),
        c(
            "environments-nested-fourth.tex",
            s(&[
                "-s",
                "-o=+-yaml-switch4",
                r#"-y=indentRules:one:"\t\t\t\t",defaultIndent: " ""#,
            ]),
            "environments-nested-fourth-yaml-switch4.tex",
        ),
        c(
            "environments-nested-fourth.tex",
            s(&[
                "-s",
                "-o=+-yaml-switch4A",
                "-y=modifyLineBreaks  :  environments: EndStartsOnOwnLine:3",
                "-m",
            ]),
            "environments-nested-fourth-yaml-switch4A.tex",
        ),
        c(
            "environments-nested-fourth.tex",
            s(&[
                "-s",
                "-o=+-yaml-switch5",
                "-y=modifyLineBreaks  :  environments: one: EndStartsOnOwnLine:3",
                "-m",
            ]),
            "environments-nested-fourth-yaml-switch5.tex",
        ),
        c(
            "environments-nested-fourth.tex",
            s(&[
                "-s",
                "-o=+-yaml-switch6",
                r#"-y=defaultIndent: "",modifyLineBreaks  :  environments: one: EndStartsOnOwnLine:3"#,
                "-m",
            ]),
            "environments-nested-fourth-yaml-switch6.tex",
        ),
    ]
}

// 段 5：脚本 L123–L125（-y 子字段用 `;` 分隔；注意 `-y='...'` 的冒号串是 YAML，不是 -o 参数）
fn group_yswitch_subfields() -> Vec<CaseSpec> {
    vec![
        c(
            "env-single-line.tex",
            s(&[
                "-s",
                "-y=modifyLineBreaks  :  environments: EndStartsOnOwnLine:3; BeginStartsOnOwnLine:3",
                "-t",
                "-m",
                "-o=+-yswitch1",
            ]),
            "env-single-line-yswitch1.tex",
        ),
        c(
            "env-single-line.tex",
            s(&[
                "-s",
                "-y=modifyLineBreaks  :  environments: myenv  : EndStartsOnOwnLine:3; BeginStartsOnOwnLine:3",
                "-t",
                "-m",
                "-o=+-yswitch2",
            ]),
            "env-single-line-yswitch2.tex",
        ),
        c(
            "env-single-line.tex",
            s(&[
                "-s",
                "-y=modifyLineBreaks:environments: myenv:EndStartsOnOwnLine:3; BeginStartsOnOwnLine:3;BodyStartsOnOwnLine:2,modifyLineBreaks:environments:EndFinishesWithLineBreak:1;BodyStartsOnOwnLine:2",
                "-t",
                "-m",
                "-o=+-yswitch3",
            ]),
            "env-single-line-yswitch3.tex",
        ),
    ]
}

// 段 5：脚本 L129–L145（issue-256 / 488 / 508）
fn group_issues() -> Vec<CaseSpec> {
    vec![
        c(
            "issue-256.tex",
            s(&["-s", "-o=+-mod1"]),
            "issue-256-mod1.tex",
        ),
        c(
            "issue-256.tex",
            s(&["-s", "-y=dos2unixlinebreaks:1", "-o=+-mod2"]),
            "issue-256-mod2.tex",
        ),
        c(
            "issue-488.tex",
            s(&["-s", "-l=issue-488", "-r", "-m", "-o=+-mod1"]),
            "issue-488-mod1.tex",
        ),
        c(
            "issue-488a.tex",
            s(&["-s", "-l=issue-488a", "-m", "-o=+-mod1"]),
            "issue-488a-mod1.tex",
        ),
        c(
            "issue-508.tex",
            s(&["-s", "-l=issue-508", "-m", "-o=+-mod1"]),
            "issue-508-mod1.tex",
        ),
        c(
            "issue-508.tex",
            s(&["-s", "-l=issue-508a", "-m", "-o=+-mod2"]),
            "issue-508-mod2.tex",
        ),
        c(
            "issue-508.tex",
            s(&["-s", "-l=issue-508b", "-m", "-o=+-mod3"]),
            "issue-508-mod3.tex",
        ),
        c(
            "issue-508.tex",
            s(&["-s", "-l=issue-508c", "-r", "-o=+-mod4"]),
            "issue-508-mod4.tex",
        ),
        c(
            "issue-508.tex",
            s(&["-s", "-l=issue-508d", "-m", "-o=+-mod5"]),
            "issue-508-mod5.tex",
        ),
    ]
}

// 段 4 主循环：脚本 L86–L89，i = 1..48（loopmin=1 / loopmax=48）
fn group_main_loop() -> Vec<CaseSpec> {
    let mut v = Vec::new();
    for i in 1..=48 {
        v.push(c(
            "env-single-line.tex",
            vec![
                "-s".to_string(),
                "-m".to_string(),
                format!("-o=+-mod{i}"),
                format!("-l=env-mod{i}"),
            ],
            &format!("env-single-line-mod{i}.tex"),
        ));
        v.push(c(
            "environments-modify-multiple-line-breaks.tex",
            vec![
                "-s".to_string(),
                format!("-o=+-protect-mod{i}"),
                "-m".to_string(),
                format!("-l=env-mod{i}"),
            ],
            &format!("environments-modify-multiple-line-breaks-protect-mod{i}.tex"),
        ));
        v.push(c(
            "environments-modify-multiple-line-breaks.tex",
            vec![
                "-s".to_string(),
                format!("-o=+-un-protect-mod{i}"),
                "-m".to_string(),
                format!("-l=env-mod{i},unprotect-blank-lines"),
            ],
            &format!("environments-modify-multiple-line-breaks-un-protect-mod{i}.tex"),
        ));
        v.push(c(
            "env-no-blank-after-end.tex",
            vec![
                "-s".to_string(),
                "-m".to_string(),
                format!("-o=+-mod{i}"),
                format!("-l=env-mod{i}"),
            ],
            &format!("env-no-blank-after-end-mod{i}.tex"),
        ));
    }
    v
}

#[test]
fn overwrite_gap() {
    run_group(&group_overwrite(), false);
    report();
}

#[test]
fn challenging_output_gap() {
    run_group(&group_challenging_output(), false);
    report();
}

#[test]
fn mod_lines_gap() {
    run_group(&group_mod_lines(), false);
    report();
}

#[test]
fn line_breaks_gap() {
    run_group(&group_line_breaks(), false);
    report();
}

#[test]
fn percent_and_global_gap() {
    run_group(&group_percent_and_global(), false);
    report();
}

#[test]
fn max_indentation_gap() {
    run_group(&group_max_indentation(), false);
    report();
}

#[test]
fn no_body_aronvgi_gap() {
    run_group(&group_no_body_aronvgi(), false);
    report();
}

#[test]
fn yaml_switch_gap() {
    run_group(&group_yaml_switch(), false);
    report();
}

#[test]
fn yswitch_subfields_gap() {
    run_group(&group_yswitch_subfields(), false);
    report();
}

#[test]
fn issue_cases_gap() {
    run_group(&group_issues(), false);
    report();
}

#[test]
fn main_loop_gap() {
    run_group(&group_main_loop(), false);
    report();
}

// ---------------------------------------------------------------------------
// 上游长尾调用：L35（缺失 YAML）与 L136/L138（warn 日志断言）
// ---------------------------------------------------------------------------

/// L35：`latexindent.pl -s -l=env-all-on,testlogfile environments-nested-fourth`
///
/// `testlogfile.yaml` 在上游与本仓库都不存在（上游本行意为"改日志文件名"的示例），
/// Perl 版对缺失 `-l` YAML 只打 warning 并继续执行。该调用没有 `-o`/`-w`，
/// 也不落盘产物；此处断言退出码为 0（即缺失 YAML 不致命）。
#[test]
fn missing_local_yaml_is_tolerated() {
    let case = common::run_allow_missing_yaml(
        "environments",
        "environments-nested-fourth.tex",
        &["-s", "-l=env-all-on,testlogfile"]
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>(),
    );
    case.assert_success();
}

/// L136/L138：`latexindent.pl -s -m -l issue-488b issue-488a`（无 `-o`，只查日志）
/// 及 `egrep -i 'warn:' indent.log > issue-488b.txt`（488c 同理）。
///
/// 上游用 `egrep -i`（大小写不敏感），日志行首为 `WARN:`，这里用大写模式匹配。
#[test]
fn environments_warn_logs() {
    common::check_log_lines(
        "environments",
        "issue-488a.tex",
        &["-s", "-m", "-l=issue-488b"],
        "WARN:",
        "issue-488b.txt",
    );
    common::check_log_lines(
        "environments",
        "issue-488a.tex",
        &["-s", "-m", "-l=issue-488c"],
        "WARN:",
        "issue-488c.txt",
    );
}

// CRLF 输入不能在 \begin / \item 行尾累积缩进（Windows/VS Code 光标漂移回归）。
#[test]
fn crlf_does_not_accumulate_trailing_indent() {
    let case = common::run(
        "environments",
        "crlf-no-trailing-indent.tex",
        &["-y=defaultIndent: '    '".to_string()],
    );
    case.assert_success();
    let out = String::from_utf8(case.output.stdout.clone()).expect("stdout 不是 UTF-8");
    assert!(out.contains("\r\n"), "CRLF 输入应保持 CRLF 输出");
    assert_eq!(
        out.matches('\n').count(),
        out.matches("\r\n").count(),
        "输出不应混入裸 LF"
    );
    for line in out.split_inclusive('\n') {
        let content = line.trim_end_matches(['\r', '\n']);
        assert!(
            !content.ends_with(' ') && !content.ends_with('\t'),
            "行尾残留缩进: {line:?}"
        );
    }
}


