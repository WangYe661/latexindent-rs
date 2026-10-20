//! back-up-tests：备份编号/cycleThroughBackUps/-wd 不写备份/中文名文件，覆盖
//! tests/cases/back-up-tests/back-up-tests.sh 的 4 个调用。基准取自已提交的 latexindent.pl 结果。
//! 注意：上游测试框架循环执行脚本，issue-505 基准是第 2 遍稳态（新建.bak 已存在），
//! 此处同样跑两遍、断言第 2 遍。
mod common;
use common::Session;

const DIR: &str = "back-up-tests";

#[test]
fn backup_numbering_and_cycle() {
    let s = Session::new(DIR, &[]);
    // backupExtension .txt + maxNumberOfBackUps: 2：txt0 已存在 → 递增到 txt1 并覆盖
    let out = s.run(&["-s", "-w", "-l", "backup1.yaml", "-g", "one.log", "myfile"]);
    assert!(out.status.success());
    s.assert_log_section(
        "one.log",
        "Backup procedure",
        Some("INFO:"),
        "backup-info1.txt",
    );
    s.assert_file("myfile.tex", "myfile.tex");

    // backupExtension .txtn + cycleThroughBackUps: txtn1 轮转到 txtn0 后备份到 txtn1
    let out = s.run(&["-s", "-w", "-l", "backup2.yaml", "-g", "two.log", "myfile"]);
    assert!(out.status.success());
    s.assert_log_section(
        "two.log",
        "Backup procedure",
        Some("INFO:"),
        "backup-info2.txt",
    );
    s.assert_file("myfile.tex", "myfile.tex");
}

#[test]
fn overwrite_if_different_no_backup() {
    let s = Session::new(DIR, &[]);
    // -wd 且内容不变：不写、不备份（截取自 INFO:  -wd 到日志结尾）
    let out = s.run(&[
        "-s",
        "-wd",
        "-l",
        "backup2.yaml",
        "-g",
        "three.log",
        "myfile",
    ]);
    assert!(out.status.success());
    s.assert_log_section("three.log", "INFO:  -wd", None, "backup-info3.txt");
    s.assert_file("myfile.tex", "myfile.tex");
}

#[test]
fn only_one_backup_cjk_file() {
    let s = Session::new(DIR, &[]);
    // -y onlyOneBackUp:1 + 中文文件名；基准为稳态（第 2 遍才有 "will be overwritten"）
    for _ in 0..2 {
        let out = s.run(&[
            "-s",
            "-w",
            "新建.tex",
            "-g",
            "issue-505.log",
            "-y=onlyOneBackUp:1",
        ]);
        assert!(out.status.success());
    }
    s.assert_log_section("issue-505.log", "INFO:  Backup", None, "issue-505.txt");
    s.assert_file("新建.tex", "新建.tex");
}

/// 备份文件的内容与轮转结果（审计 `09-test-porting-audit.md` §3.4 指出的缺口：
/// 上游靠 `git diff` 覆盖「备份文件被正确写出/轮转」，原断言只查了日志与主文件不动点）。
///
/// 对应的上游调用：L5 `backup1.yaml`（`backupExtension: .txt`、`maxNumberOfBackUps: 2`）、
/// L10 `backup2.yaml`（`.txtn` + `cycleThroughBackUps: 1`）、
/// L19 `新建.tex` + `-y onlyOneBackUp:1`（单备份原地覆盖）。
///
/// ⚠️ `Session::new` 会把 case 目录里的**基准副本备份文件**一并复制进临时目录，
/// 不先清掉就会出现「产物断言假通过」（见 `doc/pending-work.md` §4 假通过陷阱）。
#[test]
fn backup_files_contents_and_rotation() {
    let s = Session::new(DIR, &[]);
    for name in [
        "myfile.txt0",
        "myfile.txt1",
        "myfile.txt2",
        "myfile.txtn0",
        "myfile.txtn1",
        "myfile.txtn2",
        "新建.bak",
    ] {
        let _ = std::fs::remove_file(s.work.join(name));
    }
    let myfile = std::fs::read(s.work.join("myfile.tex")).unwrap();
    let read = |name: &str| std::fs::read(s.work.join(name)).unwrap();
    let exists = |name: &str| s.work.join(name).exists();

    // L5：跑三遍 → myfile.txt0 / myfile.txt1；maxNumberOfBackUps=2 后不再新增
    for _ in 0..3 {
        let out = s.run(&["-s", "-w", "-l", "backup1.yaml", "-g", "one.log", "myfile"]);
        assert!(out.status.success());
    }
    assert!(exists("myfile.txt0"), "backup1：应写出 myfile.txt0");
    assert!(exists("myfile.txt1"), "backup1：第二遍应递增出 myfile.txt1");
    assert!(
        !exists("myfile.txt2"),
        "backup1：maxNumberOfBackUps=2 时不应出现 myfile.txt2"
    );
    assert_eq!(read("myfile.txt0"), myfile, "备份内容应与原文件一致（txt0）");
    assert_eq!(read("myfile.txt1"), myfile, "备份内容应与原文件一致（txt1）");

    // L10：cycleThroughBackUps → myfile.txtn0 / myfile.txtn1
    for _ in 0..3 {
        let out = s.run(&["-s", "-w", "-l", "backup2.yaml", "-g", "two.log", "myfile"]);
        assert!(out.status.success());
    }
    assert!(exists("myfile.txtn0"), "backup2：应写出 myfile.txtn0");
    assert!(exists("myfile.txtn1"), "backup2：应递增出 myfile.txtn1");
    assert!(
        !exists("myfile.txtn2"),
        "backup2：maxNumberOfBackUps=2 时不应出现 myfile.txtn2"
    );
    assert_eq!(read("myfile.txtn0"), myfile, "备份内容应与原文件一致（txtn0）");
    assert_eq!(read("myfile.txtn1"), myfile, "备份内容应与原文件一致（txtn1）");

    // L19：onlyOneBackUp:1 → 新建.bak 原地覆盖，内容等于原文件
    let cjk = std::fs::read(s.work.join("新建.tex")).unwrap();
    for _ in 0..2 {
        let out = s.run(&[
            "-s",
            "-w",
            "新建.tex",
            "-g",
            "issue-505.log",
            "-y=onlyOneBackUp:1",
        ]);
        assert!(out.status.success());
    }
    assert!(exists("新建.bak"), "onlyOneBackUp:1 应写出 新建.bak");
    assert_eq!(read("新建.bak"), cjk, "新建.bak 内容应与原文件一致");
}
