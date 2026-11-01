# latexindent-rs 提速调研：并行处理与增量格式化

状态：**已落地五项改动**（选择性 PCRE2 JIT + 正则编译缓存、多文件并行、`--cache` 增量缓存、
尾注释还原改单遍、行首/行尾空白改手写扫描）**并修复了一个既有缺陷**
（基准文件 `sampleBEFORE-smaller.tex` 在 `HEAD` 上会 panic）。

测量环境：Windows 11 x64、16 逻辑核、`cargo build --release`（`opt-level = 2`），
基准文件取自 `tests/cases/benchmarks/`（与上游 `benchmarks.sh` 同一组）。
对照数据为上游 Perl 版在同一批文件上记录的时间（`tests/cases/benchmarks/*.data`）。**下表所有
`latexindent-rs` 数字都是本次改动前后在同一台机器上实测**（每例取多跑中的最小值）。

## 1. 结果总览

| 基准 | 文件大小 | 上游 Perl | 改动前 | 改动后 | 相对改动前 |
| --- | ---: | ---: | ---: | ---: | ---: |
| `issue-268` | 41 KB | 0.61 s | 0.50 s | 0.40 s | 1.3× |
| `sampleBEFORE-smaller`¹ | 70 KB | 0.57 s | 3.51 s（**panic**） | **0.52 s**（正常输出） | — |
| `sampleBEFORE` | 208 KB | 1.73 s | 2.09 s | 1.29 s | 1.6× |
| `commands-simple-big` | 359 KB | 2.62 s | 6.17 s | **1.49 s** | **4.1×** |
| `paperSS122018arxivv2` | 1.9 MB | 21.5 s | 57.8 s | **20.5 s** | **2.8×** |

¹ 旧版二进制在这个文件上直接 panic（`match limit exceeded`）；现在能跑完，输出与「同一文档的
完整版 `sampleBEFORE.tex`」在重叠区**逐行一致**（见 `tests/engine_limits.rs`），且结果幂等。
五个基准现在都不慢于上游 Perl。

`sampleBEFORE` 只涨 1.6×，是因为它的时间几乎全在递归的 `all_code_blocks`
（按第 3 节的结论**不能**开 JIT）；而 `commands-simple-big`、`paper…` 里大量时间原本花在
行首/行尾正则与重复编译上，所以收益明显。

多文件并行与增量缓存的收益（`commands-simple-big` 的 4 份副本）：

| 场景 | 耗时 | 说明 |
| --- | ---: | --- |
| `-j 1`（顺序） | 5.98 s | 基线 |
| `-j 4` | 1.83 s | 3.3×（4 个文件） |
| `-j 8` | 1.93 s | 4 个文件已跑满，不再有收益 |
| `-w --cache` 冷启动（2 文件） | 3.92 s | 首次格式化 |
| `-w --cache` 二次运行（内容未变） | **0.027 s** | 145×，命中缓存 |
| 改动 1 个文件后再跑 | 1.58 s | 只重算变化的那一个 |

## 2. 方法：先定位，再优化

动手前用了两种手段：

1. **分阶段探针**：在 `operate_on_file` 的每个流水线步骤之间打点（临时 `src/prof.rs`，
   现已删除），把 2 MB 文件的耗时拆到各步骤；
2. **PCRE2 微基准**：对同一份 360 KB 文本，分别用「解释器 / JIT / 手写扫描」跑同一条正则。

探针在 `commands-simple-big`（359 KB）上的原始分布：

```
dodge/remove_leading/mlb_pre            2154.9 ms   ← 其中 remove_leading_space 2179 ms
blocks::find_all_code_blocks            1573.4 ms
unprotect/un_dodge/max_indent/rm_trailing(after) 2494.9 ms ← 其中 max_indentation_check 2516 ms
file_contents::find_file_contents         14.9 ms
verbatim::* / trailing_comments::*          ~8 ms
TOTAL                                   6244.7 ms
```

微基准（`(?m)^\h*` 对 359 KB / 26026 行的整文替换）：

| 实现 | 耗时 |
| --- | ---: |
| PCRE2 解释器 | 2153.2 ms |
| PCRE2 JIT | 3.5 ms |
| 手写逐行扫描 | 0.4 ms |

即：**绝大部分时间不是花在「识别代码块」上，而是花在几条再简单不过的行首/行尾正则上**，
而它们慢的唯一原因是 `pcre2` crate 默认**不开 JIT**。

## 3. 改动一：选择性 PCRE2 JIT + 正则编译缓存（收益最大）

### 3.1 为什么解释器会慢 600 倍

`(?m)^\h*` 这类「锚点 + 可变长量词」的模式，在未锚定扫描时解释器要**逐字节**尝试起点；
PCRE2 的 JIT 会为此生成专门的「找下一行首」快速路径，于是 2.1 s 变成 3 ms。
`pcre2::bytes::RegexBuilder` 的默认值是 `jit_if_available(false)`（`JITChoice::Never`），
所以之前的每一个正则都是解释执行。

### 3.2 为什么不能「全部开 JIT」

直接把所有正则都开 JIT 后，**递归模式**（`all_code_blocks` 等由 `(?(DEFINE)…)` +
`(?&dNAME)` 拼出来的代码块正则）会开始报错：

| 输入 | 错误 | 后果 |
| --- | --- | --- |
| `paperSS122018arxivv2.tex` | `-46 JIT stack limit reached`（默认 32 KB 栈） | 精确版失效，整段回退到「所有格」兜底版 |
| 同上，栈上调到 64 MB | `-47 match limit exceeded` | 同上 |
| `sampleBEFORE-smaller.tex` | `-46` → 兜底版也 `-47` | **panic** |

原因：PCRE2 解释器的 `match_limit`（默认 10 000 000）统计的是「内部 `match()` 调用次数」，
而 JIT 没有 `match()` 递归，它用**另一个计数**近似同一个上限；对这种深度递归、单次匹配覆盖
整篇文档的模式，JIT 的计数会先耗尽。`pcre2` 0.2 没有暴露 `pcre2_set_match_limit`，
也就无法在 JIT 下放宽这个限制。

因此最终规则是：**含子程序调用/递归的正则不用 JIT，其余全部用 JIT**。
判定放在唯一入口 `patterns::re_new` 里（`has_subroutine_calls`：源码含 `(?&`、`(?R`、`(?0`），
这样行为与 Perl 一致的解释器路径保持原样，其余热点正则拿到数十倍加速。

### 3.3 正则编译缓存

`re_new` 的调用点常常在「每个块」上重复编译同一份源码：

- `blocks::make_middle_regex` 每次都拼一遍 `defs`（十几 KB）；
- `trailing_comments::put_trailing_comments_back_in` 对**每条注释**编译 1–2 条正则；
- 多文件时每个文件还要重建整套 `PatternSet`（`patterns::build`）。

改成一个全局 `RwLock<HashMap<src, Regex>>`（上限 512 条，满了整体丢弃，只退化重新编译）。
`Regex::clone` 只复制 `Arc<Code>` 与 match-data 池，远低于重新编译（含 JIT 编译）的成本。

## 4. 改动二：多文件并行

### 4.1 为什么这次能并行

这个移植版的全局状态**本来就是 thread-local 的**（`state::SETTINGS/PATTERNS/STORAGE`、
`switches::SWITCHES`、`tokens::TOKENS`、`document::CURRENT_FILE/CURRENT_EXT/BODY/ORIGINAL`），
所以「一个文件一个线程」不需要给整条流水线加锁；需要额外处理的只有四处：

| 问题 | 处理 |
| --- | --- |
| `indent.log` 顺序 | 每个文件的日志先写进自己的 `Arc<Mutex<Vec<String>>>`，收尾阶段按文件顺序 `merge_captured`；实测 `-j 1` 与 `-j 4` 的日志除 `UNIX time` 时间戳与文件名外逐行相同 |
| 输出顺序 / stdout | 处理与写文件分离：worker 只产出文本，主线程按命令行顺序写 `-o`/`-w`/stdout |
| `-o ++` 计数、`fileExtensionPreference` 补全 | 仍在主线程顺序完成（准备阶段） |
| 备份文件名竞争（`maxNumberOfBackUps` 轮转） | `create_back_up_file` 内加 `BACKUP_LOCK` 串行化（纯 IO，代价可忽略） |

### 4.2 保持兼容性

- `-j 1`（以及只有单个文件、或开了 `-sl` 屏幕日志时）走**原来的顺序代码路径**，包括
  `-wd` 的开关继承这类上游 quirk；
- 并行路径每个文件独立初始化 `settings`/`tokens`/`patterns`，因此 `token_check` 的
  「撞车后追加 `-LIN`」不会跨文件串味（顺序路径也随之改成每文件 `tokens::reset()`，
  这只会影响日志里的 token 名，不影响产物；全套用例通过）；
- 默认 `-j 0`（按 CPU 数，且不超过文件数）。

## 5. 改动三：`--cache` 增量格式化

编辑器「保存即格式化」的场景里，同一个文件会被反复喂进来。缓存条目（默认
`<cruft>/.latexindent-cache/<hash(绝对路径)>.fmt`）记录：

```
lirs-cache-1 <设置指纹> <输入哈希> <输出哈希>
<上次的输出文本>
```

命中两种：

1. **输入哈希相同** —— 内容没变，上次的输出就是答案（完全确定，无任何假设）；
2. **输出哈希等于当前输入** —— 文件已经是本工具（同版本、同设置）上一次的产物，
   格式化幂等 → 直接跳过。

指纹 = 工具版本 + 生效的 YAML 设置 + `-m/-r/-rv/-rr/--GCString`。任一变化即失效。
`-t/-tt`（trace）绕过缓存，否则日志会缺内容。

第 2 条的「幂等」是本项目的一条可验证性质：对 `tests/cases/{environments,commands,items}`
里 120 个输入各格式化两遍，第二遍与第一遍**全部逐字节相同**（120/120）。

## 6. 改动四：修掉那个 panic（块识别的四级回退）

### 6.1 病灶

`tests/cases/benchmarks/sampleBEFORE-smaller.tex` 在 `HEAD` 的二进制上直接 abort：
精确版 `all_code_blocks` 撞 `match limit`，兜底的所有格版同样超限，`.expect()` 收场。
上游 Perl 对这个文件 0.57 s 跑完，所以这是移植引入的缺陷，不是输入不合法。

根因有两条，**必须同时收紧两个维度才治得好**：

1. `(?<ENVBODY>(?:(?&dENV)|(?!\end{).)*)` —— 环境体里每遇到一个 `\begin` 都有
   「当嵌套环境吃」和「当单字符吃」两种读法；该文件末尾有一个**未闭合**的
   `\begin{exercises}`，`\end{exercises}` 找不到时就是 2^N 条路径；
2. 参数体循环 `(?:(?:head)|(?:arg_body)|(?&dCODE))*` 同样可回溯。

单独把哪一条改成「确定性」都还不够：实测（把两个变体分别跑在同一份 token 化正文上）
`env body 原子化` 和 `参数体所有格` 各自仍然超限，**两者叠加**才 0.19 s 扫完全文。

### 6.2 四级回退链

`src/blocks.rs::scan_with_fallback` 逐级收紧，每一级都从上一级停下的**字节偏移**继续
（`patterns::try_replace_all_with_upto` 交出已渲染前缀），所以前面的块语义仍是 Perl 那套：

| 级别 | 变体 | 治的病 |
| --- | --- | --- |
| 0 | 精确版（照抄 Perl，可回溯） | 正常输入只走这一级，行为与旧版逐字节一致 |
| 1 | env body 原子化（`(?>…)`） | 未闭合环境造成的 2^N |
| 2 | 参数体所有格（`dCODE` 优先 + `*+`） | 参数体回溯 |
| 3 | 1 + 2 | 两者并存（`sampleBEFORE-smaller` 就是这种） |
| 4 | 按行分块（优先空行，深度 ≤ 8，下限 4 KB） | 兜底：再切小重跑，实在不行原样保留并告警 |

变体 1–3 都**惰性编译**（`patterns::lazy_variant`），正常文件一行都不会多编译；
`-t` 会在 `indent.log` 里留下每级失败的位置，便于排查。

### 6.3 正确性验证

- `tests/engine_limits.rs`：该文件不再 panic；且它的输出与「同一文档的完整版
  `sampleBEFORE.tex`（精确版全程成功）」在重叠区 **2001 行逐行一致**；
- 该输出再格式化一次不变（幂等）；
- 全套用例 288 通过 / 0 失败 / 1 `#[ignore]`。

## 7. 改动五：两处线性扫描替换

| 位置 | 旧实现 | 新实现 |
| --- | --- | --- |
| `trailing_comments::put_trailing_comments_back_in` | 每条注释编译两条正则 + 整篇扫 1–2 次（O(n×m)，1.9 MB 文件 0.76 s） | 单遍扫描 + 行内定位 |
| `whitespace::{remove_leading_space, remove_trailing_whitespace, max_indentation_check}` | `(?m)^\h*` / `\h+$` / `^((?:[ \t]*))` 三条正则 | 逐行手写扫描（`\h` 字面量精确对齐，含 `\r` 不算 `\h` 的 quirk） |

实测整篇 `paperSS122018arxivv2`：**22.9 s → 20.5 s**（这两处合计约 2.4 s；
空白三条正则在有 JIT 之后本来已降到十几毫秒，改成手写更多是为了去掉正则开销）。

尾注释还原保留了一条**兼容路径**：同一行出现多个注释 token 时（`-m` 的行合并会产生，
如 `pgfkeys-remove-line-breaks` 用例），token 之间会互相影响，此时回退到逐条处理的旧实现，
保证语义不变（这种情况在真实文档里极少：`%` 注释天然吃到行尾）。

顺带记一个 PCRE2 语义坑：Perl 的条件正则 `token((?:(?!(?<!\\)%).*?))(\h*)?$` 里那个负向断言
**没有量化**，所以只在 token 紧后面检查一次——即「紧跟的字符是不是未转义的 `%`」，
后面 `.*?` 照样能吞掉 `%`。手写版本必须按这个（而不是「行内是否出现 `%`」）来判断，
否则 `-m` 生成的 `}%tc5-END{%tc3-END` 这类行会左右颠倒。

## 8. 还剩哪些提速空间

按「收益 ÷ 风险」排序：

1. **超大文档上精确版的「白跑」**：1.9 MB 的 `paperSS122018arxivv2` 现在 20.5 s，其中
   精确版扫到 12% 处（约 5 s）才失败，随后用原子/所有格版从断点续跑。再往前的路线是
   给 `pcre2` crate 打补丁以暴露 `pcre2_set_match_limit`（或用 `pcre2-sys` 自行编译），
   之后就能让 JIT 跑精确版（实测 JIT 下这个模式是秒级）。
2. **单文件内部并行**：不建议。`render_block` 递归里会按顺序写入 verbatim/注释计数器，
   id 顺序直接决定产物；拆块并行会改变输出。
3. **`expand_tabs` 等剩余逐字符 `push`**：只在设置了 `maximumIndentation` 时生效
   （默认 `-1`），优先级低。
4. **缓存条目的垃圾回收**：目前按路径一文件一条目，长期使用后目录里会有失效条目；
   可按 mtime 清理（收益是磁盘，不是 CPU）。


## 9. 复现

```bash
cargo build --release

cd tests/cases/benchmarks
BIN=../../target/release/latexindent-rs
time $BIN -s -l sample sampleBEFORE
time $BIN -s commands-simple-big.tex -o=+-out1 -l arg-minimal-between -y="defaultIndent: '  '"
time $BIN -s -l paperSS122018arxivv2 -o=+-mod1 paperSS122018arxivv2.tex

# 并行
cp commands-simple-big.tex a.tex && cp a.tex b.tex && cp a.tex c.tex && cp a.tex d.tex
time $BIN -s -j 1 -o=+-s a.tex b.tex c.tex d.tex
time $BIN -s -j 4 -o=+-p a.tex b.tex c.tex d.tex

# 增量缓存
$BIN -s -w --cache a.tex b.tex c.tex d.tex   # 冷
$BIN -s -w --cache a.tex b.tex c.tex d.tex   # 热（应 < 0.1 s）
```

回归测试：
`tests/performance_switches.rs`（并行/顺序产物与日志一致、缓存命中与失效、trace 绕过缓存、
`--cache=<dir>` 落盘位置）、`tests/engine_limits.rs`（未闭合环境不再 panic，且回退输出与
完整文档逐行一致）、`src/patterns.rs` 与 `src/whitespace.rs` 里的单元测试（回退变体真的改到了
源码、`\h` 语义与 PCRE2 一致）。
