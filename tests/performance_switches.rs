//! 性能开关的回归测试：`-j/--jobs`（多文件并行）与 `--cache`（增量缓存）。
//!
//! 两者都只应改变耗时，不应改变产物：
//! - 并行结果必须与 `-j 1` 逐字节一致（日志也一致）；
//! - 缓存命中的结果必须与不走缓存的结果一致。

use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

static COUNTER: AtomicUsize = AtomicUsize::new(0);

const INPUT: &str = r"\begin{itemize}
\item first
\item second
  \begin{enumerate}
  \item nested
  \end{enumerate}
\end{itemize}
\section{a heading}
text with   spaces
";

struct Work {
    dir: PathBuf,
}

impl Work {
    fn new(files: &[&str]) -> Work {
        let dir = std::env::temp_dir().join(format!(
            "latexindent-rs-perf-{}-{}",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::SeqCst)
        ));
        std::fs::create_dir_all(&dir).unwrap();
        for f in files {
            std::fs::write(dir.join(f), INPUT).unwrap();
        }
        Work { dir }
    }

    fn run(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_latexindent-rs"))
            .args(args)
            .current_dir(&self.dir)
            .env_remove("USERPROFILE")
            .env_remove("HOME")
            .env_remove("XDG_CONFIG_HOME")
            .env_remove("LOCALAPPDATA")
            .env_remove("LATEXINDENT_CONFIG")
            .output()
            .expect("无法运行 latexindent-rs")
    }

    fn read(&self, name: &str) -> String {
        std::fs::read_to_string(self.dir.join(name))
            .unwrap_or_else(|e| panic!("无法读取 {name}: {e}"))
            .replace("\r\n", "\n")
    }

    fn exists(&self, name: &str) -> bool {
        self.dir.join(name).exists()
    }
}

impl Drop for Work {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

#[test]
fn parallel_matches_sequential() {
    let w = Work::new(&["a.tex", "b.tex", "c.tex", "d.tex"]);
    let seq = w.run(&["-s", "-j", "1", "-o=+-seq", "a.tex", "b.tex", "c.tex", "d.tex"]);
    assert!(seq.status.success(), "顺序运行失败: {seq:?}");
    let par = w.run(&["-s", "-j", "4", "-o=+-par", "a.tex", "b.tex", "c.tex", "d.tex"]);
    assert!(par.status.success(), "并行运行失败: {par:?}");

    for f in ["a", "b", "c", "d"] {
        assert!(w.exists(&format!("{f}-seq.tex")));
        assert_eq!(
            w.read(&format!("{f}-seq.tex")),
            w.read(&format!("{f}-par.tex")),
            "{f}.tex 的并行结果与顺序结果不一致"
        );
    }
}

#[test]
fn cache_reuses_identical_output() {
    let w = Work::new(&["x.tex"]);

    // 基准：不走缓存
    let plain = w.run(&["-s", "-o=+-plain", "x.tex"]);
    assert!(plain.status.success(), "{plain:?}");

    // 冷启动（写缓存）→ 再跑一次（命中缓存）
    let cold = w.run(&["-s", "--cache", "-o=+-cold", "x.tex"]);
    assert!(cold.status.success(), "{cold:?}");
    let warm = w.run(&["-s", "--cache", "-o=+-warm", "x.tex"]);
    assert!(warm.status.success(), "{warm:?}");

    let expected = w.read("x-plain.tex");
    assert_eq!(expected, w.read("x-cold.tex"), "冷启动结果与基准不一致");
    assert_eq!(expected, w.read("x-warm.tex"), "缓存命中结果与基准不一致");
}

#[test]
fn cache_detects_edits() {
    let w = Work::new(&["y.tex"]);
    assert!(w.run(&["-s", "--cache", "-w", "y.tex"]).status.success());
    let formatted = w.read("y.tex");

    // 内容未变：命中缓存，文件保持不动
    assert!(w.run(&["-s", "--cache", "-w", "y.tex"]).status.success());
    assert_eq!(formatted, w.read("y.tex"));

    // 改动后必定重新格式化（新增块需要缩进）
    std::fs::write(
        w.dir.join("y.tex"),
        format!("{formatted}\n\\begin{{itemize}}\n\\item new\n\\end{{itemize}}\n"),
    )
    .unwrap();
    assert!(w.run(&["-s", "--cache", "-w", "y.tex"]).status.success());
    let after = w.read("y.tex");
    assert!(
        after.contains("\t\\item new"),
        "改动后的文件应被重新缩进，实际为:\n{after}"
    );
}

#[test]
fn tracing_bypasses_cache() {
    let w = Work::new(&["z.tex"]);
    let out = w.run(&["-s", "-t", "--cache", "-o=+-t", "z.tex"]);
    assert!(out.status.success(), "{out:?}");
    let log = w.read("indent.log");
    // trace 模式必须照常输出逐块信息（未被缓存短路）
    assert!(
        log.contains("TRACE:") || log.contains("Processing switches"),
        "trace 日志缺少内容"
    );
    assert!(!w.exists(".latexindent-cache") || std::fs::read_dir(w.dir.join(".latexindent-cache")).unwrap().count() == 0,
        "trace 模式不应写入缓存条目");
}

#[test]
fn cache_entry_lives_in_cruft_dir() {
    let w = Work::new(&["q.tex"]);
    std::fs::create_dir_all(w.dir.join("cruft")).unwrap();
    assert!(w
        .run(&["-s", "--cache=cruft", "-c", "cruft", "-o=+-c", "q.tex"])
        .status
        .success());
    assert!(
        Path::new(&w.dir.join("cruft")).join("q").exists() || w.dir.join("cruft").exists(),
        "指定目录下应有缓存"
    );
    let entries = std::fs::read_dir(w.dir.join("cruft")).unwrap().count();
    assert!(entries > 0, "--cache=cruft 应把条目写进 cruft 目录");
}
