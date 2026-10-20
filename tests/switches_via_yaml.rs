//! switchesViaYaml 开关激活测试，基准取自 tests/cases/switches-via-yaml。
//! 对应上游 switches-via-yaml.sh 的 10 个调用（switches6.yaml 上游未使用）。
mod common;
use common::Session;

const DIR: &str = "switches-via-yaml";

const G_SWITCH: &str = "INFO:  Switch activated via YAML:";

/// kSwitch/sSwitch（check + silent）。
#[test]
fn check_and_silent() {
    let s = Session::new(DIR, &[]);
    s.run(&["command1", "-l=switches1.yaml"]);
    s.assert_log_grep_multi("indent.log", &[G_SWITCH, "INFO:  simple diff:"], "switches1.txt");
    // kSwitch: 0 → 不激活
    s.run(&["command1", "-l=switches2.yaml"]);
    s.assert_log_grep_multi("indent.log", &[G_SWITCH], "switches2.txt");
    // kvSwitch → verbose check
    s.run(&["command1", "-l=switches3.yaml"]);
    s.assert_log_grep_multi("indent.log", &[G_SWITCH, "INFO:  simple diff:"], "switches3.txt");
}

/// mSwitch 输出与 kvSwitch 的 stdout diff。
#[test]
fn m_switch_and_kv_stdout() {
    let s = Session::new(DIR, &[]);
    s.run(&["command1", "-l=switches4.yaml", "-o=+-mod4"]);
    s.assert_file("command1-mod4.tex", "command1-mod4.tex");
    s.assert_log_grep_multi("indent.log", &[G_SWITCH], "switches4.txt");
    let out = s.run(&["command1", "-l=switches5.yaml"]);
    s.assert_log_grep_multi("indent.log", &[G_SWITCH], "switches5.txt");
    s.assert_stdout(&out, "command1-mod5-diff.txt");
}

/// rrSwitch / rvSwitch 的 Replacement mode 消息与输出。
#[test]
fn replacement_switches() {
    let s = Session::new(DIR, &[]);
    s.run(&["command1", "-l=switches7.yaml", "-o=+-mod7"]);
    s.assert_file("command1-mod7.tex", "command1-mod7.tex");
    s.assert_log_grep_multi(
        "indent.log",
        &[G_SWITCH, "INFO:  Replacement mode"],
        "switches7.txt",
    );
    s.run(&["command1", "-l=switches8.yaml", "-o=+-mod8"]);
    s.assert_file("command1-mod8.tex", "command1-mod8.tex");
    s.assert_log_grep_multi(
        "indent.log",
        &[G_SWITCH, "INFO:  Replacement mode"],
        "switches8.txt",
    );
}

/// slSwitch 屏幕日志（激活后上屏）与 t/ttSwitch 的 found 行。
#[test]
fn screenlog_and_trace() {
    let s = Session::new(DIR, &[]);
    let out = s.run(&["command1", "-l=switches9.yaml", "-o=+-mod9"]);
    s.assert_file("command1-mod9.tex", "command1-mod9.tex");
    s.assert_log_grep_multi("indent.log", &[G_SWITCH], "switches9.txt");
    s.assert_stdout(&out, "command1-mod9-sl.txt");
    s.run(&["command1", "-l=switches10.yaml", "-o=+-mod10"]);
    s.assert_file("command1-mod10.tex", "command1-mod10.tex");
    s.assert_log_grep_multi("indent.log", &[G_SWITCH, "found:"], "switches10.txt");
    s.run(&["command1", "-l=switches11.yaml", "-o=+-mod11"]);
    s.assert_file("command1-mod11.tex", "command1-mod11.tex");
    s.assert_log_grep_multi("indent.log", &[G_SWITCH, "found:"], "switches11.txt");
}
