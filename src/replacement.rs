//! 替换（对应 LatexIndent/Replacement.pm）：-r / -rv / -rr 开关的
//! replacements 处理（before / after 两个时机）。

use crate::logger;
use crate::settings::{as_bool, as_scalar_string, as_str, get};
use crate::state;
use crate::switches;

/// make_replacements(when)：遍历 replacements 列表执行 this→that 或 substitution。
pub fn make_replacements(body: &str, when: &str) -> String {
    let is_rr = switches::with(|s| s.is_rr_switch_active());
    if !is_rr {
        logger::info(&format!("*Replacement mode *{when}* indentation: -r"));
    } else {
        logger::info("*Replacement mode, -rr switch is active");
    }
    let entries: Vec<serde_yaml::Value> = state::settings(|s| {
        get(&s.root, "replacements")
            .and_then(|v| v.as_sequence())
            .cloned()
            .unwrap_or_default()
    });
    let mut body = body.to_string();
    for entry in &entries {
        let m = match entry.as_mapping() {
            Some(m) => m,
            None => continue,
        };
        let this = as_scalar_string(get(m, "this"));
        let substitution = as_str(get(m, "substitution"));
        if this.is_none() && substitution.is_none() {
            continue;
        }
        let look = get(m, "lookForThis");
        if look.is_some() && !as_bool(look) {
            continue;
        }
        let mut r_when = as_str(get(m, "when")).unwrap_or("before").to_string();
        if is_rr {
            r_when = "before".to_string();
        }
        if r_when != when {
            continue;
        }
        if this.is_some() && substitution.is_some() {
            logger::warn("*replacement field contains both 'this' and 'substitution'");
            logger::warn("the 'substitution' field will be ignored");
        }
        if let Some(this) = this {
            // 字符串字面全替换（Perl: quotemeta 后 split/join）；that 缺省即删除
            let that = as_scalar_string(get(m, "that")).unwrap_or_default();
            body = body.split(&this).collect::<Vec<_>>().join(&that);
        } else if let Some(sub) = substitution {
            body = apply_substitution(&body, sub);
        }
    }
    body
}

/// 执行 `s/pat/rep/flags` 形式的 substitution（Perl eval 的近似）。
fn apply_substitution(body: &str, sub: &str) -> String {
    let text = sub.trim();
    let rest = match text.strip_prefix('s') {
        Some(r) => r,
        None => {
            logger::warn(&format!("*substitution not understood: {sub}"));
            return body.to_string();
        }
    };
    let mut chars = rest.chars();
    let delim = match chars.next() {
        Some(c) => c,
        None => return body.to_string(),
    };
    // 切出 pattern / replacement / flags（支持 \d 定界符转义）
    let mut parts: Vec<String> = vec![String::new()];
    let mut escaped = false;
    let mut flags = String::new();
    let mut phase = 0; // 0=pat, 1=rep, 2=flags
    for c in chars {
        if phase == 2 {
            flags.push(c);
            continue;
        }
        if escaped {
            parts[phase].push('\\');
            parts[phase].push(c);
            escaped = false;
            continue;
        }
        if c == '\\' {
            escaped = true;
            continue;
        }
        if c == delim {
            phase += 1;
            parts.push(String::new());
            continue;
        }
        parts[phase].push(c);
    }
    if phase < 2 {
        logger::warn(&format!("*substitution not understood: {sub}"));
        return body.to_string();
    }
    let (pat, rep) = (&parts[0], &parts[1]);
    let mut src = String::new();
    if flags.contains('s') {
        src.push_str("(?s)");
    }
    if flags.contains('m') {
        src.push_str("(?m)");
    }
    if flags.contains('x') {
        src.push_str("(?x)");
    }
    // `/i` 之前漏了：`s/.../.../gxi` 这种带大小写不敏感的标志会退化成区分大小写，
    // 于是 `Lorem`/`Nulla` 匹配不到（replacements/test7-mod14）。
    if flags.contains('i') {
        src.push_str("(?i)");
    }
    src.push_str(pat);
    let re = match crate::patterns::re_new(&src) {
        Ok(re) => re,
        Err(_) => {
            logger::warn(&format!("*substitution pattern not understood: {sub}"));
            return body.to_string();
        }
    };
    let expand = |caps: &pcre2::bytes::Captures| -> String {
        // Perl 替换串（非 /e）语义 = 双引号串：转义字符、$1 引用、\u\l\U\L 大小写
        // 修饰符、\Q...\E quotemeta（Perl 5.16+ 非 ASCII 也转义）。
        #[derive(Clone, Copy, PartialEq)]
        enum Case {
            None,
            UpNext,
            DownNext,
            UpAll,
            DownAll,
        }
        let mut out = String::new();
        let mut it = rep.chars().peekable();
        let mut case = Case::None;
        let mut quote = false;
        fn emit(out: &mut String, ch: char, case: &mut Case, quote: bool) {
            let c = match case {
                Case::UpNext => {
                    *case = Case::None;
                    ch.to_uppercase().next().unwrap_or(ch)
                }
                Case::DownNext => {
                    *case = Case::None;
                    ch.to_lowercase().next().unwrap_or(ch)
                }
                Case::UpAll => ch.to_uppercase().next().unwrap_or(ch),
                Case::DownAll => ch.to_lowercase().next().unwrap_or(ch),
                Case::None => ch,
            };
            if quote && !(c == '_' || (c.is_ascii() && c.is_alphanumeric())) {
                out.push('\\');
            }
            out.push(c);
        }
        while let Some(c) = it.next() {
            match c {
                '$' => {
                    // `$+{name}`（命名捕获，issue-517.yaml）与 `${name}` / `${1}`
                    if it.peek() == Some(&'+') || it.peek() == Some(&'{') {
                        let plus = it.peek() == Some(&'+');
                        if plus {
                            it.next();
                        }
                        let mut name = String::new();
                        if it.peek() == Some(&'{') {
                            it.next();
                            for d in it.by_ref() {
                                if d == '}' {
                                    break;
                                }
                                name.push(d);
                            }
                            if name.chars().all(|c| c.is_ascii_digit()) {
                                let idx: usize = name.parse().unwrap_or(0);
                                for ch in crate::patterns::gn(caps, idx).chars() {
                                    emit(&mut out, ch, &mut case, quote);
                                }
                            } else {
                                for ch in crate::patterns::gs(caps, &name).unwrap_or("").chars() {
                                    emit(&mut out, ch, &mut case, quote);
                                }
                            }
                        } else {
                            emit(&mut out, '$', &mut case, quote);
                        }
                        continue;
                    }
                    let mut num = String::new();
                    while let Some(&d) = it.peek() {
                        if d.is_ascii_digit() {
                            num.push(d);
                            it.next();
                        } else {
                            break;
                        }
                    }
                    if num.is_empty() {
                        emit(&mut out, '$', &mut case, quote);
                    } else {
                        let idx: usize = num.parse().unwrap_or(0);
                        for ch in crate::patterns::gn(caps, idx).chars() {
                            emit(&mut out, ch, &mut case, quote);
                        }
                    }
                }
                '\\' => match it.next() {
                    None => emit(&mut out, '\\', &mut case, quote),
                    Some(n) => {
                        // 逃逸/修饰符不被 \Q 转义（Perl：\Q 内的 \\ 产出 \）
                        match n {
                            'n' => emit(&mut out, '\n', &mut case, quote),
                            't' => emit(&mut out, '\t', &mut case, quote),
                            'r' => emit(&mut out, '\r', &mut case, quote),
                            'f' => emit(&mut out, '\u{c}', &mut case, quote),
                            'b' => emit(&mut out, '\u{8}', &mut case, quote),
                            'a' => emit(&mut out, '\u{7}', &mut case, quote),
                            'e' => emit(&mut out, '\u{1b}', &mut case, quote),
                            'x' => {
                                let mut hex = String::new();
                                if it.peek() == Some(&'{') {
                                    it.next();
                                    for d in it.by_ref() {
                                        if d == '}' {
                                            break;
                                        }
                                        hex.push(d);
                                    }
                                } else {
                                    while hex.len() < 2 {
                                        match it.peek() {
                                            Some(&d) if d.is_ascii_hexdigit() => {
                                                hex.push(d);
                                                it.next();
                                            }
                                            _ => break,
                                        }
                                    }
                                }
                                let ch = u32::from_str_radix(&hex, 16)
                                    .ok()
                                    .and_then(char::from_u32)
                                    .unwrap_or('\u{fffd}');
                                emit(&mut out, ch, &mut case, quote);
                            }
                            '0'..='7' => {
                                // 八进制（\0 开头）；\1-\9 是反向引用
                                let mut oct = String::new();
                                oct.push(n);
                                while oct.len() < 3 {
                                    match it.peek() {
                                        Some(&d) if ('0'..='7').contains(&d) => {
                                            oct.push(d);
                                            it.next();
                                        }
                                        _ => break,
                                    }
                                }
                                let ch = u32::from_str_radix(&oct, 8)
                                    .ok()
                                    .and_then(char::from_u32)
                                    .unwrap_or('\0');
                                emit(&mut out, ch, &mut case, quote);
                            }
                            '1'..='9' => {
                                for ch in crate::patterns::gn(caps, n as usize - '0' as usize).chars()
                                {
                                    emit(&mut out, ch, &mut case, quote);
                                }
                            }
                            'u' => case = Case::UpNext,
                            'l' => case = Case::DownNext,
                            'U' => case = Case::UpAll,
                            'L' => case = Case::DownAll,
                            'Q' => quote = true,
                            'E' => {
                                quote = false;
                                case = Case::None;
                            }
                            other => emit(&mut out, other, &mut case, quote),
                        }
                    }
                },
                other => emit(&mut out, other, &mut case, quote),
            }
        }
        out
    };
    if flags.contains('g') {
        crate::patterns::replace_all_with(body, &re, expand)
    } else {
        let mut done = false;
        crate::patterns::replace_all_with(body, &re, |caps| {
            if done {
                caps.get(0)
                    .map(|m| String::from_utf8_lossy(m.as_bytes()).into_owned())
                    .unwrap_or_default()
            } else {
                done = true;
                expand(caps)
            }
        })
    }
}
