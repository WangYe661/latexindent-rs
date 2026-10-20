//! horizontalWhiteSpace：removeTrailingWhitespace 开关，覆盖
//! tests/cases/horizontalWhiteSpace/whitespace-test-cases.sh 的 2 个调用。
mod common;
use common::check;

const DIR: &str = "horizontalWhiteSpace";

#[test]
fn remove_trailing_whitespace_switch() {
    check(
        DIR,
        "whitespace.tex",
        &["-s", "-l=remove-TWS.yaml", "-o", "+-rm-yes"],
        "whitespace-rm-yes.tex",
    );
    check(
        DIR,
        "whitespace.tex",
        &["-s", "-l=remove-TWS1.yaml", "-o", "+-rm-no"],
        "whitespace-rm-no.tex",
    );
}
