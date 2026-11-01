//! 正则引擎限额（pcre2 match limit）的回退链回归测试。
//!
//! `sampleBEFORE-smaller.tex` 里有一个**未闭合**的 `\begin{exercises}`：
//! 精确版会在这里指数回溯，撞上 pcre2 的 match limit（10M 步）。
//! 旧版本直接 panic；现在应当回退到「env body 原子化」变体并完整输出。
//!
//! 同一个文档的完整版 `sampleBEFORE.tex` 能被精确版跑通，因此用它当基准：
//! 小文件是它的前缀，两者的输出在重叠区必须逐行一致。

use std::path::PathBuf;
use std::process::{Command, Output};

fn bench_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("cases")
        .join("benchmarks")
}

fn cruft_dir() -> PathBuf {
    let dir = std::env::temp_dir().join(format!("latexindent-rs-limits-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn run(file: &str) -> Output {
    Command::new(env!("CARGO_BIN_EXE_latexindent-rs"))
        .args(["-l", "sample", "-c", &cruft_dir().to_string_lossy(), file])
        .current_dir(bench_dir())
        .env_remove("USERPROFILE")
        .env_remove("HOME")
        .env_remove("XDG_CONFIG_HOME")
        .env_remove("LOCALAPPDATA")
        .env_remove("LATEXINDENT_CONFIG")
        .stdin(std::process::Stdio::null())
        .output()
        .expect("无法运行 latexindent-rs")
}

#[test]
fn unclosed_environment_does_not_abort() {
    let out = run("sampleBEFORE-smaller.tex");
    assert_eq!(
        out.status.code(),
        Some(0),
        "含未闭合环境的文档不应 panic:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let text = String::from_utf8_lossy(&out.stdout).replace("\r\n", "\n");
    assert!(!text.is_empty(), "应产生输出");
    // 失败区域里的块仍应被缩进（`\begin{subproblem}` 在 `multicols` 里）
    assert!(
        text.contains("\t\\begin{subproblem}"),
        "回退路径应继续完成缩进"
    );
}

#[test]
fn fallback_matches_the_complete_document() {
    let small = run("sampleBEFORE-smaller.tex");
    assert_eq!(small.status.code(), Some(0));
    let big = run("sampleBEFORE.tex");
    assert_eq!(
        big.status.code(),
        Some(0),
        "完整版应与精确版一致地跑完:\n{}",
        String::from_utf8_lossy(&big.stderr)
    );

    let small: Vec<String> = String::from_utf8_lossy(&small.stdout)
        .replace("\r\n", "\n")
        .lines()
        .map(str::to_string)
        .collect();
    let big: Vec<String> = String::from_utf8_lossy(&big.stdout)
        .replace("\r\n", "\n")
        .lines()
        .map(str::to_string)
        .collect();

    assert!(!small.is_empty());
    let n = small.len().min(big.len());
    for i in 0..n {
        assert_eq!(
            small[i], big[i],
            "回退输出与完整文档在第 {} 行不一致",
            i + 1
        );
    }
}
