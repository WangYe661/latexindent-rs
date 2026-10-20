//! fineTuning 设置测试，基准取自 tests/cases/fine-tuning。
//! 对应上游 fine-tuning-test-cases.sh 的调用（19 个 tex 用例 + 3 个日志摘录用例）。
mod common;
use common::{check_log_lines, check_owned, report};

const DIR: &str = "fine-tuning";

fn a(v: &[&str]) -> Vec<String> {
    v.iter().map(|s| s.to_string()).collect()
}

#[test]
fn environments() {
    check_owned(DIR, "environment1.tex", &a(&["-s", "-o=+-mod0"]), "environment1-mod0.tex");
    check_owned(DIR, "environment1.tex", &a(&["-s", "-o=+-mod1", "-l=fine-tuning-env1"]), "environment1-mod1.tex");
    check_owned(DIR, "environment2.tex", &a(&["-s", "-o=+-mod0"]), "environment2-mod0.tex");
    check_owned(DIR, "environment2.tex", &a(&["-s", "-o=+-mod2", "-l=fine-tuning-env2"]), "environment2-mod2.tex");
    report();
}

#[test]
fn ifelsefi_and_commands() {
    check_owned(DIR, "ifelsefi1.tex", &a(&["-s", "-o=+-mod0"]), "ifelsefi1-mod0.tex");
    check_owned(DIR, "ifelsefi1.tex", &a(&["-s", "-o=+-mod1", "-l=fine-tuning-ifelsefi1"]), "ifelsefi1-mod1.tex");
    check_owned(DIR, "commands1.tex", &a(&["-s", "-o=+-mod0"]), "commands1-mod0.tex");
    check_owned(DIR, "commands1.tex", &a(&["-s", "-t", "-o=+-mod1", "-l=fine-tuning-commands1"]), "commands1-mod1.tex");
    check_log_lines(DIR, "commands1.tex", &["-s", "-t", "-o=+-mod1", "-l=fine-tuning-commands1"], "found:", "commands1-mod1.txt");
    report();
}

#[test]
fn key_equals_value_and_named() {
    check_owned(DIR, "keyequalsvalue1.tex", &a(&["-s", "-o=+-mod0"]), "keyequalsvalue1-mod0.tex");
    check_owned(DIR, "keyequalsvalue1.tex", &a(&["-s", "-t", "-o=+-mod1", "-l=fine-tuning-keyequalsvalue1"]), "keyequalsvalue1-mod1.tex");
    check_log_lines(DIR, "keyequalsvalue1.tex", &["-s", "-t", "-o=+-mod1", "-l=fine-tuning-keyequalsvalue1"], "found:", "keyequalsvalue1-mod1.txt");
    check_owned(DIR, "keyequalsvalue1.tex", &a(&["-s", "-t", "-o=+-mod2", "-l=fine-tuning-keyequalsvalue2"]), "keyequalsvalue1-mod2.tex");
    check_owned(DIR, "namedbracketsbraces1.tex", &a(&["-s", "-o=+-mod0"]), "namedbracketsbraces1-mod0.tex");
    check_owned(DIR, "namedbracketsbraces1.tex", &a(&["-s", "-t", "-o=+-mod1", "-l=fine-tuning-namedbracketsbraces1"]), "namedbracketsbraces1-mod1.tex");
    check_log_lines(DIR, "namedbracketsbraces1.tex", &["-s", "-t", "-o=+-mod1", "-l=fine-tuning-namedbracketsbraces1"], "found:", "namedbracketsbraces1-mod1.txt");
    check_owned(DIR, "namedbracketsbraces1.tex", &a(&["-s", "-o=+-mod2", "-l=fine-tuning-namedbracketsbraces2"]), "namedbracketsbraces1-mod2.tex");
    report();
}

#[test]
fn unnamed_and_arguments() {
    check_owned(DIR, "unnamedbracesbrackets1.tex", &a(&["-s", "-o=+-mod0", "-l=fine-tuning-unnamed"]), "unnamedbracesbrackets1-mod0.tex");
    check_owned(DIR, "unnamedbracesbrackets1.tex", &a(&["-s", "-o=+-mod1", "-l=fine-tuning-unnamed1"]), "unnamedbracesbrackets1-mod1.tex");
    check_owned(DIR, "arguments1.tex", &a(&["-s", "-o=+-mod0"]), "arguments1-mod0.tex");
    check_owned(DIR, "arguments1.tex", &a(&["-s", "-t", "-o=+-mod1", "-l=fine-tuning-args1"]), "arguments1-mod1.tex");
    check_owned(DIR, "arguments2.tex", &a(&["-s", "-o=+-mod0"]), "arguments2-mod0.tex");
    check_owned(DIR, "arguments2.tex", &a(&["-s", "-t", "-o=+-mod2", "-l=fine-tuning-args2"]), "arguments2-mod2.tex");
    report();
}

#[test]
fn trailing_comments_href() {
    check_owned(DIR, "href.tex", &a(&["-s", "-m", "-o=+-mod1", "-l=href1"]), "href-mod1.tex");
    check_owned(DIR, "href.tex", &a(&["-s", "-m", "-o=+-mod2", "-l=href2"]), "href-mod2.tex");
    report();
}
