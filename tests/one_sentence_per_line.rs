//! 一句一行（oneSentencePerLine）测试。
//!
//! 上游脚本：`tests/cases/oneSentencePerLine/one-sentence-per-line.sh`（唯一 .sh，无子目录）。
//! 口径：`loopmax=6`、未设 `verbatimTest` ⇒ `common.sh` 的 `loopmin=-1` 分支不生效、仍为 1
//! ⇒ `pcc-program-review` 循环 `i = 1..=6`，循环体 2 次调用（脚本第 82、83 行）。
//! 上游调用总数 = 静态 97 + 循环 6×2 = **109**。
//!
//! 本文件结构：前 4 个函数为原有搬运（7 条断言）；其后为本次 T1 补齐的 102 条缺口，
//! 每条缺口都有一条断言；109 条全部转正，不再有 `#[ignore]`。
mod common;
use common::{check, check_owned, report};

const DIR: &str = "oneSentencePerLine";

fn owned(args: &[&str]) -> Vec<String> {
    args.iter().map(|s| s.to_string()).collect()
}

// ---------------------------------------------------------------------------
// 原有搬运（7 条断言）
// ---------------------------------------------------------------------------

#[test]
fn two_and_three_sentences() {
    check(
        DIR,
        "two-sentences.tex",
        &["-s", "-m", "-o=+mod0"],
        "two-sentencesmod0.tex",
    );
    check(
        DIR,
        "two-sentences.tex",
        &["-s", "-m", "-o=+mod1", "-l=manipulateSentences"],
        "two-sentencesmod1.tex",
    );
    check(
        DIR,
        "three-sentences-trailing-comments.tex",
        &["-s", "-m", "-o=+mod1", "-l=manipulateSentences"],
        "three-sentences-trailing-commentsmod1.tex",
    );
}

#[test]
fn six_sentences() {
    check(
        DIR,
        "six-sentences.tex",
        &["-s", "-m", "-o=+mod1", "-l=manipulateSentences"],
        "six-sentencesmod1.tex",
    );
}

#[test]
fn six_sentences_preserve_blank() {
    check(
        DIR,
        "six-sentences.tex",
        &[
            "-s",
            "-m",
            "-o=+mod2",
            "-l=manipulateSentences",
            "-y=modifyLineBreaks:preserveBlankLines:0",
        ],
        "six-sentencesmod2.tex",
    );
}

#[test]
fn six_sentences_mult_blank() {
    check(
        DIR,
        "six-sentences-mult-blank.tex",
        &["-s", "-m", "-o=+mod0", "-l=manipulateSentences"],
        "six-sentences-mult-blankmod0.tex",
    );
    check(
        DIR,
        "other-punctuation.tex",
        &["-s", "-m", "-o=+mod0", "-l=manipulateSentences"],
        "other-punctuationmod0.tex",
    );
}

// ---------------------------------------------------------------------------
// T1 补齐（102 条缺口）
// ---------------------------------------------------------------------------

/// 循环前静态缺口 35 条：脚本第 13、18–20、24–29、33–35、38–39、42、45–47、50、
/// 53–55、58–60、63–64、67–68、71–75 行。
fn gaps_before_loop() {
    let cases: &[(&str, &[&str], &str)] = &[
        // L13
        (
            "three-sentences-trailing-comments.tex",
            &[
                "-s",
                "-m",
                "-o=+mod2",
                "-l=manipulateSentences",
                "-y=modifyLineBreaks:preserveBlankLines:0",
            ],
            "three-sentences-trailing-commentsmod2.tex",
        ),
        // L18–L20
        (
            "six-sentences.tex",
            &[
                "-s",
                "-m",
                "-o=+mod3",
                "-l=manipulateSentences",
                "-y=modifyLineBreaks:oneSentencePerLine:sentencesEndWith:exclamationMark:0",
            ],
            "six-sentencesmod3.tex",
        ),
        (
            "six-sentences.tex",
            &[
                "-s",
                "-m",
                "-o=+mod4",
                "-l=manipulateSentences",
                "-y=modifyLineBreaks:oneSentencePerLine:sentencesEndWith:betterFullStop:0",
            ],
            "six-sentencesmod4.tex",
        ),
        (
            "six-sentences.tex",
            &[
                "-s",
                "-m",
                "-o=+mod5",
                "-l=manipulateSentences",
                "-y=modifyLineBreaks:oneSentencePerLine:sentencesEndWith:questionMark:0",
            ],
            "six-sentencesmod5.tex",
        ),
        // L24–L29
        (
            "six-sentences-mult-blank.tex",
            &[
                "-s",
                "-m",
                "-o=+mod1",
                "-l=manipulateSentences",
                "-y=modifyLineBreaks:preserveBlankLines:0;condenseMultipleBlankLinesInto:0",
            ],
            "six-sentences-mult-blankmod1.tex",
        ),
        (
            "six-sentences-mult-blank.tex",
            &[
                "-s",
                "-m",
                "-o=+mod2",
                "-l=manipulateSentences",
                "-y=modifyLineBreaks:oneSentencePerLine:sentencesFollow:blankLine:0",
            ],
            "six-sentences-mult-blankmod2.tex",
        ),
        (
            "six-sentences-mult-blank.tex",
            &[
                "-s",
                "-m",
                "-o=+mod3",
                "-l=manipulateSentences",
                "-y=modifyLineBreaks:oneSentencePerLine:sentencesFollow:blankLine: 0;fullStop:0",
            ],
            "six-sentences-mult-blankmod3.tex",
        ),
        (
            "six-sentences-mult-blank.tex",
            &[
                "-s",
                "-m",
                "-o=+mod4",
                "-l=manipulateSentences",
                "-y=modifyLineBreaks:oneSentencePerLine:sentencesFollow:blankLine: 0;exclamationMark:0",
            ],
            "six-sentences-mult-blankmod4.tex",
        ),
        (
            "six-sentences-mult-blank.tex",
            &[
                "-s",
                "-m",
                "-o=+mod5",
                "-l=manipulateSentences",
                "-y=modifyLineBreaks:oneSentencePerLine:sentencesFollow:blankLine: 0;questionMark:0",
            ],
            "six-sentences-mult-blankmod5.tex",
        ),
        (
            "six-sentences-mult-blank.tex",
            &[
                "-s",
                "-m",
                "-o=+mod6",
                "-l=manipulateSentences",
                "-y=modifyLineBreaks:oneSentencePerLine:removeSentenceLineBreaks:0",
            ],
            "six-sentences-mult-blankmod6.tex",
        ),
        // L33–L35（`-y` 值里的反斜杠为 shell 字面量，照抄）
        (
            "other-punctuation.tex",
            &[
                "-s",
                "-m",
                "-o=+mod1",
                "-l=manipulateSentences,sentences-start-with-lower-case",
                r#"-y=modifyLineBreaks:oneSentencePerLine:sentencesEndWith:other:\;"#,
            ],
            "other-punctuationmod1.tex",
        ),
        (
            "other-punctuation.tex",
            &[
                "-s",
                "-m",
                "-o=+mod2",
                "-l=manipulateSentences,sentences-start-with-lower-case",
                r#"-y=modifyLineBreaks:oneSentencePerLine:sentencesEndWith:other:\;|\:"#,
            ],
            "other-punctuationmod2.tex",
        ),
        (
            "other-punctuation.tex",
            &[
                "-s",
                "-m",
                "-o=+mod3",
                "-l=manipulateSentences,sentences-start-with-lower-case",
                r#"-y=modifyLineBreaks:oneSentencePerLine:sentencesEndWith:other:\;|\:|\""#,
            ],
            "other-punctuationmod3.tex",
        ),
        // L38–L39
        (
            "sentences-across-blank-lines.tex",
            &[
                "-s",
                "-m",
                "-o=+mod0",
                "-l=manipulateSentences,issue-419b",
            ],
            "sentences-across-blank-linesmod0.tex",
        ),
        (
            "sentences-across-blank-lines.tex",
            &[
                "-s",
                "-m",
                "-o=+mod1",
                "-l=manipulateSentences",
                "-y=modifyLineBreaks:preserveBlankLines:0",
            ],
            "sentences-across-blank-linesmod1.tex",
        ),
        // L42
        (
            "sentences-across-blocks.tex",
            &[
                "-s",
                "-m",
                "-o=+mod0",
                "-l=manipulateSentences,issue-419b",
            ],
            "sentences-across-blocksmod0.tex",
        ),
        // L45–L47
        (
            "texbook-snippet.tex",
            &["-s", "-m", "-o=+mod1", "-l=manipulateSentences"],
            "texbook-snippetmod1.tex",
        ),
        (
            "texbook-snippet.tex",
            &[
                "-s",
                "-m",
                "-o=+mod2",
                "-l=manipulateSentences,sentences-start-with-lower-case",
                r#"-y=modifyLineBreaks:oneSentencePerLine:sentencesEndWith:other:\;|\,|\:"#,
            ],
            "texbook-snippetmod2.tex",
        ),
        (
            "texbook-snippet.tex",
            &[
                "-s",
                "-m",
                "-o=+mod3",
                "-l=manipulateSentences",
                "-y=modifyLineBreaks:oneSentencePerLine:sentencesFollow:commentOnPreviousLine:0",
            ],
            "texbook-snippetmod3.tex",
        ),
        // L50
        (
            "verbatim-test.tex",
            &["-s", "-m", "-o=+mod0", "-l=manipulateSentences"],
            "verbatim-testmod0.tex",
        ),
        // L53–L55
        (
            "more-code-blocks.tex",
            &[
                "-s",
                "-m",
                "-o=+mod0",
                "-l=manipulateSentences,itemize,issue-419b",
            ],
            "more-code-blocksmod0.tex",
        ),
        (
            "more-code-blocks.tex",
            &[
                "-s",
                "-m",
                "-o=+mod1",
                "-l=manipulateSentences",
                r#"-y=modifyLineBreaks:oneSentencePerLine:sentencesEndWith:other:\:"#,
            ],
            "more-code-blocksmod1.tex",
        ),
        (
            "more-code-blocks.tex",
            &[
                "-s",
                "-m",
                "-o=+mod2",
                "-l=manipulateSentences,sentences-start-with-lower-case,item",
                r#"-y=modifyLineBreaks:oneSentencePerLine:sentencesEndWith:other:\:"#,
            ],
            "more-code-blocksmod2.tex",
        ),
        // L58–L60
        (
            "webwork-guide.tex",
            &[
                "-s",
                "-m",
                "-o=+mod0",
                "-l=manipulateSentences,basic-full-stop,keepMultipleSpaces,issue-419b",
            ],
            "webwork-guidemod0.tex",
        ),
        (
            "webwork-guide.tex",
            &[
                "-s",
                "-m",
                "-o=+mod1",
                "-l=manipulateSentences,alt-full-stop,keepMultipleSpaces,issue-419b",
            ],
            "webwork-guidemod1.tex",
        ),
        (
            "webwork-guide.tex",
            &[
                "-s",
                "-m",
                "-o=+mod2",
                "-l=manipulateSentences,alt-full-stop,itemize,keepMultipleSpaces,issue-419b",
            ],
            "webwork-guidemod2.tex",
        ),
        // L63–L64
        (
            "trailing-comments.tex",
            &["-s", "-m", "-o=+mod0", "-l=manipulateSentences"],
            "trailing-commentsmod0.tex",
        ),
        (
            "trailing-comments.tex",
            &[
                "-s",
                "-m",
                "-o=+mod1",
                "-l=manipulateSentences",
                "-y=modifyLineBreaks:oneSentencePerLine:removeSentenceLineBreaks:0",
            ],
            "trailing-commentsmod1.tex",
        ),
        // L67–L68
        (
            "other-begins.tex",
            &["-s", "-m", "-o=+mod0", "-l=manipulateSentences"],
            "other-beginsmod0.tex",
        ),
        (
            "other-begins.tex",
            &["-s", "-m", "-o=+mod1", "-l=manipulateSentences,other-begins"],
            "other-beginsmod1.tex",
        ),
        // L71–L75
        (
            "mlep.tex",
            &["-s", "-m", "-o=+-mod0", "-l=manipulateSentences"],
            "mlep-mod0.tex",
        ),
        (
            "mlep.tex",
            &[
                "-s",
                "-m",
                "-o=+-mod1",
                "-l=manipulateSentences,basic-full-stop,inlineMath",
            ],
            "mlep-mod1.tex",
        ),
        (
            "mlep2.tex",
            &[
                "-s",
                "-m",
                "-o=+-mod0",
                "-l=manipulateSentences,itemize,issue-419b",
            ],
            "mlep2-mod0.tex",
        ),
        (
            "mlep2.tex",
            &[
                "-s",
                "-m",
                "-o=+-mod1",
                "-l=manipulateSentences,itemize,mlep2",
            ],
            "mlep2-mod1.tex",
        ),
        (
            "mlep2.tex",
            &[
                "-s",
                "-m",
                "-o=+-mod2",
                "-l=manipulateSentences,itemize",
                "-y=modifyLineBreaks:oneSentencePerLine:removeSentenceLineBreaks:0",
            ],
            "mlep2-mod2.tex",
        ),
    ];
    for (input, args, expected) in cases {
        check_owned(DIR, input, &owned(args), expected);
    }
}

#[test]
fn opl_before_loop() {
    gaps_before_loop();
    report();
}

/// L82/L83：`pcc-program-review$i`，i = 1..=6，每轮 2 次调用（12 条）。
fn gaps_pcc_program_review() {
    for i in 1..=6 {
        let input = format!("pcc-program-review{i}.tex");
        for (suffix, yaml, expected) in [
            (
                "mod1",
                "manipulateSentences,basic-full-stop,keepMultipleSpaces,issue-419b,pcc-review",
                format!("pcc-program-review{i}-mod1.tex"),
            ),
            (
                "mod2",
                "manipulateSentences,item,pcc-program-review,keepMultipleSpaces,issue-419b,pcc-review",
                format!("pcc-program-review{i}-mod2.tex"),
            ),
        ] {
            check_owned(
                DIR,
                &input,
                &owned(&["-s", "-m", &format!("-o=+-{suffix}"), &format!("-l={yaml}")]),
                &expected,
            );
        }
    }
}

#[test]
fn opl_pcc_program_review() {
    gaps_pcc_program_review();
    report();
}

/// 循环后静态缺口 55 条：脚本第 89–95、98–99、102、105–107、110、113–114、117、120、
/// 123–124、127–130、133、136、138–139、142–143、146、149–154、156–157、159–160、
/// 162–163、165、167、169、171、173–174、176–178、180–182 行。
fn gaps_after_loop() {
    let cases: &[(&str, &[&str], &str)] = &[
        // L89–L95
        (
            "dbmrq.tex",
            &["-s", "-m", "-l=dbmrq1", "-o=+-mod1"],
            "dbmrq-mod1.tex",
        ),
        (
            "dbmrq.tex",
            &["-s", "-m", "-l=dbmrq2", "-o=+-mod2"],
            "dbmrq-mod2.tex",
        ),
        (
            "dbmrq.tex",
            &["-s", "-m", "-l=dbmrq3", "-o=+-mod3"],
            "dbmrq-mod3.tex",
        ),
        (
            "dbmrq.tex",
            &["-s", "-m", "-l=dbmrq1,dbmrq4", "-o=+-mod4"],
            "dbmrq-mod4.tex",
        ),
        (
            "dbmrq2.tex",
            &["-s", "-m", "-l=dbmrq1", "-o=+-mod1"],
            "dbmrq2-mod1.tex",
        ),
        (
            "dbmrq3.tex",
            &["-s", "-m", "-l=dbmrq1", "-o=+-mod1"],
            "dbmrq3-mod1.tex",
        ),
        (
            "dbmrq3.tex",
            &["-s", "-m", "-l=dbmrq5", "-o=+-mod5"],
            "dbmrq3-mod5.tex",
        ),
        // L98–L99
        (
            "kiryph.tex",
            &["-s", "-m", "-l=kiryph1,issue-419b", "-o=+-mod1"],
            "kiryph-mod1.tex",
        ),
        (
            "kiryph1.tex",
            &["-s", "-m", "-l=kiryph2", "-o=+-mod2"],
            "kiryph1-mod2.tex",
        ),
        // L102
        (
            "dot-followed-by-tilde.tex",
            &[
                "-s",
                "-m",
                "-o=+-mod1",
                "-y=modifyLineBreaks:oneSentencePerLine:manipulateSentences:1",
            ],
            "dot-followed-by-tilde-mod1.tex",
        ),
        // L105–L107
        (
            "konfekt.tex",
            &["-s", "-m", "-o=+-mod0", "-l=konfekt0"],
            "konfekt-mod0.tex",
        ),
        (
            "konfekt.tex",
            &["-s", "-m", "-o=+-mod1", "-l=konfekt1"],
            "konfekt-mod1.tex",
        ),
        (
            "konfekt.tex",
            &[
                "-s",
                "-m",
                "-o=+-mod2",
                r#"-y=modifyLineBreaks:oneSentencePerLine:manipulateSentences:1;sentencesEndWith:other:\;;sentencesBeginWith:other:[[:lower:]]"#,
            ],
            "konfekt-mod2.tex",
        ),
        // L110
        (
            "sentence-wrap-no-columns-specified.tex",
            &[
                "-s",
                "-m",
                "-l=sentence-wrap-no-columns-specified",
                "-o=+-mod1",
            ],
            "sentence-wrap-no-columns-specified-mod1.tex",
        ),
        // L113–L114（`-l psttfN` 空格形式）
        (
            "psttf.tex",
            &["-s", "-m", "-l", "psttf1", "-o=+-mod1"],
            "psttf-mod1.tex",
        ),
        (
            "psttf.tex",
            &["-s", "-m", "-l", "psttf2", "-o=+-mod2"],
            "psttf-mod2.tex",
        ),
        // L117：`-y` 内联 betterFullStop 正则（原样照抄 shell 单引号内字面量）
        (
            "issue-243.tex",
            &[
                "-s",
                "-m",
                r#"-y=modifyLineBreaks:oneSentencePerLine:manipulateSentences: 1,modifyLineBreaks:oneSentencePerLine:sentencesBeginWith:a-z: 1,fineTuning:modifyLineBreaks:betterFullStop: "(?:\.|;|:(?![a-z]))|(?:(?<!(?:(?:e\.g)|(?:i\.e)|(?:etc))))\.(?!(?:[a-z]|[A-Z]|\-|~|\,|[0-9]))""#,
                "-o=+-mod1",
            ],
            "issue-243-mod1.tex",
        ),
        // L120
        (
            "issue-321.tex",
            &["-s", "-m", "-o=+mod0", "-l=manipulateSentences"],
            "issue-321mod0.tex",
        ),
        // L123–L124
        (
            "issue-355.tex",
            &["-s", "-m", "-l", "issue-355", "-o=+-mod1"],
            "issue-355-mod1.tex",
        ),
        (
            "issue-355.tex",
            &["-s", "-m", "-l", "issue-355a", "-o=+-mod2"],
            "issue-355-mod2.tex",
        ),
        // L127–L130
        (
            "issue-376.tex",
            &["-s", "-m", "-l=issue-376-orig,issue-419b", "-o=+-mod0", "-r"],
            "issue-376-mod0.tex",
        ),
        (
            "issue-376.tex",
            &["-s", "-m", "-l=issue-376,issue-419b", "-o=+-mod1"],
            "issue-376-mod1.tex",
        ),
        (
            "issue-376.tex",
            &["-s", "-m", "-l=issue-376a", "-o=+-mod2"],
            "issue-376-mod2.tex",
        ),
        (
            "issue-376.tex",
            &["-s", "-m", "-l=issue-376b", "-o=+-mod3"],
            "issue-376-mod3.tex",
        ),
        // L133
        (
            "issue-377.tex",
            &["-s", "-m", "-l=issue-377a", "-o=+-mod1"],
            "issue-377-mod1.tex",
        ),
        // L136
        (
            "issue-392.tex",
            &["-s", "-m", "-l=issue-392", "-o=+-mod1"],
            "issue-392-mod1.tex",
        ),
        // L138–L139（脚本里 `-o=+-modN` 写了两遍，语义等价，此处只传一次）
        (
            "multiple-sentences9.tex",
            &["-s", "-m", "-l=sentence-wrap4", "-o=+-mod1"],
            "multiple-sentences9-mod1.tex",
        ),
        (
            "multiple-sentences9.tex",
            &["-s", "-m", "-l=sentence-wrap5", "-o=+-mod2"],
            "multiple-sentences9-mod2.tex",
        ),
        // L142–L143
        (
            "issue-417.tex",
            &["-s", "-m", "-l=issue-417", "-o=+-mod1"],
            "issue-417-mod1.tex",
        ),
        (
            "issue-417.tex",
            &["-s", "-m", "-l=issue-417a", "-o=+-mod2"],
            "issue-417-mod2.tex",
        ),
        // L146
        (
            "issue-447.tex",
            &["-s", "-m", "-o=+-mod1", "-l=manipulateSentences"],
            "issue-447-mod1.tex",
        ),
        // L149–L154
        (
            "issue-419a.tex",
            &["-s", "-m", "-o=+-mod1", "-l=issue-419a"],
            "issue-419a-mod1.tex",
        ),
        (
            "issue-419a.tex",
            &["-s", "-m", "-o=+-mod2", "-l=manipulateSentences"],
            "issue-419a-mod2.tex",
        ),
        (
            "issue-419a.tex",
            &["-s", "-m", "-o=+-mod3", "-l=issue-419b"],
            "issue-419a-mod3.tex",
        ),
        (
            "issue-419a.tex",
            &["-s", "-m", "-o=+-mod4", "-l=issue-419c"],
            "issue-419a-mod4.tex",
        ),
        (
            "issue-419a.tex",
            &["-s", "-m", "-o=+-mod5", "-l=issue-419d"],
            "issue-419a-mod5.tex",
        ),
        (
            "issue-419a.tex",
            &["-s", "-m", "-o=+-mod6", "-l=issue-419e"],
            "issue-419a-mod6.tex",
        ),
        // L156–L157
        (
            "issue-419b.tex",
            &["-s", "-m", "-o=+-mod1", "-l=manipulateSentences"],
            "issue-419b-mod1.tex",
        ),
        (
            "issue-419b.tex",
            &["-s", "-m", "-o=+-mod2", "-l=manipulateSentences,issue-419c"],
            "issue-419b-mod2.tex",
        ),
        // L159–L160
        (
            "issue-419c.tex",
            &["-s", "-m", "-o=+-mod6", "-l=issue-419e"],
            "issue-419c-mod6.tex",
        ),
        (
            "issue-419c.tex",
            &["-s", "-m", "-o=+-mod7", "-l=issue-419f"],
            "issue-419c-mod7.tex",
        ),
        // L162–L163
        (
            "issue-463.tex",
            &["-s", "-m", "-o=+-mod1", "-l=manipulateSentences"],
            "issue-463-mod1.tex",
        ),
        (
            "issue-463.tex",
            &["-s", "-m", "-o=+-mod2", "-l=issue-419b"],
            "issue-463-mod2.tex",
        ),
        // L165
        (
            "issue-514.tex",
            &["-s", "-m", "-o=+-mod1", "-l=issue-514"],
            "issue-514-mod1.tex",
        ),
        // L167
        (
            "issue-527.tex",
            &["-s", "-m", "-o=+-mod1", "-l=issue-527"],
            "issue-527-mod1.tex",
        ),
        // L169
        (
            "issue-557.tex",
            &["-s", "-l", "issue-557", "-m", "-o=+-mod1"],
            "issue-557-mod1.tex",
        ),
        // L171
        (
            "issue-575.tex",
            &["-s", "-l", "issue-575", "-m", "-o=+-mod1"],
            "issue-575-mod1.tex",
        ),
        // L173–L174
        (
            "issue-567.tex",
            &["-s", "-m", "-l=issue-567", "-o=+-mod1"],
            "issue-567-mod1.tex",
        ),
        (
            "issue-567a.tex",
            &["-s", "-r", "-m", "-l=issue-567a", "-o=+-mod1"],
            "issue-567a-mod1.tex",
        ),
        // L176–L178（L177/L178 以上一轮产物为输入，`-o` 为字面名）
        (
            "issue-622.tex",
            &["-s", "-l=issue-622.yaml", "-m", "-o=+-mod1"],
            "issue-622-mod1.tex",
        ),
        (
            "issue-622-mod1.tex",
            &["-s", "-l=issue-622.yaml", "-m", "-o=issue-622-mod2"],
            "issue-622-mod2.tex",
        ),
        (
            "issue-622-mod1.tex",
            &["-s", "-l=issue-622.yaml", "-m", "-r", "-o=issue-622-mod3"],
            "issue-622-mod3.tex",
        ),
        // L180–L182
        (
            "issue-645.tex",
            &["-s", "-l=issue-645", "-o=+-mod1"],
            "issue-645-mod1.tex",
        ),
        (
            "issue-645.tex",
            &["-s", "-l=issue-645a", "-o=+-mod2"],
            "issue-645-mod2.tex",
        ),
        (
            "issue-645.tex",
            &["-s", "-l=issue-645b", "-o=+-mod3"],
            "issue-645-mod3.tex",
        ),
    ];
    for (input, args, expected) in cases {
        check_owned(DIR, input, &owned(args), expected);
    }
}

#[test]
fn opl_after_loop() {
    gaps_after_loop();
    report();
}

