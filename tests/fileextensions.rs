//! 文件扩展名与 -o 命名测试，基准取自 tests/cases/fileextensions。
//! 对应上游 file-extension-test-cases.sh 的全部 26 个调用（顺序相关，共享目录）。
mod common;
use common::Session;

const DIR: &str = "fileextensions";

/// -w 用例（不动点）+ 扩展名解析 FATAL + 全局副作用用例。
#[test]
fn overwrite_and_fatal() {
    let s = Session::new(DIR, &[]);
    s.run(&["-w", "-s", "myfile.tex"]);
    s.assert_file("myfile.tex", "myfile.tex");
    s.run(&["-w", "-s", "myfile.cmh"]);
    s.assert_file("myfile.cmh", "myfile.cmh");
    s.run(&["-w", "-s", "myfile.cmh", "-l=fileExtension1.yaml"]);
    s.assert_file("myfile.cmh", "myfile.cmh");
    // 无扩展名 → 按 fileExtensionPreference 解析到 myfile.tex
    s.run(&["-w", "-s", "myfile", "-g=myfile.log"]);
    s.assert_file("myfile.tex", "myfile.tex");
    // style-file-first.yaml → 解析到 myfile.sty
    s.run(&[
        "-w",
        "-s",
        "myfile",
        "-l=style-file-first.yaml",
        "-g=myfile-sty.log",
    ]);
    s.assert_file("myfile.sty", "myfile.sty");

    // 无任何匹配 → FATAL + 退出码 3
    let out = s.run(&["-w", "-s", "another"]);
    assert_eq!(out.status.code(), Some(3), "another 应退出码 3");
    s.assert_log_section(
        "indent.log",
        "FATAL:",
        Some("INFO:  Please"),
        "fatal-info1.txt",
    );

    // 任意扩展名文件（issue-154）
    s.run(&["-w", "cmh.txt", "-s"]);
    s.assert_file("cmh.txt", "cmh.txt");
    s.run(&["-w", "cmh.jab", "-s"]);
    s.assert_file("cmh.jab", "cmh.jab");
}

/// -o 命名：无扩展/尾点/`+` 前缀（基准即输出文件本身 + ext-info 日志摘录）。
#[test]
fn output_naming() {
    let s = Session::new(DIR, &[]);
    s.run(&["-s", "cmh.tex", "-o", "one"]);
    s.assert_file("one.tex", "one.tex");
    s.run(&["-s", "cmh", "-o", "two"]);
    s.assert_file("two.tex", "two.tex");
    s.assert_log_section(
        "indent.log",
        "File extension work",
        Some("INFO:  Please"),
        "ext-info1.txt",
    );
    s.run(&["-s", "cmh", "-o", "three."]);
    s.assert_file("three.tex", "three.tex");
    s.assert_log_section(
        "indent.log",
        "File extension work",
        Some("INFO:  Please"),
        "ext-info2.txt",
    );
    s.run(&["-s", "cmh.bib", "-o", "one"]);
    s.assert_file("one.bib", "one.bib");
    s.assert_log_section(
        "indent.log",
        "INFO:  -o",
        Some("INFO:  Please"),
        "ext-info3.txt",
    );
    s.run(&["-s", "cmh.bib", "-o", "two."]);
    s.assert_file("two.bib", "two.bib");
    s.assert_log_section(
        "indent.log",
        "INFO:  -o",
        Some("INFO:  Please"),
        "ext-info4.txt",
    );
    // + 前缀 → 基名拼接
    s.run(&["-s", "cmh.tex", "-o", "+one", "-g=six.log"]);
    s.assert_file("cmhone.tex", "cmhone.tex");
    s.run(&["-s", "cmh.tex", "-o", "+two.tex", "-g=seven.log"]);
    s.assert_file("cmhtwo.tex", "cmhtwo.tex");
}

/// `-l=+file` 各种写法（yaml-info 日志摘录，顺序相关：yaml-info5 复制的是上一次的 indent.log）。
#[test]
fn yaml_plus_syntax() {
    let s = Session::new(DIR, &["../alignment/multiColumnGrouping.yaml"]);
    let yaml_section = |log: &str, expected: &str| {
        s.assert_log_section(
            log,
            "INFO:  YAML settings read: -l",
            Some("INFO:  File"),
            expected,
        )
    };
    s.run(&[
        "tabular-karl",
        "-l=+../alignment/multiColumnGrouping.yaml",
        "-s",
        "-o=tabular-karl1.tex",
    ]);
    yaml_section("indent.log", "yaml-info1.txt");
    s.run(&[
        "tabular-karl",
        "-l=   +../alignment/multiColumnGrouping.yaml",
        "-s",
        "-o=tabular-karl2.tex",
    ]);
    yaml_section("indent.log", "yaml-info2.txt");
    s.run(&[
        "tabular-karl",
        "-l=   +   ../alignment/multiColumnGrouping.yaml",
        "-s",
        "-o=tabular-karl3.tex",
    ]);
    yaml_section("indent.log", "yaml-info3.txt");
    s.run(&[
        "tabular-karl",
        "-l=../alignment/multiColumnGrouping.yaml+",
        "-s",
        "-o=tabular-karl4.tex",
    ]);
    yaml_section("indent.log", "yaml-info4.txt");
    // -g=one.log 使本条日志进 one.log；indent.log 仍是上一条的（上游 cp 即如此）
    s.run(&[
        "tabular-karl",
        "-l=../alignment/multiColumnGrouping.yaml    +",
        "-s",
        "-o=tabular-karl5.tex",
        "-g=one.log",
    ]);
    yaml_section("indent.log", "yaml-info5.txt");
    s.run(&[
        "tabular-karl",
        "-l=+../alignment/multiColumnGrouping",
        "-s",
        "-o=tabular-karl6.tex",
    ]);
    yaml_section("indent.log", "yaml-info6.txt");
}

/// 上述 6 条的输出内容基准（依赖 multiColumnGrouping）。
#[test]
fn yaml_plus_outputs_multicolumn() {
    let s = Session::new(DIR, &["../alignment/multiColumnGrouping.yaml"]);
    s.run(&[
        "tabular-karl",
        "-l=+../alignment/multiColumnGrouping.yaml",
        "-s",
        "-o=tabular-karl1.tex",
    ]);
    s.assert_file("tabular-karl1.tex", "tabular-karl1.tex");
    s.run(&[
        "tabular-karl",
        "-l=   +../alignment/multiColumnGrouping.yaml",
        "-s",
        "-o=tabular-karl2.tex",
    ]);
    s.assert_file("tabular-karl2.tex", "tabular-karl2.tex");
    s.run(&[
        "tabular-karl",
        "-l=   +   ../alignment/multiColumnGrouping.yaml",
        "-s",
        "-o=tabular-karl3.tex",
    ]);
    s.assert_file("tabular-karl3.tex", "tabular-karl3.tex");
    s.run(&[
        "tabular-karl",
        "-l=../alignment/multiColumnGrouping.yaml+",
        "-s",
        "-o=tabular-karl4.tex",
    ]);
    s.assert_file("tabular-karl4.tex", "tabular-karl4.tex");
    s.run(&[
        "tabular-karl",
        "-l=../alignment/multiColumnGrouping.yaml    +",
        "-s",
        "-o=tabular-karl5.tex",
        "-g=one.log",
    ]);
    s.assert_file("tabular-karl5.tex", "tabular-karl5.tex");
    s.run(&[
        "tabular-karl",
        "-l=+../alignment/multiColumnGrouping",
        "-s",
        "-o=tabular-karl6.tex",
    ]);
    s.assert_file("tabular-karl6.tex", "tabular-karl6.tex");
}

/// -o=++ 递增命名（顺序相关）。
#[test]
fn counter_naming() {
    let s = Session::new(DIR, &[]);
    s.run(&["cmh.tex", "-o=++", "-s"]);
    s.assert_file("cmh0.tex", "one.tex");
    s.assert_log_section(
        "indent.log",
        "output file check",
        Some("INFO:"),
        "cmh-info1.txt",
    );
    s.run(&["cmh.tex", "-o=output++", "-s"]);
    s.assert_file("output0.tex", "one.tex");
    s.assert_log_section(
        "indent.log",
        "output file check",
        Some("INFO:"),
        "cmh-info2.txt",
    );
    s.run(&["cmh.tex", "-o", "+output++", "-s"]);
    s.assert_file("cmhoutput0.tex", "one.tex");
    s.assert_log_section(
        "indent.log",
        "output file check",
        Some("INFO:"),
        "cmh-info3.txt",
    );
    s.run(&["cmh.tex", "-o", "+myfile++.tex", "-s"]);
    s.assert_file("cmhmyfile0.tex", "one.tex");
    s.assert_log_section(
        "indent.log",
        "output file check",
        Some("INFO:"),
        "cmh-info4.txt",
    );
    s.run(&["cmh.tex", "-o", "myfile++.tex", "-s"]);
    s.assert_file("myfile0.tex", "one.tex");
    s.assert_log_section(
        "indent.log",
        "output file check",
        Some("INFO:"),
        "cmh-info5.txt",
    );
}
