//! 日志（对应 LatexIndent/Logger.pm + LogFile.pm 的输出格式）。

use std::sync::Mutex;

#[derive(Default)]
pub struct Logger {
    pub screenlog: bool,
    pub lines: Vec<String>,
}

static LOG: Mutex<Logger> = Mutex::new(Logger::new_const());

impl Logger {
    pub const fn new_const() -> Logger {
        Logger {
            screenlog: false,
            lines: Vec::new(),
        }
    }
}

/// 单线程 CLI 里这把锁不会被争用；即便某处 panic 毒化了锁，也不该让后续日志跟着崩。
fn with_log<T>(f: impl FnOnce(&mut Logger) -> T) -> T {
    let mut guard = LOG
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    f(&mut guard)
}

fn emit(msg: &str, star_prefix: &str, cont_indent: &str, always_print: bool) {
    let formatted = if let Some(rest) = msg.strip_prefix('*') {
        let head = format!("{star_prefix}{rest}");
        let mut s: String = prefix_lines_keep_tail(&head, "        ");
        // 首行去掉前导空白（与 Logger.pm 的 s/^\h+INFO/INFO/ 等价）
        if let Some(idx) = s.find(star_prefix.trim_end()) {
            s = s[idx..].to_string();
        }
        s
    } else {
        prefix_lines_keep_tail(msg, cont_indent)
    };
    let screenlog = with_log(|log| {
        log.lines.push(formatted.clone());
        log.screenlog
    });
    if screenlog || always_print {
        println!("{formatted}");
    }
}

/// Perl s/^/X/mg 等价：每行行首加前缀，但消息以换行结尾时末尾空段不加。
fn prefix_lines_keep_tail(msg: &str, prefix: &str) -> String {
    let mut parts: Vec<String> = msg.split('\n').map(|l| format!("{prefix}{l}")).collect();
    if msg.ends_with('\n') {
        if let Some(last) = parts.last_mut() {
            *last = String::new();
        }
    }
    parts.join("\n")
}

/// info：`*` 开头消息首行 "INFO:  "，续行 8 空格；否则每行 7 空格。
pub fn info(msg: &str) {
    emit(msg, "INFO:  ", "       ", false);
}

/// warn：`*` 开头消息首行 "WARN:  "；否则每行只缩进 7 空格（Perl Logger.pm：非 `*`
/// 的 warn 不加 WARN 前缀，只做续行缩进）。
pub fn warn(msg: &str) {
    if let Some(rest) = msg.strip_prefix('*') {
        let formatted = format!("WARN:  {rest}");
        let screenlog = with_log(|log| {
            log.lines.push(formatted.clone());
            log.screenlog
        });
        if screenlog {
            println!("{formatted}");
        }
    } else {
        emit(msg, "       ", "       ", false);
    }
}

/// fatal：`*` 消息首行 "FATAL: "（Perl s/^\h+FATAL/FATAL:/）、续行 7 空格；
/// 非 `*` 消息每行 6 空格。总是打印到屏幕。
pub fn fatal(msg: &str) {
    let formatted = if let Some(rest) = msg.strip_prefix('*') {
        let indented: String = format!("FATAL {rest}")
            .split('\n')
            .map(|l| format!("       {l}"))
            .collect::<Vec<_>>()
            .join("\n");
        indented.replacen("       FATAL ", "FATAL: ", 1)
    } else {
        msg.split('\n')
            .map(|l| format!("      {l}"))
            .collect::<Vec<_>>()
            .join("\n")
    };
    with_log(|log| log.lines.push(formatted.clone()));
    // 诊断信息走 stderr：stdout 可能正被重定向为缩进后的文档
    eprintln!("{formatted}");
}

/// trace：`*` 开头消息首行 "TRACE: "；否则每行 7 空格。
/// 仅 -t/-tt 时生效（Perl 各调用点均以 $is_t_switch_active 守卫）。
pub fn trace(msg: &str) {
    if !crate::switches::with(|s| s.is_t_switch_active() || s.is_tt_switch_active()) {
        return;
    }
    if let Some(rest) = msg.strip_prefix('*') {
        let formatted = format!("TRACE: {rest}");
        let screenlog = with_log(|log| {
            log.lines.push(formatted.clone());
            log.screenlog
        });
        if screenlog {
            println!("{formatted}");
        }
    } else {
        emit(msg, "TRACE: ", "       ", false);
    }
}

pub fn set_screenlog(on: bool) {
    with_log(|log| log.screenlog = on);
}

pub fn take_lines() -> Vec<String> {
    with_log(|log| std::mem::take(&mut log.lines))
}
