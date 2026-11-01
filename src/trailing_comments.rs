//! 尾注释（对应 LatexIndent/TrailingComments.pm）。

use crate::patterns::esc_lit;
use crate::settings::{as_bool, get_path};
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
    // %*\begin{...} markup 检测（对齐块）：模式恒定，只编译一次
    let re_markup =
        crate::patterns::re_new(r"(?s)^\*\h*\\(?:begin|end)\{([^}]+)\}*").expect("align markup");
    let out = crate::patterns::replace_all_with(body, &re, |caps| {
        let value = crate::patterns::gn(caps, 1).to_string();
        // %*\begin{...} markup 检测（对齐块）
        let align_markup = {
            match re_markup.captures(value.as_bytes()).unwrap_or(None) {
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
        let tc_esc = esc_lit(&t.trailing_comment);
        let eot_esc = esc_lit(&t.end_of_token);
        let re_end =
            crate::patterns::re_new(&format!(r"(?s){not_preceded_by}%{tc_esc}\d+{eot_esc}\h*\Z"))
                .expect("trailing comment end regex");
        let flag = re_end.is_match(out.as_bytes()).unwrap_or(false);
        state::storage(|s| s.file_ends_with_trailing_comment = flag);
    }
    out
}

/// put_trailing_comments_back_in：token 还原为 % 注释文本。
///
/// 旧实现逐条注释编译两条正则并**整篇扫描 1–2 次**，2 MB 文档上是 O(n×m)
/// （实测 0.76 s）。这里改成：先扫一遍拿到所有 token 的位置，
/// 若每行最多只有一个 token（`%` 注释到行尾为止，正常文档都是如此），
/// 就单遍拼出结果；否则回退到逐个 token 的旧路径（token 会互相影响时保持原语义）。
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
            esc_lit(&t.trailing_comment),
            esc_lit(&t.end_of_token)
        ))
        .unwrap();
        body = crate::patterns::replace_all_with(&body, &re, |caps| {
            format!("{}\n", crate::patterns::gn(caps, 1))
        });
    }

    let re = crate::patterns::re_new(&format!(
        r"%{}\d+{}",
        esc_lit(&t.trailing_comment),
        esc_lit(&t.end_of_token)
    ))
    .unwrap();
    let spans: Vec<(usize, usize)> = re
        .captures_iter(body.as_bytes())
        .flatten()
        .map(|c| {
            let m = c.get(0).unwrap();
            (m.start(), m.end())
        })
        .collect();

    // 每行是否只有一个 token？否则走旧路径（见函数注释）。
    let mut multi_token_line = false;
    {
        let mut line = 0usize;
        let mut pos = 0usize;
        let mut last_line: Option<usize> = None;
        for &(start, _) in &spans {
            line += body[pos..start].matches('\n').count();
            pos = start;
            if last_line == Some(line) {
                multi_token_line = true;
            }
            last_line = Some(line);
        }
    }

    if !multi_token_line {
        return restore_single_pass(&body, &spans, &comments);
    }
    restore_per_token(body, &comments)
}

/// 单遍还原（每行至多一个 token，互不影响）。
///
/// `at_line_end` 的判据照抄 Perl 的条件正则 `token((?:(?!(?<!\\)%).*?))(\h*)?$`：
/// 注意那个负向断言**没有量化**，所以只在 token 紧后面检查一次——
/// 也就是「token 紧跟的字符是不是未转义的 `%`」；之后 `.*?` 可以吞任意字符到行尾。
/// 断言不成立（紧跟 `%`）时整条正则匹配不到，Perl 那边当作「在行尾」处理。
fn restore_single_pass(
    body: &str,
    spans: &[(usize, usize)],
    comments: &[state::TrailingComment],
) -> String {
    let mut out = String::with_capacity(body.len());
    let mut last = 0usize;
    for &(start, end) in spans {
        let id = &body[start + 1..end];
        let Some(c) = comments.iter().find(|c| c.id == id) else {
            continue;
        };
        out.push_str(&body[last..start]);
        let line_end = body[end..]
            .find('\n')
            .map(|i| end + i)
            .unwrap_or(body.len());
        let rest = &body[end..line_end];
        let at_line_end = rest.starts_with('%')
            || rest
                .trim_end_matches(crate::whitespace::is_hspace)
                .is_empty();
        if at_line_end {
            out.push('%');
            out.push_str(&c.value);
            last = end;
        } else {
            // 不在行尾：整行剩余内容前移，注释正文挪到换行前
            out.push_str(rest);
            out.push('%');
            out.push_str(&c.value);
            last = line_end;
        }
    }
    out.push_str(&body[last..]);
    out
}

/// 旧路径：同一行出现多个 token 时，token 之间会互相影响，逐条处理保持原语义。
fn restore_per_token(mut body: String, comments: &[state::TrailingComment]) -> String {
    for c in comments.iter().rev() {
        let token = format!("%{}", c.id);
        let esc = esc_lit(&token);
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
