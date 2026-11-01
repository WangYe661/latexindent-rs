mod alignment;
mod blank_lines;
mod blocks;
mod document;
mod double_backslash;
mod file_contents;
mod heading;
mod logger;
mod mlb;
mod num;
mod patterns;
mod replacement;
mod replacement_e;
mod sentence;
mod settings;
mod state;
mod switches;
mod tokens;
mod trailing_comments;
mod verbatim;
mod whitespace;
mod wrap;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    document::run(&args);
}
