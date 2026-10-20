//! verbatim / noindentblock 保护与还原（对应 LatexIndent/Verbatim.pm）。

use crate::logger;
use crate::settings::{as_bool, as_str, get, get_path};
use crate::state::{self, VerbatimObj};
use crate::switches;
use crate::tokens::tok;

fn create_unique_id(typ: &str) -> String {
    let t = tok(|t| t.clone());
    let counter = state::storage(|s| {
        let n = s.verbatim.len() + s.filecontents.len() + 1;
        n
    });
    let prefix = if typ == "command" {
        t.verbatim_inline
    } else {
        t.verbatim
    };
    format!("{prefix}{counter}{}", t.end_of_token)
}

fn verbatim_common_tasks(obj: &mut VerbatimObj) {
    obj.id = create_unique_id(&obj.typ);
    obj.replacement_text = obj.id.clone();

    let is_m = switches::with(|s| s.is_m_switch_active());
    if !is_m {
        // 非 -m：replacementText = id + hts + linebreaksAtEnd{end}
        obj.replacement_text
            .push_str(&obj.horizontal_trailing_space);
        obj.replacement_text.push_str(&obj.linebreaks_at_end);
        return;
    }

    // ---- -m 分支（Perl Verbatim.pm verbatim_common_tasks）----
    let ends_poly = state::settings(|s| {
        s.poly_switch_value(&obj.yaml_name, &obj.name, "EndFinishesWithLineBreak")
    })
    .unwrap_or(0);
    // 记录下来，供 `put_verbatim_back_in` 判断是否需要先吃掉占位符后的换行（== -1）
    obj.end_finishes_with_line_break = ends_poly;

    // horizontalTrailingSpace 挂到 replacementText；唯一例外是 EndFinishesWithLineBreak == 2
    if ends_poly != 2 {
        obj.replacement_text
            .push_str(&obj.horizontal_trailing_space);
    }

    // VerbatimEndFinishesWithLineBreak != 0：修改 end，并把 m-switch token 转挂到 replacementText
    if ends_poly != 0 {
        let original_end = obj.end.clone();
        let t = tok(|t| t.clone());
        let mut m = crate::mlb::MlbObj {
            begin: String::new(),
            body: String::new(),
            end: obj.end.clone(),
            horizontal_trailing_space: obj.horizontal_trailing_space.clone(),
            trailing_comment: String::new(),
            linebreaks_at_end_begin: false,
            linebreaks_at_end_body: false,
            linebreaks_at_end_end: obj.linebreaks_at_end.clone(),
        };
        crate::mlb::end_finishes_with_line_break(&mut m, ends_poly);
        obj.end = m.end;
        if original_end != obj.end {
            // Perl Verbatim.pm L625-634：
            //   `${$self}{end} =~ s/\Q${originalEnd}\E(.*)//s;  my $mSwitchTokens = $1;`
            //   `${$self}{replacementText} =~ s/\s*$//s;`
            //   `${$self}{replacementText} .= $mSwitchTokens;`
            // 关键是**先清尾空白再追加**：replacementText 此前已被拼上
            // `horizontalTrailingSpace`（`\end{verbatim}` 行尾的那个空格），
            // 而 mSwitchTokens 只含 `\n`（mAfterEndLineBreak），不含该空格。
            // 若不清尾，`\end{verbatim} ` 会残留一个尾空格，与基准不符。
            let tail = obj.end[original_end.len().min(obj.end.len())..].to_string();
            obj.end = original_end;
            while obj.replacement_text.ends_with(|c: char| c.is_whitespace()) {
                obj.replacement_text.pop();
            }
            obj.replacement_text.push_str(&tail);
        } else {
            // 未改动：replacementText 仍按 0 分支的形式收尾
            while obj.replacement_text.ends_with(|c: char| c.is_whitespace()) {
                obj.replacement_text.pop();
            }
            obj.replacement_text
                .push_str(&obj.horizontal_trailing_space);
            obj.replacement_text.push_str(&obj.linebreaks_at_end);
        }
        let _ = t;
    } else {
        // == 0：replacementText = id + hts + linebreaksAtEnd{end}，随后清空二者
        while obj.replacement_text.ends_with(|c: char| c.is_whitespace()) {
            obj.replacement_text.pop();
        }
        obj.replacement_text
            .push_str(&obj.horizontal_trailing_space);
        obj.replacement_text.push_str(&obj.linebreaks_at_end);
        obj.linebreaks_at_end = String::new();
        obj.horizontal_trailing_space = String::new();
    }
}

fn store(obj: VerbatimObj) {
    let id = obj.id.clone();
    state::storage(|s| {
        s.verbatim.insert(id, obj);
    });
}

/// find_noindent_block：`% \begin{noindent}` 保护块。
pub fn find_noindent_block(body: &str) -> String {
    logger::trace("*Searching for NOINDENTBLOCK (see noIndentBlock)");
    let mut body = body.to_string();
    let entries: Vec<(String, serde_yaml::Value)> = state::settings(|s| {
        get(&s.root, "noIndentBlock")
            .and_then(|v| v.as_mapping())
            .map(|m| {
                m.iter()
                    .map(|(k, v)| (k.as_str().unwrap_or_default().to_string(), v.clone()))
                    .collect()
            })
            .unwrap_or_default()
    });
    for (key, val) in entries {
        logger::trace(&format!("looking for {key} noIndentBlock"));
        let (src, name) = if val.is_mapping() {
            let m = val.as_mapping().unwrap();
            if !as_bool(get(m, "lookForThis")) && get(m, "lookForThis").is_some() {
                continue;
            }
            let name = as_str(get(m, "name")).unwrap_or_default().to_string();
            let begin = as_str(get(m, "begin")).unwrap_or_default().to_string();
            let end = as_str(get(m, "end")).unwrap_or_default().to_string();
            if name.is_empty() && (begin.is_empty() || end.is_empty()) {
                continue;
            }
            let body_re = as_str(get(m, "body")).unwrap_or(".*?").to_string();
            if !name.is_empty() && begin.is_empty() && end.is_empty() {
                // Perl Verbatim.pm L77-85：`name` 只给 name、未给 begin/end 时，
                // 默认 `begin = \\begin\{(name)\}`、`end = \\end\{\2\}`，
                // **不带 `%` 前缀**（那是 `noIndentBlock:<env>: 1` 布尔写法才有的语义）。
                // `name` 本身即正则，**不做 `*` 转义**（`\w+noindent\*?` 的星号是量词）。
                let n = name.clone();
                (
                    format!(
                        r"(?s)(?<B>\\begin\{{(?<N>{n})\}})(?<BY>{body_re})(?<E>\\end\{{\k<N>\}})",
                        n = n,
                        body_re = body_re
                    ),
                    name,
                )
            } else {
                (
                    format!(
                        r"(?s)(?<B>{begin})(?<BY>{body_re})(?<E>{end})",
                        begin = begin,
                        end = end,
                        body_re = body_re
                    ),
                    name,
                )
            }
        } else if as_bool(Some(&val)) {
            let n = key.replace('*', r"\*");
            (
                format!(
                    r"(?s)(?<B>(?!<\\)%(?:\h|(?!<\\)%)*\\begin\{{(?<N>{n})\}})(?<BY>.*?)(?<E>(?!<\\)%(?:\h|(?!<\\)%)*\\end\{{\k<N>\}})",
                    n = n
                ),
                key.clone(),
            )
        } else {
            continue;
        };
        let re = match crate::patterns::re_new(&src) {
            Ok(re) => re,
            Err(_) => continue,
        };
        body = crate::patterns::replace_all_with(&body, &re, |caps| {
            let begin = crate::patterns::gs(caps, "B").unwrap_or("").to_string();
            let b = crate::patterns::gs(caps, "BY").unwrap_or("").to_string();
            let end = crate::patterns::gs(caps, "E").unwrap_or("").to_string();
            let mut obj = VerbatimObj {
                begin,
                body: b,
                end,
                name: name.clone(),
                typ: "noindentblock".into(),
                yaml_name: "verbatim".into(),
                ..Default::default()
            };
            obj.id = create_unique_id(&obj.typ);
            let id = obj.id.clone();
            store(obj);
            id
        });
    }
    body
}

/// find_verbatim_commands：\verb!body! 等。
pub fn find_verbatim_commands(body: &str) -> String {
    logger::trace("*Searching for VERBATIM commands (see verbatimCommands)");
    let t = tok(|t| t.clone());
    let mut body = body.to_string();
    let entries: Vec<(String, serde_yaml::Value)> = state::settings(|s| {
        get(&s.root, "verbatimCommands")
            .and_then(|v| v.as_mapping())
            .map(|m| {
                m.iter()
                    .map(|(k, v)| (k.as_str().unwrap_or_default().to_string(), v.clone()))
                    .collect()
            })
            .unwrap_or_default()
    });
    for (key, val) in entries {
        logger::trace(&format!("looking for {key} Commands"));
        let name = if val.is_mapping() {
            let m = val.as_mapping().unwrap();
            if !as_bool(get(m, "lookForThis")) && get(m, "lookForThis").is_some() {
                continue;
            }
            as_str(get(m, "name")).unwrap_or(&key).to_string()
        } else if as_bool(Some(&val)) {
            key
        } else {
            continue;
        };
        let src = format!(
            r"(?mx)(?<B1>\\(?<N>{name})\h*)(?<OA>\[(?:(?!(?:(?<!\\)\[)).)*?(?<!\\)\]\h*)?(?<D>.)(?<BY>.*?)\k<D>(?<HS>\h*)?(?<LB>\R)?",
            name = name
        );
        let re = match crate::patterns::re_new(&src) {
            Ok(re) => re,
            Err(_) => continue,
        };
        body = crate::patterns::replace_all_with(&body, &re, |caps| {
            let opt_arg = crate::patterns::gs(caps, "OA").unwrap_or("").to_string();
            let delim = crate::patterns::gs(caps, "D").unwrap_or("").to_string();
            let mut b = crate::patterns::gs(caps, "BY").unwrap_or("").to_string();
            let hs = crate::patterns::gs(caps, "HS").unwrap_or("").to_string();
            let lb = crate::patterns::gs(caps, "LB").unwrap_or("").to_string();
            let name = crate::patterns::gs(caps, "N").unwrap_or("").to_string();
            // 嵌套 \verb：还原内层并回收
            let nested_re = crate::patterns::re_new(&format!(
                r"(?s){}\d+{}",
                regex_esc(&t.verbatim_inline),
                regex_esc(&t.end_of_token)
            ))
            .unwrap();
            if nested_re.is_match(b.as_bytes()).unwrap_or(false) {
                let ids: Vec<String> = state::storage(|s| s.verbatim.keys().cloned().collect());
                for id in ids {
                    if b.contains(&id) {
                        let inner = state::storage(|s| s.verbatim.remove(&id));
                        if let Some(inner) = inner {
                            b = b.replace(
                                &id,
                                &format!("{}{}{}", inner.begin, inner.body, inner.end),
                            );
                        }
                    }
                }
            }
            let begin = format!(
                "{}{}{}{}",
                crate::patterns::gs(caps, "B1").unwrap_or(""),
                opt_arg,
                delim,
                ""
            );
            let mut obj = VerbatimObj {
                begin,
                body: b,
                end: delim,
                name,
                typ: "command".into(),
                yaml_name: "verbatim".into(),
                opt_arg,
                horizontal_trailing_space: hs,
                linebreaks_at_end: lb,
                ..Default::default()
            };
            verbatim_common_tasks(&mut obj);
            let repl = obj.replacement_text.clone();
            store(obj);
            repl
        });
    }
    body
}

/// find_verbatim_environments。
pub fn find_verbatim_environments(body: &str) -> String {
    logger::trace("*Searching for VERBATIM environments (see verbatimEnvironments)");
    let mut body = body.to_string();
    let entries: Vec<(String, serde_yaml::Value)> = state::settings(|s| {
        get(&s.root, "verbatimEnvironments")
            .and_then(|v| v.as_mapping())
            .map(|m| {
                m.iter()
                    .map(|(k, v)| (k.as_str().unwrap_or_default().to_string(), v.clone()))
                    .collect()
            })
            .unwrap_or_default()
    });
    for (key, val) in entries {
        logger::trace(&format!("looking for {key} environments"));
        // Perl：只有**布尔写法**（`verbatim: 1`）或直接用 key 作环境名时才
        // `s/\*/\\*/sg`（把字面星号转义）；HASH 写法里的 `name` 是正则，
        // 星号必须保留为量词（见 `nameAsRegex.yaml` 的 `\w+code\*?`）。
        let (name, is_regex_name) = if val.is_mapping() {
            let m = val.as_mapping().unwrap();
            if !as_bool(get(m, "lookForThis")) && get(m, "lookForThis").is_some() {
                continue;
            }
            match as_str(get(m, "name")) {
                Some(n) => (n.to_string(), true),
                None => (key.clone(), false),
            }
        } else if as_bool(Some(&val)) {
            (key.clone(), false)
        } else {
            continue;
        };
        let n = if is_regex_name {
            name.clone()
        } else {
            name.replace('*', r"\*")
        };
        let src = format!(
            r"(?s)(?<B>\\begin\{{(?<N>{n})\}})(?<BY>.*?)(?<E>\\end\{{\k<N>\}})(?<HS>\h*)?(?<LB>\R\s*)?",
            n = n
        );
        let re = match crate::patterns::re_new(&src) {
            Ok(re) => re,
            Err(_) => continue,
        };
        body = crate::patterns::replace_all_with(&body, &re, |caps| {
            let mut obj = VerbatimObj {
                begin: crate::patterns::gs(caps, "B").unwrap_or("").to_string(),
                body: crate::patterns::gs(caps, "BY").unwrap_or("").to_string(),
                end: crate::patterns::gs(caps, "E").unwrap_or("").to_string(),
                name: crate::patterns::gs(caps, "N").unwrap_or("").to_string(),
                typ: "environment".into(),
                yaml_name: "verbatim".into(),
                horizontal_trailing_space: crate::patterns::gs(caps, "HS")
                    .unwrap_or("")
                    .to_string(),
                linebreaks_at_end: crate::patterns::gs(caps, "LB").unwrap_or("").to_string(),
                ..Default::default()
            };
            verbatim_common_tasks(&mut obj);
            let repl = obj.replacement_text.clone();
            store(obj);
            repl
        });
    }
    body
}

/// find_verbatim_special：specialBeginEnd 中 lookForThis == 'verbatim' 的块。
pub fn find_verbatim_special(body: &str) -> String {
    let mut body = body.to_string();
    let specials: Vec<(String, String, String)> = state::settings(|s| {
        get(&s.root, "specialBeginEnd")
            .and_then(|v| v.as_sequence())
            .map(|seq| {
                seq.iter()
                    .filter_map(|item| {
                        let m = item.as_mapping()?;
                        if as_str(get(m, "lookForThis")) != Some("verbatim") {
                            return None;
                        }
                        Some((
                            as_str(get(m, "name"))?.to_string(),
                            as_str(get(m, "begin"))?.to_string(),
                            as_str(get(m, "end"))?.to_string(),
                        ))
                    })
                    .collect()
            })
            .unwrap_or_default()
    });
    for (name, begin, end) in specials {
        logger::trace("*Searching for VERBATIM special (see specialBeginEnd)");
        let src = format!(
            r"(?s)(?<B>{begin})(?<BY>.*?)(?<E>{end})(?<HS>\h*)?(?<LB>\R)?",
            begin = begin,
            end = end
        );
        let re = match crate::patterns::re_new(&src) {
            Ok(re) => re,
            Err(_) => continue,
        };
        body = crate::patterns::replace_all_with(&body, &re, |caps| {
            let mut obj = VerbatimObj {
                begin: crate::patterns::gs(caps, "B").unwrap_or("").to_string(),
                body: crate::patterns::gs(caps, "BY").unwrap_or("").to_string(),
                end: crate::patterns::gs(caps, "E").unwrap_or("").to_string(),
                name: name.clone(),
                typ: "special".into(),
                yaml_name: "specialBeginEnd".into(),
                horizontal_trailing_space: crate::patterns::gs(caps, "HS")
                    .unwrap_or("")
                    .to_string(),
                linebreaks_at_end: crate::patterns::gs(caps, "LB").unwrap_or("").to_string(),
                ..Default::default()
            };
            verbatim_common_tasks(&mut obj);
            logger::trace(&format!("*VERBATIM special found: {name}"));
            let repl = obj.replacement_text.clone();
            logger::trace(&format!("replaced with ID: {}", obj.id));
            store(obj);
            repl
        });
    }
    // 处理完后这些 special 不再参与代码块识别。
    // Perl：`unless ${$_}{lookForThis} == 0` —— 数值 0 与字符串 "verbatim"
    // （数值化也是 0）都删除；只删 "verbatim" 会让 `lookForThis: 0` 的条目留在列表里，
    // 之后按 begin 搜索做名字回查时抢先命中（specialAlgo：默认 inlineMath 的 `$`
    // 匹配到 `\For{...$...}` 内部 → 整块被命名为 inlineMath，MIDDLE/trace 全错位）。
    logger::trace("*specialBeginEnd removing any elements with lookForThis: 0");
    state::settings_mut(|s| {
        if let Some(seq) = crate::settings::get_mut_special_seq(&mut s.root) {
            seq.retain(|item| {
                item.as_mapping()
                    .map(|m| as_bool(get(m, "lookForThis")))
                    .unwrap_or(false)
            });
        }
    });
    body
}

/// put_verbatim_back_in：token 还原。kind = "everything-except-commands" | "just-commands"
pub fn put_verbatim_back_in(body: &str, kind: &str) -> String {
    let mut body = body.to_string();
    let to_match = if kind == "just-commands" {
        "command".to_string()
    } else if kind == "preamble" {
        "preamble".to_string()
    } else {
        "noindentblockenvironmentspeciallinesprotectpreamble".to_string()
    };
    let mut restored_any = false;
    loop {
        let ids: Vec<(String, String)> = state::storage(|s| {
            s.verbatim
                .iter()
                .filter(|(_, v)| to_match.contains(v.typ.as_str()))
                .map(|(k, _)| (k.clone(), String::new()))
                .collect()
        });
        let mut restored = false;
        for (id, _) in ids {
            if body.contains(&id) {
                let obj = state::storage(|s| s.verbatim.remove(&id));
                if let Some(obj) = obj {
                    // Perl Verbatim.pm L517-524：EndFinishesWithLineBreak == -1 时，
                    // 还原前先把占位符右侧的换行（含行首空白）压掉，只留一个空格，
                    // 这样 begin/body/end 拼回来后 `\end{...}` 与原同行后续文本重新接在同一行。
                    // 判据 `$id\h*\R` 与 Perl 完全一致：没有换行就不动。
                    if crate::switches::with(|s| s.is_m_switch_active())
                        && obj.end_finishes_with_line_break == -1
                    {
                        let probe =
                            crate::patterns::re_new(&format!(r"(?s){}\h*\R", regex_esc(&id)))
                                .unwrap();
                        if probe.is_match(body.as_bytes()).unwrap_or(false) {
                            let re = crate::patterns::re_new(&format!(
                                r"(?s){}(\h*)?(\R|\h)*",
                                regex_esc(&id)
                            ))
                            .unwrap();
                            let fixed = format!("{id} ");
                            body = crate::patterns::replace_all_with(&body, &re, |_| fixed.clone());
                        }
                    }
                    if obj.typ == "linesprotect" {
                        let re = crate::patterns::re_new(&format!(r"(?m)^\h*{}", regex_esc(&id)))
                            .unwrap();
                        body = crate::patterns::replace_all_with(&body, &re, |_| id.clone());
                        let re =
                            crate::patterns::re_new(&format!(r"(?m)({})\h*(\S)", regex_esc(&id)))
                                .unwrap();
                        body = crate::patterns::replace_all_with(&body, &re, |caps| {
                            format!(
                                "{}\n{}",
                                crate::patterns::gn(caps, 1),
                                crate::patterns::gn(caps, 2)
                            )
                        });
                    }
                    // Perl Verbatim.pm L545：
                    //   `${$self}{body} =~ s/$verbatimID/${begin}${body}${end}/s;`
                    body = body.replace(&id, &format!("{}{}{}", obj.begin, obj.body, obj.end));
                    restored = true;
                    restored_any = true;
                }
            }
        }
        if !restored {
            break;
        }
    }
    if restored_any {
        // 后端缀里含 m-switch 换行 token：还原后统一归一（与 Perl 收尾的全局 adjust 等价）
        body = crate::mlb::line_break_token_adjust(&body);
    }
    let _ = restored_any;
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

/// _mlb_verbatim：verbatim 块的 -m 换行调整（BeginStartsOnOwnLine/EndFinishesWithLineBreak）。
pub fn mlb_verbatim(body: &str, when: &str) -> String {
    let is_m = switches::with(|s| s.is_m_switch_active());
    if !is_m {
        return body.to_string();
    }
    let t = tok(|t| t.clone());
    let mut body = body.to_string();
    let children: Vec<(String, String, String)> = state::storage(|s| {
        s.verbatim
            .iter()
            .map(|(k, v)| (k.clone(), v.yaml_name.clone(), v.name.clone()))
            .collect()
    });
    for (id, yaml_name, name) in children {
        let poly = |k: &str| -> Option<i64> {
            state::settings(|s| s.poly_switch_value(&yaml_name, &name, k))
        };
        if let Some(vb) = poly("BeginStartsOnOwnLine") {
            let values: Vec<i64> = if vb == 4 { vec![-1, 3] } else { vec![vb] };
            for v in values {
                // Perl 在 foreach 每次迭代里**重新**求值 `m/^\h*id/m`：
                // `*4` 展开为 (-1, 3) 时，-1 分支先删掉前导换行，
                // 随后 3 分支必须看到「已不在行首」才会补空行+换行。
                // 若只算一次（旧实现），3 分支会误判仍在行首而整段跳过。
                let at_line_start = crate::patterns::re_new(&format!(r"(?m)^\h*{}", regex_esc(&id)))
                    .unwrap()
                    .is_match(body.as_bytes())
                    .unwrap_or(false);
                if v == -1 {
                    if at_line_start {
                        let re =
                            crate::patterns::re_new(&format!(r"(?s)(\R|\h)*{}", regex_esc(&id)))
                                .unwrap();
                        body = crate::patterns::replace_all_with(&body, &re, |_| id.clone());
                    }
                } else if v >= 1 && !at_line_start {
                    let token = match v {
                        2 => format!("%{}", crate::trailing_comments::add_comment_symbol("")),
                        3 => "\n".to_string(),
                        _ => String::new(),
                    };
                    let re =
                        crate::patterns::re_new(&format!(r"(?s)\h*{}", regex_esc(&id))).unwrap();
                    body =
                        crate::patterns::replace_all_with(&body, &re, |_| format!("{token}\n{id}"));
                }
            }
        }

        // after text wrap poly-switch check（Perl ModifyLineBreaks.pm L471-509）
        //
        // 折行会把 `\end{...}` 后面的换行换成空格，于是占位符变成 `ID after` 挤在同一行。
        // 此时 Perl 会**再补一次**：`linebreaksAtEnd{end}=1`、削掉 `end` 的尾空白，
        // 并在 body 的 ID 后插入换行（判据 `body =~ /ID\h*\S+/m`：ID 后面还跟着非空白）。
        // 少了这一步，`verbatim4-mod1` 会把 `\end{commandshell}` 与 `after` 拼成一行。
        if when == "afterTextWrap" {
            if let Some(ef) = poly("EndFinishesWithLineBreak") {
                if ef >= 1 {
                    let tail_re =
                        crate::patterns::re_new(&format!(r"(?m){}\h*\S+", regex_esc(&id))).unwrap();
                    if tail_re.is_match(body.as_bytes()).unwrap_or(false) {
                        let mut tc_token = String::new();
                        let mut lb_char = String::new();
                        if ef == 1 {
                            lb_char = "\n".to_string();
                        } else if ef == 2 {
                            let tc = state::patterns(|p| p.trailing_comment.clone());
                            let re_tc = crate::patterns::re_new(&format!(
                                r"(?s){}\h*{}",
                                regex_esc(&id),
                                tc
                            ))
                            .unwrap();
                            if !re_tc.is_match(body.as_bytes()).unwrap_or(false) {
                                tc_token = format!(
                                    "%{}\n",
                                    crate::trailing_comments::add_comment_symbol("")
                                );
                            }
                        } else if ef == 3 {
                            lb_char = if crate::mlb::preserve_blank_lines() {
                                format!("{}\n", t.blanklines)
                            } else {
                                "\n".to_string()
                            };
                        }
                        // `linebreaksAtEnd{end} = 1` + `end =~ s/\s*$//s`
                        state::storage(|s| {
                            if let Some(o) = s.verbatim.get_mut(&id) {
                                o.linebreaks_at_end = "1".to_string();
                                while o.end.ends_with(|c: char| c.is_whitespace()) {
                                    o.end.pop();
                                }
                            }
                        });
                        let re = crate::patterns::re_new(&format!(
                            r"(?s){}(\h*)",
                            regex_esc(&id)
                        ))
                        .unwrap();
                        body = crate::patterns::replace_all_with(&body, &re, |c| {
                            format!("{}{}{}{}", id, crate::patterns::gn(c, 1), tc_token, lb_char)
                        });
                    }
                }
            }
        }
    }
    // 收尾（顺序固定）
    let re = crate::patterns::re_new(&format!(r"(?s)({})\h+", t.m_after_end_remove)).unwrap();
    body = crate::patterns::replace_all_with(&body, &re, |c| {
        format!("{} ", crate::patterns::gn(c, 1))
    });
    crate::mlb::line_break_token_adjust(&body)
}

/// lines 保护块（-n 选择行时用）。
pub fn create_line_block(lines: &[String], start: usize, finish: usize) -> String {
    // Perl: join 后 s/\R?$// —— 只剥一个行尾换行
    let mut verb_body: String = if start <= finish && finish < lines.len() {
        lines[start..=finish].concat()
    } else {
        String::new()
    };
    if verb_body.ends_with("\r\n") {
        verb_body.truncate(verb_body.len() - 2);
    } else if verb_body.ends_with('\n') || verb_body.ends_with('\r') {
        verb_body.truncate(verb_body.len() - 1);
    }
    let mut obj = VerbatimObj {
        begin: String::new(),
        body: verb_body,
        end: String::new(),
        name: "line-switch-verbatim-protection".into(),
        typ: "linesprotect".into(),
        yaml_name: "lines-not-to-be-indented".into(),
        ..Default::default()
    };
    obj.id = create_unique_id(&obj.typ);
    let id = obj.id.clone();
    store(obj);
    id
}

/// 供 file_contents 使用的 verbatim 对象创建
pub fn make_verbatim_obj(
    begin: String,
    body: String,
    end: String,
    name: String,
    typ: String,
    yaml_name: String,
    linebreaks_at_end: String,
) -> VerbatimObj {
    let mut obj = VerbatimObj {
        begin,
        body,
        end,
        name,
        typ,
        yaml_name,
        linebreaks_at_end,
        ..Default::default()
    };
    verbatim_common_tasks(&mut obj);
    obj
}
