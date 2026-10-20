//! 标题缩进（对应 LatexIndent/Heading.pm）：indentAfterHeadings 配置的
//! \part/\chapter/\section... 后续内容缩进。

use crate::logger;
use crate::patterns::{gn, replace_all_with};
use crate::settings::{as_bool, as_i64, as_str, get};
use crate::state;
use serde_yaml::Value;

fn regex_esc_name(s: &str) -> String {
    s.replace('*', r"\*")
}

// ---------------------------------------------------------------------------
// Data::Dumper 输出（logFilePreferences.Dumper 默认：Terse=1 Indent=1 Useqq=1
// Quotekeys=0 Sortkeys=1 Pair=" => "）
// ---------------------------------------------------------------------------

fn dumper(v: Option<&Value>) -> String {
    match v {
        Some(v) => format!("{}\n", dumper_node(v, 0)),
        None => "{}\n".to_string(),
    }
}

fn dumper_node(v: &Value, depth: usize) -> String {
    match v {
        Value::Mapping(m) => {
            if m.is_empty() {
                return "{}".to_string();
            }
            let mut s = String::from("{\n");
            let mut keys: Vec<&Value> = m.keys().collect();
            keys.sort_by_key(|k| key_str(k));
            let n = keys.len();
            for (i, k) in keys.iter().enumerate() {
                s.push_str(&"  ".repeat(depth + 1));
                s.push_str(&dumper_key(&key_str(k)));
                s.push_str(" => ");
                s.push_str(&dumper_node(m.get(*k).unwrap(), depth + 1));
                s.push_str(if i + 1 == n { "\n" } else { ",\n" });
            }
            s.push_str(&"  ".repeat(depth));
            s.push('}');
            s
        }
        _ => dumper_scalar(v),
    }
}

fn key_str(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => if *b { "1" } else { "0" }.to_string(),
        other => format!("{other:?}"),
    }
}

/// Quotekeys=0：形如标识符的键不加引号，其余按 Useqq=1 加双引号。
fn dumper_key(k: &str) -> String {
    let bare = !k.is_empty()
        && (k.chars().next().unwrap().is_ascii_alphabetic() || k.starts_with('_'))
        && k.chars().all(|c| c.is_ascii_alphanumeric() || c == '_');
    if bare {
        k.to_string()
    } else {
        format!("\"{}\"", useqq_escape(k))
    }
}

fn dumper_scalar(v: &Value) -> String {
    match v {
        Value::Null => "undef".to_string(),
        Value::Bool(b) => if *b { "1" } else { "0" }.to_string(),
        Value::Number(n) => n.to_string(),
        Value::String(s) => format!("\"{}\"", useqq_escape(s)),
        other => format!("{}", dumper_node(other, 0)),
    }
}

fn useqq_escape(s: &str) -> String {
    let mut out = String::new();
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            '\r' => out.push_str("\\r"),
            '$' => out.push_str("\\$"),
            '@' => out.push_str("\\@"),
            c => out.push(c),
        }
    }
    out
}

/// construct_headings_levels：收集启用的标题名并按 level 分组。
/// 返回 (allHeadingsRegexp, 每“实际级别”的正则数组, 名字→级别)。
fn construct_levels() -> (String, Vec<String>, Vec<(String, usize)>) {
    // 旧语法 indentAfterThisHeading → lookForThis（Heading.pm:45-57）
    let old_syntax = state::settings_mut(|s| {
        let mut old = false;
        if let Some(v) = s.root.get_mut(Value::String("indentAfterHeadings".into())) {
            if let Some(m) = v.as_mapping_mut() {
                let keys: Vec<Value> = m.keys().cloned().collect();
                for k in keys {
                    if let Some(info) = m.get_mut(&k).and_then(|x| x.as_mapping_mut()) {
                        if let Some(ia) = info.remove(&Value::String("indentAfterThisHeading".into())) {
                            info.insert(Value::String("lookForThis".into()), ia);
                            old = true;
                        }
                    }
                }
            }
        }
        old
    });
    if old_syntax {
        logger::warn(
            "*Obsolete indentAfterHeadings YAML syntax used, changed to the following:",
        );
        let dump = state::settings(|s| dumper(get(&s.root, "indentAfterHeadings")));
        logger::warn(&dump);
    }

    let entries: Vec<(String, usize)> = state::settings(|s| {
        let mut v = Vec::new();
        if let Some(m) = get(&s.root, "indentAfterHeadings").and_then(|x| x.as_mapping()) {
            for (k, info) in m {
                let name = k.as_str().unwrap_or_default().to_string();
                let info = match info.as_mapping() {
                    Some(i) => i,
                    None => continue,
                };
                let look = get(info, "lookForThis")
                    .or_else(|| get(info, "indentAfterThisHeading"));
                if !as_bool(look) {
                    continue;
                }
                let level = as_i64(get(info, "level")).unwrap_or(1) as usize;
                v.push((name, level));
            }
        }
        v
    });
    if entries.is_empty() {
        return (String::new(), Vec::new(), Vec::new());
    }
    // 星名排前；长名排前（避免 part 前缀吞掉 paragraph）
    let mut order: Vec<&(String, usize)> = entries.iter().collect();
    order.sort_by_key(|(n, l)| (!n.contains('*'), std::cmp::Reverse(n.len()), *l));
    let all = order
        .iter()
        .map(|(n, _)| regex_esc_name(n))
        .collect::<Vec<_>>()
        .join("|");

    let min_level = entries.iter().map(|e| e.1).min().unwrap_or(1);
    let max_level = entries.iter().map(|e| e.1).max().unwrap_or(1);
    let mut level_array: Vec<String> = Vec::new();
    for lvl in min_level..=max_level {
        let mut names: Vec<&(String, usize)> =
            entries.iter().filter(|e| e.1 == lvl).collect();
        if names.is_empty() {
            continue;
        }
        // 星名排前，非星名加 (?!\*) 后缀（Perl 先写 (?!*) 再转义 *）
        names.sort_by_key(|(n, _)| (!n.contains('*'), std::cmp::Reverse(n.len())));
        let joined = names
            .iter()
            .map(|(n, _)| {
                if n.contains('*') {
                    regex_esc_name(n)
                } else {
                    format!("{}(?!\\*)", regex_esc_name(n))
                }
            })
            .collect::<Vec<_>>()
            .join("|");
        level_array.push(joined);
    }
    (all, level_array, entries)
}

struct BodyPart {
    body: String,
    level: usize,
}

/// headings_get_body_parts：按标题切分，同级/更深的段落并入当前 part。
fn get_body_parts(body: &str, all_re: &str, lookup: &[(String, usize)]) -> Vec<BodyPart> {
    let re = crate::patterns::re_new(&format!(r"(?s)(\\({all_re}))")).expect("headings regexp");
    let mut raw: Vec<String> = Vec::new();
    let mut last = 0usize;
    for m in re.captures_iter(body.as_bytes()) {
        let c = m.expect("match");
        let whole = c.get(0).unwrap();
        raw.push(body[last..whole.start()].to_string());
        raw.push(gn(&c, 1).to_string());
        raw.push(gn(&c, 2).to_string());
        last = whole.end();
    }
    raw.push(body[last..].to_string());

    let level_of = |name: &str| -> usize {
        lookup
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, l)| *l)
            .unwrap_or(0)
    };

    // 严格按 Perl 的索引规则重组
    let mut parts: Vec<BodyPart> = Vec::new();
    let mut current_heading_level = 0usize;
    let mut heading_value = String::new();
    for (index, chunk) in raw.iter().enumerate() {
        if index == 0 {
            parts.push(BodyPart {
                body: chunk.clone(),
                level: 0,
            });
        } else if index == 1 || index == 3 {
            parts[0].body.push_str(chunk);
        } else if index == 2 {
            parts[0].level = level_of(chunk);
        } else if index % 3 == 1 {
            heading_value = chunk.clone();
        } else if index % 3 == 2 {
            current_heading_level = level_of(chunk);
        } else {
            let last_level = parts.last().map(|p| p.level).unwrap_or(0);
            if current_heading_level >= last_level {
                if let Some(p) = parts.last_mut() {
                    p.body.push_str(&heading_value);
                    p.body.push_str(chunk);
                }
            } else {
                parts.push(BodyPart {
                    body: format!("{heading_value}{chunk}"),
                    level: current_heading_level,
                });
            }
        }
    }
    parts
}

/// find_heading：处理 indentAfterHeadings 的后续缩进。
pub fn find_heading(body: &str) -> String {
    let (all_re, level_array, lookup) = construct_levels();
    if level_array.is_empty() {
        return body.to_string();
    }

    // preamble 已经缩进过了，现在把它改成 verbatim，
    // 这样 preamble 里的 \section 之类不会被当成标题（Heading.pm:142-148）。
    let mut body = body.to_string();
    let preamble_wanted = state::settings(|s| as_bool(get(&s.root, "indentPreamble")));
    if preamble_wanted {
        logger::trace(
            "*protecting preamble which can contain headings commands (indentPreamble: 1)",
        );
        state::settings_mut(|s| {
            s.root
                .insert(Value::String("indentPreamble".into()), Value::Number(0.into()));
        });
        body = crate::file_contents::find_file_contents_environments_and_preamble(&body);
    }

    let parts = get_body_parts(&body, &all_re, &lookup);
    let mut out = String::new();
    for p in &parts {
        out.push_str(&after_heading_indentation(&p.body, 0, &all_re, &level_array, &lookup));
    }

    if preamble_wanted {
        out = crate::verbatim::put_verbatim_back_in(&out, "preamble");
    }
    out
}

/// after_heading_indentation：当前级别（数组索引）的标题后内容加缩进，并递归更深级别。
fn after_heading_indentation(
    body: &str,
    current_level: usize,
    all_re: &str,
    level_array: &[String],
    lookup: &[(String, usize)],
) -> String {
    let pat = match level_array.get(current_level) {
        Some(p) if !p.is_empty() => p.clone(),
        _ => return body.to_string(),
    };
    let re = crate::patterns::re_new(&format!(r"(?s)(\\({pat}))")).expect("level regexp");
    if !re.is_match(body.as_bytes()).unwrap_or(false) {
        return after_heading_indentation(body, current_level + 1, all_re, level_array, lookup);
    }
    // [文本, "\"+名, 名, 文本, ...]
    let mut out: Vec<String> = Vec::new();
    let mut last = 0usize;
    for m in re.captures_iter(body.as_bytes()) {
        let c = m.expect("match");
        let whole = c.get(0).unwrap();
        out.push(body[last..whole.start()].to_string());
        out.push(gn(&c, 1).to_string());
        out.push(gn(&c, 2).to_string());
        last = whole.end();
    }
    out.push(body[last..].to_string());

    let mut heading_value = String::new();
    let mut name = String::new();
    for index in 1..out.len() {
        if index % 3 == 1 {
            heading_value = out[index].clone();
            continue;
        }
        if index % 3 == 2 {
            name = out[index].clone();
            out[index] = String::new();
            logger::trace(&format!("*found: {name} heading"));
            continue;
        }
        // 正文段
        let mut text = out[index].clone();
        let sub_parts = get_body_parts(&text, all_re, lookup);
        if sub_parts.len() > 1 {
            let mut merged = String::new();
            for (k, sp) in sub_parts.iter().enumerate() {
                if k + 1 == sub_parts.len() {
                    merged.push_str(&sp.body);
                } else {
                    merged.push_str(&after_heading_indentation(
                        &sp.body, 0, all_re, level_array, lookup,
                    ));
                }
            }
            text = merged;
        }
        text = after_heading_indentation(&text, current_level + 1, all_re, level_array, lookup);

        let os = state::settings_mut(|s| s.settings_for_object(&name, "afterHeading", false, false));
        let added = os.indentation.clone();
        if !added.is_empty() {
            let mut tail1 = String::new();
            let mut tail2 = String::new();
            let blocks_end_before: Option<String> = state::settings(|s| {
                get_path_opt(&s.root, &["indentAfterHeadings", &name, "blocksEndBefore"])
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
            });
            if let Some(beb) = blocks_end_before {
                if let Ok(re) = crate::patterns::re_new(&format!(r"(?s)({beb})")) {
                    if let Some(c) = re.captures(text.as_bytes()).unwrap() {
                        let whole = c.get(0).unwrap();
                        let (s, e) = (whole.start(), whole.end());
                        tail1 = text[s..e].to_string();
                        tail2 = text[e..].to_string();
                        text = text[..s].to_string();
                    }
                }
            }
            let re = crate::patterns::re_new(r"(?m)^").unwrap();
            text = replace_all_with(&text, &re, |_| added.clone());
            text = text.replacen(&added, "", 1);
            // s"\R$added(\h*)$"\n$1"s：去掉末行多余缩进
            let re = crate::patterns::re_new(&format!(r"(?s)\R{}(\h*)$", regex_esc_str(&added)))
                .unwrap();
            text = replace_all_with(&text, &re, |c| format!("\n{}", gn(c, 1)));
            text.push_str(&tail1);
            text.push_str(&tail2);
        }
        out[index] = text;
    }
    out.join("")
}

fn regex_esc_str(s: &str) -> String {
    let mut out = String::new();
    for ch in s.chars() {
        if r"\.^$|?*+()[]{}".contains(ch) {
            out.push('\\');
        }
        out.push(ch);
    }
    out
}

fn get_path_opt<'a>(map: &'a serde_yaml::Mapping, path: &[&str]) -> Option<&'a serde_yaml::Value> {
    let mut cur = crate::settings::get(map, *path.first()?)?;
    for k in &path[1..] {
        cur = cur.as_mapping().and_then(|m| crate::settings::get(m, k))?;
    }
    Some(cur)
}
