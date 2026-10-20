//! 文本折行（textWrapOptions）与行选择（-n/--lines）测试。
mod common;
use common::{check, check_owned, check_overwrite, report};

// ---- text-wrap ----
#[test]
fn long_lines1() {
    check("text-wrap", "long-lines1.tex", &["-s", "-o=+-default"], "long-lines1-default.tex");
}

#[test]
fn long_lines2() {
    check("text-wrap", "long-lines2.tex", &["-s", "-o=+-default"], "long-lines2-default.tex");
}

// ---- 行选择 ----
#[test]
fn lines_3_5() {
    check(
        "line-switch-test-cases",
        "environments-simple-nested.tex",
        &["--lines", "3-5", "-s", "-o=+-mod1"],
        "environments-simple-nested-mod1.tex",
    );
}

#[test]
#[ignore = "known gap: -r substitution 带 /e（text-wrap-4.yaml 的 ruler 替换是 Perl eval 代码块），用户明确暂缓"]
fn text_wrap_jowens_short_ruler() {
    check("text-wrap", "jowens-short.tex", &["-s", "-o=+-mod-4", "-l=text-wrap-4", "-m", "-r"], "jowens-short-mod-4.tex");
}

#[test]
fn text_wrap_jowens() {
    check("text-wrap", "jowens-long.tex", &["-s", "-o=+-mod-4", "-l=text-wrap-4", "-m"], "jowens-long-mod-4.tex");
    check("text-wrap", "long-lines1.tex", &["-s", "-o=+-mod-6", "-l=text-wrap-4,not-stop-comment-block", "-m"], "long-lines1-mod-6.tex");
    check("text-wrap", "long-lines4.tex", &["-s", "-o=+-mod-6", "-l=text-wrap-6", "-m"], "long-lines4-mod-6.tex");
}

#[test]
fn text_wrap_verbatim() {
    check("text-wrap", "verbatim-long-lines.tex", &["-s", "-m", "-o=verbatim-long-lines-mod1", "-l=text-wrap-1", "-y=modifyLineBreaks:textWrapOptions:huge:wrap"], "verbatim-long-lines-mod1.tex");
    check("text-wrap", "verbatim2.tex", &["-s", "-m", "-o=+-mod1", "-l=verbatim1"], "verbatim2-mod1.tex");
    check("text-wrap", "verbatim2.tex", &["-s", "-m", "-o=+-mod2", "-l=verbatim2"], "verbatim2-mod2.tex");
}

#[test]
fn remove_para_line_breaks() {
    check("text-wrap", "jowens-short-multi-line.tex", &["-m", "-s", "-l=removeParaLineBreaks0", "-o=+0"], "jowens-short-multi-line0.tex");
    check("text-wrap", "jowens-short-multi-line.tex", &["-m", "-s", "-l=removeParaLineBreaks1", "-o=+1"], "jowens-short-multi-line1.tex");
    check("text-wrap", "multi-object.tex", &["-m", "-s", "-l=removeParaLineBreaks0", "-o=+0"], "multi-object0.tex");
    check("text-wrap", "trailingComments.tex", &["-m", "-s", "-l=removeParaLineBreaks0", "-o=+0"], "trailingComments0.tex");
    check("text-wrap", "chapter-file.tex", &["-m", "-s", "-l=removeParaLineBreaks0", "-o=+0"], "chapter-file0.tex");
}

/// 上游 L83-85：`cp indent.log file-contents-warn1.txt` + 两处 `perl -p0i` 截取
/// （从首个 `WARN:  Obsolete` 到首个 `INFO:`）。
#[test]
fn text_wrap_file_contents_warn_log() {
    let case = common::run(
        "text-wrap",
        "jowens-short-multi-line-stop.tex",
        &[
            "-m",
            "-s",
            "-l=removeParaLineBreaks0,stopParaAtFileContents",
            "-o=+-mod10",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect::<Vec<_>>(),
    );
    case.assert_success();
    case.assert_log_section(
        "indent.log",
        "WARN:  Obsolete",
        Some("INFO:"),
        "file-contents-warn1.txt",
    );
}

#[test]
fn text_wrap_misc() {
    check("text-wrap", "key-equals-value.tex", &["-m", "-s", "-l=key-equals-value", "-o=+0"], "key-equals-value0.tex");
    check("text-wrap", "unnamed-group-braces.tex", &["-m", "-s", "-l=removeParaLineBreaks0,unnamed-braces", "-o=+0"], "unnamed-group-braces0.tex");
    check("text-wrap", "jowens-short-multi-line-stop.tex", &["-m", "-s", "-l=removeParaLineBreaks0", "-o=+-mod0"], "jowens-short-multi-line-stop-mod0.tex");
}

// ---------------------------------------------------------------------------
// T1 搬运：上游 latexindent.pl-main/test-cases/text-wrap/text-wrap-remove-PLB.sh
// ---------------------------------------------------------------------------
//
// 口径：唯一有效判据 = `tests/wrap.rs` 里有无对应断言（**不得**用 tests/cases 有无基准文件判断）。
// 上游 180 次调用（两个 for 循环已展开：`i=1..5` × 3 = 15 条，`i=1..7` = 7 条）：
//   - 已搬运 17（另有 1 条 `lines_3_5` 属 line-switch-test-cases，计本地多余）
//   - 依赖 T2（`-r` + addruler*/issue-359*/issue-444*/issue-450* 的 `/e` ruler 替换）：64 条，跳过
//   - 产物上游未提交：2 条（L165 issue-362-default、L285 issue-552-default），跳过（A2）
//   - 其余 96 个产物 / 97 条调用（issue-341-mod1 由 L161 `-o` 与 L162 `-w` 两条产出）在此补齐。
//

/// 已无待修用例：22 条 `PENDING` 全部修好（见 doc/pending-work.md §5A.5），列表清空。
const PENDING: &[&str] = &[];

const HUGE: &str = "-y=modifyLineBreaks:textWrapOptions:huge:wrap";
const COLS_M1: &str = "-y=modifyLineBreaks:textWrapOptions:columns:-1";

struct WCase {
    group: &'static str,
    input: String,
    args: Vec<String>,
    expected: String,
}

fn owned(args: &[&str]) -> Vec<String> {
    args.iter().map(|s| s.to_string()).collect()
}

fn case(group: &'static str, input: &str, args: &[&str], expected: &str) -> WCase {
    WCase {
        group,
        input: input.to_string(),
        args: owned(args),
        expected: expected.to_string(),
    }
}

/// 上游脚本尚未搬成断言的全部调用（96 个产物 / 97 条调用）。
fn build_cases() -> Vec<WCase> {
    let mut v: Vec<WCase> = Vec::new();

    // L17-25：`for (( i=1; i<=5; i++ ))` × long-lines1/2/3
    for i in 1..=5 {
        let o = format!("-o=+-mod-{i}");
        let l = format!("-l=text-wrap-{i}");
        for base in ["long-lines1", "long-lines2", "long-lines3"] {
            v.push(case(
                "long-lines",
                &format!("{base}.tex"),
                &["-s", &o, &l, "-m", HUGE],
                &format!("{base}-mod-{i}.tex"),
            ));
        }
    }
    // L33：commentOnPreviousLine:0
    v.push(case(
        "long-lines",
        "long-lines4.tex",
        &[
            "-s",
            "-o=+-mod-7",
            "-l=text-wrap-6",
            "-m",
            "-y=modifyLineBreaks:textWrapOptions:blocksFollow:commentOnPreviousLine:0",
        ],
        "long-lines4-mod-7.tex",
    ));

    // L39 / L141 / L150-155：verbatim 族
    v.push(case(
        "verbatim",
        "verbatim2.tex",
        &["-s", "-m", "-o=+-mod3", "-l=verbatim3"],
        "verbatim2-mod3.tex",
    ));
    v.push(case(
        "verbatim",
        "verbatim3.tex",
        &["-s", "-l=textwrap-most-useful,verbatim", "-m", "-o=+-mod1"],
        "verbatim3-mod1.tex",
    ));
    for (n, shell, out) in [
        ("1", "commandshell1", "1"),
        ("2", "commandshell2", "2"),
        ("3", "commandshell3", "3"),
    ] {
        v.push(case(
            "verbatim",
            "verbatim4.tex",
            &["-s", "-m", &format!("-l={shell}"), &format!("-o=+-mod{out}")],
            &format!("verbatim4-mod{n}.tex"),
        ));
    }
    v.push(case(
        "verbatim",
        "verbatim5.tex",
        &["-s", "-m", "-l=commandshell2", "-o=+-mod2"],
        "verbatim5-mod2.tex",
    ));
    v.push(case(
        "verbatim",
        "verbatim6.tex",
        &["-s", "-m", "-l=commandshell1", "-o=+-mod1"],
        "verbatim6-mod1.tex",
    ));

    // L44-46 / L50-52 / L56 / L60 / L63：removeParaLineBreaks 族
    for (n, out) in [("2", "2"), ("3", "3"), ("4", "4")] {
        v.push(case(
            "plb",
            "jowens-short-multi-line.tex",
            &[
                "-m",
                "-s",
                &format!("-l=removeParaLineBreaks{n}"),
                &format!("-o=+{out}"),
            ],
            &format!("jowens-short-multi-line{n}.tex"),
        ));
    }
    for (n, out) in [("5", "5"), ("7", "7"), ("9", "9")] {
        v.push(case(
            "plb",
            "multi-object.tex",
            &[
                "-m",
                "-s",
                &format!("-l=removeParaLineBreaks{n}"),
                &format!("-o=+{out}"),
            ],
            &format!("multi-object{n}.tex"),
        ));
    }
    v.push(case(
        "plb",
        "trailingComments.tex",
        &[
            "-m",
            "-s",
            "-l=removeParaLineBreaks0,unprotect-blank-lines",
            "-o=+-unprotect0",
        ],
        "trailingComments-unprotect0.tex",
    ));
    v.push(case(
        "plb",
        "chapter-file.tex",
        &[
            "-m",
            "-s",
            "-l=removeParaLineBreaks0,unprotect-blank-lines",
            "-o=+unprotect0",
        ],
        "chapter-fileunprotect0.tex",
    ));
    v.push(case(
        "plb",
        "chapter-file-par.tex",
        &["-m", "-s", "-l=removeParaLineBreaks0", "-o=chapter-file-par0"],
        "chapter-file-par0.tex",
    ));

    // L73-82：paragraph stop 族
    v.push(case(
        "para-stop",
        "jowens-short-multi-line-stop.tex",
        &[
            "-m",
            "-s",
            "-l=removeParaLineBreaks0,stopParaAtCommands",
            "-o=+-mod1",
        ],
        "jowens-short-multi-line-stop-mod1.tex",
    ));
    v.push(case(
        "para-stop",
        "jowens-short-multi-line-stop.tex",
        &[
            "-m",
            "-s",
            "-l=removeParaLineBreaks0,noStopParaAtEnvironments",
            "-o=+-mod2",
        ],
        "jowens-short-multi-line-stop-mod2.tex",
    ));
    v.push(case(
        "para-stop",
        "jowens-short-multi-line-stop.tex",
        &[
            "-m",
            "-s",
            "-l=removeParaLineBreaks0,../headings/levels1",
            "-o=+-mod7",
        ],
        "jowens-short-multi-line-stop-mod7.tex",
    ));
    v.push(case(
        "para-stop",
        "jowens-short-multi-line-stop.tex",
        &[
            "-m",
            "-s",
            "-l=removeParaLineBreaks0,../headings/levels1",
            "-o=jowens-short-multi-line-stop-mod8",
        ],
        "jowens-short-multi-line-stop-mod8.tex",
    ));
    v.push(case(
        "para-stop",
        "jowens-short-multi-line-stop.tex",
        &[
            "-m",
            "-s",
            "-l=removeParaLineBreaks0,../headings/levels1,removeParaLineBreaks17",
            "-o=+-mod9",
        ],
        "jowens-short-multi-line-stop-mod9.tex",
    ));
    v.push(case(
        "para-stop",
        "jowens-short-multi-line-stop.tex",
        &[
            "-m",
            "-s",
            "-l=removeParaLineBreaks0,stopParaAtFileContents",
            "-o=+-mod10",
        ],
        "jowens-short-multi-line-stop-mod10.tex",
    ));

    // L88-92
    v.push(case(
        "follow-ups",
        "jowens-follow-ups.tex",
        &[
            "-m",
            "-s",
            "-l=removeParaLineBreaks0,jowens-follow-up",
            "-o=+-mod0",
        ],
        "jowens-follow-ups-mod0.tex",
    ));
    v.push(case(
        "follow-ups",
        "jowens-follow-ups.tex",
        &[
            "-m",
            "-s",
            "-l=removeParaLineBreaks0,jowens-follow-up,description",
            "-o=+-mod1",
        ],
        "jowens-follow-ups-mod1.tex",
    ));
    v.push(case(
        "follow-ups",
        "bug1.tex",
        &["-m", "-s", "-l=lower-alph", "-o=+-mod0"],
        "bug1-mod0.tex",
    ));

    // L96-114：issue-103 相关的一批折行用例
    for (n, out) in [("1", "1"), ("4", "4")] {
        v.push(case(
            "zoehneto",
            "zoehneto.tex",
            &[
                "-m",
                "-s",
                &format!("-l=zoehneto{n}"),
                &format!("-o=+-mod{out}"),
            ],
            &format!("zoehneto-mod{n}.tex"),
        ));
    }
    v.push(case(
        "zoehneto",
        "zoehneto.tex",
        &["-m", "-s", "-l=zoehneto9", "-o=+-mod9", HUGE],
        "zoehneto-mod9.tex",
    ));
    v.push(case(
        "zoehneto",
        "zoehneto1.tex",
        &["-m", "-s", "-l=zoehneto1", "-o=+-mod1"],
        "zoehneto1-mod1.tex",
    ));
    v.push(case(
        "zoehneto",
        "zoehneto2.tex",
        &["-m", "-s", "-l=zoehneto3", "-o=+-mod3"],
        "zoehneto2-mod3.tex",
    ));
    v.push(case(
        "zoehneto",
        "zoehneto1.tex",
        &["-m", "-s", "-l=zoehneto4", "-o=+-mod4"],
        "zoehneto1-mod4.tex",
    ));
    v.push(case(
        "zoehneto",
        "zoehneto1.tex",
        &["-m", "-s", "-l=zoehneto7", "-o=+-mod7"],
        "zoehneto1-mod7.tex",
    ));
    v.push(case(
        "zoehneto",
        "environments-nested-fourth.tex",
        &["-m", "-s", "-l=env1", "-o=+-mod1"],
        "environments-nested-fourth-mod1.tex",
    ));
    v.push(case(
        "zoehneto",
        "dbmrq.tex",
        &["-m", "-s", "-l=dbmrq1", "-o=+-mod1"],
        "dbmrq-mod1.tex",
    ));
    v.push(case(
        "zoehneto",
        "zoehneto3.tex",
        &["-m", "-s", "-l=zoehneto-config1", "-o=+-mod1"],
        "zoehneto3-mod1.tex",
    ));
    v.push(case(
        "zoehneto",
        "zoehneto4.tex",
        &["-m", "-s", "-l=zoehneto11", "-o=+-mod11", HUGE],
        "zoehneto4-mod11.tex",
    ));
    v.push(case(
        "zoehneto",
        "zoehneto3.tex",
        &["-m", "-s", "-l=zoehneto17", "-o=+-mod17"],
        "zoehneto3-mod17.tex",
    ));
    for (n, out) in [("21", "21"), ("22", "22"), ("23", "23")] {
        v.push(case(
            "zoehneto",
            "zoehneto5.tex",
            &[
                "-m",
                "-s",
                &format!("-l=zoehneto{n}"),
                &format!("-o=+-mod{out}"),
            ],
            &format!("zoehneto5-mod{n}.tex"),
        ));
    }

    // L117-119
    v.push(case(
        "waschk",
        "waschk.tex",
        &["-s", "-m", "-l=waschk", "-o=+-mod", HUGE],
        "waschk-mod.tex",
    ));
    v.push(case(
        "waschk",
        "waschk.tex",
        &["-s", "-m", "-l=waschk0", "-o=+-mod0"],
        "waschk-mod0.tex",
    ));
    v.push(case(
        "waschk",
        "waschk.tex",
        &["-s", "-m", "-l=waschk1", "-o=+-mod1"],
        "waschk-mod1.tex",
    ));

    // L122-139
    v.push(case(
        "issue-183-228",
        "issue-183.tex",
        &["-s", "-m", "-l=issue-183-mod1", "-o=+-mod1"],
        "issue-183-mod1.tex",
    ));
    v.push(case(
        "issue-183-228",
        "issue-183.tex",
        &["-s", "-m", "-l=issue-183-mod2", "-o=+-mod2"],
        "issue-183-mod2.tex",
    ));
    v.push(case(
        "issue-183-228",
        "issue-279.tex",
        &["-s", "-l=issue-279", "-m", "-o=+-mod1"],
        "issue-279-mod1.tex",
    ));
    v.push(case(
        "issue-183-228",
        "issue-158.tex",
        &["-s", "-l=textwrap-most-useful", "-m", "-o=+-mod1"],
        "issue-158-mod1.tex",
    ));
    v.push(case(
        "issue-183-228",
        "issue-158.tex",
        &["-s", "-l=issue-158-mod4", "-m", "-o=+-mod4"],
        "issue-158-mod4.tex",
    ));
    v.push(case(
        "issue-183-228",
        "issue-158.tex",
        &["-s", "-l=issue-158-mod5", "-m", "-o=+-mod5"],
        "issue-158-mod5.tex",
    ));
    v.push(case(
        "issue-183-228",
        "issue-228.tex",
        &["-s", "-l=textwrap-most-useful,document", "-m", "-o=+-mod1"],
        "issue-228-mod1.tex",
    ));
    v.push(case(
        "issue-183-228",
        "issue-228.tex",
        &["-s", COLS_M1, "-m", "-o=+-mod3"],
        "issue-228-mod3.tex",
    ));
    v.push(case(
        "issue-183-228",
        "headings1.tex",
        &["-s", "-l=textwrap-most-useful", "-m", "-o=+-mod1"],
        "headings1-mod1.tex",
    ));
    v.push(case(
        "issue-183-228",
        "headings2.tex",
        &["-s", "-l=textwrap-most-useful", "-m", "-o=+-mod1"],
        "headings2-mod1.tex",
    ));
    v.push(case(
        "issue-183-228",
        "headings1.tex",
        &["-s", "-l=textwrap-most-useful,headings2", "-m", "-o=+-mod2"],
        "headings1-mod2.tex",
    ));

    // L158-161
    v.push(case(
        "issue-337-341",
        "issue-337.tex",
        &["-s", "-m", "-l=issue-337", "-o=+-mod1"],
        "issue-337-mod1.tex",
    ));
    v.push(case(
        "issue-337-341",
        "issue-337.tex",
        &["-s", "-m", "-l=issue-337a", "-o=+-mod2"],
        "issue-337-mod2.tex",
    ));
    v.push(case(
        "issue-337-341",
        "issue-344.tex",
        &["-s", "-m", "-l=issue-344", "-o=+-mod1"],
        "issue-344-mod1.tex",
    ));
    v.push(case(
        "issue-337-341",
        "issue-341.tex",
        &["-s", "-m", "-l=issue-341", "-o=+-mod1"],
        "issue-341-mod1.tex",
    ));

    // L167-169 与 L172-177 的 `for i in {1..7}`
    for f in ["issue-367", "issue-367a", "issue-367b"] {
        v.push(case(
            "columns-m1",
            &format!("{f}.tex"),
            &["-s", "-m", COLS_M1, "-o=+-mod1"],
            &format!("{f}-mod1.tex"),
        ));
    }
    for i in 1..=7 {
        v.push(case(
            "columns-m1",
            &format!("tw-tc{i}.tex"),
            &["-s", "-m", COLS_M1, "-o=+-mod1"],
            &format!("tw-tc{i}-mod1.tex"),
        ));
    }

    // L242 / L271-275（非 `-r` 族）
    v.push(case(
        "issue-4xx",
        "issue-412.tex",
        &["-s", "-m", "-l=issue-412", "-o=+-mod1"],
        "issue-412-mod1.tex",
    ));
    v.push(case(
        "issue-4xx",
        "issue-471.tex",
        &["-s", "-m", "-l=issue-471", "-o=+-mod1"],
        "issue-471-mod1.tex",
    ));
    v.push(case(
        "issue-4xx",
        "issue-487.tex",
        &["-s", "-m", "-l=issue-487", "-o=+-mod1"],
        "issue-487-mod1.tex",
    ));
    v.push(case(
        "issue-4xx",
        "issue-495.tex",
        &["-s", "-m", "-l=issue-495", "-o=+-mod1"],
        "issue-495-mod1.tex",
    ));

    // L283-295（issue-556-mod2 / issue-608-mod2 虽带 `-r`，其 YAML 无 `/e`，不依赖 T2）
    v.push(case(
        "issue-5xx-6xx",
        "issue-506.tex",
        &["-s", "-l=issue-506", "-m", "-o=+-mod1"],
        "issue-506-mod1.tex",
    ));
    v.push(case(
        "issue-5xx-6xx",
        "issue-553.tex",
        &["-s", "-l=issue-553", "-m", "-o=+-mod1"],
        "issue-553-mod1.tex",
    ));
    v.push(case(
        "issue-5xx-6xx",
        "issue-556.tex",
        &["-s", "-l=issue-556", "-m", "-o=+-mod1"],
        "issue-556-mod1.tex",
    ));
    v.push(case(
        "issue-5xx-6xx",
        "issue-556.tex",
        &["-s", "-l=issue-556a", "-m", "-r", "-o=+-mod2"],
        "issue-556-mod2.tex",
    ));
    v.push(case(
        "issue-5xx-6xx",
        "issue-562.tex",
        &["-s", "-l=issue-562", "-m", "-o=+-mod1"],
        "issue-562-mod1.tex",
    ));
    v.push(case(
        "issue-5xx-6xx",
        "issue-608.tex",
        &["-s", "-l=issue-608.yaml", "-m", "-o=+-mod1"],
        "issue-608-mod1.tex",
    ));
    v.push(case(
        "issue-5xx-6xx",
        "issue-608.tex",
        &["-s", "-r", "-l=issue-608a.yaml", "-m", "-o=+-mod2"],
        "issue-608-mod2.tex",
    ));
    v.push(case(
        "issue-5xx-6xx",
        "sec-intro.tex",
        &["-s", "-l=sec-intro", "-m", "-o=+-mod1"],
        "sec-intro-mod1.tex",
    ));

    v
}

/// 跑一组（跳过 `PENDING` 里的用例）。
fn run_group(group: &str) {
    for c in build_cases() {
        if c.group != group || PENDING.contains(&c.expected.as_str()) {
            continue;
        }
        check_owned("text-wrap", &c.input, &c.args, &c.expected);
    }
}

// ---------------------------------------------------------------------------
// T2 依赖用例：上游脚本中 64 条 `-r` + `/e`（ruler 替换）调用 + 2 条无产物调用
// （`doc/verify/09-test-porting-audit.md` §3.2；本次搬运前完全未写成断言）。
//
// - 7 条 `tw-tc1..7`（`columns:-1`，不含 `/e`）本来就已由 `columns-m1` 组覆盖，
//   这里仅为对齐上游脚本行号而重复列出，归入非 ignore 的 `text_wrap_e_dependent`。
// - 其余 57 条全部依赖 `-r` substitution 的 Perl `/e`（addruler1/2/3、
//   issue-359*、issue-444*、issue-450*、issue-389* 等 YAML），当前引擎不支持 `/e`
//   → 归入 `text_wrap_e_dependent_pending`（`#[ignore]`），待 T2 决策后统一启用。
// ---------------------------------------------------------------------------

/// 一条 T2 用例：输入、参数、产物（基准）。
fn build_e_cases() -> Vec<WCase> {
    let mut v: Vec<WCase> = Vec::new();
    let cols = |n: u32| format!("modifyLineBreaks:textWrapOptions:columns:{n}");

    // L143-148：textwrap-bfccb / bfccb1 / bfccb2（addruler1 / addruler2）
    v.push(case("e-dep", "textwrap-bfccb.tex",
        &["-s", "-l", "textwrap-most-useful,test1,addruler1", "-m", "-r", "-o=+-mod1"], "textwrap-bfccb-mod1.tex"));
    v.push(case("e-dep", "textwrap-bfccb.tex",
        &["-s", "-l", "textwrap-most-useful,addruler1,test2", "-m", "-r", "-o=+-mod2"], "textwrap-bfccb-mod2.tex"));
    v.push(case("e-dep", "textwrap-bfccb1.tex",
        &["-s", "-l", "textwrap-most-useful,test1,addruler1", "-m", "-r", "-o=+-mod1"], "textwrap-bfccb1-mod1.tex"));
    v.push(case("e-dep", "textwrap-bfccb1.tex",
        &["-s", "-l", "textwrap-most-useful,test2,addruler1", "-m", "-r", "-o=+-mod2"], "textwrap-bfccb1-mod2.tex"));
    v.push(case("e-dep", "textwrap-bfccb1.tex",
        &["-s", "-l", "textwrap-most-useful,test3,addruler1", "-m", "-r", "-o=+-mod3"], "textwrap-bfccb1-mod3.tex"));
    v.push(case("e-dep", "textwrap-bfccb2.tex",
        &["-s", "-l", "textwrap-most-useful-preamble,addruler2", "-m", "-r", "-o=+-mod1"], "textwrap-bfccb2-mod1.tex"));

    // L170-177：`for i in {1..7}` tw-tc（columns:-1，无 `/e`，已由 columns-m1 组覆盖）
    for i in 1..=7 {
        v.push(case("e-dep", &format!("tw-tc{i}.tex"),
            &["-s", "-m", COLS_M1, "-o=+-mod1"],
            &format!("tw-tc{i}-mod1.tex")));
    }

    // L181-209：issue-359 族（`-r` + addruler 系列）
    let c80 = cols(80).to_string();
    let c60 = cols(60).to_string();
    let c40 = cols(40).to_string();
    v.push(case("e-dep", "issue-306.tex",
        &["-s", "-r", "-m", "-l", "issue-359a", "-y", &c80, "-o=+-mod1"], "issue-306-mod1.tex"));
    for (input, y, o, out) in [
        ("issue-359a.tex", None, "-o=+-mod1", "issue-359a-mod1.tex"),
        ("issue-359b.tex", None, "-o=+-mod1", "issue-359b-mod1.tex"),
        ("issue-359c.tex", None, "-o=+-mod1", "issue-359c-mod1.tex"),
        ("issue-359a.tex", Some(c40.as_str()), "-o=+-mod2", "issue-359a-mod2.tex"),
        ("issue-359b.tex", Some(c40.as_str()), "-o=+-mod2", "issue-359b-mod2.tex"),
        ("issue-359c.tex", Some(c40.as_str()), "-o=+-mod2", "issue-359c-mod2.tex"),
        ("issue-359a.tex", Some(c60.as_str()), "-o=+-mod3", "issue-359a-mod3.tex"),
        ("issue-359b.tex", Some(c60.as_str()), "-o=+-mod3", "issue-359b-mod3.tex"),
        ("issue-359c.tex", Some(c60.as_str()), "-o=+-mod3", "issue-359c-mod3.tex"),
    ] {
        let mut args = vec!["-s", "-r", "-m", "-l", "issue-359a"];
        if let Some(y) = y {
            args.push("-y");
            args.push(y);
        }
        args.push(o);
        v.push(case("e-dep", input, &args, out));
    }
    v.push(case("e-dep", "issue-359a.tex",
        &["-s", "-r", "-m", "-l", "issue-359d", "-y", &c60, "-o=+-mod5"], "issue-359a-mod5.tex"));
    v.push(case("e-dep", "issue-359a.tex",
        &["-s", "-r", "-m", "-l", "issue-359e", "-y", &c60, "-o=+-mod6"], "issue-359a-mod6.tex"));
    v.push(case("e-dep", "issue-359a.tex",
        &["-s", "-r", "-m", "-l", "issue-359b", "-o=+-mod4"], "issue-359a-mod4.tex"));
    v.push(case("e-dep", "issue-359d.tex",
        &["-s", "-r", "-m", "-l", "issue-359b", "-o=+-mod1"], "issue-359d-mod1.tex"));
    v.push(case("e-dep", "issue-359d.tex",
        &["-s", "-r", "-m", "-l", "issue-359b", "-y", &c40, "-o=+-mod2"], "issue-359d-mod2.tex"));
    v.push(case("e-dep", "issue-359e.tex",
        &["-s", "-r", "-m", "-l", "issue-359b", "-o=+-mod1"], "issue-359e-mod1.tex"));
    v.push(case("e-dep", "issue-359e.tex",
        &["-s", "-r", "-m", "-l", "issue-359c", "-o=+-mod2"], "issue-359e-mod2.tex"));

    // L211-212：issue-356（addruler3）
    v.push(case("e-dep", "issue-356.tex",
        &["-s", "-r", "-m", "-l", "issue-356,addruler3", "-o=+-mod1"], "issue-356-mod1.tex"));
    v.push(case("e-dep", "issue-356.tex",
        &["-s", "-r", "-m", "-l", "issue-356a,addruler3", "-o=+-mod2"], "issue-356-mod2.tex"));

    // L215-240：issue-389a..l（wrap-comments* + addruler1），mod 号不连续，按产物名推导
    for (input, l, out) in [
        ("issue-389a.tex", "wrap-comments,addruler1", "issue-389a-mod1.tex"),
        ("issue-389b.tex", "wrap-comments,addruler1", "issue-389b-mod1.tex"),
        ("issue-389c.tex", "wrap-comments,addruler1", "issue-389c-mod1.tex"),
        ("issue-389c.tex", "wrap-comments2,addruler1", "issue-389c-mod2.tex"),
        ("issue-389d.tex", "wrap-comments,addruler1", "issue-389d-mod1.tex"),
        ("issue-389e.tex", "wrap-comments,addruler1", "issue-389e-mod1.tex"),
        ("issue-389e.tex", "wrap-comments2,addruler1", "issue-389e-mod2.tex"),
        ("issue-389f.tex", "wrap-comments3,addruler1", "issue-389f-mod1.tex"),
        ("issue-389g.tex", "wrap-comments2,addruler1", "issue-389g-mod2.tex"),
        ("issue-389h.tex", "wrap-comments2,addruler1", "issue-389h-mod2.tex"),
        ("issue-389i.tex", "wrap-comments2,addruler1", "issue-389i-mod2.tex"),
        ("issue-389j.tex", "wrap-comments2,addruler1,inherit-leading-space", "issue-389j-mod2.tex"),
        ("issue-389k.tex", "wrap-comments2,addruler1,inherit-leading-space", "issue-389k-mod2.tex"),
        ("issue-389l.tex", "wrap-comments4,addruler1", "issue-389l-mod2.tex"),
        ("issue-389l.tex", "wrap-comments5,addruler1", "issue-389l-mod3.tex"),
    ] {
        let arg = format!("-l={l}");
        let m = out.rsplit("-mod").next().unwrap();
        let o = format!("-o=+-mod{}", m.trim_end_matches(".tex"));
        v.push(case("e-dep", input, &["-s", "-r", "-m", &arg, &o], out));
    }

    // L245-257：issue-444 族
    v.push(case("e-dep", "issue-444a.tex",
        &["-s", "-m", "-r", "-l", "issue-444", "-o=+-mod1"], "issue-444a-mod1.tex"));
    for (l, out) in [
        ("issue-444", "issue-444b-mod1.tex"),
        ("issue-444c", "issue-444b-mod2.tex"),
        ("issue-444d", "issue-444b-mod3.tex"),
        ("issue-444e", "issue-444b-mod4.tex"),
        ("issue-444f", "issue-444b-mod5.tex"),
    ] {
        let arg = format!("-l={l}");
        let m = out.rsplit("-mod").next().unwrap();
        let o = format!("-o=+-mod{}", m.trim_end_matches(".tex"));
        v.push(case("e-dep", "issue-444b.tex", &["-s", "-m", "-r", &arg, &o], out));
    }
    v.push(case("e-dep", "issue-444c.tex",
        &["-s", "-m", "-r", "-l", "issue-444g", "-o=+-mod1"], "issue-444c-mod1.tex"));

    // L259-269：issue-450 族
    v.push(case("e-dep", "issue-450a.tex",
        &["-s", "-m", "-r", "-l", "issue-450", "-o=+-mod1"], "issue-450a-mod1.tex"));
    v.push(case("e-dep", "issue-450b.tex",
        &["-s", "-m", "-r", "-l", "issue-450", "-o=+-mod1"], "issue-450b-mod1.tex"));
    v.push(case("e-dep", "issue-450c.tex",
        &["-s", "-m", "-r", "-l", "issue-450a", "-o=+-mod1"], "issue-450c-mod1.tex"));
    let c89 = cols(89).to_string();
    v.push(case("e-dep", "issue-450d.tex",
        &["-s", "-m", "-r", "-l", "issue-450a", "-o=+-mod1", "-y", &c89], "issue-450d-mod1.tex"));
    v.push(case("e-dep", "issue-450e.tex",
        &["-s", "-m", "-r", "-l", "issue-450", "-o=+-mod1"], "issue-450e-mod1.tex"));
    v.push(case("e-dep", "issue-450f.tex",
        &["-s", "-m", "-r", "-l", "issue-450", "-o=+-mod1"], "issue-450f-mod1.tex"));

    // L277-281：issue-499 / issue-502（addruler2）
    v.push(case("e-dep", "issue-499.tex",
        &["-s", "-m", "-l", "issue-499,addruler2", "-r", "-o=+-mod1"], "issue-499-mod1.tex"));
    v.push(case("e-dep", "issue-499.tex",
        &["-s", "-m", "-l", "issue-499a,addruler2", "-r", "-o=+-mod2"], "issue-499-mod2.tex"));
    v.push(case("e-dep", "issue-502.tex",
        &["-s", "-m", "-r", "-l", "issue-502,addruler2", "-o=+-mod1"], "issue-502-mod1.tex"));
    v.push(case("e-dep", "issue-502a.tex",
        &["-s", "-m", "-r", "-l", "issue-502a,addruler2", "-o=+-mod1"], "issue-502a-mod1.tex"));

    v
}

/// T2 依赖、当前引擎不支持的用例（引用含 `/e` 的替换 YAML）。
/// 列出的产物在 `text_wrap_e_dependent_pending`（`#[ignore]`）中执行；启用 T2 后把它们
/// 清空即可让这些断言并入 `text_wrap_e_dependent`。
const E_PENDING: &[&str] = &[
    "textwrap-bfccb-mod1.tex", "textwrap-bfccb-mod2.tex",
    "textwrap-bfccb1-mod1.tex", "textwrap-bfccb1-mod2.tex", "textwrap-bfccb1-mod3.tex",
    "textwrap-bfccb2-mod1.tex",
    "issue-306-mod1.tex",
    "issue-359a-mod1.tex", "issue-359b-mod1.tex", "issue-359c-mod1.tex",
    "issue-359a-mod2.tex", "issue-359b-mod2.tex", "issue-359c-mod2.tex",
    "issue-359a-mod3.tex", "issue-359b-mod3.tex", "issue-359c-mod3.tex",
    "issue-359a-mod4.tex", "issue-359a-mod5.tex", "issue-359a-mod6.tex",
    "issue-359d-mod1.tex", "issue-359d-mod2.tex",
    "issue-359e-mod1.tex", "issue-359e-mod2.tex",
    "issue-356-mod1.tex", "issue-356-mod2.tex",
    "issue-389a-mod1.tex", "issue-389b-mod1.tex",
    "issue-389c-mod1.tex", "issue-389c-mod2.tex",
    "issue-389d-mod1.tex", "issue-389e-mod1.tex", "issue-389e-mod2.tex",
    "issue-389f-mod1.tex", "issue-389g-mod2.tex", "issue-389h-mod2.tex",
    "issue-389i-mod2.tex", "issue-389j-mod2.tex", "issue-389k-mod2.tex",
    "issue-389l-mod2.tex", "issue-389l-mod3.tex",
    "issue-444a-mod1.tex", "issue-444b-mod1.tex", "issue-444b-mod2.tex",
    "issue-444b-mod3.tex", "issue-444b-mod4.tex", "issue-444b-mod5.tex",
    "issue-444c-mod1.tex",
    "issue-450a-mod1.tex", "issue-450b-mod1.tex", "issue-450c-mod1.tex",
    "issue-450d-mod1.tex", "issue-450e-mod1.tex", "issue-450f-mod1.tex",
    "issue-499-mod1.tex", "issue-499-mod2.tex",
    "issue-502-mod1.tex", "issue-502a-mod1.tex",
];

fn run_e_cases(pending: bool) {
    for c in build_e_cases() {
        if E_PENDING.contains(&c.expected.as_str()) != pending {
            continue;
        }
        check_owned("text-wrap", &c.input, &c.args, &c.expected);
    }
    report();
}

/// 非 `/e` 部分（tw-tc 7 条）。
#[test]
fn text_wrap_e_dependent() {
    run_e_cases(false);
}

/// T2 依赖的 57 条（`-r` + `/e`）。
#[test]
#[ignore = "T2 依赖：-r substitution 的 Perl /e（ruler 替换，见 doc/pending-work.md T2）未实现"]
fn text_wrap_e_dependent_pending() {
    run_e_cases(true);
}

/// 上游 L165 / L285：两条不落产物的调用（`doc/verify/09-test-porting-audit.md` §3.2 的
/// 「2 条上游不产生可比对产物」）。此处只断言退出码为 0，并（L165）确认警告日志生成。
#[test]
fn text_wrap_no_product_calls() {
    let case = common::run(
        "text-wrap",
        "issue-362.tex",
        &["-s", "-l", "issue-362", "-g", "issue-362-warn.log"]
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>(),
    );
    case.assert_success();
    assert!(
        case.work.join("issue-362-warn.log").is_file(),
        "L165 应生成 -g 指定的警告日志"
    );

    let case = common::run(
        "text-wrap",
        "issue-552.tex",
        &["-s", "-l", "issue-552", "-m"]
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>(),
    );
    case.assert_success();
}

#[test]
fn text_wrap_long_lines_huge() {
    run_group("long-lines");
    report();
}

#[test]
fn text_wrap_verbatim_batch() {
    run_group("verbatim");
    report();
}

#[test]
fn text_wrap_remove_plb_batch() {
    run_group("plb");
    report();
}

#[test]
fn text_wrap_para_stop() {
    run_group("para-stop");
    report();
}

#[test]
fn text_wrap_follow_ups() {
    run_group("follow-ups");
    report();
}

#[test]
fn text_wrap_zoehneto_batch() {
    run_group("zoehneto");
    report();
}

#[test]
fn text_wrap_waschk() {
    run_group("waschk");
    report();
}

#[test]
fn text_wrap_issue_183_228() {
    run_group("issue-183-228");
    report();
}

#[test]
fn text_wrap_issue_337_341() {
    run_group("issue-337-341");
    report();
}

#[test]
fn text_wrap_columns_minus_one() {
    run_group("columns-m1");
    report();
}

#[test]
fn text_wrap_issue_4xx() {
    run_group("issue-4xx");
    report();
}

#[test]
fn text_wrap_issue_5xx_6xx() {
    run_group("issue-5xx-6xx");
    report();
}

/// L162：`latexindent.pl -s issue-341-mod1 -m -l issue-341 -w`（与 L161 同产物）。
#[test]
fn text_wrap_issue_341_overwrite() {
    check_overwrite("text-wrap", "issue-341-mod1.tex", &["-s", "-m", "-l=issue-341"]);
}


