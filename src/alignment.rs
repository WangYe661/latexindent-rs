//! & 对齐（对应 LatexIndent/AlignmentAtAmpersand.pm）。
//!
//! 覆盖默认语义：justification=left、delimiterJustification=left、
//! multiColumnGrouping、dontMeasure（largest/字符串/this/regex × applyTo cell|row）、
//! leadingBlankColumn（含 spacesBeforeAmpersand 映射形）、
//! alignContentAfterDoubleBackSlash（DBS 后递归对齐）、alignFinalDoubleBackSlash、
//! delimiterRegEx 自定义、隐藏子代码块按内容重建最大行宽测量、DBS poly-switch
//! （含 DBSStartsOnOwnLine=2 的注释变体）、parentBegin 的 begin 计量 quirk
//! （begin 为 {/[ 时量 `parentBegin + "{"`，与 Perl 一致但无独立测试覆盖）。

use pcre2::bytes::Regex;
use serde_yaml::Value;

use crate::patterns::{gn, replace_all_with};
use crate::settings::{as_bool, as_i64, as_str, get, get_path, ObjectSettings};
use crate::state;
use crate::tokens::tok;

fn opt_i(os: &ObjectSettings, key: &str, default: i64) -> i64 {
    as_i64(os.align.get(key)).unwrap_or(default)
}

fn opt_b(os: &ObjectSettings, key: &str, default: bool) -> bool {
    match os.align.get(key) {
        Some(v) => as_bool(Some(v)),
        None => default,
    }
}

fn opt_s(os: &ObjectSettings, key: &str, default: &str) -> String {
    os.align
        .get(key)
        .and_then(|v| v.as_str())
        .unwrap_or(default)
        .to_string()
}

fn switches_is_m() -> bool {
    crate::switches::with(|s| s.is_m_switch_active())
}

/// 尾注释正则片段（token 形式）。patterns 在 token_check 升级 token 名前构建，
/// 其 token 名可能过期，故用当前 token 重建，避免注释 token 混进单元格被计量。
pub(crate) fn trailing_comment_pattern() -> String {
    let t = tok(|t| t.clone());
    let not_preceded_by = state::settings(|s| {
        as_str(get_path(
            &s.root,
            &["fineTuning", "trailingComments", "notPrecededBy"],
        ))
        .unwrap_or(r"(?<!\\)")
        .to_string()
    });
    format!(
        "{}%{}\\d+{}",
        not_preceded_by, t.trailing_comment, t.end_of_token
    )
}

fn trailing_comment_re() -> Regex {
    crate::patterns::re_new(&format!(r"(?s){}", trailing_comment_pattern()))
        .expect("trailingCommentRegExp")
}

/// 量宽（对应 Perl get_column_width）：默认字素簇计数（\X）；
/// `--GCString` 时用 Unicode::GCString->columns() 语义（东亚宽/全角记 2 列）
fn width(s: &str) -> usize {
    use unicode_segmentation::UnicodeSegmentation;
    if crate::switches::with(|sw| sw.gcstring) {
        s.graphemes(true).map(gc_columns).sum()
    } else {
        s.graphemes(true).count()
    }
}

/// 字素簇的显示列宽：取基字符的 East Asian Width，Wide/Fullwidth 记 2
fn gc_columns(g: &str) -> usize {
    let c = g.chars().next().unwrap_or(' ');
    if is_ea_wide(c) {
        2
    } else {
        1
    }
}

fn is_ea_wide(c: char) -> bool {
    matches!(c as u32,
        0x1100..=0x115F
            | 0x2E80..=0x303E
            | 0x3041..=0x33FF
            | 0x3400..=0x4DBF
            | 0x4E00..=0x9FFF
            | 0xA000..=0xA4CF
            | 0xAC00..=0xD7A3
            | 0xF900..=0xFAFF
            | 0xFE30..=0xFE4F
            | 0xFF00..=0xFF60
            | 0xFFE0..=0xFFE6
            | 0x1F300..=0x1F64F
            | 0x1F900..=0x1F9FF
            | 0x20000..=0x2FFFD
            | 0x30000..=0x3FFFD
    )
}

pub struct AlignResult {
    pub begin_adjusted: bool,
    pub indentation: Option<String>,
}

/// double_back_slash_else：`\\` 的 poly-switch（DBSStartsOnOwnLine/DBSFinishesWithLineBreak）。
/// v_start/v_finish 由调用方给出：参数对象用 mand/opt 映射值，其余用 previouslyFound（os.poly）。
pub fn double_back_slash_else(body: &str, os: &ObjectSettings, v_start: i64, v_finish: i64) -> String {
    let t = tok(|t| t.clone());
    if v_start == 0 && v_finish == 0 {
        return body.to_string();
    }
    let tc = trailing_comment_pattern();
    let dbs = opt_s(
        os,
        "doubleBackSlash",
        r"\\\\(?:\h*\[\h*[0-9.]+\h*[a-zA-Z]+\h*\])?",
    );
    let defs = state::patterns(|p| p.defs.clone());
    let re = crate::patterns::re_new(&format!(
        r"{defs}(?s)(?<WS>\s*)(?:(?<DBSSTATEMENT>{})(?<ET>\h*)?(?<EC>{tc})?(?<EL>\R\s*)?)",
        crate::patterns::x_guard(&dbs)
    ))
    .expect("DBS regex");
    let out = replace_all_with(body, &re, |caps| {
        // 注意：defs 的 (?(DEFINE)...) 内组名占编号，WS 必须按命名组取
        let ws = gs_named(caps, "WS");
        let stmt = gs_named(caps, "DBSSTATEMENT").to_string();
        crate::logger::trace(&format!("*found: {stmt} \t type: DBS"));
        // Perl $horizontalSpaceBeforeDBS：ws 纯水平空白且 DBSStartsOnOwnLine>0 时前置保留
        let ws_before = if v_start > 0 && ws.chars().all(|c| c == ' ' || c == '\t') {
            ws.to_string()
        } else {
            String::new()
        };
        let mut m = crate::mlb::MlbObj {
            begin: format!("{ws}{stmt}"),
            body: format!(
                "{}{}{}",
                gs_named(caps, "ET"),
                gs_named(caps, "EC"),
                gs_named(caps, "EL")
            ),
            end: String::new(),
            horizontal_trailing_space: String::new(),
            trailing_comment: String::new(),
            linebreaks_at_end_begin: false,
            linebreaks_at_end_body: false,
            linebreaks_at_end_end: String::new(),
        };
        crate::mlb::begin_starts_on_own_line(&mut m, v_start, "block");
        crate::mlb::body_starts_on_own_line(&mut m, v_finish);
        if v_finish == -1 && !m.body.starts_with([' ', '\t']) {
            m.body = format!(" {}", m.body);
        }
        format!("{}{}{}{}", ws_before, m.begin, m.body, m.end)
    });
    // Perl double_back_slash_else 收尾：token 当场还原为真换行（后续对齐/缩进
    // 才能看到行结构；否则 ADD token 拖到 find_all_code_blocks 末尾才还原，
    // 对齐的行切分/缩进的 /mg 都看不到这些行）
    let out = crate::mlb::line_break_token_adjust(&out);
    crate::mlb::after_indentation_token_adjust(&out)
}

fn gs_named<'t>(caps: &'t pcre2::bytes::Captures, name: &str) -> &'t str {
    crate::patterns::gs(caps, name).unwrap_or("")
}

struct Cell {
    entry: String,
    width: usize,
    delimiter: String,
    delimiter_length: usize,
    individual_padding: usize,
    group_padding: usize,
    grouping_width: usize,
    measure_this: bool,
    /// Perl cellStorage type："X" 计量对齐、"*" 无分隔符不计量、"-" 补位幽灵格、数字为 multicolumn 跨列数
    typ: String,
    /// Perl colSpan："." 或跨列数
    col_span: String,
    /// Perl DBSpadding：alignContentAfterDoubleBackSlash 递归里首格的前导补空
    db_padding: usize,
}

impl Cell {
    fn gap() -> Cell {
        Cell {
            entry: String::new(),
            width: 0,
            delimiter: String::new(),
            delimiter_length: 0,
            individual_padding: 0,
            group_padding: 0,
            grouping_width: 0,
            measure_this: false,
            typ: "-".to_string(),
            col_span: ".".to_string(),
            db_padding: 0,
        }
    }
}

struct Row {
    cells: Vec<Cell>,
    end_piece: String,
    trailing_comment: String,
    n_amp: usize,
    unformatted: String,
}

/// Perl type 的首数字（`m/(\d)/` 语义，"12"→1）
fn type_first_digit(t: &str) -> Option<u32> {
    t.chars().next().and_then(|c| c.to_digit(10))
}

/// Perl `m/(\d+)/` 语义：全数字串才解析
fn type_full_span(t: &str) -> Option<usize> {
    if !t.is_empty() && t.bytes().all(|b| b.is_ascii_digit()) {
        t.parse().ok()
    } else {
        None
    }
}

/// Perl 真值语义：dontMeasure 为 0/空即关闭
fn dm_active(v: Option<&serde_yaml::Value>) -> bool {
    match v {
        None | Some(serde_yaml::Value::Null) => false,
        Some(serde_yaml::Value::Bool(b)) => *b,
        Some(serde_yaml::Value::Number(n)) => n.as_i64().map(|x| x != 0).unwrap_or(true),
        Some(serde_yaml::Value::String(s)) => !s.is_empty() && s != "0",
        Some(_) => true,
    }
}

/// Perl dont_measure(mode=>"row")：仅 applyTo:row 的 this/regex 条目命中整行
fn dont_measure_row(dm: &serde_yaml::Value, row_text: &str, measure_row: &mut bool) {
    let seq = match dm {
        serde_yaml::Value::Sequence(s) => s,
        _ => return,
    };
    for item in seq {
        let m = match item.as_mapping() {
            Some(m) => m,
            None => continue,
        };
        if crate::settings::get(m, "applyTo").and_then(|v| v.as_str()) != Some("row") {
            continue;
        }
        if let Some(this) = crate::settings::get(m, "this").and_then(|v| v.as_str()) {
            if this == row_text {
                *measure_row = false;
            }
        } else if let Some(rx) = crate::settings::get(m, "regex").and_then(|v| v.as_str()) {
            if crate::patterns::re_new(rx)
                .map(|re| re.is_match(row_text.as_bytes()).unwrap_or(false))
                .unwrap_or(false)
            {
                *measure_row = false;
            }
        }
    }
}

/// Perl dont_measure(mode=>"cell")：largest/字符串/this/regex 命中则不计量且 type 置 "X"
fn dont_measure_cell(
    dm: &serde_yaml::Value,
    entry: &str,
    cell_width: usize,
    col: usize,
    max_column_width: &[usize],
    cell: &mut Cell,
) {
    let hit = |cell: &mut Cell| {
        cell.measure_this = false;
        cell.typ = "X".to_string();
    };
    match dm {
        serde_yaml::Value::String(s) if s == "largest" => {
            if max_column_width.get(col).copied().unwrap_or(0) == cell_width {
                hit(cell);
            }
        }
        serde_yaml::Value::Sequence(seq) => {
            for item in seq {
                match item {
                    serde_yaml::Value::String(s) => {
                        if s == entry {
                            hit(cell);
                        }
                    }
                    serde_yaml::Value::Mapping(m) => {
                        let apply = crate::settings::get(m, "applyTo").and_then(|v| v.as_str());
                        if apply.map(|a| a == "cell").unwrap_or(true) {
                            if let Some(this) = crate::settings::get(m, "this").and_then(|v| v.as_str())
                            {
                                if this == entry {
                                    hit(cell);
                                }
                            }
                            if let Some(rx) =
                                crate::settings::get(m, "regex").and_then(|v| v.as_str())
                            {
                                if crate::patterns::re_new(rx)
                                    .map(|re| re.is_match(entry.as_bytes()).unwrap_or(false))
                                    .unwrap_or(false)
                                {
                                    hit(cell);
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        _ => {}
    }
}

/// alignContentAfterDoubleBackSlash 递归上下文（对应 Perl align_at_ampersand 的
/// measure_after_DBS / beforeDBSlengths / maxDBSlength 参数）
struct DbsCtx {
    before_dbs_lengths: Vec<usize>,
    max_dbs_length: usize,
    measure_after_dbs: bool,
}

/// align_at_ampersand 主入口：就地整理 begin/body，返回缩进补偿信息。
pub fn align_at_ampersand(
    begin: &mut String,
    body: &mut String,
    os: &ObjectSettings,
) -> AlignResult {
    align_at_ampersand_with_parent(begin, body, os, None, None)
}

/// 参数对象版：parent_begin 供 begin 恰为 `{`/`[` 时的计量 quirk（Perl parentBegin）；
/// dbs_override 为参数对象自身的 DBS 开关（Perl `${$self}{DBSStartsOnOwnLine}`，
/// 取自 mand/opt 映射），其余对象用 previouslyFound（os.poly）
#[allow(clippy::too_many_arguments)]
pub fn align_at_ampersand_with_parent(
    begin: &mut String,
    body: &mut String,
    os: &ObjectSettings,
    parent_begin: Option<&str>,
    dbs_override: Option<(i64, i64)>,
) -> AlignResult {
    let mut hidden: Vec<(String, String, usize)> = Vec::new();
    align_at_ampersand_impl(begin, body, os, None, &mut hidden, parent_begin, dbs_override).0
}

/// Perl begin 计量：begin 恰为 {/[ 且有 parentBegin 时改量 `parentBegin + "{"`（quirk，
/// 注意 Perl 写死 "{" 即使 begin 是 "["）
fn begin_measure_raw(begin: &str, parent_begin: Option<&str>) -> String {
    if (begin == "{" || begin == "[") && parent_begin.map(|p| !p.is_empty()).unwrap_or(false) {
        format!("{}{{", parent_begin.unwrap())
    } else {
        begin.to_string()
    }
}

/// 核心实现。返回 (AlignResult, 首格是否满足 begin 缩进补偿条件)。
/// `hidden` 跨递归共享（Perl 的 @hiddenChildrenStorage 是全局且不清空，递归里
/// 新增的隐藏子块由外层统一还原）。
fn align_at_ampersand_impl(
    begin: &mut String,
    body: &mut String,
    os: &ObjectSettings,
    dbs: Option<&DbsCtx>,
    hidden: &mut Vec<(String, String, usize)>,
    parent_begin: Option<&str>,
    dbs_override: Option<(i64, i64)>,
) -> (AlignResult, bool) {
    let t = tok(|t| t.clone());
    let mut res = AlignResult {
        begin_adjusted: false,
        indentation: None,
    };
    // Perl measureVerbatim：body 含 verbatim token 即走隐藏子块量宽路径
    let measure_verbatim = body.contains(t.verbatim.as_str());
    // -m 时 DBS poly-switch 先行（Perl 在无换行早退之前执行）
    if switches_is_m() {
        let (v_start, v_finish) = dbs_override.unwrap_or_else(|| {
            (
                *os.poly.get("DBSStartsOnOwnLine").unwrap_or(&0),
                *os.poly.get("DBSFinishesWithLineBreak").unwrap_or(&0),
            )
        });
        *body = double_back_slash_else(body, os, v_start, v_finish);
    }
    if !body.contains('\n') {
        return (res, false); // Perl: return unless body =~ \R
    }

    // begin/body 换行边界（Perl linebreaksAtEnd{begin}/{body}）
    let re_head_lb = crate::patterns::re_new(r"\A\h*\R").unwrap();
    let re_tail_lb = crate::patterns::re_new(r"(?s)\h*\R$").unwrap();
    let mut linebreaks_at_end_begin = re_head_lb.is_match(body.as_bytes()).unwrap_or(false);
    if switches_is_m() && !linebreaks_at_end_begin {
        linebreaks_at_end_begin = re_tail_lb.is_match(begin.as_bytes()).unwrap_or(false);
    }
    if !linebreaks_at_end_begin {
        let cut = body.len() - body.trim_start_matches([' ', '\t']).len();
        let lead = body[..cut].to_string();
        body.drain(..cut);
        begin.push_str(&lead);
    }
    let linebreaks_at_end_body = re_tail_lb.is_match(body.as_bytes()).unwrap_or(false);
    let mut linebreaks_at_end_body = linebreaks_at_end_body;

    let spaces_before_amp = opt_i(os, "spacesBeforeAmpersand", 1).max(0) as usize;
    let spaces_after_amp = opt_i(os, "spacesAfterAmpersand", 1).max(0) as usize;
    let mut leading_blank = opt_i(os, "leadingBlankColumn", -1);
    let align_dbs = opt_b(os, "alignDoubleBackSlash", true);
    let spaces_before_dbs = opt_i(os, "spacesBeforeDoubleBackSlash", 1);
    let align_final_dbs = opt_b(os, "alignFinalDoubleBackSlash", false);
    let align_rows_without_max = opt_b(os, "alignRowsWithoutMaxDelims", true);
    let look_for_child = opt_b(os, "lookForChildCodeBlocks", true);
    let multi_column = opt_b(os, "multiColumnGrouping", false);
    let justification = opt_s(os, "justification", "left");
    let delim_justification = opt_s(os, "delimiterJustification", "left");
    let delimiter_re = opt_s(os, "delimiterRegEx", r"(?<!\\)(&)");
    let dbs_re_src = opt_s(
        os,
        "doubleBackSlash",
        r"\\\\(?:\h*\[\h*[0-9.]+\h*[a-zA-Z]+\h*\])?",
    );

    // ---- 隐藏子代码块（多行 / 含 \\ 或 & 的块整体按宽度计量） ----
    let mut dbs_hidden: Vec<(String, String)> = Vec::new();
    let mut work = body.clone();
    let delim = crate::patterns::re_new(&delimiter_re).expect("delimiterRegEx");
    if look_for_child {
        // Perl：先隐藏 `\\ [`（可选参数间距），避免被 all_code_blocks 当参数吞掉
        let re_dbs_brack = crate::patterns::re_new(r"(?s)(\\\\\h+\[)").unwrap();
        work = replace_all_with(&work, &re_dbs_brack, |caps| {
            let id = format!(
                "{}D{}{}",
                t.alignment_block,
                dbs_hidden.len() + 1,
                t.end_of_token
            );
            dbs_hidden.push((id.clone(), gn(caps, 0).to_string()));
            id
        });
        state::patterns(|p| {
            work = replace_all_with(&work, &p.all_code_blocks, |caps| {
                let whole = gn(caps, 0).to_string();
                // 前后空白留在 token 外（还原后不产生多余空格）
                let trimmed = whole.trim_matches([' ', '\t', '\n', '\r']).to_string();
                let lead_end = whole.find(trimmed.as_str()).unwrap_or(0);
                let leading = whole[..lead_end].to_string();
                let trailing = whole[lead_end + trimmed.len()..].to_string();
                let bad = trimmed.contains('\n')
                    || crate::patterns::re_new(&format!("(?s){}", crate::patterns::x_guard(&dbs_re_src)))
                        .map(|re| re.is_match(trimmed.as_bytes()).unwrap_or(false))
                        .unwrap_or(false)
                    // Perl：`=~ m"${$self}{delimiterRegEx}"` —— 用配置的分隔符正则，
                    // 不是写死的 `&`（用户自定义 delimiterRegEx 时 `&` 可能不是分隔符）
                    || delim.is_match(trimmed.as_bytes()).unwrap_or(false);
                if !bad {
                    return whole;
                }
                // Perl AlignmentAtAmpersand.pm：命中隐藏子块时打 trace
                crate::logger::trace("*found: hidden child in alignment block");
                let w = trimmed.split('\n').map(width).max().unwrap_or(0);
                let id = format!(
                    "{}H{}{}",
                    t.alignment_block,
                    hidden.len() + 1,
                    t.end_of_token
                );
                hidden.push((id.clone(), trimmed, w));
                format!("{leading}{id}{trailing}")
            });
        });
        // 还原 `\\ [`
        for (id, v) in dbs_hidden.iter().rev() {
            work = work.replace(id.as_str(), v);
        }
    }

    // ---- 逐行拆行 ----
    let tc_re = trailing_comment_re();

    let mut rows: Vec<Row> = Vec::new();
    let mut maximum_amp = 0usize;
    let mut min_multi_col_span: Vec<usize> = Vec::new();
    let mut max_delimiter_width: Vec<usize> = Vec::new();
    // Perl loop-1 的 $maxColumnWidth（dontMeasure:largest 的比较基准）
    let mut max_column_width: Vec<usize> = Vec::new();
    let dm_val = os.align.get("dontMeasure").cloned();
    let dm_on = dm_active(dm_val.as_ref());
    let mut lines: Vec<&str> = work.split('\n').collect();
    // Perl 的 split 把行尾空串丢弃
    while lines.last().map(|l| l.is_empty()).unwrap_or(false) {
        lines.pop();
    }
    for line in lines {
        // Perl measureRow：dontMeasure 的 applyTo:row 条目可整行免计量
        let mut measure_row = true;
        if dm_on {
            if let Some(dm) = dm_val.as_ref() {
                dont_measure_row(dm, line, &mut measure_row);
            }
        }
        // Perl：递归里首格宽度 += maxDBSlength、DBSpadding = max-current
        let dbs_first = dbs.map(|d| {
            let current = d.before_dbs_lengths.get(rows.len() + 1).copied().unwrap_or(0);
            (d.max_dbs_length, d.max_dbs_length.saturating_sub(current))
        });
        let mut line = line.to_string();

        // 尾注释剥离
        let trailing_comment = match tc_re.captures(line.as_bytes()).unwrap() {
            Some(c) => {
                let tc = gn(&c, 0).to_string();
                line = line.replace(&tc, "");
                tc
            }
            None => String::new(),
        };

        // 行末 \\ 剥离（alignFinalDoubleBackSlash=0：第一个 \\ 起全部）
        let end_piece = if align_final_dbs {
            // 取最后一个 \\ 起到行尾
            let re = crate::patterns::re_new(&format!(
                r"(?s)({}).*$",
                crate::patterns::x_guard(&dbs_re_src)
            ))
            .unwrap();
            let mut last = None;
            for mm in re.find_iter(line.as_bytes()) {
                last = Some(mm.unwrap());
            }
            match last {
                Some(m) => {
                    let ep = line[m.start()..m.end()].to_string();
                    line = line[..m.start()].to_string();
                    ep
                }
                None => String::new(),
            }
        } else {
            match crate::patterns::re_new(&format!(r"(?s)\\\\.*$"))
                .unwrap()
                .find(line.as_bytes())
                .unwrap()
            {
                Some(m) => {
                    let ep = line[m.start()..m.end()].to_string();
                    line = line[..m.start()].to_string();
                    ep
                }
                None => String::new(),
            }
        };
        let unformatted = line.clone();

        // 环绕 & 的空白折叠、行首尾空白剥离
        let fold = crate::patterns::re_new(&format!(
            r"(?<!\\)\h*({})\h*",
            crate::patterns::x_guard(&delimiter_re)
        ))
        .unwrap();
        line = replace_all_with(&line, &fold, |c| gn(c, 1).to_string());
        let line = line.trim_matches([' ', '\t']).to_string();

        let n_amp = delim.find_iter(line.as_bytes()).count();
        maximum_amp = maximum_amp.max(n_amp);
        // 行尾是分隔符 → 补一个空格（空末格可见）
        let mut line = line;
        if n_amp > 0 && ends_with_delim(&line, &delim) {
            line.push(' ');
        }

        // 按分隔符切列（分隔符归属它前面的单元格；multicolumn 占位补 gap 格）
        let mut cells: Vec<Cell> = Vec::new();
        let mut last = 0usize;
        let mut last_entry = 0usize;
        for m in delim.find_iter(line.as_bytes()) {
            let m = m.unwrap();
            last_entry = push_column(
                &mut cells,
                &mut min_multi_col_span,
                &mut max_column_width,
                &line[last..m.start()],
                n_amp,
                measure_row,
                multi_column,
                dbs_first,
                &hidden,
            );
            let dtext = match crate::patterns::re_new(&delimiter_re)
                .unwrap()
                .captures(&line.as_bytes()[m.start()..m.end()])
                .unwrap_or(None)
            {
                Some(c) => gn(&c, 1).to_string(),
                None => String::new(),
            };
            let dlen = width(&dtext);
            cells[last_entry].delimiter = dtext;
            cells[last_entry].delimiter_length = dlen;
            if max_delimiter_width.len() <= last_entry {
                max_delimiter_width.resize(last_entry + 1, 0);
            }
            max_delimiter_width[last_entry] = max_delimiter_width[last_entry].max(dlen);
            last = m.end();
        }
        push_column(
            &mut cells,
            &mut min_multi_col_span,
            &mut max_column_width,
            &line[last..],
            n_amp,
            measure_row,
            multi_column,
            dbs_first,
            &hidden,
        );

        rows.push(Row {
            cells,
            end_piece,
            trailing_comment,
            n_amp,
            unformatted,
        });
    }

    // ---- multicolumn 预检 / 列宽 / 分组填充 / 复检（对应 Perl 同名子过程） ----
    if multi_column {
        multicolumn_pre_check(&mut rows, &min_multi_col_span);
    }
    individual_padding(
        &mut rows,
        &max_delimiter_width,
        align_rows_without_max,
        maximum_amp,
        &delim_justification,
        dm_val.as_ref(),
        &max_column_width,
        &mut leading_blank,
    );
    if multi_column {
        multicolumn_padding(
            &mut rows,
            spaces_before_amp,
            spaces_after_amp,
            &justification,
        );
        multicolumn_post_check(&mut rows, spaces_before_amp, spaces_after_amp);
    }

    // ---- main_formatting（对应 Perl main_formatting） ----
    let final_spacing = if spaces_before_dbs >= 1 {
        " ".repeat(spaces_before_dbs as usize)
    } else {
        String::new()
    };
    let mut formatted: Vec<(String, usize)> = Vec::new(); // (行文本, 行宽)
    let measure_hv = !hidden.is_empty() || measure_verbatim;
    // Perl hidden_child_row_width 的 begin 判定（无 entry 非空条件）
    let begin_ok_row = !linebreaks_at_end_begin
        && rows
            .first()
            .and_then(|r| r.cells.first())
            .map(|c0| c0.typ == "X" && c0.measure_this)
            .unwrap_or(false);
    for (ri, r) in rows.iter().enumerate() {
        let mut row = String::new();
        for c in &r.cells {
            if c.typ == "*" || c.typ == "-" {
                row.push_str(&c.entry);
                continue;
            }
            // Perl：alignContentAfterDoubleBackSlash 的首格前导补空
            row.push_str(&" ".repeat(c.db_padding));
            if justification == "left" {
                row.push_str(&c.entry);
                row.push_str(&" ".repeat(c.individual_padding));
                row.push_str(&" ".repeat(c.group_padding));
            } else {
                row.push_str(&" ".repeat(c.group_padding));
                row.push_str(&" ".repeat(c.individual_padding));
                row.push_str(&c.entry);
            }
            // Perl 对每个格都补 sb + delim + sa（末格 delimiter 为空串）
            row.push_str(&" ".repeat(spaces_before_amp));
            row.push_str(&c.delimiter);
            row.push_str(&" ".repeat(spaces_after_amp));
        }
        if !align_rows_without_max && r.n_amp < maximum_amp {
            row = r.unformatted.clone();
        }
        // Perl s/\h*$/$finalSpacing/：行尾空白折叠为固定间距
        while row.ends_with(' ') || row.ends_with('\t') {
            row.pop();
        }
        row.push_str(&final_spacing);
        if row.trim_matches([' ', '\t']).is_empty() {
            row.clear();
        }
        let w_plain = cell_width(&row, &hidden);
        let w = if measure_hv {
            let begin_meas = begin_measure_raw(begin.as_str(), parent_begin);
            hidden_child_row_width(&row, ri, w_plain, &hidden, &begin_meas, begin_ok_row)
        } else {
            w_plain
        };
        formatted.push((row, w));
    }
    // Perl measure_after_DBS：只回填 main_formatting 的行（无 \\ 尾件/注释）就返回
    if dbs.map(|d| d.measure_after_dbs).unwrap_or(false) {
        *body = formatted.iter().map(|(s, _)| format!("{s}\n")).collect();
        let ok = rows
            .first()
            .and_then(|r| r.cells.first())
            .map(|c0| c0.typ == "X" && c0.measure_this && !c0.entry.is_empty())
            .unwrap_or(false);
        return (res, ok);
    }

    // maximumRowWidth：无 \\ 且无 & 的行不参与
    let mut maximum_row_width = 0usize;
    for (ri, (_, w)) in formatted.iter().enumerate() {
        if *w > maximum_row_width && !(rows[ri].n_amp == 0 && rows[ri].end_piece.is_empty()) {
            maximum_row_width = *w;
        }
    }

    // ---- 最终 \\ 循环（对应 Perl final loop） ----
    let mut out_lines: Vec<String> = Vec::new();
    for (ri, r) in rows.iter().enumerate() {
        let (mut line, row_width) = formatted[ri].clone();
        let mut padding = String::new();
        if !line.trim_matches([' ', '\t']).is_empty() {
            if !align_dbs {
                while line.ends_with(' ') || line.ends_with('\t') {
                    line.pop();
                }
            }
            if spaces_before_dbs < 0 && !align_dbs {
                padding = String::new();
            } else if spaces_before_dbs >= 0 && !align_dbs {
                padding = " ".repeat(spaces_before_dbs as usize);
            } else {
                padding = " ".repeat((maximum_row_width as i64 - row_width as i64).max(0) as usize);
            }
        }
        line.push_str(&padding);
        line.push_str(&r.end_piece);
        line.push_str(&r.trailing_comment);
        if (r.n_amp == 0 && r.end_piece.is_empty())
            || (r.n_amp < maximum_amp && !align_rows_without_max && r.end_piece.is_empty())
        {
            line = r.unformatted.clone();
            line.push_str(&r.trailing_comment);
        }
        // Perl leadingBlankColumn：行首水平空白定宽替换（s/^\h*/$padding/）
        if leading_blank > -1 {
            let pad = " ".repeat(leading_blank as usize);
            let trimmed = line.trim_start_matches([' ', '\t']);
            line = format!("{pad}{trimmed}");
        }
        out_lines.push(line);
    }

    // ---- alignContentAfterDoubleBackSlash：以首个 \\ 为界拆行，after 部分递归对齐 ----
    let mut rec_first_ok = false;
    let mut dbs_ran = false;
    if opt_b(os, "alignContentAfterDoubleBackSlash", false) {
        // Perl：linebreaksAtEnd{body} 在此重测（body 尾恰好一个 \R）
        let re_lb_end = crate::patterns::re_new(r"\R$").unwrap();
        linebreaks_at_end_body = re_lb_end.is_match(body.as_bytes()).unwrap_or(false);
        let spaces_after_dbs = opt_i(os, "spacesAfterDoubleBackSlash", 1).max(0) as usize;
        let re_split = crate::patterns::re_new(&format!(
            r"(?s)(.*?)({})\h*",
            crate::patterns::x_guard(&dbs_re_src)
        ))
        .unwrap();
        let mut before_dbs_lengths: Vec<usize> = vec![0];
        let mut parts: Vec<(String, String)> = Vec::new();
        let mut max_dbs_length = 0usize;
        let mut after_body = String::new();
        for row in out_lines.iter_mut() {
            let (before, dbs_text, rest) = match re_split.captures(row.as_bytes()).unwrap() {
                Some(c) => {
                    let m = c.get(0).unwrap();
                    let before = gn(&c, 1).to_string();
                    let mut dbs_text = gn(&c, 2).to_string();
                    if !dbs_text.is_empty() {
                        dbs_text.push_str(&" ".repeat(spaces_after_dbs));
                    }
                    (before, dbs_text, row[m.end()..].to_string())
                }
                None => (String::new(), String::new(), row.clone()),
            };
            let len = width(&format!("{before}{dbs_text}"));
            before_dbs_lengths.push(len);
            max_dbs_length = max_dbs_length.max(len);
            after_body.push_str(&rest);
            after_body.push('\n');
            parts.push((before, dbs_text));
        }
        if !linebreaks_at_end_body {
            after_body = replace_all_with(&after_body, &re_tail_lb, |_| String::new());
        }
        let ctx = DbsCtx {
            before_dbs_lengths,
            max_dbs_length,
            measure_after_dbs: true,
        };
        let (_, ok) = align_at_ampersand_impl(
            begin,
            &mut after_body,
            os,
            Some(&ctx),
            hidden,
            parent_begin,
            dbs_override,
        );
        rec_first_ok = ok;
        dbs_ran = true;
        let after_split: Vec<&str> = after_body.split('\n').collect();
        for (i, row) in out_lines.iter_mut().enumerate() {
            let (before, dbs_text) = &parts[i];
            *row = format!(
                "{before}{dbs_text}{}",
                after_split.get(i).copied().unwrap_or("")
            );
        }
    }

    // ---- begin 缩进补偿（Perl beginAdjusted） ----
    // Perl quirk：alignContentAfterDoubleBackSlash 递归把全局 @cellStorage 换成递归
    // 的首格，判定因此用递归首格
    let first_ok = if dbs_ran {
        rec_first_ok
    } else {
        rows.first()
            .and_then(|r| r.cells.first())
            .map(|c0| c0.typ == "X" && c0.measure_this && !c0.entry.is_empty())
            .unwrap_or(false)
    };
    if !linebreaks_at_end_begin && first_ok {
        // Perl：量宽去前导空白；begin 恰为 {/[ 时改量 parentBegin+"{"
        let measure = begin_measure_raw(begin.as_str(), parent_begin);
        res.indentation = Some(" ".repeat(width(measure.trim_start())));
        res.begin_adjusted = true;
    }

    let mut new_body: String = out_lines.iter().map(|l| format!("{l}\n")).collect();
    if !linebreaks_at_end_body {
        // Perl s/\h*\R$//s：去掉串尾换行及其前的水平空白
        new_body = replace_all_with(&new_body, &re_tail_lb, |_| String::new());
    }

    // ---- 隐藏子块还原（token 后续行按 token 前列宽补缩进；Perl foreach 正序还原） ----
    for (id, text, _) in hidden.iter() {
        let added = match new_body.find(id.as_str()) {
            Some(pos) => {
                let line_start = new_body[..pos].rfind('\n').map(|i| i + 1).unwrap_or(0);
                " ".repeat(width(&new_body[line_start..pos]))
            }
            None => String::new(),
        };
        let mut content = prefix_lines(text, &added);
        content = strip_first(&content, &added);
        new_body = new_body.replacen(id.as_str(), &content, 1);
    }
    *body = new_body;
    (res, first_ok)
}

/// Perl 列存储的 entry 列入栈（含 multicolumn 跨列占位），返回该单元格下标。
fn push_column(
    cells: &mut Vec<Cell>,
    min_multi_col_span: &mut Vec<usize>,
    max_column_width: &mut Vec<usize>,
    raw: &str,
    n_amp: usize,
    measure_row: bool,
    multi_column: bool,
    dbs_first: Option<(usize, usize)>,
    hidden: &[(String, String, usize)],
) -> usize {
    let mut entry = raw.to_string();
    if entry.ends_with('\\') {
        entry.push(' ');
    }
    let mut cell = Cell {
        width: 0,
        entry,
        delimiter: String::new(),
        delimiter_length: 0,
        individual_padding: 0,
        group_padding: 0,
        grouping_width: 0,
        measure_this: n_amp > 0 && measure_row,
        typ: if n_amp > 0 { "X" } else { "*" }.to_string(),
        col_span: ".".to_string(),
        db_padding: 0,
    };
    cell.width = cell_width(&cell.entry, hidden);
    if let Some((add, pad)) = dbs_first {
        if cells.is_empty() {
            cell.width += add;
            cell.db_padding = pad;
        }
    }
    let idx = cells.len();
    // Perl loop-1 的 $maxColumnWidth（type 为 "X" 才计量；dontMeasure:largest 的比较基准）
    if cell.typ == "X" {
        if max_column_width.len() <= idx {
            max_column_width.resize(idx + 1, 0);
        }
        max_column_width[idx] = max_column_width[idx].max(cell.width);
    }
    let mut span = 0usize;
    if multi_column {
        if let Ok(Some(caps)) = crate::patterns::re_new(r"\\multicolumn\{(\d+)\}")
            .unwrap()
            .captures(cell.entry.as_bytes())
        {
            if let Ok(n) = gn(&caps, 1).parse::<usize>() {
                if n > 1 {
                    span = n;
                }
            }
        }
    }
    cells.push(cell);
    if span > 0 {
        cells[idx].typ = span.to_string();
        cells[idx].col_span = span.to_string();
        cells[idx].measure_this = false;
        for _ in 1..span {
            cells.push(Cell::gap());
        }
        if min_multi_col_span.len() <= idx {
            min_multi_col_span.resize(idx + 1, 0);
        }
        min_multi_col_span[idx] = if min_multi_col_span[idx] == 0 {
            span
        } else {
            min_multi_col_span[idx].min(span)
        };
    }
    idx
}

/// Perl individual_padding：最大列宽 + individual padding + 行内补位。
#[allow(clippy::too_many_arguments)]
fn individual_padding(
    rows: &mut [Row],
    max_delimiter_width: &[usize],
    align_rows_without_max: bool,
    maximum_amp: usize,
    delim_justification: &str,
    dm: Option<&serde_yaml::Value>,
    max_column_width: &[usize],
    leading_blank: &mut i64,
) {
    let mut maximum_column_widths: Vec<usize> = Vec::new();
    let mut maximum_number_of_columns = 0usize;
    for row in rows.iter_mut() {
        for (j, cell) in row.cells.iter_mut().enumerate() {
            if !align_rows_without_max && row.n_amp < maximum_amp {
                cell.measure_this = false;
                cell.typ = "*".to_string();
            }
            // Perl dont_measure(mode=>"cell")
            if let Some(dm) = dm {
                let w = cell.width;
                let entry = cell.entry.clone();
                dont_measure_cell(dm, &entry, w, j, max_column_width, cell);
            }
            // 分隔符不等宽时按 delimiterJustification 补齐
            let max_d = max_delimiter_width.get(j).copied().unwrap_or(0);
            if cell.delimiter_length > 0 && cell.delimiter_length < max_d {
                let pad = max_d - cell.delimiter_length;
                if delim_justification == "right" {
                    cell.delimiter = format!("{}{}", " ".repeat(pad), cell.delimiter);
                } else {
                    cell.delimiter.push_str(&" ".repeat(pad));
                }
                cell.delimiter_length = max_d;
            }
            // Perl leadingBlankColumn 守卫：首列出现非空实测格即关闭该特性
            if j == 0 && *leading_blank > -1 && cell.width > 0 && cell.measure_this {
                *leading_blank = -1;
            }
            if !cell.measure_this {
                continue;
            }
            if maximum_column_widths.len() <= j {
                maximum_column_widths.resize(j + 1, 0);
            }
            maximum_column_widths[j] = maximum_column_widths[j].max(cell.width);
            maximum_number_of_columns = maximum_number_of_columns.max(j);
        }
    }
    for row in rows.iter_mut() {
        for j in 0..=maximum_number_of_columns {
            if j < row.cells.len() {
                let maximum = maximum_column_widths.get(j).copied().unwrap_or(0);
                row.cells[j].individual_padding += maximum.saturating_sub(row.cells[j].width);
            } else {
                row.cells.push(Cell::gap());
            }
        }
    }
}

/// Perl multicolumn_pre_check：同列多 multicolumn 的计量取舍与宽度找平。
fn multicolumn_pre_check(rows: &mut [Row], min_multi_col_span: &[usize]) {
    for row in rows.iter_mut() {
        for (j, cell) in row.cells.iter_mut().enumerate() {
            if let Some(d) = type_first_digit(&cell.typ) {
                let min = min_multi_col_span.get(j).copied().unwrap_or(0);
                if d as usize > min {
                    cell.typ = "X".to_string();
                    cell.measure_this = false;
                }
            }
        }
    }
    for ri in 0..rows.len() {
        for j in 0..rows[ri].cells.len() {
            let span = match type_full_span(&rows[ri].cells[j].typ) {
                Some(s) => s,
                None => continue,
            };
            for iri in 0..rows.len() {
                if iri == ri || j >= rows[iri].cells.len() {
                    continue;
                }
                let ok = matches!(type_first_digit(&rows[iri].cells[j].typ), Some(d) if d as usize >= span);
                if !ok {
                    continue;
                }
                let w_self = rows[ri].cells[j].width;
                let w_inner = rows[iri].cells[j].width;
                if w_self < w_inner {
                    let c = &mut rows[ri].cells[j];
                    c.typ = "X".to_string();
                    c.measure_this = false;
                    c.individual_padding = w_inner - w_self;
                } else {
                    let c = &mut rows[iri].cells[j];
                    c.typ = "X".to_string();
                    c.measure_this = false;
                    c.individual_padding = w_self - w_inner;
                }
            }
        }
    }
}

/// Perl multicolumn_padding：跨列分组宽度找平（groupPadding）。
fn multicolumn_padding(rows: &mut [Row], sb: usize, sa: usize, justification: &str) {
    for ri in 0..rows.len() {
        for j in 0..rows[ri].cells.len() {
            let span = match type_full_span(&rows[ri].cells[j].typ) {
                Some(s) => s,
                None => continue,
            };
            let offset = if justification == "left" {
                j + span - 1
            } else {
                j
            };
            let mut max_grouping_width = 0usize;
            for iri in 0..rows.len() {
                if iri == ri {
                    continue;
                }
                let mut gw = 0usize;
                for inner_j in 0..span {
                    let idx = j + inner_j;
                    if idx >= rows[iri].cells.len() {
                        continue;
                    }
                    let ic = &rows[iri].cells[idx];
                    if !ic.measure_this {
                        continue;
                    }
                    gw += ic.width + ic.individual_padding;
                    if inner_j < span - 1 {
                        gw += ic.delimiter_length;
                    }
                }
                gw += (span - 1) * (sb + sa);
                if offset < rows[iri].cells.len() {
                    rows[iri].cells[offset].grouping_width = gw;
                }
                max_grouping_width = max_grouping_width.max(gw);
            }
            for iri in 0..rows.len() {
                if iri == ri {
                    continue;
                }
                let w_self = rows[ri].cells[j].width;
                // 同跨列且未计量的 multicolumn：窄者补 groupPadding
                if j < rows[iri].cells.len() {
                    let ic = &rows[iri].cells[j];
                    let colspan_eq = type_full_span(&ic.col_span) == Some(span);
                    if colspan_eq
                        && !ic.measure_this
                        && max_grouping_width > ic.width
                        && ic.width >= w_self
                    {
                        let pad = max_grouping_width - ic.width;
                        let ic = &mut rows[iri].cells[j];
                        ic.individual_padding = 0;
                        ic.group_padding = pad;
                    }
                }
                if w_self > max_grouping_width && offset < rows[iri].cells.len() {
                    rows[iri].cells[offset].group_padding = w_self - max_grouping_width;
                }
            }
            if rows[ri].cells[j].width < max_grouping_width {
                let c = &mut rows[ri].cells[j];
                c.group_padding += max_grouping_width - c.width;
                c.individual_padding = 0;
            }
        }
    }
}

/// Perl multicolumn_post_check：被降级为 X 的 multicolumn 补 groupPadding。
fn multicolumn_post_check(rows: &mut [Row], sb: usize, sa: usize) {
    for ri in 0..rows.len() {
        for j in 0..rows[ri].cells.len() {
            let span = {
                let c = &rows[ri].cells[j];
                if c.typ != "X" {
                    continue;
                }
                match type_full_span(&c.col_span) {
                    Some(s) => s,
                    None => continue,
                }
            };
            let mut max_grouping_width: i64 = 0;
            for iri in 0..rows.len() {
                if iri == ri {
                    continue;
                }
                let mut gw: i64 = 0;
                let mut amps: i64 = -1;
                let mut inner_j = 0usize;
                while inner_j < span {
                    let idx = j + inner_j;
                    if idx >= rows[iri].cells.len() {
                        break;
                    }
                    let ic = &rows[iri].cells[idx];
                    if ic.typ == "*" || ic.typ == "-" {
                        inner_j += 1;
                        continue;
                    }
                    gw += (ic.width + ic.individual_padding) as i64;
                    amps += 1;
                    if amps > 0 {
                        gw += ic.delimiter_length as i64;
                    }
                    if let Some(cs) = type_full_span(&ic.col_span) {
                        inner_j += cs.saturating_sub(1);
                    }
                    inner_j += 1;
                }
                gw += amps * (sb + sa) as i64;
                max_grouping_width = max_grouping_width.max(gw);
            }
            let c = &mut rows[ri].cells[j];
            c.individual_padding = 0;
            c.group_padding = (max_grouping_width - c.width as i64).max(0) as usize;
        }
    }
}

fn ends_with_delim(line: &str, delim: &Regex) -> bool {
    for m in delim.find_iter(line.as_bytes()).flatten() {
        if m.end() == line.len() {
            return true;
        }
    }
    false
}

/// Perl hidden_child_cell_row_width：隐藏/verbatim 内容按原文重建后取最大行宽。
fn cell_width(entry: &str, hidden: &[(String, String, usize)]) -> usize {
    let mut s = entry.to_string();
    for (id, content, _) in hidden {
        if s.contains(id.as_str()) {
            s = s.replacen(id.as_str(), content, 1);
        }
    }
    // verbatim token 同样展开为存储内容计量
    let t = tok(|t| t.clone());
    for token_prefix in [&t.verbatim, &t.verbatim_inline, &t.preamble] {
        if s.contains(token_prefix.as_str()) {
            let ids: Vec<(String, String)> = state::storage(|st| {
                st.verbatim
                    .iter()
                    .map(|(k, v)| (k.clone(), format!("{}{}{}", v.begin, v.body, v.end)))
                    .collect()
            });
            for (k, v) in ids {
                if s.contains(&k) {
                    s = s.replacen(&k, &v, 1);
                }
            }
        }
    }
    if s.contains('\n') {
        s.split('\n').map(width).max().unwrap_or(0)
    } else {
        width(&s)
    }
}

/// Perl hidden_child_row_width：含 token 的行按隐藏内容重建多行（后续行以点号对齐
/// 到 token 前列宽、逐行剥尾随 `\\`）后取最大行宽；begin 首行并入计量。
#[allow(clippy::too_many_arguments)]
fn hidden_child_row_width(
    tmp_row: &str,
    row_index: usize,
    row_width_default: usize,
    hidden: &[(String, String, usize)],
    begin_text: &str,
    begin_ok: bool,
) -> usize {
    let t = tok(|t| t.clone());
    let mut tmp = tmp_row.to_string();
    let mut length_of_begin = 0usize;
    if begin_ok {
        length_of_begin = width(begin_text);
        if row_index == 0 {
            tmp = format!("{begin_text}{tmp}");
        }
    }
    if !tmp.contains(t.begin_of_token.as_str()) {
        return if begin_ok { width(&tmp) } else { row_width_default };
    }
    tmp = format!("{}{}", ".".repeat(length_of_begin), tmp);
    for (id, content, _) in hidden {
        if let Some(pos) = tmp.find(id.as_str()) {
            let line_start = tmp[..pos].rfind('\n').map(|i| i + 1).unwrap_or(0);
            let len_before = width(&tmp[line_start..pos]);
            let joined: String = content
                .split('\n')
                .collect::<Vec<_>>()
                .join(&format!("\n{}", ".".repeat(len_before)));
            let re_dbs_tail = crate::patterns::re_new(r"(?s)\\\\\h*$").unwrap();
            let joined = joined
                .split('\n')
                .map(|l| replace_all_with(l, &re_dbs_tail, |_| String::new()))
                .collect::<Vec<_>>()
                .join("\n");
            tmp = tmp.replacen(id.as_str(), &joined, 1);
        }
    }
    for token_prefix in [&t.verbatim, &t.verbatim_inline, &t.preamble] {
        if tmp.contains(token_prefix.as_str()) {
            let ids: Vec<(String, String)> = state::storage(|st| {
                st.verbatim
                    .iter()
                    .map(|(k, v)| (k.clone(), format!("{}{}{}", v.begin, v.body, v.end)))
                    .collect()
            });
            for (k, v) in ids {
                if tmp.contains(&k) {
                    tmp = tmp.replacen(&k, &v, 1);
                }
            }
        }
    }
    if tmp.contains('\n') {
        tmp.split('\n').map(width).max().unwrap_or(0)
    } else {
        width(&tmp)
    }
}

/// 每个行首加前缀（Perl `s"^"X"mg`：含末尾换行后的空行行首）。
fn prefix_lines(s: &str, p: &str) -> String {
    if p.is_empty() {
        return s.to_string();
    }
    s.split('\n')
        .map(|l| format!("{p}{l}"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn strip_first(s: &str, p: &str) -> String {
    if p.is_empty() {
        return s.to_string();
    }
    s.strip_prefix(p).unwrap_or(s).to_string()
}

/// _align_mark_down_block：`%* \begin{X}...%* \end{X}` 标记注释对齐块
/// （对应 AlignmentAtAmpersand.pm::_align_mark_down_block）。
pub fn align_mark_down_block(body: &str) -> String {
    let mut body = body.to_string();
    // 1. 把标记注释 token 还原为文本（仅限仍出现在 body 中的）
    let mut returned: Vec<(String, String)> = Vec::new();
    state::storage(|s| {
        for c in &s.trailing_comments {
            if c.align_markup && body.contains(&c.id) {
                body = body.replacen(&c.id, &c.value, 1);
                returned.push((c.id.clone(), c.value.clone()));
            }
        }
    });
    // 2. 对每个启用的 lookForAlignDelims 名称做块替换
    let names: Vec<String> = state::settings(|s| {
        let mut v = Vec::new();
        if let Some(Value::Mapping(m)) = get(&s.root, "lookForAlignDelims") {
            for (k, val) in m {
                let enabled = match val.as_mapping() {
                    Some(mm) => match get(mm, "delims") {
                        Some(d) => as_bool(Some(d)),
                        None => true,
                    },
                    None => as_bool(Some(val)),
                };
                if enabled {
                    if let Some(name) = k.as_str() {
                        v.push(name.to_string());
                    }
                }
            }
        }
        v
    });
    for name in names {
        // 名称按 Perl 语义原样插值（如 align* 为正则）
        let re = match crate::patterns::re_new(&format!(
            r"(?sm)(\h*)((?!<\\)%\*\h*\\begin\{{({name})\}})(.*?\R)\h*((?!<\\)%\*\h*\\end\{{\3\}})"
        )) {
            Ok(r) => r,
            Err(_) => continue,
        };
        body = replace_all_with(&body, &re, |caps| {
            let indentation = gn(caps, 1);
            let begin = gn(caps, 2);
            let block_name = gn(caps, 3);
            let mut block_body = gn(caps, 4).to_string();
            let end = gn(caps, 5);
            let os = state::settings_mut(|s| {
                s.settings_for_object(block_name, "environments", false, false)
            });
            let mut dummy = String::new();
            align_at_ampersand(&mut dummy, &mut block_body, &os);
            block_body = prefix_lines(&block_body, indentation);
            block_body = strip_first(&block_body, indentation);
            block_body = prefix_lines(&block_body, &os.indentation);
            block_body = strip_first(&block_body, &os.indentation);
            // Perl: s"\s*$"\n"
            block_body = format!("{}\n", block_body.trim_end_matches(char::is_whitespace));
            format!("{begin}{block_body}{end}")
        });
    }
    // 3. 标记注释重新 token 化
    let not_preceded_by = state::settings(|s| {
        as_str(get_path(
            &s.root,
            &["fineTuning", "trailingComments", "notPrecededBy"],
        ))
        .unwrap_or(r"(?<!\\)")
        .to_string()
    });
    for (id, value) in returned {
        let re = match crate::patterns::re_new(&format!(
            r"(?s){not_preceded_by}%{}",
            crate::trailing_comments::regex_esc(&value)
        )) {
            Ok(r) => r,
            Err(_) => continue,
        };
        if let Ok(Some(c)) = re.captures(body.as_bytes()) {
            if let Some(m) = c.get(0) {
                body = format!("{}%{}{}", &body[..m.start()], id, &body[m.end()..]);
            }
        }
    }
    body
}
