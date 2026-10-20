//! cruftdirectory：`-c` 备份/日志目录、`-g` 日志命名与备份编号，覆盖
//! tests/cases/cruftdirectory/cruft-directory-test-cases.sh 的 5 个调用。
//! 上游无基准产物（bash 靠 git status 验证不动点），此处验证：
//! -w 不动点、产物落盘位置（-c 目录 vs 工作目录）、备份编号递增。
mod common;
use common::Session;

const DIR: &str = "cruftdirectory";

fn fresh_cruft(tag: &str) -> String {
    let d = std::env::temp_dir().join(format!(
        "latexindent-rs-cruft-{}-{tag}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d.to_string_lossy().to_string()
}

/// 前 3 个调用：`-c` 指定独立目录（上游每条命令 mktemp -d），备份与日志都落那里，
/// 工作目录不留任何产物，cruft-directory1.tex 为不动点。
#[test]
fn cruft_dir_gets_backups_and_logs() {
    let s = Session::new(DIR, &[]);
    for (log, extra) in [
        ("one.log", vec![]),
        ("two.log", vec![]),
        ("three.log", vec!["-l=more-than-one-back-up.yaml"]),
    ] {
        let cruft = fresh_cruft(log.trim_end_matches(".log"));
        let c_arg = format!("-c={cruft}");
        let g_arg = format!("-g={log}");
        let mut args = vec!["-s", "cruft-directory1", &c_arg, "-w", &g_arg];
        args.extend_from_slice(&extra);
        let out = s.run(&args);
        assert!(out.status.success(), "{log} 退出码非 0");
        s.assert_file("cruft-directory1.tex", "cruft-directory1.tex");
        assert!(
            std::path::Path::new(&cruft).join(log).is_file(),
            "日志 {log} 应写入 -c 目录"
        );
        assert!(
            std::path::Path::new(&cruft)
                .join("cruft-directory1.bak0")
                .is_file(),
            "备份应写入 -c 目录"
        );
        assert!(!s.work.join(log).is_file(), "工作目录不应出现日志 {log}");
    }
}

/// 后 2 个调用：无 `-c` → 备份/日志落工作目录，备份编号递增（bak0 → bak1）；
/// cycleThroughBackUps/noCycle 的冲突设置（onlyOneBackUp 与 maxNumberOfBackUps）
/// 由 maxNumberOfBackUps 生效（Perl 警告后置 onlyOneBackUp=0）。
#[test]
fn default_cruft_increments_backups() {
    let s = Session::new(DIR, &[]);
    let out = s.run(&[
        "-s",
        "cruft-directory1",
        "-w",
        "-l=cycleThroughBackUps.yaml",
        "-g=four.log",
    ]);
    assert!(out.status.success(), "four.log 调用退出码非 0");
    s.assert_file("cruft-directory1.tex", "cruft-directory1.tex");
    assert!(s.work.join("cruft-directory1.bak0").is_file());
    assert!(s.work.join("four.log").is_file());

    let out = s.run(&[
        "-s",
        "cruft-directory1",
        "-w",
        "-l=cycleThroughBackUps.yaml,noCycle.yaml",
        "-g=five.log",
    ]);
    assert!(out.status.success(), "five.log 调用退出码非 0");
    s.assert_file("cruft-directory1.tex", "cruft-directory1.tex");
    assert!(s.work.join("cruft-directory1.bak1").is_file());
    assert!(s.work.join("five.log").is_file());
}

/// 上游 L7-13 的 `~/Desktop/tmp/` 分支（审计 §3.4：原移植固定走 `else` 的
/// 「每次 mktemp -d」路径，共用目录分支未覆盖）。
///
/// 该分支与 else 分支的差别是三条调用**共用同一个 cruft 目录**，
/// 因此备份编号在同一目录里累计（bak0 → bak1 → bak2），日志也全部落在那里。
#[test]
fn shared_desktop_tmp_cruft_dir() {
    let s = Session::new(DIR, &[]);
    // 模拟上游的 ~/Desktop/tmp（shell 会把 ~ 展开成绝对路径再传给程序）
    let home = std::env::temp_dir().join(format!(
        "latexindent-rs-home-{}-{}",
        std::process::id(),
        "desktop"
    ));
    let cruft = home.join("Desktop/tmp");
    let _ = std::fs::remove_dir_all(&home);
    std::fs::create_dir_all(&cruft).unwrap();
    let c_arg = format!("-c={}", cruft.to_string_lossy());

    let out = s.run(&["-s", "cruft-directory1", &c_arg, "-w", "-g=one.log"]);
    assert!(out.status.success(), "共享 cruft 第一条退出码非 0");
    assert!(cruft.join("one.log").is_file(), "one.log 应落在共享 cruft 目录");
    assert!(
        cruft.join("cruft-directory1.bak0").is_file(),
        "第一条应在共享 cruft 目录写出 bak0"
    );

    let out = s.run(&["-s", "cruft-directory1", &c_arg, "-w", "-g=two.log"]);
    assert!(out.status.success(), "共享 cruft 第二条退出码非 0");
    assert!(
        cruft.join("cruft-directory1.bak1").is_file(),
        "第二条应在同一目录内递增出 bak1"
    );

    let out = s.run(&[
        "-s",
        "cruft-directory1",
        &c_arg,
        "-w",
        "-l=more-than-one-back-up.yaml",
        "-g=three.log",
    ]);
    assert!(out.status.success(), "共享 cruft 第三条退出码非 0");
    assert!(cruft.join("three.log").is_file());
    assert!(
        cruft.join("cruft-directory1.bak2").is_file(),
        "第三条应继续递增出 bak2"
    );
    s.assert_file("cruft-directory1.tex", "cruft-directory1.tex");

    let _ = std::fs::remove_dir_all(&home);
}
