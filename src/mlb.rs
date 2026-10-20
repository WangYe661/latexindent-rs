//! -m 修改开关（对应 LatexIndent/ModifyLineBreaks.pm）：poly-switch 的
//! 换行增删与冲突 token 调整。
//!
//! poly-switch 语义：-1 删除换行；0 关闭；1 加换行；2 加注释再换行；
//! 3 加空行；4 = 先 -1 再 3（保证空行）。

use pcre2::bytes::Regex;

use crate::patterns::{gn, replace_all_with};
use crate::settings::as_bool;
use crate::state;
use crate::tokens::tok;

/// 参与 poly-switch 调整的对象切片。
pub struct MlbObj {
    pub begin: String,
    pub body: String,
    pub end: String,
    pub horizontal_trailing_space: String,
    pub trailing_comment: String,
    pub linebreaks_at_end_begin: bool,
    pub linebreaks_at_end_body: bool,
    pub linebreaks_at_end_end: String,
}

pub(crate) fn preserve_blank_lines() -> bool {
    state::settings(|s| {
        as_bool(crate::settings::get_path(
            &s.root,
            &["modifyLineBreaks", "preserveBlankLines"],
        ))
    })
}

fn re(src: &str) -> Regex {
    crate::patterns::re_new(src).expect("mlb regex")
}

fn poly_values(v: i64) -> Vec<i64> {
    if v == 4 {
        vec![-1, 3]
    } else {
        vec![v]
    }
}

/// _mlb_begin_starts_on_own_line（begin 前的换行处理）。
/// typ: "block" | "something-with-braces" | "argument-commands" | "argument-other"
pub fn begin_starts_on_own_line(obj: &mut MlbObj, value: i64, typ: &str) {
    let t = tok(|t| t.clone());
    let tc = state::patterns(|p| p.trailing_comment.clone());
    let guard = re(&format!(r"(?s)^\s*(?:{tc})\s*\R"));
    let guard2 = re(r"(?s)^\h*\R");
    for v in poly_values(value) {
        if v >= 1
            && !guard.is_match(obj.begin.as_bytes()).unwrap_or(false)
            && !guard2.is_match(obj.begin.as_bytes()).unwrap_or(false)
            && !obj.begin.is_empty()
        {
            match v {
                1 => {
                    if typ == "something-with-braces" {
                        let r = re(r"(?s)^\s*");
                        obj.begin = replace_all_with(&obj.begin, &r, |_| String::new());
                    }
                    if typ == "argument-commands" {
                        let between = state::settings(|s| {
                            crate::settings::as_str(crate::settings::get_path(
                                &s.root,
                                &["fineTuning", "arguments", "between"],
                            ))
                            .unwrap_or("")
                            .to_string()
                        });
                        let r = re(&format!(r"(?s)^(\h*(?:{})*)", between));
                        obj.begin = replace_all_with(&obj.begin, &r, |c| format!("{}\n", gn(c, 1)));
                    } else {
                        let r = re(r"^(\h*)");
                        obj.begin = replace_all_with(&obj.begin, &r, |c| {
                            format!("{}{}", gn(c, 1), t.m_before_begin_line_break_add)
                        });
                    }
                }
                2 => {
                    // 行首空白丢弃，前置注释占位 token（收尾时变为 %注释）
                    let r = re(r"(?s)^\h*");
                    obj.begin =
                        replace_all_with(&obj.begin, &r, |_| format!("{}\n", t.m_switch_comment));
                }
                3 => {
                    let r = re(r"^\h*");
                    obj.begin = replace_all_with(&obj.begin, &r, |_| String::new());
                    let blank = if preserve_blank_lines() {
                        t.blanklines.clone()
                    } else {
                        "\n".to_string()
                    };
                    obj.begin =
                        format!("{}{blank}\n{}", t.m_before_begin_line_break_add, obj.begin);
                    if value == 4 {
                        let r = re(&format!(
                            r"(?s)({})(\s*)({})",
                            t.m_before_begin_line_break_add, t.m_before_begin_line_break_remove
                        ));
                        obj.begin = replace_all_with(&obj.begin, &r, |c| {
                            format!("{}{}{}", gn(c, 3), gn(c, 2), gn(c, 1))
                        });
                    }
                }
                _ => {}
            }
        } else if v == -1 {
            let r = re(&format!(r"(?s)^(\h*)(?>\h*\R)*"));
            obj.begin = replace_all_with(&obj.begin, &r, |c| {
                format!("{}{}", gn(c, 1), t.m_before_begin_line_break_remove)
            });
        }
    }
}

/// _mlb_body_starts_on_own_line（begin 后 / body 前）。
pub fn body_starts_on_own_line(obj: &mut MlbObj, value: i64) {
    let t = tok(|t| t.clone());
    let tc = state::patterns(|p| p.trailing_comment.clone());
    obj.linebreaks_at_end_begin = re(r"^\h*\R").is_match(obj.body.as_bytes()).unwrap_or(false);
    for v in poly_values(value) {
        if v >= 1 && !obj.linebreaks_at_end_begin {
            match v {
                1 => {
                    obj.begin.push('\n');
                    obj.linebreaks_at_end_begin = true;
                    obj.body = replace_all_with(&obj.body, &re(r"^\h*"), |_| String::new());
                }
                2 => {
                    let has = re(&format!(r"(?s)^\h*(?:{tc})"))
                        .is_match(obj.body.as_bytes())
                        .unwrap_or(false);
                    if !has {
                        let cs = crate::trailing_comments::add_comment_symbol("");
                        while obj.begin.ends_with(' ') || obj.begin.ends_with('\t') {
                            obj.begin.pop();
                        }
                        obj.begin.push_str(&format!("%{cs}\n"));
                        obj.linebreaks_at_end_begin = true;
                        obj.body = replace_all_with(&obj.body, &re(r"^\h*"), |_| String::new());
                    }
                }
                3 => {
                    obj.body = replace_all_with(&obj.body, &re(r"^\h*"), |_| String::new());
                    let blank = if preserve_blank_lines() {
                        t.blanklines.clone()
                    } else {
                        "\n".to_string()
                    };
                    obj.body = format!("{blank}\n{}", obj.body);
                    obj.linebreaks_at_end_begin = true;
                }
                _ => {}
            }
        } else if v == -1 && obj.linebreaks_at_end_begin {
            let r = re(r"(?s)^(\h*)\R*\s*");
            obj.body = replace_all_with(&obj.body, &r, |c| gn(c, 1).to_string());
            obj.linebreaks_at_end_begin = false;
        }
    }
}

/// _mlb_end_starts_on_own_line（body 后 / end 前）。
pub fn end_starts_on_own_line(obj: &mut MlbObj, value: i64) {
    let t = tok(|t| t.clone());
    let tc = state::patterns(|p| p.trailing_comment.clone());
    for v in poly_values(value) {
        obj.linebreaks_at_end_body = re(r"\R\s*$").is_match(obj.body.as_bytes()).unwrap_or(false);
        if v >= 1 && !obj.linebreaks_at_end_body {
            let mut token = String::new();
            if v == 2 {
                let cs = crate::trailing_comments::add_comment_symbol("");
                token = format!("%{cs}");
                obj.body = replace_all_with(&obj.body, &re(r"\h*$"), |_| String::new());
            } else if v == 3 {
                token = format!(
                    "\n{}",
                    if preserve_blank_lines() {
                        t.blanklines.clone()
                    } else {
                        String::new()
                    }
                );
                obj.body = replace_all_with(&obj.body, &re(r"\h*$"), |_| String::new());
            }
            // 空 body 且 body 已独占一行时不重复换行
            let body_blank = re(r"(?s)^\h*$")
                .is_match(obj.body.as_bytes())
                .unwrap_or(false);
            if body_blank && obj.linebreaks_at_end_begin {
                obj.linebreaks_at_end_body = false;
            } else {
                obj.body = format!("{}{token}\n", obj.body);
                obj.linebreaks_at_end_body = true;
            }
        } else if v == -1 && obj.linebreaks_at_end_body {
            // end 前独占一行的注释搬到 end 之后
            let mv = re(&format!(r"(?s)\R\h*(?:{tc})\h*\R$"));
            if mv.is_match(obj.body.as_bytes()).unwrap_or(false) {
                let r = re(&format!(r"(?s)\R(\h*)({tc})\h*\R$"));
                obj.body = replace_all_with(&obj.body, &r, |c| {
                    let (h, cmt) = (gn(c, 1).to_string(), gn(c, 2).to_string());
                    obj.end = format!("{}{}{}", h, obj.end, cmt);
                    "\n".to_string()
                });
                return;
            }
            // Perl: m/$tokens{blanklines}$/s —— $ 匹配串尾换行之前，ends_with 判不出
            let r = re(&format!(r"(?s){}$", t.blanklines));
            if !r.is_match(obj.body.as_bytes()).unwrap_or(false) {
                let r = re(r"(?s)(\h*)\R\s*$");
                obj.body = replace_all_with(&obj.body, &r, |c| gn(c, 1).to_string());
                obj.linebreaks_at_end_body = false;
            }
        }
    }
}

/// _mlb_end_finishes_with_line_break（end 后）。
pub fn end_finishes_with_line_break(obj: &mut MlbObj, value: i64) {
    let t = tok(|t| t.clone());
    let v = value;
    if v == 0 {
        obj.end.push_str(&obj.horizontal_trailing_space.clone());
        obj.end.push_str(&obj.trailing_comment.clone());
        obj.end.push_str(&obj.linebreaks_at_end_end.clone());
        return;
    }
    let original = v;
    for x in poly_values(v) {
        if x == 3 && original == 4 {
            obj.linebreaks_at_end_end = String::new();
        }
        let ab = t.m_after_end_line_break.clone();
        let blank = t.blanklines.clone();
        let seg3 = if preserve_blank_lines() {
            format!("{ab}{blank}")
        } else {
            ab.clone()
        };
        if x >= 1 && !obj.end.is_empty() {
            match x {
                1 => {
                    if !obj.trailing_comment.is_empty() {
                        obj.end.push_str(&obj.horizontal_trailing_space.clone());
                        obj.end.push_str(&ab);
                        obj.end.push_str(&obj.trailing_comment.clone());
                        obj.end.push_str(&obj.linebreaks_at_end_end.clone());
                    } else if !obj.linebreaks_at_end_end.is_empty() {
                        obj.end.push_str(&obj.horizontal_trailing_space.clone());
                        obj.end.push_str(&obj.linebreaks_at_end_end.clone());
                    } else {
                        obj.end.push_str(&obj.horizontal_trailing_space.clone());
                        obj.end.push_str(&ab);
                    }
                }
                2 => {
                    if !obj.trailing_comment.is_empty() {
                        obj.end.push_str(&obj.horizontal_trailing_space.clone());
                        obj.end.push_str(&obj.trailing_comment.clone());
                        obj.end.push_str(&obj.linebreaks_at_end_end.clone());
                    } else if !obj.linebreaks_at_end_end.is_empty() {
                        obj.end.push_str(&obj.linebreaks_at_end_end.clone());
                    } else {
                        while obj.end.ends_with(' ') || obj.end.ends_with('\t') {
                            obj.end.pop();
                        }
                        obj.end.push_str(&t.m_switch_comment);
                        obj.end.push_str(&ab);
                    }
                }
                3 => {
                    if !obj.trailing_comment.is_empty() {
                        obj.end.push_str(&obj.horizontal_trailing_space.clone());
                        obj.end.push_str(&seg3);
                        obj.end.push_str(&seg3);
                        obj.end.push_str(&ab);
                        obj.end.push_str(&obj.trailing_comment.clone());
                        obj.end.push_str(&obj.linebreaks_at_end_end.clone());
                    } else if !obj.linebreaks_at_end_end.is_empty() {
                        obj.end.push_str(&obj.horizontal_trailing_space.clone());
                        obj.end.push_str(&obj.linebreaks_at_end_end.clone());
                    } else {
                        obj.end.push_str(&obj.horizontal_trailing_space.clone());
                        obj.end.push_str(&seg3);
                        obj.end.push_str(&seg3);
                        obj.end.push_str(&ab);
                    }
                }
                _ => {}
            }
        } else if x >= 1 && !obj.linebreaks_at_end_end.is_empty() {
            obj.end.push_str(&obj.horizontal_trailing_space.clone());
            obj.end.push_str(&obj.trailing_comment.clone());
            obj.end.push_str(&obj.linebreaks_at_end_end.clone());
        } else if x == -1 {
            if !obj.trailing_comment.is_empty() {
                obj.end.push_str(&obj.horizontal_trailing_space.clone());
                obj.end.push_str(&obj.trailing_comment.clone());
                obj.end.push_str(&ab);
                obj.end.push_str(&obj.linebreaks_at_end_end.clone());
            } else {
                obj.end.push_str(&obj.horizontal_trailing_space.clone());
                if !obj.linebreaks_at_end_end.is_empty() {
                    obj.end.push_str(&t.m_after_end_remove);
                }
            }
        }
    }
}

/// _mlb_line_break_token_adjust：冲突 token → 真实换行（顺序敏感）。
pub fn line_break_token_adjust(body: &str) -> String {
    let t = tok(|t| t.clone());
    let tc = crate::alignment::trailing_comment_pattern();
    let add = t.m_before_begin_line_break_add.clone();
    let rm = t.m_before_begin_line_break_remove.clone();
    let ab = t.m_after_end_line_break.clone();
    let mut b = body.to_string();
    // ① ADD 周围已有换行归一为单个 \n
    let r = re(&format!(r"(?s)\R\h*{add}"));
    b = replace_all_with(&b, &r, |_| "\n".to_string());
    let r = re(&format!(r"(?s){add}\R"));
    b = replace_all_with(&b, &r, |_| "\n".to_string());
    let r = re(&format!(r"(?s){add}"));
    b = replace_all_with(&b, &r, |_| "\n".to_string());
    // ② REMOVE 优先：吞掉左侧的 mAfterEndLineBreak
    let r = re(&format!(r"(?s)(?:{ab})+\s*{rm}"));
    b = replace_all_with(&b, &r, |_| String::new());
    let r = re(&format!(r"(?s){ab}({tc})\s*{rm}"));
    b = replace_all_with(&b, &r, |c| format!("\n{}", gn(c, 1)));
    // ③ REMOVE 左侧换行吞掉
    let r = re(&format!(r"(?s)(?!\A)(\h*)\R\s*{rm}"));
    b = replace_all_with(&b, &r, |c| gn(c, 1).to_string());
    let r = re(&format!(r"(?s)(\h*)\R\s*{rm}"));
    b = replace_all_with(&b, &r, |c| gn(c, 1).to_string());
    // ④⑤⑥
    let r = re(&format!(r"(?s){ab}({tc})"));
    b = replace_all_with(&b, &r, |c| format!("\n{}\n", gn(c, 1)));
    let r = re(&format!(r"(?s)(?:{ab}){{3}}\s*"));
    b = replace_all_with(&b, &r, |_| "\n\n".to_string());
    // 换行 token 之后的行首水平空白（原同行的后续文本）一并剥离
    let r = re(&format!(r"(?s){ab}\R?\h*"));
    b = replace_all_with(&b, &r, |_| "\n".to_string());
    b
}

/// _mlb_after_indentation_token_adjust：缩进后收尾（顺序敏感）。
pub fn after_indentation_token_adjust(body: &str) -> String {
    let t = tok(|t| t.clone());
    let tc = crate::alignment::trailing_comment_pattern();
    let rm = t.m_before_begin_line_break_remove.clone();
    let aer = t.m_after_end_remove.clone();
    let msc = t.m_switch_comment.clone();
    let msf = t.m_switch_comment_fi.clone();
    let bl = t.blanklines.clone();
    let mut b = body.to_string();
    // —— 换行移除类 ——
    let r = re(&format!(r"(?s)(\S){aer}(\S)"));
    b = replace_all_with(&b, &r, |c| format!("{}{}", gn(c, 1), gn(c, 2)));
    let r = re(&format!(r"(?s)(\}}|\]){aer}{msc}"));
    b = replace_all_with(&b, &r, |c| format!("{}%", gn(c, 1)));
    let r = re(&format!(r"(?s)(\}}|\]){aer}(\h*{tc})"));
    b = replace_all_with(&b, &r, |c| format!("{}{}", gn(c, 1), gn(c, 2)));
    let r = re(&format!(r"(?s)(.)(\h*){aer}"));
    b = replace_all_with(&b, &r, |c| format!("{}{}", gn(c, 1), gn(c, 2)));
    let r = re(&format!(r"(?s)(\h*)\R\s*{rm}"));
    b = replace_all_with(&b, &r, |c| gn(c, 1).to_string());
    let r = re(&format!(r"(?s){rm}"));
    b = replace_all_with(&b, &r, |_| String::new());
    // 尾注释类
    let r = re(&format!(r"(?s)\h{msf}"));
    b = replace_all_with(&b, &r, |_| msc.clone());
    let r = re(&format!(r"(?s){msc}(\s*{bl})"));
    b = replace_all_with(&b, &r, |c| gn(c, 1).to_string());
    let r = re(&format!(r"(?s)({tc})\R\h*{msc}"));
    b = replace_all_with(&b, &r, |c| gn(c, 1).to_string());
    let r = re(&format!(r"(?s){msc}\s*{msc}\R"));
    b = replace_all_with(&b, &r, |_| {
        format!("%{}\n", crate::trailing_comments::add_comment_symbol(""))
    });
    let r = re(&format!(r"(?s)\A{msc}"));
    b = replace_all_with(&b, &r, |_| String::new());
    let r = re(&format!(r"(?s){msc}\Z"));
    b = replace_all_with(&b, &r, |_| String::new());
    let r = re(&format!(r"(?s)(\R\h*\R){msc}\R"));
    b = replace_all_with(&b, &r, |c| gn(c, 1).to_string());
    let r = re(&format!(r"(?s){msc}"));
    b = replace_all_with(&b, &r, |_| {
        format!("%{}", crate::trailing_comments::add_comment_symbol(""))
    });
    b
}

/// _mlb_commas_in_arg_body：参数体内逗号的 poly-switch（CommaStartsOnOwnLine /
/// CommaFinishesWithLineBreak）。
pub fn commas_in_arg_body(body: &str, v_comma_start: i64, v_comma_finish: i64) -> String {
    let t = tok(|t| t.clone());
    let tc = state::patterns(|p| p.trailing_comment.clone());
    let simple = r"(?>\\[{}\[\]]|[^{}\[\]])";
    let src = format!(
        r"(?s)(?<WS>\s*)(?:(?<COMMA>,(?<CT>\h*)?(?<CC>{tc})?(?<CL>\R\s*)?)|(?<ARG>(?:(?:\s|{tc}|{bl}|{between})*(?<!\\)[\{{\[](?:(?:{simple}))*(?<!\\)[\}}\]])))",
        tc = tc,
        bl = t.blanklines,
        between = state::settings(|s| {
            crate::settings::as_str(crate::settings::get_path(
                &s.root,
                &["fineTuning", "arguments", "between"],
            ))
            .unwrap_or("")
            .to_string()
        }),
        simple = simple
    );
    let re = re(&src);
    replace_all_with(body, &re, |caps| {
        let ws = gn(caps, 1);
        if caps.name("COMMA").is_some() {
            let mut m = MlbObj {
                begin: format!("{ws},"),
                body: format!(
                    "{}{}{}",
                    gs_named(caps, "CT"),
                    gs_named(caps, "CC"),
                    gs_named(caps, "CL")
                ),
                end: String::new(),
                horizontal_trailing_space: String::new(),
                trailing_comment: String::new(),
                linebreaks_at_end_begin: false,
                linebreaks_at_end_body: false,
                linebreaks_at_end_end: String::new(),
            };
            begin_starts_on_own_line(&mut m, v_comma_start, "block");
            body_starts_on_own_line(&mut m, v_comma_finish);
            format!("{}{}{}", m.begin, m.body, m.end)
        } else {
            format!("{ws}{}", gs_named(caps, "ARG"))
        }
    })
}

fn gs_named<'t>(caps: &'t pcre2::bytes::Captures, name: &str) -> &'t str {
    crate::patterns::gs(caps, name).unwrap_or("")
}

/// mlb_PRE_indent_sentence_and_text_wrap：句/折行的“缩进前”阶段。
pub fn pre_indent_sentence_and_text_wrap(body: &str) -> String {
    let cfg = mlb_text_cfg();
    let mut body = body.to_string();
    if cfg.manipulate_sentences {
        body = crate::sentence::one_sentence_per_line(&body);
    } else if !cfg.text_wrap_sentences && cfg.columns != 0 && !cfg.when_after {
        body = crate::wrap::text_wrap(&body);
        body = crate::verbatim::mlb_verbatim(&body, "afterTextWrap");
    }
    if cfg.comments_wrap && !cfg.when_after {
        body = crate::wrap::text_wrap_comment_blocks(&body);
    }
    body
}

/// mlb_POST_indent_sentence_and_text_wrap：句/折行的“缩进后”阶段。
pub fn post_indent_sentence_and_text_wrap(body: &str) -> String {
    let cfg = mlb_text_cfg();
    let mut body = body.to_string();
    if !cfg.manipulate_sentences && !cfg.text_wrap_sentences && cfg.columns != 0 && cfg.when_after {
        body = crate::wrap::text_wrap(&body);
    }
    if cfg.comments_wrap && cfg.when_after {
        body = crate::wrap::text_wrap_comment_blocks(&body);
    }
    body
}

struct MlbTextCfg {
    manipulate_sentences: bool,
    text_wrap_sentences: bool,
    columns: i64,
    when_after: bool,
    comments_wrap: bool,
}

fn mlb_text_cfg() -> MlbTextCfg {
    state::settings(|s| {
        use crate::settings::{as_bool, as_i64, as_str, get_path};
        let osp = |k: &str| -> bool {
            as_bool(get_path(
                &s.root,
                &["modifyLineBreaks", "oneSentencePerLine", k],
            ))
        };
        MlbTextCfg {
            manipulate_sentences: osp("manipulateSentences"),
            text_wrap_sentences: osp("textWrapSentences"),
            columns: as_i64(get_path(
                &s.root,
                &["modifyLineBreaks", "textWrapOptions", "columns"],
            ))
            .unwrap_or(0),
            when_after: as_str(get_path(
                &s.root,
                &["modifyLineBreaks", "textWrapOptions", "when"],
            ))
            .unwrap_or("before")
                == "after",
            comments_wrap: as_bool(get_path(
                &s.root,
                &["modifyLineBreaks", "textWrapOptions", "comments", "wrap"],
            )),
        }
    })
}

/// _mlb_file_starts_with_line_break：before 记录文件起始是否有换行；after 按需去除。
pub fn file_starts_with_line_break(body: &str, when: &str) -> (bool, String) {
    if when == "before" {
        let starts = re(r"(?s)\A\h*\R")
            .is_match(body.as_bytes())
            .unwrap_or(false);
        return (starts, body.to_string());
    }
    // after：无条件去掉开头换行（调用方按 before 记录的标志决定是否采用）
    let b = replace_all_with(body, &re(r"(?s)\A\h*\R+"), |_| String::new());
    (false, b)
}
