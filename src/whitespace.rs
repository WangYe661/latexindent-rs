//! 空白处理（对应 LatexIndent/HorizontalWhiteSpace.pm）。

use crate::settings::{as_bool, as_str, get};
use crate::state;

fn remove_trailing_ws_enabled(when: &str) -> bool {
    state::settings(|s| {
        match get(&s.root, "removeTrailingWhitespace") {
            Some(v) if v.is_mapping() => {
                let key = if when == "before" {
                    "beforeProcessing"
                } else {
                    "afterProcessing"
                };
                as_bool(get(v.as_mapping().unwrap(), key))
            }
            other => as_bool(other),
        }
    })
}

/// remove_trailing_whitespace(before|after)
pub fn remove_trailing_whitespace(body: &str, when: &str) -> String {
    if !remove_trailing_ws_enabled(when) {
        return body.to_string();
    }
    let re = crate::patterns::re_new(r"(?m)\h+$").unwrap();
    crate::patterns::replace_all_with(body, &re, |_| String::new())
}

/// remove_leading_space：去掉所有行首水平空白（verbatim 已先行保护）。
pub fn remove_leading_space(body: &str) -> String {
    let re = crate::patterns::re_new(r"(?m)^\h*").unwrap();
    crate::patterns::replace_all_with(body, &re, |_| String::new())
}

/// max_indentation_check：tab 归位 + maximumIndentation 截断。
pub fn max_indentation_check(body: &str) -> String {
    // 混排缩进中的 tab 移到最前
    let re = crate::patterns::re_new(r"(?m)^((?:[ \t]*))").unwrap();
    let mut body = crate::patterns::replace_all_with(body, &re, |caps| {
        let indent = crate::patterns::gn(caps, 1);
        let tabs = indent.matches('\t').count();
        let spaces: String = indent.chars().filter(|c| *c != '\t').collect();
        format!("{}{}", "\t".repeat(tabs), spaces)
    });

    let max_indent = state::settings(|s| {
        as_str(get(&s.root, "maximumIndentation")).unwrap_or("-1").to_string()
    });
    let is_hspace = !max_indent.is_empty() && max_indent.chars().all(|c| c == ' ' || c == '\t');
    if !is_hspace {
        return body;
    }
    body = expand_tabs(&body);
    let len = max_indent.chars().count();
    let re = crate::patterns::re_new(&format!(r"(?m)^\h{{{len},}}")).unwrap();
    body = crate::patterns::replace_all_with(&body, &re, |_| max_indent.clone());
    body
}

/// Text::Tabs::expand 的等价实现（tabstop=8）。
fn expand_tabs(s: &str) -> String {
    let mut out = String::new();
    for line in s.split('\n') {
        let mut col = 0usize;
        for c in line.chars() {
            if c == '\t' {
                let n = 8 - (col % 8);
                out.push_str(&" ".repeat(n));
                col += n;
            } else {
                out.push(c);
                col += 1;
            }
        }
        out.push('\n');
    }
    out.pop();
    out
}
