//! 尾注释（对应 LatexIndent/TrailingComments.pm）。

use crate::settings::{as_bool, get, get_path};
use crate::state;
use crate::tokens::tok;

/// add_comment_symbol：注册注释内容，返回 "tcN-END"。
pub fn add_comment_symbol(value: &str) -> String {
    // Perl $tokens{trailingComment}：token_check 可能已把 tc 改名（如撞 document**tc**lass），
    // 写死 "tc" 会造出与 tc 正则失配的 token
    let t = crate::tokens::tok(|t| t.clone());
    state::storage(|s| {
        s.comment_counter += 1;
        let id = format!(
            "{}{}{}",
            t.trailing_comment, s.comment_counter, t.end_of_token
        );
        s.trailing_comments.push(state::TrailingComment {
            id: id.clone(),
            value: value.to_string(),
            align_markup: false,
        });
        id
    })
}

/// remove_trailing_comments：把所有 % 注释替换为 token。
pub fn remove_trailing_comments(body: &str) -> String {
    let t = tok(|t| t.clone());
    let after_comment = state::settings(|s| {
        get_path(&s.root, &["fineTuning", "trailingComments", "afterComment"])
            .and_then(|v| v.as_str())
            .unwrap_or(".*?")
            .to_string()
    });
    let not_preceded_by = state::settings(|s| {
        get_path(
            &s.root,
            &["fineTuning", "trailingComments", "notPrecededBy"],
        )
        .and_then(|v| v.as_str())
        .unwrap_or(r"(?<!\\)")
        .to_string()
    });
    let re = crate::patterns::re_new(&format!(r"(?m){not_preceded_by}%({after_comment})$"))
        .expect("trailing comment regex");
    let out = crate::patterns::replace_all_with(body, &re, |caps| {
        let value = crate::patterns::gn(caps, 1).to_string();
        // %*\begin{...} markup 检测（对齐块）
        let align_markup = {
            let re2 = crate::patterns::re_new(r"(?s)^\*\h*\\(?:begin|end)\{([^}]+)\}*").unwrap();
            match re2.captures(value.as_bytes()).unwrap() {
                Some(c) => {
                    let name = crate::patterns::gn(&c, 1);
                    // Perl：标量真值或映射（哈希引用恒真）均视为标记
                    state::settings(|s| match get_path(&s.root, &["lookForAlignDelims", name]) {
                        Some(v) => as_bool(Some(v)) || v.as_mapping().is_some(),
                        None => false,
                    })
                }
                None => false,
            }
        };
        if align_markup {
            state::storage(|s| s.align_markup_block_present = true);
        }
        let id = state::storage(|s| {
            s.comment_counter += 1;
            let id = format!(
                "{}{}{}",
                t.trailing_comment, s.comment_counter, t.end_of_token
            );
            s.trailing_comments.push(state::TrailingComment {
                id: id.clone(),
                value: value.to_string(),
                align_markup,
            });
            id
        });
        format!("%{id}")
    });

    // Perl TrailingComments.pm:111-117 —— 记录文件是否以尾注释结束（-m 时）。
    // 对应 `=~ m/$trailingCommentRegExp\h*\Z/s`，其中 \Z 允许结尾一个换行。
    if crate::switches::with(|s| s.is_m_switch_active()) {
        let tc_esc = regex_esc(&t.trailing_comment);
        let eot_esc = regex_esc(&t.end_of_token);
        let re_end =
            crate::patterns::re_new(&format!(r"(?s){not_preceded_by}%{tc_esc}\d+{eot_esc}\h*\Z"))
                .expect("trailing comment end regex");
        let flag = re_end.is_match(out.as_bytes()).unwrap_or(false);
        state::storage(|s| s.file_ends_with_trailing_comment = flag);
    }
    out
}

/// put_trailing_comments_back_in：token 还原为 % 注释文本。
pub fn put_trailing_comments_back_in(body: &str) -> String {
    let t = tok(|t| t.clone());
    let mut body = body.to_string();
    let (comments, file_ends) = state::storage(|s| {
        (
            std::mem::take(&mut s.trailing_comments),
            s.file_ends_with_trailing_comment,
        )
    });
    if comments.is_empty() {
        return body;
    }
    if file_ends {
        // 末尾注释补一个换行（Perl：s/($trailingCommentRegExp)\s*$/$1\n/s）
        let re = crate::patterns::re_new(&format!(
            r"(?s)(%{}\d+{})\s*$",
            regex_esc(&t.trailing_comment),
            regex_esc(&t.end_of_token)
        ))
        .unwrap();
        body = crate::patterns::replace_all_with(&body, &re, |caps| {
            format!("{}\n", crate::patterns::gn(caps, 1))
        });
    }
    for c in comments.iter().rev() {
        let token = format!("%{}", c.id);
        let esc = regex_esc(&token);
        // Perl：若注释 token 不在行尾（同行其后有非 % 内容），把注释移到行尾
        let re_cond =
            crate::patterns::re_new(&format!(r"(?m){esc}((?:(?!(?<!\\)%).*?))(\h*)?$")).unwrap();
        let at_line_end = match re_cond.captures(body.as_bytes()).unwrap() {
            Some(caps) => crate::patterns::gn(&caps, 1).is_empty(),
            None => true,
        };
        if !at_line_end {
            let re_sub = crate::patterns::re_new(&format!(r"(?m){esc}(.*)$")).unwrap();
            body = crate::patterns::replace_all_with(&body, &re_sub, |caps| {
                format!("{}%{}", crate::patterns::gn(caps, 1), c.value)
            });
        } else {
            body = body.replacen(&token, &format!("%{}", c.value), 1);
        }
    }
    body
}

pub(crate) fn regex_esc(s: &str) -> String {
    let mut out = String::new();
    for ch in s.chars() {
        if r"\.^$|?*+()[]{}".contains(ch) {
            out.push('\\');
        }
        out.push(ch);
    }
    out
}
