//! filecontents 环境与 preamble 保护（对应 LatexIndent/FileContents.pm）。

use crate::logger;
use crate::settings::{as_bool, as_str, get};
use crate::state::{self, VerbatimObj};
use crate::tokens::tok;

/// find_file_contents_environments_and_preamble
pub fn find_file_contents_environments_and_preamble(body: &str) -> String {
    let mut body = body.to_string();

    // 1) filecontents 环境 → token
    let entries: Vec<String> = state::settings(|s| {
        get(&s.root, "fileContentsEnvironments")
            .and_then(|v| v.as_mapping())
            .map(|m| {
                m.iter()
                    .filter(|(k, v)| as_bool(Some(v)) || v.is_mapping())
                    .map(|(k, _)| k.as_str().unwrap_or_default().to_string())
                    .collect()
            })
            .unwrap_or_default()
    });
    for name in entries {
        logger::trace(&format!("looking for {name} environments"));
        let n = if name.ends_with('*') {
            format!("{}\\*", &name[..name.len() - 1])
        } else {
            name.replace('*', r"\*")
        };
        let src = format!(
            r"(?s)(?<B>\\begin\{{(?<N>{n})\}})(?<BY>.*?)(?<E>\\end\{{\k<N>\}}\h*)(?<LB>\R)?",
            n = n
        );
        let re = match crate::patterns::re_new(&src) {
            Ok(re) => re,
            Err(_) => continue,
        };
        body = crate::patterns::replace_all_with(&body, &re, |caps| {
            let lb = crate::patterns::gs(caps, "LB").unwrap_or("").to_string();
            let t = tok(|t| t.clone());
            let counter = state::storage(|s| s.filecontents.len() + 1);
            let id = format!("{}{}{}", t.filecontents, counter, t.end_of_token);
            state::storage(|s| {
                s.filecontents.push(VerbatimObj {
                    begin: crate::patterns::gs(caps, "B").unwrap_or("").to_string(),
                    body: crate::patterns::gs(caps, "BY").unwrap_or("").to_string(),
                    end: crate::patterns::gs(caps, "E").unwrap_or("").to_string(),
                    name: name.clone(),
                    typ: "filecontents".into(),
                    yaml_name: "filecontents".into(),
                    id: id.clone(),
                    replacement_text: id.clone(),
                    linebreaks_at_end: lb.clone(),
                    ..Default::default()
                });
            });
            if lb.is_empty() {
                id
            } else {
                format!("{id}\n")
            }
        });
    }

    // 2) preamble 保护（lookForPreamble 以 {fileExtension} 带点形式为键；
    //    STDIN 输入键为 "STDIN"：为真强制启用，为假回退空扩展名键值）
    let look_for_preamble = {
        let file = crate::document::current_file();
        let ext_key = if file == "-" {
            String::new()
        } else {
            format!(".{}", crate::document::current_extension())
        };
        state::settings(|s| {
            let empty = serde_yaml::Mapping::new();
            let map = get(&s.root, "lookForPreamble")
                .and_then(|v| v.as_mapping())
                .unwrap_or(&empty);
            let mut look = as_bool(get(map, &ext_key));
            if file == "-" && as_bool(get(map, "STDIN")) {
                look = true;
            }
            look
        })
    };
    let indent_preamble = state::settings(|s| as_bool(get(&s.root, "indentPreamble")));
    let re = crate::patterns::re_new(r"(?s)(?<P>.*?)(?<D>\R?\h*\\begin\{document\})").unwrap();
    if look_for_preamble && !indent_preamble {
        if let Some(caps) = re.captures(body.as_bytes()).unwrap() {
            logger::trace(
                "\\begin{document} found in body (after searching for filecontents)-- assuming that a preamble exists",
            );
            let pre = crate::patterns::gs(&caps, "P").unwrap_or("").to_string();
            let doc_begin = crate::patterns::gs(&caps, "D").unwrap_or("").to_string();
            // Perl：preamble 作为 verbatim 存储前先还原其中的空行 token（防止泄漏到输出）
            let mut pre_body = pre.clone();
            let is_m = crate::switches::with(|s| s.is_m_switch_active());
            let preserve = state::settings(|s| {
                as_bool(crate::settings::get_path(
                    &s.root,
                    &["modifyLineBreaks", "preserveBlankLines"],
                ))
            });
            if is_m && preserve {
                pre_body = crate::blank_lines::unprotect_blank_lines(&pre_body);
            }
            // filecontents 回填进 preamble 体
            let ids: Vec<(String, String)> = state::storage(|s| {
                s.filecontents
                    .iter()
                    .map(|o| (o.id.clone(), String::new()))
                    .collect()
            });
            for (id, _) in &ids {
                if pre_body.contains(id) {
                    let obj =
                        state::storage(|s| s.filecontents.iter().find(|o| &o.id == id).cloned());
                    if let Some(obj) = obj {
                        pre_body =
                            pre_body.replace(id, &format!("{}{}{}", obj.begin, obj.body, obj.end));
                    }
                }
            }
            let t = tok(|t| t.clone());
            let mut obj = VerbatimObj {
                begin: String::new(),
                body: pre_body,
                end: String::new(),
                name: "preamble".into(),
                typ: "preamble".into(),
                yaml_name: "preamble".into(),
                ..Default::default()
            };
            obj.id = crate::verbatim::make_verbatim_obj(
                String::new(),
                obj.body.clone(),
                String::new(),
                "preamble".into(),
                "preamble".into(),
                "preamble".into(),
                String::new(),
            )
            .id;
            let id = obj.id.clone();
            state::storage(|s| {
                s.verbatim.insert(id.clone(), obj);
            });
            logger::trace("*found: preamble, storing as verbatim (indentPreamble: 0)");
            let _ = t;
            body = body.replacen(
                &format!("{}{}", pre, doc_begin),
                &format!("{id}{doc_begin}"),
                1,
            );
        }
    }

    // 3) 未进 preamble 的 filecontents 回填
    let ids: Vec<(String, String, String, String)> = state::storage(|s| {
        s.filecontents
            .iter()
            .map(|o| (o.id.clone(), o.begin.clone(), o.body.clone(), o.end.clone()))
            .collect()
    });
    for (id, b, mid, e) in ids {
        if body.contains(&id) {
            body = body.replace(&id, &format!("{b}{mid}{e}"));
        }
    }
    body
}
