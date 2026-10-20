//! 主流程（对应 LatexIndent/Document.pm、FileExtension.pm、BackUpFileProcedure.pm、
//! Check.pm、LogFile.pm 的流程部分）。

use crate::logger;
use crate::settings::{as_bool, as_i64, as_str, get, get_path};
use crate::state;
use crate::switches::{self, Switches};
use crate::tokens::tok;
use std::cell::{Cell, RefCell};
use std::path::{Path, PathBuf};

thread_local! {
    static CURRENT_EXT: RefCell<String> = RefCell::new("tex".to_string());
    static CURRENT_FILE: RefCell<String> = const { RefCell::new(String::new()) };
}

pub fn current_extension() -> String {
    CURRENT_EXT.with(|e| e.borrow().clone())
}

pub fn current_file() -> String {
    CURRENT_FILE.with(|c| c.borrow().clone())
}

pub const SCRIPT_NAME: &str = "latexindent-rs";
pub const VERSION: &str = "4.0.2";
pub const VERSION_DATE: &str = "2026-06-06";

const HELP: &str = r#"usage: latexindent-rs [options] [file]
      -v, --version
          displays the version number and date of release
      -vv, --vversion
          displays verbose version details
      -h, --help
          help (see the documentation for detailed instructions and examples)
      -sl, --screenlog
          log file will also be output to the screen
      -o, --outputfile=<name-of-output-file>
          output to another file
      -w, --overwrite
          overwrite the current file; a backup will be made
      -wd, --overwriteIfDifferent
          overwrite the current file IF the indented text is different
      -s, --silent
          silent mode: no output will be given to the terminal
      -t, --trace
          tracing mode: verbose information given to the log file
      -l, --local[=myyaml.yaml]
          use localSettings.yaml (or the specified yaml files)
      -y, --yaml=<yaml settings>
          specify YAML settings
      -d, --onlydefault
          ONLY use defaultSettings.yaml, ignore ALL (yaml) user files
      -g, --logfile=<name of log file>
          used to specify the name of logfile (default is indent.log)
      -c, --cruft=<cruft directory>
          used to specify the location of backup files and indent.log
      -m, --modifylinebreaks
          modify linebreaks before, during, and at the end of code blocks
      -r, --replacement
          replacement mode (verbatim blocks not respected)
      -rv, --replacementrespectverb
          replacement mode, respecting verbatim code blocks
      -rr, --onlyreplacement
          *only* replacement mode, no indentation
      -k, --check
          will exit with 0 if document body unchanged, 1 if changed
      -kv, --check mode verbose
          as in check mode, but outputs diff to screen as well as logfile
      -n, --lines=<MIN-MAX>
          only operate on selected lines
      --GCString
          measure widths using unicode grapheme clusters"#;

/// 主入口（对应 Document::latexindent）。
pub fn run(args: &[String]) {
    let (mut sw, mut files) = switches::parse_args(args);
    // Perl latexindent.pl 尾部回退：-l 带值且无输入文件 → 该值即输入文件
    if files.is_empty() {
        if let Some(v) = sw.read_local_settings.clone() {
            if !v.is_empty() {
                files.push(v);
                sw.read_local_settings = Some(String::new());
            }
        }
    }

    // -l 误吞文件名的纠正（latexindent.pl 同款）
    if sw.read_local_settings.is_some() && files.is_empty() {
        files.push(sw.read_local_settings.clone().unwrap_or_default());
        sw.read_local_settings = Some(String::new());
    }
    if files.is_empty() {
        files.push("-".to_string());
    }

    if sw.version {
        println!("{VERSION}, {VERSION_DATE}");
        if sw.vversion {
            println!(
                "{SCRIPT_NAME} lives here: {}",
                std::env::current_exe().unwrap_or_default().display()
            );
        }
        return;
    }
    if sw.show_help {
        println!("{HELP}");
        return;
    }

    sw.store();
    logger::set_screenlog(sw.screenlog);
    switches::with_mut(|s| *s = sw.clone());

    let multiple_files = files.len() > 1;
    let mut check_status = 0usize;
    let mut file_status = 0usize;

    // ---- 一次性操作（LogFile::process_switches） ----
    let first = files[0].clone();
    process_switches(&mut sw, &files, &first);

    // ---- 读取 YAML 设置 ----
    let mut root = crate::settings::load_root_settings();
    // switchesViaYaml
    let yaml = sw.yaml.clone();
    let local = sw.read_local_settings.clone();
    let working_dir = Path::new(&first)
        .parent()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();
    // Perl dirname 返回 "."，路径展示形如 ./localSettings.yaml
    let working_dir = if working_dir.is_empty() {
        ".".to_string()
    } else {
        working_dir
    };
    let ms =
        crate::settings::read_settings(&mut root, &local, &yaml, sw.only_default, &working_dir);
    apply_switches_via_yaml(&mut sw, &ms.root);
    switches::with_mut(|s| *s = sw.clone());
    state::init_settings(ms);
    let root_clone = state::settings(|s| s.root.clone());
    state::init_patterns(crate::patterns::build(
        &root_clone,
        sw.is_m_switch_active(),
    ));

    // ---- 逐文件操作 ----
    let raw_output_to_file = sw.output_to_file.clone();
    let mut file_count = 0usize;
    for f in &files {
        file_count += 1;
        state::reset_storage();
        sw.output_to_file = raw_output_to_file.clone();
        switches::with_mut(|s| s.output_to_file = raw_output_to_file.clone());
        if multiple_files {
            logger::info(&format!("*Filename: {f} ({file_count} of {})", files.len()));
        }
        CURRENT_FILE.with(|c| *c.borrow_mut() = f.clone());
        let cruft = sw.cruft_directory.clone().unwrap_or_else(|| dirname(f));
        let status = file_extension_check(f, &sw, &cruft);
        if status > 0 {
            file_status = status;
            continue;
        }
        sw = switches::with(|s| s.clone());
        let mut body = take_body();
        crate::tokens::token_check(&body);
        // token_check 可能升级 token 名（追加 -LIN），patterns 里的 token 正则必须用最终 token 重建
        state::init_patterns(crate::patterns::build(
            &state::settings(|s| s.root.clone()),
            sw.is_m_switch_active(),
        ));
        // Perl `_construct_code_blocks_regex`（operate_on_file 开头）的构建期 trace
        if state::patterns(|p| p.special_present) {
            logger::trace("*specialBeginEnd: at least one specialBeginEnd code block present");
        } else {
            logger::trace("*specialBeginEnd empty, NO specialBeginEnd code blocks present");
        }

        // ---------- 主处理 ----------
        body = operate_on_file(body, &sw);

        // ---------- -wd：内容不变则不写不备份（BackUpFileProcedure::check_if_different） ----------
        let original = if sw.is_check_switch_active() || sw.overwrite_if_different {
            Some(take_original())
        } else {
            None
        };
        if sw.overwrite_if_different {
            check_if_different(original.as_deref().unwrap_or(""), &body, &mut sw);
        }

        // ---------- check 汇总 ----------
        if sw.is_check_switch_active() {
            let original = original.unwrap_or_default();
            if original != body {
                check_status = 1;
            }
            simple_diff(&original, &body, sw.is_check_verbose_switch_active());
        }
        output_indented_text(
            &body,
            &sw,
            &CURRENT_FILE.with(|c| c.borrow().clone()),
            &cruft,
        );
    }

    if sw.is_check_switch_active() && multiple_files {
        if check_status == 1 {
            logger::info(
                "*check switch across multiple files: differences to report from at least one file",
            );
        } else {
            logger::info("*check switch across multiple files: no differences to report");
        }
    }
    if file_status > 2 {
        logger::warn(
            "*at least one of the files you specified does not exist or could not be read",
        );
    }
    output_logfile(&cruft_of(&files[0], &sw), &sw);
    if file_status > 2 {
        std::process::exit(file_status as i32);
    }
    if check_status == 1 {
        std::process::exit(1);
    }
}

fn cruft_of(file: &str, sw: &Switches) -> String {
    sw.cruft_directory.clone().unwrap_or_else(|| dirname(file))
}

/// operate_on_file（对应 Document::operate_on_file）。
fn operate_on_file(mut body: String, sw: &Switches) -> String {
    create_back_up_file(sw, &body);
    let is_m = sw.is_m_switch_active();
    let mut file_starts_with_line_break = false;

    if sw.is_r_switch_active() && !sw.is_rv_switch_active() {
        body = crate::replacement::make_replacements(&body, "before");
    }

    if !sw.is_rr_switch_active() {
        if is_m {
            let (starts, b) = crate::mlb::file_starts_with_line_break(&body, "before");
            file_starts_with_line_break = starts;
            body = b;
        }
        // ---------- verbatim ----------
        body = crate::verbatim::find_noindent_block(&body);
        body = crate::verbatim::find_verbatim_commands(&body);
        body = crate::trailing_comments::remove_trailing_comments(&body);
        body = crate::verbatim::find_verbatim_environments(&body);
        body = crate::verbatim::find_verbatim_special(&body);
        if is_m {
            // verbatim 块的 -m 尾换行处理（beforeTextWrap）
            body = crate::verbatim::mlb_verbatim(&body, "beforeTextWrap");
            body = crate::blank_lines::protect_blank_lines(&body);
        }
        if sw.is_rv_switch_active() {
            body = crate::replacement::make_replacements(&body, "before");
        }
        body = crate::whitespace::remove_trailing_whitespace(&body, "before");
        body = crate::file_contents::find_file_contents_environments_and_preamble(&body);
        body = crate::double_backslash::dodge_double_backslash(&body);
        body = crate::whitespace::remove_leading_space(&body);
        if is_m {
            body = crate::mlb::pre_indent_sentence_and_text_wrap(&body);
        }

        // ---------- 标记注释对齐块：%*\begin{tabular}...%*\end{tabular} ----------
        if state::storage(|s| s.align_markup_block_present) {
            body = crate::alignment::align_mark_down_block(&body);
        }

        // ---------- 主代码块 ----------
        body = crate::blocks::find_all_code_blocks(&body, "");
        if state::settings(|s| as_bool(get(&s.root, "dos2unixlinebreaks"))) {
            body = body.replace("\r\n", "\n");
        }
        if is_m {
            body = crate::sentence::mlb_one_sentence_per_line_indent(&body);
            body = crate::mlb::after_indentation_token_adjust(&body);
        }
        body = crate::heading::find_heading(&body);
        if is_m {
            if state::settings(|s| {
                as_i64(get_path(
                    &s.root,
                    &["modifyLineBreaks", "condenseMultipleBlankLinesInto"],
                ))
                .unwrap_or(1)
                    != 0
            }) {
                body = crate::blank_lines::condense_blank_lines(&body);
            }
            body = crate::mlb::post_indent_sentence_and_text_wrap(&body);
        }

        if is_m
            && state::settings(|s| {
                as_bool(get_path(
                    &s.root,
                    &["modifyLineBreaks", "preserveBlankLines"],
                ))
            })
        {
            body = crate::blank_lines::unprotect_blank_lines(&body);
        }
        body = crate::double_backslash::un_dodge_double_backslash(&body);
        body = crate::whitespace::max_indentation_check(&body);
        body = crate::whitespace::remove_trailing_whitespace(&body, "after");
        if sw.is_rv_switch_active() {
            body = crate::replacement::make_replacements(&body, "after");
        }
        body = crate::verbatim::put_verbatim_back_in(&body, "everything-except-commands");
        body = crate::trailing_comments::put_trailing_comments_back_in(&body);
        body = crate::verbatim::put_verbatim_back_in(&body, "just-commands");
        if is_m {
            let (_, b) = crate::mlb::file_starts_with_line_break(&body, "after");
            if !file_starts_with_line_break {
                body = b;
            }
        }
        if sw.is_r_switch_active() && !sw.is_rv_switch_active() {
            body = crate::replacement::make_replacements(&body, "after");
        }
        if state::settings(|s| as_bool(get(&s.root, "dos2unixlinebreaks"))) {
            body = body.replace("\r\n", "\n");
        }
    }
    body
}

// ---------------------------------------------------------------------------
// 文件扩展名检查 / IO（对应 FileExtension.pm）
// ---------------------------------------------------------------------------

thread_local! {
    static BODY: RefCell<String> = const { RefCell::new(String::new()) };
    static ORIGINAL: RefCell<String> = const { RefCell::new(String::new()) };
    // 注：**不**模拟 PerlIO 的 :crlf 层（Windows 上 `:encoding(UTF-8)` 会把 `\r\n` 折成 `\n`）。
    // 参考实现的 checked-in 基准是在 Linux 上生成的（无 :crlf，`\r` 原样保留），
    // 例如 environments/issue-256-mod1.tex 保留行尾空格、空行保留 `\t`——
    // 这正是「`\r` 不属于 `\h`，`\h+$` 匹配不到」的结果。
    // 若在此归一为 `\n`，会反过来与 6166 个 LF/Linux 基准中的语义相悖。
}

fn take_body() -> String {
    BODY.with(|b| b.borrow().clone())
}

fn take_original() -> String {
    ORIGINAL.with(|b| b.borrow().clone())
}

fn dirname(f: &str) -> String {
    Path::new(f)
        .parent()
        .map(|p| {
            let s = p.to_string_lossy().to_string();
            if s.is_empty() {
                ".".to_string()
            } else {
                s
            }
        })
        .unwrap_or_else(|| ".".to_string())
}

/// fileparse($file, $suffix) 的等价物：返回 (name, 是否剥离了字面后缀)。
fn fileparse_suffix(file: &str, suffix: &str) -> (String, bool) {
    let base = Path::new(file)
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| file.to_string());
    if !suffix.is_empty() && base.len() > suffix.len() && base.ends_with(suffix) {
        (base[..base.len() - suffix.len()].to_string(), true)
    } else {
        (base, false)
    }
}

fn file_extension_check(file: &str, sw: &Switches, cruft: &str) -> usize {
    let mut file_name = file.to_string();
    let mut ext = String::new();
    // Perl fileparse($fileName, qr/\..[^.]*$/) 的 name（不含最后一个 .ext）
    let base_name = match Path::new(&file_name).file_name() {
        Some(n) => {
            let n = n.to_string_lossy().to_string();
            match n.rfind('.') {
                Some(i) if i > 0 && i + 1 < n.len() => n[..i].to_string(),
                _ => n,
            }
        }
        None => file_name.clone(),
    };
    if file_name != "-" {
        let p = PathBuf::from(&file_name);
        let cur_ext = p
            .extension()
            .map(|e| format!(".{}", e.to_string_lossy()))
            .unwrap_or_default();
        if cur_ext.is_empty() {
            // 按 fileExtensionPreference 顺序尝试补全
            let prefs: Vec<(String, i64)> = state::settings(|s| {
                get(&s.root, "fileExtensionPreference")
                    .and_then(|v| v.as_mapping())
                    .map(|m| {
                        let mut v: Vec<(String, i64)> = m
                            .iter()
                            .map(|(k, v)| {
                                (
                                    k.as_str().unwrap_or_default().to_string(),
                                    as_i64(Some(v)).unwrap_or(0),
                                )
                            })
                            .collect();
                        v.sort_by_key(|(_, n)| *n);
                        v
                    })
                    .unwrap_or_default()
            });
            logger::info("*File extension work:");
            logger::info(&format!(
                "latexindent called to act upon {file} without a file extension;\nsearching for files in the following order (see fileExtensionPreference):"
            ));
            logger::info(
                &prefs
                    .iter()
                    .map(|(e, _)| format!("\t{file}{e}"))
                    .collect::<Vec<_>>()
                    .join("\n"),
            );
            let mut found = false;
            for (e, _) in &prefs {
                let cand = format!("{file_name}{e}");
                if Path::new(&cand).is_file() {
                    logger::info(&format!("{file_name}{e} found!"));
                    file_name = cand;
                    logger::info(&format!("Updated fileName to {file_name}"));
                    ext = e.clone();
                    found = true;
                    break;
                }
            }
            if !found {
                logger::fatal(&format!(
                    "*I couldn't find a match for {file} in fileExtensionPreference (see defaultSettings.yaml)"
                ));
                for (e, _) in &prefs {
                    logger::fatal(&format!("I searched for {file}{e}"));
                }
                logger::fatal(
                    "but couldn't find any of them.\nConsider updating fileExtensionPreference.",
                );
                logger::fatal("*Exiting, no indentation done.");
                exit_with(3);
            }
        } else if !p.is_file() {
            logger::fatal(&format!(
                "*I couldn't find {file_name}, are you sure it exists?"
            ));
            logger::fatal("*Exiting, no indentation done.");
            exit_with(3);
        } else {
            ext = cur_ext;
        }
    }
    CURRENT_EXT.with(|e| {
        *e.borrow_mut() = ext.trim_start_matches('.').to_string();
    });
    // Perl file_extension_check 更新 ${$self}{fileName}，后续备份/覆盖写入用解析后的名字
    CURRENT_FILE.with(|c| *c.borrow_mut() = file_name.clone());

    // -o 命名（对应 FileExtension.pm 的 output file check）
    if let Some(raw_out) = sw.output_to_file.clone() {
        logger::info("*-o switch active: output file check");
        let mut out = raw_out.clone();
        if file_name == "-" && raw_out.starts_with('+') {
            logger::info("STDIN input mode active, -o switch is removing all + symbols");
            out = out.replace('+', "");
        }
        if let Some(rest) = raw_out.strip_prefix('+') {
            if rest != "+" {
                logger::info(&format!(
                    "-o switch called with + symbol at the beginning: {out}"
                ));
                out = format!("{base_name}{rest}");
                logger::info(&format!("output file is now: {out}"));
            }
        }
        let mut stripped = ext.trim_start_matches('.').to_string();
        if stripped.is_empty() {
            stripped = "tex".to_string();
        }
        let (mut name, out_ext) = fileparse_suffix(&out, &stripped);
        if !out_ext {
            logger::info(&format!(
                "-o switch called with file name without extension: {raw_out}"
            ));
            out = format!(
                "{name}{}{stripped}",
                if name.ends_with('.') { "" } else { "." }
            );
            logger::info(&format!(
                "Updated to {out} as the file extension of the input file is {stripped}"
            ));
        }
        if name.ends_with('.') {
            name.truncate(name.len() - 1);
        }
        if name.ends_with("++") {
            logger::info(&format!(
                "-o switch called with file name ending with ++: {out}"
            ));
            name.truncate(name.len() - 2);
            if name.is_empty() {
                name = base_name.clone();
            }
            let mut counter = 0usize;
            let mut fname = format!("{name}{counter}.{stripped}");
            logger::info(&format!(
                "will search for existence and increment counter, starting with {fname}"
            ));
            while Path::new(&fname).exists() {
                logger::info(&format!("{fname} exists, incrementing counter"));
                counter += 1;
                fname = format!("{name}{counter}.{stripped}");
            }
            logger::info(&format!(
                "{fname} does not exist, and will be the output file"
            ));
            out = fname;
        }
        switches::with_mut(|s| s.output_to_file = Some(out));
    }
    let _ = cruft;

    // 读取内容
    let content = if file_name == "-" {
        use std::io::Read;
        let mut buf = String::new();
        std::io::stdin().read_to_string(&mut buf).ok();
        buf
    } else {
        match std::fs::read_to_string(&file_name) {
            Ok(s) => s,
            Err(_) => {
                logger::fatal(&format!("*{file_name} could not be opened"));
                logger::fatal("Exiting, no indentation done.");
                exit_with(4);
            }
        }
    };

    let body = if let Some(lines_spec) = sw.lines.clone() {
        lines_body_selected_lines(&lines_spec, &content)
    } else {
        content
    };
    if sw.is_check_switch_active() || sw.overwrite_if_different {
        ORIGINAL.with(|o| *o.borrow_mut() = body.clone());
    }
    BODY.with(|b| *b.borrow_mut() = body);
    0
}

/// -n 行选择（对应 Lines.pm 的行范围解析 + verbatim 保护）。
fn lines_body_selected_lines(spec: &str, body: &str) -> String {
    // Perl: 去水平空白、连续 - 压成单个
    let cleaned: String = spec
        .chars()
        .filter(|c| *c != ' ' && *c != '\t')
        .collect::<String>();
    let cleaned = {
        let mut out = String::new();
        let mut prev_dash = false;
        for c in cleaned.chars() {
            if c == '-' {
                if !prev_dash {
                    out.push(c);
                }
                prev_dash = true;
            } else {
                out.push(c);
                prev_dash = false;
            }
        }
        out
    };
    let lines: Vec<String> = body.split_inclusive('\n').map(|s| s.to_string()).collect();
    let total = lines.len();
    let mut min_max: std::collections::BTreeMap<usize, usize> = Default::default();
    let mut neg: std::collections::BTreeMap<usize, usize> = Default::default();
    for part in cleaned.split(',') {
        if part.is_empty() {
            continue;
        }
        let (negation, rest) = match part.split_once('!') {
            Some((pre, post)) => (true, format!("{pre}{post}")),
            None => (false, part.to_string()),
        };
        let (min_s, max_s) = match rest.split_once('-') {
            Some((a, b)) => (a.to_string(), b.to_string()),
            None => (rest.clone(), rest.clone()),
        };
        // 两端都必须是纯数字（Perl: ^\d+$）
        let valid = |s: &str| !s.is_empty() && s.chars().all(|c| c.is_ascii_digit());
        if !valid(&min_s) || !valid(&max_s) {
            continue;
        }
        let mut min: usize = min_s.parse().unwrap();
        let mut max: usize = max_s.parse().unwrap();
        if min > max {
            std::mem::swap(&mut min, &mut max);
        }
        // min/max 超出文件行数时钳到行数
        if min == 0 || max == 0 {
            continue;
        }
        if min > total {
            min = total;
        }
        if max > total {
            max = total;
        }
        if negation {
            neg.insert(min, max);
        } else {
            min_max.insert(min, max);
        }
    }
    if min_max.is_empty() && neg.is_empty() {
        // Perl：无有效区间 → 整个 body 照常处理
        return body.to_string();
    }
    // 取反区间 → 补集并入 min_max
    if !neg.is_empty() {
        let not_ranges = lines_sort_and_combine(neg);
        for (i, &(nmin, nmax)) in not_ranges.iter().enumerate() {
            if i == 0 {
                if nmin > 1 {
                    min_max.insert(1, nmin - 1);
                }
            } else {
                min_max.insert(not_ranges[i - 1].1 + 1, nmin - 1);
            }
        }
        if let Some(&(_, last_max)) = not_ranges.last() {
            if last_max < total {
                min_max.insert(last_max + 1, total);
            }
        }
    }
    let ranges = lines_sort_and_combine(min_max);
    // 组装：选中行原样拼接，其余整体进 linesprotect 保护块
    let mut out = String::new();
    let mut start = 0usize;
    for &(min, max) in &ranges {
        let (min, max) = (min - 1, max - 1);
        if min != 0 {
            out.push_str(&crate::verbatim::create_line_block(&lines, start, min - 1));
            out.push('\n');
        }
        out.push_str(&lines[min..=max].concat());
        start = max + 1;
    }
    if start < total {
        out.push_str(&crate::verbatim::create_line_block(
            &lines,
            start,
            total - 1,
        ));
        out.push('\n');
    }
    out
}

/// lines_sort_and_combine_line_range：按 min 排序后合并重叠/相邻区间（Perl 含 splice 后 index 归零重扫）。
fn lines_sort_and_combine(map: std::collections::BTreeMap<usize, usize>) -> Vec<(usize, usize)> {
    let mut r: Vec<usize> = Vec::new();
    for (a, b) in map {
        r.push(a);
        r.push(b);
    }
    let mut index = 1usize;
    while index < r.len() / 2 {
        let cur_min = r[2 * index];
        let cur_max = r[2 * index + 1];
        let prev_max = r[2 * index - 1];
        if cur_min.saturating_sub(1) <= prev_max && cur_max > prev_max {
            r.drain(2 * index - 1..=2 * index);
            index = 0;
        } else if cur_min.saturating_sub(1) <= prev_max && cur_max <= prev_max {
            r.drain(2 * index..=2 * index + 1);
            index = 0;
        }
        index += 1;
    }
    r.chunks(2).map(|c| (c[0], c[1])).collect()
}

fn exit_with(code: i32) -> ! {
    output_logfile_current();
    std::process::exit(code);
}

fn output_logfile_current() {
    let sw = switches::with(|s| s.clone());
    let file = CURRENT_FILE.with(|c| c.borrow().clone());
    output_logfile(&cruft_of(&file, &sw), &sw);
}

// ---------------------------------------------------------------------------
// 备份（对应 BackUpFileProcedure.pm）
// ---------------------------------------------------------------------------

fn create_back_up_file(sw: &Switches, body: &str) {
    if !sw.overwrite {
        return;
    }
    let file = CURRENT_FILE.with(|c| c.borrow().clone());
    if file == "-" {
        return;
    }
    logger::info("*Backup procedure (-w flag active):");
    let cruft = cruft_of(&file, sw);
    let _ = std::fs::create_dir_all(&cruft);
    let base = Path::new(&file)
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();
    let backup_ext = state::settings(|s| {
        as_str(get(&s.root, "backupExtension"))
            .unwrap_or(".bak")
            .to_string()
    });
    let only_one = state::settings(|s| as_bool(get(&s.root, "onlyOneBackUp")));
    let max_number = state::settings(|s| as_i64(get(&s.root, "maxNumberOfBackUps")).unwrap_or(0));
    let cycle = state::settings(|s| as_bool(get(&s.root, "cycleThroughBackUps")));
    let _ = body;

    let mut only_one = only_one;
    let mut max_number = max_number;
    if only_one && max_number >= 1 {
        logger::warn(&format!(
            "*onlyOneBackUp={only_one} and maxNumberOfBackUps={max_number}"
        ));
        logger::warn(&format!(
            "setting onlyOneBackUp=0 which will allow you to reach {max_number} backups"
        ));
        only_one = false;
    }
    let backup_file_no_ext = format!("{cruft}/{base}");
    let mut backup_file = format!("{backup_file_no_ext}{backup_ext}");
    if !only_one {
        let mut counter = 0i64;
        loop {
            let cand = format!("{backup_file}{counter}");
            if !Path::new(&cand).exists() || (max_number > 0 && counter == max_number - 1) {
                backup_file = cand;
                break;
            }
            logger::info(&format!(
                "{cand} already exists, incrementing by 1 (see maxNumberOfBackUps)"
            ));
            counter += 1;
        }
    }
    if Path::new(&backup_file).exists() {
        if only_one {
            logger::info(&format!(
                "{backup_file} will be overwritten (see onlyOneBackUp)"
            ));
        } else {
            logger::info(&format!(
                "{backup_file} will be overwritten (maxNumberOfBackUps reached, see maxNumberOfBackUps)"
            ));
            if cycle {
                logger::info("cycleThroughBackUps detected (see cycleThroughBackUps)");
                for i in 1..max_number {
                    let from = format!("{backup_file_no_ext}{backup_ext}{i}");
                    let to = format!("{backup_file_no_ext}{backup_ext}{}", i - 1);
                    if Path::new(&from).exists() {
                        logger::info(&format!("Copying {from} to {to}..."));
                        if std::fs::copy(&from, &to).is_err() {
                            logger::fatal(&format!(
                                "*Could not write to backup file {to}. Please check permissions."
                            ));
                            logger::fatal("Exiting, no indentation done.");
                            exit_with(5);
                        }
                    }
                }
            }
        }
    }
    logger::info(&format!("Backing up {file} to {backup_file}..."));
    logger::info(&format!("{file} will be overwritten after indentation"));
    if std::fs::copy(&file, &backup_file).is_err() {
        logger::fatal(&format!(
            "*Could not write to backup file {backup_file}. Please check permissions."
        ));
        logger::fatal("Exiting, no indentation done.");
        exit_with(5);
    }
}

/// check_if_different（对应 BackUpFileProcedure.pm）：-wd 时内容不变则不写不备份。
fn check_if_different(original: &str, body: &str, sw: &mut Switches) {
    logger::info("*-wd switch active");
    if original == body {
        logger::info(
            "Original body matches indented body, NOT overwriting, no backup files created",
        );
        sw.overwrite = false;
        sw.overwrite_if_different = false;
        return;
    }
    logger::info("Original body is *different* from indented body");
    logger::info("activating overwrite switch, -w");
    sw.overwrite = true;
    create_back_up_file(sw, body);
}

// ---------------------------------------------------------------------------
// 输出（对应 Document::output_indented_text） + check diff（Check.pm）
// ---------------------------------------------------------------------------

fn output_indented_text(body: &str, sw: &Switches, file: &str, _cruft: &str) {
    logger::info("*Output routine:");
    if sw.overwrite && sw.output_to_file.is_none() {
        logger::info(&format!("Overwriting file {file}"));
        if std::fs::write(file, body).is_err() {
            logger::warn(&format!("Could not write to {file}"));
        }
    } else if let Some(out) = &sw.output_to_file {
        logger::info(&format!("Outputting to file {out}"));
        if std::fs::write(out, body).is_err() {
            logger::warn(&format!("Could not write to {out}"));
        }
    } else {
        logger::info("Not outputting to file; see -w and -o switches for more options.");
    }
    if !sw.silent_mode && !sw.is_check_switch_active() {
        print!("{body}");
    }
}

fn simple_diff(original: &str, new: &str, verbose: bool) {
    logger::info("*simple diff: (check switch active)");
    if original == new {
        logger::info("no differences, no diff to report");
        return;
    }
    // Perl split("\n", ...) 丢尾部空串
    let split_body = |s: &str| -> Vec<String> {
        let mut v: Vec<String> = s.split('\n').map(|x| x.to_string()).collect();
        while v.last().map(|l: &String| l.is_empty()).unwrap_or(false) {
            v.pop();
        }
        v
    };
    let old_body = split_body(original);
    let new_body = split_body(new);
    // 逐行差分条目（-m 时两侧行数可不同）
    struct Entry {
        old: Option<String>,
        new: Option<String>,
        line: usize,
    }
    let mut diff: Vec<Entry> = Vec::new();
    let max_lines = old_body.len().max(new_body.len());
    for i in 0..max_lines {
        let o = old_body.get(i);
        let n = new_body.get(i);
        match (o, n) {
            (Some(o), Some(n)) if o != n => diff.push(Entry {
                old: Some(o.clone()),
                new: Some(n.clone()),
                line: i + 1,
            }),
            (Some(_), Some(_)) => {}
            (Some(o), None) => diff.push(Entry {
                old: Some(o.clone()),
                new: None,
                line: i + 1,
            }),
            (None, Some(n)) => diff.push(Entry {
                old: None,
                new: Some(n.clone()),
                line: i + 1,
            }),
            (None, None) => {}
        }
    }
    if diff.is_empty() {
        logger::info("no differences, no diff to report");
        return;
    }
    // chunk 合并（照抄 Check.pm 的组合循环，含边界 quirk）
    let mut tmp_old = "-".to_string();
    let mut tmp_new = "+".to_string();
    let mut prev_line: i64 = -1;
    let mut chunks: Vec<(String, String, usize, usize)> = Vec::new();
    let mut first_line = diff[0].line;
    for i in 0..diff.len() {
        let cur = diff[i].line;
        if i == diff.len() - 1 || (prev_line >= 0 && (cur as i64) > prev_line + 1) {
            let mut last_line = if i == 0 { cur } else { diff[i - 1].line };
            if i == diff.len() - 1 {
                last_line = cur;
                if tmp_old != "-" {
                    tmp_old.push_str("\n-");
                }
                if let Some(o) = &diff[i].old {
                    tmp_old.push_str(o);
                }
                if tmp_new != "+" {
                    tmp_new.push_str("\n+");
                }
                if let Some(n) = &diff[i].new {
                    tmp_new.push_str(n);
                }
            }
            chunks.push((tmp_old.clone(), tmp_new.clone(), first_line, last_line));
            if let Some(o) = &diff[i].old {
                tmp_old = format!("-{o}");
            }
            if let Some(n) = &diff[i].new {
                tmp_new = format!("+{n}");
            }
            first_line = cur;
        } else {
            if let Some(o) = &diff[i].old {
                if tmp_old != "-" {
                    tmp_old.push_str("\n-");
                }
                tmp_old.push_str(o);
            }
            if let Some(n) = &diff[i].new {
                if tmp_new != "+" {
                    tmp_new.push_str("\n+");
                }
                tmp_new.push_str(n);
            }
        }
        prev_line = cur as i64;
    }
    for (old, new, first, last) in &chunks {
        logger::info(&format!("@@ {first} -- {last} @@"));
        logger::info(old);
        logger::info(new);
        if verbose {
            print!("\n@@ {first} -- {last} @@\n");
            print!("{old}\n");
            print!("{new}\n");
        }
    }
}

fn output_logfile(cruft: &str, sw: &Switches) {
    let _ = std::fs::create_dir_all(cruft);
    state::settings(|s| {
        let empty = serde_yaml::Mapping::new();
        let lp = get(&s.root, "logFilePreferences").and_then(|v| v.as_mapping());
        let lp = lp.unwrap_or(&empty);
        if let Some(end) = as_str(get(lp, "endLogFileWith")) {
            logger::info(end);
        }
        if as_bool(get(lp, "showGitHubInfoFooter")) {
            logger::info("*Please direct all communication/issues to:");
            logger::info("https://github.com/cmhughes/latexindent.pl");
            logger::info("documentation: https://latexindentpl.readthedocs.io/en/latest/");
        }
    });
    let log_name = sw
        .log_file_name
        .clone()
        .unwrap_or_else(|| "indent.log".to_string());
    let path = format!("{cruft}/{log_name}");
    let lines = logger::take_lines();
    if std::fs::write(&path, lines.join("\n") + "\n").is_err() && sw.screenlog {
        println!("WARN:  Could not open the logfile {path}");
    }
}

// ---------------------------------------------------------------------------
// process_switches（对应 LogFile.pm 的日志部分）
// ---------------------------------------------------------------------------

fn process_switches(sw: &mut Switches, files: &[String], first: &str) {
    let cruft = cruft_of(first, sw);
    logger::info(&format!(
        "*{SCRIPT_NAME} version {VERSION}, {VERSION_DATE}, a script to indent .tex files"
    ));
    logger::info(&format!(
        "{SCRIPT_NAME} lives here: {}",
        std::env::current_exe().unwrap_or_default().display()
    ));
    logger::info(&chrono_now());
    if first != "-" {
        if files.len() > 1 {
            logger::info("Filenames:");
            for f in files {
                logger::info(&format!("   {f}"));
            }
            logger::info(&format!("total number of files: {}", files.len()));
        } else {
            logger::info(&format!("Filename: {first}"));
        }
    } else {
        logger::info("Reading input from STDIN");
    }
    logger::info("*Processing switches:");
    if sw.screenlog {
        logger::info("-sl|--screenlog: log file will also be output to the screen");
    }
    if sw.is_t_switch_active() && !sw.is_tt_switch_active() {
        logger::info("-t|--trace: Trace mode active (you have used either -t or --trace)");
    }
    if sw.is_tt_switch_active() {
        logger::info("-tt|--ttrace: TTrace mode active (you have used either -tt or --ttrace)");
    }
    if sw.silent_mode {
        logger::info("-s|--silent: Silent mode active (you have used either -s or --silent)");
    }
    if sw.only_default {
        logger::info("-d|--onlydefault: Only defaultSettings.yaml will be used");
    }
    if sw.overwrite {
        logger::info(
            "-w|--overwrite: Overwrite mode active, will make a back up before overwriting",
        );
    }
    if sw.overwrite_if_different {
        logger::info(
            "-wd|--overwriteIfDifferent: will overwrite ONLY if indented text is different",
        );
    }
    if sw.read_local_settings.is_some() {
        logger::info("-l|--localSettings: Read localSettings YAML file");
    }
    if sw.yaml.is_some() {
        logger::info("-y|--yaml: YAML settings specified via command line");
    }
    if sw.output_to_file.is_some() {
        logger::info("-o|--outputfile: output to file");
    }
    if sw.modify_line_breaks {
        logger::info("-m|--modifylinebreaks: modify line breaks");
    }
    if sw.log_file_name.is_some() {
        logger::info("-g|--logfile: logfile name");
    }
    if sw.cruft_directory.is_some() {
        logger::info("-c|--cruft: cruft directory");
    }
    if sw.replacement {
        logger::info("-r|--replacement: replacement mode");
    }
    if sw.only_replacement {
        logger::info("-rr|--onlyreplacement: *only* replacement mode, no indentation");
    }
    if sw.replacement_respect_verb {
        logger::info("-rv|--replacementrespectverb replacement mode, respecting verbatim");
    }
    if sw.check {
        logger::info("-k|--check mode: will exit with 0 if document body unchanged, 1 if changed");
    }
    if sw.check_verbose {
        logger::info("-kv|--check mode verbose: as in check mode, but outputs diff to screen");
    }
    if sw.lines.is_some() {
        logger::info(&format!(
            "-n|--lines mode: will only operate on specific lines {}",
            sw.lines.clone().unwrap_or_default()
        ));
    }
    if sw.gcstring {
        logger::info("--GCString switch active");
    }

    if sw.overwrite && sw.output_to_file.is_some() {
        logger::info("*Options check: -w and -o specified");
        logger::info("The -o switch will take priority, and -w (overwrite) will be ignored");
        sw.overwrite = false;
        switches::with_mut(|s| s.overwrite = false);
    }
    if sw.overwrite && sw.overwrite_if_different {
        logger::info("*Options check: -w and -wd specified");
        logger::info("The -wd switch will take priority, and -w (overwrite) will be ignored");
        sw.overwrite = false;
        switches::with_mut(|s| s.overwrite = false);
    }
    if sw.overwrite_if_different && sw.output_to_file.is_some() {
        logger::info("*Options check: -wd and -o specified");
        logger::info("The -o switch will take priority, and -wd ignored");
        sw.overwrite_if_different = false;
        switches::with_mut(|s| s.overwrite_if_different = false);
    }

    // 多文件 + 固定名 -o：忽略 -o（Perl LogFile.pm L252-262）。
    // `-o` 值不以 `+` 开头时，多文件下只会把最后一个文件的输出写进同一文件，
    // 故警告并关闭 -o。注意此判定在「-w 与 -o 同时给出」之后，-w 已先被关闭。
    if files.len() > 1 {
        if let Some(o) = sw.output_to_file.clone() {
            if !o.trim_start_matches('h').starts_with('+') {
                logger::warn("*-o switch specified as single file, but multiple files given as input");
                logger::warn(&format!("ignoring your specification -o {o}"));
                logger::warn("perhaps you might specify it using, for example, -o=++ or -o=+myoutput");
                sw.output_to_file = None;
                switches::with_mut(|s| s.output_to_file = None);
            }
        }
    }
    logger::info("*Directory for backup files and log file indent.log:");
    logger::info(&cruft);
}

fn chrono_now() -> String {
    // 与 Perl 的 localtime() 输出格式近似
    use std::time::SystemTime;
    let _ = SystemTime::now();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    format!("UNIX time {now}")
}

fn apply_switches_via_yaml(sw: &mut Switches, root: &serde_yaml::Mapping) {
    let getb = |key: &str| -> bool {
        as_bool(get(
            get(root, "switchesViaYaml")
                .and_then(|v| v.as_mapping())
                .unwrap_or(&serde_yaml::Mapping::new()),
            key,
        ))
    };
    // 只开不关（消息顺序与 Perl GetYamlSettings 一致）
    if getb("kSwitch") {
        logger::info("*Switch activated via YAML: check mode (-k)");
        sw.check = true;
    }
    if getb("kvSwitch") {
        logger::info("*Switch activated via YAML: verbose check mode (-kv)");
        sw.check_verbose = true;
    }
    if getb("mSwitch") {
        logger::info("*Switch activated via YAML: modifyLineBreaks (-m)");
        sw.modify_line_breaks = true;
    }
    if getb("rSwitch") {
        logger::info("*Switch activated via YAML: replacement mode (-r)");
        sw.replacement = true;
    }
    if getb("rrSwitch") {
        logger::info("*Switch activated via YAML: replacement ONLY mode (-rr)");
        sw.only_replacement = true;
    }
    if getb("rvSwitch") {
        logger::info("*Switch activated via YAML: replacement respect verbatim mode (-rv)");
        sw.replacement_respect_verb = true;
    }
    if getb("sSwitch") {
        logger::info("*Switch activated via YAML: silent mode (-s)");
        sw.silent_mode = true;
    }
    if getb("slSwitch") {
        logger::info("*Switch activated via YAML: screen log (-sl)");
        // 本条消息本身不上屏（Perl 同）：记录后再开屏幕日志
        sw.screenlog = true;
        logger::set_screenlog(true);
    }
    if getb("tSwitch") {
        logger::info("*Switch activated via YAML: trace mode (-t)");
        sw.trace = true;
    }
    if getb("ttSwitch") {
        logger::info("*Switch activated via YAML: ttrace mode (-tt)");
        sw.ttrace = true;
    }
    sw.store();
}
