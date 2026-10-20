//! 表格对齐（lookForAlignDelims）测试，基准取自 tests/cases/alignment。
mod common;
use common::{check, check_log_lines, check_owned, report};

fn a(args: &[&str]) -> Vec<String> {
    args.iter().map(|s| s.to_string()).collect()
}

#[test]
fn simple_align() {
    check(
        "alignment",
        "simple.tex",
        &["-s", "-o=+-default"],
        "simple-default.tex",
    );
}

#[test]
fn table1() {
    check(
        "alignment",
        "table1.tex",
        &["-s", "-o=table1-default"],
        "table1-default.tex",
    );
}

#[test]
fn matrix1() {
    check(
        "alignment",
        "matrix1.tex",
        &["-s", "-o=+-default"],
        "matrix1-default.tex",
    );
}

#[test]
fn multi_column_grouping() {
    check(
        "alignment",
        "unicode1.tex",
        &["-s", "-o=unicode1-default"],
        "unicode1-default.tex",
    );
    check(
        "alignment",
        "table1.tex",
        &["-s", "-o=+-mod3", "-l=tabular3"],
        "table1-mod3.tex",
    );
}

#[test]
fn align_mark_down() {
    check(
        "alignment",
        "align-mark-down.tex",
        &["-s", "-o=+-mod1"],
        "align-mark-down-mod1.tex",
    );
    check(
        "alignment",
        "align-mark-down.tex",
        &["-s", "-o=+-mod2", "-y=lookForAlignDelims:tabular:0"],
        "align-mark-down-mod2.tex",
    );
}

#[test]
fn multicolumn_grouping_unimplemented() {
    check(
        "alignment",
        "simple-multi-column.tex",
        &[
            "-s",
            "-o=+-mod1",
            "-y=lookForAlignDelims:tabular:multiColumnGrouping:1",
        ],
        "simple-multi-column-mod1.tex",
    );
    check("alignment", "simple-multi-column.tex", &["-s", "-o=+-mod2", "-y=lookForAlignDelims:tabular:multiColumnGrouping:1;justification:right"], "simple-multi-column-mod2.tex");
    check("alignment", "simple-multi-column.tex", &["-s", "-o=+-mod3", "-y=lookForAlignDelims:tabular:multiColumnGrouping:1;spacesBeforeAmpersand:3;justification:right"], "simple-multi-column-mod3.tex");
    check("alignment", "simple-multi-column.tex", &["-s", "-o=+-mod4", "-y=lookForAlignDelims:tabular:multiColumnGrouping:1;spacesAfterAmpersand:3;justification:right"], "simple-multi-column-mod4.tex");
    check(
        "alignment",
        "tabular-karl.tex",
        &["-s", "-o=+-out", "-l=multiColumnGrouping"],
        "tabular-karl-out.tex",
    );
    check(
        "alignment",
        "multicol.tex",
        &["-s", "-o=+-out", "-l=multiColumnGrouping"],
        "multicol-out.tex",
    );
}

#[test]
fn matrix_env_legacy() {
    check(
        "alignment",
        "matrix.tex",
        &["-s", "-o=+-default"],
        "matrix-default.tex",
    );
    check(
        "alignment",
        "environments.tex",
        &["-s", "-o=+-default"],
        "environments-default.tex",
    );
    check(
        "alignment",
        "legacy-align.tex",
        &["-s", "-o=+-default"],
        "legacy-align-default.tex",
    );
    check(
        "alignment",
        "matrix1.tex",
        &["-s", "-o=+-no-align", "-l=noMatrixAlign"],
        "matrix1-no-align.tex",
    );
}

#[test]
fn alignment_outside() {
    check(
        "alignment",
        "alignmentoutsideEnvironmentsCommands.tex",
        &["-s", "-o=+-default"],
        "alignmentoutsideEnvironmentsCommands-default.tex",
    );
}

#[test]
fn align_double_backslash() {
    check(
        "alignment",
        "environments.tex",
        &[
            "-s",
            "-o=+-no-align-double-back-slash",
            "-l=align-no-double-back-slash",
        ],
        "environments-no-align-double-back-slash.tex",
    );
}

#[test]
fn tabular_karl() {
    check(
        "alignment",
        "tabular-karl.tex",
        &["-s", "-o=+-default"],
        "tabular-karl-default.tex",
    );
    check(
        "alignment",
        "multicol-no-ampersands.tex",
        &["-s", "-o=+-default"],
        "multicol-no-ampersands-default.tex",
    );
    check(
        "alignment",
        "longcells.tex",
        &["-s", "-o=+-default"],
        "longcells-default.tex",
    );
}

#[test]
fn end_not_on_own_line() {
    check(
        "alignment",
        "end-not-on-own-line.tex",
        &["-s", "-o=+-default"],
        "end-not-on-own-line-default.tex",
    );
    check(
        "alignment",
        "end-not-on-own-line1.tex",
        &["-s", "-o=+-default"],
        "end-not-on-own-line1-default.tex",
    );
    check(
        "alignment",
        "end-not-on-own-line1.tex",
        &["-s", "-o=+-mod1", "-m", "-l=env-mod-lines1"],
        "end-not-on-own-line1-mod1.tex",
    );
    check(
        "alignment",
        "special1.tex",
        &["-s", "-o=+-aligned", "-l=special-align"],
        "special1-aligned.tex",
    );
}

// ---- dontMeasure（上游 alignment-test-cases.sh:167-180 + issue-426） ----
#[test]
fn dont_measure_cases() {
    let d = "alignment";
    check_owned(
        d,
        "yangmw.tex",
        &a(&["-s", "-o=+-default", "-y=lookForAlignDelims:tabular:dontMeasure:0"]),
        "yangmw-default.tex",
    );
    let yamls = [
        "dontMeasure",
        "dontMeasure2",
        "dontMeasure3",
        "dontMeasure4",
        "dontMeasure5",
        "dontMeasure6",
        "dontMeasure7",
        "dontMeasure8",
        "dontMeasure9",
    ];
    for (i, l) in yamls.iter().enumerate() {
        let n = i + 1;
        check_owned(
            d,
            "yangmw.tex",
            &a(&["-s", &format!("-o=+-mod{n}"), &format!("-l={l}")]),
            &format!("yangmw-mod{n}.tex"),
        );
    }
    check_owned(
        d,
        "yangmw.tex",
        &a(&[
            "-s",
            "-o=+-just-right",
            "-l=dontMeasure",
            "-y=lookForAlignDelims:tabular:spacesBeforeAmpersand:3;justification:right",
        ]),
        "yangmw-just-right.tex",
    );
    check_owned(
        d,
        "swaroopguggilam.tex",
        &a(&["-s", "-o=+-mod1", "-l=dontMeasure1"]),
        "swaroopguggilam-mod1.tex",
    );
    // issue-426：dontMeasure × -m 交互
    check_owned(d, "issue-426.tex", &a(&["-s", "-m", "-o=+-mod0"]), "issue-426-mod0.tex");
    check_owned(
        d,
        "issue-426.tex",
        &a(&["-s", "-m", "-o=+-mod1", "-l=issue-426"]),
        "issue-426-mod1.tex",
    );
    check_owned(d, "issue-426a.tex", &a(&["-s", "-m", "-o=+-mod0"]), "issue-426a-mod0.tex");
    check_owned(
        d,
        "issue-426a.tex",
        &a(&["-s", "-m", "-o=+-mod1", "-l=issue-426"]),
        "issue-426a-mod1.tex",
    );
    report();
}

// ---- alignContentAfterDoubleBackSlash（上游 sh:330-353） ----
#[test]
fn align_content_after_dbs_cases() {
    let d = "alignment";
    check_owned(
        d,
        "issue-393.tex",
        &a(&[
            "-s",
            "-m",
            "-l=double-back-slash1,double-back-slash-finish-1",
            "-o=+-mod1",
        ]),
        "issue-393-mod1.tex",
    );
    check_owned(
        d,
        "issue-393.tex",
        &a(&[
            "-s",
            "-m",
            "-l=double-back-slash1,double-back-slash-finish-1,alignContentAfterDBS",
            "-o=+-mod2",
        ]),
        "issue-393-mod2.tex",
    );
    check_owned(
        d,
        "issue-393.tex",
        &a(&[
            "-s",
            "-m",
            "-l=double-back-slash1,double-back-slash-finish-1,alignContentAfterDBS,cases3",
            "-o=+-mod3",
        ]),
        "issue-393-mod3.tex",
    );
    check_owned(
        d,
        "issue-393.tex",
        &a(&[
            "-s",
            "-m",
            "-l=double-back-slash1,double-back-slash-finish-1,alignContentAfterDBS4",
            "-o=+-mod4",
        ]),
        "issue-393-mod4.tex",
    );
    check_owned(
        d,
        "issue-393a.tex",
        &a(&[
            "-s",
            "-m",
            "-l=double-back-slash1,double-back-slash-finish-1,alignContentAfterDBS,cases3",
            "-o=+-mod3",
        ]),
        "issue-393a-mod3.tex",
    );
    check_owned(d, "issue-393b.tex", &a(&["-s", "-o=+-mod1"]), "issue-393b-mod1.tex");
    check_owned(
        d,
        "issue-393b.tex",
        &a(&["-s", "-l=alignContentAfterDBS2", "-o=+-mod2"]),
        "issue-393b-mod2.tex",
    );
    check_owned(
        d,
        "issue-393b.tex",
        &a(&["-s", "-l=alignContentAfterDBS3", "-o=+-mod3"]),
        "issue-393b-mod3.tex",
    );
    check_owned(
        d,
        "issue-393b.tex",
        &a(&["-s", "-l=alignContentAfterDBS4", "-o=+-mod4"]),
        "issue-393b-mod4.tex",
    );
    check_owned(d, "issue-393c.tex", &a(&["-s", "-o=+-mod1"]), "issue-393c-mod1.tex");
    for (i, l) in [
        (3, "alignContentAfterDBS3"),
        (4, "alignContentAfterDBS4"),
        (5, "alignContentAfterDBS5"),
        (6, "alignContentAfterDBS6"),
        (7, "alignContentAfterDBS7"),
    ] {
        check_owned(
            d,
            "issue-393c.tex",
            &a(&["-s", &format!("-l={l}"), &format!("-o=+-mod{i}")]),
            &format!("issue-393c-mod{i}.tex"),
        );
    }
    check_owned(
        d,
        "issue-393d.tex",
        &a(&["-s", "-l=alignContentAfterDBS", "-o=+-mod1"]),
        "issue-393d-mod1.tex",
    );
    check_owned(
        d,
        "tabular5.tex",
        &a(&["-s", "-l=alignContentAfterDBS1", "-o=+-mod1"]),
        "tabular5-mod1.tex",
    );
    check_owned(
        d,
        "tabular5.tex",
        &a(&["-s", "-l=alignContentAfterDBS8", "-o=+-mod8"]),
        "tabular5-mod8.tex",
    );
    report();
}

// ---- delimiterRegEx 自定义（上游 sh:182-190） ----
#[test]
fn delimiter_regex_cases() {
    let d = "alignment";
    check_owned(d, "tabbing.tex", &a(&["-s", "-o=+-default"]), "tabbing-default.tex");
    check_owned(
        d,
        "tabbing.tex",
        &a(&["-s", "-o=+-mod1", "-l=delimiterRegEx1"]),
        "tabbing-mod1.tex",
    );
    check_owned(
        d,
        "tabbing.tex",
        &a(&["-s", "-o=+-mod2", "-l=delimiterRegEx2"]),
        "tabbing-mod2.tex",
    );
    check_owned(
        d,
        "tabbing.tex",
        &a(&["-s", "-o=+-mod4", "-l=delimiterRegEx4"]),
        "tabbing-mod4.tex",
    );
    check_owned(
        d,
        "tabbing1.tex",
        &a(&["-s", "-o=+-mod3", "-l=delimiterRegEx3"]),
        "tabbing1-mod3.tex",
    );
    for i in 5..=8 {
        check_owned(
            d,
            "tabbing2.tex",
            &a(&["-s", &format!("-o=+-mod{i}"), &format!("-l=delimiterRegEx{i}")]),
            &format!("tabbing2-mod{i}.tex"),
        );
    }
    report();
}

// ---- alignFinalDoubleBackSlash（上游 sh:161-166） ----
#[test]
fn align_final_dbs_cases() {
    let d = "alignment";
    check_owned(d, "hudcap.tex", &a(&["-s", "-o=+-default"]), "hudcap-default.tex");
    check_owned(
        d,
        "hudcap.tex",
        &a(&["-s", "-o=+-mod1", "-l=alignFinalDoubleBackSlash"]),
        "hudcap-mod1.tex",
    );
    check_owned(d, "hudcap1.tex", &a(&["-s", "-o=+-default"]), "hudcap1-default.tex");
    check_owned(
        d,
        "hudcap1.tex",
        &a(&["-s", "-o=+-mod1", "-l=alignFinalDoubleBackSlash"]),
        "hudcap1-mod1.tex",
    );
    check_owned(
        d,
        "hudcap1.tex",
        &a(&[
            "-s",
            "-o=+-mod2",
            "-l=alignFinalDoubleBackSlash",
            "-y=lookForAlignDelims:tabular:spacesBeforeDoubleBackSlash:5",
        ]),
        "hudcap1-mod2.tex",
    );
    check_owned(
        d,
        "hudcap1.tex",
        &a(&[
            "-s",
            "-o=+-mod3",
            "-l=alignFinalDoubleBackSlash",
            "-y=lookForAlignDelims:tabular:spacesBeforeDoubleBackSlash:5;alignDoubleBackSlash:0",
        ]),
        "hudcap1-mod3.tex",
    );
    report();
}

// ---- issue-426 的 DBS poly-switch 组（上游 sh:312-327） ----
#[test]
fn dbs_poly_switch_cases() {
    let d = "alignment";
    for f in ["issue-426", "issue-426a"] {
        check_owned(
            d,
            &format!("{f}.tex"),
            &a(&["-s", "-m", "-o=+-mod2", "-l=issue-426,double-back-slash2"]),
            &format!("{f}-mod2.tex"),
        );
        check_owned(
            d,
            &format!("{f}.tex"),
            &a(&["-s", "-m", "-o=+-mod3", "-l=issue-426,double-back-slash3"]),
            &format!("{f}-mod3.tex"),
        );
        check_owned(
            d,
            &format!("{f}.tex"),
            &a(&["-s", "-m", "-o=+-mod-1", "-l=issue-426,double-back-slash-finish-1"]),
            &format!("{f}-mod-1.tex"),
        );
        for (suf, l) in [
            ("mod-fin1", "double-back-slash-finish1"),
            ("mod-fin2", "double-back-slash-finish2"),
            ("mod-fin3", "double-back-slash-finish3"),
            ("mod-fin4", "double-back-slash-finish4"),
        ] {
            check_owned(
                d,
                &format!("{f}.tex"),
                &a(&["-s", "-m", &format!("-o=+-{suf}"), &format!("-l=issue-426,{l}")]),
                &format!("{f}-{suf}.tex"),
            );
        }
    }
    report();
}

// ---- vassar（上游 sh:215-218，issue-223 的 begin 计量/空白语义） ----
#[test]
fn vassar_cases() {
    let d = "alignment";
    check_owned(d, "vassar0.tex", &a(&["-s", "-o=+-mod1"]), "vassar0-mod1.tex");
    check_owned(d, "vassar00.tex", &a(&["-s", "-o=+-mod1"]), "vassar00-mod1.tex");
    check_owned(d, "vassar1.tex", &a(&["-s", "-o=+-mod1"]), "vassar1-mod1.tex");
    check_owned(
        d,
        "vassar.tex",
        &a(&["-s", "-l=vassar1", "-o=+-mod1"]),
        "vassar-mod1.tex",
    );
    report();
}

// ---- table/mod 大循环（上游 sh:16-45，loopmin=1..loopmax=4，共 68 调用） ----
#[test]
fn table_mod_loop_cases() {
    let d = "alignment";
    for i in 1..=4 {
        check_owned(
            d,
            "table1.tex",
            &a(&["-s", &format!("-o=+-mod{i}"), &format!("-l=tabular{i},multiColumnGrouping2")]),
            &format!("table1-mod{i}.tex"),
        );
        for t in ["table2", "table3", "table4", "table5"] {
            check_owned(
                d,
                &format!("{t}.tex"),
                &a(&["-s", &format!("-o=+-mod{i}"), &format!("-l=tabular{i}")]),
                &format!("{t}-mod{i}.tex"),
            );
        }
        let j = i + 4;
        check_owned(
            d,
            "table1.tex",
            &a(&["-s", &format!("-o=table1-mod{j}"), &format!("-l=tabular{i},tabular5,multiColumnGrouping2")]),
            &format!("table1-mod{j}.tex"),
        );
        check_owned(
            d,
            "table2.tex",
            &a(&["-s", &format!("-o=table2-mod{j}"), &format!("-l=tabular{i},tabular5")]),
            &format!("table2-mod{j}.tex"),
        );
        let k = i + 8;
        check_owned(
            d,
            "table1.tex",
            &a(&["-s", &format!("-o=table1-mod{k}"), &format!("-l=tabular{i},tabular6,multiColumnGrouping2")]),
            &format!("table1-mod{k}.tex"),
        );
        check_owned(
            d,
            "table2.tex",
            &a(&["-s", &format!("-o=table2-mod{k}"), &format!("-l=tabular{i},tabular6")]),
            &format!("table2-mod{k}.tex"),
        );
        // 注释块内对齐（±noAdd/indentRules）
        check_owned(
            d,
            "alignmentoutsideEnvironmentsCommands.tex",
            &a(&["-s", &format!("-o=+-mod{i}"), &format!("-l=tabular{i}")]),
            &format!("alignmentoutsideEnvironmentsCommands-mod{i}.tex"),
        );
        for (tag, y) in [
            ("no-Add-1", "noAddtabular1"),
            ("no-Add-2", "noAddtabular2"),
            ("no-Add-3", "noAddtabular3"),
            ("indent-rules-1", "indentRules1"),
            ("indent-rules-2", "indentRules2"),
            ("indent-rules-3", "indentRules3"),
        ] {
            check_owned(
                d,
                "alignmentoutsideEnvironmentsCommands.tex",
                &a(&[
                    "-s",
                    &format!("-o=+-{tag}-mod{i}"),
                    &format!("-l=tabular{i},{y}"),
                ]),
                &format!("alignmentoutsideEnvironmentsCommands-{tag}-mod{i}.tex"),
            );
        }
    }
    report();
}

// ---- uni-code / multiColumnGrouping / 间距 / 两端对齐 / issue-95（上游 sh:101-152） ----
#[test]
fn multicolumn_spacing_justification_cases() {
    let d = "alignment";
    for i in 1..=6 {
        check_owned(
            d,
            "uni-code1.tex",
            &a(&["-s", &format!("-o=+-mod{i}"), &format!("-l=tabular{i}")]),
            &format!("uni-code1-mod{i}.tex"),
        );
    }
    // multiColumnGrouping 组
    check_owned(d, "tabular-karl.tex", &a(&["-s", "-o=+-out", "-l=multiColumnGrouping"]), "tabular-karl-out.tex");
    check_owned(d, "tabular-karl.tex", &a(&["-s", "-o=+-out1", "-l=multiColumnGrouping1"]), "tabular-karl-out1.tex");
    check_owned(d, "tabular-karl.tex", &a(&["-s", "-o=+-out2", "-l=multiColumnGrouping2"]), "tabular-karl-out2.tex");
    check_owned(d, "tabular-karl.tex", &a(&["-s", "-o=+-out3", "-l=multiColumnGrouping3"]), "tabular-karl-out3.tex");
    check_owned(d, "tabular-karl.tex", &a(&["-s", "-o=+-out5", "-l=multiColumnGrouping,tabular5"]), "tabular-karl-out5.tex");
    check_owned(d, "tabular-karl.tex", &a(&["-s", "-o=+-out6", "-l=multiColumnGrouping,tabular6"]), "tabular-karl-out6.tex");
    check_owned(d, "multicol.tex", &a(&["-s", "-o=+-out", "-l=multiColumnGrouping"]), "multicol-out.tex");
    check_owned(d, "unicode-multicol.tex", &a(&["-s", "-o=+-out", "-l=multiColumnGrouping"]), "unicode-multicol-out.tex");
    check_owned(d, "table3.tex", &a(&["-s", "-o=+-out", "-l=multiColumnGrouping"]), "table3-out.tex");
    check_owned(d, "tabular-multiple-multicol.tex", &a(&["-s", "-o=+-out", "-l=multiColumnGrouping"]), "tabular-multiple-multicol-out.tex");
    check_owned(d, "multicol-no-ampersands.tex", &a(&["-s", "-o=+-default"]), "multicol-no-ampersands-default.tex");
    check_owned(d, "multicol-no-ampersands.tex", &a(&["-s", "-o=+-out", "-l=multiColumnGrouping"]), "multicol-no-ampersands-out.tex");
    check_owned(d, "multicol-no-ampersands.tex", &a(&["-s", "-o=+-out5", "-l=multiColumnGrouping,tabular5"]), "multicol-no-ampersands-out5.tex");
    check_owned(d, "multicol-no-ampersands.tex", &a(&["-s", "-o=+-out6", "-l=multiColumnGrouping,tabular6"]), "multicol-no-ampersands-out6.tex");
    check_owned(d, "longcells.tex", &a(&["-s", "-o=+-default"]), "longcells-default.tex");
    check_owned(d, "longcells.tex", &a(&["-s", "-o=+-multicol", "-l=multiColumnGrouping"]), "longcells-multicol.tex");
    // & 前后间距
    check_owned(d, "table5.tex", &a(&["-s", "-o=+-mod7", "-l=tabular7"]), "table5-mod7.tex");
    check_owned(d, "table5.tex", &a(&["-s", "-o=+-mod8", "-l=tabular8"]), "table5-mod8.tex");
    check_owned(d, "table5.tex", &a(&["-s", "-o=+-mod9", "-l=tabular9"]), "table5-mod9.tex");
    check_owned(d, "table5.tex", &a(&["-s", "-o=+-mod10", "-l=tabular10"]), "table5-mod10.tex");
    check_owned(d, "tabular-karl.tex", &a(&["-s", "-o=+-out9", "-l=multiColumnGrouping,tabular9"]), "tabular-karl-out9.tex");
    check_owned(d, "tabular-karl.tex", &a(&["-s", "-o=+-out10", "-l=multiColumnGrouping,tabular10"]), "tabular-karl-out10.tex");
    check_owned(d, "matrix1.tex", &a(&["-s", "-o=+-mod1", "-l=matrix1"]), "matrix1-mod1.tex");
    check_owned(d, "matrix1.tex", &a(&["-s", "-o=+-mod2", "-l=matrix2"]), "matrix1-mod2.tex");
    // justification
    check_owned(d, "table5.tex", &a(&["-s", "-o=+-mod11", "-l=tabular11"]), "table5-mod11.tex");
    check_owned(d, "table5.tex", &a(&["-s", "-o=+-mod12", "-l=tabular11,tabular7,tabular8"]), "table5-mod12.tex");
    check_owned(d, "table5.tex", &a(&["-s", "-o=+-mod13", "-l=tabular11,tabular9"]), "table5-mod13.tex");
    check_owned(d, "tabular-karl.tex", &a(&["-s", "-o=+-out11", "-l=multiColumnGrouping,tabular11"]), "tabular-karl-out11.tex");
    // issue-95
    check_owned(d, "issue-95.tex", &a(&["-s", "-o=+-mod0"]), "issue-95-mod0.tex");
    check_owned(d, "issue-95.tex", &a(&["-s", "-o=+-mod1", "-l=noMaxDelims"]), "issue-95-mod1.tex");
    report();
}

// ---- DBS poly-switch 全系（上游 sh:157-181，i=-1..3，55 调用 + pmatrix2/3） ----
#[test]
fn dbs_poly_full_cases() {
    let d = "alignment";
    for i in -1..=3 {
        check_owned(
            d,
            "pmatrix1.tex",
            &a(&["-s", "-m", &format!("-o=+-mod{i}"), &format!("-l=double-back-slash{i}")]),
            &format!("pmatrix1-mod{i}.tex"),
        );
        check_owned(
            d,
            "pmatrix1.tex",
            &a(&[
                "-s",
                "-m",
                &format!("-o=+-finish-mod{i}"),
                &format!("-l=double-back-slash-finish{i}"),
            ]),
            &format!("pmatrix1-finish-mod{i}.tex"),
        );
        check_owned(
            d,
            "pmatrix1.tex",
            &a(&[
                "-s",
                "-m",
                &format!("-o=+-mod-per-name{i}"),
                &format!("-l=double-back-slash-per-name{i}"),
            ]),
            &format!("pmatrix1-mod-per-name{i}.tex"),
        );
        check_owned(
            d,
            "command-align.tex",
            &a(&[
                "-s",
                "-m",
                &format!("-o=+-opt-mod{i}"),
                &format!("-l=command-dbs-optional{i},mycommand"),
            ]),
            &format!("command-align-opt-mod{i}.tex"),
        );
        check_owned(
            d,
            "command-align.tex",
            &a(&[
                "-s",
                "-m",
                &format!("-o=+-mand-mod{i}"),
                &format!("-l=command-dbs-mandatory{i},mycommand"),
            ]),
            &format!("command-align-mand-mod{i}.tex"),
        );
        check_owned(
            d,
            "command-align.tex",
            &a(&[
                "-s",
                "-m",
                &format!("-o=+-opt-finish-mod{i}"),
                "-l=mycommand",
                &format!("-y=modifyLineBreaks:optionalArguments:DBSFinishesWithLineBreak:{i}"),
            ]),
            &format!("command-align-opt-finish-mod{i}.tex"),
        );
        check_owned(
            d,
            "command-align.tex",
            &a(&[
                "-s",
                "-m",
                &format!("-o=+-mand-finish-mod{i}"),
                "-l=mycommand",
                &format!("-y=modifyLineBreaks:mandatoryArguments:DBSFinishesWithLineBreak:{i}"),
            ]),
            &format!("command-align-mand-finish-mod{i}.tex"),
        );
        check_owned(
            d,
            "special-align.tex",
            &a(&[
                "-s",
                "-m",
                &format!("-o=+-mod{i}"),
                "-l=special-align1",
                &format!("-y=modifyLineBreaks:specialBeginEnd:DBSStartsOnOwnLine:{i}"),
            ]),
            &format!("special-align-mod{i}.tex"),
        );
        check_owned(
            d,
            "special-align.tex",
            &a(&[
                "-s",
                "-m",
                &format!("-o=+-finish-mod{i}"),
                "-l=special-align1",
                &format!("-y=modifyLineBreaks:specialBeginEnd:DBSFinishesWithLineBreak:{i}"),
            ]),
            &format!("special-align-finish-mod{i}.tex"),
        );
        check_owned(
            d,
            "special-align1.tex",
            &a(&[
                "-s",
                "-m",
                &format!("-o=+-mod{i}"),
                "-l=special-align1",
                &format!("-y=modifyLineBreaks:specialBeginEnd:DBSStartsOnOwnLine:{i}"),
            ]),
            &format!("special-align1-mod{i}.tex"),
        );
        check_owned(
            d,
            "special-align1.tex",
            &a(&[
                "-s",
                "-m",
                &format!("-o=+-finish-mod{i}"),
                "-l=special-align1",
                &format!("-y=modifyLineBreaks:specialBeginEnd:DBSFinishesWithLineBreak:{i}"),
            ]),
            &format!("special-align1-finish-mod{i}.tex"),
        );
    }
    check_owned(
        d,
        "pmatrix2.tex",
        &a(&["-s", "-m", "-o=+-mod1", "-l=double-back-slash-finish1,pmatrix"]),
        "pmatrix2-mod1.tex",
    );
    check_owned(
        d,
        "pmatrix3.tex",
        &a(&["-s", "-m", "-o=+-mod1", "-l=double-back-slash-finish1,pmatrix"]),
        "pmatrix3-mod1.tex",
    );
    report();
}

// ---- 杂项脚本段 I（上游 sh:155-270：douglasrizzo/mixed/issue-201/207/ams 系/pgregory/hidden-child/85/162/212/251/308） ----
#[test]
fn misc_script_cases_a() {
    let d = "alignment";
    for (inp, out, l) in [
        ("douglasrizzo.tex", "douglasrizzo-mod1.tex", "multiColumnGrouping"),
        ("douglasrizzo1.tex", "douglasrizzo1-mod1.tex", "multiColumnGrouping"),
        ("douglasrizzo2.tex", "douglasrizzo2-mod1.tex", "multiColumnGrouping"),
        ("issue-301.tex", "issue-301-mod1.tex", "issue-301"),
    ] {
        check_owned(d, inp, &a(&["-s", &format!("-o=+-mod1"), &format!("-l={l}")]), out);
    }
    check_owned(d, "mixed.tex", &a(&["-s", "-o=+-out", "-l=mixed"]), "mixed-out.tex");
    for i in 1..=4 {
        check_owned(
            d,
            &format!("issue-201-mk{i}.tex"),
            &a(&["-s", "-o=+-out"]),
            &format!("issue-201-mk{i}-out.tex"),
        );
    }
    check_owned(d, "issue-207.tex", &a(&["-s", "-o=+-mod0"]), "issue-207-mod0.tex");
    check_owned(d, "issue-207.tex", &a(&["-s", "-l=alusiani", "-o=+-mod1"]), "issue-207-mod1.tex");
    for n in ["amsmath", "mathtools", "nicematrix"] {
        check_owned(
            d,
            &format!("{n}.tex"),
            &a(&["-s", "-o=+-default"]),
            &format!("{n}-default.tex"),
        );
    }
    for i in 1..=4 {
        check_owned(
            d,
            "pgregory.tex",
            &a(&["-s", "-m", &format!("-l=pgregory{i}"), &format!("-o=+-mod{i}")]),
            &format!("pgregory-mod{i}.tex"),
        );
    }
    for i in 1..=6 {
        check_owned(
            d,
            &format!("hidden-child{i}.tex"),
            &a(&["-s", "-o=+-default"]),
            &format!("hidden-child{i}-default.tex"),
        );
    }
    check_owned(d, "issue-85.tex", &a(&["-s", "-o=+-default"]), "issue-85-default.tex");
    check_owned(d, "issue-162.tex", &a(&["-s", "-o=+-default"]), "issue-162-default.tex");
    check_owned(d, "issue-162.tex", &a(&["-s", "-o=+-mod1", "-l=align1"]), "issue-162-mod1.tex");
    check_owned(d, "issue-162.tex", &a(&["-s", "-o=+-mod2", "-l=align2"]), "issue-162-mod2.tex");
    check_owned(d, "issue-212.tex", &a(&["-s", "-o=+-default"]), "issue-212-default.tex");
    check_owned(d, "issue-251.tex", &a(&["-s", "-o=+-default"]), "issue-251-default.tex");
    check_owned(
        d,
        "issue-251.tex",
        &a(&["-s", "-o=+-mod1", "-y=lookForAlignDelims:tabular:spacesBeforeAmpersand:3;justification:right"]),
        "issue-251-mod1.tex",
    );
    check_owned(
        d,
        "issue-251.tex",
        &a(&["-s", "-o=+-mod2", "-y=lookForAlignDelims:tabular:spacesAfterAmpersand:5;justification:right"]),
        "issue-251-mod2.tex",
    );
    // issue-308：lookForChildCodeBlocks 关闭
    for (inp, l, o, out) in [
        ("issue-308.tex", "issue-308", "-o=+-mod1", "issue-308-mod1.tex"),
        ("issue-308.tex", "issue-308a", "-o=+-mod1a", "issue-308-mod1a.tex"),
        ("issue-308-special.tex", "issue-308-special", "-o=+-mod1", "issue-308-special-mod1.tex"),
        ("issue-308-special.tex", "issue-308-special-a", "-o=+-mod1a", "issue-308-special-mod1a.tex"),
        ("issue-308-mand-arg.tex", "issue-308-mand-arg", "-o=+-mod1", "issue-308-mand-arg-mod1.tex"),
        ("issue-308-mand-arg.tex", "issue-308-mand-arg-a", "-o=+-mod1a", "issue-308-mand-arg-mod1a.tex"),
        ("issue-308-opt-arg.tex", "issue-308-mand-arg", "-o=+-mod1", "issue-308-opt-arg-mod1.tex"),
        ("issue-308-opt-arg.tex", "issue-308-mand-arg-a", "-o=+-mod1a", "issue-308-opt-arg-mod1a.tex"),
    ] {
        check_owned(d, inp, &a(&["-s", &format!("-l={l}"), o]), out);
    }
    report();
}

// ---- 杂项脚本段 II（上游 sh:270-410：issue-326/382/402/445/456/526+/GCString/pmatrix 等） ----
#[test]
fn misc_script_cases_b() {
    let d = "alignment";
    // issue-326
    check_owned(d, "issue-326.tex", &a(&["-s", "-r", "-l=issue-326", "-o=+-mod1"]), "issue-326-mod1.tex");
    for n in ["a", "b", "c", "d"] {
        check_owned(
            d,
            &format!("issue-326{n}.tex"),
            &a(&["-s", &format!("-l=issue-326{n}"), "-o=+-mod1"]),
            &format!("issue-326{n}-mod1.tex"),
        );
    }
    check_owned(d, "issue-326d.tex", &a(&["-s", "-l=issue-326d2", "-o=+-mod2"]), "issue-326d-mod2.tex");
    // GCString 开关
    check_owned(d, "unicode-quick-brown.tex", &a(&["-s", "-o=+-no-GCString"]), "unicode-quick-brown-no-GCString.tex");
    check_owned(d, "unicode-quick-brown.tex", &a(&["-s", "--GCString", "-o=+-with-GCString"]), "unicode-quick-brown-with-GCString.tex");
    // issue-402：无 lookForAlignDelims 的 DBS poly
    for inp in ["issue-402", "issue-402a"] {
        for (l, md) in [("issue-402", 1), ("issue-402a", 2), ("issue-402b", 3)] {
            check_owned(
                d,
                &format!("{inp}.tex"),
                &a(&["-s", "-m", &format!("-l={l}"), &format!("-o=+-mod{md}")]),
                &format!("{inp}-mod{md}.tex"),
            );
        }
    }
    check_owned(d, "issue-402c.tex", &a(&["-s", "-m", "-l=issue-402c", "-o=+-mod1"]), "issue-402c-mod1.tex");
    check_owned(d, "issue-402c.tex", &a(&["-s", "-m", "-l=issue-402cc", "-o=+-mod2"]), "issue-402c-mod2.tex");
    check_owned(d, "issue-402d.tex", &a(&["-s", "-m", "-l=issue-402d", "-o=+-mod1"]), "issue-402d-mod1.tex");
    // issue-456：EndStartsOnOwnLine poly
    for (v, md) in [(2, "2"), (3, "3"), (4, "4"), (-1, "-1")] {
        check_owned(
            d,
            "issue-456.tex",
            &a(&["-s", "-m", &format!("-y=modifyLineBreaks:environments:EndStartsOnOwnLine:{v}"), &format!("-o=+-mod{md}")]),
            &format!("issue-456-mod{md}.tex"),
        );
    }
    for n in ["issue-526", "issue-535", "issue-543"] {
        check_owned(d, &format!("{n}.tex"), &a(&["-s", "-o=+-mod1"]), &format!("{n}-mod1.tex"));
    }
    check_owned(d, "issue-541.tex", &a(&["-s", "-m", "-l=issue-541", "-o=+-mod1"]), "issue-541-mod1.tex");
    // 上游脚本 issue-551 有两次调用写同一产物名（后者覆盖前者），基准即后者的输出
    check_owned(
        d,
        "issue-551.tex",
        &a(&["-s", "-y=lookForAlignDelims:tblr:alignFinalDoubleBackSlash:1", "-o=+-mod1"]),
        "issue-551-mod1.tex",
    );
    for (l, md) in [("issue-445a", "2"), ("issue-445b", "3"), ("issue-445c", "4")] {
        check_owned(
            d,
            "issue-445a.tex",
            &a(&["-s", &format!("-l={l}"), "-m", &format!("-o=+-mod{md}")]),
            &format!("issue-445a-mod{md}.tex"),
        );
    }
    // new specials / nested / pmatrix
    check_owned(d, "nestedalignment.tex", &a(&["-s", "-o=nestedalignment-NEW", "-local=new-specials"]), "nestedalignment-NEW.tex");
    check_owned(d, "nestedalignment.tex", &a(&["-s", "-o=nestedalignment-NEW1", "-local=new-specials1"]), "nestedalignment-NEW1.tex");
    check_owned(d, "nested-align1.tex", &a(&["-l=indentPreamble", "-m", "-s", "-o=+-mod0"]), "nested-align1-mod0.tex");
    check_owned(d, "pmatrix.tex", &a(&["-s", "-o=+-default"]), "pmatrix-default.tex");
    check_owned(d, "pmatrix.tex", &a(&["-s", "-o=+-special-mod1", "-m", "-l=../specials/special-mod1"]), "pmatrix-special-mod1.tex");
    check_owned(d, "pmatrix.tex", &a(&["-s", "-o=+-pmatrix-mod1", "-m", "-l=pmatrix-mod1"]), "pmatrix-pmatrix-mod1.tex");
    check_owned(d, "pmatrix.tex", &a(&["-s", "-o=+-pmatrix-mod2", "-m", "-l=pmatrix-mod1,special-left-right"]), "pmatrix-pmatrix-mod2.tex");
    check_owned(d, "pmatrix.tex", &a(&["-s", "-o=+-special-mod0", "-m", "-l=pmatrix-mod1,../specials/special-mod1"]), "pmatrix-special-mod0.tex");
    // issue-611 / te-752552 / issue-599 / issue-583
    check_owned(d, "issue-611.tex", &a(&["-s", "-o=+-mod1"]), "issue-611-mod1.tex");
    check_owned(d, "issue-611.tex", &a(&["-s", "-l=issue-611a", "-o=+-mod2"]), "issue-611-mod2.tex");
    check_owned(d, "te-752552.tex", &a(&["-s", "-o=+-mod1"]), "te-752552-mod1.tex");
    for n in ["issue-599", "issue-599a", "issue-599b"] {
        check_owned(d, &format!("{n}.tex"), &a(&["-s", "-o=+-mod1"]), &format!("{n}-mod1.tex"));
    }
    check_owned(d, "simple.tex", &a(&["-s", "-l=issue-583", "-o=+-mod1"]), "simple-mod1.tex");
    check_owned(d, "simple.tex", &a(&["-s", "-l=issue-583a", "-o=+-mod2"]), "simple-mod2.tex");
    // issue-564：产物 + 日志 found: 轨迹
    check_owned(d, "issue-564.tex", &a(&["-m", "-s", "-l=issue-564", "-o=+-mod1", "-t"]), "issue-564-mod1.tex");
    check_log_lines(d, "issue-564.tex", &["-m", "-s", "-l=issue-564", "-o=+-mod1", "-t"], "found:", "issue-564.txt");
    // issue-602 / issue-630
    check_owned(d, "issue-602.tex", &a(&["-s", "-l=issue-602", "-o=+-mod1"]), "issue-602-mod1.tex");
    check_owned(d, "issue-602a.tex", &a(&["-s", "-o=+-mod1"]), "issue-602a-mod1.tex");
    check_owned(d, "issue-630.tex", &a(&["-s", "-o=+-mod1"]), "issue-630-mod1.tex");
    check_owned(d, "issue-630.tex", &a(&["-s", "-l=issue-630a", "-o=+-mod2"]), "issue-630-mod2.tex");
    report();
}

// ---- `-r` substitution 带 /e（Perl eval 代码块），用户明确暂缓 ----
#[test]
#[ignore = "known gap: -r substitution 带 /e（issue-382/te-752552a 的替换是 Perl eval 代码块），用户明确暂缓"]
fn misc_e_substitution_cases() {
    let d = "alignment";
    check_owned(d, "issue-382.tex", &a(&["-s", "-r", "-l=issue-382", "-o=+-mod1"]), "issue-382-mod1.tex");
    check_owned(d, "issue-382.tex", &a(&["-s", "-r", "-l=issue-382a", "-o=+-mod1a"]), "issue-382-mod1a.tex");
    check_owned(d, "te-752552.tex", &a(&["-s", "-r", "-l=te-752552a", "-o=+-mod2"]), "te-752552-mod2.tex");
    report();
}

// ---- leadingBlankColumn / spacesBeforeAmpersand 映射形（上游 sh:222-247） ----
#[test]
fn leading_blank_column_cases() {
    let d = "alignment";
    check_owned(
        d,
        "issue-275.tex",
        &a(&["-s", "-o=+-default", "-y=defaultIndent: ' '"]),
        "issue-275-default.tex",
    );
    for i in 1..=6 {
        for f in ["issue-275", "issue-275a", "issue-275b", "issue-275c"] {
            check_owned(
                d,
                &format!("{f}.tex"),
                &a(&["-s", &format!("-o=+-mod{i}"), &format!("-l=sba{i}")]),
                &format!("{f}-mod{i}.tex"),
            );
        }
    }
    check_owned(
        d,
        "hidden-child1.tex",
        &a(&["-s", "-o=+-mod1", "-l=sba-align1"]),
        "hidden-child1-mod1.tex",
    );
    check_owned(
        d,
        "hidden-child1.tex",
        &a(&["-s", "-o=+-mod2", "-l=sba-align2"]),
        "hidden-child1-mod2.tex",
    );
    report();
}
