//! STDIN：管道输入模式（`cat file | latexindent ...`），覆盖
//! tests/cases/STDIN/stdin-test-cases.sh 的 7 个调用。基准取自已提交的 latexindent.pl 结果。
mod common;
use common::{run, run_stdin};

const DIR: &str = "STDIN";

/// stdin 输入（无文件参数 / 显式 `-`）与文件输入混用。
#[test]
fn stdin_outputs() {
    // 无文件参数 → 自动 STDIN（-o 名不带 +）
    let c = run_stdin(
        DIR,
        "myfile.tex",
        &["-s".into(), "-o=myfile-default".into()],
    );
    c.assert_success();
    c.assert_matches("myfile-default.tex", "myfile-default.tex");

    // 显式 `-` + dontMeasure（本例与默认输出一致）
    let c = run_stdin(
        DIR,
        "myfile.tex",
        &[
            "-s".into(),
            "-o=myfile-default".into(),
            "-l=dontMeasure.yaml".into(),
            "-".into(),
        ],
    );
    c.assert_success();
    c.assert_matches("myfile-default.tex", "myfile-default.tex");
}

/// issue-493：stdin/文件两种输入 + -m + 裸 -l / lookForPreamble: STDIN: 0。
#[test]
fn issue_493_stdin_and_file() {
    let c = run_stdin(
        DIR,
        "issue-493.tex",
        &[
            "-s".into(),
            "-m".into(),
            "-l".into(),
            "latexindent.yaml".into(),
            "-o=issue-493-mod1".into(),
            "-".into(),
        ],
    );
    c.assert_success();
    c.assert_matches("issue-493-mod1.tex", "issue-493-mod1.tex");

    // 裸 -l（文件输入）
    let c = run(
        DIR,
        "issue-493.tex",
        &["-s".into(), "-l".into(), "-m".into(), "-o=+-mod2".into()],
    );
    c.assert_success();
    c.assert_matches("issue-493-mod2.tex", "issue-493-mod2.tex");

    let c = run_stdin(
        DIR,
        "issue-493.tex",
        &[
            "-s".into(),
            "-m".into(),
            "-l".into(),
            "issue-493a.yaml".into(),
            "-o=issue-493-mod3".into(),
            "-".into(),
        ],
    );
    c.assert_success();
    c.assert_matches("issue-493-mod3.tex", "issue-493-mod3.tex");
}

/// issue-528 / 713971：纯 stdin 输出。
#[test]
fn stdin_plain_outputs() {
    let c = run_stdin(
        DIR,
        "issue-528.tex",
        &["-s".into(), "-o=issue-528-mod1".into(), "-".into()],
    );
    c.assert_success();
    c.assert_matches("issue-528-mod1.tex", "issue-528-mod1.tex");

    let c = run_stdin(
        DIR,
        "713971.tex",
        &["-s".into(), "-o=713971-mod1".into(), "-".into()],
    );
    c.assert_success();
    c.assert_matches("713971-mod1.tex", "713971-mod1.tex");
}
