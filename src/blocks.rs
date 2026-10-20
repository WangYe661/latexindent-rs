//! 代码块识别与缩进（对应 LatexIndent/Blocks.pm 的 _find_all_code_blocks 家族）。

use pcre2::bytes::{Captures, Regex};

use crate::logger;
use crate::patterns::{g, gs, replace_all_with};
use crate::settings::ObjectSettings;
use crate::state;
use crate::switches;
use crate::tokens::tok;

fn indent_lines(body: &str, indent: &str) -> String {
    if body.is_empty() || indent.is_empty() {
        return body.to_string();
    }
    // Perl 的 s/^/indent/mg：^ 不匹配串尾（结尾换行后不再附加缩进）
    let mut out = String::with_capacity(body.len() + indent.len() * 8);
    // 首行只有水平空白（随后即换行）时，行首缩进会落在换行之前；LF 下它会被
    // removeTrailingWhitespace 清掉，但 CRLF 下 `\h+$` 跨不过 `\r`，
    // 于是每次格式化都会再叠一层（Windows/VS Code 光标漂移的根因）。
    let first_line_blank = body
        .trim_start_matches([' ', '\t'])
        .starts_with(['\r', '\n']);
    if !first_line_blank {
        out.push_str(indent);
    }
    let mut chars = body.chars().peekable();
    while let Some(c) = chars.next() {
        out.push(c);
        if c == '\n' && chars.peek().is_some() {
            out.push_str(indent);
        }
    }
    out
}

fn strip_first_indent(body: &str, indent: &str) -> String {
    if indent.is_empty() {
        return body.to_string();
    }
    match body.strip_prefix(indent) {
        Some(rest) => rest.to_string(),
        None => body.to_string(),
    }
}

fn looks_for_code(body: &str) -> bool {
    state::patterns(|p| {
        p.code_block_look_for
            .is_match(body.as_bytes())
            .unwrap_or(false)
    })
}

/// 主入口：识别并缩进 body 中的全部代码块（递归）。
pub fn find_all_code_blocks(body: &str, current_indent: &str) -> String {
    if body.is_empty() {
        return String::new();
    }
    let is_m = switches::with(|s| s.is_m_switch_active());
    // 精确版（Perl 同款「参数体优先 + 普通星号」）先跑：它能靠回溯让嵌套块
    // （`\if...\fi`、special、环境）不越过参数的闭括号，语义与 Perl 一致。
    // 病态输入会指数回溯撞 pcre2 match limit → 整段重跑「所有格」兜底版
    // （永不回溯，块边界在极端嵌套下可能与 Perl 略有出入，但绝不 panic）。
    let accurate = state::patterns(|p| {
        crate::patterns::try_replace_all_with(body, &p.all_code_blocks, |caps| {
            render_block(caps, current_indent, is_m)
        })
    });
    let out = match accurate {
        Ok(o) => o,
        Err(_) => state::patterns(|p| {
            replace_all_with(body, &p.all_code_blocks_fast, |caps| {
                render_block(caps, current_indent, is_m)
            })
        }),
    };
    if is_m {
        crate::mlb::line_break_token_adjust(&out)
    } else {
        out
    }
}

fn render_block(caps: &Captures, current_indent: &str, is_m: bool) -> String {
    let leading = gs(caps, "LEADINGSPACE").unwrap_or("");
    if g(caps, "ENVBEGIN").is_some() {
        render_environment(caps, leading, current_indent, is_m)
    } else if g(caps, "IFELSEFIBEGIN").is_some() {
        render_ifelsefi(caps, leading, current_indent, is_m)
    } else if g(caps, "SPECIALBEGIN").is_some() {
        render_special(caps, leading, current_indent, is_m)
    } else {
        render_thing_with_args(caps, leading, current_indent, is_m)
    }
}

fn settings_for(name: &str, yaml_name: &str, has_args: bool, is_m: bool) -> ObjectSettings {
    state::settings_mut(|s| s.settings_for_object(name, yaml_name, has_args, is_m))
}

fn indent_after_items(name: &str) -> bool {
    state::settings(|s| {
        crate::settings::as_bool(crate::settings::get_path(
            &s.root,
            &["indentAfterItems", name],
        ))
    })
}

fn render_environment(caps: &Captures, leading: &str, current_indent: &str, is_m: bool) -> String {
    let t = tok(|t| t.clone());
    let env_begin = gs(caps, "ENVBEGIN").unwrap_or("");
    let name = gs(caps, "ENVNAME").unwrap_or_default();
    let mut begin = format!("{leading}{env_begin}");
    let mut body = gs(caps, "ENVBODY").unwrap_or("").to_string();
    let end = gs(caps, "ENVEND").unwrap_or("").to_string();
    let mut end = end;
    let mut arg_body = gs(caps, "ENVARGS").unwrap_or("").to_string();
    logger::trace(&format!("*found: {name} \t type: environments"));

    let os = settings_for(name, "environments", true, is_m);

    // 参数处理
    let mut had_args = false;
    if !arg_body.is_empty() {
        logger::trace(&format!("*found: arguments for {name} environment"));
        logger::trace(&arg_body);
        if state::patterns(|p| {
            p.re_arg_only_comment
                .is_match(arg_body.as_bytes())
                .unwrap_or(false)
        }) {
            logger::trace(&format!(
                "arguments for {name} environment is ONLY a comment, pre-pending it to body"
            ));
            body = arg_body + &body;
            arg_body = String::new();
        } else if state::patterns(|p| {
            p.re_arg_only_space
                .is_match(arg_body.as_bytes())
                .unwrap_or(false)
        }) {
            logger::trace(&format!(
                "arguments for {name} environment is ONLY space, pre-pending it to body"
            ));
            let h = leading_hspace(&arg_body);
            let lb = rest_linebreak(&arg_body);
            begin.push_str(&h);
            body = lb + &body;
            arg_body = String::new();
        } else {
            had_args = true;
            arg_body = indent_all_args(name, "environments", &arg_body, current_indent);
        }
    } else {
        logger::trace(&format!("no arguments for {name} environment"));
    }

    // 嵌套
    if indent_after_items(name) {
        logger::trace(&format!("looking for environments ITEM for {name}"));
        body = find_env_items(&body, name, current_indent);
    } else if looks_for_code(&body) {
        body = find_all_code_blocks(&body, current_indent);
    }

    let mut linebreaks_at_end_begin = body_starts_linebreak(&body);
    let mut added = os.indentation.clone();

    // -m poly-switch 调整（对应 Blocks.pm 环境分支的 m 段）
    // Perl 语义：参数整体并入 begin，poly-switch 在「参数 }」与 body 的边界上操作，
    // 收尾把 begin 中 \begin{...} 之后的参数行按 added 缩进（add indentation to BEGIN statement）。
    let mut arg_text = String::new();
    if is_m {
        if !arg_body.is_empty() {
            // 参数尾随空白挪到 body
            let trimmed = arg_body.trim_end().to_string();
            let ws = arg_body[trimmed.len()..].to_string();
            arg_body = trimmed;
            body = ws + &body;
            begin.push_str(&arg_body);
            let t2 = tok(|t| t.clone());
            if arg_body.ends_with(&t2.m_after_end_remove) {
                linebreaks_at_end_begin = false;
                let r = crate::patterns::re_new(r"(?s)^\s*").unwrap();
                body = crate::patterns::replace_all_with(&body, &r, |_| String::new());
            }
            arg_text = arg_body.clone();
            arg_body = String::new();
        }
        let mut m = crate::mlb::MlbObj {
            begin,
            body,
            end,
            horizontal_trailing_space: gs(caps, "TRAILINGHSPACE").unwrap_or("").to_string(),
            trailing_comment: gs(caps, "TRAILINGCOMMENT").unwrap_or("").to_string(),
            linebreaks_at_end_begin,
            linebreaks_at_end_body: false,
            linebreaks_at_end_end: gs(caps, "TRAILINGLINEBREAK").unwrap_or("").to_string(),
        };
        let poly = |k: &str| *os.poly.get(k).unwrap_or(&0);
        crate::mlb::begin_starts_on_own_line(&mut m, poly("BeginStartsOnOwnLine"), "block");
        crate::mlb::body_starts_on_own_line(&mut m, poly("BodyStartsOnOwnLine"));
        crate::mlb::end_starts_on_own_line(&mut m, poly("EndStartsOnOwnLine"));
        crate::mlb::end_finishes_with_line_break(&mut m, poly("EndFinishesWithLineBreak"));
        begin = m.begin;
        body = m.body;
        end = m.end;
        linebreaks_at_end_begin = m.linebreaks_at_end_begin;
        // add indentation to BEGIN statement：\begin{...} 之后的参数行按 added 缩进
        if !arg_text.is_empty() && !added.is_empty() {
            let re_split = crate::patterns::re_new(r"(?s)^(.*?\\begin\{[^}]+\})").unwrap();
            if let Some(caps2) = re_split.captures(begin.as_bytes()).unwrap_or(None) {
                let prefix = crate::patterns::gn(&caps2, 1).to_string();
                let mut rest = begin[prefix.len()..].to_string();
                rest = indent_lines(&rest, &added);
                rest = strip_first_indent(&rest, &added);
                begin = prefix + &rest;
            }
        }
    }

    // align at ampersand（lookForAlignDelims 时列对齐，可能带回缩进补偿）
    if os.look_for_align_delims {
        let mut arguments_present = false;
        if !arg_body.is_empty() && !body.starts_with('\n') {
            body = arg_body + &body;
            arg_body = String::new();
            arguments_present = true;
        }
        // Perl env 分支：begin 的迁移/调整结果只在 beginAdjusted 时才采纳
        // （否则沿用原 begin，被移走的前导空白即丢弃——issue-599）
        let mut aligned_begin = begin.clone();
        let res = crate::alignment::align_at_ampersand(&mut aligned_begin, &mut body, &os);
        if res.begin_adjusted {
            begin = aligned_begin;
        }
        if let Some(ind) = res.indentation {
            added = ind;
        }
        if arguments_present {
            linebreaks_at_end_begin = body_starts_linebreak(&body);
        }
    }

    // Perl：body 首行缩进剥离条件 = (!$linebreaksAtEndBegin or $argBody)
    let strip_first = !linebreaks_at_end_begin || !arg_body.is_empty();
    let mut body = arg_body + &body;
    if !added.is_empty() {
        if !state::patterns(|p| {
            p.re_arg_only_space
                .is_match(body.as_bytes())
                .unwrap_or(false)
        }) {
            body = indent_lines(&body, &added);
        }
        if strip_first {
            body = strip_first_indent(&body, &added);
        }
    }
    let _ = t;
    begin + &body + &end
}

fn render_ifelsefi(caps: &Captures, leading: &str, current_indent: &str, is_m: bool) -> String {
    let t = tok(|t| t.clone());
    let if_begin = gs(caps, "IFELSEFIBEGIN").unwrap_or("");
    let name = gs(caps, "IFELSEFINAME").unwrap_or_default().to_string();
    let mut begin = format!("{leading}{if_begin}");
    let mut body = gs(caps, "IFELSEFIBODY").unwrap_or("").to_string();
    let end = gs(caps, "IFELSEFIEND").unwrap_or("").to_string();
    let mut end = end;
    logger::trace(&format!("*found: {name} \t type: ifElseFi"));

    let os = settings_for(&name, "ifElseFi", false, is_m);

    if looks_for_code(&body) {
        body = find_all_code_blocks(&body, current_indent);
    }
    let (mut body, else_exists) = find_ifelsefi_else_or(&body, &name);

    // -m poly-switch 调整（ifelsefi 分支）
    let mut linebreaks_at_end_begin = body_starts_linebreak(&body);
    if is_m {
        let t2 = tok(|t| t.clone());
        for (src, rep) in [
            (
                format!(
                    r"(?s)\R\h*{}({})?",
                    t2.m_before_begin_line_break_add, t2.blanklines
                ),
                "\n".to_string(),
            ),
            (
                format!(r"(?s){}", t2.m_before_begin_line_break_add),
                "\n".to_string(),
            ),
            (
                format!(r"(?s)([a-zA-Z0-9]){}([^\\])", t2.m_else_trailing_space),
                "$1 $2".to_string(),
            ),
            (format!(r"(?s){}", t2.m_else_trailing_space), String::new()),
        ] {
            let r = crate::patterns::re_new(&src).unwrap();
            body = if rep.contains("$1") {
                crate::patterns::replace_all_with(&body, &r, |c| {
                    format!(
                        "{} {}",
                        crate::patterns::gn(c, 1),
                        crate::patterns::gn(c, 2)
                    )
                })
            } else {
                crate::patterns::replace_all_with(&body, &r, |_| rep.clone())
            };
        }
        let mut m = crate::mlb::MlbObj {
            begin,
            body,
            end,
            horizontal_trailing_space: gs(caps, "TRAILINGHSPACE").unwrap_or("").to_string(),
            trailing_comment: gs(caps, "TRAILINGCOMMENT").unwrap_or("").to_string(),
            linebreaks_at_end_begin,
            linebreaks_at_end_body: false,
            linebreaks_at_end_end: gs(caps, "TRAILINGLINEBREAK").unwrap_or("").to_string(),
        };
        let poly = |k: &str| *os.poly.get(k).unwrap_or(&0);
        crate::mlb::begin_starts_on_own_line(&mut m, poly("BeginStartsOnOwnLine"), "block");
        crate::mlb::body_starts_on_own_line(&mut m, poly("BodyStartsOnOwnLine"));
        crate::mlb::end_starts_on_own_line(&mut m, poly("EndStartsOnOwnLine"));
        // \fi / \else 后需补一个空格的边界情形（对应 Perl Blocks.pm 的 space 规则）
        // Perl: body 以 (specialMiddle)?RM-token\ 开头时不补（token 感知）
        let body_joins_cmd = crate::patterns::re_new(&format!(
            r"(?s)^({})?{}\\",
            t2.special_begin_end_middle, t2.m_before_begin_line_break_remove
        ))
        .map(|r| r.is_match(m.body.as_bytes()).unwrap_or(false))
        .unwrap_or(false);
        if poly("BodyStartsOnOwnLine") == -1
            && !m.body.starts_with([' ', '\t'])
            && m.body.chars().next().map(|c| c != '\\').unwrap_or(false)
            && !body_joins_cmd
        {
            m.begin.push(' ');
        }
        if poly("EndFinishesWithLineBreak") == -1 && m.horizontal_trailing_space.is_empty() {
            m.horizontal_trailing_space = " ".to_string();
        }
        crate::mlb::end_finishes_with_line_break(&mut m, poly("EndFinishesWithLineBreak"));
        if poly("EndFinishesWithLineBreak") == 2
            && m.end.starts_with(&format!("\\fi{}", t2.m_switch_comment))
        {
            m.end = m.end.replacen(
                &format!("\\fi{}", t2.m_switch_comment),
                &format!("\\fi {}", t2.m_switch_comment_fi),
                1,
            );
        }
        begin = m.begin;
        body = m.body;
        end = m.end;
        linebreaks_at_end_begin = m.linebreaks_at_end_begin;
    }

    let added = os.indentation.clone();
    if !added.is_empty() || else_exists {
        if is_m {
            let t2 = tok(|t| t.clone());
            let r = crate::patterns::re_new(&format!(
                r"(?m)^(\\h*){}\s*({})",
                t2.m_switch_comment, t2.special_begin_end_middle
            ))
            .unwrap();
            body = crate::patterns::replace_all_with(&body, &r, |c| {
                format!("{}{}", crate::patterns::gn(c, 1), crate::patterns::gn(c, 2))
            });
        }
        body = indent_lines(&body, &added);
        if !linebreaks_at_end_begin {
            body = strip_first_indent(&body, &added);
        }
        if else_exists {
            // \else 行与 \if 对齐：去掉缩进+middle token，再清掉 token
            let combo = format!("{added}{}", t.special_begin_end_middle);
            body = body.replace(&combo, "");
            body = body.replace(&t.special_begin_end_middle, "");
        }
    }
    begin + &body + &end
}

fn render_special(caps: &Captures, leading: &str, current_indent: &str, is_m: bool) -> String {
    let t = tok(|t| t.clone());
    let sp_begin = gs(caps, "SPECIALBEGIN")
        .or_else(|| gs(caps, "SPECIALBEGIN_spn"))
        .unwrap_or("");
    let body_txt = gs(caps, "SPECIALBODY")
        .or_else(|| gs(caps, "SPECIALBODY_spn"))
        .unwrap_or("");
    let sp_end = gs(caps, "SPECIALEND")
        .or_else(|| gs(caps, "SPECIALEND_spn"))
        .unwrap_or("");
    let mut begin = format!("{leading}{sp_begin}");
    let mut body = body_txt.to_string();
    let end = sp_end.to_string();
    let mut end = end;

    // specialLookUpName：按 begin 匹配确定 name
    let name = lookup_special_name(sp_begin);
    logger::trace(&format!("*found: {name} \t type: specialBeginEnd"));

    let os = settings_for(&name, "specialBeginEnd", false, is_m);

    let has_middle = state::settings(|s| s.special_lookup_middle.contains_key(&name));
    // Perl quirk：非 -m 时 align_at_ampersand 处理的是对象构造时的【内层渲染前】body，
    // 其结果整体替换渲染后的 body（special 分支仅 -m 时同步对象 body，env 分支无条件同步）
    let stale_body = body.clone();
    if looks_for_code(&body) {
        body = find_all_code_blocks(&body, current_indent);
    }
    if has_middle {
        body = find_special_middle(&body, &name);
    }

    // -m poly-switch 调整（special 分支；Perl 顺序：poly 在 align 之前，align 的
    // begin 计量判定依赖 poly 之后的 body 换行状态）
    let mut poly_linebreaks_at_end_begin = false;
    if is_m {
        let t2 = tok(|t| t.clone());
        body = crate::patterns::replace_all_with(
            &body,
            &crate::patterns::re_new(&format!(
                r"(?s)\R\h*{}(?:{})?",
                t2.m_before_begin_line_break_add, t2.blanklines
            ))
            .unwrap(),
            |_| "\n".to_string(),
        );
        body = crate::patterns::replace_all_with(
            &body,
            &crate::patterns::re_new(&format!(r"(?s){}", t2.m_before_begin_line_break_add))
                .unwrap(),
            |_| "\n".to_string(),
        );
        let mut m = crate::mlb::MlbObj {
            begin,
            body,
            end,
            horizontal_trailing_space: gs(caps, "TRAILINGHSPACE").unwrap_or("").to_string(),
            trailing_comment: gs(caps, "TRAILINGCOMMENT").unwrap_or("").to_string(),
            linebreaks_at_end_begin: false,
            linebreaks_at_end_body: false,
            linebreaks_at_end_end: gs(caps, "TRAILINGLINEBREAK").unwrap_or("").to_string(),
        };
        let poly = |k: &str| *os.poly.get(k).unwrap_or(&0);
        crate::mlb::begin_starts_on_own_line(&mut m, poly("BeginStartsOnOwnLine"), "block");
        crate::mlb::body_starts_on_own_line(&mut m, poly("BodyStartsOnOwnLine"));
        crate::mlb::end_starts_on_own_line(&mut m, poly("EndStartsOnOwnLine"));
        crate::mlb::end_finishes_with_line_break(&mut m, poly("EndFinishesWithLineBreak"));
        begin = m.begin;
        body = m.body;
        end = m.end;
        poly_linebreaks_at_end_begin = m.linebreaks_at_end_begin;
    }

    // Perl special 分支：align 的 begin 计量补偿覆盖块缩进（$addedIndentation）；
    // 非 -m 时对齐的是渲染前的陈旧 body，结果整体替换渲染后的 body（Perl quirk）
    let mut align_indentation: Option<String> = None;
    // Perl：非 -m 的 linebreaksAtEndBegin 在 align 覆盖 body 之前按渲染后 body 计算
    let non_m_linebreaks_at_end_begin = body_starts_linebreak(&body);
    if os.look_for_align_delims {
        let mut align_body = if is_m {
            body.clone()
        } else {
            stale_body.clone()
        };
        let res = crate::alignment::align_at_ampersand(&mut begin, &mut align_body, &os);
        align_indentation = res.indentation;
        body = align_body;
    }

    let added = align_indentation.unwrap_or_else(|| os.indentation.clone());
    if !added.is_empty() {
        // Perl：m 模式取 poly 对象的 linebreaksAtEnd{begin}（begin 尾换行由 poly 决定）
        let linebreaks_at_end_begin = if is_m {
            poly_linebreaks_at_end_begin
        } else {
            non_m_linebreaks_at_end_begin
        };
        // Perl Blocks.pm:1017-1019：
        //   s"^(\h)*mSwitchComment\s*(specialBeginEndMiddle)"($1?$1:q()).$2;"emg
        // 把 middle token 提到 mSwitchComment 之前（Perl 的 (\h)* 只留最后一次重复）
        if is_m && has_middle {
            let mid_tok = t.special_begin_end_middle.clone();
            let msc = t.m_switch_comment.clone();
            let r = crate::patterns::re_new(&format!(r"(?m)^(\h)*{msc}\s*({mid_tok})")).unwrap();
            body = crate::patterns::replace_all_with(&body, &r, |c| {
                format!("{}{}", crate::patterns::gn(c, 1), crate::patterns::gn(c, 2))
            });
        }
        body = indent_lines(&body, &added);
        if !linebreaks_at_end_begin {
            body = strip_first_indent(&body, &added);
        }
        if has_middle {
            let combo = format!("{added}{}", t.special_begin_end_middle);
            body = body.replace(&combo, "");
            body = body.replace(&t.special_begin_end_middle, "");
        }
    }
    begin + &body + &end
}

fn lookup_special_name(begin: &str) -> String {
    state::settings(|s| {
        if let Some(v) = s.special_lookup_middle.get("__names__") {
            let _ = v;
        }
        if let Some(serde_yaml::Value::Sequence(seq)) =
            crate::settings::get(&s.root, "specialBeginEnd")
        {
            for item in seq {
                if let Some(m) = item.as_mapping() {
                    let b = crate::settings::as_str(crate::settings::get(m, "begin"));
                    if let Some(b) = b {
                        let re = crate::patterns::re_new(&format!("(?s){b}"));
                        if let Ok(re) = re {
                            if re.is_match(begin.as_bytes()).unwrap_or(false) {
                                return crate::settings::as_str(crate::settings::get(m, "name"))
                                    .unwrap_or("unknown special")
                                    .to_string();
                            }
                        }
                    }
                }
            }
        }
        "unknown special".to_string()
    })
}

fn render_thing_with_args(
    caps: &Captures,
    leading: &str,
    current_indent: &str,
    is_m: bool,
) -> String {
    let t = tok(|t| t.clone());
    let arg_body = gs(caps, "ARGUMENTS").unwrap_or("");
    let (mut begin, name, yaml_name, mut key_equals_value): (String, String, &str, String) =
        if g(caps, "COMMAND").is_some() {
            (
                format!("{leading}\\"),
                gs(caps, "COMMANDNAME").unwrap_or("").to_string(),
                "commands",
                String::new(),
            )
        } else if g(caps, "NAMEDBRACESBRACKETS").is_some() {
            (
                leading.to_string(),
                gs(caps, "NAMEDBRACESBRACKETS").unwrap_or("").to_string(),
                "namedGroupingBracesBrackets",
                String::new(),
            )
        } else if g(caps, "KEYEQVAL").is_some() {
            let keyeq = gs(caps, "KEYEQVAL").unwrap_or("").to_string();
            let (n, kv) = split_key_equals(&keyeq);
            (
                format!("{leading}{n}"),
                n,
                "keyEqualsValuesBracesBrackets",
                kv,
            )
        } else {
            (
                leading.to_string(),
                String::new(),
                "UnNamedGroupingBracesBrackets",
                String::new(),
            )
        };
    logger::trace(&format!("*found: {name} \t type: {yaml_name}"));

    let os = settings_for(&name, yaml_name, true, is_m);

    let mut arg_body = arg_body.to_string();

    // key=value 恒定段：第一个 = 之后的内容并入参数体（非 -m 或无 equals poly 时）
    if yaml_name == "keyEqualsValuesBracesBrackets" && !is_m {
        if let Some(pos) = key_equals_value.find('=') {
            let rest = key_equals_value[pos + 1..].to_string();
            key_equals_value = key_equals_value[..pos + 1].to_string();
            arg_body = rest + &arg_body;
        }
    }

    // -m：命令名换行与参数起始换行
    if is_m {
        let poly = |k: &str| *os.poly.get(k).unwrap_or(&0);
        if poly("BeginStartsOnOwnLine") != 0 {
            let mut m = crate::mlb::MlbObj {
                begin: begin.clone(),
                body: String::new(),
                end: String::new(),
                horizontal_trailing_space: String::new(),
                trailing_comment: String::new(),
                linebreaks_at_end_begin: false,
                linebreaks_at_end_body: false,
                linebreaks_at_end_end: String::new(),
            };
            crate::mlb::begin_starts_on_own_line(
                &mut m,
                poly("BeginStartsOnOwnLine"),
                "something-with-braces",
            );
            let t2 = tok(|t| t.clone());
            begin = m.begin.replacen(
                &format!(
                    "{}{}",
                    t2.m_after_end_line_break, t2.m_before_begin_line_break_remove
                ),
                "",
                1,
            );
        }
        if poly("BodyStartsOnOwnLine") == -1 {
            arg_body = arg_body.trim_start().to_string();
        }

        // key=value 的 "=" 前后换行（EqualsStartsOnOwnLine / EqualsFinishesWithLineBreak）
        if yaml_name == "keyEqualsValuesBracesBrackets" {
            let v_es = poly("EqualsStartsOnOwnLine");
            let v_ef = poly("EqualsFinishesWithLineBreak");
            if v_es != 0 || v_ef != 0 {
                let t2 = tok(|t| t.clone());
                let mut kev = key_equals_value.clone();
                kev = kev.replacen('=', "", 1);
                let lead_re = crate::patterns::re_new(&format!(
                    r"(?s)^(?<ET>\h*)?(?<EC>{})?(?<EL>\R\s*)?",
                    state::patterns(|p| p.trailing_comment.clone())
                ))
                .unwrap();
                // Perl: 此 s/// 成功后 $+ 读到的是它的捕获（而非外层匹配的）
                let (hts, tcmt, lb_end) = match lead_re.captures(arg_body.as_bytes()).unwrap() {
                    Some(c) => (
                        gs(&c, "ET").unwrap_or("").to_string(),
                        gs(&c, "EC").unwrap_or("").to_string(),
                        gs(&c, "EL").unwrap_or("").to_string(),
                    ),
                    None => (String::new(), String::new(), String::new()),
                };
                arg_body =
                    crate::patterns::replace_all_with(&arg_body, &lead_re, |_| String::new());
                let mut m = crate::mlb::MlbObj {
                    begin: String::new(),
                    body: kev,
                    end: "=".to_string(),
                    horizontal_trailing_space: hts,
                    trailing_comment: tcmt,
                    linebreaks_at_end_begin: false,
                    linebreaks_at_end_body: false,
                    linebreaks_at_end_end: lb_end,
                };
                crate::mlb::end_starts_on_own_line(&mut m, v_es);
                if v_ef != 0 {
                    crate::mlb::end_finishes_with_line_break(&mut m, v_ef);
                }
                m.body.push_str(&m.end);
                m.body = crate::mlb::after_indentation_token_adjust(&m.body);
                key_equals_value = m.body;
                let _ = t2;
            }
            // 恒定段：第一个 = 之后的内容并入参数体
            if let Some(pos) = key_equals_value.find('=') {
                let rest = key_equals_value[pos + 1..].to_string();
                key_equals_value = key_equals_value[..pos + 1].to_string();
                arg_body = rest + &arg_body;
            }
            // EqualsFinishesWithLineBreak 追加的换行 token 移到 keyEqualsValue 尾
            if is_m {
                let t2 = tok(|t| t.clone());
                let r = crate::patterns::re_new(&format!(
                    r"(?s)^({}?{})",
                    state::patterns(|p| p.trailing_comment.clone()),
                    t2.m_after_end_line_break
                ))
                .unwrap();
                if r.is_match(arg_body.as_bytes()).unwrap_or(false) {
                    arg_body = crate::patterns::replace_all_with(&arg_body, &r, |c| {
                        key_equals_value.push_str(crate::patterns::gn(c, 1));
                        String::new()
                    });
                }
            }
        }
    }

    arg_body = indent_all_args(&name, yaml_name, &arg_body, current_indent);
    if yaml_name == "keyEqualsValuesBracesBrackets" {
        arg_body = key_equals_value + &arg_body;
    }

    let added = os.indentation.clone();
    if !added.is_empty() {
        arg_body = indent_lines(&arg_body, &added);
        arg_body = strip_first_indent(&arg_body, &added);
    }

    let body = name;
    let body = if yaml_name == "keyEqualsValuesBracesBrackets" {
        String::new()
    } else {
        body
    };
    let _ = t;
    begin + &body + &arg_body
}

fn split_key_equals(keyeq: &str) -> (String, String) {
    state::patterns(|p| match p.re_key_split.find(keyeq.as_bytes()).unwrap() {
        Some(m) => {
            let (s, e) = (m.start(), m.end());
            (
                format!("{}{}", &keyeq[..s], &keyeq[e..]),
                keyeq[s..e].to_string(),
            )
        }
        None => (keyeq.to_string(), String::new()),
    })
}

// ---------------------------------------------------------------------------
// _indent_all_args：逐个参数缩进
// ---------------------------------------------------------------------------

pub fn indent_all_args(name: &str, yaml_name: &str, body: &str, current_indent: &str) -> String {
    let is_m = switches::with(|s| s.is_m_switch_active());
    let os = settings_for(name, yaml_name, true, is_m);
    // Perl $parentBegin：命令 = "\"+name，命名花括号/keyEqVal = name，其余无
    let parent_begin: Option<String> = match yaml_name {
        "commands" => Some(format!("\\{name}")),
        "namedGroupingBracesBrackets" | "keyEqualsValuesBracesBrackets" => Some(name.to_string()),
        _ => None,
    };
    let (mand_ind, opt_ind) = (
        os.mandatory_arguments_indentation.clone(),
        os.optional_arguments_indentation.clone(),
    );
    let mut out = String::new();
    let mut pos = 0usize;
    loop {
        // \G 锚定的逐参数匹配（captures_read_at 的 start offset 即 \G 位置）
        let step = state::patterns(|p| {
            let mut locs = p.one_arg.capture_locations();
            match p.one_arg.captures_read_at(&mut locs, body.as_bytes(), pos) {
                Ok(Some(m)) if m.start() == pos => {
                    let spans: Vec<Option<(usize, usize)>> =
                        (0..locs.len()).map(|i| locs.get(i)).collect();
                    Some((m.end(), spans))
                }
                _ => None,
            }
        });
        let (end_pos, spans) = match step {
            Some(x) => x,
            None => break,
        };
        if end_pos == pos {
            break;
        }
        let names = state::patterns(|p| p.one_arg_names.clone());
        let pick = |logical: &str| -> &str {
            for suf in ["", "_env", "_if", "_sp", "_spn", "_mid"] {
                let key = format!("{logical}{suf}");
                if let Some(idx) = names.get(&key) {
                    if let Some(Some((s, e))) = spans.get(*idx) {
                        return &body[*s..*e];
                    }
                }
            }
            ""
        };
        let mandatory = !pick("MANDARGBEGIN").is_empty();
        let (bg, by, eg) = if mandatory {
            ("MANDARGBEGIN", "MANDARGBODY", "MANDARGEND")
        } else {
            ("OPTARGBEGIN", "OPTARGBODY", "OPTARGEND")
        };
        let mut begin = pick(bg).to_string();
        let mut arg_body = pick(by).to_string();
        let mut end = pick(eg).to_string();
        let mut arg_indent = format!(
            "{}{}",
            if mandatory { &mand_ind } else { &opt_ind },
            current_indent
        );

        if looks_for_code(&arg_body) {
            arg_body = find_all_code_blocks(&arg_body, current_indent);
        }

        // -m：单个参数的 poly-switch（左花括号/方括号起止换行）
        let mut arg_body_starts_own_line;
        if is_m {
            let get2 = |map: &std::collections::HashMap<String, i64>, k: &str| -> i64 {
                *map.get(k).unwrap_or(&0)
            };
            let (map, arg_typ) = if mandatory {
                (&os.mand, "mandatoryArguments")
            } else {
                (&os.opt, "optionalArguments")
            };
            // comma poly-switch
            let v_cs = get2(map, "CommaStartsOnOwnLine");
            let v_cf = get2(map, "CommaFinishesWithLineBreak");
            if v_cs != 0 || v_cf != 0 {
                arg_body = crate::mlb::commas_in_arg_body(&arg_body, v_cs, v_cf);
                let t2 = tok(|t| t.clone());
                for pat in [
                    format!(r"(?s)\R\h*{}", t2.m_before_begin_line_break_add),
                    format!(r"(?s){}", t2.m_before_begin_line_break_add),
                    format!(r"(?s){}", t2.m_after_end_line_break),
                ] {
                    let r = crate::patterns::re_new(&pat).unwrap();
                    arg_body =
                        crate::patterns::replace_all_with(&arg_body, &r, |_| "\n".to_string());
                }
            }
            let mut m = crate::mlb::MlbObj {
                begin: begin.clone(),
                body: arg_body.clone(),
                end: end.clone(),
                horizontal_trailing_space: pick("TRAILINGHSPACE").to_string(),
                trailing_comment: pick("TRAILINGCOMMENT").to_string(),
                linebreaks_at_end_begin: false,
                linebreaks_at_end_body: false,
                linebreaks_at_end_end: pick("TRAILINGLINEBREAK").to_string(),
            };
            // Perl：裸 \n 变体仅限 commands / keyEqualsValues 的参数，其余（环境等）用
            // ADD token（会被前导换行吸收，避免与上一参数 trailing 换行叠加）
            let typ_arg = if yaml_name == "commands" || yaml_name == "keyEqualsValuesBracesBrackets"
            {
                "argument-commands"
            } else {
                "argument-other"
            };
            crate::mlb::begin_starts_on_own_line(&mut m, get2(map, "LCuBStartsOnOwnLine"), typ_arg);
            crate::mlb::begin_starts_on_own_line(&mut m, get2(map, "LSqBStartsOnOwnLine"), typ_arg);
            crate::mlb::body_starts_on_own_line(
                &mut m,
                if arg_typ == "mandatoryArguments" {
                    get2(map, "MandArgBodyStartsOnOwnLine")
                } else {
                    get2(map, "OptArgBodyStartsOnOwnLine")
                },
            );
            crate::mlb::end_starts_on_own_line(
                &mut m,
                if arg_typ == "mandatoryArguments" {
                    get2(map, "RCuBStartsOnOwnLine")
                } else {
                    get2(map, "RSqBStartsOnOwnLine")
                },
            );
            crate::mlb::end_finishes_with_line_break(
                &mut m,
                if arg_typ == "mandatoryArguments" {
                    get2(map, "RCuBFinishesWithLineBreak")
                } else {
                    get2(map, "RSqBFinishesWithLineBreak")
                },
            );
            begin = m.begin;
            arg_body = m.body;
            end = m.end;
            // Perl：poly 对象的 linebreaksAtEnd{begin} 决定 body 是否独行
            arg_body_starts_own_line = m.linebreaks_at_end_begin;
        } else {
            arg_body_starts_own_line = body_starts_linebreak(&arg_body);
        }

        // lookForAlignDelims：poly 之后对齐（Perl _indent_all_args 顺序）；
        // Perl $currentIndentation = argument.indentation if defined
        if os.look_for_align_delims {
            // Perl `${$self}{DBSStartsOnOwnLine}`：参数对象取 mand/opt 映射值
            let dbs_map = if mandatory { &os.mand } else { &os.opt };
            let dbs_vals = (
                *dbs_map.get("DBSStartsOnOwnLine").unwrap_or(&0),
                *dbs_map.get("DBSFinishesWithLineBreak").unwrap_or(&0),
            );
            let res = crate::alignment::align_at_ampersand_with_parent(
                &mut begin,
                &mut arg_body,
                &os,
                parent_begin.as_deref(),
                Some(dbs_vals),
            );
            if let Some(ind) = res.indentation {
                arg_indent = ind;
            }
        }

        // 参数体缩进（Perl：poly/align 后统一处理）
        let body_has_line_breaks = arg_body.contains('\n');
        if !arg_body.is_empty() && (arg_body_starts_own_line || body_has_line_breaks) {
            arg_body = indent_lines(&arg_body, &arg_indent);
            if !arg_body_starts_own_line && body_has_line_breaks {
                arg_body = strip_first_indent(&arg_body, &arg_indent);
            }
        }
        out.push_str(&begin);
        out.push_str(&arg_body);
        out.push_str(&end);
        // 注：参数后的水平空白/注释/换行已由 end_finishes_with_line_break 并入 end，勿重复还原
        pos = end_pos;
    }
    out.push_str(&body[pos..]);
    if is_m {
        // Perl: ($REMOVE)($argumentsBetween*)\R? → $2$1，随后 token 归一
        let between = state::settings(|s| {
            crate::settings::as_str(crate::settings::get_path(
                &s.root,
                &["fineTuning", "arguments", "between"],
            ))
            .unwrap_or("")
            .to_string()
        });
        if !between.is_empty() {
            let t2 = tok(|t| t.clone());
            // `between` 常以 `(?x)` + 行尾 `#` 注释书写（texexchange/348 的 yaml 就是），
            // 直接拼进去会把紧随其后的 `)` 注释掉 → missing closing parenthesis。
            // 与 `arg_trivia` 一致，先过 `x_guard` 补一个换行。
            let r = crate::patterns::re_new(&format!(
                r"(?s)({})({})\R?",
                t2.m_before_begin_line_break_remove,
                crate::patterns::x_guard(&between)
            ))
            .unwrap();
            out = crate::patterns::replace_all_with(&out, &r, |c| {
                format!("{}{}", crate::patterns::gn(c, 2), crate::patterns::gn(c, 1))
            });
        }
        out = crate::mlb::line_break_token_adjust(&out);
    }
    out
}

// ---------------------------------------------------------------------------
// _find_ifelsefi_else_or / _find_special_middle / _find_env_items
// ---------------------------------------------------------------------------

fn middle_regexp(name: &str) -> Option<String> {
    state::settings(|s| s.special_lookup_middle.get(name).cloned())
}

fn gws<'t>(caps: &'t Captures) -> &'t str {
    gs(caps, "WS").unwrap_or("")
}

pub fn find_ifelsefi_else_or(body: &str, name: &str) -> (String, bool) {
    let t = tok(|t| t.clone());
    logger::trace(&format!("looking for ifelsefi ELSE for {name}"));
    let else_re = state::settings(|s| {
        crate::settings::as_str(crate::settings::get_path(
            &s.root,
            &["fineTuning", "ifElseFi", "elseOrRegEx"],
        ))
        .unwrap_or(r"\\(?:else|or)")
        .to_string()
    });
    let is_m = switches::with(|s| s.is_m_switch_active());
    let tail = if is_m {
        state::patterns(|p| {
            format!(
                r"(?:(?<ET>\h*)?(?<EC>{})?(?<EL>\R\s*)?)",
                p.trailing_comment
            )
        })
    } else {
        String::new()
    };
    let re = make_middle_regex(&format!(
        r"(?s)(?<WS>\s*)(?:(?<IFELSEFIELSE>(?<IFELSEFIELSESTATEMENT>(?:{else_re})){tail})|(?<IFELSEFIALL>(?&dIF)))"
    ));
    let mut else_exists = false;
    let out = replace_all_with(body, &re, |caps| {
        let ws = gws(caps);
        if g(caps, "IFELSEFIELSE").is_some() {
            else_exists = true;
            let stmt = gs(caps, "IFELSEFIELSESTATEMENT").unwrap_or("");
            if is_m {
                // \else / \or 的 poly-switch（middle 对象）
                let t2 = tok(|t| t.clone());
                let poly = |k: &str| -> i64 {
                    state::settings(|s| s.poly_switch_value("ifElseFi", name, k)).unwrap_or(0)
                };
                let (v_begin, v_body) = if stmt.contains("\\or") {
                    (poly("OrStartsOnOwnLine"), poly("OrFinishesWithLineBreak"))
                } else {
                    (
                        poly("ElseStartsOnOwnLine"),
                        poly("ElseFinishesWithLineBreak"),
                    )
                };
                let mut m = crate::mlb::MlbObj {
                    begin: format!("{ws}{stmt}"),
                    body: format!(
                        "{}{}{}",
                        gs(caps, "ET").unwrap_or(""),
                        gs(caps, "EC").unwrap_or(""),
                        gs(caps, "EL").unwrap_or("")
                    ),
                    end: String::new(),
                    horizontal_trailing_space: String::new(),
                    trailing_comment: String::new(),
                    linebreaks_at_end_begin: false,
                    linebreaks_at_end_body: false,
                    linebreaks_at_end_end: String::new(),
                };
                crate::mlb::begin_starts_on_own_line(&mut m, v_begin, "block");
                // Perl: \else 后无水平空白且将拼接时插 mElseTrailingSpace token
                let else_needs_space = v_body == -1 && gs(caps, "ET").unwrap_or("").is_empty();
                crate::mlb::body_starts_on_own_line(&mut m, v_body);
                let mut s = format!(
                    "{}{}{}{}",
                    m.begin,
                    if else_needs_space {
                        t2.m_else_trailing_space.clone()
                    } else {
                        String::new()
                    },
                    m.body,
                    m.end
                );
                // Perl: s/^(\s*)/$1 specialMiddle/ 后再做 token 重排
                let r = crate::patterns::re_new(r"(?s)^(\s*)").unwrap();
                s = crate::patterns::replace_all_with(&s, &r, |c| {
                    format!(
                        "{}{}",
                        crate::patterns::gn(c, 1),
                        t2.special_begin_end_middle
                    )
                });
                let r = crate::patterns::re_new(&format!(
                    r"(?s)({}\s*)({}|{})(\s*)",
                    t2.special_begin_end_middle, t2.m_switch_comment, t2.blanklines
                ))
                .unwrap();
                s = crate::patterns::replace_all_with(&s, &r, |c| {
                    format!(
                        "{}{}{}",
                        crate::patterns::gn(c, 2),
                        crate::patterns::gn(c, 3),
                        crate::patterns::gn(c, 1)
                    )
                });
                let r = crate::patterns::re_new(&format!(
                    r"(?s)({}\s*)({}(?:{})?)(\s*)",
                    t2.special_begin_end_middle, t2.m_before_begin_line_break_add, t2.blanklines
                ))
                .unwrap();
                s = crate::patterns::replace_all_with(&s, &r, |c| {
                    format!(
                        "{}{}{}",
                        crate::patterns::gn(c, 2),
                        crate::patterns::gn(c, 3),
                        crate::patterns::gn(c, 1)
                    )
                });
                s
            } else {
                format!("{ws}{}{stmt}", t.special_begin_end_middle)
            }
        } else {
            let all = gs(caps, "IFELSEFIALL").unwrap_or("");
            format!("{ws}{all}")
        }
    });
    (out, else_exists)
}

pub fn find_special_middle(body: &str, name: &str) -> String {
    let t = tok(|t| t.clone());
    logger::trace(&format!("looking for special MIDDLE for {name}"));
    let middle = match middle_regexp(name) {
        Some(m) => m,
        None => return body.to_string(),
    };
    let is_m = switches::with(|s| s.is_m_switch_active());
    let tail = if is_m {
        state::patterns(|p| {
            format!(
                r"(?:(?<ET>\h*)?(?<EC>{})?(?<EL>\R\s*)?)",
                p.trailing_comment
            )
        })
    } else {
        String::new()
    };
    // middle 语句标注 token；其它内容透传。
    // Perl 的 SPECIALALL 分支是 $specialAllRegEx（完整 special 块），
    // 因此已渲染的嵌套 special 会被整块吞掉，其中的 middle 不会被二次标注。
    let special_all = state::patterns(|p| p.special_all.clone());
    let re = make_middle_regex(&format!(
        r"(?s)(?<WS>\s*)(?:(?<SPECIALMIDDLE>(?<SPECIALMIDDLESTATEMENT>(?:{middle})){tail})|(?<SPECIALALL>(?:{special_all})))",
        middle = middle,
        special_all = special_all,
        tail = tail
    ));
    replace_all_with(body, &re, |caps| {
        let ws = gws(caps);
        if g(caps, "SPECIALMIDDLE").is_some() {
            let stmt = gs(caps, "SPECIALMIDDLESTATEMENT").unwrap_or("");
            // Perl Blocks.pm `_find_special_middle`：命中 MIDDLE 时打 trace
            // （注意与其他块不同，这里 `\t` 前没有空格）
            logger::trace(&format!("*found: {stmt}\t type: specialBeginEnd MIDDLE"));
            if is_m {
                // SpecialMiddle 的 poly-switch（middle 对象）
                let t2 = tok(|t| t.clone());
                let poly = |k: &str| -> i64 {
                    state::settings(|s| s.poly_switch_value("specialBeginEnd", name, k))
                        .unwrap_or(0)
                };
                let mut m = crate::mlb::MlbObj {
                    begin: format!("{ws}{stmt}"),
                    body: format!(
                        "{}{}{}",
                        gs(caps, "ET").unwrap_or(""),
                        gs(caps, "EC").unwrap_or(""),
                        gs(caps, "EL").unwrap_or("")
                    ),
                    end: String::new(),
                    horizontal_trailing_space: String::new(),
                    trailing_comment: String::new(),
                    linebreaks_at_end_begin: false,
                    linebreaks_at_end_body: false,
                    linebreaks_at_end_end: String::new(),
                };
                crate::mlb::begin_starts_on_own_line(
                    &mut m,
                    poly("SpecialMiddleStartsOnOwnLine"),
                    "block",
                );
                crate::mlb::body_starts_on_own_line(
                    &mut m,
                    poly("SpecialMiddleFinishesWithLineBreak"),
                );
                let mut s = format!("{}{}{}", m.begin, m.body, m.end);
                // Perl: s/^(\s*)/$1 specialMiddle/ 后再做 token 重排
                let r = crate::patterns::re_new(r"(?s)^(\s*)").unwrap();
                s = crate::patterns::replace_all_with(&s, &r, |c| {
                    format!(
                        "{}{}",
                        crate::patterns::gn(c, 1),
                        t2.special_begin_end_middle
                    )
                });
                let r = crate::patterns::re_new(&format!(
                    r"(?s)({}\s*)({}|{})(\s*)",
                    t2.special_begin_end_middle, t2.m_switch_comment, t2.blanklines
                ))
                .unwrap();
                s = crate::patterns::replace_all_with(&s, &r, |c| {
                    format!(
                        "{}{}{}",
                        crate::patterns::gn(c, 2),
                        crate::patterns::gn(c, 3),
                        crate::patterns::gn(c, 1)
                    )
                });
                let r = crate::patterns::re_new(&format!(
                    r"(?s)({}\s*)({}(?:{})?)(\s*)",
                    t2.special_begin_end_middle, t2.m_before_begin_line_break_add, t2.blanklines
                ))
                .unwrap();
                s = crate::patterns::replace_all_with(&s, &r, |c| {
                    format!(
                        "{}{}{}",
                        crate::patterns::gn(c, 2),
                        crate::patterns::gn(c, 3),
                        crate::patterns::gn(c, 1)
                    )
                });
                s
            } else {
                format!("{ws}{}{stmt}", t.special_begin_end_middle)
            }
        } else {
            let all = gs(caps, "SPECIALALL").unwrap_or("");
            format!("{ws}{all}")
        }
    })
}

fn make_middle_regex(src: &str) -> Regex {
    let defs = state::patterns(|p| p.defs.clone());
    crate::patterns::re_new(&format!("{defs}{src}")).expect("middle regex")
}

pub fn find_env_items(body: &str, name: &str, current_indent: &str) -> String {
    let is_m = switches::with(|s| s.is_m_switch_active());
    logger::trace(&format!("looking for items for {name}"));
    let item_re_src = ();

    // 先处理首个 item 之前的内容（Perl: split 后 shift 出来单独递归）
    let (before, after) = state::patterns(|p| match p.re_item.find(body.as_bytes()).unwrap() {
        Some(m) if m.start() > 0 => (body[..m.start()].to_string(), body[m.start()..].to_string()),
        Some(_) => (String::new(), body.to_string()),
        None => (body.to_string(), String::new()),
    });
    let mut out = find_all_code_blocks(&before, current_indent);

    let rendered = state::patterns(|p| {
        replace_all_with(&after, &p.re_item, |caps| {
            let ws = gws(caps);
            let item_begin = gs(caps, "ITEMBEGIN").unwrap_or("");
            let mut body = gs(caps, "ITEMBODY").unwrap_or("").to_string();
            let mut linebreaks_at_end_begin;
            let mut begin = format!("{ws}{item_begin}");
            let name: String = begin
                .chars()
                .filter(|c| c.is_ascii_alphanumeric())
                .collect();
            logger::trace(&format!("*found: {name} \t type: ITEM"));
            let os = settings_for(&name, "items", false, is_m);
            if looks_for_code(&body) {
                body = find_all_code_blocks(&body, current_indent);
            }
            // -m：item poly-switch
            if is_m {
                let t2 = tok(|t| t.clone());
                let mut m = crate::mlb::MlbObj {
                    begin: begin.clone(),
                    body: body.clone(),
                    end: String::new(),
                    horizontal_trailing_space: String::new(),
                    trailing_comment: String::new(),
                    linebreaks_at_end_begin: false,
                    linebreaks_at_end_body: false,
                    linebreaks_at_end_end: String::new(),
                };
                let poly = |k: &str| *os.poly.get(k).unwrap_or(&0);
                crate::mlb::begin_starts_on_own_line(&mut m, poly("BeginStartsOnOwnLine"), "block");
                crate::mlb::body_starts_on_own_line(&mut m, poly("BodyStartsOnOwnLine"));
                if poly("BeginStartsOnOwnLine") == 4 {
                    let r = crate::patterns::re_new(&format!(
                        r"(?s)({})\s*{}",
                        t2.blanklines, t2.m_before_begin_line_break_remove
                    ))
                    .unwrap();
                    m.begin = crate::patterns::replace_all_with(&m.begin, &r, |c| {
                        format!("{}\n", crate::patterns::gn(c, 1))
                    });
                    let r = crate::patterns::re_new(&format!(
                        r"(?s){}",
                        t2.m_before_begin_line_break_add
                    ))
                    .unwrap();
                    m.begin = crate::patterns::replace_all_with(&m.begin, &r, |_| "\n".to_string());
                }
                if poly("BodyStartsOnOwnLine") == -1 {
                    m.begin.push_str(&t2.m_else_trailing_space);
                }
                begin = m.begin;
                body = m.body;
                linebreaks_at_end_begin = m.linebreaks_at_end_begin;
            } else {
                linebreaks_at_end_begin = body_starts_linebreak(&body);
            }
            let added = os.indentation.clone();
            if !added.is_empty() {
                if !state::patterns(|p| {
                    p.re_arg_only_space
                        .is_match(body.as_bytes())
                        .unwrap_or(false)
                }) {
                    body = indent_lines(&body, &added);
                }
                if !linebreaks_at_end_begin {
                    body = strip_first_indent(&body, &added);
                }
            }
            begin + &body
        })
    });
    let mut out = out + &rendered;
    if is_m {
        let t = tok(|t| t.clone());
        let item = state::patterns(|p| p.item_regexp.clone());
        // Perl 顺序敏感的四项收尾（itemRegEx 按正则原样嵌入）
        let r = crate::patterns::re_new(&format!(
            r"(?s)\R\h*{}\R(\h*(?:{}))",
            t.m_switch_comment, item
        ))
        .unwrap();
        out = crate::patterns::replace_all_with(&out, &r, |c| {
            format!("\n{}", crate::patterns::gn(c, 1))
        });
        let r =
            crate::patterns::re_new(&format!(r"(?s)^{}\R(\h*(?:{}))", t.m_switch_comment, item))
                .unwrap();
        out =
            crate::patterns::replace_all_with(&out, &r, |c| crate::patterns::gn(c, 1).to_string());
        let r = crate::patterns::re_new(&format!(
            r"(?s)\R\h*{}(?:{})?\s*((?:{}))",
            t.m_before_begin_line_break_add, t.blanklines, item
        ))
        .unwrap();
        out = crate::patterns::replace_all_with(&out, &r, |c| {
            format!("\n{}", crate::patterns::gn(c, 1))
        });
        let r = crate::patterns::re_new(&format!(
            r"(?s)([a-zA-Z0-9]){}([a-zA-Z0-9])",
            t.m_else_trailing_space
        ))
        .unwrap();
        out = crate::patterns::replace_all_with(&out, &r, |c| {
            format!(
                "{} {}",
                crate::patterns::gn(c, 1),
                crate::patterns::gn(c, 2)
            )
        });
        out = out.replace(&t.m_else_trailing_space, "");
        out = crate::mlb::line_break_token_adjust(&out);
    }
    out
}

fn regex_escape(s: &str) -> String {
    let mut out = String::new();
    for ch in s.chars() {
        if r"\.^$|?*+()[]{}".contains(ch) {
            out.push('\\');
        }
        out.push(ch);
    }
    out
}

// ---------------------------------------------------------------------------
// 小工具
// ---------------------------------------------------------------------------

fn body_starts_linebreak(body: &str) -> bool {
    state::patterns(|p| {
        p.re_body_linebreak
            .is_match(body.as_bytes())
            .unwrap_or(false)
    })
}

fn leading_hspace(s: &str) -> String {
    s.chars().take_while(|c| *c == ' ' || *c == '\t').collect()
}

fn rest_linebreak(arg: &str) -> String {
    let h = leading_hspace(arg);
    let rest = &arg[h.len()..];
    let mut out = String::new();
    let mut chars = rest.chars().peekable();
    while let Some(c) = chars.peek() {
        if *c == '\r' || *c == '\n' {
            out.push(*c);
            chars.next();
        } else {
            break;
        }
    }
    while let Some(c) = chars.peek() {
        if c.is_whitespace() {
            out.push(*c);
            chars.next();
        } else {
            break;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::indent_lines;

    /// 抽象不变式：首行只有水平空白（随后即换行）时，indent_lines 不得把缩进
    /// 插到换行之前。否则 CRLF 下这层缩进无法被 `\h+$` 清理，每次格式化都会
    /// 在 `\begin{...}` / `\item` 行尾再叠一层（Windows/VS Code 光标漂移根因）。
    ///
    /// 覆盖维度：缩进样式 × 行尾（LF/CRLF）× 首行空白组合 × 首行之后的内容。
    #[test]
    fn never_indents_a_blank_first_line() {
        for indent in ["\t", "  ", "    "] {
            for eol in ["\n", "\r\n"] {
                for blank in ["", " ", "\t", "  \t"] {
                    for rest in ["", "content", "\tcontent", "  content"] {
                        let input = format!("{blank}{eol}{rest}{eol}");
                        let out = indent_lines(&input, indent);
                        let first_out = out.split(eol).next().unwrap();
                        assert_eq!(
                            first_out, blank,
                            "首行空白被改写: indent={indent:?}, eol={eol:?}, input={input:?}"
                        );
                    }
                }
            }
        }
    }

    /// 互补不变式：首行有实际内容时仍按 Perl `s/^/indent/mg` 的语义加缩进。
    #[test]
    fn indents_a_non_blank_first_line() {
        for indent in ["\t", "  ", "    "] {
            for eol in ["\n", "\r\n"] {
                for content in ["content", "\tcontent", "  content"] {
                    let input = format!("{content}{eol}");
                    let out = indent_lines(&input, indent);
                    let first_out = out.split(eol).next().unwrap();
                    assert_eq!(
                        first_out,
                        format!("{indent}{content}"),
                        "首行内容缩进错误: indent={indent:?}, eol={eol:?}"
                    );
                }
            }
        }
    }
}
