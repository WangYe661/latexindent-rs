# latexindent.pl 替换模式中的 Perl `/e`：上游行为与 Rust 方案调研

状态：**方案 B（原生模板 handler）已落地**，见 `src/replacement_e.rs`；截至本次实现，
测试结果为 278 通过 / 0 失败 / 1 `#[ignore]`（仅剩 PCRE2 无法复现的 `PRUNE` quirk）。

方法：上游 `cmhughes/latexindent.pl` 提交 `c8caec3cbf6bc017c7bdee2b08652ff58367a8f4`（master；`latexindent.pl` 头部标注版本 4.0.3，2026-10-04）。Perl 语义通过本机参考解释器 Perl 5.42.3（`perl -v`，MSWin32-x64）实测。

## 1. 上游实现

### 1.1 开关与调用路径

| 开关 | `GetOptions` 规格 | 标志变量 | 替换执行时机 |
|---|---|---|---|
| `-r` | `replacement\|r`（`latexindent.pl:60`） | `$is_r_switch_active` | 缩进之前，以及 verbatim 恢复之后 |
| `-rv` | `replacementrespectverb\|rv`（`latexindent.pl:62`） | `$is_rv_switch_active` | verbatim 被保护之后，以及恢复之前 |
| `-rr` | `onlyreplacement\|rr`（`latexindent.pl:61`） | `$is_rr_switch_active` | 只执行一遍；完全不做缩进 |

`LatexIndent/Switches.pm:43-49` 把解析后的开关映射到全局变量，并使 `-rr`/`-rv` 隐含 `-r`。在 `LatexIndent/Document.pm` 中，`-r` 在 `:166`（verbatim 保护之前）和 `:220`（恢复之后）执行替换；`-rv` 则在 `:181`（verbatim 已被保护）和 `:215`（恢复之前）执行，它不是独立算法，只是位置不同；`-rr` 在 `:168` 跳过整个缩进块，因此只执行 `:166`，并由 `Replacement.pm:51` 强制 `when` 为 `before`——与手册（`documentation/sec-replacements.rst:258-269`）一致。

### 1.2 `substitution` 如何执行

整个循环在 `LatexIndent/Replacement.pm:39-87`，关键三行是 `:82-84`：

```perl
my $body = ${$self}{body};
eval("\$body =~ ${$_}{substitution}");
${$self}{body} = $body;
```

这是**对任意 Perl 的字符串 eval**，不是专门的 `s///e` 解析器。由此可得：

- `substitution` 被插值成 `$body =~ <substitution>` 并在运行时编译。它必须是以 `=~` 开头的合法 Perl 表达式；`s///`、`tr///`、`m//` 以及额外语句都可以。实测：`s/(b)/X/; $body .= "!"` 得到 `aXc!`；`tr/a-z/A-Z/` 得到 `ABC`。
- eval 在 `make_replacements` 的词法作用域内编译，且处于 `use strict; use warnings` 下（`Replacement.pm:18-19`）。`$body`、`$self`、`%input`、`@replacements`、`%mainSetting` 均可见；未声明变量是**编译错误**，且会被静默吞掉。
- `$_` **不是**当前匹配文本：在 `foreach (@replacements)` 里它是当前哈希引用，所以在 `/e` 中裸写 `$_` 会插入 `HASH(0x...)`（实测：`aHASH(0x...)c`）。
- **没有检查 `$@`**。语法错误、坏正则，或 `/e` 内的 `die`，都会使 `$body` 保持不变、继续执行，且不影响退出码。eval 内的 warning 仍可能到达 STDERR。赋值是原子的：即使 `/g` 在前面已匹配多次，后续某次 `die` 也会让目标串完全不变。
- 若同时存在 `this` 和 `substitution`，`:64-66` 会警告并让 `this` 胜出。`this` 是字面量（`quotemeta` + `split`/`join`，`that` 缺省为空，`:72-74`）。条目按数组顺序执行（`:41`）；`lookForThis` 缺省为 `1`，任何假值都跳过（`:45-48`）；`when` 缺省为 `before`（`:51`）。

### 1.3 YAML 与 amalgamate

默认设置（`defaultSettings.yaml:418-424`）为：

```yaml
replacements:
  - amalgamate: 1
  - this: latexindent.pl
    that: pl.latexindent
    lookForThis: 0
    when: before
```

第一个元素是指令，不是替换。`LatexIndent/GetYamlSettings.pm:516-543` 实现合并：若传入数组的第一个元素定义了 `amalgamate`，该值覆盖主设置并移除该指令（`:519-521`）；为真时追加条目（`:531-539`），否则替换整个数组（`:541-543`）。这个顶层数组**不做去重**（与 `:497-501` 不同），所以重复条目都会执行。手册确认默认值为 1（`sec-replacements.rst:44-46`），以及 `amalgamate: 0` 会覆盖（`:101`）；字段语义见 `latexindent-yaml-schema.json:1366-1402`。

### 1.4 手册写了什么——以及没写什么

`documentation/sec-replacements.rst:5-32` 按上表定义三个开关。`:189` 给出 `s/regex/replacement/modifiers` 形式；`:195-199` 只记录 `s` 和 `g`；`:231` 记录 `when`；`:310-316` 记录替换文本中的 `$1`。对手册和 changelog 全文搜索“`e modifier`”无结果：**上游没有文档化 `/e`**。它能工作，只是因为 substitution 被原样交给 Perl。

在已检查的文件中未发现明确的最低 Perl 版本要求。发布工作流用 Perl 5.38 构建 Windows 可执行文件（`build-documentation-and-executables.yaml:78-100`）；手册只讨论模块可用性（`sec-introduction.rst:283-290`）。

## 2. 需要对齐的 Perl `/e` 语义

参考：本机 Perl 5.42.3；perldoc 5.42 的 `perlop`、`perlre`、`perlvar`、`perlrun`、`perlembed`。

| 行为 | 语义 | 本机实测 |
|---|---|---|
| `s/PAT/REPL/e` | 右侧是完整的 Perl 表达式，与外围代码一起编译，在匹配时求值；其值（最后一个表达式的值）就是替换字符串 | `s/(b)/uc($1)."X"/e` 作用于 `abc` → `aBXc` |
| `s/PAT/REPL/ee` | 右侧先作为字符串求值，再把该字符串作为 Perl `eval` | `s/(\d)/"$1 x 2"/ee` 作用于 `a1b2` → `a11b2` |
| 捕获 | `$1..$n`、`$+{name}`、`$&`、`$-[0]` 指向当前匹配；只读，动态作用域 | `$+{mid}` 和 `$-[0]` 均可用 |
| `$_` | 不会局部化为当前匹配；就是外围代码里的值 | 独立测试中裸 `$_` 产生空替换 |
| `/g` + `/e` | 每个匹配执行一次表达式，从左到右 | `s/(\w)/$1."!"/eg` 作用于 `a-b-c` → `a!-b!-c!` |
| 零宽匹配 | `/g` 会越过零宽匹配；不会死循环 | `s/a*/X/g` 作用于 `aaa` → `XX`，共 2 次替换 |
| 运行时 `die` | 中止该次 `s///`；目标字符串不变；`$@` 被设置 | 即使前面已有 `/g` 匹配也确认成立 |
| 词法作用域 | 表达式在外围词法作用域内编译，因此外层 `my` 变量可见 | 已确认 |
| `use re 'eval'` | 当*模式*由运行时文本构造且包含代码块时，出于安全需要启用 | 字符串 eval 的 `s///e` 不需要；它本来就按代码编译 |

主要引文：

- `perlop`（`https://perldoc.perl.org/perlop#s/PATTERN/REPLACEMENT/msixpodualngcer`）：“`e` 把右侧作为表达式求值；`ee` 先把右侧作为字符串求值，再 eval 结果。”以及“`/e` 会使替换部分被当作完整的 Perl 表达式，当场求值；不过它会在编译期做语法检查。”
- `perlre`（`https://perldoc.perl.org/perlre`）：修饰符列表“`e` - 将右侧作为表达式求值”；安全说明指出，对于来自运行时信息的代码，“`use re 'eval'` 必须在作用域内。这是为了阻止用户提供的、包含代码片段的模式被执行”，并建议污点检查或 `Safe` 隔间。
- `perlvar`：`$1` “包含当前动态作用域中最后一次成功模式匹配的子模式”，且“这些变量是只读的”。
- `perlrun`：`-e` “可用于输入一行程序”；`-0777`（或 `-g`）会 slurp 整个文件。

对移植版有两点实际推论：(1) `/e` 不是一个可以独立解析的标志——右侧就是 Perl，任意函数调用、字符串操作甚至嵌套正则都合法；(2) 右侧中的嵌套正则会覆盖 `$1`，与 Perl 完全一致。

## 3. Rust 实现方案

下表中的 crate 信息均来自 2026-10-31 通过 `cargo info`/`cargo search` 读取的 crates.io 元数据。“MSRV”是 crate 声明的 `rust-version`；本项目要求 1.70。

| 方案 | 具体 crate/工具（最新） | 是否 Perl 精确 | Windows | 构建前提 | MSRV 对比 1.70 | 运行时/体积 | 沙箱 | 许可 |
|---|---|---|---|---|---|---|---|---|
| (a) 嵌入 libperl | `libperl-sys` 0.4.6、`libperl-rs` 0.4.6、`ruperl` 0.0.1-alpha.1 | 是 | 未验证；无 Windows CI；构建脚本用 `:` 拼接 PATH | Perl 开发头文件、LLVM/libclang、首次构建需联网 | 未知 | 链接 libperl；体积未验证 | 无 | GPL-1.0+ OR Artistic-1.0-Perl（`ruperl`：Apache-2.0） |
| (b) 外部进程 | 系统 `perl`，配合 `-e` / `-0777 -pe` | 是（同一个二进制） | 需已安装 Perl；Strawberry/ActivePerl 可用 | 构建期无 | 不适用 | 本机实测约 21 ms/进程；200 KB 约 36 ms | 操作系统进程 | 仅调用；分发 Perl 会触发 Perl 许可 |
| (c) Perl→WASM | `zeroperl`（Perl 5.42，wasm32-wasi）、`goccy/perl-wasm`（5.42.2，wasm32-wasip1）、`wasmtime`/`wasmer` | 是，真实 Perl 核 | wasmtime 可在 Windows 运行 | 用 Docker/wasi-sdk 构建 guest；运行时 crate | wasmtime 14 = 1.70（最新 50 = 1.97）；wasmer 4.2.5 = 1.70（最新 7.5 = 1.96） | guest 的 Perl + stdlib 达数十 MB（未验证）；JIT 宿主再增加数 MB | WASI 能力 + fuel/epoch | guest 用 Perl 许可；运行时 Apache-2.0/MIT |
| (d) 翻译到其他运行时 | `mlua` 0.12.2（Lua）、`rhai` 1.27.0、`pyo3` 0.30.1、`boa_engine` 0.22.0、`deno_core` 0.413.0 | 否 | 除部分 Python 构建外均可 | 各异；pyo3 需要 Python，deno_core/V8 很重 | mlua 1.88（0.9.9 = 1.71）；rhai 1.66 可用；pyo3 1.85（0.20.3 = 1.56）；boa 1.91（0.17.3 = 1.69）；deno 未知 | Lua/Rhai 小，V8 大 | 取决于运行时 | 多为 MIT/Apache |
| (e) 自研解释器 | `perl-parser` 0.17.0、`perl-lexer`、`tree-sitter-perl` 1.1.2、`strykelang` 0.17.60 | 仅当完整实现时 | Rust 代码，可用 | 无 | `perl-parser` 1.95——不兼容；tree-sitter 未知；stryke 未知 | 小到大 | 完全可控 | MIT/Apache |
| (f) 指纹识别已知块 | 未找到现成方案 | 仅限白名单片段 | 不适用 | 无 | 不适用 | 可忽略 | 完全可控 | 不适用 |

**(a) 嵌入 libperl。** crates.io 上不存在名为 `perl-sys`、`perl-embed`、`perl` 或 `perl5-sys` 的 crate（`cargo info` 报 “could not find”）。真正的栈是 hkoba 的 `libperl-sys`/`libperl-rs`：需要 Perl 开发头文件、LLVM + libclang 供 bindgen 使用，首次构建需要联网；CI 覆盖 Perl 5.30–5.42（`hkoba/libperl-rs` README:26-37）。未声明支持 Windows；`libperl-sys/build.rs:52-55` 用 `:` 拼接 PATH，这是 Unix 分隔符（推断：在 Windows 上未测试）。`ruperl` 是 Apache-2.0 的概念验证，最后推送于 2024-08。`perlembed` 要求链接在目标机器上构建的 perl 库，多个同时存在的解释器需要 multiplicity/ithreads。

**(b) 外部进程。** `perl -0777 -pe '<substitution>'` 是完整的 slurp 模式过滤器（`perlrun`：`0400` 及以上的值会 slurp 整个文件；`0777` 是惯例）。本机实测：`perl -e 1` 约 21 ms/进程；200 KB 替换约 36 ms。每个条目起一个进程可行但开销可感知；常驻 worker 可省去启动，但会在片段之间泄漏解释器状态，且必须把每个 job 包在 `eval {}` 中并返回 `$@`。上游自己就打包了 Perl：`latexindent.exe` 在 Windows Perl 5.38 上使用 PAR::Packer（`build-documentation-and-executables.yaml:5-7, 78-100`）。

**(c) WASM。** `goccy/perl-wasm` 把 Perl 5.42.2 编译到 `wasm32-wasip1`，产出 `perl.wasm`、stdlib zip 以及 `perl_new`/`perl_eval`；它很新（2026 年，2 stars），需要 wasi-sdk/binaryen。`uswriting/zeroperl` 是一个“实验性”的 Perl 5.42 wasm32-wasi reactor 构建，带 asyncify。Go 生态已经出现这种形态：`goccy/go-perl`，以及报告 7–10 秒冷启动的 `lbe/go-exiftool-wasm`。WebPerl 不是可直接替换的 WASI 模块：它是 Perl 5.30 用 Emscripten 1.38 加 JS 胶水构建的（`haukex/webperl:build/emperl_config.sh`），最后推送于 2023。

**(d) 翻译到其他运行时。** 它们都不执行 Perl。Lua 有自己的 pattern 且没有 `/e`；`rhai` 没有 Perl 语法，正则只能靠插件；Python 的 `re.sub` 回调可以模拟 `/e`，但右侧仍需 Perl→Python 翻译；JS 同理。`mlua` 0.9.9 的 MSRV 是 1.71，已经高于本项目的下限。只对很小的白名单表达式子集现实可行。

**(e) 自研解释器。** `perl-parser` 是真正的解析器/AST，但声明 MSRV 1.95，且不是求值器；`tree-sitter-perl` 是文法。`strykelang` 声称兼容 Perl 5，但创建于 2026-04、6 stars，其 `/e` 保真度未经验证。覆盖任意 `/e` 表达式的子集实际上就是 Perl 本身：右侧可以调用内建函数、使用正则、声明词法变量并修改外部状态。

**(f) 指纹识别。** 未在 latexindent 移植版或 Perl 嵌入生态中找到先例。识别 14 个已知片段并用原生 Rust 替换，工作量小且安全，但只覆盖语料：用户 YAML 是开放的、写法会变，未匹配的片段需要明确的“不支持”诊断。

## 4. 先例

- `hkoba/libperl-rs`——活跃（0.4.6），对 libperl-sys 的较安全封装，Linux CI，pre-1.0。
- `foursixnine/ruperl`——alpha 概念验证，14 stars，最后推送 2024-08。
- `goccy/go-perl`（16 stars）和 `lbe/go-exiftool-wasm`——通过 WASM/wasm2go 运行真实 Perl，单一静态二进制，冷启动重；证明方案 (c) 可以跨平台工作。
- `haukex/webperl` + `TeXlyre/wasm-latex-tools`——面向 LaTeX 工具的浏览器 Perl，依赖 JS，不能在没有 JS 引擎的情况下嵌入 Rust 宿主。
- `inline-vbs`——通过 Windows ActiveScript 嵌入 VBScript/JScript/Perl；需要已注册的 PerlScript 引擎，不是通用方案。
- latexindent 移植版：GitHub 仓库搜索只找到 `WangYe661/latexindent-rs`；没有发现 Python 或 JavaScript 移植版。`WGUNDERWOOD/tex-fmt` 是独立的 Rust 格式化器，不是移植版。没有发现实现 `/e` 的移植版（这是搜索级结论，不是穷尽审计）。

## 5. 未决问题/风险

1. **许可。** 本仓库是 MIT。libperl 是 GPL-1.0-or-later OR Artistic-1.0-Perl；链接它，或分发 Perl WASM guest，会产生需要法务复核的分发义务。调用用户已安装的 Perl 避免了链接，但不等于“分发”。
2. **安全。** 上游设计执行来自 YAML 的任意 Perl；任何忠实实现都会继承这一点。沙箱（WASI 能力、无文件系统）会是有意偏离，读取文件的片段会失效。
3. **MSRV。** 当前 wasmtime (1.97)、wasmer (1.96)、mlua (1.88)、pyo3 (1.85)、boa (1.91)、perl-parser (1.95) 都超过 1.70；只有固定旧版本（wasmtime 14 / wasmer 4.2.5 / boa 0.17.3 / pyo3 0.20）可用，带来维护和 CVE 跟踪债务。
4. **Windows 构建。** libperl 需要 Perl 头文件加 libclang，且没有 Windows CI；构建脚本用 `:` 拼接 PATH 是危险信号。外部进程或 WASM 路线在 Windows 上明显更简单。
5. **可复现性与产物。** WASM guest 用 Docker/wasi-sdk 构建，未发布为 crate；供应链固定与校验和是未决问题。二进制增长和冷启动需要在承诺前实测。
6. **语义 oracle。** Perl 的正则/Unicode 行为与 pcre2 不同，且 `/e` 片段可以调用 Perl 模块。与固定 Perl（5.38，与上游发布的可执行文件一致）做差分测试是唯一可靠的 oracle；上游未文档化最低 Perl 版本。

---

## 6. 本仓库缺口审计与落地建议

> 本节审计对象是 `src/replacement.rs` 与 `tests/cases/**`。上游事实以第 1、2 节为准。

### 6.1 当前实现

- `apply_substitution()`（`src/replacement.rs`）只解析 `s<delim>pat<delim>rep<delim>flags`；`e` 被当作未知标志忽略。
- 含 `/e` 时，Perl 代码被当成普通双引号替换串展开，**代码文本会进入输出**，且**没有警告**。这比“保持原文不变”更糟。
- `-r` / `-rv` / `-rr` 的调用时机、`this`/`that`、`when`、`lookForThis` 已与上游一致。
- 上游执行的是 `eval("\$body =~ $substitution")`，因此兼容目标不是“解析 `/e` 标志”，而是“执行任意 Perl 字符串 eval”。`tr///`、`m//`、`s///; $body .= "!"` 都合法；`/e` 只是其中一个用法。这一点决定了方案边界。

### 6.2 夹具中的 `/e` 代码块

共 **14 个不同代码块、32 处引用**（同一代码块会被多个用例复用，故 32 处引用 ≠ 106 条调用），可分四类：

| 类别 | 代表文件 | 需要的 Perl 特性 | 难度 |
|---|---|---|---|
| ruler 生成 | `text-wrap/{addruler1..3,issue-359*,issue-444*,issue-450*,text-wrap-4,text-wrap-6}.yaml`、`documentation/demonstrations/addruler{1,2}.yaml` | `my`、C 风格 `for`、`int`、`x` 重复、`.=`、字符串插值、`\z` | 低 |
| cmhlistings 展平 | `documentation/manual/latexindent.yaml`、`batch-tests/documentation.yaml` | `my`、内部 `s@\R@@sg`、`.=` | 低 |
| 对齐填充 | `alignment/issue-382.yaml`、`issue-382a.yaml`、`te-752552a.yaml` | `split`/`join`、`foreach` 的 `$_` 别名写回、`sort { $a <=> $b }`、`push`、`length`、内部 `s///`、`/\X/g`、负数组下标；`issue-382` 另需 `LatexIndent::AlignmentAtAmpersand::get_column_width` | 高 |
| 数学公式规范化 | `replacements/replace26.yaml`、`issue-513.yaml`、`issue-517a.yaml`、`issue-524.yaml` | `my`、多条内部 `s///`、`$1`、`$+{name}`、`q()`、`.=` | 中 |

`issue-382a.yaml` 是 `issue-382.yaml` 的纯 Perl 版本：把 `get_column_width()` 换成 `while ( $s =~ /\X/g ) { $count++ }`。它证明对齐类代码可以完全不依赖 Perl 模块。

### 6.3 受影响的测试

| `#[ignore]` 测试函数 | 调用/断言数 | 依赖类别 |
|---|---:|---|
| `tests/wrap.rs::text_wrap_e_dependent_pending` | 57 | ruler |
| `tests/documentation.rs::documentation_demos_pending` | 11 | ruler |
| `tests/documentation.rs::documentation_manual_sources_pending` | 28（1 + 25 个 `s*.tex` + 2 个 `.bib`） | cmh |
| `tests/replacements.rs::replacement_e_cases_pending` | 6 | 数学 |
| `tests/alignment.rs::misc_e_substitution_cases` | 3 | 对齐 |
| `tests/batch_tests.rs::replacement_mode` | 1 | cmh |
| **合计** | **106** | |

README 目前写的“7 个测试”是按测试函数计；按调用数计是约 **106 条**。其中 **97 条**（ruler 68 + cmh 29）只需很低成本的实现，**9 条**（对齐 3 + 数学 6）才是真正复杂的部分。

### 6.4 子集解释器必须处理的 Perl 语义陷阱

如果选择自研解释器，以下行为必须逐项对齐，否则会“看起来能跑、字节不一致”：

- `foreach (@lines)` 的 `$_` 是数组元素的**别名**，对 `$_` 赋值会写回数组。
- `split("\n", $body)` 默认**丢弃尾部空字段**；`join("\n", @lines) . "\n"` 会重新补回。
- 标量的字符串/数字双重性：`$beforeJustLengths[-1]`、`" " x n`、`$1 ? $1 : q()`。
- 运算符优先级：`.` 低于 `*`，所以 `"   ".$i*5` 等价于 `"   ".($i*5)`。
- `=~ s///` 会**覆盖** `$1`/`$+{name}`；嵌套 `s///` 后要能读到新的捕获。
- `/g` 需要维护 `pos`；零宽匹配（`\z`、`a*`）要正确前进。
- 匹配/替换失败或 `die` 时，上游静默保持 `$body` 不变；实现也应“警告 + 原文不变”，不能 panic。
- `while ($s =~ /\X/g)` 的 `\X` 是字素簇；与 `--GCString` 的显示列宽是两套语义。

## 7. 方案对比

| 方案 | 语义精确度 | 新增依赖/构建 | Windows | 预计工作量 | 主要风险 | 可覆盖 |
|---|---|---|---|---|---|---|
| A. 现状 + 明确警告 | 无 | 无 | 无 | 0.5 小时 | 功能仍缺失 | 0 |
| B. 原生模板 handler（指纹识别） | 仅已识别模板 | 无 | 无 | 1–3 天 | 不通用、上游改夹具需跟进 | 106（按模板） |
| C. Rust Perl 子集解释器 | 子集内精确 | 无 | 无 | 1–2 周 | 语义细节多、维护成本 | 106 + 子集内用户 YAML |
| D. 外部 `perl` bridge | 与 Perl 完全一致 | 运行时需 Perl | Strawberry/ActivePerl 可用 | 1–2 天 | 运行时依赖、安全、CI | 106 + 任意 Perl |
| E. 嵌入 libperl | 与 Perl 完全一致 | Perl 头文件 + libclang + 链接 | 无 Windows CI，风险高 | 3–5 天+ | 构建、GPL/Artistic、unsafe | 106 + 任意 Perl |
| F. Perl→WASM | 真实 Perl 核 | wasmtime/wasmer + 预编译 guest | 可 | 1–2 周 | 体积、7–10s 冷启动报告、供应链 | 106 + 任意 Perl |
| G. Lua/Rhai/Python/JS + 翻译 | 否 | 各自运行时 | 部分 | 高 | 翻译层才是难点 | 不推荐 |
| H. B + D 混合 | 默认无依赖；可选精确 | 无 / 可选 Perl | 是 | 2–4 天 | 两套路径需一致性测试 | 106 + 任意 Perl |

### 7.1 成本收益

- ruler 类 68 条 + cmh 类 29 条 = **97/106**，实现都是纯字符串/行处理，约 1 天即可拿下的收益最大。
- 对齐类 3 条、数学类 6 条才是复杂部分；它们也最适合用“子集解释器”或“原生逐条移植”解决。
- 外部 `perl` bridge 是唯一能用 **1–2 天**换来“任意 Perl 语义”的方案；它还能反过来作为原生 handler 的**差分 oracle**。
- 嵌入 libperl 与 WASM 的收益和 bridge 相同，但构建/许可/体积风险高一个数量级，短期不建议。

### 7.2 外部 Perl bridge 的落地要点

- 二进制内嵌一个**不包含用户代码**的 driver，通过 `perl -e <driver>` 启动；用户数据走 stdin/stdout，避免命令行长度限制与注入。
- driver 复刻上游核心：
  ```perl
  my $body = decode('UTF-8', $body_bytes);
  my $sub  = decode('UTF-8', $sub_bytes);
  my $ok = eval "\$body =~ $sub";   # 与 Replacement.pm:83 相同
  # 返回 $body 与 $@
  ```
  这样 `s///e`、`tr///`、额外语句全部自然支持，比只实现 `/e` 更接近上游。
- 协议用长度前缀（或 JSON）承载 UTF-8 字节；`binmode` 设为 raw，显式 `Encode::decode`，避免 Windows 代码页问题。
- 每次替换起一个进程约 **21 ms**；106 条约 2 秒，可接受。若做常驻 worker，必须在每个 job 外包 `eval {}` 并回传 `$@`，避免状态泄漏。
- Perl 查找顺序：`PERL` 环境变量 → `PATH` 中的 `perl` → Windows 常见路径（Strawberry/ActivePerl）。找不到时**只警告一次**，原文不变。
- 建议做成可选/可关闭（`--perl-eval`、`LATEXINDENT_PERL_EVAL`、`--no-perl-eval`），并在 README 明示“会执行 YAML 中的任意 Perl 代码”。这与上游行为一致，但对 Rust 用户是新的安全边界。
- 许可：调用用户已安装的 Perl 不产生链接/分发义务；**不要随二进制打包 Perl**。

### 7.3 原生模板 handler 的落地要点

- 在 `apply_substitution()` 解析出 `flags` 后，若含 `e`，先尝试 `try_native_eval(rep_code)`；返回 `Option<impl Fn(&str) -> String>`。
- 识别方式建议“结构指纹”而非整段哈希：归一化空白/注释后检查特征串（`$rulerMax`、`$body =~ s@\R@@sg`、`\bnode\b`、`$+{name}` 等）。这样对上游的小改动更稳。
- ruler：直接按 Perl 的 `int`/`x`/字符串拼接语义生成常量，再替换 `\z` 处的零宽匹配。
- cmh：`$1` 去掉所有 `\R` 后补一个 `\n`。
- 对齐：`issue-382` 可复用 `src/alignment.rs` 的 `width()`（目前是私有，需要 `pub(crate)`），它已区分 `--GCString`；`issue-382a` 必须用字素簇计数（`\X`）而不能用 `width()` 的 GCString 语义。
- 数学：逐条移植，注意内部 `s///` 的捕获覆盖与 `$+{name}`。
- 未识别或执行失败：`logger::warn` + 原文不变（对齐上游的静默行为，但保留可诊断性）。

## 8. 已采纳的路线与结果

**已选方案 B（原生模板 handler）并完成 Phase 0 + Phase 1**：

1. **Phase 0（已完成）**：`src/replacement.rs` 检测 `e` 标志后先查原生 handler；未识别则 `WARN` 且原文不变、不 panic。消除了“把 Perl 代码写进输出”的静默损坏。
2. **Phase 1（已完成）**：`src/replacement_e.rs` 覆盖全部 14 个 eval 代码块（ruler / cmh / 对齐 / 数学），106 条受影响调用逐字节通过，`#[ignore]` 全部转正，且未新增 Cargo 依赖。
3. **Phase 2（未做，可选）**：外部 `perl` bridge。当前原生 handler 已覆盖测试语料；只有当需要支持**任意**用户 Perl 时才需要它。
4. **Phase 3（观察）**：WASM Perl / libperl，暂不评估。

在实现 `/e` 过程中顺带修掉的、与 `/e` 无关的上游对齐缺口（均有 Perl 侧 oracle 验证）：

- `blocksEndBefore` 分支缺括号 → `\begin{...}` 前的换行被吃掉（documentation/manual、text-wrap）。
- verbatim 命令的 id 用 `len + 1` 反推 → 嵌套 `\verb` 删除条目后 id 撞车，`\lstinline|...|` 被杀成别的命令。
- `-m` + `when: after` 与 `sentenceIndent` 不兼容规则缺失（上游会清空并告警）。
- 句子折行缺“缩进前”第一次（上游折两次）、缺 `removeSentenceLineBreaks` 合并与 `multipleSpacesToSingle` 压缩。
- `textWrapAfter` 缓存只记 per-name 规则、漏了 global 规则；列宽在 tab 折算后才计算。
- 注释折行的 `split(/\R/)[-1]`（须丢弃尾部空字段）与第二行缺 `subsequentSpace`。

## 9. 验收清单

- [x] `/e` 相关 `#[ignore]` 全部移除，106 条调用逐字节通过（`text-wrap` 57、`documentation` 11、`manual` 28、`replacements` 6、`alignment` 3、`batch` 1）。
- [x] 无 `/e` 的既有测试无回归（总计 278 通过 / 0 失败）。
- [x] 不新增 Cargo 依赖；`cargo clippy` 无新增告警。
- [x] 遇到未支持的 `/e` 时：`WARN`、原文不变、退出码不变、不 panic。
- [x] 用本机 Perl 5.42.3 + 上游 `latexindent.pl` 4.0.3 对拍（`THING/TWA/PCOUNT/REIND/BLOCK` 逐点比对）。
- [x] README 的 Known Limitations、本文件的结论同步更新。
- [ ] 安全说明：目前只在 README 里说明“`/e` 不支持任意 Perl”，未单独写“YAML 可触发任意 Perl 代码执行”的安全提示（Phase 2 若引入 bridge 才需要补）。