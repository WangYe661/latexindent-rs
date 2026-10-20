//! items 列表项测试，覆盖 tests/cases/items/items-test-cases.sh 的调用。
//!
//! T1 搬运：补上脚本中尚未搬成断言的 **126** 条调用。
//! 脚本第 2 行 `loopmax=10`、未设 `verbatimTest`，故 `common.sh` 的 `loopmin` 保持 1，
//! 第 54 行的 `for (( i=$loopmin ; i <= $loopmax ; i++ ))` 展开为 **i = 1..=10 共 10 轮**
//! （每轮 9 条 = 90 条），加上循环外 45 条 = 上游调用 135 条。
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

/// 脚本 135 条调用中尚未搬成断言的 126 条
/// （原有 9 条见本文件末尾 `items_spaces_and_tabs` / `items_default` / `items_overwrite`）。
fn ported_cases() -> Vec<Case> {
    let mut v: Vec<Case> = Vec::new();

    // ---- 循环前：L24 – L51（L10–L23 的前 9 条已由原有断言覆盖）----
    // L24 / L27：`-w` 原地覆盖，基准 = 输入文件本身
    v.push(case("items4-5.tex", &["-s", "-w"], "items4-5.tex"));
    v.push(case("items4-6.tex", &["-s", "-w"], "items4-6.tex"));
    v.push(case(
        "items4-6.tex",
        &["-s", "-l=yetanotheritem", "-o=+-yetanotheritem"],
        "items4-6-yetanotheritem.tex",
    ));
    v.push(case(
        "items5.tex",
        &["-s", "-o=+-default"],
        "items5-default.tex",
    ));
    // L34 – L36：noAdditionalIndent
    v.push(case(
        "items1.tex",
        &["-s", "-l=noAdditionalIndentItemize", "-o=+-noAdditionalIndentItemize"],
        "items1-noAdditionalIndentItemize.tex",
    ));
    v.push(case(
        "items1.tex",
        &["-s", "-l=noAdditionalIndentItems", "-o=+-noAdditionalIndentItems"],
        "items1-noAdditionalIndentItems.tex",
    ));
    v.push(case(
        "items3.tex",
        &["-s", "-l=noAdditionalIndent-myenv", "-o=+-noAdditionalIndent-myenv"],
        "items3-noAdditionalIndent-myenv.tex",
    ));
    // L39 – L40：indentRules
    v.push(case(
        "items1.tex",
        &["-s", "-l=indentRulesItemize", "-o=+-indentRulesItemize"],
        "items1-indentRulesItemize.tex",
    ));
    v.push(case(
        "items1.tex",
        &["-s", "-l=indentRulesItems", "-o=+-indentRulesItems"],
        "items1-indentRulesItems.tex",
    ));
    // L43 – L44：不同 item 名，`-w` 原地覆盖
    v.push(case("items1-myitem.tex", &["-s", "-l=myitem", "-w"], "items1-myitem.tex"));
    v.push(case("items1-part.tex", &["-s", "-l=part", "-w"], "items1-part.tex"));
    // L47 – L51：modifyLineBreaks
    v.push(case("items1.tex", &["-s", "-m", "-o=+-mod0"], "items1-mod0.tex"));
    v.push(case(
        "items1-blanklines.tex",
        &["-s", "-m", "-o=+-mod0"],
        "items1-blanklines-mod0.tex",
    ));
    v.push(case("items2.tex", &["-s", "-m", "-o=+-mod0"], "items2-mod0.tex"));
    v.push(case(
        "items2-1.tex",
        &["-s", "-m", "-l=items-mod3", "-o=+-mod3"],
        "items2-1-mod3.tex",
    ));
    v.push(case(
        "items2-1.tex",
        &["-s", "-m", "-l=items-mod3,removeTWS-before", "-o=+-mod4"],
        "items2-1-mod4.tex",
    ));

    // ---- 循环体：i = 1..=10（10 轮 × 9 条 = 90 条）----
    for i in 1..=10u32 {
        let n = i.to_string();
        // L60 – L62
        let l = format!("-l=../environments/env-all-on,items-mod{n}");
        let o = format!("-o=+-mod{n}");
        for base in ["items1", "items2", "items3"] {
            v.push(case(
                &format!("{base}.tex"),
                &["-s", "-m", &l, &o],
                &format!("{base}-mod{n}.tex"),
            ));
        }
        // L65：blank line tests
        v.push(case(
            "items1-blanklines.tex",
            &["-s", "-m", &l, &o],
            &format!("items1-blanklines-mod{n}.tex"),
        ));
        // L66：unPreserveBlankLines
        let lu = format!("-l=../environments/env-all-on,items-mod{n},unPreserveBlankLines");
        let ou = format!("-o=+-unPreserveBlankLines-mod{n}");
        v.push(case(
            "items1-blanklines.tex",
            &["-s", "-m", &lu, &ou],
            &format!("items1-blanklines-unPreserveBlankLines-mod{n}.tex"),
        ));
        // L67：BodyStartsOnOwnLine
        let lb = format!("-l=../environments/env-all-on,items-mod{n},BodyStartsOnOwnLine");
        let ob = format!("-o=+-BodyStartsOnOwnLine-mod{n}");
        v.push(case(
            "items1.tex",
            &["-s", "-m", &lb, &ob],
            &format!("items1-BodyStartsOnOwnLine-mod{n}.tex"),
        ));
        // L70 – L72：starts on one line, adds linebreaks accordingly
        for base in ["items5", "items6", "items7"] {
            v.push(case(
                &format!("{base}.tex"),
                &["-s", "-m", &l, &o],
                &format!("{base}-mod{n}.tex"),
            ));
        }
    }

    // ---- 循环后：L78 – L115 共 20 条 ----
    v.push(case(
        "items13.tex",
        &["-s", "-m", "-l=items-mod2", "-o=+-mod2"],
        "items13-mod2.tex",
    ));
    v.push(case(
        "items13.tex",
        &["-s", "-m", "-l=items-mod2,removeTWS-before", "-o=+-remove-before-mod2"],
        "items13-remove-before-mod2.tex",
    ));
    v.push(case(
        "items8.tex",
        &["-s", "-o=+-mod0", "-l=../environments/env-all-on"],
        "items8-mod0.tex",
    ));
    v.push(case(
        "items9.tex",
        &["-s", "-o=+-mod0", "-l=../environments/env-all-on"],
        "items9-mod0.tex",
    ));
    // L84 / L87：`-w` 原地覆盖
    v.push(case("items10.tex", &["-s", "-w"], "items10.tex"));
    v.push(case(
        "items10.tex",
        &[
            "-s",
            "-o=+-myenv-noAdditionalIndent",
            "-l=../environments/env-all-on,myenv",
        ],
        "items10-myenv-noAdditionalIndent.tex",
    ));
    v.push(case(
        "items10.tex",
        &[
            "-s",
            "-o=+-items-noAdditionalIndent",
            "-l=../environments/env-all-on,noAdditionalIndentItems",
        ],
        "items10-items-noAdditionalIndent.tex",
    ));
    v.push(case("items11.tex", &["-s", "-w"], "items11.tex"));
    v.push(case(
        "items12.tex",
        &["-s", "-o=+-mod0", "-l=../environments/env-all-on"],
        "items12-mod0.tex",
    ));
    v.push(case(
        "items12.tex",
        &[
            "-s",
            "-m",
            "-l=../opt-args/opt-args-remove-all,../environments/env-all-on,env-mod-lines1",
            "-o=+-mod1",
        ],
        "items12-mod1.tex",
    ));
    // L92：noAdditionalIndent（全局）
    v.push(case(
        "items12.tex",
        &[
            "-s",
            "-o=items12-Global",
            "-l=../environments/env-all-on,noAdditionalIndentGlobal,resetItem",
        ],
        "items12-Global.tex",
    ));
    // L95：indentRules（全局）
    v.push(case(
        "items12.tex",
        &[
            "-s",
            "-o=items12-indent-rules-Global",
            "-l=../environments/env-all-on,indentRulesGlobal,resetItem",
        ],
        "items12-indent-rules-Global.tex",
    ));
    // L98：poly-switch bug（issue 94）
    v.push(case(
        "bug1.tex",
        &["-s", "-m", "-o=+-mod0", "-l=bug"],
        "bug1-mod0.tex",
    ));
    v.push(case(
        "issue-249.tex",
        &["-s", "-o=+-mod0"],
        "issue-249-mod0.tex",
    ));
    v.push(case(
        "issue-307.tex",
        &["-s", "-o=+-mod0"],
        "issue-307-mod0.tex",
    ));
    v.push(case(
        "issue-467.tex",
        &["-s", "-o=+-mod1"],
        "issue-467-mod1.tex",
    ));
    v.push(case(
        "issue-492.tex",
        &["-s", "-m", "-l=issue-492a", "-o=+-mod2"],
        "issue-492-mod2.tex",
    ));
    v.push(case(
        "issue-507.tex",
        &["-s", "-l=issue-507", "-o=+-mod1"],
        "issue-507-mod1.tex",
    ));
    v.push(case(
        "issue-507.tex",
        &["-s", "-l=issue-507a", "-o=+-mod2"],
        "issue-507-mod2.tex",
    ));
    v.push(case(
        "issue-518.tex",
        &["-s", "-l=issue-518", "-o=+-mod1"],
        "issue-518-mod1.tex",
    ));

    v
}

/// 与基准不一致 / 退出码非 0 的产物名；这些用例被分流到 `items_pending`
/// （本轮只搬运、不修码，故保留为 `#[ignore]` 待修）。
/// 已无待修用例：`issue-518-mod1` 随「数字 YAML 键回落字符串」修好（§5A.9）。
const PENDING: &[&str] = &[];

fn run_ported_cases(pending: bool) {
    for (input, args, expected) in ported_cases() {
        if PENDING.contains(&expected.as_str()) == pending {
            check_owned("items", &input, &args, &expected);
        }
    }
    report();
}

#[test]
fn items_ported() {
    run_ported_cases(false);
}


// ---------------------------------------------------------------------------
// 本轮搬运前已有的 9 条断言（对应脚本 L10–L23）
// ---------------------------------------------------------------------------

#[test]
fn items_spaces_and_tabs() {
    check("items", "spaces-and-tabs.tex", &["-s", "-l=../environments/env-all-on,tabs-follow-tabs", "-o=spaces-and-tabs-tft"], "spaces-and-tabs-tft.tex");
    check("items", "spaces-and-tabs.tex", &["-s", "-l=../environments/env-all-on,spaces-follow-tabs", "-o=spaces-and-tabs-sft"], "spaces-and-tabs-sft.tex");
    check("items", "spaces-and-tabs.tex", &["-s", "-l=../environments/env-all-on,tabs-follow-spaces", "-o=spaces-and-tabs-tfs"], "spaces-and-tabs-tfs.tex");
}

#[test]
fn items_default() {
    check("items", "items1.tex", &["-s", "-o=+-default"], "items1-default.tex");
    check("items", "items1-5.tex", &["-s", "-o=+-mod1"], "items1-5-mod1.tex");
    check("items", "items2.tex", &["-s", "-t", "-o=+-default"], "items2-default.tex");
    check("items", "items2-5.tex", &["-s", "-o=+-mod1"], "items2-5-mod1.tex");
    check("items", "items3.tex", &["-s", "-o=+-default"], "items3-default.tex");
}

#[test]
fn items_overwrite() {
    check_overwrite("items", "items4.tex", &["-s"]);
}

/// L20：`latexindent.pl -s -t items2 -o=+-default` 的 `egrep 'found:' indent.log > items2.txt`。
#[test]
fn items_logs() {
    common::check_log_lines(
        "items",
        "items2.tex",
        &["-s", "-t", "-o=+-default"],
        "found:",
        "items2.txt",
    );
}
