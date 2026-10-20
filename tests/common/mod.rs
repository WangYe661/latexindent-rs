//! 测试夹具驱动工具（对应上游 latexindent.pl-main/test-cases 的 shell 测试脚本）。
//!
//! 每个用例把 tests/cases 下的输入文件与 `-l` 引用的 YAML 复制到独立临时目录，
//! 运行 latexindent 二进制，再把产物与上游基准文件逐字节比对。
//! 对 `-w`（原地覆盖）用例，基准文件本身就是期望输出（不动点检查）。

use std::path::{Component, Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

static COUNTER: AtomicUsize = AtomicUsize::new(0);

pub fn cases_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/cases")
}

/// 一次用例运行：`work` 是二进制工作目录（临时），`sub` 是 cases 下的子目录。
pub struct Case {
    pub work: PathBuf,
    sub: String,
    pub output: Output,
}

impl Drop for Case {
    fn drop(&mut self) {
        if let Some(root) = self.work.parent() {
            let _ = std::fs::remove_dir_all(root);
        }
    }
}

impl Case {
    pub fn assert_success(&self) {
        assert!(
            self.output.status.success(),
            "latexindent 退出码非 0（{:?}）\nstdout:\n{}\nstderr:\n{}",
            self.output.status.code(),
            String::from_utf8_lossy(&self.output.stdout),
            String::from_utf8_lossy(&self.output.stderr),
        );
    }

    #[allow(dead_code)]
    pub fn exit_code(&self) -> i32 {
        self.output.status.code().unwrap_or(-1)
    }

    /// 产物 `produced`（相对工作目录）应与基准 `expected`（相对 cases/<sub>）逐字节一致。
    pub fn assert_matches(&self, produced: &str, expected: &str) {
        if let Err(e) = self.diff_of(produced, expected) {
            panic!("{e}");
        }
    }

    /// 比对产物与基准；不一致返回 Err（供收集失败用）。
    pub fn diff_of(&self, produced: &str, expected: &str) -> Result<(), String> {
        let got = std::fs::read(self.work.join(produced))
            .map_err(|e| format!("无法读取产物 {produced}: {e}"))?;
        let want = std::fs::read(cases_root().join(&self.sub).join(expected))
            .map_err(|e| format!("无法读取基准 {}/{}: {e}", self.sub, expected))?;
        if String::from_utf8_lossy(&got) != String::from_utf8_lossy(&want) {
            return Err(format!(
                "输出与基准不一致 (产物 {produced} vs 基准 {}/{})",
                self.sub, expected
            ));
        }
        Ok(())
    }

    /// `-w` 用例：处理后文件应与基准（即输入文件本身）一致。
    pub fn assert_overwrite_matches(&self, file: &str) {
        self.assert_matches(file, file);
    }

    /// 日志 grep 比对（对应 `grep PATTERN log > x.txt`）。
    #[allow(dead_code)]
    #[track_caller]
    pub fn assert_log_grep(&self, log: &str, pattern: &str, expected: &str) {
        let text = std::fs::read_to_string(self.work.join(log))
            .unwrap_or_else(|e| panic!("无法读取日志 {log}: {e}"));
        let got: Vec<&str> = text.lines().filter(|l| l.contains(pattern)).collect();
        let want = std::fs::read_to_string(cases_root().join(&self.sub).join(expected))
            .unwrap_or_else(|e| panic!("无法读取基准 {}/{}: {e}", self.sub, expected));
        assert_eq!(
            got.join("\n"),
            want.replace("\r\n", "\n").trim_end(),
            "日志 {log} grep {pattern:?} 与基准 {expected} 不一致"
        );
    }

    /// 日志截取比对：从 start 首现处截到 end 首现处（对应 perl -p0i 截取）。
    #[allow(dead_code)]
    #[track_caller]
    pub fn assert_log_section(&self, log: &str, start: &str, end: Option<&str>, expected: &str) {
        let text = std::fs::read_to_string(self.work.join(log))
            .unwrap_or_else(|e| panic!("无法读取日志 {log}: {e}"));
        let mut section = match text.find(start) {
            Some(i) => text[i..].to_string(),
            None => panic!("日志 {log} 中未找到起始标记 {start:?}"),
        };
        if let Some(e) = end {
            if let Some(i) = section.find(e) {
                section.truncate(i);
            }
        }
        let want = std::fs::read_to_string(cases_root().join(&self.sub).join(expected))
            .unwrap_or_else(|e| panic!("无法读取基准 {}/{}: {e}", self.sub, expected));
        assert_eq!(
            section.replace("\r\n", "\n"),
            want.replace("\r\n", "\n"),
            "日志 {log} 摘录与基准 {expected} 不一致"
        );
    }
}

/// 解析 `-l` / `-local` 参数引用的 YAML 文件名（逗号分列，缺省补 .yaml）。
/// 返回 `(文件名, 是否显式给出)`；裸 `-l` 按 Perl 语义回落到默认名 `latexindent.yaml`，
/// `+` 按 Perl 语义展开为 `localSettings.yaml,latexindent.yaml,.localSettings.yaml,.latexindent.yaml`；
/// 这两种来源缺失时静默跳过（上游同目录未必总有它们）。
fn local_yaml_files(args: &[String]) -> Vec<(String, bool)> {
    const DEFAULTS: &str =
        "localSettings.yaml,latexindent.yaml,.localSettings.yaml,.latexindent.yaml";
    let mut out = Vec::new();
    let mut it = args.iter().peekable();
    while let Some(a) = it.next() {
        let (val, explicit): (Option<String>, bool) =
            if let Some(rest) = a.strip_prefix("-l=").or_else(|| a.strip_prefix("-local=")) {
                (Some(rest.to_string()), true)
            } else if a == "-l" || a == "-local" {
                // Perl Getopt l:s 可选值：下一个 token 以 - 开头则不取（裸 -l 用默认名）
                match it.peek() {
                    Some(next) if !next.starts_with('-') => (it.next().cloned(), true),
                    _ => (Some(DEFAULTS.to_string()), false),
                }
            } else if a.starts_with("-l")
                && !a.starts_with("-local")
                && !a.starts_with("-logfile")
                && !a.starts_with("-lines")
            {
                // -lfoo 形式（与单字符选项附加参数一致）
                (Some(a[2..].to_string()), true)
            } else {
                (None, false)
            };
        if let Some(mut v) = val {
            // `+` 展开（GetYamlSettings.pm：前导 `+` 在后追加默认名，尾随 `+` 则前置）
            let plus_implicit = v.contains('+');
            if plus_implicit {
                let at_start = v.trim_start().starts_with('+');
                let (comma_front, comma_back) = if at_start { ("", ",") } else { (",", "") };
                v = v.replacen('+', &format!("{comma_front}{DEFAULTS}{comma_back}"), 1);
            }
            if v.is_empty() {
                continue;
            }
            for name in v.split(',') {
                let mut f = name.trim().to_string();
                if f.is_empty() {
                    continue;
                }
                if !f.ends_with(".yaml") {
                    if !f.ends_with('.') {
                        f.push('.');
                    }
                    f.push_str("yaml");
                }
                // `+` 带出的默认名可以缺失；显式给出的名称仍要求存在
                out.push((f, explicit && !plus_implicit));
            }
        }
    }
    out
}

fn copy_into(src: &Path, dst: &Path) {
    if let Some(p) = dst.parent() {
        std::fs::create_dir_all(p).unwrap();
    }
    std::fs::copy(src, dst)
        .unwrap_or_else(|e| panic!("无法复制 {} -> {}: {e}", src.display(), dst.display()));
}

/// 把 `tests/cases` 内的相对路径规范化为临时目录下的物理路径（吸收 `..`）。
fn normalize(base: &Path, rel: &str) -> PathBuf {
    let mut p = base.to_path_buf();
    for c in Path::new(rel).components() {
        match c {
            Component::ParentDir => {
                p.pop();
            }
            Component::CurDir => {}
            other => p.push(other),
        }
    }
    p
}

/// 运行用例：
/// - `dir`：tests/cases 下的子目录（脚本工作目录）
/// - `input`：输入文件名（相对 `dir`）
/// - `args`：传给 latexindent 的参数（与 shell 脚本一致）
///
/// 会自动复制输入文件及所有 `-l` YAML（保持相对层级，支持 `../` 跨目录引用），
/// 并把 HOME/USERPROFILE 等指向空目录以隔离用户配置。
pub fn run(dir: &str, input: &str, args: &[String]) -> Case {
    run_impl(dir, input, args, true, false, false)
}

/// 与 `run` 相同，但容忍 `-l` 引用的 YAML 缺失（对应上游本就不存在的 `testlogfile.yaml` 一类引用；
/// Perl 版只打 warning 继续执行）。
#[allow(dead_code)]
pub fn run_allow_missing_yaml(dir: &str, input: &str, args: &[String]) -> Case {
    run_impl(dir, input, args, true, true, false)
}

/// STDIN 用例：input 文件内容经 stdin 传给二进制，不追加文件参数
/// （对应 `cat file | latexindent.pl ...`，是否带 `-` 由 args 决定）。
#[allow(dead_code)]
pub fn run_stdin(dir: &str, input: &str, args: &[String]) -> Case {
    run_impl(dir, input, args, false, false, false)
}

/// 输入文件本就不存在的用例（上游死行，如 documentation 的 `finetuning1`）：
/// 不复制输入、不追加文件参数，只按 args 运行。
#[allow(dead_code)]
pub fn run_missing_input(dir: &str, args: &[String]) -> Case {
    run_impl_no_input(dir, args)
}

fn run_impl_no_input(dir: &str, args: &[String]) -> Case {
    let id = COUNTER.fetch_add(1, Ordering::SeqCst);
    let tmp =
        std::env::temp_dir().join(format!("latexindent-rs-case-{}-{}", std::process::id(), id));
    let work = tmp.join(dir);
    std::fs::create_dir_all(&work).unwrap();

    let root = cases_root();
    for (y, explicit) in local_yaml_files(args) {
        let src = root.join(dir).join(&y);
        if !src.exists() && !explicit {
            continue;
        }
        copy_into(&src, &normalize(&tmp, &format!("{dir}/{y}")));
    }

    let output = Command::new(env!("CARGO_BIN_EXE_latexindent-rs"))
        .args(args)
        .current_dir(&work)
        .stdin(std::process::Stdio::null())
        .env_remove("USERPROFILE")
        .env_remove("HOME")
        .env_remove("XDG_CONFIG_HOME")
        .env_remove("LOCALAPPDATA")
        .env_remove("LATEXINDENT_CONFIG")
        .output()
        .expect("无法运行 latexindent 二进制");

    Case {
        work,
        sub: dir.to_string(),
        output,
    }
}

/// 整目录用例：把 `tests/cases/<dir>` **全部**内容复制到临时目录再运行。
///
/// 普通 `run` 只复制输入与 `-l` 直接引用的 YAML；当 YAML 里的 `paths:` 传递引用
/// 同目录其它 YAML（paths-demo 的 path1.yaml → path2.yaml → …）时复制不全，
/// 必须整目录搬运（对应上游在用例目录原地运行）。目录、输入与参数都由调用方
/// 以固定字面量给出。
#[allow(dead_code)]
pub fn run_full_dir(dir: &str, input: &str, args: &[String]) -> Case {
    run_impl(dir, input, args, true, false, true)
}

fn run_impl(
    dir: &str,
    input: &str,
    args: &[String],
    append_input: bool,
    allow_missing_yaml: bool,
    full_dir: bool,
) -> Case {
    let id = COUNTER.fetch_add(1, Ordering::SeqCst);
    let tmp =
        std::env::temp_dir().join(format!("latexindent-rs-case-{}-{}", std::process::id(), id));
    let work = tmp.join(dir);
    std::fs::create_dir_all(&work).unwrap();

    let root = cases_root();
    if full_dir {
        copy_dir(&root.join(dir), &work);
    } else {
        copy_into(&root.join(dir).join(input), &work.join(input));
        for (y, explicit) in local_yaml_files(args) {
            // y 可能含 ../ 跨目录；相对 cases/<dir> 解析，复制到临时目录的对应物理位置
            let src = root.join(dir).join(&y);
            if !src.exists() && (allow_missing_yaml || !explicit) {
                continue;
            }
            copy_into(&src, &normalize(&tmp, &format!("{dir}/{y}")));
        }
    }

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_latexindent-rs"));
    cmd.args(args);
    if append_input {
        cmd.arg(input);
    }
    cmd.current_dir(&work)
        .env_remove("USERPROFILE")
        .env_remove("HOME")
        .env_remove("XDG_CONFIG_HOME")
        .env_remove("LOCALAPPDATA")
        .env_remove("LATEXINDENT_CONFIG");
    let output = if append_input {
        cmd.output().expect("无法运行 latexindent 二进制")
    } else {
        use std::io::Write;
        let content = std::fs::read(root.join(dir).join(input)).expect("读取 STDIN 输入");
        let mut child = cmd
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .expect("无法运行 latexindent 二进制");
        child
            .stdin
            .take()
            .unwrap()
            .write_all(&content)
            .expect("写入 stdin");
        child.wait_with_output().expect("等待 latexindent 二进制")
    };
    Case {
        work,
        sub: dir.to_string(),
        output,
    }
}

/// 需要运行产物与基准一致的用例。
pub fn check(dir: &str, input: &str, args: &[&str], expected: &str) {
    let case = run(
        dir,
        input,
        &args.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
    );
    case.assert_success();
    case.assert_matches(expected, expected);
}

/// 动态参数版（循环展开 `-o=+-mod$i` 等场景）。收集失败，末尾 `report()` 汇总。
#[allow(dead_code)]
pub fn check_owned(dir: &str, input: &str, args: &[String], expected: &str) {
    let case = run(dir, input, args);
    if !case.output.status.success() {
        fail(format!("{expected}: 退出码非 0"));
        return;
    }
    if let Err(e) = case.diff_of(expected, expected) {
        fail(format!("{expected}: {e}"));
    }
}

/// 同 `check_owned`，但容忍 `-l` 引用的 YAML 缺失（上游存在 `-l=multi-line1yaml` 这类笔误，
/// Perl 只打 warning 继续执行，文档演示用例需要同等宽容度）。
#[allow(dead_code)]
pub fn check_owned_lenient_yaml(dir: &str, input: &str, args: &[String], expected: &str) {
    let case = run_allow_missing_yaml(dir, input, args);
    if !case.output.status.success() {
        fail(format!("{expected}: 退出码非 0"));
        return;
    }
    if let Err(e) = case.diff_of(expected, expected) {
        fail(format!("{expected}: {e}"));
    }
}

/// `-w` 原地覆盖用例：结果必须与基准文件（不动点）一致。
pub fn check_overwrite(dir: &str, input: &str, args: &[&str]) {
    let mut all = args.iter().map(|s| s.to_string()).collect::<Vec<_>>();
    if !all.iter().any(|a| a == "-w" || a == "-wd") {
        all.push("-w".to_string());
    }
    let case = run(dir, input, &all);
    case.assert_success();
    case.assert_overwrite_matches(input);
}

/// -t 日志摘录用例：提取工作目录 indent.log 中含 pattern 的行，与基准比对
/// （对应上游 `egrep 'pattern' indent.log > xxx.txt` 一类用例）。
#[allow(dead_code)]
pub fn check_log_lines(dir: &str, input: &str, args: &[&str], pattern: &str, expected: &str) {
    let case = run(
        dir,
        input,
        &args.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
    );
    case.assert_success();
    let log = std::fs::read_to_string(case.work.join("indent.log")).expect("读取 indent.log");
    let got: Vec<&str> = log.lines().filter(|l| l.contains(pattern)).collect();
    let want = std::fs::read_to_string(cases_root().join(dir).join(expected)).expect("读取基准");
    assert_eq!(
        got.join("\n"),
        want.replace("\r\n", "\n").trim_end(),
        "日志摘录与基准不一致 ({expected})"
    );
}

thread_local! {
    static FAIL: std::cell::RefCell<Vec<String>> = const { std::cell::RefCell::new(Vec::new()) };
}

#[allow(dead_code)]
pub fn fail(msg: String) {
    FAIL.with(|f| f.borrow_mut().push(msg));
}

/// 汇总本测试内所有失败用例；无失败则通过。每个 #[test] 末尾调用。
#[allow(dead_code)]
#[track_caller]
pub fn report() {
    let v = FAIL.with(|f| f.borrow().clone());
    assert!(v.is_empty(), "\n{} 个用例失败:\n{}", v.len(), v.join("\n"));
}

// ---------------------------------------------------------------------------
// 共享目录的顺序执行会话（fileextensions 等顺序相关用例）
// ---------------------------------------------------------------------------

pub struct Session {
    pub work: PathBuf,
    tmp: PathBuf,
    dir: String,
}

impl Session {
    /// 把 tests/cases/<dir> 全部内容复制到临时目录；extra 为一并复制的跨目录文件
    /// （相对 cases/<dir> 的路径，如 "../alignment/x.yaml"）。
    #[allow(dead_code)]
    pub fn new(dir: &str, extra: &[&str]) -> Session {
        let tmp = std::env::temp_dir().join(format!(
            "latexindent-rs-sess-{}-{}",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::SeqCst)
        ));
        let work = tmp.join(dir);
        let root = cases_root().join(dir);
        copy_dir(&root, &work);
        for e in extra {
            let src = normalize(&root, e);
            let dst = normalize(&tmp, &format!("{dir}/{e}"));
            copy_into(&src, &dst);
        }
        Session {
            work,
            tmp,
            dir: dir.to_string(),
        }
    }

    /// 在共享目录中运行一次 latexindent。
    #[allow(dead_code)]
    pub fn run(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_latexindent-rs"))
            .args(args)
            .current_dir(&self.work)
            .env_remove("USERPROFILE")
            .env_remove("HOME")
            .env_remove("XDG_CONFIG_HOME")
            .env_remove("LOCALAPPDATA")
            .env_remove("LATEXINDENT_CONFIG")
            .output()
            .expect("无法运行 latexindent 二进制")
    }

    /// 工作目录中的产物应与基准（相对 cases/<dir>）一致。
    #[allow(dead_code)]
    #[track_caller]
    pub fn assert_file(&self, produced: &str, expected: &str) {
        let got = std::fs::read(self.work.join(produced))
            .unwrap_or_else(|e| panic!("无法读取产物 {produced}: {e}"));
        let want = std::fs::read(cases_root().join(&self.dir).join(expected))
            .unwrap_or_else(|e| panic!("无法读取基准 {}/{}: {e}", self.dir, expected));
        assert_eq!(
            String::from_utf8_lossy(&got).replace("\r\n", "\n"),
            String::from_utf8_lossy(&want).replace("\r\n", "\n"),
            "产物 {produced} 与基准 {expected} 不一致"
        );
    }

    /// 日志裁剪比对：从 start 首次出现处截到 end 首次出现处（对应上游 perl -p0i 截取）。
    #[allow(dead_code)]
    #[track_caller]
    pub fn assert_log_section(&self, log: &str, start: &str, end: Option<&str>, expected: &str) {
        let text = std::fs::read_to_string(self.work.join(log))
            .unwrap_or_else(|e| panic!("无法读取日志 {log}: {e}"));
        let mut section = match text.find(start) {
            Some(i) => text[i..].to_string(),
            None => panic!("日志 {log} 中未找到起始标记 {start:?}"),
        };
        if let Some(e) = end {
            if let Some(i) = section.find(e) {
                section.truncate(i);
            }
        }
        let want = std::fs::read_to_string(cases_root().join(&self.dir).join(expected))
            .unwrap_or_else(|e| panic!("无法读取基准 {}/{}: {e}", self.dir, expected));
        assert_eq!(
            section.replace("\r\n", "\n"),
            want.replace("\r\n", "\n"),
            "日志 {log} 摘录与基准 {expected} 不一致"
        );
    }

    /// 日志 grep 比对：含 pattern 的行集与基准比对（对应 `grep PATTERN log > x.txt`）。
    #[allow(dead_code)]
    #[track_caller]
    pub fn assert_log_grep(&self, log: &str, pattern: &str, expected: &str) {
        self.assert_log_grep_multi(log, &[pattern], expected);
    }

    /// 多模式顺序 grep（对应 `grep A log > x; grep B log >> x`）。
    #[allow(dead_code)]
    #[track_caller]
    pub fn assert_log_grep_multi(&self, log: &str, patterns: &[&str], expected: &str) {
        let text = std::fs::read_to_string(self.work.join(log))
            .unwrap_or_else(|e| panic!("无法读取日志 {log}: {e}"));
        let mut got: Vec<&str> = Vec::new();
        for p in patterns {
            got.extend(text.lines().filter(|l| l.contains(p)));
        }
        let want = std::fs::read_to_string(cases_root().join(&self.dir).join(expected))
            .unwrap_or_else(|e| panic!("无法读取基准 {}/{}: {e}", self.dir, expected));
        assert_eq!(
            got.join("\n"),
            want.replace("\r\n", "\n").trim_end(),
            "日志 {log} grep {patterns:?} 与基准 {expected} 不一致"
        );
    }

    /// stdout 与基准逐字节比对。
    #[allow(dead_code)]
    #[track_caller]
    pub fn assert_stdout(&self, out: &Output, expected: &str) {
        let want = std::fs::read(cases_root().join(&self.dir).join(expected))
            .unwrap_or_else(|e| panic!("无法读取基准 {}/{}: {e}", self.dir, expected));
        assert_eq!(
            String::from_utf8_lossy(&out.stdout).replace("\r\n", "\n"),
            String::from_utf8_lossy(&want).replace("\r\n", "\n"),
            "stdout 与基准 {expected} 不一致"
        );
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.tmp);
        let _ = &self.dir;
    }
}

fn copy_dir(src: &Path, dst: &Path) {
    std::fs::create_dir_all(dst).unwrap();
    for entry in std::fs::read_dir(src).unwrap() {
        let entry = entry.unwrap();
        let to = dst.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_dir(&entry.path(), &to);
        } else {
            std::fs::copy(entry.path(), to).unwrap();
        }
    }
}
