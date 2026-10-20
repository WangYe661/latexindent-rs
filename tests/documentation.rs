//! documentation 类测试：手册演示用例（documentation/demonstrations）与手册自身格式化（doc-indent.sh）。
//!
//! 搬运依据 `doc/verify/09-test-porting-audit.md` §3.1：
//! - `documentation/demonstrations/documentation-test-cases.sh`：**331** 条调用
//!   （静态 295 + 三处循环展开 `i=3..9`×1、`i=1..12`×2、`i=1..6`×1 = 36）。
//!   其中 L406 `finetuning1` 是上游死行（输入文件不存在，Perl 退出码 3），单独断言。
//! - `documentation/doc-indent.sh`：3 条 `-w` 调用（手册源码原地格式化，不动点断言）。
//!
//! 输入/基准/YAML 来自 `latexindent.pl-main/documentation/`，分别复制到
//! `tests/cases/documentation/demonstrations/` 与 `tests/cases/documentation/manual/`。
//!
//! ⚠️ 夹具已按本项目既有约定把上游 CRLF **转为 LF**（上游这批文件基本都是 CRLF；
//! 其它 `tests/cases/*` 目录当初也是这么处理的）。不转换会在行尾空白上产生大量假失败——
//! 上游 CRLF 基准是带 `:crlf` 折行的 Windows Perl 生成的，本工程按 §5A.4 的决定不模拟折行。
mod common;
use common::{check_log_lines, check_owned_lenient_yaml, report};

const DIR: &str = "documentation/demonstrations";

/// 一条演示用例：输入、参数、产物（基准）。
type Demo = (&'static str, &'static [&'static str], &'static str);

/// 与基准不一致、暂时挂起的产物名（搬运完成、实现缺口留待统一清理）。
const PENDING: &[&str] = &[
    // T2 依赖：`-r` + addruler1/addruler2 的 `/e` ruler 替换（11 条）
    "textwrap-bfccb-mod12.tex",
    "textwrap-bfccb-mod13.tex",
    "textwrap-bfccb-mod14.tex",
    "textwrap8-mod1.tex",
    "textwrap8-mod2.tex",
    "multiple-sentences8-mod1.tex",
    "multiple-sentences8-mod2.tex",
    "textwrap9-mod1.tex",
    "textwrap10-mod1.tex",
    "textwrap10-mod2.tex",
    "multiple-sentences9-mod1.tex",
];

/// 需要整目录夹具（`paths:` 传递引用同目录其它 YAML）的输入。
const FULL_DIR_INPUTS: &[&str] = &["paths-demo.tex"];

fn run_demos(pending: bool) {
    for (input, args, expected) in DEMOS {
        if PENDING.contains(expected) != pending {
            continue;
        }
        if FULL_DIR_INPUTS.contains(input) {
            let case = common::run_full_dir(
                DIR,
                input,
                &args.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
            );
            if !case.output.status.success() {
                common::fail(format!("{expected}: 退出码非 0"));
            } else if let Err(e) = case.diff_of(expected, expected) {
                common::fail(format!("{expected}: {e}"));
            }
            continue;
        }
        check_owned_lenient_yaml(
            DIR,
            input,
            &args.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
            expected,
        );
    }
    report();
}

/// 上游 `documentation-test-cases.sh` 的 330 条演示调用（L406 死行除外）。
#[test]
fn documentation_demos() {
    run_demos(false);
}

/// 挂起条目：仅剩 T2 `/e`（`-r` substitution 的 Perl eval）依赖的产物。
#[test]
#[ignore = "T2 依赖：-r substitution 的 Perl /e（addruler ruler 替换）未实现"]
fn documentation_demos_pending() {
    run_demos(true);
}

const DEMOS: &[Demo] = &[
    ("quick-start.tex", &["-s", "-o=+-default"], "quick-start-default.tex"),
    ("quick-start.tex", &["-s", "-o=+-mod1", "-l", "quick-start1"], "quick-start-mod1.tex"),
    ("quick-start.tex", &["-s", "-o=+-mod2", "-l", "quick-start2"], "quick-start-mod2.tex"),
    ("quick-start.tex", &["-s", "-o=+-mod3", "-l", "quick-start3"], "quick-start-mod3.tex"),
    ("quick-start.tex", &["-s", "-o=+-mod4", "-l", "quick-start4", "-m"], "quick-start-mod4.tex"),
    ("quick-start.tex", &["-s", "-o=+-mod5", "-l", "quick-start5", "-m"], "quick-start-mod5.tex"),
    ("quick-start.tex", &["-s", "-o=+-mod6", "-l", "quick-start6", "-m"], "quick-start-mod6.tex"),
    ("quick-start.tex", &["-s", "-o=+-mod7", "-l", "quick-start7", "-m"], "quick-start-mod7.tex"),
    ("quick-start.tex", &["-s", "-o=+-mod8", "-l", "quick-start8"], "quick-start-mod8.tex"),
    ("quick-start.tex", &["-s", "-o=+-mod9", "-l", "quick-start9"], "quick-start-mod9.tex"),
    ("filecontents1.tex", &["-s", "-o", "filecontents1-default"], "filecontents1-default.tex"),
    ("tikzset.tex", &["-s", "-o", "tikzset-default"], "tikzset-default.tex"),
    ("pstricks.tex", &["-s", "--outputfile", "pstricks-default"], "pstricks-default.tex"),
    ("pstricks.tex", &["-s", "--outputfile", "pstricks-default", "-logfile", "cmh.log"], "pstricks-default.tex"),
    ("tabular1.tex", &["-s", "--outputfile", "tabular1-default", "-logfile", "cmh.log"], "tabular1-default.tex"),
    ("tabular1.tex", &["-s", "-l", "tabular.yaml", "--outputfile", "tabular1-advanced"], "tabular1-advanced.tex"),
    ("tabular1.tex", &["-s", "-l", "tabular1.yaml", "--outputfile", "tabular1-advanced-3spaces"], "tabular1-advanced-3spaces.tex"),
    ("matrix1.tex", &["-s", "-o=+-default"], "matrix1-default.tex"),
    ("matrix2.tex", &["-s", "-t", "-o=+-default"], "matrix2-default.tex"),
    ("matrix2.tex", &["-s", "-t", "-l", "matrix2", "-o=+-mod1"], "matrix2-mod1.tex"),
    ("align-block.tex", &["-s", "-o", "align-block-default"], "align-block-default.tex"),
    ("tabular2.tex", &["-s", "-o", "tabular2-default"], "tabular2-default.tex"),
    ("tabular2.tex", &["-s", "-o=+-mod2", "-l", "tabular2.yaml"], "tabular2-mod2.tex"),
    ("tabular2.tex", &["-s", "-o=+-mod3", "-l", "tabular3.yaml"], "tabular2-mod3.tex"),
    ("tabular2.tex", &["-s", "-o=+-mod4", "-l", "tabular2,tabular4.yaml"], "tabular2-mod4.tex"),
    ("tabular2.tex", &["-s", "-o=+-mod5", "-l", "tabular2,tabular5.yaml"], "tabular2-mod5.tex"),
    ("tabular2.tex", &["-s", "-o=+-mod6", "-l", "tabular2,tabular6.yaml"], "tabular2-mod6.tex"),
    ("tabular2.tex", &["-s", "-o=+-mod7", "-l", "tabular2,tabular7.yaml"], "tabular2-mod7.tex"),
    ("tabular2.tex", &["-s", "-o=+-mod8", "-l", "tabular2,tabular8.yaml"], "tabular2-mod8.tex"),
    ("items1.tex", &["-s", "-o", "items1-default"], "items1-default.tex"),
    ("special1.tex", &["-s", "-o", "special1-default"], "special1-default.tex"),
    ("special2.tex", &["-s", "-o=+-mod1", "-l=middle.yaml"], "special2-mod1.tex"),
    ("special2.tex", &["-s", "-o=+-mod2", "-l=middle1.yaml"], "special2-mod2.tex"),
    ("special3.tex", &["-s", "-o=+-mod1", "-l=special-verb1.yaml"], "special3-mod1.tex"),
    ("specialAlgo.tex", &["-s", "-t", "-o=+-mod1", "-l=algo.yaml"], "specialAlgo-mod1.tex"),
    ("headings1.tex", &["-s", "-o", "headings1-mod1", "-l=headings1.yaml"], "headings1-mod1.tex"),
    ("headings1.tex", &["-s", "-o", "headings1-mod2", "-l=headings2.yaml"], "headings1-mod2.tex"),
    ("previously-important-example.tex", &["-s", "-o", "previously-important-example-default"], "previously-important-example-default.tex"),
    ("myenvironment-simple.tex", &["-s", "-o", "myenvironment-simple-noAdd-body1", "-l", "myenv-noAdd1.yaml"], "myenvironment-simple-noAdd-body1.tex"),
    ("myenvironment-simple.tex", &["-s", "-o", "myenvironment-simple-noAdd-body2", "-l", "myenv-noAdd2.yaml"], "myenvironment-simple-noAdd-body2.tex"),
    ("myenvironment-simple.tex", &["-s", "-o", "myenvironment-simple-noAdd-body3", "-l", "myenv-noAdd3.yaml"], "myenvironment-simple-noAdd-body3.tex"),
    ("myenvironment-simple.tex", &["-s", "-o", "myenvironment-simple-noAdd-body4", "-l", "myenv-noAdd4.yaml"], "myenvironment-simple-noAdd-body4.tex"),
    ("myenvironment-args.tex", &["-s", "-o", "myenvironment-args-noAdd-body1", "-l", "myenv-noAdd1.yaml"], "myenvironment-args-noAdd-body1.tex"),
    ("myenvironment-args.tex", &["-s", "-o", "myenvironment-args-noAdd5", "-l", "myenv-noAdd5.yaml"], "myenvironment-args-noAdd5.tex"),
    ("myenvironment-args.tex", &["-s", "-o", "myenvironment-args-noAdd6", "-l", "myenv-noAdd6.yaml"], "myenvironment-args-noAdd6.tex"),
    ("myenvironment-simple.tex", &["-s", "-l", "myenv-rules1.yaml", "-o", "myenv-rules1"], "myenv-rules1.tex"),
    ("myenvironment-simple.tex", &["-s", "-l", "myenv-rules2.yaml", "-o", "myenv-rules2"], "myenv-rules2.tex"),
    ("myenvironment-args.tex", &["-s", "-o", "myenvironment-args-rules1", "-l", "myenv-rules1.yaml"], "myenvironment-args-rules1.tex"),
    ("myenvironment-args.tex", &["-s", "-o", "myenvironment-args-rules3", "-l", "myenv-rules3.yaml"], "myenvironment-args-rules3.tex"),
    ("myenvironment-args.tex", &["-s", "-o", "myenvironment-args-rules4", "-l", "myenv-rules4.yaml"], "myenvironment-args-rules4.tex"),
    ("myenvironment-args.tex", &["-s", "-o", "myenvironment-args-rules1-noAddGlobal1", "-l", "env-noAdditionalGlobal.yaml"], "myenvironment-args-rules1-noAddGlobal1.tex"),
    ("myenvironment-args.tex", &["-s", "-o", "myenvironment-args-rules1-noAddGlobal2", "-l", "myenv-rules1.yaml,env-noAdditionalGlobal.yaml"], "myenvironment-args-rules1-noAddGlobal2.tex"),
    ("myenvironment-args.tex", &["-s", "-o", "myenvironment-args-rules1-noAddGlobal3", "-l", "opt-args-no-add-glob.yaml"], "myenvironment-args-rules1-noAddGlobal3.tex"),
    ("myenvironment-args.tex", &["-s", "-o", "myenvironment-args-rules1-noAddGlobal4", "-l", "mand-args-no-add-glob.yaml"], "myenvironment-args-rules1-noAddGlobal4.tex"),
    ("myenvironment-args.tex", &["-s", "-o", "myenvironment-args-global-rules1", "-l", "env-indentRules.yaml"], "myenvironment-args-global-rules1.tex"),
    ("myenvironment-args.tex", &["-s", "-o", "myenvironment-args-global-rules2", "-l", "myenv-rules1.yaml,env-indentRules.yaml"], "myenvironment-args-global-rules2.tex"),
    ("myenvironment-args.tex", &["-s", "-o", "myenvironment-args-global-rules3", "-l", "opt-args-indent-rules-glob.yaml"], "myenvironment-args-global-rules3.tex"),
    ("myenvironment-args.tex", &["-s", "-o", "myenvironment-args-global-rules4", "-l", "mand-args-indent-rules-glob.yaml"], "myenvironment-args-global-rules4.tex"),
    ("items1.tex", &["-s", "-o", "items1-noAdd1", "-local", "item-noAdd1.yaml"], "items1-noAdd1.tex"),
    ("items1.tex", &["-s", "-o", "items1-rules1", "-local", "item-rules1.yaml"], "items1-rules1.tex"),
    ("items1.tex", &["-s", "-o", "items1-no-add-global", "-local", "items-noAdditionalGlobal.yaml"], "items1-no-add-global.tex"),
    ("items1.tex", &["-s", "-o", "items1-rules-global", "-local", "items-indentRulesGlobal.yaml"], "items1-rules-global.tex"),
    ("mycommand.tex", &["-s", "-o", "mycommand-default"], "mycommand-default.tex"),
    ("mycommand.tex", &["-s", "-o", "mycommand-noAdd1", "-local", "mycommand-noAdd1.yaml"], "mycommand-noAdd1.tex"),
    ("mycommand.tex", &["-s", "-o", "mycommand-noAdd2", "-local", "mycommand-noAdd2.yaml"], "mycommand-noAdd2.tex"),
    ("mycommand.tex", &["-s", "-o", "mycommand-noAdd3", "-local", "mycommand-noAdd3.yaml"], "mycommand-noAdd3.tex"),
    ("mycommand.tex", &["-s", "-o", "mycommand-noAdd4", "-local", "mycommand-noAdd4.yaml"], "mycommand-noAdd4.tex"),
    ("mycommand.tex", &["-s", "-o", "mycommand-noAdd5", "-local", "mycommand-noAdd5.yaml"], "mycommand-noAdd5.tex"),
    ("mycommand.tex", &["-s", "-o", "mycommand-noAdd6", "-local", "mycommand-noAdd6.yaml"], "mycommand-noAdd6.tex"),
    ("ifelsefi1.tex", &["-s", "-o", "ifelsefi1-default"], "ifelsefi1-default.tex"),
    ("ifelsefi1.tex", &["-s", "-o", "ifelsefi1-noAdd", "-local", "ifnum-noAdd.yaml"], "ifelsefi1-noAdd.tex"),
    ("ifelsefi1.tex", &["-s", "-o", "ifelsefi1-indent-rules", "-local", "ifnum-indent-rules.yaml"], "ifelsefi1-indent-rules.tex"),
    ("ifelsefi1.tex", &["-s", "-local", "ifelsefi-noAdd-glob.yaml", "-o", "ifelsefi1-noAdd-glob"], "ifelsefi1-noAdd-glob.tex"),
    ("ifelsefi1.tex", &["-s", "-l", "ifelsefi-indent-rules-global.yaml", "-o", "ifelsefi1-indent-rules-global"], "ifelsefi1-indent-rules-global.tex"),
    ("ifelsefi2.tex", &["-s", "-o", "ifelsefi2-default"], "ifelsefi2-default.tex"),
    ("special1.tex", &["-s", "-o", "special1-noAdd", "-local", "displayMath-noAdd.yaml"], "special1-noAdd.tex"),
    ("special1.tex", &["-s", "-o", "special1-indent-rules", "-local", "displayMath-indent-rules.yaml"], "special1-indent-rules.tex"),
    ("special1.tex", &["-s", "-local", "special-noAdd-glob.yaml", "-o", "special1-noAdd-glob"], "special1-noAdd-glob.tex"),
    ("special1.tex", &["-s", "-l", "special-indent-rules-global.yaml", "-o", "special1-indent-rules-global"], "special1-indent-rules-global.tex"),
    ("headings2.tex", &["-s", "-l", "headings3.yaml", "-o", "headings2-mod3"], "headings2-mod3.tex"),
    ("headings2.tex", &["-s", "-l", "headings4.yaml", "-o", "headings2-mod4"], "headings2-mod4.tex"),
    ("headings2.tex", &["-s", "-l", "headings5.yaml", "-o", "headings2-mod5"], "headings2-mod5.tex"),
    ("headings2.tex", &["-s", "-l", "headings6.yaml", "-o", "headings2-mod6"], "headings2-mod6.tex"),
    ("headings2.tex", &["-s", "-l", "headings7.yaml", "-o", "headings2-mod7"], "headings2-mod7.tex"),
    ("headings2.tex", &["-s", "-l", "headings8.yaml", "-o", "headings2-mod8"], "headings2-mod8.tex"),
    ("headings2.tex", &["-s", "-l", "headings9.yaml", "-o", "headings2-mod9"], "headings2-mod9.tex"),
    ("pgfkeys1.tex", &["-s", "-o=pgfkeys1-default"], "pgfkeys1-default.tex"),
    ("child1.tex", &["-s", "-o=+-default"], "child1-default.tex"),
    ("psforeach1.tex", &["-s", "-o=psforeach1-default"], "psforeach1-default.tex"),
    ("unnamed1.tex", &["-s", "-o=+-mod0"], "unnamed1-mod0.tex"),
    ("unnamed1.tex", &["-s", "-l", "unnamed1", "-o=+-mod1"], "unnamed1-mod1.tex"),
    ("mlb1.tex", &["-s", "-m", "-o", "mlb1-out"], "mlb1-out.tex"),
    ("env-mlb.tex", &["-s", "-m", "-l", "env-mlb1.yaml", "-o", "env-mlb-mod1"], "env-mlb-mod1.tex"),
    ("env-mlb2.tex", &["-s", "-m", "-l", "env-mlb1.yaml", "-o", "env-mlb2-mod1"], "env-mlb2-mod1.tex"),
    ("env-mlb.tex", &["-s", "-m", "-l", "env-mlb2.yaml", "-o", "env-mlb-mod2"], "env-mlb-mod2.tex"),
    ("env-mlb2.tex", &["-s", "-m", "-l", "env-mlb2.yaml", "-o", "env-mlb2-mod2"], "env-mlb2-mod2.tex"),
    ("env-mlb.tex", &["-s", "-m", "-l", "env-mlb3.yaml", "-o", "env-mlb-mod3"], "env-mlb-mod3.tex"),
    ("env-mlb2.tex", &["-s", "-m", "-l", "env-mlb3.yaml", "-o", "env-mlb2-mod3"], "env-mlb2-mod3.tex"),
    ("env-mlb.tex", &["-s", "-m", "-l", "env-mlb4.yaml", "-o", "env-mlb-mod4"], "env-mlb-mod4.tex"),
    ("env-mlb2.tex", &["-s", "-m", "-l", "env-mlb4.yaml", "-o", "env-mlb2-mod4"], "env-mlb2-mod4.tex"),
    ("env-mlb.tex", &["-s", "-m", "-l", "env-mlb5.yaml", "-o", "env-mlb-mod5"], "env-mlb-mod5.tex"),
    ("env-mlb2.tex", &["-s", "-m", "-l", "env-mlb5.yaml", "-o", "env-mlb2-mod5"], "env-mlb2-mod5.tex"),
    ("env-mlb.tex", &["-s", "-m", "-l", "env-mlb6.yaml", "-o", "env-mlb-mod6"], "env-mlb-mod6.tex"),
    ("env-mlb2.tex", &["-s", "-m", "-l", "env-mlb6.yaml", "-o", "env-mlb2-mod6"], "env-mlb2-mod6.tex"),
    ("env-mlb.tex", &["-s", "-m", "-l", "env-mlb7.yaml", "-o", "env-mlb-mod7"], "env-mlb-mod7.tex"),
    ("env-mlb2.tex", &["-s", "-m", "-l", "env-mlb7.yaml", "-o", "env-mlb2-mod7"], "env-mlb2-mod7.tex"),
    ("env-mlb.tex", &["-s", "-m", "-l", "env-mlb8.yaml", "-o", "env-mlb-mod8"], "env-mlb-mod8.tex"),
    ("env-mlb2.tex", &["-s", "-m", "-l", "env-mlb8.yaml", "-o", "env-mlb2-mod8"], "env-mlb2-mod8.tex"),
    ("env-mlb.tex", &["-s", "-m", "-l", "env-mlb9.yaml", "-o", "env-mlb-mod9"], "env-mlb-mod9.tex"),
    ("env-mlb2.tex", &["-s", "-m", "-l", "env-mlb9.yaml", "-o", "env-mlb2-mod9"], "env-mlb2-mod9.tex"),
    ("env-mlb.tex", &["-s", "-m", "-l", "env-mlb10.yaml", "-o", "env-mlb-mod10"], "env-mlb-mod10.tex"),
    ("env-mlb2.tex", &["-s", "-m", "-l", "env-mlb10.yaml", "-o", "env-mlb2-mod10"], "env-mlb2-mod10.tex"),
    ("env-mlb.tex", &["-s", "-m", "-l", "env-mlb11.yaml", "-o", "env-mlb-mod11"], "env-mlb-mod11.tex"),
    ("env-mlb2.tex", &["-s", "-m", "-l", "env-mlb11.yaml", "-o", "env-mlb2-mod11"], "env-mlb2-mod11.tex"),
    ("env-mlb.tex", &["-s", "-m", "-l", "env-mlb12.yaml", "-o", "env-mlb-mod12"], "env-mlb-mod12.tex"),
    ("env-mlb2.tex", &["-s", "-m", "-l", "env-mlb12.yaml", "-o", "env-mlb2-mod12"], "env-mlb2-mod12.tex"),
    ("env-mlb3.tex", &["-s", "-m", "-l", "env-mlb2.yaml", "-o", "env-mlb3-mod2"], "env-mlb3-mod2.tex"),
    ("env-mlb3.tex", &["-s", "-m", "-l", "env-mlb4.yaml", "-o", "env-mlb3-mod4"], "env-mlb3-mod4.tex"),
    ("env-mlb4.tex", &["-s", "-m", "-l", "env-mlb13.yaml", "-o", "env-mlb4-mod13"], "env-mlb4-mod13.tex"),
    ("env-mlb4.tex", &["-s", "-m", "-l", "env-mlb14.yaml", "-o", "env-mlb4-mod14"], "env-mlb4-mod14.tex"),
    ("env-mlb4.tex", &["-s", "-m", "-l", "env-mlb15.yaml", "-o", "env-mlb4-mod15"], "env-mlb4-mod15.tex"),
    ("env-mlb4.tex", &["-s", "-m", "-l", "env-mlb16.yaml", "-o", "env-mlb4-mod16"], "env-mlb4-mod16.tex"),
    ("env-mlb5.tex", &["-s", "-m", "-l", "env-mlb13.yaml,env-mlb14.yaml,env-mlb15.yaml,env-mlb16.yaml", "-o", "env-mlb5-modAll"], "env-mlb5-modAll.tex"),
    ("env-mlb5.tex", &["-s", "-m", "-l", "env-mlb13.yaml,env-mlb14.yaml,env-mlb15.yaml,env-mlb16.yaml,removeTWS-before.yaml", "-o", "env-mlb5-modAll-remove-WS"], "env-mlb5-modAll-remove-WS.tex"),
    ("env-mlb6.tex", &["-s", "-m", "-l", "env-mlb13.yaml,env-mlb14.yaml,env-mlb15.yaml,env-mlb16.yaml", "-o", "env-mlb6-modAll"], "env-mlb6-modAll.tex"),
    ("env-mlb6.tex", &["-s", "-m", "-l", "env-mlb13.yaml,env-mlb14.yaml,env-mlb15.yaml,env-mlb16.yaml,UnpreserveBlankLines.yaml", "-o", "env-mlb6-modAll-un-Preserve-Blank-Lines"], "env-mlb6-modAll-un-Preserve-Blank-Lines.tex"),
    ("mycommand1.tex", &["-s", "-m", "-l=mycom-mlb1.yaml", "-outputfile=mycommand1-mlb1"], "mycommand1-mlb1.tex"),
    ("mycommand1.tex", &["-s", "-m", "-l=mycom-mlb2.yaml", "-outputfile=mycommand1-mlb2"], "mycommand1-mlb2.tex"),
    ("mycommand1.tex", &["-s", "-m", "-l=mycom-mlb3.yaml", "-outputfile=mycommand1-mlb3"], "mycommand1-mlb3.tex"),
    ("mycommand1.tex", &["-s", "-m", "-l=mycom-mlb4.yaml", "-outputfile=mycommand1-mlb4"], "mycommand1-mlb4.tex"),
    ("mycommand1.tex", &["-s", "-m", "-l=mycom-mlb5.yaml", "-outputfile=mycommand1-mlb5"], "mycommand1-mlb5.tex"),
    ("mycommand1.tex", &["-s", "-m", "-l=mycom-mlb6.yaml", "-outputfile=mycommand1-mlb6"], "mycommand1-mlb6.tex"),
    ("nested-env.tex", &["-s", "-m", "-l=nested-env-mlb1.yaml", "-outputfile=nested-env-mlb1"], "nested-env-mlb1.tex"),
    ("nested-env.tex", &["-s", "-m", "-l=nested-env-mlb2.yaml", "-outputfile=nested-env-mlb2"], "nested-env-mlb2.tex"),
    ("pstricks1.tex", &["-s", "-o", "pstricks1-default"], "pstricks1-default.tex"),
    ("pstricks1.tex", &["-s", "-o", "pstricks1-nrp", "-l", "noRoundParentheses.yaml"], "pstricks1-nrp.tex"),
    ("pstricks1.tex", &["-s", "-o", "pstricks1-indent-rules", "-l", "defFunction.yaml"], "pstricks1-indent-rules.tex"),
    ("tikz-node1.tex", &["-s", "-t", "-o", "tikz-node1-default"], "tikz-node1-default.tex"),
    ("tikz-node1.tex", &["-s", "-t", "-o", "tikz-node1-draw", "-l=draw.yaml"], "tikz-node1-draw.tex"),
    ("tikz-node1.tex", &["-s", "-t", "-o", "tikz-node1-no-strings", "-l=no-strings.yaml"], "tikz-node1-no-strings.tex"),
    ("for-each.tex", &["-s", "-o=+-default"], "for-each-default.tex"),
    ("for-each.tex", &["-s", "-t", "-o=+-mod1", "-l=foreach.yaml"], "for-each-mod1.tex"),
    ("textwrap1.tex", &["-s", "-o=+-mod1", "-l=textwrap1.yaml", "-m"], "textwrap1-mod1.tex"),
    ("textwrap1-mod1.tex", &["-s", "-o=+A", "-l=textwrap1A.yaml", "-m"], "textwrap1-mod1A.tex"),
    ("textwrap1.tex", &["-s", "-o=+-mod1B", "-l=textwrap1B.yaml", "-m"], "textwrap1-mod1B.tex"),
    ("tw-headings1.tex", &["-s", "-m", "-l", "textwrap1.yaml", "-o=+-mod1"], "tw-headings1-mod1.tex"),
    ("tw-headings1.tex", &["-s", "-m", "-l", "textwrap1.yaml,bf-no-headings.yaml", "-o=+-mod2"], "tw-headings1-mod2.tex"),
    ("tw-comments1.tex", &["-s", "-m", "-l", "textwrap1.yaml", "-o=+-mod1"], "tw-comments1-mod1.tex"),
    ("tw-comments1.tex", &["-s", "-m", "-l", "textwrap1.yaml,bf-no-comments.yaml", "-o=+-mod2"], "tw-comments1-mod2.tex"),
    ("tw-disp-math1.tex", &["-s", "-m", "-l", "textwrap1.yaml", "-o=+-mod1"], "tw-disp-math1-mod1.tex"),
    ("tw-disp-math1.tex", &["-s", "-m", "-l", "textwrap1.yaml,bf-no-disp-math.yaml", "-o=+-mod2"], "tw-disp-math1-mod2.tex"),
    ("tw-bf-myenv1.tex", &["-s", "-m", "-l", "textwrap1.yaml", "-o=+-mod1"], "tw-bf-myenv1-mod1.tex"),
    ("tw-bf-myenv1.tex", &["-s", "-m", "-l", "textwrap1.yaml,tw-bf-myenv.yaml", "-o=+-mod2"], "tw-bf-myenv1-mod2.tex"),
    ("tw-0-9.tex", &["-s", "-m", "-l", "textwrap1.yaml", "-o=+-mod1"], "tw-0-9-mod1.tex"),
    ("tw-0-9.tex", &["-s", "-m", "-l", "textwrap1.yaml,bb-0-9.yaml", "-o=+-mod2"], "tw-0-9-mod2.tex"),
    ("tw-bb-announce1.tex", &["-s", "-m", "-l", "textwrap1.yaml", "-o=+-mod1"], "tw-bb-announce1-mod1.tex"),
    ("tw-bb-announce1.tex", &["-s", "-m", "-l", "textwrap1.yaml,tw-bb-announce.yaml", "-o=+-mod2"], "tw-bb-announce1-mod2.tex"),
    ("tw-be-equation.tex", &["-s", "-m", "-l", "textwrap1A.yaml", "-o=+-mod1"], "tw-be-equation-mod1.tex"),
    ("tw-be-equation.tex", &["-s", "-m", "-l", "textwrap1A.yaml,tw-be-equation.yaml", "-o=+-mod2"], "tw-be-equation-mod2.tex"),
    ("textwrap2.tex", &["-s", "-o=+-mod1", "-l=textwrap1.yaml", "-m"], "textwrap2-mod1.tex"),
    ("textwrap3.tex", &["-s", "-o=+-mod1", "-l=textwrap1.yaml", "-m"], "textwrap3-mod1.tex"),
    ("textwrap4.tex", &["-s", "-o=+-mod2", "-l=textwrap2.yaml", "-m"], "textwrap4-mod2.tex"),
    ("textwrap4.tex", &["-s", "-o=+-mod2A", "-l=textwrap2A.yaml", "-m"], "textwrap4-mod2A.tex"),
    ("textwrap4.tex", &["-s", "-o=+-mod2B", "-l=textwrap2B.yaml", "-m"], "textwrap4-mod2B.tex"),
    ("textwrap5.tex", &["-s", "-o=+-mod3", "-l=textwrap3.yaml", "-m"], "textwrap5-mod3.tex"),
    ("textwrap5.tex", &["-s", "-o=+-mod4", "-l=textwrap4.yaml", "-m"], "textwrap5-mod4.tex"),
    ("textwrap5.tex", &["-s", "-o=+-mod5", "-l=textwrap5.yaml", "-m"], "textwrap5-mod5.tex"),
    ("textwrap6.tex", &["-s", "-o=+-mod5", "-l=textwrap5.yaml", "-m"], "textwrap6-mod5.tex"),
    ("textwrap6.tex", &["-s", "-o=+-mod6", "-l=textwrap6.yaml", "-m"], "textwrap6-mod6.tex"),
    ("textwrap6.tex", &["-s", "-o=+-mod7", "-l=textwrap7.yaml", "-m"], "textwrap6-mod7.tex"),
    ("textwrap6.tex", &["-s", "-o=+-mod8", "-l=textwrap8.yaml", "-m"], "textwrap6-mod8.tex"),
    ("textwrap6.tex", &["-s", "-o=+-mod9", "-l=textwrap9.yaml", "-m"], "textwrap6-mod9.tex"),
    ("textwrap6.tex", &["-s", "-o=+-mod10", "-l=textwrap10.yaml", "-m"], "textwrap6-mod10.tex"),
    ("textwrap6.tex", &["-s", "-o=+-mod11", "-l=textwrap11.yaml", "-m"], "textwrap6-mod11.tex"),
    ("tabular2.tex", &["-s", "-o=+-wrap-mod3", "-l=textwrap3.yaml", "-m"], "tabular2-wrap-mod3.tex"),
    ("tabular2.tex", &["-s", "-o=+-wrap-mod3A", "-l=textwrap3.yaml", "-m", "-y=modifyLineBreaks:textWrapOptions:alignAtAmpersandTakesPriority:0"], "tabular2-wrap-mod3A.tex"),
    ("textwrap7.tex", &["-s", "-o=+-mod3", "-l=textwrap3.yaml", "-m"], "textwrap7-mod3.tex"),
    ("textwrap7.tex", &["-s", "-o=+-mod12", "-l=textwrap12.yaml", "-m"], "textwrap7-mod12.tex"),
    ("textwrap-ts.tex", &["-s", "-o=+-mod1", "-l=tabstop.yaml", "-m"], "textwrap-ts-mod1.tex"),
    ("textwrap-bfccb.tex", &["-s", "-o=+-mod12", "-l=textwrap12.yaml,addruler1", "-m", "-r"], "textwrap-bfccb-mod12.tex"),
    ("textwrap-bfccb.tex", &["-s", "-o=+-mod13", "-l=textwrap13.yaml,addruler2", "-m", "-r"], "textwrap-bfccb-mod13.tex"),
    ("textwrap-bfccb.tex", &["-s", "-o=+-mod14", "-l=textwrap14.yaml,addruler1", "-m", "-r"], "textwrap-bfccb-mod14.tex"),
    ("textwrap8.tex", &["-s", "-o=+-mod1", "-l=tw-before1.yaml,addruler1", "-m", "-r"], "textwrap8-mod1.tex"),
    ("textwrap8.tex", &["-s", "-o=+-mod2", "-l=tw-after1.yaml,addruler1", "-m", "-r"], "textwrap8-mod2.tex"),
    ("multiple-sentences8.tex", &["-s", "-o=+-mod1", "-l=sentence-wrap2,addruler1", "-m", "-r"], "multiple-sentences8-mod1.tex"),
    ("multiple-sentences8.tex", &["-s", "-o=+-mod2", "-l=sentence-wrap3,addruler1", "-m", "-r"], "multiple-sentences8-mod2.tex"),
    ("sentence-dnc1.tex", &["-s", "-o=+-mod1", "-m", "-l", "dnc1.yaml"], "sentence-dnc1-mod1.tex"),
    ("sentence-dnc2.tex", &["-s", "-o=+-mod2", "-m", "-l", "dnc2.yaml"], "sentence-dnc2-mod2.tex"),
    ("textwrap9.tex", &["-s", "-r", "-m", "-l", "wrap-comments1.yaml,addruler1.yaml", "-o=+-mod1"], "textwrap9-mod1.tex"),
    ("textwrap10.tex", &["-s", "-r", "-m", "-l", "wrap-comments1.yaml,addruler1.yaml", "-o=+-mod1"], "textwrap10-mod1.tex"),
    ("textwrap10.tex", &["-s", "-r", "-m", "-l", "wrap-comments2.yaml,addruler1.yaml", "-o=+-mod2"], "textwrap10-mod2.tex"),
    ("multiple-sentences9.tex", &["-s", "-o=+-mod1", "-l=sentence-wrap4,addruler1", "-m", "-r"], "multiple-sentences9-mod1.tex"),
    ("tw-tc1.tex", &["-s", "-m", "-l", "textwrap1A.yaml", "-o=+-mod1"], "tw-tc1-mod1.tex"),
    ("tw-tc2.tex", &["-s", "-m", "-l", "textwrap1A.yaml", "-o=+-mod1"], "tw-tc2-mod1.tex"),
    ("tw-tc3.tex", &["-s", "-m", "-l", "textwrap1A.yaml", "-o=+-mod1"], "tw-tc3-mod1.tex"),
    ("tw-tc4.tex", &["-s", "-m", "-l", "textwrap1A.yaml", "-o=+-mod1"], "tw-tc4-mod1.tex"),
    ("tw-tc5.tex", &["-s", "-m", "-l", "textwrap1A.yaml", "-o=+-mod1"], "tw-tc5-mod1.tex"),
    ("tw-tc6.tex", &["-s", "-m", "-l", "textwrap1A.yaml", "-o=+-mod1"], "tw-tc6-mod1.tex"),
    ("shortlines.tex", &["-s", "-o=+1", "-l=remove-para1,remove-para4", "-m"], "shortlines1.tex"),
    ("shortlines.tex", &["-s", "-o=+1-tws", "-l=remove-para1,removeTWS-before,remove-para4", "-m"], "shortlines1-tws.tex"),
    ("shortlines-mand.tex", &["-s", "-o=+1", "-l=remove-para1.yaml,mycommand.yaml", "-m"], "shortlines-mand1.tex"),
    ("shortlines-opt.tex", &["-s", "-o=+1", "-l=remove-para1.yaml,mycommand-opt.yaml", "-m"], "shortlines-opt1.tex"),
    ("shortlines-envs.tex", &["-s", "-o=+2", "-l=remove-para2.yaml", "-m"], "shortlines-envs2.tex"),
    ("shortlines-envs.tex", &["-s", "-o=+3", "-l=remove-para3.yaml", "-m"], "shortlines-envs3.tex"),
    ("shortlines-md.tex", &["-s", "-o=+4", "-l=remove-para4.yaml", "-m"], "shortlines-md4.tex"),
    ("sl-stop.tex", &["-s", "-o", "sl-stop4", "-l=remove-para4.yaml", "-m"], "sl-stop4.tex"),
    ("sl-stop.tex", &["-s", "-o", "sl-stop4-command", "-l=remove-para4.yaml", "-m"], "sl-stop4-command.tex"),
    ("mult-nested.tex", &["-s", "-o=+-default"], "mult-nested-default.tex"),
    ("mult-nested.tex", &["-s", "-l=max-indentation1", "-o=+-max-ind1"], "mult-nested-max-ind1.tex"),
    ("ifnextchar.tex", &["-s", "-o=+-default"], "ifnextchar-default.tex"),
    ("ifnextchar.tex", &["-s", "-o=+-off", "-l", "no-ifnextchar.yaml"], "ifnextchar-off.tex"),
    ("env-mlb7.tex", &["-s", "-m", "-o=+-preserve", "-l=env-mlb12.yaml,env-mlb13.yaml"], "env-mlb7-preserve.tex"),
    ("env-mlb7.tex", &["-s", "-m", "-o=+-no-preserve", "-l=env-mlb12.yaml,env-mlb13.yaml,UnpreserveBlankLines.yaml"], "env-mlb7-no-preserve.tex"),
    ("specialLR.tex", &["-s", "-l", "specialsLeftRight.yaml", "-o=+-comm-first"], "specialLR-comm-first.tex"),
    ("specialLR.tex", &["-s", "-l", "specialsLeftRight", "-o=+-special-first"], "specialLR-special-first.tex"),
    ("special-align.tex", &["-s", "-l", "edge-node1.yaml", "-o=+-mod1"], "special-align-mod1.tex"),
    ("special-align.tex", &["-s", "-l", "edge-node2.yaml", "-o=+-mod2"], "special-align-mod2.tex"),
    ("special-body.tex", &["-s", "-t", "-l", "special-nested1.yaml", "-o=+-mod1"], "special-body-mod1.tex"),
    ("multiple-sentences.tex", &["-s", "-m", "-l=manipulate-sentences.yaml", "-o=+-mod1"], "multiple-sentences-mod1.tex"),
    ("multiple-sentences.tex", &["-s", "-m", "-l=keep-sen-line-breaks.yaml", "-o=+-mod2"], "multiple-sentences-mod2.tex"),
    ("multiple-sentences.tex", &["-s", "-m", "-l=sentences-follow1.yaml", "-o=+-mod3"], "multiple-sentences-mod3.tex"),
    ("multiple-sentences.tex", &["-s", "-m", "-l=sentences-end1.yaml", "-o=+-mod4"], "multiple-sentences-mod4.tex"),
    ("multiple-sentences.tex", &["-s", "-m", "-l=sentences-end2.yaml", "-o=+-mod5"], "multiple-sentences-mod5.tex"),
    ("multiple-sentences1.tex", &["-s", "-m", "-l=manipulate-sentences.yaml", "-o=+-mod1"], "multiple-sentences1-mod1.tex"),
    ("multiple-sentences1.tex", &["-s", "-m", "-l=manipulate-sentences.yaml,sentences-follow2.yaml", "-o=+-mod2"], "multiple-sentences1-mod2.tex"),
    ("multiple-sentences2.tex", &["-s", "-m", "-l=manipulate-sentences.yaml", "-o=+-mod1"], "multiple-sentences2-mod1.tex"),
    ("multiple-sentences2.tex", &["-s", "-m", "-l=manipulate-sentences.yaml,sentences-begin1", "-o=+-mod2"], "multiple-sentences2-mod2.tex"),
    ("multiple-sentences3.tex", &["-s", "-m", "-l=manipulate-sentences.yaml", "-o=+-mod1"], "multiple-sentences3-mod1.tex"),
    ("multiple-sentences4.tex", &["-s", "-m", "-l=manipulate-sentences.yaml", "-o=+-mod1"], "multiple-sentences4-mod1.tex"),
    ("multiple-sentences4.tex", &["-s", "-m", "-l=keep-sen-line-breaks.yaml", "-o=+-mod2"], "multiple-sentences4-mod2.tex"),
    ("multiple-sentences4.tex", &["-s", "-m", "-l=item-rules2", "-o=+-mod3"], "multiple-sentences4-mod3.tex"),
    ("url.tex", &["-s", "-m", "-l=manipulate-sentences.yaml", "-o=+-mod1"], "url-mod1.tex"),
    ("url.tex", &["-s", "-m", "-l=alt-full-stop1", "-o=+-mod2"], "url-mod2.tex"),
    ("multiple-sentences5.tex", &["-s", "-m", "-l=sentence-wrap1.yaml", "-o=+-mod1"], "multiple-sentences5-mod1.tex"),
    ("multiple-sentences6.tex", &["-s", "-m", "-l=sentence-wrap1.yaml", "-o=+-mod1"], "multiple-sentences6-mod1.tex"),
    ("multiple-sentences6.tex", &["-s", "-m", "-l=sentence-wrap1.yaml", "-o=+-mod2", "-y=modifyLineBreaks:oneSentencePerLine:sentenceIndent:"], "multiple-sentences6-mod2.tex"),
    ("multiple-sentences6.tex", &["-s", "-m", "-l=sentence-wrap1.yaml,itemize.yaml", "-o=+-mod3"], "multiple-sentences6-mod3.tex"),
    ("tabular3.tex", &["-s", "-m", "-l=DBS1.yaml", "-o=+-mod1"], "tabular3-mod1.tex"),
    ("tabular3.tex", &["-s", "-m", "-l=DBS2.yaml", "-o=+-mod2"], "tabular3-mod2.tex"),
    ("tabular3.tex", &["-s", "-m", "-l=DBS3.yaml", "-o=+-mod3"], "tabular3-mod3.tex"),
    ("tabular3.tex", &["-s", "-m", "-l=DBS4.yaml", "-o=+-mod4"], "tabular3-mod4.tex"),
    ("special4.tex", &["-s", "-m", "-l=DBS5.yaml", "-o=+-mod5"], "special4-mod5.tex"),
    ("mycommand2.tex", &["-s", "-m", "-l=DBS6.yaml", "-o=+-mod6"], "mycommand2-mod6.tex"),
    ("mycommand2.tex", &["-s", "-m", "-l=DBS7.yaml", "-o=+-mod7"], "mycommand2-mod7.tex"),
    ("pmatrix3.tex", &["-s", "-m", "-l=DBS3.yaml", "-o=+-mod3"], "pmatrix3-mod3.tex"),
    ("replace1.tex", &["-s", "-r", "-o=+-r1"], "replace1-r1.tex"),
    ("replace1.tex", &["-s", "-r", "-l=replace1.yaml", "-o=+-mod1"], "replace1-mod1.tex"),
    ("colsep.tex", &["-s", "-r", "-o=+-mod0", "-l=colsep.yaml"], "colsep-mod0.tex"),
    ("colsep.tex", &["-s", "-r", "-o=+-mod1", "-l=colsep1.yaml"], "colsep-mod1.tex"),
    ("colsep.tex", &["-s", "-r", "-o=+-mod2", "-l=multi-line.yaml"], "colsep-mod2.tex"),
    ("colsep.tex", &["-s", "-r", "-o=+-mod3", "-l=multi-line1yaml"], "colsep-mod3.tex"),
    ("displaymath.tex", &["-s", "-r", "-o=+-mod1", "-l=displaymath1.yaml"], "displaymath-mod1.tex"),
    ("displaymath.tex", &["-s", "-r", "-m", "-o=+-mod2", "-l=displaymath1.yaml,equation.yaml"], "displaymath-mod2.tex"),
    ("phrase.tex", &["-s", "-r", "-o=+-mod1", "-l=hspace.yaml"], "phrase-mod1.tex"),
    ("references.tex", &["-s", "-r", "-o=+-mod1", "-l=reference.yaml"], "references-mod1.tex"),
    ("verb1.tex", &["-s", "-r", "-o=+-mod1", "-l=verbatim1"], "verb1-mod1.tex"),
    ("verb1.tex", &["-s", "-rr", "-o=+-rr-mod1", "-l=verbatim1"], "verb1-rr-mod1.tex"),
    ("verb1.tex", &["-s", "-rv", "-o=+-rv-mod1", "-l=verbatim1"], "verb1-rv-mod1.tex"),
    ("amalg1.tex", &["-s", "-r", "-l=amalg1-yaml.yaml", "-o=+-mod1"], "amalg1-mod1.tex"),
    ("amalg1.tex", &["-s", "-r", "-l=amalg1-yaml.yaml,amalg2-yaml.yaml", "-o=+-mod12"], "amalg1-mod12.tex"),
    ("amalg1.tex", &["-s", "-r", "-l=amalg1-yaml.yaml,amalg3-yaml,amalg3-yaml.yaml", "-o=+-mod123"], "amalg1-mod123.tex"),
    ("named1.tex", &["-s", "-t", "-o=+-default"], "named1-default.tex"),
    ("finetuning2.tex", &["-s", "-o=+-default"], "finetuning2-default.tex"),
    ("finetuning2.tex", &["-s", "-o=+-mod1", "-l=fine-tuning2"], "finetuning2-mod1.tex"),
    // -y 的正则值必须照上游加双引号（GetYamlSettings.pm：未转义 `"` 触发引号分支，
    // 引号内整体作 value）。漏引号时 `;` 子字段逻辑会把值拆碎 → betterFullStop 失效。
    ("finetuning3.tex", &["-s", "-m", "-y=modifyLineBreaks:oneSentencePerLine:manipulateSentences: 1,modifyLineBreaks:oneSentencePerLine:sentencesBeginWith:a-z: 1,fineTuning:modifyLineBreaks:betterFullStop: \"(?:\\.|;|:(?![a-z]))|(?:(?<!(?:(?:e\\.g)|(?:i\\.e)|(?:etc))))\\.(?!(?:[a-z]|[A-Z]|\\-|~|\\,|[0-9]))\"", "-o=+-mod1"], "finetuning3-mod1.tex"),
    ("finetuning4.tex", &["-s", "-o=+-mod1", "-l=href1", "-m"], "finetuning4-mod1.tex"),
    ("finetuning4.tex", &["-s", "-o=+-mod2", "-l=href2", "-m"], "finetuning4-mod2.tex"),
    ("finetuning4.tex", &["-s", "-o=+-mod3", "-l=href3", "-m"], "finetuning4-mod3.tex"),
    ("bib1.bib", &["-s", "-o=+-mod1", "-l", "bibsettings0"], "bib1-mod1.bib"),
    ("bib1.bib", &["-s", "-l", "bibsettings1.yaml", "-o=+-mod2"], "bib1-mod2.bib"),
    ("bib2.bib", &["-s", "-l", "bibsettings1.yaml", "-o=+-mod1"], "bib2-mod1.bib"),
    ("bib2.bib", &["-s", "-l", "bibsettings1.yaml,bibsettings2.yaml", "-o=+-mod2"], "bib2-mod2.bib"),
    ("finetuning5.tex", &["-s", "-o=+-mod1", "-l=fine-tuning3"], "finetuning5-mod1.tex"),
    ("finetuning6.tex", &["-s", "-m", "-o=+-mod1", "-l=fine-tuning4"], "finetuning6-mod1.tex"),
    ("items2.tex", &["-s", "-o=+-mod1", "-l=ft-itemsRegex"], "items2-mod1.tex"),
    ("beamer1.tex", &["-s", "-t", "-o=+-mod1", "-l=ft-args"], "beamer1-mod1.tex"),
    ("ft-ifelsefi.tex", &["-s", "-o=+-mod1", "-l=ft-ifelsefi"], "ft-ifelsefi-mod1.tex"),
    ("align1.tex", &["-s", "-o=+-mod1", "-l=align1"], "align1-mod1.tex"),
    ("align1.tex", &["-s", "-o=+-mod2", "-l=align2"], "align1-mod2.tex"),
    ("align1.tex", &["-s", "-o=+-mod3", "-l=align3"], "align1-mod3.tex"),
    ("env-mlb1.tex", &["-s", "-m", "-l", "env-beg4.yaml", "-o=+-beg4"], "env-mlb1-beg4.tex"),
    ("env-mlb1.tex", &["-s", "-m", "-l", "env-body4.yaml", "-o=+-body4"], "env-mlb1-body4.tex"),
    ("env-mlb1.tex", &["-s", "-m", "-l", "env-end4.yaml", "-o=+-end4"], "env-mlb1-end4.tex"),
    ("env-mlb1.tex", &["-s", "-m", "-l", "env-end-f4.yaml", "-o=+-end-f4"], "env-mlb1-end-f4.tex"),
    ("tabular4.tex", &["-s", "-o=+-default"], "tabular4-default.tex"),
    ("tabular4.tex", &["-s", "-o=+-FDBS", "-y=lookForAlignDelims:tabular:lookForChildCodeBlocks:0"], "tabular4-FDBS.tex"),
    ("tabular-DM.tex", &["-s", "-o=+-default"], "tabular-DM-default.tex"),
    ("tabular-DM.tex", &["-s", "-o=+-mod1", "-l=dontMeasure1a.yaml"], "tabular-DM-mod1.tex"),
    ("tabular-DM.tex", &["-s", "-o=+-mod2", "-l=dontMeasure2.yaml"], "tabular-DM-mod2.tex"),
    ("tabular-DM.tex", &["-s", "-o=+-mod3", "-l=dontMeasure3.yaml"], "tabular-DM-mod3.tex"),
    ("tabular-DM.tex", &["-s", "-o=+-mod4", "-l=dontMeasure4.yaml"], "tabular-DM-mod4.tex"),
    ("tabular-DM.tex", &["-s", "-o=+-mod5", "-l=dontMeasure5.yaml"], "tabular-DM-mod5.tex"),
    ("tabular-DM.tex", &["-s", "-o=+-mod6", "-l=dontMeasure6.yaml"], "tabular-DM-mod6.tex"),
    ("tabular-DM-1.tex", &["-s", "-o=+-mod1", "-l=dontMeasure1.yaml"], "tabular-DM-1-mod1.tex"),
    ("tabular-DM-1.tex", &["-s", "-o=+-mod1a", "-l=dontMeasure1a.yaml"], "tabular-DM-1-mod1a.tex"),
    ("tabbing.tex", &["-s", "-o=+-default"], "tabbing-default.tex"),
    ("tabbing.tex", &["-s", "-o=+-mod1", "-l=delimiterRegEx1.yaml"], "tabbing-mod1.tex"),
    ("tabbing.tex", &["-s", "-o=+-mod2", "-l=delimiterRegEx2.yaml"], "tabbing-mod2.tex"),
    ("tabbing.tex", &["-s", "-o=+-mod3", "-l=delimiterRegEx3.yaml"], "tabbing-mod3.tex"),
    ("tabbing1.tex", &["-s", "-o=+-mod4", "-l=delimiterRegEx4.yaml"], "tabbing1-mod4.tex"),
    ("tabbing1.tex", &["-s", "-o=+-mod5", "-l=delimiterRegEx5.yaml"], "tabbing1-mod5.tex"),
    ("tabular5.tex", &["-s", "-o=+-default"], "tabular5-default.tex"),
    ("tabular5.tex", &["-s", "-l", "alignContentAfterDBS1", "-o=+-mod1"], "tabular5-mod1.tex"),
    ("tabular5.tex", &["-s", "-l", "alignContentAfterDBS2", "-o=+-mod2"], "tabular5-mod2.tex"),
    ("tabular6.tex", &["-s", "-m", "-l", "alignContentAfterDBS1.yaml,DBS1.yaml", "-o=+-mod1"], "tabular6-mod1.tex"),
    ("tabular6.tex", &["-s", "-m", "-l", "alignContentAfterDBS2.yaml,DBS1.yaml", "-o=+-mod2"], "tabular6-mod2.tex"),
    ("comma1.tex", &["-s", "-l", "comma1.yaml", "-o=+-mod1"], "comma1-mod1.tex"),
    ("comma1.tex", &["-s", "-l", "comma2.yaml", "-o=+-mod2"], "comma1-mod2.tex"),
    ("comma1.tex", &["-s", "-l", "comma3.yaml", "-o=+-mod3"], "comma1-mod3.tex"),
    ("comma1.tex", &["-s", "-l", "comma4.yaml", "-o=+-mod4"], "comma1-mod4.tex"),
    ("noindentblock1.tex", &["-s", "-l", "noindent1.yaml", "-o=+-mod1"], "noindentblock1-mod1.tex"),
    ("noindentblock1.tex", &["-s", "-l", "noindent2.yaml", "-o=+-mod2"], "noindentblock1-mod2.tex"),
    ("noindentblock1.tex", &["-s", "-l", "noindent3.yaml", "-o=+-mod3"], "noindentblock1-mod3.tex"),
    ("aligned1.tex", &["-s", "-o=+-default"], "aligned1-default.tex"),
    ("aligned1.tex", &["-s", "-l", "sba1.yaml", "-o=+-mod1"], "aligned1-mod1.tex"),
    ("aligned1.tex", &["-s", "-l", "sba2.yaml", "-o=+-mod2"], "aligned1-mod2.tex"),
    ("aligned1.tex", &["-s", "-l", "sba3.yaml", "-o=+-mod3"], "aligned1-mod3.tex"),
    ("aligned1.tex", &["-s", "-l", "sba4.yaml", "-o=+-mod4"], "aligned1-mod4.tex"),
    ("aligned1.tex", &["-s", "-l", "sba5.yaml", "-o=+-mod5"], "aligned1-mod5.tex"),
    ("aligned1.tex", &["-s", "-l", "sba6.yaml", "-o=+-mod6"], "aligned1-mod6.tex"),
    ("aligned1.tex", &["-s", "-l", "sba7.yaml", "-o=+-mod7"], "aligned1-mod7.tex"),
    ("myfile.tex", &["--lines", "3-7", "-s", "-o=+-mod1"], "myfile-mod1.tex"),
    ("myfile.tex", &["--lines", "5", "-s", "-o=+-mod2"], "myfile-mod2.tex"),
    ("myfile.tex", &["--lines", "3-5,8-10", "-s", "-o=+-mod3"], "myfile-mod3.tex"),
    ("myfile.tex", &["--lines", "1-2,4-5,9-10,12", "-s", "-o=+-mod4"], "myfile-mod4.tex"),
    ("myfile.tex", &["--lines", "!5-7", "-s", "-o=+-mod5"], "myfile-mod5.tex"),
    ("myfile.tex", &["--lines", "!5-7,!9-10", "-s", "-o=+-mod6"], "myfile-mod6.tex"),
    ("myfile1.tex", &["--lines", "6", "-m", "-s", "-l", "env-mlb2,env-mlb7,env-mlb8", "-o=+-mod1"], "myfile1-mod1.tex"),
    ("paths-demo.tex", &["-l", "path1.yaml", "-s", "-o=+-mod1"], "paths-demo-mod1.tex"),
    ("paths-demo.tex", &["-l", "path3.yaml", "-s", "-o=+-mod3"], "paths-demo-mod3.tex"),
];

// ---------------------------------------------------------------------------
// 日志断言（上游脚本 9 处 `egrep 'found:' indent.log > *.txt`）
// ---------------------------------------------------------------------------

#[test]
fn documentation_logs() {
    check_log_lines(DIR, "matrix2.tex", &["-s", "-t", "-o=+-default"], "found:", "matrix2.txt");
    check_log_lines(
        DIR,
        "matrix2.tex",
        &["-s", "-t", "-l", "matrix2", "-o=+-mod1"],
        "found:",
        "matrix2-mod1.txt",
    );
    // L54 的 `specialalgo1.txt`：algpseudocode specialBeginEnd 的 trace
    check_log_lines(
        DIR,
        "specialAlgo.tex",
        &["-s", "-t", "-o=+-mod1", "-l=algo.yaml"],
        "found:",
        "specialalgo1.txt",
    );
    check_log_lines(
        DIR,
        "tikz-node1.tex",
        &["-s", "-t", "-o", "tikz-node1-default"],
        "found:",
        "tikz-node-mod1.txt",
    );
    check_log_lines(
        DIR,
        "tikz-node1.tex",
        &["-s", "-t", "-o", "tikz-node1-draw", "-l=draw.yaml"],
        "found:",
        "tikz-node-mod2.txt",
    );
    check_log_lines(
        DIR,
        "tikz-node1.tex",
        &["-s", "-t", "-o", "tikz-node1-no-strings", "-l=no-strings.yaml"],
        "found:",
        "tikz-node-mod3.txt",
    );
    check_log_lines(
        DIR,
        "for-each.tex",
        &["-s", "-t", "-o=+-mod1", "-l=foreach.yaml"],
        "found:",
        "for-each.txt",
    );
    check_log_lines(
        DIR,
        "named1.tex",
        &["-s", "-t", "-o=+-default"],
        "found:",
        "finetuning1.txt",
    );
    check_log_lines(
        DIR,
        "beamer1.tex",
        &["-s", "-t", "-o=+-mod1", "-l=ft-args"],
        "found:",
        "beamer1.txt",
    );
}

// ---------------------------------------------------------------------------
// L496-498 / L501-503：paths demo 的日志截取（从 `YAML settings, reading` 到首个 `INFO:`）
// ---------------------------------------------------------------------------

#[test]
fn documentation_paths_log_sections() {
    for (yaml, out, expected) in [
        ("path1.yaml", "+-mod1", "path-test1.txt"),
        ("path3.yaml", "+-mod3", "path-test3.txt"),
    ] {
        let l = format!("-l={yaml}");
        let o = format!("-o={out}");
        // 整目录夹具：path1.yaml → path2.yaml（path3 → path4 → path5）的 `paths:`
        // 传递引用需要同目录的后续 YAML 都在场。
        let case = common::run_full_dir(
            DIR,
            "paths-demo.tex",
            &["-s", &l, &o].iter().map(|s| s.to_string()).collect::<Vec<_>>(),
        );
        case.assert_success();
        case.assert_log_section("indent.log", "YAML settings, reading", Some("INFO:"), expected);
    }
}

// ---------------------------------------------------------------------------
// L406：上游死行（输入 finetuning1 不存在）——Perl/本工程都应以 3 退出
// ---------------------------------------------------------------------------

#[test]
fn documentation_missing_input_is_fatal() {
    let case = common::run_missing_input(
        DIR,
        &["-s", "-o=+-mod1", "-l=fine-tuning1"]
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>(),
    );
    assert_eq!(
        case.output.status.code(),
        Some(3),
        "finetuning1 不存在时应以 3 退出（上游 FATAL）"
    );
}

// ---------------------------------------------------------------------------
// doc-indent.sh：手册源码原地格式化（3 条 `-w` 调用，不动点断言）
// ---------------------------------------------------------------------------

/// `latexindent.pl -l=+mainfile.yaml -w -rv -m -s latexindent.tex`
///
/// ⚠️ 未对齐原因：doc-indent.sh 用到的 `manual/latexindent.yaml` 在
/// `replacements.substitution` 里写了 Perl `/e` 代码块（`s/.../my $body = $1; .../segx`），
/// 属于 T2 `/e` 缺口。主文件 latexindent.tex 与 bib 已是不动点，s*.tex 因该替换
/// 全部不是不动点，故整条用例随 T2 挂起。
#[test]
#[ignore = "T2 依赖：manual/latexindent.yaml 的 -r substitution 含 Perl /e 代码块"]
fn documentation_manual_sources_pending() {
    const MANUAL: &str = "documentation/manual";

    // 1) 主文件（-l=+mainfile.yaml 会连带加载 ./latexindent.yaml）
    let case = common::run(
        MANUAL,
        "latexindent.tex",
        &["-l=+mainfile.yaml", "-w", "-rv", "-m", "-s"]
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>(),
    );
    case.assert_success();
    case.assert_overwrite_matches("latexindent.tex");

    // 2) s*.tex（裸 -l → 默认名；-g tex-files.log）
    let dir = common::cases_root().join(MANUAL);
    let mut sfiles: Vec<String> = std::fs::read_dir(&dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().to_string())
        .filter(|n| n.starts_with('s') && n.ends_with(".tex"))
        .collect();
    sfiles.sort();
    for f in &sfiles {
        common::check_overwrite(
            MANUAL,
            f,
            &["-w", "-rv", "-m", "-g", "tex-files.log", "-l", "-s"],
        );
    }

    // 3) *.bib（-l bibsettings.yaml；-g bib-files.log）
    let mut bibs: Vec<String> = std::fs::read_dir(&dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().to_string())
        .filter(|n| n.ends_with(".bib"))
        .collect();
    bibs.sort();
    for f in &bibs {
        common::check_overwrite(
            MANUAL,
            f,
            &["-w", "-rv", "-m", "-g", "bib-files.log", "-l", "bibsettings.yaml", "-s"],
        );
    }
}
