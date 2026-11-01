//! 空白处理（对应 LatexIndent/HorizontalWhiteSpace.pm）。

use crate::settings::{as_bool, as_str, get};
use crate::state;

fn remove_trailing_ws_enabled(when: &str) -> bool {
    state::settings(|s| match get(&s.root, "removeTrailingWhitespace") {
        Some(v) if v.is_mapping() => {
            let key = if when == "before" {
                "beforeProcessing"
            } else {
                "afterProcessing"
            };
            as_bool(get(v.as_mapping().unwrap(), key))
        }
        other => as_bool(other),
    })
}

/// PCRE2 的 `\h`：水平空白（不含 `\r`/`\v`）。
///
/// 手写扫描必须与 `\h` 的字面含义完全一致，否则行尾 CR、不间断空格这类字符
/// 的处理会与正则版不同（`\r` 不属于 `\h`，所以 CRLF 文件里 `\h+$` 匹配不到
/// 行尾空格——这个 quirk 必须保留）。
/// PCRE2 `\h`：水平空白（`\r` 不属于 `\h`）。供逐行手写扫描复用。
pub fn is_hspace(c: char) -> bool {
    matches!(
        c,
        '\u{09}'
            | '\u{20}'
            | '\u{a0}'
            | '\u{1680}'
            | '\u{2000}'..='\u{200a}'
            | '\u{202f}'
            | '\u{205f}'
            | '\u{3000}'
    )
}

/// 逐行处理（按 `\n` 切分），保留换行符与 `\r`；整篇只走一遍、只分配一次输出缓冲。
fn map_lines(body: &str, mut f: impl FnMut(&str, &mut String)) -> String {
    let mut out = String::with_capacity(body.len());
    let mut rest = body;
    while let Some(nl) = rest.find('\n') {
        f(&rest[..nl], &mut out);
        out.push('\n');
        rest = &rest[nl + 1..];
    }
    f(rest, &mut out);
    out
}

/// remove_trailing_whitespace(before|after)：等价于 `(?m)\h+$`。
pub fn remove_trailing_whitespace(body: &str, when: &str) -> String {
    if !remove_trailing_ws_enabled(when) {
        return body.to_string();
    }
    map_lines(body, |line, out| {
        out.push_str(line.trim_end_matches(is_hspace));
    })
}

/// remove_leading_space：去掉所有行首水平空白（verbatim 已先行保护）。
pub fn remove_leading_space(body: &str) -> String {
    map_lines(body, |line, out| {
        out.push_str(line.trim_start_matches(is_hspace));
    })
}

/// max_indentation_check：tab 归位 + maximumIndentation 截断。
pub fn max_indentation_check(body: &str) -> String {
    // 混排缩进中的 tab 移到最前：`(?m)^((?:[ \t]*))` 的替换
    let mut body = map_lines(body, |line, out| {
        let indent_len = line.find(|c| c != ' ' && c != '\t').unwrap_or(line.len());
        let (indent, tail) = line.split_at(indent_len);
        let tabs = indent.bytes().filter(|b| *b == b'\t').count();
        if tabs == 0 {
            out.push_str(line);
            return;
        }
        for _ in 0..tabs {
            out.push('\t');
        }
        for _ in 0..(indent.len() - tabs) {
            out.push(' ');
        }
        out.push_str(tail);
    });

    let max_indent = state::settings(|s| {
        as_str(get(&s.root, "maximumIndentation"))
            .unwrap_or("-1")
            .to_string()
    });
    let is_hspace_literal =
        !max_indent.is_empty() && max_indent.chars().all(|c| c == ' ' || c == '\t');
    if !is_hspace_literal {
        return body;
    }
    body = expand_tabs(&body);
    let len = max_indent.chars().count();
    map_lines(&body, |line, out| {
        let mut cut = 0usize;
        let mut n = 0usize;
        for c in line.chars() {
            if !is_hspace(c) {
                break;
            }
            n += 1;
            cut += c.len_utf8();
        }
        if n >= len {
            out.push_str(&max_indent);
            out.push_str(&line[cut..]);
        } else {
            out.push_str(line);
        }
    })
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hspace_matches_pcre2_h() {
        for c in ['\t', ' ', '\u{a0}', '\u{2003}', '\u{3000}'] {
            assert!(is_hspace(c), "{c:?} 应算 \\h");
        }
        // \h 不含 \r / \n / \v
        for c in ['\r', '\n', '\u{b}', '\u{c}', 'a'] {
            assert!(!is_hspace(c), "{c:?} 不应算 \\h");
        }
    }

    #[test]
    fn remove_leading_space_matches_regex_semantics() {
        // 行首空白删除；CRLF 的 \r 不是 \h（保留），行中间的空格不动
        assert_eq!(remove_leading_space("  a b\n\tc\nd "), "a b\nc\nd ");
        assert_eq!(remove_leading_space(" \r\nx"), "\r\nx");
        assert_eq!(remove_leading_space("\u{2003}a"), "a");
        // 只有空白的一行会变成空行
        assert_eq!(remove_leading_space("a\n \t \nb"), "a\n\nb");
    }
}
