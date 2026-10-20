//! 行范围开关（--lines/-n）测试，基准取自 tests/cases/line-switch-test-cases。
//! 对应上游 line-switch-test-cases.sh 的全部 28 个调用。
mod common;
use common::{check_owned, report};

const DIR: &str = "line-switch-test-cases";

fn a(v: &[&str]) -> Vec<String> {
    v.iter().map(|s| s.to_string()).collect()
}

#[test]
fn single_line_ranges() {
    let f = "environments-simple-nested.tex";
    check_owned(DIR, f, &a(&["--lines", "3-5", "-s", "-o=+-mod1"]), "environments-simple-nested-mod1.tex");
    check_owned(DIR, f, &a(&["--lines", "8-10", "-s", "-o=+-mod2"]), "environments-simple-nested-mod2.tex");
    // 区间顺序颠倒
    check_owned(DIR, f, &a(&["--lines", "10-8", "-s", "-o=+-mod3"]), "environments-simple-nested-mod3.tex");
    // 单行规格
    check_owned(DIR, f, &a(&["--lines", "8", "-s", "-o=+-mod11"]), "environments-simple-nested-mod11.tex");
    check_owned(DIR, f, &a(&["--lines", "8,4", "-s", "-o=+-mod12"]), "environments-simple-nested-mod12.tex");
    // 非法区间被忽略
    check_owned(DIR, f, &a(&["--lines", "3x-7", "-s", "-o=+-mod8"]), "environments-simple-nested-mod8.tex");
    // maxline 超出文件行数
    check_owned(DIR, f, &a(&["--lines", "3-15", "-s", "-o=+-mod7"]), "environments-simple-nested-mod7.tex");
    // 单行文件
    check_owned(DIR, "single-line.tex", &a(&["--lines", "8,4", "-s", "-o=+-mod1"]), "single-line-mod1.tex");
    report();
}

#[test]
fn multiple_line_ranges() {
    let f = "environments-simple-nested.tex";
    check_owned(DIR, f, &a(&["--lines", "3-5,8-10", "-s", "-o=+-mod4"]), "environments-simple-nested-mod4.tex");
    // 多区间实际并成一个
    check_owned(DIR, f, &a(&["--lines", "3-5,6-10", "-s", "-o=+-mod5"]), "environments-simple-nested-mod5.tex");
    // 重叠区间
    check_owned(DIR, f, &a(&["--lines", "3-5,4-10", "-s", "-o=+-mod6"]), "environments-simple-nested-mod6.tex");
    check_owned(DIR, f, &a(&["--lines", "3-5,4-10", "-s", "-o=+-mod6A"]), "environments-simple-nested-mod6A.tex");
    check_owned(DIR, f, &a(&["--lines", "3-7,4-6", "-s", "-o=+-mod6B"]), "environments-simple-nested-mod6B.tex");
    check_owned(DIR, f, &a(&["--lines", "3-7,4-7", "-s", "-o=+-mod6C"]), "environments-simple-nested-mod6C.tex");
    check_owned(DIR, f, &a(&["--lines", "5-6,4-10", "-s", "-o=+-mod6D"]), "environments-simple-nested-mod6D.tex");
    check_owned(DIR, f, &a(&["--lines", "5-6,4-10,7-8", "-s", "-o=+-mod6E"]), "environments-simple-nested-mod6E.tex");
    check_owned(DIR, f, &a(&["--lines", "3-7,7-8", "-s", "-o=+-mod6F"]), "environments-simple-nested-mod6F.tex");
    // 乱序区间
    check_owned(DIR, f, &a(&["--lines", "8-10,3-5", "-s", "-o=+-mod9"]), "environments-simple-nested-mod9.tex");
    check_owned(DIR, f, &a(&["--lines", "8-10,3-5,1-2", "-s", "-o=+-mod10"]), "environments-simple-nested-mod10.tex");
    // 几乎全部行
    check_owned(DIR, f, &a(&["--lines", "2-12", "-s", "-o=+-mod16"]), "environments-simple-nested-mod16.tex");
    report();
}

#[test]
fn line_switch_m_and_r() {
    check_owned(
        DIR,
        "environments-simple-nested.tex",
        &a(&["--lines", "6-11", "-s", "-m", "-l=env-mod-lines1", "-o=+-mod13"]),
        "environments-simple-nested-mod13.tex",
    );
    check_owned(
        DIR,
        "environments-to-multiple.tex",
        &a(&["--lines", "6", "-s", "-m", "-l=env-mod-lines2", "-o=+-mod2"]),
        "environments-to-multiple-mod2.tex",
    );
    check_owned(
        DIR,
        "environments-simple-nested.tex",
        &a(&["--lines", "3", "-s", "-r", "-l=replace1", "-o=+-mod14"]),
        "environments-simple-nested-mod14.tex",
    );
    report();
}

#[test]
fn not_mode_and_blank_lines() {
    let f = "environments-simple-nested.tex";
    // not 模式
    check_owned(DIR, f, &a(&["--lines", "!8-10,!3-5", "-s", "-o=+-mod15"]), "environments-simple-nested-mod15.tex");
    // 空行
    check_owned(DIR, "blank-lines.tex", &a(&["--lines", "1", "-s", "-o=+-mod1"]), "blank-lines-mod1.tex");
    check_owned(DIR, "blank-lines.tex", &a(&["--lines", "4", "-s", "-o=+-mod4"]), "blank-lines-mod4.tex");
    check_owned(DIR, "blank-lines.tex", &a(&["--lines", "6", "-s", "-o=+-mod6"]), "blank-lines-mod6.tex");
    check_owned(DIR, "blank-lines.tex", &a(&["--lines", "8", "-s", "-o=+-mod8"]), "blank-lines-mod8.tex");
    report();
}
