//! 空行保护（对应 LatexIndent/BlankLines.pm）。

use crate::settings::{as_bool, as_i64, get, get_path};
use crate::state;
use crate::tokens::tok;

/// protect_blank_lines：空行 → blanklines token + 换行。
pub fn protect_blank_lines(body: &str) -> String {
    if !state::settings(|s| {
        as_bool(get_path(
            &s.root,
            &["modifyLineBreaks", "preserveBlankLines"],
        ))
    }) {
        return body.to_string();
    }
    let t = tok(|t| t.clone());
    let re = crate::patterns::re_new(r"(?m)^(\h*)?\R").unwrap();
    crate::patterns::replace_all_with(body, &re, |_| format!("{}\n", t.blanklines))
}

/// condense_blank_lines：连续空行 token 压缩为 condenseMultipleBlankLinesInto 个。
pub fn condense_blank_lines(body: &str) -> String {
    let t = tok(|t| t.clone());
    let mut body = body.to_string();
    if !state::settings(|s| {
        as_bool(get_path(
            &s.root,
            &["modifyLineBreaks", "preserveBlankLines"],
        ))
    }) {
        // Perl：到这一步仍可能被移除的空行已处理完；翻转 preserveBlankLines 并补 protect
        state::settings_mut(|s| {
            crate::settings::set_path(
                &mut s.root,
                &["modifyLineBreaks", "preserveBlankLines"],
                serde_yaml::Value::Bool(true),
            )
        });
        body = protect_blank_lines(&body);
    }
    let n = state::settings(|s| {
        as_i64(get_path(
            &s.root,
            &["modifyLineBreaks", "condenseMultipleBlankLinesInto"],
        ))
        .unwrap_or(1)
    });
    let mut replacement = String::new();
    for i in 1..n.max(1) {
        replacement.push_str(&t.blanklines);
        replacement.push('\n');
    }
    replacement.push_str(&t.blanklines);
    let token_esc = regex_esc(&t.blanklines);
    let re = crate::patterns::re_new(&format!(r"(?m)(?:{token_esc}\h*\R*\h*){{1,}}{token_esc}"))
        .unwrap();
    crate::patterns::replace_all_with(&body, &re, |_| replacement.clone())
}

/// unprotect_blank_lines：blanklines token → 真实空行。
pub fn unprotect_blank_lines(body: &str) -> String {
    let t = tok(|t| t.clone());
    let token = t.blanklines.clone();
    let mut body = body.to_string();
    // 1) 清掉 text_wrap 产生的空行
    let re = crate::patterns::re_new(r"(?m)^\h*\R").unwrap();
    body = crate::patterns::replace_all_with(&body, &re, |_| String::new());
    loop {
        if !body.contains(&token) {
            break;
        }
        let before = body.clone();
        // 2) token 独占整行 → 删除
        let re = crate::patterns::re_new(&format!(r"(?m)^\h*{}\h*$", regex_esc(&token))).unwrap();
        body = crate::patterns::replace_all_with(&body, &re, |_| String::new());
        // 3) 行首 token（后有内容）→ 变换行
        let re = crate::patterns::re_new(&format!(r"(?m)(^\h*){}", regex_esc(&token))).unwrap();
        body = crate::patterns::replace_all_with(&body, &re, |caps| {
            format!("\n{}", crate::patterns::gn(caps, 1))
        });
        // 4) 前后皆有内容
        let re = crate::patterns::re_new(&format!(r"(?m)^(.*?){}\h*(.*?)\h*$", regex_esc(&token)))
            .unwrap();
        body = crate::patterns::replace_all_with(&body, &re, |caps| {
            let a = crate::patterns::gn(caps, 1);
            let b = crate::patterns::gn(caps, 2);
            if b.is_empty() {
                format!("{a}\n")
            } else {
                format!("{a}\n\n{b}")
            }
        });
        // 5) 仅后有内容
        let re = crate::patterns::re_new(&format!(r"(?m)^{}\h*(.*?)$", regex_esc(&token))).unwrap();
        body = crate::patterns::replace_all_with(&body, &re, |caps| {
            format!("{}\n", crate::patterns::gn(caps, 1))
        });
        if body == before {
            break;
        }
    }
    body
}

fn regex_esc(s: &str) -> String {
    let mut out = String::new();
    for ch in s.chars() {
        if r"\.^$|?*+()[]{}".contains(ch) {
            out.push('\\');
        }
        out.push(ch);
    }
    out
}
