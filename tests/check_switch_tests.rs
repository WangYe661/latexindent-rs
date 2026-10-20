//! check-switch-tests：`--check -s -w` 对不动点文件退出 0，覆盖
//! tests/cases/check-switch-tests/check-switch-test-cases.sh 的循环调用。
mod common;
use common::Session;

const DIR: &str = "check-switch-tests";

#[test]
fn check_switch_on_fixed_points() {
    let s = Session::new(DIR, &[]);
    for f in [
        "environments-simple-nested.tex",
        "unicode-multicol.tex",
        "verbatim1.tex",
    ] {
        let out = s.run(&["--check", "-s", "-w", f]);
        assert_eq!(
            out.status.code(),
            Some(0),
            "{f} 应退出 0（文件未被 latexindent 改动）"
        );
        s.assert_file(f, f);
    }
}
