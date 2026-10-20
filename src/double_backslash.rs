//! \\ 后随括号的保护（对应 LatexIndent/DoubleBackSlash.pm）。

use crate::tokens::tok;

/// dodge_double_backslash：`\\{ \\} \\]` → token + 括号。
pub fn dodge_double_backslash(body: &str) -> String {
    let t = tok(|t| t.clone());
    let re = crate::patterns::re_new(r"(?:\\\\(\{|\}|\]))").unwrap();
    crate::patterns::replace_all_with(body, &re, |caps| {
        let c = crate::patterns::gn(caps, 1);
        format!("{}{}", t.double_back_slash, c)
    })
}

/// un_dodge_double_backslash：token → `\\`。
pub fn un_dodge_double_backslash(body: &str) -> String {
    let t = tok(|t| t.clone());
    body.replace(&t.double_back_slash, "\\\\")
}
