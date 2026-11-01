# latexindent-rs

[English](README.md) | **简体中文**

> **实验性项目声明**
>
> 本项目是一个 100% vibe coding 的实验性项目，用于测试模型的编程能力。本项目不会活跃维护。本项目速度相较于原版仅有微小提升，使用 Rust 重写并不是解决性能问题的万能药。

用 Rust 重写的 [latexindent.pl](https://github.com/cmhughes/latexindent.pl)（v4.0.2），
用于美化 `.tex`、`.sty`、`.cls`、`.bib` 文件。

这是一个以行为兼容为目标的移植：沿用上游的 `defaultSettings.yaml` 配置模型，
并直接使用上游测试用例，对产物做逐字节比对。

## 特性

- **命令行参数与上游完全兼容**：支持 `-w`、`-o`、`-l`、`-y`、`-d`、`-m`、
  `-r/-rv/-rr`、`-k/-kv`、`-n`、`-c`、`-g`、`-s`、`-t/-tt`、`-sl`、`--GCString`
  等全部开关，参数形式与语义均与上游一致。
- **完整缩进能力**：环境、命令、必选/可选参数、`specialBeginEnd` 特殊块、
  无名组、`ifelsefi`、`items`、标题（`\part`/`\chapter`/`\section`…）缩进。
- **对齐与换行**：`&` 列对齐（含 `dontMeasure`、`multiColumnGrouping`、
  `alignContentAfterDoubleBackSlash` 等），`-m` 的 poly-switch 换行增删、
  text wrap、one-sentence-per-line。
- **正则引擎**：PCRE2（与 Perl 同族），支持 Unicode 与 grapheme cluster 宽度计算。

## 安装与配置

方式二和方式三需要 stable Rust 工具链（edition 2021）和 C 编译器；
`pcre2-sys` 在系统没有 PCRE2 时会静态编译自带源码。

### 方式一：下载预编译文件

打开 [Releases](https://github.com/WangYe661/latexindent-rs/releases) 页面，
下载对应平台的压缩包，解压后得到单个可执行文件：

- Windows：`latexindent-rs.exe`
- macOS / Linux：`latexindent-rs`

将可执行文件放到任意目录并加入 `PATH` 即可，无需安装 Rust。

### 方式二：使用 cargo install 直接从 GitHub 安装

```bash
cargo install --git https://github.com/WangYe661/latexindent-rs --locked
```

安装完成后可执行文件位于 `~/.cargo/bin`（Windows 为 `%USERPROFILE%\.cargo\bin`），
请确保该目录在 `PATH` 中。需要固定到某个标签时可加 `--tag <tag>`。

### 方式三：克隆到本地后安装

```bash
git clone https://github.com/WangYe661/latexindent-rs.git
cd latexindent-rs

# 安装到 ~/.cargo/bin
cargo install --path .

# 或者只构建、不安装
cargo build --release
# 产物：target/release/latexindent-rs（Windows 为 latexindent-rs.exe）
```

安装后验证：

```bash
latexindent-rs --version
# 4.0.2, 2026-06-06
```

### VS Code 配置（LaTeX Workshop）

安装 [LaTeX Workshop](https://marketplace.visualstudio.com/items?itemName=James-Yu.latex-workshop)
扩展后，在 VS Code 的 `settings.json` 中加入：

```json
{
  "latex-workshop.formatting.latex": "latexindent",
  "latex-workshop.formatting.latexindent.path": "path to latexindent-rs"
}
```

把 `"path to latexindent-rs"` 替换为可执行文件的实际路径。如果
`latexindent-rs` 已在 `PATH` 中，也可以直接写 `latexindent-rs`
（Windows 可写 `latexindent-rs.exe`）。

### YAML 配置

默认配置内嵌在二进制中（`assets/defaultSettings.yaml`）；若可执行文件同目录或当前目录存在 `defaultSettings.yaml`，则优先使用外部文件。用户配置按以下方式叠加：

- `indentconfig.yaml` / `.indentconfig.yaml`：放在家目录、`~/.config/latexindent/`、`%LOCALAPPDATA%\latexindent\`（Windows）等位置，用于追加 YAML 搜索路径。
- `-l` 指定本地 YAML；裸 `-l` 依次尝试 `localSettings.yaml`、`latexindent.yaml`、`.localSettings.yaml`、`.latexindent.yaml`（先工作目录，再当前目录）。
- `-y` 直接传入 YAML 片段覆盖设置；`-d` 忽略所有用户 YAML，只使用默认配置。

## 用法

```bash
# 默认将结果写到 stdout
latexindent-rs myfile.tex > myfile-indented.tex

# 原地覆盖（覆盖前生成备份）
latexindent-rs -w myfile.tex

# 输出到指定文件
latexindent-rs -o indented.tex myfile.tex

# 从 STDIN 读取
cat myfile.tex | latexindent-rs -

# 使用当前目录的 localSettings.yaml，并修改换行
latexindent-rs -l -m myfile.tex

# 命令行覆盖 YAML 设置
latexindent-rs -y="defaultIndent: '  '" myfile.tex

# 只处理第 10–20 行
latexindent-rs -n 10-20 myfile.tex

# 检查模式：未变化退出码 0，有变化退出码 1
latexindent-rs -k myfile.tex

# 多文件并行处理（-j 0 = 按可用 CPU 自动，-j 1 = 顺序）
latexindent-rs -w -j 0 chapter*.tex

# 增量缓存：内容与设置都没变时直接复用上次结果
latexindent-rs -w --cache chapter*.tex
```

输入文件不带扩展名时，按 `fileExtensionPreference` 依次查找
`.tex`、`.sty`、`.cls`、`.bib`。

### 性能相关开关（本项目新增，上游 latexindent.pl 无对应项）

| 开关 | 说明 |
| --- | --- |
| `-j, --jobs=<N>` | 多文件并行处理的线程数。`0`（默认）= 按可用 CPU 自动，`1` = 顺序执行。只有一个文件时始终顺序执行。 |
| `--cache[=<dir>]` | 开启增量缓存。目录默认是 `/…/.latexindent-cache`（与备份文件同处）；缓存条目记录「设置指纹 + 输入哈希 + 输出哈希 + 输出文本」。 |

并行处理只影响 CPU 时间，不影响结果：每个文件都在自己的线程局部状态里处理，
产物与 `indent.log` 仍按命令行给定的文件顺序写出（`-j 1` 与 `-j 4` 的产物逐字节相同，
`indent.log` 除 `UNIX time` 时间戳与文件名外逐行相同）。

增量缓存命中分两种：输入与上次完全相同（结果确定可复用），
或文件内容已经是本工具上一次的输出（格式化幂等，直接跳过）。
设置（YAML、`-m/-r/-y/-l` 等）或工具版本变化会让旧条目自动失效；
`-t/-tt` 追踪模式下不使用缓存，以免日志缺内容。

### 示例

输入：

```latex
\begin{itemize}
\item first
\item second
\end{itemize}
```

输出（默认缩进为制表符）：

```latex
\begin{itemize}
	\item first
	\item second
\end{itemize}
```

完整开关列表见 `latexindent-rs --help`；YAML 配置项的完整说明见
[上游文档](https://latexindentpl.readthedocs.io/)。

## 测试

测试用例来自上游 `latexindent.pl` 的 `test-cases`，运行时复制到独立临时目录，
执行二进制后将产物与基准文件逐字节比较。

```bash
cargo test
```

当前结果：**278 个测试通过，0 失败，1 个标记为 `#[ignore]`**。
`cargo test -- --ignored` 可查看该已知缺口。

## 已知限制

- PCRE2 的 `match limit`（10M 步）比 Perl「无上限」严格：未闭合环境、极深嵌套等畸形输入
  会让精确版正则超限。此时按「env body 原子化 → 参数体所有格 → 两级叠加 → 按行分块」
  逐级回退（见 `src/blocks.rs` 的 `scan_with_fallback`），产物仍正确、进程不会退出，
  但极端输入下块边界可能与 Perl 略有出入（`-t` 日志里会留下 `code block matching` 记录）。
- `-r` / `-rv` / `-rr` 替换模式中，Perl 的 `/e` 由原生代码模拟而非 Perl 解释器：
  `src/replacement_e.rs` 识别上游测试集里用到的 eval 代码块（ruler 生成、
  `\draw`/prooftree 对齐、`cmhlistings` 展平、数学公式规范化）并用 Rust 逐字节复刻。
  未识别的 `/e` 替换会输出警告并保持原文不变，因此仍不支持任意用户 Perl。
- 上游 Perl 正则的 `PRUNE` 泄漏属于引擎自身行为，PCRE2 无法复现；仅 trace
  日志中 `found:` 的记录不同，格式化产物完全一致（1 个测试 `#[ignore]`）。

除此之外，上游测试集覆盖的功能均按字节级一致实现。

## 项目结构

| 文件 | 对应上游模块 / 职责 |
| --- | --- |
| `src/document.rs` | `Document.pm`、`FileExtension.pm`、`BackUpFileProcedure.pm`、`Check.pm`、`LogFile.pm`：主流程 |
| `src/settings.rs` | `GetYamlSettings.pm`：YAML 加载、合并与设置查询 |
| `src/blocks.rs` | `Blocks.pm`：代码块识别与缩进 |
| `src/patterns.rs` | 代码块正则拼装（Perl 递归正则 → PCRE2 `(?(DEFINE)…)`） |
| `src/alignment.rs` | `AlignmentAtAmpersand.pm`：`&` 对齐 |
| `src/mlb.rs` | `ModifyLineBreaks.pm`：`-m` 的 poly-switch |
| `src/verbatim.rs` | `Verbatim.pm`：verbatim / noindentblock 保护 |
| `src/file_contents.rs` | `FileContents.pm`：filecontents 与 preamble |
| `src/replacement.rs` | `Replacement.pm`：`-r` 系列替换 |
| `src/replacement_e.rs` | Perl `/e` 替换片段的原生 handler |
| `src/heading.rs` | `Heading.pm`：标题后的缩进 |
| `src/sentence.rs` | `Sentence.pm`：one-sentence-per-line |
| `src/wrap.rs` | `Wrap.pm`：text wrap |
| `src/whitespace.rs` | `HorizontalWhiteSpace.pm` |
| `src/trailing_comments.rs` | `TrailingComments.pm` |
| `src/blank_lines.rs` | `BlankLines.pm` |
| `src/double_backslash.rs` | `DoubleBackSlash.pm` |
| `src/tokens.rs` | `Tokens.pm`：占位 token 表 |
| `src/switches.rs` | 命令行解析与开关状态 |
| `src/logger.rs` | `Logger.pm` / `LogFile.pm`：日志与 `indent.log` |
| `src/state.rs` | Perl 全局变量的对应物（thread-local 状态） |
| `src/cache.rs` | 增量格式化缓存（`--cache`，本项目新增） |

`tests/cases/` 是上游测试用例，`tests/common/mod.rs` 是临时目录测试夹具，
`tests/*.rs` 按功能域组织断言。

## 许可证

本项目自身代码采用 **MIT** 许可证，详见 [LICENSE](LICENSE)。

`assets/defaultSettings.yaml` 及 `tests/cases/` 中的测试数据来自
[cmhughes/latexindent.pl](https://github.com/cmhughes/latexindent.pl)，
仍遵循上游的 **GPL-3.0-or-later**，版权归原作者 Chris Hughes 及贡献者所有；
MIT 许可不适用于这些上游内容。
