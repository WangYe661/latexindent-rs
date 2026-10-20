//! 多文件批处理测试，基准取自 tests/cases/batch-tests。
//! 对应上游 batch-tests.sh 的 16 个调用（多文件 glob 由测试展开为排序列表）。
mod common;
use common::Session;

const DIR: &str = "batch-tests";

/// 展开 `prefix*suffix` 形式 glob（bash C-locale 字节序）。
fn glob(s: &Session, prefix: &str, suffix: &str) -> Vec<String> {
    let mut v: Vec<String> = std::fs::read_dir(&s.work)
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().to_string())
        .filter(|n| n.starts_with(prefix) && n.ends_with(suffix))
        .collect();
    v.sort();
    v
}

fn sargs(v: &[String]) -> Vec<&str> {
    v.iter().map(|s| s.as_str()).collect()
}

/// 多文件 -w（不动点）。
#[test]
fn overwrite_batch() {
    let s = Session::new(DIR, &["../environments/environments-simple.tex"]);
    // 1: -s -w env-batch*.tex -g env-batch.log -y onlyOneBackUp:1
    let mut args = vec![
        "-s".to_string(),
        "-w".to_string(),
        "-g".to_string(),
        "env-batch.log".to_string(),
        "-y".to_string(),
        "onlyOneBackUp:1".to_string(),
    ];
    let files = glob(&s, "env-batch", ".tex");
    args.extend(files.iter().cloned());
    s.run(&sargs(&args));
    for f in &files {
        s.assert_file(f, f);
    }

    // 2: 无输出，仅运行（-l cmh.yaml -t）
    let mut args = vec![
        "-s".to_string(),
        "-g".to_string(),
        "yaml-batch.log".to_string(),
        "-y".to_string(),
        "onlyOneBackUp:1".to_string(),
        "-l".to_string(),
        "cmh.yaml".to_string(),
        "-t".to_string(),
    ];
    args.extend(files.iter().cloned());
    let out = s.run(&sargs(&args));
    assert!(out.status.success(), "case2 应正常退出");

    // 3: 跨目录 -w
    let mut args = vec![
        "-s".to_string(),
        "-w".to_string(),
        "-g".to_string(),
        "diff-directories.log".to_string(),
        "-y".to_string(),
        "onlyOneBackUp:1".to_string(),
        "../environments/environments-simple.tex".to_string(),
    ];
    args.extend(files.iter().cloned());
    s.run(&sargs(&args));
    s.assert_file(
        "../environments/environments-simple.tex",
        "../environments/environments-simple.tex",
    );
    for f in &files {
        s.assert_file(f, f);
    }
}

/// 多文件 + -o 交互（-o 优先级 / 每文件 `+-` 命名）。
#[test]
fn o_switch_batch() {
    let s = Session::new(DIR, &[]);
    // 4：`-s -w env-batch*.tex -g o-switch-fail.log -o=myfile.tex` 见
    //    `o_switch_multi_file_fixed_name`（多文件 + 固定名 -o：忽略 -o，不写输出）。
    // 5: -o=+-myfile.tex → 每文件基名拼接
    let out = s.run(&[
        "-s",
        "env-batch1",
        "env-batch2",
        "env-batch3",
        "-g=o-switch-pass.log",
        "-o=+-myfile.tex",
    ]);
    assert!(out.status.success(), "case5 应正常退出");
    s.assert_file("env-batch1-myfile.tex", "env-batch1-myfile.tex");
    s.assert_file("env-batch2-myfile.tex", "env-batch2-myfile.tex");
    s.assert_file("env-batch3-myfile.tex", "env-batch3-myfile.tex");
}

/// 上游 L14（case 4）：`latexindent.pl -s -w env-batch*.tex -g o-switch-fail.log -o=myfile.tex`
///
/// 上游语义（`LogFile.pm`「multiple files with the -o switch needs care」）：
/// 多文件 + 不含 `+` 的 `-o` → 警告并**关闭 -o**；而在此之前已因「-w 与 -o 同时给出」
/// 关闭了 `-w`，因此最终既不写输出、也不做备份，16 个输入文件保持原样。
#[test]
fn o_switch_multi_file_fixed_name() {
    let s = Session::new(DIR, &[]);
    let files = glob(&s, "env-batch", ".tex");
    assert_eq!(files.len(), 16, "env-batch*.tex 应为 16 个输入");
    let mut args = vec![
        "-s".to_string(),
        "-w".to_string(),
        "-g".to_string(),
        "o-switch-fail.log".to_string(),
        "-o=myfile.tex".to_string(),
    ];
    args.extend(files.iter().cloned());
    let out = s.run(&sargs(&args));
    assert!(out.status.success(), "case4 应正常退出");
    // 16 个输入文件运行后不得被改写
    for f in &files {
        s.assert_file(f, f);
    }
    // 固定名 -o 在多文件下被忽略 → 不应写出 myfile.tex
    assert!(
        !s.work.join("myfile.tex").exists(),
        "多文件 + 固定 -o 应被忽略，不写 myfile.tex"
    );
    // -w 已被关闭 → 不应产生备份
    let backups: Vec<String> = std::fs::read_dir(&s.work)
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().to_string())
        .filter(|n| n.contains(".bak"))
        .collect();
    assert!(backups.is_empty(), "不应产生备份文件，实际: {backups:?}");
}

/// --lines 多文件与 --check 退出码交互。
#[test]
fn lines_and_check() {
    let s = Session::new(DIR, &[]);
    let lines_files = glob(&s, "lines", ".tex");
    // 6: --lines 1---7（连续 - 折叠为单个）
    let mut args = vec![
        "-s".to_string(),
        "--lines".to_string(),
        "1---7".to_string(),
        "-g".to_string(),
        "lines-log.log".to_string(),
    ];
    args.extend(lines_files.iter().cloned());
    let out = s.run(&sargs(&args));
    assert!(out.status.success(), "case6 应正常退出");

    // 7: --check 无差异 → 退出码 0
    let mut args = vec!["--check".to_string(), "-s".to_string(), "-g".to_string(), "check-switch-pass.log".to_string()];
    args.extend(lines_files.iter().cloned());
    let out = s.run(&sargs(&args));
    assert_eq!(out.status.code(), Some(0), "check-switch-pass 应退出 0");

    // 8: --check + defaultIndent 变化 → 退出码 1
    let mut args = vec![
        "--check".to_string(),
        "-s".to_string(),
        "-g".to_string(),
        "check-switch-fail.log".to_string(),
        "-y".to_string(),
        "defaultIndent: ' '".to_string(),
    ];
    let batch = glob(&s, "env-batch", ".tex");
    args.extend(batch.iter().cloned());
    args.push("env-batch11.tex".to_string());
    let out = s.run(&sargs(&args));
    assert_eq!(out.status.code(), Some(1), "check-switch-fail 应退出 1");
}

/// 文件不存在的 FATAL（退出码 3）。
#[test]
fn fatal_files() {
    let s = Session::new(DIR, &[]);
    let lines_files = glob(&s, "lines", ".tex");
    let mut args = vec!["-s".to_string(), "cmh.tex".to_string(), "-g".to_string(), "fatal-file-not-exist.log".to_string()];
    args.extend(lines_files.iter().cloned());
    let out = s.run(&sargs(&args));
    assert_eq!(out.status.code(), Some(3), "cmh.tex 不存在应退出 3");

    let mut args = vec!["-s".to_string(), "cmh1.tex".to_string(), "-g".to_string(), "fatal-file-no-read.log".to_string()];
    args.extend(lines_files.iter().cloned());
    let out = s.run(&sargs(&args));
    assert_eq!(out.status.code(), Some(3), "cmh1.tex 不存在应退出 3");
}

/// -l documentation.yaml -rv -w -m doc*.tex。
#[test]
#[ignore = "known gap: -r substitution 带 /e（Perl eval 代码块）未实现，见 documentation.yaml 第 4 条替换"]
fn replacement_mode() {
    let s = Session::new(DIR, &[]);
    let docs = glob(&s, "doc", ".tex");
    let mut args = vec![
        "-l".to_string(),
        "documentation.yaml".to_string(),
        "-rv".to_string(),
        "-w".to_string(),
        "-m".to_string(),
        "-s".to_string(),
        "-g".to_string(),
        "not-working.log".to_string(),
    ];
    args.extend(docs.iter().cloned());
    s.run(&sargs(&args));
    for f in &docs {
        s.assert_file(f, f);
    }
}

/// 裸 `-l` 的默认本地设置名搜索（friend 1-4 / nofriend）。
#[test]
fn local_settings_friends() {
    for dir in [
        "local-settings-friend-1",
        "local-settings-friend-2",
        "local-settings-friend-3",
        "local-settings-friend-4",
        "local-settings-nofriend",
    ] {
        let s = Session::new(DIR, &[]);
        let out = s.run(&["-s", "-l", &format!("{dir}/mwe.tex")]);
        assert!(out.status.success(), "{dir} 应正常退出");
        s.assert_log_grep(&format!("{dir}/indent.log"), "WARN:  yaml", &format!("{dir}/warnings.txt"));
    }
}
