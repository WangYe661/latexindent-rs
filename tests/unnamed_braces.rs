//! 无名分组括号（unnamed-braces）测试，覆盖 tests/cases/unnamed-braces/un-named-grouping-braces.sh。
mod common;
use common::check;

#[test]
fn unnamed_and_issues() {
    check("unnamed-braces", "unnamed.tex", &["-s", "-o=unnamed-default.tex"], "unnamed-default.tex");
    check("unnamed-braces", "issue-265.tex", &["-s", "-o=+-mod1", "-l=issue-265.yaml"], "issue-265-mod1.tex");
    check("unnamed-braces", "issue-265-second.tex", &["-s", "-o=+-mod1", "-l=issue-265.yaml"], "issue-265-second-mod1.tex");
    check("unnamed-braces", "dollars-sequential.tex", &["-s", "-o=+-mod1"], "dollars-sequential-mod1.tex");
    check("unnamed-braces", "issue-563.tex", &["-s", "-o=+-mod1"], "issue-563-mod1.tex");
    check("unnamed-braces", "issue-563.tex", &["-s", "-m", "-l=issue-563-mod2.yaml", "-o=+-mod2"], "issue-563-mod2.tex");
}

#[test]
fn issue_568() {
    check("unnamed-braces", "issue-568.tex", &["-s", "-l=issue-568.yaml", "-o=+-mod1"], "issue-568-mod1.tex");
}

#[test]
fn issue_587() {
    check("unnamed-braces", "issue-587.tex", &["-s", "-l", "issue-587", "-o", "+-mod1"], "issue-587-mod1.tex");
}
