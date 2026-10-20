# latexindent-rs

**English** | [简体中文](README.zh-CN.md)

> **Experimental Project Notice**
>
> This is a 100% vibe coding experimental project intended to test the coding capabilities of language models. It is not actively maintained. Use it in production with caution.

A Rust rewrite of [latexindent.pl](https://github.com/cmhughes/latexindent.pl) (v4.0.2)
for beautifying `.tex`, `.sty`, `.cls`, and `.bib` files.

This is a behavior-compatible port: it reuses the upstream `defaultSettings.yaml`
configuration model and runs the upstream test cases with byte-for-byte output
comparison.

## Features

- **Fully compatible command-line arguments**: supports all upstream switches,
  including `-w`, `-o`, `-l`, `-y`, `-d`, `-m`, `-r/-rv/-rr`, `-k/-kv`, `-n`,
  `-c`, `-g`, `-s`, `-t/-tt`, `-sl`, and `--GCString`, with the same argument
  forms and semantics.
- **Complete indentation support**: environments, commands, mandatory/optional
  arguments, `specialBeginEnd` blocks, unnamed groups, `ifelsefi`, `items`, and
  headings (`\part`/`\chapter`/`\section`…).
- **Alignment and line breaks**: `&` column alignment (including `dontMeasure`,
  `multiColumnGrouping`, `alignContentAfterDoubleBackSlash`, etc.), `-m`
  poly-switch line-break insertion/removal, text wrap, and one-sentence-per-line.
- **Regex engine**: PCRE2 (same family as Perl), with Unicode and grapheme-cluster
  width support.

## Installation and Configuration

Methods 2 and 3 require a stable Rust toolchain (edition 2021) and a C compiler;
`pcre2-sys` statically builds its bundled source when PCRE2 is not available on
the system.

### Method 1: Download a prebuilt binary

Open the [Releases](https://github.com/WangYe661/latexindent-rs/releases) page
and download the archive for your platform. It contains a single executable:

- Windows: `latexindent-rs.exe`
- macOS / Linux: `latexindent-rs`

Place the executable in any directory and add it to `PATH`. Rust is not required.

### Method 2: Install directly from GitHub with cargo install

```bash
cargo install --git https://github.com/WangYe661/latexindent-rs --locked
```

The executable is installed to `~/.cargo/bin`
(`%USERPROFILE%\.cargo\bin` on Windows); make sure that directory is on your
`PATH`. To pin a specific tag, add `--tag <tag>`.

### Method 3: Clone and install locally

```bash
git clone https://github.com/WangYe661/latexindent-rs.git
cd latexindent-rs

# Install to ~/.cargo/bin
cargo install --path .

# Or build without installing
cargo build --release
# Output: target/release/latexindent-rs (latexindent-rs.exe on Windows)
```

Verify the installation:

```bash
latexindent-rs --version
# 4.0.2, 2026-06-06
```

### VS Code Configuration (LaTeX Workshop)

After installing the
[LaTeX Workshop](https://marketplace.visualstudio.com/items?itemName=James-Yu.latex-workshop)
extension, add the following to VS Code's `settings.json`:

```json
{
  "latex-workshop.formatting.latex": "latexindent",
  "latex-workshop.formatting.latexindent.path": "path to latexindent-rs"
}
```

Replace `"path to latexindent-rs"` with the actual path to the executable. If
`latexindent-rs` is already on `PATH`, you can also use `latexindent-rs` directly
(`latexindent-rs.exe` on Windows).

### YAML Configuration

The default configuration is embedded in the binary
(`assets/defaultSettings.yaml`); if a `defaultSettings.yaml` exists next to the
executable or in the current directory, the external file takes precedence. User
configuration is layered as follows:

- `indentconfig.yaml` / `.indentconfig.yaml`: located in the home directory,
  `~/.config/latexindent/`, `%LOCALAPPDATA%\latexindent\` (Windows), etc.; adds
  YAML search paths.
- `-l` specifies local YAML files; a bare `-l` tries `localSettings.yaml`,
  `latexindent.yaml`, `.localSettings.yaml`, and `.latexindent.yaml` in order
  (working directory first, then current directory).
- `-y` passes a YAML snippet to override settings; `-d` ignores all user YAML and
  uses only the default configuration.

## Usage

```bash
# Write the result to stdout
latexindent-rs myfile.tex > myfile-indented.tex

# Overwrite in place (a backup is created first)
latexindent-rs -w myfile.tex

# Write to a specific file
latexindent-rs -o indented.tex myfile.tex

# Read from STDIN
cat myfile.tex | latexindent-rs -

# Use localSettings.yaml in the current directory and modify line breaks
latexindent-rs -l -m myfile.tex

# Override YAML settings on the command line
latexindent-rs -y="defaultIndent: '  '" myfile.tex

# Only process lines 10–20
latexindent-rs -n 10-20 myfile.tex

# Check mode: exit code 0 if unchanged, 1 if changed
latexindent-rs -k myfile.tex
```

If the input file has no extension, `fileExtensionPreference` is used to look for
`.tex`, `.sty`, `.cls`, and `.bib` in that order.

### Example

Input:

```latex
\begin{itemize}
\item first
\item second
\end{itemize}
```

Output (default indentation is a tab):

```latex
\begin{itemize}
	\item first
	\item second
\end{itemize}
```

See `latexindent-rs --help` for the full list of switches and the
[upstream documentation](https://latexindentpl.readthedocs.io/) for all YAML
options.

## Tests

The test cases come from the upstream `latexindent.pl` `test-cases`. Each case is
copied to an isolated temporary directory, the binary is executed, and the output
is compared byte-for-byte against the baseline file.

```bash
cargo test
```

Current result: **268 tests passed, 0 failed, 8 marked `#[ignore]`**. Run
`cargo test -- --ignored` to see these known gaps.

## Known Limitations

- **The speed improvement over the original project is lower than expected.**
- Perl's `/e` (execute code inside a substitution) is not implemented in
  `-r` / `-rv` / `-rr` replacement mode. Several upstream test cases depend on
  `/e` in their replacement rules, so 7 tests remain `#[ignore]`.
- The upstream Perl regex `PRUNE` leak is engine-specific behavior that PCRE2
  cannot reproduce; only the `found:` entries in the trace log differ, while the
  formatted output is identical (1 test `#[ignore]`).

Apart from these, all functionality covered by the upstream test suite is
implemented with byte-for-byte identical output.

## Project Structure

| File | Upstream module / responsibility |
| --- | --- |
| `src/document.rs` | `Document.pm`, `FileExtension.pm`, `BackUpFileProcedure.pm`, `Check.pm`, `LogFile.pm`: main flow |
| `src/settings.rs` | `GetYamlSettings.pm`: YAML loading, merging, and settings lookup |
| `src/blocks.rs` | `Blocks.pm`: code block detection and indentation |
| `src/patterns.rs` | Code block regex assembly (Perl recursive regex → PCRE2 `(?(DEFINE)…)`) |
| `src/alignment.rs` | `AlignmentAtAmpersand.pm`: `&` alignment |
| `src/mlb.rs` | `ModifyLineBreaks.pm`: `-m` poly-switches |
| `src/verbatim.rs` | `Verbatim.pm`: verbatim / noindentblock protection |
| `src/file_contents.rs` | `FileContents.pm`: filecontents and preamble |
| `src/replacement.rs` | `Replacement.pm`: `-r` replacement modes |
| `src/heading.rs` | `Heading.pm`: indentation after headings |
| `src/sentence.rs` | `Sentence.pm`: one-sentence-per-line |
| `src/wrap.rs` | `Wrap.pm`: text wrap |
| `src/whitespace.rs` | `HorizontalWhiteSpace.pm` |
| `src/trailing_comments.rs` | `TrailingComments.pm` |
| `src/blank_lines.rs` | `BlankLines.pm` |
| `src/double_backslash.rs` | `DoubleBackSlash.pm` |
| `src/tokens.rs` | `Tokens.pm`: placeholder token table |
| `src/switches.rs` | Command-line parsing and switch state |
| `src/logger.rs` | `Logger.pm` / `LogFile.pm`: logging and `indent.log` |
| `src/state.rs` | Counterpart of Perl globals (thread-local state) |

`tests/cases/` contains the upstream test cases, `tests/common/mod.rs` is the
temporary-directory test fixture, and `tests/*.rs` organizes assertions by feature
area.

## License

The code of this project is licensed under the **MIT** license; see
[LICENSE](LICENSE).

`assets/defaultSettings.yaml` and the test data under `tests/cases/` come from
[cmhughes/latexindent.pl](https://github.com/cmhughes/latexindent.pl) and remain
under the upstream **GPL-3.0-or-later** license. Copyright belongs to Chris Hughes
and contributors. The MIT license does not apply to this upstream content.
