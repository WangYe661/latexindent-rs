//! 可选+必选参数（opt-and-mand-args）测试，覆盖 tests/cases/opt-and-mand-args/opt-mand-args-test-cases.sh。
//! 含 `-y` 内联 comma poly-switch（CommaStartsOnOwnLine/CommaFinishesWithLineBreak）。
mod common;
use common::{check, check_owned, check_overwrite, report};

const OM: &str = "../opt-args/opt-args-remove-all,../environments/env-all-on";

#[test]
fn add_remove_linebreaks() {
    check("opt-and-mand-args", "env-opt-mand-args1.tex", &["-m", "-s", "-o=+-default", &format!("-l={OM}")], "env-opt-mand-args1-default.tex");
    check("opt-and-mand-args", "env-opt-mand-args1.tex", &["-m", "-s", "-o=+-addPercent-all", &format!("-l={OM},addPercentAll-mand,addPercentAll-opts")], "env-opt-mand-args1-addPercent-all.tex");
    check("opt-and-mand-args", "env-opt-mand-args1-remove-linebreaks1.tex", &["-m", "-s", "-o=+-mod1", &format!("-l={OM},opt-args-mod5,mand-args-mod5")], "env-opt-mand-args1-remove-linebreaks1-mod1.tex");
    check("opt-and-mand-args", "env-opt-mand-args1-remove-linebreaks1.tex", &["-m", "-s", "-o=+-mod2", &format!("-l={OM},opt-args-mod5,mand-args-mod5,unprotect-blank-lines")], "env-opt-mand-args1-remove-linebreaks1-mod2.tex");
}

#[test]
fn no_add_indent_rules() {
    let base = "env-opt-mand-args1-addPercent-all.tex";
    check("opt-and-mand-args", base, &["-s", &format!("-l={OM},noAdditionalIndentGlobal"), "-o=+-Global"], "env-opt-mand-args1-addPercent-all-Global.tex");
    check("opt-and-mand-args", base, &["-s", &format!("-l={OM},indentRulesGlobal"), "-o=+-indent-rules-Global"], "env-opt-mand-args1-addPercent-all-indent-rules-Global.tex");
    check("opt-and-mand-args", base, &["-s", "-t", &format!("-l={OM},indentRulesGlobal,indentRulesGlobalEnv"), "-o=+-indent-rules-all-Global"], "env-opt-mand-args1-addPercent-all-indent-rules-all-Global.tex");
}

// comma poly-switch（CommaStartsOnOwnLine / CommaFinishesWithLineBreak），i=-1..3
#[test]
fn loop_comma_poly() {
    for i in -1..=3 {
        check_owned("opt-and-mand-args", "mycommand.tex", &["-s".into(), "-m".into(), format!("-o=+-opt-mod{i}"), format!("-y=modifyLineBreaks:optionalArguments:CommaStartsOnOwnLine:{i}")], &format!("mycommand-opt-mod{i}.tex"));
        check_owned("opt-and-mand-args", "mycommand.tex", &["-s".into(), "-m".into(), format!("-o=+-mand-mod{i}"), format!("-y=modifyLineBreaks:mandatoryArguments:CommaStartsOnOwnLine:{i}")], &format!("mycommand-mand-mod{i}.tex"));
        check_owned("opt-and-mand-args", "mycommand.tex", &["-s".into(), "-m".into(), format!("-o=+-opt-finish-mod{i}"), format!("-y=modifyLineBreaks:optionalArguments:CommaFinishesWithLineBreak:{i}")], &format!("mycommand-opt-finish-mod{i}.tex"));
        check_owned("opt-and-mand-args", "mycommand.tex", &["-s".into(), "-m".into(), format!("-o=+-mand-finish-mod{i}"), format!("-y=modifyLineBreaks:mandatoryArguments:CommaFinishesWithLineBreak:{i}")], &format!("mycommand-mand-finish-mod{i}.tex"));
        check_owned("opt-and-mand-args", "mycommand.tex", &["-s".into(), "-m".into(), format!("-o=+-opt-mand-mod{i}"), format!("-y=modifyLineBreaks:optionalArguments:CommaStartsOnOwnLine:{i},modifyLineBreaks:mandatoryArguments:CommaStartsOnOwnLine:{i}")], &format!("mycommand-opt-mand-mod{i}.tex"));
        check_owned("opt-and-mand-args", "mycommand.tex", &["-s".into(), "-m".into(), format!("-o=+-opt-mand-finish-mod{i}"), format!("-y=modifyLineBreaks:optionalArguments:CommaFinishesWithLineBreak:{i},modifyLineBreaks:mandatoryArguments:CommaFinishesWithLineBreak:{i}")], &format!("mycommand-opt-mand-finish-mod{i}.tex"));
        check_owned("opt-and-mand-args", "mycommand2.tex", &["-s".into(), "-t".into(), "-m".into(), format!("-o=+-opt-mand-finish-mod{i}"), "-l=remove-blank-lines".into(), format!("-y=modifyLineBreaks:optionalArguments:CommaFinishesWithLineBreak:{i},modifyLineBreaks:mandatoryArguments:CommaFinishesWithLineBreak:{i},modifyLineBreaks:mandatoryArguments:CommaStartsOnOwnLine:{i},modifyLineBreaks:optionalArguments:CommaStartsOnOwnLine:{i}")], &format!("mycommand2-opt-mand-finish-mod{i}.tex"));
        check_owned("opt-and-mand-args", "mycommand1.tex", &["-s".into(), "-m".into(), format!("-o=+-opt-mod{i}"), format!("-y=modifyLineBreaks:optionalArguments:cmh:CommaStartsOnOwnLine:{i}")], &format!("mycommand1-opt-mod{i}.tex"));
        check_owned("opt-and-mand-args", "mycommand1.tex", &["-s".into(), "-m".into(), format!("-o=+-mand-mod{i}"), format!("-y=modifyLineBreaks:mandatoryArguments:another:CommaStartsOnOwnLine:{i}")], &format!("mycommand1-mand-mod{i}.tex"));
        check_owned("opt-and-mand-args", "mycommand1.tex", &["-s".into(), "-m".into(), format!("-o=+-mand-finish-mod{i}"), format!("-y=modifyLineBreaks:mandatoryArguments:another:CommaFinishesWithLineBreak:{i}")], &format!("mycommand1-mand-finish-mod{i}.tex"));
    }
    report();
}

#[test]
fn after_loop() {
    check("opt-and-mand-args", "heart.tex", &["-s", "-m", "-o=+-mod0", "-y=modifyLineBreaks:optionalArguments:CommaFinishesWithLineBreak:-1"], "heart-mod0.tex");
    check("opt-and-mand-args", "heart.tex", &["-s", "-m", "-r", "-l=heart1", "-o=+-mod1", "-y=modifyLineBreaks:optionalArguments:CommaFinishesWithLineBreak:-1"], "heart-mod1.tex");
    check("opt-and-mand-args", "issue-520.tex", &["-s", "-l=issue-520", "-m", "-o=+-mod1"], "issue-520-mod1.tex");
}
