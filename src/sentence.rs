//! one-sentence-per-line（对应 LatexIndent/Sentence.pm）。

use pcre2::bytes::Regex;

use crate::patterns::{gn, replace_all_with};
use crate::settings::{as_bool, as_i64, as_str, get};
use crate::state::{self, SentenceObj};
use crate::tokens::tok;

fn re(src: &str) -> Regex {
    crate::patterns::re_new(src).expect("sentence regex")
}

fn osp<T>(f: impl FnOnce(&serde_yaml::Mapping) -> T) -> T {
    state::settings(|s| {
        let m = get(&s.root, "modifyLineBreaks")
            .and_then(|v| v.as_mapping())
            .and_then(|v| get(v, "oneSentencePerLine"))
            .and_then(|v| v.as_mapping())
            .cloned()
            .unwrap_or_default();
        f(&m)
    })
}

fn yes_no(m: &serde_yaml::Mapping, key: &str) -> Option<String> {
    let v = get(m, key)?;
    match v {
        serde_yaml::Value::String(s) if s != "0" && !s.is_empty() => Some(s.clone()),
        serde_yaml::Value::String(_)
        | serde_yaml::Value::Bool(false)
        | serde_yaml::Value::Number(_) => {
            if as_bool(Some(v)) {
                // 标量 1 无正则可用
                None
            } else {
                None
            }
        }
        serde_yaml::Value::Mapping(mm) => as_str(get(mm, "other")).map(|s| s.to_string()),
        _ => None,
    }
}

/// one_sentence_per_line 主入口。
pub fn one_sentence_per_line(body: &str) -> String {
    let t = tok(|t| t.clone());
    let tc = state::patterns(|p| p.trailing_comment.clone());

    // ---- sentencesFollow ----
    let (follow_alt, blank_on, par_on) = osp(|m| {
        let follow = get(m, "sentencesFollow")
            .and_then(|v| v.as_mapping())
            .cloned()
            .unwrap_or_default();
        let mut parts: Vec<String> = Vec::new();
        let mut blank_on = false;
        let mut par_on = false;
        let mut push = |k: &str, piece: Option<String>, parts: &mut Vec<String>| {
            if as_bool(get(&follow, k)) {
                if let Some(p) = piece {
                    parts.push(p);
                }
            }
        };
        if as_bool(get(&follow, "par")) {
            par_on = true;
            parts.push(r"\R?\\par(?![a-zA-Z])".to_string());
        }
        if as_bool(get(&follow, "blankLine")) {
            blank_on = true;
            let bl = &t.blanklines;
            parts.push(format!(
                r"(?:\A(?:{bl}\R)+)|(?:\G(?:{bl}\R)+)|(?:(?:{bl}\h*\R)+)|\R{{2,}}|\G"
            ));
        }
        push("fullStop", Some(r"\.".to_string()), &mut parts);
        push("exclamationMark", Some(r"\!".to_string()), &mut parts);
        push("questionMark", Some(r"\?".to_string()), &mut parts);
        push("rightBrace", Some(r"\}".to_string()), &mut parts);
        push(
            "commentOnPreviousLine",
            Some(format!(r"{tc}\h*\R")),
            &mut parts,
        );
        if let Some(o) = yes_no(&follow, "other") {
            parts.push(o);
        }
        if !blank_on {
            parts.push(format!(r"\G(?!{})", t.blanklines));
        }
        (parts.join("|"), blank_on, par_on)
    });

    let follow = if blank_on {
        format!("(?:(?:{follow_alt}))(?:\\h|\\R)*")
    } else {
        format!("(?:(?:{follow_alt}))(?:\\h*\\R?)")
    };

    // ---- sentencesBeginWith / sentencesEndWith ----
    let begin_alt = osp(|m| {
        let b = get(m, "sentencesBeginWith")
            .and_then(|v| v.as_mapping())
            .cloned()
            .unwrap_or_default();
        let mut parts: Vec<String> = Vec::new();
        if as_bool(get(&b, "A-Z")) {
            parts.push(format!(
                r"(?!(?:{}|{}|{}))[A-Z]",
                t.blanklines, t.verbatim, t.preamble
            ));
        }
        if as_bool(get(&b, "a-z")) {
            parts.push("[a-z]".to_string());
        }
        if let Some(o) = yes_no(&b, "other") {
            // 同上：外层整个正则跑在 `(?sx)` 下，补一个换行（/x 里被忽略）
            parts.push(format!("{o}\n"));
        }
        parts.join("|")
    });

    let better_full_stop = state::settings(|s| {
        as_str(crate::settings::get_path(
            &s.root,
            &["fineTuning", "modifyLineBreaks", "betterFullStop"],
        ))
        .unwrap_or("")
        .to_string()
    });

    let end_alt = osp(|m| {
        let e = get(m, "sentencesEndWith")
            .and_then(|v| v.as_mapping())
            .cloned()
            .unwrap_or_default();
        let mut parts: Vec<String> = Vec::new();
        let better = as_bool(get(&e, "betterFullStop"));
        if as_bool(get(&e, "basicFullStop")) && !better {
            parts.push(r"\.".to_string());
        }
        if better && !better_full_stop.is_empty() {
            // (?x) 片段尾行可能是注释，补换行避免吞掉收尾括号
            parts.push(format!("(?:(?:{}\n))", better_full_stop));
        }
        if as_bool(get(&e, "exclamationMark")) {
            parts.push("!".to_string());
        }
        if as_bool(get(&e, "questionMark")) {
            parts.push(r"\?".to_string());
        }
        if let Some(o) = yes_no(&e, "other") {
            // 同 betterFullStop：`(?x)` 片段的尾行可能是 `#` 注释，补换行免得吞掉收尾括号
            parts.push(format!("{o}\n"));
        }
        parts.join("|")
    });

    // ---- 排除模式 ----
    let mut not_within = vec![tc.clone()];
    if !blank_on {
        not_within.push(format!(r"(?:\h*\R?{})", t.blanklines));
    }
    if par_on {
        not_within.push(r"(?:\R?\\par(?![a-zA-Z]))".to_string());
    }
    let not_within = not_within.join("|");

    let dont_contain = osp(|m| {
        get(m, "sentencesDoNOTcontain")
            .and_then(|v| v.as_mapping())
            .and_then(|mm| as_str(get(mm, "other")))
            // 外层是 `(?sx)`：补换行防止片段尾行的 `#` 注释吞掉后面的 `))`
            .map(|s| format!("{s}\n"))
    });

    let sentence_core = match &dont_contain {
        Some(dc) => format!(
            r"((?:{follow}))(\h*)(?!{not_within})((?:(?:{begin_alt})(?:(?!(?:{dc})).)*?))(?<SENTEND>(?:{end_alt}))(\h*)?(\R)?({tc})?"
        ),
        None => format!(
            r"((?:{follow}))(\h*)(?!{not_within})((?:(?:{begin_alt}).*?))(?<SENTEND>(?:{end_alt}))(\h*)?(\R)?({tc})?"
        ),
    };
    let sentence_re = crate::patterns::re_new(&format!("(?sx){sentence_core}"))
        .unwrap_or_else(|e| panic!("sentence regex: {e}\n{sentence_core}"));

    let cfg_remove_lb = osp(|m| as_bool(get(m, "removeSentenceLineBreaks")));
    let cfg_multi = osp(|m| as_bool(get(m, "multipleSpacesToSingle")));
    let cfg_indent = osp(|m| as_str(get(m, "sentenceIndent")).unwrap_or("").to_string());
    let cfg_wrap_sentences = osp(|m| as_bool(get(m, "textWrapSentences")));
    let text_wrap_after = state::settings(|s| {
        as_str(crate::settings::get_path(
            &s.root,
            &["modifyLineBreaks", "textWrapOptions", "when"],
        ))
        .unwrap_or("before")
            == "after"
    });
    let sentence_work_after_indentation =
        (cfg_wrap_sentences && text_wrap_after) || cfg_indent.contains([' ', '\t']);

    let comments_wrap = state::settings(|s| {
        as_bool(crate::settings::get_path(
            &s.root,
            &["modifyLineBreaks", "textWrapOptions", "comments", "wrap"],
        ))
    });

    let out = replace_all_with(body, &sentence_re, |caps| {
        let beginning = gn(caps, 1).to_string();
        let h_space = gn(caps, 2).to_string();
        let mut middle = gn(caps, 3).to_string();
        let end = gs_local(caps, "SENTEND");
        let tail = format!("{}{}{}", gn(caps, 6), gn(caps, 7), gn(caps, 8));
        // Perl: lineBreaksAtEnd = ($6 ? 1 : ($7 ? 1 : 0)) —— 行尾换行（g6）或尾注释（g7）
        let line_breaks_at_end = !gn(caps, 6).is_empty() || !gn(caps, 7).is_empty();
        let mut trailing_comments = String::new();

        if cfg_remove_lb {
            loop {
                let r = re(&format!(r"(?s)\h*({tc})"));
                let c2 = match r.captures(middle.as_bytes()).unwrap() {
                    Some(c) => c,
                    None => break,
                };
                let comment = gn(&c2, 1).to_string();
                middle = middle.replacen(&comment, "", 1);
                if !trailing_comments.is_empty() {
                    // `comments.wrap` 打开时各占一行（供 text_wrap_comment_blocks 合成一段）；
                    // 关闭时**必须紧邻拼接、不加分隔符**：`put_trailing_comments_back_in` 的
                    // 「注释不在行尾 → 移到行尾」分支靠 `(?<!\\)%` 负向断言排除相邻 token，
                    // 一旦中间插入空格/换行，它就会按倒序把注释逐个搬到行尾，顺序被整体翻转
                    // （pcc-program-review5-mod2）。
                    if comments_wrap {
                        trailing_comments.push('\n');
                    }
                }
                trailing_comments.push_str(&comment);
            }
            // 句内换行 → 捕获的空白或单空格
            let r = re(r"(?s)(?!\A)(\h*)\R");
            middle = replace_all_with(&middle, &r, |c| {
                let ws = gn(c, 1);
                if ws.is_empty() {
                    " ".to_string()
                } else {
                    ws.to_string()
                }
            });
        }
        if cfg_multi {
            let r = re(r"\h{2,}");
            middle = replace_all_with(&middle, &r, |_| " ".to_string());
        }
        let r = re(&format!(r"(?s){}\h*\R?", t.blanklines));
        middle = replace_all_with(&middle, &r, |_| format!("{}\n", t.blanklines));

        let id = state::storage(|s| {
            s.sentence_counter += 1;
            format!("{}{}{}", t.sentence, s.sentence_counter, t.end_of_token)
        });
        if sentence_work_after_indentation {
            // 句子缩进/折行延后到 mlb_one_sentence_per_line_indent
            state::storage(|s| {
                s.sentences.push(SentenceObj {
                    id: id.clone(),
                    value: format!("{middle}{end}"),
                    leading_horizontal_space: h_space.clone(),
                    follows: beginning.clone(),
                });
            });
        } else {
            state::storage(|s| {
                s.sentences.push(SentenceObj {
                    id: id.clone(),
                    value: format!("{middle}{end}"),
                    leading_horizontal_space: h_space.clone(),
                    follows: beginning.clone(),
                });
            });
        }
        format!(
            "{beginning}{h_space}{id}{trailing_comments}{tail}{}",
            if line_breaks_at_end { "" } else { "\n" }
        )
    });

    // ---- 后处理 ----
    let mut out = out;
    if !comments_wrap {
        let r = re(&format!(r"(?s)({tc})\h({tc})"));
        out = replace_all_with(&out, &r, |c| format!("{}{}", gn(c, 1), gn(c, 2)));
    }
    let r = re(&format!(
        r"(?s)(\S)\h*({}\d+{})",
        t.sentence, t.end_of_token
    ));
    out = replace_all_with(&out, &r, |c| format!("{}\n{}", gn(c, 1), gn(c, 2)));
    if comments_wrap {
        let r = re(&format!(
            r"(?s)({}\d+{})\h*({tc})",
            t.sentence, t.end_of_token
        ));
        out = replace_all_with(&out, &r, |c| format!("{}\n{}", gn(c, 1), gn(c, 2)));
    }

    if sentence_work_after_indentation {
        return out;
    }

    // ---- 回填句子 ----
    let cfg_columns = state::settings(|s| {
        as_i64(crate::settings::get_path(
            &s.root,
            &["modifyLineBreaks", "textWrapOptions", "columns"],
        ))
        .unwrap_or(0)
    });
    let sentences = state::storage(|s| s.sentences.clone());
    let mut body = out;
    for s in &sentences {
        let mut value = s.value.clone();
        if cfg_wrap_sentences && cfg_columns != 0 {
            let mut follows = s.follows.clone();
            let r = re(r"(?s).*?\R(\h*)$");
            follows = replace_all_with(&follows, &r, |c| gn(c, 1).to_string());
            value = crate::wrap::text_wrap_sentence(&value, &follows);
        }
        // 句子在文件末尾 / 开头 / 其余位置
        let r = re(&format!(r"(?s)\h*{}\h*$", regex_esc(&s.id)));
        if re(&format!(r"(?s)\h*{}\h*$", regex_esc(&s.id)))
            .is_match(body.as_bytes())
            .unwrap_or(false)
        {
            body = replace_all_with(&body, &r, |_| value.clone());
            continue;
        }
        let r = re(&format!(r"(?m)^{}\h*\R?", regex_esc(&s.id)));
        if re(&format!(r"(?m)^{}\h*\R?", regex_esc(&s.id)))
            .is_match(body.as_bytes())
            .unwrap_or(false)
        {
            body = replace_all_with(&body, &r, |_| format!("{value}\n"));
            continue;
        }
        let r = re(&format!(r"(?s)\R?\h*{}\h*\R?", regex_esc(&s.id)));
        body = replace_all_with(&body, &r, |_| format!("\n{value}\n"));
    }
    body
}

fn gs_local<'t>(caps: &'t pcre2::bytes::Captures, name: &str) -> &'t str {
    crate::patterns::gs(caps, name).unwrap_or("")
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

/// mlb_one_sentence_per_line_indent：句子延后缩进（sentenceIndent / wrap-after）。
pub fn mlb_one_sentence_per_line_indent(body: &str) -> String {
    let cfg_indent = osp(|m| as_str(get(m, "sentenceIndent")).unwrap_or("").to_string());
    let cfg_wrap_sentences = osp(|m| as_bool(get(m, "textWrapSentences")));
    let text_wrap_after = state::settings(|s| {
        as_str(crate::settings::get_path(
            &s.root,
            &["modifyLineBreaks", "textWrapOptions", "when"],
        ))
        .unwrap_or("before")
            == "after"
    });
    let guard = (cfg_wrap_sentences && text_wrap_after) || cfg_indent.contains([' ', '\t']);
    if !guard {
        return body.to_string();
    }
    let sentences = state::storage(|s| s.sentences.clone());
    let mut body = body.to_string();
    for s in &sentences {
        // 句子所在行的行首空白即 surroundingIndentation
        let re_id = re(&format!(r"(?m)^(\h*){}", regex_esc(&s.id)));
        let surrounding = match re_id.captures(body.as_bytes()).unwrap() {
            Some(c) => gn(&c, 1).to_string(),
            None => String::new(),
        };
        let mut value = s.value.clone();
        if cfg_wrap_sentences {
            value = crate::wrap::text_wrap_sentence(&value, &surrounding);
        }
        value = crate::blocks::find_all_code_blocks(&value, "");
        // 续行缩进
        if let Some(pos) = value.find('\n') {
            let first = value[..pos].trim_start().to_string();
            let rest = value[pos + 1..].to_string();
            let indent = format!("{cfg_indent}{surrounding}");
            let r = re(r"(?m)^");
            let rest = replace_all_with(&rest, &r, |_| indent.clone());
            value = format!("{first}\n{rest}");
        }
        let r = re(&format!(r"(?m)^(\h*){}", regex_esc(&s.id)));
        body = replace_all_with(&body, &r, |c| format!("{}{}", gn(c, 1), value));
    }
    body
}
