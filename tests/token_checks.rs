//! tokenChecks：正文含 token 样式文本时 token_check 重命名（-LIN 后缀）且输出不动点，
//! 覆盖 tests/cases/tokenChecks/token-checks.sh 的 4 个调用。
mod common;
use common::Session;

const DIR: &str = "tokenChecks";

#[test]
fn token_collisions_are_fixed_points() {
    let s = Session::new(DIR, &[]);
    for (n, log) in [
        (1, "one.log"),
        (2, "two.log"),
        (3, "three.log"),
        (4, "four.log"),
    ] {
        let input = format!("token-checks{n}");
        let g = format!("-g={log}");
        let out = s.run(&["-s", "-w", &input, &g]);
        assert!(out.status.success(), "token-checks{n} 退出码非 0");
        let f = format!("token-checks{n}.tex");
        s.assert_file(&f, &f);
    }
}
