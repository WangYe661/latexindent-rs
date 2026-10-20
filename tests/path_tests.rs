//! path-tests：`paths:` 嵌套路径展开与日志摘录，覆盖 tests/cases/path-tests/path-tests.sh 的 7 个调用。
//! 基准取自 tests/cases/path-tests 已提交的 latexindent.pl 结果。
//! 注意：缺失路径的 WARN 头固定为 `$homeDir/indentconfig.yaml`（Perl 怪癖），含机器相关
//! home 路径，比对前规范化为 `<HOME>`。
mod common;
use common::{cases_root, Session};

const DIR: &str = "path-tests";

/// 截取日志段（对应 perl -p0i：从 "YAML settings, reading" 到下一个 "INFO:" 前）并比对。
fn check_section(s: &Session, expected: &str) {
    let text = std::fs::read_to_string(s.work.join("indent.log")).expect("读取 indent.log");
    let mut section = match text.find("YAML settings, reading") {
        Some(i) => text[i..].to_string(),
        None => panic!("日志中未找到 YAML settings, reading"),
    };
    if let Some(i) = section.find("INFO:") {
        section.truncate(i);
    }
    let want = std::fs::read_to_string(cases_root().join(DIR).join(expected)).expect("读取基准");
    assert_eq!(
        normalize(&section),
        normalize(&want),
        "日志摘录与基准 {expected} 不一致"
    );
}

fn normalize(t: &str) -> String {
    t.lines()
        .map(|l| {
            if l.starts_with("WARN:  ") && l.contains("indentconfig.yaml") {
                "WARN:  <HOME>/indentconfig.yaml"
            } else {
                l
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn nested_paths_log_sections() {
    let s = Session::new(DIR, &[]);
    for n in 1..=7 {
        let l = format!("-l=nested-paths{n}.yaml");
        let out = if n == 1 {
            s.run(&["-s", &l, "mwe.tex"])
        } else {
            let o = format!("-o=+-mod{n}");
            s.run(&["-s", &l, "mwe.tex", &o])
        };
        assert!(out.status.success(), "nested-paths{n} 退出码非 0");
        if n > 1 {
            s.assert_file(&format!("mwe-mod{n}.tex"), &format!("mwe-mod{n}.tex"));
        }
        check_section(&s, &format!("path-test{n}.txt"));
    }
}
