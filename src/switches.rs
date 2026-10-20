//! 命令行开关（对应 latexindent.pl 的 GetOptions + LatexIndent/Switches.pm）。

use std::cell::RefCell;

#[derive(Default, Clone)]
pub struct Switches {
    // 原始开关
    pub version: bool,
    pub vversion: bool,
    pub silent_mode: bool,
    pub trace: bool,
    pub ttrace: bool,
    pub read_local_settings: Option<String>,
    pub yaml: Option<String>,
    pub only_default: bool,
    pub overwrite: bool,
    pub overwrite_if_different: bool,
    pub output_to_file: Option<String>,
    pub modify_line_breaks: bool,
    pub log_file_name: Option<String>,
    pub show_help: bool,
    pub cruft_directory: Option<String>,
    pub screenlog: bool,
    pub replacement: bool,
    pub only_replacement: bool,
    pub replacement_respect_verb: bool,
    pub check: bool,
    pub check_verbose: bool,
    pub lines: Option<String>,
    pub gcstring: bool,
}

impl Switches {
    /// 对应 Switches.pm 的 store_switches：派生各 is_*_switch_active。
    pub fn store(&mut self) {
        if self.vversion {
            self.version = true;
        }
        self.trace = self.ttrace || self.trace;
        self.check = self.check_verbose || self.check;
        self.replacement = self.only_replacement || self.replacement_respect_verb || self.replacement;
    }

    pub fn is_m_switch_active(&self) -> bool {
        self.modify_line_breaks
    }
    pub fn is_t_switch_active(&self) -> bool {
        self.trace
    }
    pub fn is_tt_switch_active(&self) -> bool {
        self.ttrace
    }
    pub fn is_r_switch_active(&self) -> bool {
        // Perl Switches.pm：-rr/-rv 隐含 -r
        self.replacement || self.only_replacement || self.replacement_respect_verb
    }
    pub fn is_rr_switch_active(&self) -> bool {
        self.only_replacement
    }
    pub fn is_rv_switch_active(&self) -> bool {
        self.replacement_respect_verb
    }
    pub fn is_check_switch_active(&self) -> bool {
        self.check
    }
    pub fn is_check_verbose_switch_active(&self) -> bool {
        self.check_verbose
    }
}

thread_local! {
    pub static SWITCHES: RefCell<Switches> = RefCell::new(Switches::default());
}

pub fn with<T>(f: impl FnOnce(&Switches) -> T) -> T {
    SWITCHES.with(|s| f(&s.borrow()))
}

pub fn with_mut<T>(f: impl FnOnce(&mut Switches) -> T) -> T {
    SWITCHES.with(|s| f(&mut s.borrow_mut()))
}

/// 解析命令行（Getopt::Long 兼容子集：`-x`、`--long`、`-x=v`、`-xv` 附加值）。
/// 返回 (switches, 剩余文件名)。
pub fn parse_args(args: &[String]) -> (Switches, Vec<String>) {
    let mut sw = Switches::default();
    let mut files: Vec<String> = Vec::new();
    let mut it = args.iter().peekable();
    while let Some(a) = it.next() {
        if a == "--" {
            files.extend(it.cloned());
            break;
        }
        if let Some(body) = a.strip_prefix("--") {
            let (name, val) = match body.split_once('=') {
                Some((n, v)) => (n.to_string(), Some(v.to_string())),
                None => (body.to_string(), None),
            };
            apply(&mut sw, &name, val, &mut it);
        } else if let Some(body) = a.strip_prefix('-') {
            if body.is_empty() {
                files.push(a.clone()); // "-" 表示 STDIN
                continue;
            }
            // Getopt::Long 兼容：单破折号长名（-yaml=、-outputfile、-local 等）
            const NAMES: &[&str] = &[
                "GCString", "wd", "kv", "rr", "rv", "sl", "tt", "vv",
                "version", "vversion", "silent", "trace", "ttrace", "local", "yaml",
                "onlydefault", "overwrite", "overwriteIfDifferent", "outputfile",
                "modifylinebreaks", "logfile", "help", "cruft", "screenlog",
                "replacement", "onlyreplacement", "replacementrespectverb",
                "check", "checkv", "lines",
                "v", "s", "t", "l", "y", "d", "w", "o", "m", "g", "h", "c", "r", "k", "n",
            ];
            let (namepart, inline) = match body.split_once('=') {
                Some((n, v)) => (n, Some(v.to_string())),
                None => (body, None),
            };
            if let Some(name) = NAMES.iter().find(|n| **n == namepart) {
                apply(&mut sw, name, inline, &mut it);
                continue;
            }
            // 唯一前缀自动缩写（Getopt::Long auto_abbrev）
            let m: Vec<&&str> = NAMES
                .iter()
                .filter(|n| n.len() > 1 && n.starts_with(namepart))
                .collect();
            if m.len() == 1 {
                apply(&mut sw, m[0], inline, &mut it);
                continue;
            }
            // -l=foo / -lfoo / -o=foo 等：单字符选项可带附加/独立值
            let mut chars = body.chars();
            let c = chars.next().unwrap();
            let rest: String = chars.collect();
            let name = c.to_string();
            let val = if !rest.is_empty() {
                Some(rest.strip_prefix('=').unwrap_or(&rest).to_string())
            } else {
                None
            };
            apply(&mut sw, &name, val, &mut it);
        } else {
            files.push(a.clone());
        }
    }
    (sw, files)
}

fn apply(
    sw: &mut Switches,
    name: &str,
    mut val: Option<String>,
    it: &mut std::iter::Peekable<std::slice::Iter<String>>,
) {
    use std::collections::HashMap;
    // 需要值的选项：无附加值时取下一个参数（与原版 -l myfile.tex 行为一致）
    let takes_value = matches!(
        name,
        "l" | "local" | "y" | "yaml" | "o" | "outputfile" | "g" | "logfile" | "c" | "cruft"
            | "n" | "lines"
    );
    if takes_value && val.is_none() {
        if let Some(next) = it.peek() {
            if !next.starts_with('-') {
                val = Some(next.to_string());
                it.next();
            } else {
                val = Some(String::new());
            }
        } else {
            val = Some(String::new());
        }
    }
    let flag = |b: &mut bool| *b = true;
    match name {
        "v" | "version" => flag(&mut sw.version),
        "vv" | "vversion" => flag(&mut sw.vversion),
        "s" | "silent" => flag(&mut sw.silent_mode),
        "t" | "trace" => flag(&mut sw.trace),
        "tt" | "ttrace" => flag(&mut sw.ttrace),
        "l" | "local" => sw.read_local_settings = Some(val.unwrap_or_default()),
        "y" | "yaml" => sw.yaml = val,
        "d" | "onlydefault" => flag(&mut sw.only_default),
        "w" | "overwrite" => flag(&mut sw.overwrite),
        "wd" | "overwriteIfDifferent" => flag(&mut sw.overwrite_if_different),
        "o" | "outputfile" => sw.output_to_file = val,
        "m" | "modifylinebreaks" => flag(&mut sw.modify_line_breaks),
        "g" | "logfile" => sw.log_file_name = val,
        "h" | "help" => flag(&mut sw.show_help),
        "c" | "cruft" => sw.cruft_directory = val,
        "sl" | "screenlog" => flag(&mut sw.screenlog),
        "r" | "replacement" => flag(&mut sw.replacement),
        "rr" | "onlyreplacement" => flag(&mut sw.only_replacement),
        "rv" | "replacementrespectverb" => flag(&mut sw.replacement_respect_verb),
        "k" | "check" => flag(&mut sw.check),
        "kv" | "checkv" => flag(&mut sw.check_verbose),
        "n" | "lines" => sw.lines = val,
        "GCString" => flag(&mut sw.gcstring),
        _ => {
            // 未知开关忽略（Getopt::Long 默认报错，这里宽容处理）
            let _ = HashMap::<(), ()>::new();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::parse_args;

    fn p(args: &[&str]) -> (super::Switches, Vec<String>) {
        parse_args(&args.iter().map(|s| s.to_string()).collect::<Vec<_>>())
    }

    #[test]
    fn single_dash_long_yaml() {
        let (sw, files) = p(&["-yaml=defaultIndent: ' '", "env.tex"]);
        assert_eq!(sw.yaml.as_deref(), Some("defaultIndent: ' '"));
        assert_eq!(files, vec!["env.tex"]);
    }

    #[test]
    fn output_file_plus_expansion_arg() {
        let (sw, _) = p(&["-o=+-mod1", "env.tex"]);
        assert_eq!(sw.output_to_file.as_deref(), Some("+-mod1"));
    }

    #[test]
    fn long_outputfile_takes_next_arg() {
        let (sw, files) = p(&["-outputfile", "+-default", "env.tex"]);
        assert_eq!(sw.output_to_file.as_deref(), Some("+-default"));
        assert_eq!(files, vec!["env.tex"]);
    }

    #[test]
    fn lines_takes_next_arg() {
        let (sw, _) = p(&["--lines", "3-5", "env.tex"]);
        assert_eq!(sw.lines.as_deref(), Some("3-5"));
    }

    #[test]
    fn short_flags_and_local() {
        let (sw, files) = p(&["-s", "-m", "-l=env-mod1", "env.tex"]);
        assert!(sw.silent_mode && sw.modify_line_breaks);
        assert_eq!(sw.read_local_settings.as_deref(), Some("env-mod1"));
        assert_eq!(files, vec!["env.tex"]);
    }

    #[test]
    fn long_aliases() {
        let (sw, _) = p(&["-wd", "-rr", "-vv"]);
        assert!(sw.overwrite_if_different && sw.only_replacement && sw.vversion);
    }
}
