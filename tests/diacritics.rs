//! diacritics：带变音符的文件名/目录名/日志名（对应 diacritics-test-cases.sh 的 8 个调用）。
//! 基准取自 tests/cases/diacritics 已提交的 latexindent.pl 结果；
//! -w 用例按上游 git-status 语义验证不动点。
mod common;
use common::Session;

const DIR: &str = "diacritics";

#[test]
fn diacritics_files_and_logs() {
    let s = Session::new(DIR, &[]);

    // -w 不动点 + 日志落在工作目录
    let out = s.run(&["-s", "-w", "äö.tex", "-g", "diacritics0.log"]);
    assert!(out.status.success());
    s.assert_file("äö.tex", "äö.tex");
    assert!(s.work.join("diacritics0.log").is_file());

    // 子目录文件的 -w 不动点（备份/日志默认落在该子目录）
    let out = s.run(&["-s", "-w", "äö/äö.tex"]);
    assert!(out.status.success());
    s.assert_file("äö/äö.tex", "äö/äö.tex");
    assert!(s.work.join("äö").join("indent.log").is_file());

    // -l 变音符 YAML + stdout 模式（无产物）
    let out = s.run(&["-s", "äö.tex", "-l", "äö.yaml", "-g", "diacritics.log"]);
    assert!(out.status.success());
    assert!(s.work.join("diacritics.log").is_file());

    // -o=+-mod1 命名
    let out = s.run(&["-s", "äö.tex", "-l", "äö.yaml", "-g", "diacritics1.log", "-o=+-mod1"]);
    assert!(out.status.success());
    s.assert_file("äö-mod1.tex", "äö-mod1.tex");

    // -c 变音符目录：日志落 cruft 目录，产物仍写工作目录
    let out = s.run(&["-s", "äö.tex", "-l", "äö.yaml", "-g", "diacritics2.log", "-o=+-mod2", "-c", "./äö"]);
    assert!(out.status.success());
    s.assert_file("äö-mod2.tex", "äö-mod2.tex");
    assert!(s.work.join("äö").join("diacritics2.log").is_file());

    // -g 日志名带变音符
    let out = s.run(&["-s", "äö.tex", "-l", "äö.yaml", "-g", "äö-log-file.log", "-o=+-mod3", "-c", "./äö"]);
    assert!(out.status.success());
    s.assert_file("äö-mod3.tex", "äö-mod3.tex");
    assert!(s.work.join("äö").join("äö-log-file.log").is_file());

    // -o=+-äö1：输出名带变音符
    let out = s.run(&["-s", "äö.tex", "-l", "äö.yaml", "-g", "diacritics3.log", "-o=+-äö1"]);
    assert!(out.status.success());
    s.assert_file("äö-äö1.tex", "äö-äö1.tex");

    // 变音符目录内文件的 -w 不动点
    let out = s.run(&["-s", "-w", "ěščřž/test.tex"]);
    assert!(out.status.success());
    s.assert_file("ěščřž/test.tex", "ěščřž/test.tex");
}
