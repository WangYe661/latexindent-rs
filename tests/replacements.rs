//! 替换（replacement，-r/-rr/-rv）模式测试，基准取自 tests/cases/replacements。
//! 覆盖 latexindent.pl-main/test-cases/replacements/replacement-test-cases.sh 的全部调用。
mod common;
use common::{check, check_owned, report};

/// 一条上游 shell 调用：`latexindent.pl <args...> <input>` → 产物 `expected`。
struct RCase {
    input: &'static str,
    args: &'static [&'static str],
    expected: &'static str,
}

/// 需要补的 41 条（上游 47 条缺口，扣除 6 条 T2 依赖，见文件末尾说明）。
const CASES: &[RCase] = &[
    // L6 / L7 / L10：字符串替换
    RCase { input: "test1.tex", args: &["-s", "-r", "-l=replace1.yaml", "-o=+-mod1"], expected: "test1-mod1.tex" },
    RCase { input: "test1.tex", args: &["-s", "-r", "-l=replace2.yaml", "-o=+-mod2"], expected: "test1-mod2.tex" },
    // L9：-onlyreplacement 与 L8 同产物
    RCase { input: "test1.tex", args: &["-s", "-onlyreplacement", "-l=replace1.yaml", "-o=+-rr-mod1"], expected: "test1-rr-mod1.tex" },
    RCase { input: "test1.tex", args: &["-s", "-r", "-l=replace3.yaml", "-o=+-mod3"], expected: "test1-mod3.tex" },
    RCase { input: "test2.tex", args: &["-s", "-r", "-l=replace4.yaml", "-o=+-mod4"], expected: "test2-mod4.tex" },
    RCase { input: "test2.tex", args: &["-s", "-r", "-m", "-l=replace5.yaml", "-o=+-mod5"], expected: "test2-mod5.tex" },
    RCase { input: "test3.tex", args: &["-s", "-r", "-l=replace6.yaml", "-o=+-mod6"], expected: "test3-mod6.tex" },
    // 正则替换
    RCase { input: "test4.tex", args: &["-s", "-r", "-l=replace7.yaml", "-o=+-mod7"], expected: "test4-mod7.tex" },
    // 修饰符 modifiers
    RCase { input: "test4.tex", args: &["-s", "-r", "-l=replace8.yaml", "-o=+-mod8"], expected: "test4-mod8.tex" },
    RCase { input: "test4.tex", args: &["-s", "-r", "-l=replace9.yaml", "-o=+-mod9"], expected: "test4-mod9.tex" },
    RCase { input: "test3.tex", args: &["-s", "-r", "-l=replace10.yaml", "-o=+-mod10"], expected: "test3-mod10.tex" },
    // $$...$$ → equation 环境
    RCase { input: "test5.tex", args: &["-s", "-r", "-l=replace11.yaml", "-o=+-mod11"], expected: "test5-mod11.tex" },
    RCase { input: "test5.tex", args: &["-s", "-r", "-l=replace11.yaml", "-o=+-m-mod11", "-m", "-y=modifyLineBreaks:environments:BeginStartsOnOwnLine:1;BodyStartsOnOwnLine:2;EndStartsOnOwnLine:2"], expected: "test5-m-mod11.tex" },
    RCase { input: "test6.tex", args: &["-s", "-r", "-l=replace12.yaml", "-o=+-mod12"], expected: "test6-mod12.tex" },
    RCase { input: "test7.tex", args: &["-s", "-r", "-l=replace13.yaml", "-o=+-mod13"], expected: "test7-mod13.tex" },
    RCase { input: "test7.tex", args: &["-s", "-r", "-l=replace14.yaml", "-o=+-mod14"], expected: "test7-mod14.tex" },
    RCase { input: "test8.tex", args: &["-s", "-r", "-l=replace15.yaml", "-o=+-mod15"], expected: "test8-mod15.tex" },
    RCase { input: "test9.tex", args: &["-s", "-r", "-l=replace16.yaml", "-o=+-mod16"], expected: "test9-mod16.tex" },
    RCase { input: "test9.tex", args: &["-s", "-replacement", "-l=replace17.yaml", "-o=+-mod17"], expected: "test9-mod17.tex" },
    // verbatim 相关
    RCase { input: "test10.tex", args: &["-s", "-r", "-l=replace14.yaml", "-o=+-mod14"], expected: "test10-mod14.tex" },
    RCase { input: "test10.tex", args: &["-s", "-rr", "-l=replace14.yaml", "-o=+-rr-mod14"], expected: "test10-rr-mod14.tex" },
    RCase { input: "test10.tex", args: &["-s", "-rv", "-l=replace14.yaml", "-o=+-rv-mod14"], expected: "test10-rv-mod14.tex" },
    RCase { input: "test12.tex", args: &["-s", "-r", "-l=replace19.yaml,../specials/special-verb1.yaml", "-o=+-mod19"], expected: "test12-mod19.tex" },
    RCase { input: "test12.tex", args: &["-s", "-rv", "-l=replace19.yaml,../specials/special-verb1.yaml", "-o=+-rv-mod19"], expected: "test12-rv-mod19.tex" },
    // 去 preamble
    RCase { input: "test11.tex", args: &["-s", "-r", "-l=replace18.yaml", "-o=+-mod18"], expected: "test11-mod18.tex" },
    // issue 14
    RCase { input: "test13.tex", args: &["-m", "-s", "-r", "-l=replace20.yaml", "-o=+-mod20"], expected: "test13-mod20.tex" },
    RCase { input: "JHenneberg.tex", args: &["-m", "-s", "-r", "-l=replace20.yaml", "-o=+-mod20"], expected: "JHenneberg-mod20.tex" },
    // amalgamate：多 YAML 合并
    RCase { input: "test7.tex", args: &["-r", "-l=replace21,replace22", "-s", "-o=+-mod2122"], expected: "test7-mod2122.tex" },
    RCase { input: "test7.tex", args: &["-r", "-l=replace21,replace23", "-s", "-o=+-mod2123"], expected: "test7-mod2123.tex" },
    RCase { input: "test7.tex", args: &["-r", "-l=replace23,replace24", "-s", "-o=+-mod2324"], expected: "test7-mod2324.tex" },
    // 上/下标
    RCase { input: "test14.tex", args: &["-r", "-l=replace25", "-s", "-o=+-mod25"], expected: "test14-mod25.tex" },
    // L66 test14-mod26.tex：replace26.yaml 的 substitution 带 /e（Perl eval），T2 依赖 → 跳过
    // 双注释压缩
    RCase { input: "issue-464.tex", args: &["-s", "-r", "-l", "issue-464.yaml", "-m", "-o=+-mod1"], expected: "issue-464-mod1.tex" },
    RCase { input: "issue-489.tex", args: &["-s", "-l", "-m", "-r", "-o=+-mod1"], expected: "issue-489-mod1.tex" },
    RCase { input: "issue-503.tex", args: &["-s", "-l", "issue-503.yaml", "-r", "-o=+-mod1"], expected: "issue-503-mod1.tex" },
    RCase { input: "issue-503.tex", args: &["-s", "-l", "issue-503a.yaml", "-r", "-o=+-mod2"], expected: "issue-503-mod2.tex" },
    RCase { input: "issue-503a.tex", args: &["-s", "-l", "issue-503b.yaml", "-r", "-o=+-mod1"], expected: "issue-503a-mod1.tex" },
    // L78/79/80 issue-513*.dtx：issue-513.yaml 的 substitution 带 /e（@sgmxe），T2 依赖 → 跳过
    RCase { input: "issue-517.tex", args: &["-s", "-l", "issue-517.yaml", "-r", "-o=+-mod1"], expected: "issue-517-mod1.tex" },
    // L83 issue-517-mod2.tex：issue-517a.yaml 的 substitution 带 /e（/esgx），T2 依赖 → 跳过
    RCase { input: "issue-517.tex", args: &["-s", "-l", "issue-517b.yaml", "-rv", "-o=+-mod3"], expected: "issue-517-mod3.tex" },
    RCase { input: "issue-523.tex", args: &["-s", "-l", "issue-523.yaml", "-r", "-o=+-mod1"], expected: "issue-523-mod1.tex" },
    RCase { input: "issue-523.tex", args: &["-s", "-l", "issue-523a.yaml", "-m", "-o=+-mod2"], expected: "issue-523-mod2.tex" },
    // L89 issue-524-mod1.tex：issue-524.yaml 的 substitution 带 /e（@sgxe），T2 依赖 → 跳过
    RCase { input: "issue-637.tex", args: &["-s", "-l", "issue-637.yaml", "-o=+-mod1"], expected: "issue-637-mod1.tex" },
];

/// 已无待修用例：10 条 `PENDING` 全部修好（见 doc/pending-work.md §5A.6）。
const PENDING: &[&str] = &[];

fn run_case(c: &RCase) {
    check_owned(
        "replacements",
        c.input,
        &c.args.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
        c.expected,
    );
}

// -rr 仅替换模式（上游脚本 L8，本轮搬运前已存在）
#[test]
fn only_replacement() {
    check(
        "replacements",
        "test1.tex",
        &["-s", "-rr", "-l=replace1.yaml", "-o=+-rr-mod1"],
        "test1-rr-mod1.tex",
    );
}

/// 补齐的替换用例（41 条）。收集全部失败后一次性 report。
#[test]
fn replacement_cases() {
    for c in CASES {
        if PENDING.contains(&c.expected) {
            continue;
        }
        run_case(c);
    }
    report();
}

// ---------------------------------------------------------------------------
// T2 依赖的 6 条（上游 L66 / L78-80 / L83 / L89，均引用带 `/e` 的替换 YAML；
// 见 `doc/verify/09-test-porting-audit.md` §3.2 与 `doc/pending-work.md` T2）。
// 断言已按上游原样搬运，当前默认不执行。
// ---------------------------------------------------------------------------

const E_CASES: &[RCase] = &[
    // L66：replace26.yaml（/sxge）
    RCase { input: "test14.tex", args: &["-r", "-l=replace26", "-s", "-o=+-mod26"], expected: "test14-mod26.tex" },
    // L78-80：issue-513.yaml（@sgmxe）
    RCase { input: "issue-513.dtx", args: &["-s", "-r", "-m", "-l", "issue-513.yaml", "-o=+-mod1"], expected: "issue-513-mod1.dtx" },
    RCase { input: "issue-513a.dtx", args: &["-s", "-r", "-m", "-l", "issue-513.yaml", "-o=+-mod1"], expected: "issue-513a-mod1.dtx" },
    RCase { input: "issue-513b.dtx", args: &["-s", "-m", "-r", "-l", "issue-513.yaml", "-o=+-mod1"], expected: "issue-513b-mod1.dtx" },
    // L83：issue-517a.yaml（/esgx）
    RCase { input: "issue-517.tex", args: &["-s", "-l", "issue-517a.yaml", "-r", "-o=+-mod2"], expected: "issue-517-mod2.tex" },
    // L89：issue-524.yaml（@sgxe）
    RCase { input: "issue-524.tex", args: &["-s", "-l", "issue-524.yaml", "-r", "-o=+-mod1"], expected: "issue-524-mod1.tex" },
];

#[test]
#[ignore = "T2 依赖：-r substitution 的 Perl /e（见 doc/pending-work.md T2）未实现"]
fn replacement_e_cases_pending() {
    for c in E_CASES {
        run_case(c);
    }
    report();
}


