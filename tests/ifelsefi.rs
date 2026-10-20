//! ifElseFi 条件块测试，覆盖 tests/cases/ifelsefi/ifelsefi-test-cases.sh 的全部调用。
//!
//! 上游脚本共 381 次调用（静态 61 + 嵌套双层循环 320）；本文件 381 条断言全覆盖：
//! - 静态单发用 `check`；
//! - `-w` 不动点用 `check_overwrite`；
//! - L102-114 的嵌套双层循环（i=33..48 × j=1..10 × 2）用两层 `for` + `check_owned` + `format!`，
//!   末尾 `report()` 一次暴露全部差异。
mod common;
use common::{check, check_owned, check_overwrite, report};

#[test]
fn ifelsefi_default() {
    check("ifelsefi", "ifelsefi-first.tex", &["-s", "-o=+-mod0"], "ifelsefi-first-mod0.tex");
    check("ifelsefi", "ifelsefi-simple-nested.tex", &["-s", "-o=+-mod1"], "ifelsefi-simple-nested-mod1.tex");
    check("ifelsefi", "ifelsefi-multiple-nested.tex", &["-s", "-o=+-mod1"], "ifelsefi-multiple-nested-mod1.tex");
}

#[test]
fn outn_cls() {
    check("ifelsefi", "outn.cls", &["-s", "-o=+-default", "-l=notabular,resetItem,ifstar"], "outn-default.cls");
}

#[test]
fn ifelsefi_overwrite() {
    check_overwrite("ifelsefi", "ifelsefiSmall.tex", &["-s"]);
    check_overwrite("ifelsefi", "conditional.tex", &["-s"]);
}

// -m + ifelsefi-mod-lines poly-switch
#[test]
fn ifelsefi_mod_lines() {
    check("ifelsefi", "ifelsefi-first.tex", &["-s", "-m", "-l=ifelsefi-all-on,ifelsefi-mod-lines1", "-o=+-mod1"], "ifelsefi-first-mod1.tex");
    check("ifelsefi", "ifelsefi-first-mod-A.tex", &["-s", "-m", "-l=ifelsefi-all-on,ifelsefi-mod-lines1", "-o=+-mod1"], "ifelsefi-first-mod-A-mod1.tex");
    check("ifelsefi", "ifelsefi-first-mod-B.tex", &["-s", "-m", "-l=ifelsefi-all-on,ifelsefi-mod-lines4", "-o=+-mod4"], "ifelsefi-first-mod-B-mod4.tex");
    check("ifelsefi", "ifelsefi-one-line.tex", &["-s", "-m", "-o=+-mod1", "-l=ifelsefi-all-on"], "ifelsefi-one-line-mod1.tex");
}

#[test]
fn ifelsefi_nested_blank_lines() {
    check("ifelsefi", "ifelsefi-nested-blank-lines.tex", &["-s", "-m", "-l=ifelsefi-all-on,ifelsefi-mod-lines18,removeTWS-before", "-o=+-mod18"], "ifelsefi-nested-blank-lines-mod18.tex");
}

// modify line breaks：L21-43 的其余 mod-lines poly-switch 组合
#[test]
fn ifelsefi_mod_lines_rest() {
    check("ifelsefi", "ifelsefi-first.tex", &["-s", "-m", "-l=ifelsefi-all-on,ifelsefi-mod-lines2", "-o=+-mod2"], "ifelsefi-first-mod2.tex");
    check("ifelsefi", "ifelsefi-first.tex", &["-s", "-m", "-l=ifelsefi-all-on,ifelsefi-mod-lines3", "-o=+-mod3"], "ifelsefi-first-mod3.tex");
    check("ifelsefi", "ifelsefi-first-mod-A.tex", &["-s", "-m", "-l=ifelsefi-all-on,ifelsefi-mod-lines2", "-o=+-mod2"], "ifelsefi-first-mod-A-mod2.tex");
    check("ifelsefi", "ifelsefi-first-mod-A.tex", &["-s", "-m", "-l=ifelsefi-all-on,ifelsefi-mod-lines3", "-o=+-mod3"], "ifelsefi-first-mod-A-mod3.tex");
    check("ifelsefi", "ifelsefi-first-mod-B.tex", &["-s", "-m", "-l=ifelsefi-all-on,ifelsefi-mod-lines5", "-o=+-mod5"], "ifelsefi-first-mod-B-mod5.tex");
    check("ifelsefi", "ifelsefi-first-mod-B.tex", &["-s", "-m", "-l=ifelsefi-all-on,ifelsefi-mod-lines6", "-o=+-mod6"], "ifelsefi-first-mod-B-mod6.tex");
    check("ifelsefi", "ifelsefi-first-mod-C.tex", &["-s", "-m", "-l=ifelsefi-all-on,ifelsefi-mod-lines7", "-o=+-mod7"], "ifelsefi-first-mod-C-mod7.tex");
    check("ifelsefi", "ifelsefi-first-mod-C.tex", &["-s", "-m", "-l=ifelsefi-all-on,ifelsefi-mod-lines8", "-o=+-mod8"], "ifelsefi-first-mod-C-mod8.tex");
    check("ifelsefi", "ifelsefi-first-mod-C.tex", &["-s", "-m", "-l=ifelsefi-all-on,ifelsefi-mod-lines9", "-o=+-mod9"], "ifelsefi-first-mod-C-mod9.tex");
    check("ifelsefi", "ifelsefi-first-mod-D.tex", &["-s", "-m", "-l=ifelsefi-all-on,ifelsefi-mod-lines10", "-o=+-mod10"], "ifelsefi-first-mod-D-mod10.tex");
    check("ifelsefi", "ifelsefi-first-mod-D.tex", &["-s", "-m", "-l=ifelsefi-all-on,ifelsefi-mod-lines11", "-o=+-mod11"], "ifelsefi-first-mod-D-mod11.tex");
    check("ifelsefi", "ifelsefi-first-mod-D.tex", &["-s", "-m", "-l=ifelsefi-all-on,ifelsefi-mod-lines12", "-o=+-mod12"], "ifelsefi-first-mod-D-mod12.tex");
    check("ifelsefi", "ifelsefi-first-mod-E.tex", &["-s", "-m", "-l=ifelsefi-all-on,ifelsefi-mod-lines12", "-o=+-mod12"], "ifelsefi-first-mod-E-mod12.tex");
    // else line break experiments
    check("ifelsefi", "ifelsefi-first-mod-E.tex", &["-s", "-m", "-l=ifelsefi-all-on,ifelsefi-mod-lines13", "-o=+-mod13"], "ifelsefi-first-mod-E-mod13.tex");
    check("ifelsefi", "ifelsefi-first-mod-E.tex", &["-s", "-m", "-l=ifelsefi-all-on,ifelsefi-mod-lines14", "-o=+-mod14"], "ifelsefi-first-mod-E-mod14.tex");
    check("ifelsefi", "ifelsefi-first-mod-E.tex", &["-s", "-m", "-l=ifelsefi-all-on,ifelsefi-mod-lines15", "-o=+-mod15"], "ifelsefi-first-mod-E-mod15.tex");
    check("ifelsefi", "ifelsefi-first-mod-F.tex", &["-s", "-m", "-l=ifelsefi-all-on,ifelsefi-mod-lines13", "-o=+-mod13"], "ifelsefi-first-mod-F-mod13.tex");
    check("ifelsefi", "ifelsefi-first-mod-E.tex", &["-s", "-m", "-l=ifelsefi-all-on,ifelsefi-mod-lines16", "-o=+-mod16"], "ifelsefi-first-mod-E-mod16.tex");
    check("ifelsefi", "ifelsefi-first-mod-E.tex", &["-s", "-m", "-l=ifelsefi-all-on,ifelsefi-mod-lines17", "-o=+-mod17"], "ifelsefi-first-mod-E-mod17.tex");
}

// first mixed example：L52-63
#[test]
fn ifelsefi_env_mixed() {
    check(
        "ifelsefi",
        "env-ifelsefi-mixed.tex",
        &["-s", "-m", "-l=../environments/env-all-on,ifelsefi-all-on", "-o=+-out1"],
        "env-ifelsefi-mixed-out1.tex",
    );
    check("ifelsefi", "env-ifelsefi-mixed.tex", &["-s", "-m", "-l=ifelsefi-all-on,env-ifelsefi-mixed-mod1", "-o=+-mod1"], "env-ifelsefi-mixed-mod1.tex");
    check("ifelsefi", "env-ifelsefi-mixed.tex", &["-s", "-m", "-l=ifelsefi-all-on,env-ifelsefi-mixed-mod2", "-o=+-mod2"], "env-ifelsefi-mixed-mod2.tex");
    check("ifelsefi", "env-ifelsefi-mixed.tex", &["-s", "-m", "-l=ifelsefi-all-on,env-ifelsefi-mixed-mod3", "-o=+-mod3"], "env-ifelsefi-mixed-mod3.tex");
    check("ifelsefi", "env-ifelsefi-mixed.tex", &["-s", "-m", "-l=ifelsefi-all-on,env-ifelsefi-mixed-mod4", "-o=+-mod4"], "env-ifelsefi-mixed-mod4.tex");
    check("ifelsefi", "env-ifelsefi-mixed.tex", &["-s", "-m", "-l=ifelsefi-all-on,env-ifelsefi-mixed-mod5", "-o=+-mod5"], "env-ifelsefi-mixed-mod5.tex");
    check("ifelsefi", "env-ifelsefi-mixed.tex", &["-s", "-m", "-l=ifelsefi-all-on,env-ifelsefi-mixed-mod6", "-o=+-mod6"], "env-ifelsefi-mixed-mod6.tex");
    check("ifelsefi", "env-ifelsefi-mixed.tex", &["-s", "-m", "-l=ifelsefi-all-on,env-ifelsefi-mixed-mod7", "-o=+-mod7"], "env-ifelsefi-mixed-mod7.tex");
    check("ifelsefi", "env-ifelsefi-mixed.tex", &["-s", "-m", "-l=ifelsefi-all-on,env-ifelsefi-mixed-mod8", "-o=+-mod8"], "env-ifelsefi-mixed-mod8.tex");
    check("ifelsefi", "env-ifelsefi-mixed.tex", &["-s", "-m", "-l=ifelsefi-all-on,env-ifelsefi-mixed-mod9", "-o=+-mod9"], "env-ifelsefi-mixed-mod9.tex");
    check("ifelsefi", "env-ifelsefi-mixed.tex", &["-s", "-m", "-l=ifelsefi-all-on,env-ifelsefi-mixed-mod10", "-o=+-mod10"], "env-ifelsefi-mixed-mod10.tex");
    check("ifelsefi", "env-ifelsefi-mixed.tex", &["-s", "-m", "-l=ifelsefi-all-on,env-ifelsefi-mixed-mod11", "-o=+-mod11"], "env-ifelsefi-mixed-mod11.tex");
}

// remove all example：L66
#[test]
fn ifelsefi_remove_line_breaks() {
    check(
        "ifelsefi",
        "ifelsefi-remove-line-breaks.tex",
        &["-s", "-m", "-l=env-ifelsefi-mixed-mod1", "-o=+-mod1"],
        "ifelsefi-remove-line-breaks-mod1.tex",
    );
}

// recursive object examples：L69-71
#[test]
fn ifelsefi_recursive() {
    check(
        "ifelsefi",
        "env-ifelsefi-mixed-recursive.tex",
        &["-s", "-m", "-o=+-mod1", "-l=ifelsefi-all-on"],
        "env-ifelsefi-mixed-recursive-mod1.tex",
    );
    // 上游脚本此处为 `-l=ifelsefi-all-on,env-conflicts-mod2`，但该 YAML 在上游 ifelsefi 目录里
    // 并不存在（只在 environments/ 下）→ Perl 侧实测 "yaml file not found ... Proceeding without it."，
    // 基准 env-ifelsefi-mixed-recursive-mod2.tex 即"未加载该 YAML"的产物，故此处等价地只传
    // ifelsefi-all-on（harness 也不复制不存在的 YAML）。
    check(
        "ifelsefi",
        "env-ifelsefi-mixed-recursive.tex",
        &["-s", "-m", "-o=+-mod2", "-l=ifelsefi-all-on"],
        "env-ifelsefi-mixed-recursive-mod2.tex",
    );
    check(
        "ifelsefi",
        "ifelsefi-env-mixed-recursive.tex",
        &["-m", "-l=ifelsefi-all-on,if-env-recursive-conflicts", "-s", "-o=+-mod1"],
        "ifelsefi-env-mixed-recursive-mod1.tex",
    );
}

// condense/protect line breaks：L74-76
#[test]
fn ifelsefi_condense() {
    check(
        "ifelsefi",
        "ifelsefi-nested-blank-lines.tex",
        &["-s", "-m", "-o=+-condense"],
        "ifelsefi-nested-blank-lines-condense.tex",
    );
    check(
        "ifelsefi",
        "ifelsefi-nested-blank-lines.tex",
        &["-s", "-m", "-o=+-condense-mod1", "-l=ifelsefi-all-on,ifelsefi-condense-mod1"],
        "ifelsefi-nested-blank-lines-condense-mod1.tex",
    );
    check(
        "ifelsefi",
        "ifelsefi-nested-blank-lines.tex",
        &["-s", "-m", "-o=+-condense-mod2", "-l=ifelsefi-all-on,ifelsefi-condense-mod2"],
        "ifelsefi-nested-blank-lines-condense-mod2.tex",
    );
}

// multi-switch / addPercent 注释：L79-83
#[test]
fn ifelsefi_add_percent_comments() {
    check("ifelsefi", "ifelsefi-simpletrailing-comments.tex", &["-s", "-m", "-l=ifelsefi-all-on,ifelsefi-addPercentBeforeBegin", "-o=+-addPercentBeforeBegin"], "ifelsefi-simpletrailing-comments-addPercentBeforeBegin.tex");
    check("ifelsefi", "ifelsefi-simpletrailing-comments.tex", &["-s", "-m", "-l=ifelsefi-all-on,ifelsefi-addPercentAfterBegin", "-o=+-addPercentAfterBegin"], "ifelsefi-simpletrailing-comments-addPercentAfterBegin.tex");
    check("ifelsefi", "ifelsefi-simpletrailing-comments.tex", &["-s", "-m", "-l=ifelsefi-all-on,ifelsefi-addPercentAfterBody", "-o=+-addPercentAfterBody"], "ifelsefi-simpletrailing-comments-addPercentAfterBody.tex");
    check("ifelsefi", "ifelsefi-simpletrailing-comments.tex", &["-s", "-m", "-l=ifelsefi-all-on,ifelsefi-addPercentAfterEnd", "-o=+-addPercentAfterEnd"], "ifelsefi-simpletrailing-comments-addPercentAfterEnd.tex");
    check("ifelsefi", "ifelsefi-one-line-mk1.tex", &["-s", "-m", "-l=ifelsefi-add-comments-all", "-o=+-all-comments"], "ifelsefi-one-line-mk1-all-comments.tex");
}

// noAdditionalIndent / indentRules 全局开关：L86、L89
#[test]
fn ifelsefi_global_switches() {
    check(
        "ifelsefi",
        "ifelsefi-multiple-nested.tex",
        &["-s", "-l=ifelsefi-all-on,noAdditionalIndentGlobal", "-o=+-global"],
        "ifelsefi-multiple-nested-global.tex",
    );
    check(
        "ifelsefi",
        "ifelsefi-multiple-nested.tex",
        &["-s", "-l=ifelsefi-all-on,indentRulesGlobal", "-o=+-indent-rules-global"],
        "ifelsefi-multiple-nested-indent-rules-global.tex",
    );
}

// ifx / tcbbreakable / or / issue-250 / tabdo.sty：L92、L95、L98、L118、L121
#[test]
fn ifelsefi_misc() {
    check("ifelsefi", "ifnum-bug.tex", &["-s", "-o=+-out"], "ifnum-bug-out.tex");
    check("ifelsefi", "tcbbreakable-small.tex", &["-s", "-o=+-default"], "tcbbreakable-small-default.tex");
    check("ifelsefi", "ifelsefi-first-or.tex", &["-s", "-o=+-default"], "ifelsefi-first-or-default.tex");
    check("ifelsefi", "issue-250.tex", &["-s", "-o=+-mod1"], "issue-250-mod1.tex");
    // 输入为 .sty（非 .tex），产物 tabdo-default.sty 沿用原扩展名
    check("ifelsefi", "tabdo.sty", &["-s", "-o=+-default"], "tabdo-default.sty");
}

// modifyLineBreaks 嵌套双层循环 L109：i=33..48 × j=1..10（160 条）
#[test]
fn ifelsefi_loop_test_else() {
    for i in 33..=48 {
        for j in 1..=10 {
            check_owned(
                "ifelsefi",
                "ifelsefi-loop-test.tex",
                &[
                    "-s".into(),
                    "-m".into(),
                    format!("-o=+-mod{i}-else{j}"),
                    format!("-l=ifelsefi-mod{i},else-mod{j}"),
                ],
                &format!("ifelsefi-loop-test-mod{i}-else{j}.tex"),
            );
        }
    }
    report();
}

// modifyLineBreaks 嵌套双层循环 L111：i=33..48 × j=1..10（160 条）
#[test]
fn ifelsefi_loop_test_or() {
    for i in 33..=48 {
        for j in 1..=10 {
            check_owned(
                "ifelsefi",
                "ifelsefi-loop-test-or.tex",
                &[
                    "-s".into(),
                    "-m".into(),
                    format!("-o=+-mod{i}-or{j}"),
                    format!("-l=ifelsefi-mod{i},or-mod{j}"),
                ],
                &format!("ifelsefi-loop-test-or-mod{i}-or{j}.tex"),
            );
        }
    }
    report();
}
