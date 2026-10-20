//! YAML 设置系统（对应 LatexIndent/GetYamlSettings.pm）：
//! defaultSettings.yaml + indentconfig 用户路径 + -l 本地文件 + -y 覆盖，
//! 以及缩进信息/poly-switch/对齐设置的查询与缓存。

use crate::logger;
use serde_yaml::{Mapping, Value};
use std::collections::HashMap;

pub const DEFAULT_SETTINGS_EMBEDDED: &str = include_str!("../assets/defaultSettings.yaml");

/// 单个代码块对象的设置缓存（对应 %previouslyFoundSetting 条目）。
#[derive(Default, Clone, Debug)]
pub struct ObjectSettings {
    pub indentation: String,
    pub mandatory_arguments_indentation: String,
    pub optional_arguments_indentation: String,
    pub look_for_align_delims: bool,
    pub align: HashMap<String, Value>,
    pub poly: HashMap<String, i64>,
    pub mand: HashMap<String, i64>,
    pub opt: HashMap<String, i64>,
}

pub struct MainSettings {
    pub root: Mapping,
    pub special_lookup_middle: HashMap<String, String>,
    pub previously_found: HashMap<String, ObjectSettings>,
}

// ---------------------------------------------------------------------------
// 取值助手
// ---------------------------------------------------------------------------

pub fn get<'a>(map: &'a Mapping, key: &str) -> Option<&'a Value> {
    if let Some(v) = map.get(Value::String(key.to_string())) {
        return Some(v);
    }
    // 纯数字键回落：YAML 里 `indentRules://n  1: "   "` 的键被 serde_yaml 解析成
    // `Number(1)`，而 Perl 的 YAML::Tiny 一律按字符串 "1" 存。找 "1" 时两种都要试，
    // 否则 `indentRules{1: "   "}` 静默失效、退回 defaultIndent
    // （items/issue-518：item body 只拿到 1 空格而非 3 空格）。
    if let Ok(n) = key.parse::<i64>() {
        return map.get(Value::Number(n.into()));
    }
    None
}

pub fn get_path<'a>(map: &'a Mapping, path: &[&str]) -> Option<&'a Value> {
    let mut cur = get(map, *path.first()?)?;
    for k in &path[1..] {
        cur = cur.as_mapping().and_then(|m| get(m, k))?;
    }
    Some(cur)
}

pub fn as_str(v: Option<&Value>) -> Option<&str> {
    v.and_then(|v| v.as_str())
}

/// YAML 标量 → 字符串（Perl 的 YAML::Tiny 把所有标量都当字符串存，
/// 数字/布尔也是）。`replacements: {this: one, that: 1}` 里的 `that` 是
/// 数字标量，Perl 插值后得 "1"；只认 `as_str` 会把它当缺失 → 替换成空串。
pub fn as_scalar_string(v: Option<&Value>) -> Option<String> {
    match v? {
        Value::String(s) => Some(s.clone()),
        Value::Number(n) => Some(n.to_string()),
        Value::Bool(b) => Some(if *b { "true" } else { "false" }.to_string()),
        _ => None,
    }
}

pub fn as_bool(v: Option<&Value>) -> bool {
    match v {
        Some(Value::Bool(b)) => *b,
        Some(Value::Number(n)) => n.as_i64().unwrap_or(0) != 0,
        Some(Value::String(s)) => s != "0" && !s.is_empty(),
        Some(Value::Null) | None => false,
        _ => true,
    }
}

pub fn as_i64(v: Option<&Value>) -> Option<i64> {
    match v {
        Some(Value::Number(n)) => n.as_i64(),
        Some(Value::String(s)) => s.trim().parse().ok(),
        Some(Value::Bool(b)) => Some(*b as i64),
        _ => None,
    }
}

fn is_hspace(s: &str) -> bool {
    !s.is_empty() && s.chars().all(|c| c == ' ' || c == '\t')
}

/// YAML::Tiny::Dump 兼容序列化：键按字典序、标量一律按字符串按需单引号。
fn yaml_tiny_dump(v: &Value) -> String {
    let mut out = String::from("---\n");
    yaml_tiny_write(v, 0, &mut out);
    out
}

fn yaml_tiny_scalar(s: &str) -> String {
    let plain_ok = !s.is_empty()
        && !s.starts_with([' ', '\t'])
        && !s.ends_with([' ', '\t'])
        && !s.starts_with([
            '-', '?', '[', ']', '{', '}', ',', '&', '*', '!', '|', '>', '\'', '"', '%', '@', '`',
            '#', ':',
        ])
        && !s.contains(": ")
        && !s.contains(" #")
        && s.parse::<f64>().is_err()
        && !matches!(
            s.to_ascii_lowercase().as_str(),
            "true" | "false" | "yes" | "no" | "on" | "off" | "null" | "~" | "y" | "n"
        );
    if plain_ok {
        s.to_string()
    } else {
        format!("'{}'", s.replace('\'', "''"))
    }
}

fn yaml_tiny_inline(v: &Value) -> String {
    match v {
        Value::String(s) => yaml_tiny_scalar(s),
        Value::Number(n) => yaml_tiny_scalar(&n.to_string()),
        Value::Bool(b) => yaml_tiny_scalar(if *b { "true" } else { "false" }),
        Value::Null => "''".to_string(),
        _ => String::new(),
    }
}

fn yaml_tiny_write(v: &Value, indent: usize, out: &mut String) {
    let m = match v {
        Value::Mapping(m) => m,
        _ => return,
    };
    let mut entries: Vec<(&Value, &Value)> = m.iter().collect();
    entries.sort_by_key(|(k, _)| k.as_str().unwrap_or_default().to_string());
    for (k, val) in entries {
        let pad = " ".repeat(indent);
        let ks = yaml_tiny_scalar(k.as_str().unwrap_or_default());
        match val {
            Value::Mapping(mm) if !mm.is_empty() => {
                out.push_str(&format!("{pad}{ks}:\n"));
                yaml_tiny_write(val, indent + 2, out);
            }
            Value::Sequence(seq) if !seq.is_empty() => {
                out.push_str(&format!("{pad}{ks}:\n"));
                for item in seq {
                    out.push_str(&format!("{pad}  - {}\n", yaml_tiny_inline(item)));
                }
            }
            Value::Mapping(_) | Value::Sequence(_) => {
                out.push_str(&format!("{pad}{ks}: {{}}\n"));
            }
            _ => out.push_str(&format!("{pad}{ks}: {}\n", yaml_tiny_inline(val))),
        }
    }
}

/// YAML::Tiny 语义兼容改写：
/// - `name: [a-zA-Z]+` 之类的正则值 YAML::Tiny 按字面字符串读取，serde_yaml 却当作流
///   序列/流映射（`[_^*#0-9.]` 这类合法流序列尤其会静默改变语义），故以 [ / { 开头的
///   裸值一律补引号；
/// - `@` / `` ` `` 是 YAML 保留指示符，裸标量（key 或 value）会被 serde_yaml 拒绝，
///   而 YAML::Tiny 按字面量接受（如 noAdditionalIndent: `@online`），同样补引号。
fn yaml_tiny_rewrite(content: &str) -> String {
    let quote_scalar = |s: &str| -> Option<String> {
        let t = s.trim();
        if t.is_empty() || t.starts_with('\'') || t.starts_with('"') {
            return None;
        }
        if t.starts_with('@') || t.starts_with('`') || t.starts_with('[') || t.starts_with('{') {
            Some(t.replace('\'', "''"))
        } else {
            None
        }
    };
    let mut out: Vec<String> = Vec::new();
    let mut block_indent: Option<usize> = None;
    for line in content.split('\n') {
        let indent = line.len() - line.trim_start().len();
        if let Some(bi) = block_indent {
            if line.trim().is_empty() || indent > bi {
                out.push(line.to_string());
                continue;
            }
            block_indent = None;
        }
        let trimmed = line.trim_start();
        let (prefix, rest) = if trimmed.starts_with("- ") {
            (&line[..indent + 2], &trimmed[2..])
        } else {
            (&line[..indent], trimmed)
        };
        let (key_part, val_part, sep) = match rest.find(": ") {
            Some(i) => (Some(&rest[..i]), &rest[i + 2..], ": "),
            None if rest.ends_with(':') => (Some(&rest[..rest.len() - 1]), "", ":"),
            None => (None, rest, ""),
        };
        // 块标量指示符（| / > 变体，可带缩进/吞没修饰）：其内容行不可改写
        // 注意指示符后可跟行尾注释（如 `key: |-  # comment`），先剥注释再判
        let vhead_raw = val_part.trim();
        let vhead = match vhead_raw.find('#') {
            Some(h) => vhead_raw[..h].trim_end(),
            None => vhead_raw,
        };
        let vhead = vhead.trim_end_matches(|c: char| c == '+' || c == '-' || c.is_ascii_digit());
        if key_part.is_some()
            && (vhead.starts_with('|') || vhead.starts_with('>'))
            && vhead[1..].trim().is_empty()
        {
            block_indent = Some(indent);
        }
        // key 改写（@ / ` 保留指示符）
        let mut new_line = match key_part {
            Some(k) => match quote_scalar(k) {
                Some(q) => format!("{prefix}'{q}'{sep}"),
                None => format!("{prefix}{k}{sep}"),
            },
            None => prefix.to_string(),
        };
        // value 改写
        let vt = val_part.trim_start();
        let vws = &val_part[..val_part.len() - vt.len()];
        let (scalar, comment) = match vt.find(" #") {
            Some(i) => (&vt[..i], &vt[i..]),
            None => (vt, ""),
        };
        if let Some(q) = quote_scalar(scalar) {
            new_line.push_str(&format!("{vws}'{q}'{comment}"));
        } else if let Some(fixed) = repair_dq_escapes(scalar) {
            // 双引号标量里出现 YAML 未定义的转义（正则里的 `\(`、`\.` 等）时，
            // serde_yaml 会整份文件报错，而 Perl 的 YAML::Tiny 照读不误。
            // 把非法转义的反斜杠自身转义（`\(` → `\\(`），得到与 Perl 相同的字符串。
            new_line.push_str(&format!("{vws}{fixed}{comment}"));
        } else {
            new_line.push_str(val_part);
        }
        out.push(new_line);
    }
    out.join("\n")
}

/// 修复双引号标量中的非法 YAML 转义。
///
/// 返回 `None` 表示无需修复（不是完整的双引号标量，或全部转义都合法）。
/// Perl 的 YAML::Tiny 对 `"\([A-Z]"`、`"(?:\.\)(?!\h*[a-z]))"` 这种写法一律照读，
/// serde_yaml 则直接判整份文件格式错误（`-l=` 文件被静默丢弃 → 设置全用默认值）。
fn repair_dq_escapes(scalar: &str) -> Option<String> {
    let t = scalar.trim();
    if !t.starts_with('"') || !t.ends_with('"') || t.len() < 2 {
        return None;
    }
    let inner = &t[1..t.len() - 1];
    let bytes = inner.as_bytes();
    let mut out = String::with_capacity(inner.len() + 8);
    let mut changed = false;
    let mut i = 0;
    while i < bytes.len() {
        let c = bytes[i];
        if c != b'\\' {
            out.push(c as char);
            i += 1;
            continue;
        }
        // 行尾孤立的反斜杠：交给 serde 报错
        if i + 1 >= bytes.len() {
            out.push('\\');
            i += 1;
            continue;
        }
        let n = bytes[i + 1];
        let legal = matches!(
            n,
            b'0' | b'a' | b'b' | b't' | b'n' | b'v' | b'f' | b'r' | b'e' | b' ' | b'"' | b'/'
                | b'\\' | b'N' | b'_' | b'L' | b'P' | b'x' | b'u' | b'U' | b'\n' | b'\r'
        );
        if legal {
            out.push('\\');
            out.push(n as char);
        } else {
            // 非法转义：把反斜杠转义掉，保留字面反斜杠语义
            out.push_str("\\\\");
            out.push(n as char);
            changed = true;
        }
        i += 2;
    }
    if changed {
        Some(format!("\"{out}\""))
    } else {
        None
    }
}

pub fn parse_yaml_lenient(content: &str) -> Option<Mapping> {
    let try_parse = |s: &str| -> Option<Mapping> {
        serde_yaml::from_str::<Value>(s)
            .ok()
            .and_then(|v| v.as_mapping().cloned())
    };
    try_parse(&yaml_tiny_rewrite(content))
        .or_else(|| try_parse(content))
        // 最后兜底：缩进用 TAB 的 yaml（如 texexchange/pgfplotstableread.yaml
        // `lookForAlignDelims://n//tpgfplotstableread: 1`）。YAML 规范禁止 TAB 作缩进，
        // serde_yaml 直接判整份文件非法，而 Perl 的 YAML::Tiny 照收不误。
        // 把行首 TAB 换成 2 空格后重试，语义与 Perl 一致。
        .or_else(|| try_parse(&yaml_tiny_rewrite(&detab_leading(content))))
}

/// 行首（纯前导空白段）里的 TAB 换成 2 个空格；正文中的 TAB 原样保留。
fn detab_leading(content: &str) -> String {
    let mut out = String::with_capacity(content.len());
    for line in content.split_inclusive('\n') {
        let n = line
            .find(|c: char| c != ' ' && c != '\t')
            .unwrap_or(line.len());
        let (lead, rest) = line.split_at(n);
        if lead.contains('\t') {
            out.push_str(&lead.replace('\t', "  "));
        } else {
            out.push_str(lead);
        }
        out.push_str(rest);
    }
    out
}

// ---------------------------------------------------------------------------
// 深合并（对应 yaml_read_settings 中的用户设置合并规则）
// ---------------------------------------------------------------------------

/// Perl GetYamlSettings.pm 的 "specialBeginEnd backwards compatibility"：
/// 用户设置中的 HASH 形式（旧写法）转换为 ARRAY 形式（name + begin/end/nested/
/// middle/lookForThis）。必须在 deep_merge **之前**对每个用户文件执行——
/// 否则默认的 Sequence 会被用户的 Mapping 整体覆盖（deep_merge 的
/// (Some(b), Value::Mapping) 分支），默认 displayMath/inlineMath 条目全部丢失。
/// 只有值为 HASH 的条目被转换，其余（标量）条目按 Perl 语义直接跳过。
fn special_begin_end_hash_to_array(m: &mut Mapping) {
    let converted: Option<Vec<Value>> = match get(m, "specialBeginEnd") {
        Some(Value::Mapping(h)) => {
            let mut arr = Vec::new();
            for (k, v) in h {
                let Some(vm) = v.as_mapping() else { continue };
                let mut e = Mapping::new();
                e.insert(Value::String("name".into()), k.clone());
                for key in ["begin", "end", "nested", "middle", "lookForThis"] {
                    if let Some(x) = get(vm, key) {
                        e.insert(Value::String(key.into()), x.clone());
                    }
                }
                arr.push(Value::Mapping(e));
            }
            Some(arr)
        }
        _ => None,
    };
    if let Some(arr) = converted {
        m.insert(
            Value::String("specialBeginEnd".into()),
            Value::Sequence(arr),
        );
    }
}

fn deep_merge(base: &mut Mapping, user: &Mapping, parent_key: &str) {
    for (k, v) in user {
        let key = k.as_str().unwrap_or_default().to_string();
        if key == "paths" {
            continue; // 路径列表在别处展开
        }
        match (base.get_mut(k), v) {
            (Some(Value::Mapping(bm)), Value::Mapping(um)) => deep_merge(bm, um, &key),
            (Some(b), Value::Mapping(um)) => {
                // 原为标量、新值为映射：lookForAlignDelims 把旧标量转为 {delims}
                if parent_key == "lookForAlignDelims" {
                    let delims = b.clone();
                    let mut m = Mapping::new();
                    m.insert(Value::String("delims".into()), delims);
                    deep_merge(&mut m, um, &key);
                    *b = Value::Mapping(m);
                } else {
                    *b = Value::Mapping(um.clone());
                }
            }
            (Some(Value::Sequence(ba)), Value::Sequence(ua)) => {
                // 完全照 Perl GetYamlSettings.pm L518-541：
                //   1) 用户数组 [0] 若定义了 amalgamate → 写入 master[0].amalgamate，
                //      且 master 的 amalgamate 为真时 **shift 掉**用户数组的标记元素；
                //   2) master[0].amalgamate 为真 → 追加（标量去重）；否则整体覆盖。
                // 关键：amalgamate 是**跨文件累积**的状态——`-l=a,b` 里 b 的
                // `amalgamate: 1` 会把 master 的标志置 1，从而与 a 已合并的内容继续相加
                // （replacements/test7-mod2324：a 设 0 + end→END，b 设 1 + begin→BEGIN，
                //  期望两者都生效）。
                let user_flag = ua
                    .first()
                    .and_then(|v| v.as_mapping())
                    .and_then(|m| get(m, "amalgamate"))
                    .map(|v| as_bool(Some(v)));
                if let Some(f) = user_flag {
                    if let Some(Value::Mapping(m0)) = ba.first_mut() {
                        m0.insert(Value::String("amalgamate".into()), Value::Bool(f));
                    }
                }
                let master_amalg =
                    matches!(ba.first(), Some(Value::Mapping(m)) if as_bool(get(m, "amalgamate")));
                if master_amalg {
                    let skip = if user_flag.is_some() { 1 } else { 0 };
                    let mut extra: Vec<Value> = ua
                        .iter()
                        .skip(skip)
                        .filter(|e| !e.is_mapping())
                        .filter(|e| !ba.contains(e))
                        .cloned()
                        .collect();
                    let mut maps: Vec<Value> = ua.iter().skip(skip).cloned().collect();
                    ba.append(&mut maps);
                    ba.append(&mut extra);
                } else {
                    *ba = ua.clone();
                }
            }
            (Some(b), _) => *b = v.clone(),
            (None, _) => {
                base.insert(k.clone(), v.clone());
            }
        }
    }
}

// ---------------------------------------------------------------------------
// -y 命令行设置（对应 yaml_read_settings 的 -y 解析段）
// ---------------------------------------------------------------------------

/// 按 `sep` 切分，跳过反斜杠转义的分隔符。
///
/// 对 `:` 额外套用 Perl 的 `split(/(?<!(?:\\|\[)):(?!\])/)` 规则：
/// **前一个字符不是 `\` 或 `[`，且后一个字符不是 `]`**。
/// 这样 `[[:lower:]]` 里的两个 `:` 都不会被当作层级分隔符
/// （konfekt-mod2 / issue-243 的 `-y` 就靠这条规则才能正确落值）。
fn split_unescaped<'a>(s: &'a str, sep: char) -> Vec<&'a str> {
    let bytes = s.as_bytes();
    let mut out = Vec::new();
    let mut start = 0;
    let mut escaped = false;
    for (i, c) in s.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        if c == '\\' {
            escaped = true;
            continue;
        }
        if c != sep {
            continue;
        }
        if sep == ':' {
            let prev = if i == 0 { 0u8 } else { bytes[i - 1] };
            let next = bytes.get(i + 1).copied().unwrap_or(0);
            if prev == b'[' || next == b']' {
                continue;
            }
        }
        out.push(&s[start..i]);
        start = i + 1;
    }
    out.push(&s[start..]);
    out
}

/// 按 Perl 的 `split(/(?<!(?:\\))"/, $_)` 语义：取第一个未转义 `"` 之前的部分作为 key 路径，
/// 第一个与第二个未转义 `"` 之间作为 value 主体。
fn split_at_first_dq(s: &str) -> Option<(&str, &str)> {
    let bytes = s.as_bytes();
    let mut open = None;
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'"' && (i == 0 || bytes[i - 1] != b'\\') {
            match open {
                None => open = Some(i),
                Some(o) => return Some((&s[..o], &s[o + 1..i])),
            }
        }
        i += 1;
    }
    // 只有开引号没有闭引号：value 取到串尾
    open.map(|o| (&s[..o], &s[o + 1..]))
}

fn unescape_value(v: &str) -> Value {
    let v = v.trim();
    if v.len() >= 2 && v.starts_with('"') && v.ends_with('"') {
        let inner = &v[1..v.len() - 1];
        let mut s = String::new();
        let mut chars = inner.chars();
        while let Some(c) = chars.next() {
            if c == '\\' {
                match chars.next() {
                    Some('t') => s.push('\t'),
                    Some('n') => s.push('\n'),
                    Some('"') => s.push('"'),
                    Some('\\') => s.push('\\'),
                    Some(other) => {
                        s.push('\\');
                        s.push(other);
                    }
                    None => s.push('\\'),
                }
            } else {
                s.push(c);
            }
        }
        Value::String(s)
    } else if v.len() >= 2 && v.starts_with('\'') && v.ends_with('\'') {
        let inner = &v[1..v.len() - 1];
        Value::String(inner.replace("\\t", "\t").replace("\\n", "\n"))
    } else {
        Value::String(v.replace("\\:", ":"))
    }
}

pub fn set_path(root: &mut Mapping, path: &[&str], value: Value) {
    let mut cur = root;
    for (i, k) in path.iter().enumerate() {
        let key = Value::String((*k).to_string());
        if i == path.len() - 1 {
            cur.insert(key, value);
            return;
        }
        let entry = cur
            .entry(key)
            .or_insert_with(|| Value::Mapping(Mapping::new()));
        if !entry.is_mapping() {
            *entry = Value::Mapping(Mapping::new());
        }
        cur = entry.as_mapping_mut().unwrap();
    }
}

pub fn apply_yaml_overrides(root: &mut Mapping, yaml: &str) {
    // 规整分隔符周围空白
    let cleaned: String = {
        let mut out = String::new();
        let mut escaped = false;
        let chars: Vec<char> = yaml.chars().collect();
        let mut i = 0;
        while i < chars.len() {
            let c = chars[i];
            if escaped {
                out.push(c);
                escaped = false;
                i += 1;
                continue;
            }
            if c == '\\' {
                escaped = true;
                out.push(c);
                i += 1;
                continue;
            }
            if c == ',' || c == ':' || c == ';' {
                while !out.is_empty() && out.ends_with(' ') {
                    out.pop();
                }
                out.push(c);
                i += 1;
                while i < chars.len() && (chars[i] == ' ' || chars[i] == '\t') {
                    i += 1;
                }
                continue;
            }
            out.push(c);
            i += 1;
        }
        out
    };

    // 单条 key:value 链写入（无引号分支）
    fn set_plain(root: &mut Mapping, item: &str) {
        let parts = split_unescaped(item, ':');
        if parts.len() >= 2 {
            let value = unescape_value(parts[parts.len() - 1]);
            let path: Vec<String> = parts[..parts.len() - 1]
                .iter()
                .map(|s| s.trim().to_string())
                .collect();
            let refs: Vec<&str> = path.iter().map(|s| s.as_str()).collect();
            set_path(root, &refs, value);
        }
    }

    for item in split_unescaped(&cleaned, ',') {
        if item.trim().is_empty() {
            continue;
        }
        // Perl 判据是 `m/(?<!\\)"/`——**未转义**的双引号才算引号分支；
        // `other://;|//:|//"` 这类值里的 `"` 前有反斜杠，必须走普通分支。
        if let Some((head, body)) = split_at_first_dq(item) {
            // Perl GetYamlSettings.pm：取第一段做 key 路径，引号内**整体**做 value
            // （Perl 在 `;` 重组阶段就跳过了带引号的项）。
            let mut parts: Vec<String> = split_unescaped(head, ':')
                .iter()
                .map(|x| x.trim().to_string())
                // Perl 的 split 丢弃尾部空字段（head 以 `:` 结尾）
                .filter(|x| !x.is_empty())
                .collect();
            if !parts.is_empty() {
                let value = unescape_value(&format!("\"{body}\""));
                let refs: Vec<&str> = parts.iter().map(|x| x.as_str()).collect();
                set_path(root, &refs, value);
                parts.clear();
            }
        } else {
            // Perl issue #243 语义：`;` 子字段共享 root 去掉最后两级后的前缀
            //   -y=a:b:k1:v1;k2:v2  →  a:b:k1:v1 与 a:b:k2:v2
            let fields = split_unescaped(item, ';');
            if fields.len() > 1 {
                let root_parts = split_unescaped(fields[0], ':');
                let prefix = if root_parts.len() >= 2 {
                    root_parts[..root_parts.len() - 2].join(":")
                } else {
                    String::new()
                };
                for (i, f) in fields.iter().enumerate() {
                    let piece = if i == 0 {
                        f.to_string()
                    } else {
                        format!("{prefix}:{f}")
                    };
                    set_plain(root, &piece);
                }
            } else {
                set_plain(root, item);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// 主设置对象
// ---------------------------------------------------------------------------

/// poly-switch 别名表（对应 %polySwitchNames）。
pub fn poly_switch_alias(yaml_name: &str, name: &str) -> String {
    match yaml_name {
        "ifElseFi" => match name {
            "BeginStartsOnOwnLine" => "IfStartsOnOwnLine".into(),
            "EndStartsOnOwnLine" => "FiStartsOnOwnLine".into(),
            "EndFinishesWithLineBreak" => "FiFinishesWithLineBreak".into(),
            other => other.to_string(),
        },
        "commands" => match name {
            "BeginStartsOnOwnLine" => "CommandStartsOnOwnLine".into(),
            other => other.to_string(),
        },
        "optionalArguments" => match name {
            "BeginStartsOnOwnLine" => "LSqBStartsOnOwnLine".into(),
            "BodyStartsOnOwnLine" => "OptArgBodyStartsOnOwnLine".into(),
            "EndStartsOnOwnLine" => "RSqBStartsOnOwnLine".into(),
            "EndFinishesWithLineBreak" => "RSqBFinishesWithLineBreak".into(),
            other => other.to_string(),
        },
        "mandatoryArguments" => match name {
            "BeginStartsOnOwnLine" => "LCuBStartsOnOwnLine".into(),
            "BodyStartsOnOwnLine" => "MandArgBodyStartsOnOwnLine".into(),
            "EndStartsOnOwnLine" => "RCuBStartsOnOwnLine".into(),
            "EndFinishesWithLineBreak" => "RCuBFinishesWithLineBreak".into(),
            other => other.to_string(),
        },
        "keyEqualsValuesBracesBrackets" => match name {
            "BeginStartsOnOwnLine" => "KeyStartsOnOwnLine".into(),
            other => other.to_string(),
        },
        "namedGroupingBracesBrackets" => match name {
            // Perl GetYamlSettings.pm L1104-1107：这个 yaml 组只有两个别名，
            // 且 `NameFinishesWithLineBreak` 对应的是 **BodyStartsOnOwnLine**（不是 End…）。
            // 旧表把 EndFinishesWithLineBreak 映射过去，于是 `NameFinishesWithLineBreak: -1`
            // 取不到值、`Vo` 与 `{` 之间的空行没被消掉（contributors-remove-line-breaks-mod3）。
            "BeginStartsOnOwnLine" => "NameStartsOnOwnLine".into(),
            "BodyStartsOnOwnLine" => "NameFinishesWithLineBreak".into(),
            other => other.to_string(),
        },
        "items" => match name {
            "BeginStartsOnOwnLine" => "ItemStartsOnOwnLine".into(),
            "BodyStartsOnOwnLine" => "ItemFinishesWithLineBreak".into(),
            other => other.to_string(),
        },
        "verbatim" => match name {
            "BeginStartsOnOwnLine" => "VerbatimBeginStartsOnOwnLine".into(),
            "EndFinishesWithLineBreak" => "VerbatimEndFinishesWithLineBreak".into(),
            other => other.to_string(),
        },
        "specialBeginEnd" => match name {
            "BeginStartsOnOwnLine" => "SpecialBeginStartsOnOwnLine".into(),
            "BodyStartsOnOwnLine" => "SpecialBodyStartsOnOwnLine".into(),
            "EndStartsOnOwnLine" => "SpecialEndStartsOnOwnLine".into(),
            "EndFinishesWithLineBreak" => "SpecialEndFinishesWithLineBreak".into(),
            other => other.to_string(),
        },
        _ => name.to_string(),
    }
}

/// 对齐设置 17 字段（对应 @alignAtAmpersandInformation）。
pub const ALIGN_FIELDS: &[(&str, &str)] = &[
    ("lookForAlignDelims", "delims"),
    ("alignDoubleBackSlash", "alignDoubleBackSlash"),
    ("spacesBeforeDoubleBackSlash", "spacesBeforeDoubleBackSlash"),
    ("multiColumnGrouping", "multiColumnGrouping"),
    ("alignRowsWithoutMaxDelims", "alignRowsWithoutMaxDelims"),
    ("spacesBeforeAmpersand", "spacesBeforeAmpersand"),
    ("spacesAfterAmpersand", "spacesAfterAmpersand"),
    ("justification", "justification"),
    ("alignFinalDoubleBackSlash", "alignFinalDoubleBackSlash"),
    ("dontMeasure", "dontMeasure"),
    ("delimiterRegEx", "delimiterRegEx"),
    ("delimiterJustification", "delimiterJustification"),
    ("leadingBlankColumn", "leadingBlankColumn"),
    ("lookForChildCodeBlocks", "lookForChildCodeBlocks"),
    (
        "alignContentAfterDoubleBackSlash",
        "alignContentAfterDoubleBackSlash",
    ),
    ("spacesAfterDoubleBackSlash", "spacesAfterDoubleBackSlash"),
    ("doubleBackSlash", "doubleBackSlash"),
];

impl MainSettings {
    pub fn new(mut root: Mapping) -> MainSettings {
        // specialBeginEnd：HASH → ARRAY 兼容 + 缺省 lookForThis + middle 汇总
        if let Some(Value::Mapping(h)) = get(&root, "specialBeginEnd").cloned() {
            let mut arr = Vec::new();
            for (k, v) in h {
                let mut m = Mapping::new();
                m.insert(Value::String("name".into()), k);
                if let Some(vm) = v.as_mapping() {
                    for key in ["begin", "end", "nested", "middle", "lookForThis"] {
                        if let Some(x) = get(vm, key) {
                            m.insert(Value::String(key.into()), x.clone());
                        }
                    }
                }
                arr.push(Value::Mapping(m));
            }
            root.insert(
                Value::String("specialBeginEnd".into()),
                Value::Sequence(arr),
            );
        }
        // specialBeginEnd: amalgamate duplicates（同名后条目并入先出现者，再删除；
        // 对应 GetYamlSettings.pm 的 specialBeginEnd: amalgamate duplicates 段）
        if let Some(Value::Sequence(seq)) = get(&root, "specialBeginEnd").cloned() {
            let mut first_index: HashMap<String, usize> = HashMap::new();
            let mut merged: Vec<Value> = seq.clone();
            let mut remove: Vec<usize> = Vec::new();
            for (i, item) in seq.iter().enumerate() {
                let m = match item.as_mapping() {
                    Some(m) => m,
                    None => continue,
                };
                let name = match as_str(get(m, "name")) {
                    Some(n) => n.to_string(),
                    None => continue,
                };
                if let Some(&oi) = first_index.get(&name) {
                    logger::trace(&format!("*specialBeginEnd amalgamating {name} settings"));
                    for field in ["lookForThis", "begin", "end", "nested", "middle"] {
                        if let Some(v) = get(m, field) {
                            if let Some(om) = merged[oi].as_mapping_mut() {
                                om.insert(Value::String(field.into()), v.clone());
                            }
                        }
                    }
                    remove.push(i);
                } else {
                    first_index.insert(name, i);
                }
            }
            for i in remove.into_iter().rev() {
                merged.remove(i);
            }
            root.insert(
                Value::String("specialBeginEnd".into()),
                Value::Sequence(merged),
            );
        }
        let mut special_lookup_middle = HashMap::new();
        if let Some(Value::Sequence(seq)) = get(&root, "specialBeginEnd") {
            for item in seq {
                if let Some(m) = item.as_mapping() {
                    if let (Some(name), Some(mid)) = (as_str(get(m, "name")), get(m, "middle")) {
                        let joined = match mid {
                            Value::Sequence(s) => s
                                .iter()
                                .filter_map(|x| x.as_str())
                                .collect::<Vec<_>>()
                                .join("|"),
                            other => as_str(Some(other)).unwrap_or_default().to_string(),
                        };
                        if !joined.is_empty() {
                            special_lookup_middle.insert(name.to_string(), joined);
                        }
                    }
                }
            }
        }
        // 缺省 lookForThis：缺 begin/end/name 的条目置 0，其余默认 1
        if let Some(seq) = get_mut_seq(&mut root) {
            for item in seq {
                if let Some(m) = item.as_mapping_mut() {
                    let has_core = ["name", "begin", "end"].iter().all(|k| get(m, k).is_some());
                    if !has_core {
                        m.insert(Value::String("lookForThis".into()), Value::Bool(false));
                    } else if get(m, "lookForThis").is_none() {
                        m.insert(Value::String("lookForThis".into()), Value::Bool(true));
                    }
                }
            }
        }
        MainSettings {
            root,
            special_lookup_middle,
            previously_found: HashMap::new(),
        }
    }

    /// yaml_get_indentation_information
    pub fn indentation_information(&self, name: &str, thing: &str, yaml_name: &str) -> String {
        let name = if name.is_empty() {
            "always-un-named"
        } else {
            name
        };
        for about in ["noAdditionalIndent", "indentRules"] {
            if let Some(entry) = get_path(&self.root, &[about, name]) {
                let val = match entry.as_mapping() {
                    Some(m) => get(m, thing),
                    None => Some(entry),
                };
                if let Some(v) = val {
                    match about {
                        "noAdditionalIndent" if as_bool(Some(v)) => return String::new(),
                        "indentRules" => {
                            if let Some(s) = as_str(Some(v)) {
                                if is_hspace(s) || s.is_empty() {
                                    return s.to_string();
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        for (about, kind) in [
            ("noAdditionalIndentGlobal", "noAdditionalIndentGlobal"),
            ("indentRulesGlobal", "indentRulesGlobal"),
        ] {
            if let Some(v) = get_path(&self.root, &[about, yaml_name]) {
                match kind {
                    "noAdditionalIndentGlobal" if as_bool(Some(v)) => return String::new(),
                    "indentRulesGlobal" => {
                        if let Some(s) = as_str(Some(v)) {
                            if is_hspace(s) {
                                return s.to_string();
                            }
                            if s != "0" {
                                logger::warn(&format!(
                                    "*indentRulesGlobal->{yaml_name} has been specified, but it needs to only contain horizontal space -- I'm ignoring this one"
                                ));
                            }
                        } else if as_i64(Some(v)).unwrap_or(0) != 0 {
                            logger::warn(&format!(
                                "*indentRulesGlobal->{yaml_name} has been specified, but it needs to only contain horizontal space -- I'm ignoring this one"
                            ));
                        }
                    }
                    _ => {}
                }
            }
        }
        as_str(get(&self.root, "defaultIndent"))
            .unwrap_or("\t")
            .to_string()
    }

    /// yaml_poly_switch_get_every_or_custom_value
    pub fn poly_switch_value(&self, yaml_name: &str, obj_name: &str, switch: &str) -> Option<i64> {
        let alias = poly_switch_alias(yaml_name, switch);
        let custom = get_path(
            &self.root,
            &["modifyLineBreaks", yaml_name, obj_name, &alias],
        );
        if let Some(v) = custom {
            return as_i64(Some(v));
        }
        as_i64(get_path(
            &self.root,
            &["modifyLineBreaks", yaml_name, &alias],
        ))
    }

    /// yaml_alignment_at_ampersand_settings：填充对齐字段（存在 lookForAlignDelims[name] 时）。
    pub fn alignment_settings(&self, name: &str) -> Option<HashMap<String, Value>> {
        let entry = get_path(&self.root, &["lookForAlignDelims", name])?;
        if !as_bool(Some(entry)) && entry.as_mapping().is_none() {
            return None;
        }
        let defaults = get_path(&self.root, &["fineTuning", "lookForAlignDelimsDefaults"]);
        let mut out = HashMap::new();
        for (field, yamlname) in ALIGN_FIELDS {
            let mut val = match entry.as_mapping() {
                Some(m) => get(m, yamlname).cloned(),
                None => None,
            };
            if *field == "spacesBeforeAmpersand" {
                if let Some(Value::Mapping(sb)) = get(
                    entry.as_mapping().unwrap_or(&Mapping::new()),
                    "spacesBeforeAmpersand",
                ) {
                    if let Some(lbc) = get(sb, "leadingBlankColumn") {
                        out.insert("leadingBlankColumn".into(), lbc.clone());
                    }
                    val = get(sb, "default").cloned();
                }
            }
            let val = val.or_else(|| {
                defaults
                    .and_then(|d| d.as_mapping())
                    .and_then(|m| get(m, yamlname))
                    .cloned()
            });
            let val = val.unwrap_or_else(|| match *field {
                "leadingBlankColumn" => Value::Number((-1).into()),
                "justification" | "delimiterJustification" => Value::String("left".into()),
                "delimiterRegEx" => Value::String(r"(?<!\\)(&)".into()),
                "doubleBackSlash" => {
                    Value::String(r"\\\\(?:\h*\[\h*[0-9.]+\h*[a-zA-Z]+\h*\])?".into())
                }
                _ => Value::Number(0.into()),
            });
            if !(*field == "leadingBlankColumn" && out.contains_key("leadingBlankColumn")) {
                out.insert(field.to_string(), val);
            }
        }
        // 标量写法（`lookForAlignDelims:` + 缩进 + `tabular: 1`）时，Perl 是**整体取默认值**：
        // GetYamlSettings.pm L1547-1552 的 `else` 分支把每个字段都赋成
        // `@alignAtAmpersandInformation` 里的 default，**标量本身被忽略**（它只是个
        // 「这个对象要对齐」的标记）。所以 `lookForAlignDelims` 实际取
        // `fineTuning.lookForAlignDelimsDefaults.delims`（默认 1）。
        // 千万不要把标量当值塞进去：alignment/issue-583 用
        // `lookForAlignDelimsDefaults.delims: 0` 关掉对齐，按标量取会被反转成开。
        Some(out)
    }

    /// yaml_get_indentation_settings_for_this_object（核心缓存流程）。
    pub fn settings_for_object(
        &mut self,
        name: &str,
        yaml_name: &str,
        has_args: bool,
        is_m: bool,
    ) -> ObjectSettings {
        let storage_name = name.to_string() + yaml_name;
        if let Some(cached) = self.previously_found.get(&storage_name) {
            return cached.clone();
        }
        let thing = if yaml_name == "afterHeading" {
            "afterHeading"
        } else {
            "body"
        };
        let mut os = ObjectSettings::default();
        os.indentation = self.indentation_information(name, thing, yaml_name);
        if let Some(align) = self.alignment_settings(name) {
            // Perl：$lookForAlignDelims = previouslyFound{lookForAlignDelims}（即 delims 键），
            // 映射形 delims: 0 虽带其它对齐设置也整体关闭对齐
            os.look_for_align_delims = as_bool(align.get("lookForAlignDelims"));
            os.align = align;
        }
        if is_m {
            let mut switches = vec![
                "BeginStartsOnOwnLine",
                "BodyStartsOnOwnLine",
                "EndStartsOnOwnLine",
                "EndFinishesWithLineBreak",
                "DBSStartsOnOwnLine",
                "DBSFinishesWithLineBreak",
            ];
            if yaml_name == "ifElseFi" {
                switches.extend([
                    "ElseStartsOnOwnLine",
                    "ElseFinishesWithLineBreak",
                    "OrStartsOnOwnLine",
                    "OrFinishesWithLineBreak",
                ]);
            }
            if yaml_name == "specialBeginEnd" {
                switches.extend([
                    "SpecialMiddleStartsOnOwnLine",
                    "SpecialMiddleFinishesWithLineBreak",
                ]);
            }
            if yaml_name == "keyEqualsValuesBracesBrackets" {
                switches.extend(["EqualsStartsOnOwnLine", "EqualsFinishesWithLineBreak"]);
            }
            for switch in switches {
                if let Some(v) = self.poly_switch_value(yaml_name, name, switch) {
                    os.poly.insert(switch.to_string(), v);
                }
            }
        }
        if has_args {
            os.mandatory_arguments_indentation =
                self.indentation_information(name, "mandatoryArguments", "mandatoryArguments");
            os.optional_arguments_indentation =
                self.indentation_information(name, "optionalArguments", "optionalArguments");
            if is_m {
                for switch in [
                    "LCuBStartsOnOwnLine",
                    "MandArgBodyStartsOnOwnLine",
                    "RCuBStartsOnOwnLine",
                    "RCuBFinishesWithLineBreak",
                    "CommaStartsOnOwnLine",
                    "CommaFinishesWithLineBreak",
                    "DBSStartsOnOwnLine",
                    "DBSFinishesWithLineBreak",
                ] {
                    if let Some(v) = self.poly_switch_value("mandatoryArguments", name, switch) {
                        os.mand.insert(switch.to_string(), v);
                    }
                }
                for switch in [
                    "LSqBStartsOnOwnLine",
                    "OptArgBodyStartsOnOwnLine",
                    "RSqBStartsOnOwnLine",
                    "RSqBFinishesWithLineBreak",
                    "CommaStartsOnOwnLine",
                    "CommaFinishesWithLineBreak",
                    "DBSStartsOnOwnLine",
                    "DBSFinishesWithLineBreak",
                ] {
                    if let Some(v) = self.poly_switch_value("optionalArguments", name, switch) {
                        os.opt.insert(switch.to_string(), v);
                    }
                }
            }
        }
        self.previously_found.insert(storage_name, os.clone());
        // Perl yaml_get_indentation_settings_for_this_object：`-m` + `when: after` 时把
        // 「按名生效的自定义缩进」另存到 `{textWrapAfter}{$name}`（供 text_wrap 的
        // indentRules 分支使用）。判定与 Perl 的 $self->{indentRule} 一致：
        // 只认 per-name 的 noAdditionalIndent / indentRules（global 不记录）。
        if is_m
            && as_str(get_path(
                &self.root,
                &["modifyLineBreaks", "textWrapOptions", "when"],
            )) == Some("after")
        {
            let lookup = if name.is_empty() { "always-un-named" } else { name };
            let rule_indent: Option<String> = if let Some(entry) =
                get_path(&self.root, &["noAdditionalIndent", lookup])
            {
                let v = match entry.as_mapping() {
                    Some(m) => get(m, thing),
                    None => Some(entry),
                };
                match v {
                    Some(v) if as_bool(Some(v)) => Some(String::new()),
                    _ => None,
                }
            } else {
                None
            }
            .or_else(|| {
                get_path(&self.root, &["indentRules", lookup]).and_then(|entry| {
                    let v = match entry.as_mapping() {
                        Some(m) => get(m, thing),
                        None => Some(entry),
                    };
                    v.and_then(|v| as_str(Some(v)))
                        .filter(|s| is_hspace(s) || s.is_empty())
                        .map(|s| s.to_string())
                })
            });
            if let Some(ind) = rule_indent {
                self.previously_found.insert(
                    format!("textWrapAfter{name}"),
                    ObjectSettings {
                        indentation: ind,
                        ..ObjectSettings::default()
                    },
                );
            }
        }
        os
    }
}

fn get_mut_seq(root: &mut Mapping) -> Option<&mut Vec<Value>> {
    root.get_mut(&Value::String("specialBeginEnd".into()))
        .and_then(|v| v.as_sequence_mut())
}

pub fn get_mut_special_seq(root: &mut Mapping) -> Option<&mut Vec<Value>> {
    get_mut_seq(root)
}

// ---------------------------------------------------------------------------
// 加载与合并入口
// ---------------------------------------------------------------------------

fn find_default_settings() -> Option<(String, String)> {
    let candidates: Vec<std::path::PathBuf> = {
        let mut v = Vec::new();
        if let Ok(exe) = std::env::current_exe() {
            if let Some(dir) = exe.parent() {
                v.push(dir.join("defaultSettings.yaml"));
                v.push(dir.join("../../texmf-dist/scripts/latexindent/defaultSettings.yaml"));
                v.push(dir.join("LatexIndent/defaultSettings.yaml"));
            }
        }
        if let Ok(cwd) = std::env::current_dir() {
            v.push(cwd.join("defaultSettings.yaml"));
        }
        v
    };
    for p in candidates {
        if let Ok(s) = std::fs::read_to_string(&p) {
            return Some((s, p.display().to_string()));
        }
    }
    None
}

pub fn load_root_settings() -> Mapping {
    let (content, source) = find_default_settings().unwrap_or_else(|| {
        (
            DEFAULT_SETTINGS_EMBEDDED.to_string(),
            "<embedded>".to_string(),
        )
    });
    logger::info("*YAML settings read: defaultSettings.yaml");
    logger::info(&format!("Reading defaultSettings.yaml from {source}"));
    parse_yaml_lenient(&content).unwrap_or_else(|| {
        logger::fatal("*Could not open defaultSettings.yaml");
        std::process::exit(2);
    })
}

/// File::HomeDir::my_home 等价（Windows: USERPROFILE，其余 HOME）。
fn home_dir() -> String {
    std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .unwrap_or_default()
}

fn indentconfig_path() -> Option<std::path::PathBuf> {
    use std::path::PathBuf;
    if let Ok(p) = std::env::var("LATEXINDENT_CONFIG") {
        let pb = PathBuf::from(&p);
        if pb.is_file() {
            logger::info("The $LATEXINDENT_CONFIG variable was detected.");
            logger::info(&format!("The value of $LATEXINDENT_CONFIG is: \"{p}\""));
            return Some(pb);
        }
        logger::warn(
            "*The $LATEXINDENT_CONFIG variable is assigned, but does not point to a file!",
        );
        logger::info(&format!("The value of $LATEXINDENT_CONFIG is: \"{p}\""));
    }
    let home = home_dir();
    if home.is_empty() {
        return None;
    }
    let home = PathBuf::from(home);
    let mut candidates = Vec::new();
    if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
        candidates.push(PathBuf::from(xdg).join("latexindent/indentconfig.yaml"));
    }
    if let Ok(la) = std::env::var("LOCALAPPDATA") {
        candidates.push(PathBuf::from(la).join("latexindent/indentconfig.yaml"));
    }
    candidates.push(home.join("AppData/Local/latexindent/indentconfig.yaml"));
    candidates.push(home.join(".config/latexindent/indentconfig.yaml"));
    candidates.push(home.join("Library/Preferences/latexindent/indentconfig.yaml"));
    candidates.push(home.join("indentconfig.yaml"));
    candidates.push(home.join(".indentconfig.yaml"));
    candidates.into_iter().find(|p| p.is_file())
}

/// 组装最终设置（默认 + indentconfig + -l + -y），返回主设置对象。
pub fn read_settings(
    root: &mut Mapping,
    local_settings_arg: &Option<String>,
    yaml_arg: &Option<String>,
    only_default: bool,
    working_dir: &str,
) -> MainSettings {
    let mut abs_paths: Vec<std::path::PathBuf> = Vec::new();

    if !only_default {
        if let Some(cfg) = indentconfig_path() {
            logger::info(&format!(
                "The config file in \"{}\" will be read",
                cfg.display()
            ));
            if let Ok(content) = std::fs::read_to_string(&cfg) {
                if let Some(m) = parse_yaml_lenient(&content) {
                    if let Some(Value::Sequence(paths)) = get(&m, "paths") {
                        logger::info("*YAML settings read: paths");
                        for p in paths.iter().filter_map(|v| v.as_str()) {
                            abs_paths.push(std::path::PathBuf::from(p));
                        }
                    }
                }
            }
        } else {
            logger::info("latexindent didn't find indentconfig.yaml or .indentconfig.yaml");
        }
    }

    // -l 处理（+ 展开、逗号分列、补 .yaml、本地文件定位）
    if let Some(ls) = local_settings_arg.clone() {
        if only_default {
            logger::info(&format!(
                "Ignoring the -l switch: {ls} (you used the -d switch)"
            ));
        } else {
            // Perl: -l 无值 → 搜索默认 4 个本地设置名
            let ls = if ls.is_empty() {
                logger::info(
                    "*-l switch used without filename, will search for the following files in turn:",
                );
                logger::info(
                    "localSettings.yaml,latexindent.yaml,.localSettings.yaml,.latexindent.yaml",
                );
                "localSettings.yaml,latexindent.yaml,.localSettings.yaml,.latexindent.yaml"
                    .to_string()
            } else {
                ls
            };
            // Perl: 去首尾水平空白、逗号周围空白
            let ls = ls.trim_matches(|c: char| c == ' ' || c == '\t');
            let ls: String = ls
                .split(',')
                .map(|s| s.trim().to_string())
                .collect::<Vec<_>>()
                .join(",");
            logger::info("*YAML settings read: -l switch");
            let mut value = ls;
            if value.contains('+') {
                logger::info(
                    "+ found in call for -l switch: will add localSettings.yaml,latexindent.yaml,.localSettings.yaml,.latexindent.yaml",
                );
                // Perl: s/\h*\+\h*/$commaAtBeginning.defaults.$commaAtEnd/（仅首个 +）
                let default_names =
                    "localSettings.yaml,latexindent.yaml,.localSettings.yaml,.latexindent.yaml";
                let at_start = value.trim_start_matches([' ', '\t']).starts_with('+');
                let (pre, post) = if at_start { ("", ",") } else { (",", "") };
                let idx = value.find('+').unwrap();
                let before = value[..idx].trim_end_matches([' ', '\t']);
                let after = value[idx + 1..].trim_start_matches([' ', '\t']);
                value = format!("{before}{pre}{default_names}{post}{after}");
                logger::info(&format!("New value of -l switch: {value}"));
            }
            if value.contains(',') {
                logger::info("Multiple localSettings found, separated by commas:");
                let names: Vec<&str> = value.split(',').collect();
                logger::info(&names.join(", "));
            }
            apply_local_settings(root, &value, working_dir, &mut abs_paths);
        }
    }

    // 逐个读入用户 YAML 并合并（含 paths 追加展开）
    if !abs_paths.is_empty() {
        logger::info("*YAML settings, reading from the following files:");
    }
    let home = home_dir();
    let mut i = 0;
    while i < abs_paths.len() {
        let p = abs_paths[i].clone();
        i += 1;
        let empty_file = std::fs::metadata(&p).map(|m| m.len() == 0).unwrap_or(false);
        let content = std::fs::read_to_string(&p);
        if content.is_err() || empty_file {
            // Perl 总是以 $homeDir/indentconfig.yaml 为 WARN 头（即使缺失路径来自别的文件）
            logger::warn(&format!("*{home}/indentconfig.yaml"));
            if empty_file {
                logger::info(&format!(
                    "specifies {} but this file is EMPTY -- not reading from it",
                    p.display()
                ));
            } else {
                logger::info(&format!(
                    "specifies {} but this file does not exist - unable to read settings from this file",
                    p.display()
                ));
            }
            continue;
        }
        logger::info(&format!("Reading USER settings from {}", p.display()));
        if let Some(mut m) = parse_yaml_lenient(&content.unwrap()) {
            if let Some(Value::Sequence(paths)) = get(&m, "paths") {
                logger::info(&format!("Reading path information from {}", p.display()));
                logger::info(&yaml_tiny_dump(&Value::Mapping(m.clone())));
                for q in paths.iter().filter_map(|v| v.as_str()) {
                    abs_paths.push(std::path::PathBuf::from(q));
                }
            }
            // Perl：specialBeginEnd HASH→ARRAY 向后兼容，在合并前执行
            special_begin_end_hash_to_array(&mut m);
            deep_merge(root, &m, "");
            // showEveryYamlRead → YAML::Tiny::Dump 形式输出到日志
            if as_bool(get_path(root, &["logFilePreferences", "showEveryYamlRead"])) {
                logger::info(&yaml_tiny_dump(&Value::Mapping(m.clone())));
            } else {
                logger::info(
                    "Not showing settings in the log file (see showEveryYamlRead and showAmalgamatedSettings).",
                );
            }
        } else {
            logger::warn(&format!(
                "*{} contains invalid yaml format- not reading from it",
                p.display()
            ));
        }
    }

    // -y 覆盖
    if let Some(y) = yaml_arg {
        if only_default {
            logger::info(&format!(
                "Ignoring the -y switch: {y} (you used the -d switch)"
            ));
        } else {
            logger::info("-y|--yaml: YAML settings specified via command line");
            apply_yaml_overrides(root, y);
        }
    }

    yaml_obsolete_checks(root);

    // fineTuning environment checks（GetYamlSettings.pm L895-908）：begin/end 只给一半时
    // 整段覆写被忽略，退回默认环境正则，并各打一条 WARN。
    let env_begin_set = matches!(
        get_path(root, &["fineTuning", "environments", "begin"]),
        Some(v) if !matches!(v, Value::Null)
    );
    let env_end_set = matches!(
        get_path(root, &["fineTuning", "environments", "end"]),
        Some(v) if !matches!(v, Value::Null)
    );
    if env_begin_set && !env_end_set {
        logger::warn(
            "*fineTuning environment begin specified and *NOT* end, ignoring this and using default environment regex",
        );
    }
    if !env_begin_set && env_end_set {
        logger::warn(
            "*fineTuning environment end specified and *NOT* begin, ignoring this and using default environment regex",
        );
    }

    MainSettings::new(root.clone())
}

/// yaml_obsolete_checks（对应 GetYamlSettings.pm）：废弃字段警告并从设置中删除。
/// 检查顺序与 Perl 的 if 链一致（obsolete.txt 基准按此顺序）。
fn yaml_obsolete_checks(root: &mut Mapping) {
    let obsolete = " obsolete, no need for it from V4.0";
    // Perl 的 defined 判定：键存在且值非 undef
    let defined = |m: Option<&Value>| -> bool {
        match m {
            Some(Value::Null) | None => false,
            Some(_) => true,
        }
    };

    let checks: [(&[&str], String); 12] = [
        (
            &["preambleCommandsBeforeEnvironments"],
            format!("preambleCommandsBeforeEnvironments{obsolete}"),
        ),
        (
            &["itemNames"],
            "itemNames obsolete, use fineTuning: items: itemRegex instead".to_string(),
        ),
        (
            &["noAdditionalIndentGlobal", "filecontents"],
            format!("noAdditionalIndentGlobal: filecontents{obsolete}"),
        ),
        (
            &["indentRulesGlobal", "filecontents"],
            format!("indentRulesGlobal: filecontents{obsolete}"),
        ),
        (
            &["commandCodeBlocks"],
            format!("commandCodeBlocks{obsolete}"),
        ),
        (
            &[
                "modifyLineBreaks",
                "textWrapOptions",
                "blocksFollow",
                "filecontents",
            ],
            format!("modifyLineBreaks: textWrapOptions: blocksFollow: filecontents{obsolete}"),
        ),
        (
            &[
                "modifyLineBreaks",
                "textWrapOptions",
                "blocksEndBefore",
                "filecontents",
            ],
            format!("modifyLineBreaks: textWrapOptions: blocksEndBefore: filecontents{obsolete}"),
        ),
        (
            &["fineTuning", "items", "canBeFollowedBy"],
            format!("fineTuning: items: canBeFollowedBy{obsolete}"),
        ),
        (
            &["fineTuning", "keyEqualsValuesBracesBrackets", "follow"],
            format!("fineTuning: keyEqualsValuesBracesBrackets: follow{obsolete}"),
        ),
        (
            &["fineTuning", "namedGroupingBracesBrackets", "follow"],
            format!("fineTuning: namedGroupingBracesBrackets: follow{obsolete}"),
        ),
        (
            &["fineTuning", "UnNamedGroupingBracesBrackets", "follow"],
            format!("fineTuning: UnNamedGroupingBracesBrackets: follow{obsolete}"),
        ),
        (
            &["fineTuning", "arguments", "before"],
            format!("fineTuning: arguments: before{obsolete}"),
        ),
    ];

    if !checks.iter().any(|(path, _)| defined(get_path(root, path))) {
        return;
    }
    logger::warn("*Obsolete YAML");
    for (path, msg) in &checks {
        if !defined(get_path(root, path)) {
            continue;
        }
        logger::warn(msg);
        remove_path(root, path);
    }
}

/// 删除嵌套键（Perl delete 语义：沿途非映射则放弃）。
fn remove_path(root: &mut Mapping, path: &[&str]) {
    let mut dst: &mut Mapping = root;
    for key in &path[..path.len() - 1] {
        dst = match dst.get_mut(Value::String((*key).to_string())) {
            Some(Value::Mapping(m)) => m,
            _ => return,
        };
    }
    dst.remove(Value::String(path[path.len() - 1].to_string()));
}

fn apply_local_settings(
    _root: &mut Mapping,
    spec: &str,
    working_dir: &str,
    abs_paths: &mut Vec<std::path::PathBuf>,
) {
    for name in spec.split(',') {
        let name = name.trim();
        if name.is_empty() {
            continue;
        }
        let mut file = name.to_string();
        if !file.ends_with(".yaml") {
            if !file.ends_with('.') {
                file.push('.');
            }
            file.push_str("yaml");
        }
        let is_default_name = [
            "localSettings.yaml",
            "latexindent.yaml",
            ".localSettings.yaml",
            ".latexindent.yaml",
        ]
        .iter()
        .any(|n| *n == file);
        let candidate = if is_default_name {
            // Perl 以 "/" 拼接（dirname "." → ./localSettings.yaml）
            let in_work = std::path::PathBuf::from(format!("{working_dir}/{file}"));
            let in_cwd = std::env::current_dir()
                .ok()
                .map(|d| std::path::PathBuf::from(format!("{}/{}", d.display(), file)));
            if in_work.is_file() {
                in_work
            } else if in_cwd.as_ref().map(|p| p.is_file()).unwrap_or(false) {
                in_cwd.unwrap()
            } else {
                std::path::PathBuf::from(&file)
            }
        } else {
            std::path::PathBuf::from(&file)
        };
        if candidate.is_file()
            && std::fs::metadata(&candidate)
                .map(|m| m.len() > 0)
                .unwrap_or(false)
        {
            logger::info(&format!(
                "Adding {} to YAML read paths",
                candidate.display()
            ));
            abs_paths.push(candidate);
        } else if !candidate.is_file() {
            // Perl：默认名仅当 CWD 四个默认文件全不存在时才告警
            let name_matches = file.contains("localSettings") || file.contains("latexindent");
            let any_default_in_cwd = [
                "localSettings.yaml",
                ".localSettings.yaml",
                "latexindent.yaml",
                ".latexindent.yaml",
            ]
            .iter()
            .any(|n| std::path::Path::new(n).is_file());
            if !name_matches || !any_default_in_cwd {
                logger::warn(&format!(
                    "*yaml file not found: {file} not found. Proceeding without it."
                ));
            }
        }
    }
}
