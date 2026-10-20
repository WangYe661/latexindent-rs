//! 运行期全局状态（原版 Perl 全局变量的对应物；单线程 CLI 用 thread_local 即可）。

use crate::patterns::PatternSet;
use crate::settings::MainSettings;
use std::cell::RefCell;
use std::collections::HashMap;

/// verbatim / noindentblock / preamble / linesprotect 存储块
#[derive(Clone, Debug, Default)]
pub struct VerbatimObj {
    pub begin: String,
    pub body: String,
    pub end: String,
    pub name: String,
    pub typ: String, // noindentblock|environment|command|special|preamble|linesprotect
    pub yaml_name: String,
    pub id: String,
    pub replacement_text: String,
    pub opt_arg: String,
    pub horizontal_trailing_space: String,
    pub linebreaks_at_end: String,
    /// EndFinishesWithLineBreak 的生效值（仅 -m 且 verbatim/special 有该 poly-switch 时有意义）。
    /// `put_verbatim_back_in` 需要知道它是否为 -1，才能按 Perl Verbatim.pm L517-524
    /// 在还原前先吃掉占位符后面的换行。
    pub end_finishes_with_line_break: i64,
}

/// 尾注释存储条目
#[derive(Clone, Debug)]
pub struct TrailingComment {
    pub id: String,
    pub value: String,
    pub align_markup: bool,
}

/// one-sentence-per-line 的句子存储（对应 @sentenceStorage）。
#[derive(Clone, Debug, Default)]
pub struct SentenceObj {
    pub id: String,
    pub value: String,
    pub leading_horizontal_space: String,
    pub follows: String,
}

#[derive(Default)]
pub struct Storage {
    pub verbatim: HashMap<String, VerbatimObj>,
    pub trailing_comments: Vec<TrailingComment>,
    pub comment_counter: usize,
    pub filecontents: Vec<VerbatimObj>,
    pub file_starts_with_line_break: bool,
    pub file_ends_with_trailing_comment: bool,
    pub align_markup_block_present: bool,
    pub sentences: Vec<SentenceObj>,
    pub sentence_counter: usize,
}

thread_local! {
    static SETTINGS: RefCell<Option<MainSettings>> = const { RefCell::new(None) };
    static PATTERNS: RefCell<Option<PatternSet>> = const { RefCell::new(None) };
    static STORAGE: RefCell<Storage> = RefCell::new(Storage::default());
}

pub fn init_settings(s: MainSettings) {
    SETTINGS.with(|g| *g.borrow_mut() = Some(s));
}

pub fn init_patterns(p: PatternSet) {
    PATTERNS.with(|g| *g.borrow_mut() = Some(p));
}

pub fn settings<T>(f: impl FnOnce(&MainSettings) -> T) -> T {
    SETTINGS.with(|g| f(g.borrow().as_ref().expect("settings not initialised")))
}

pub fn settings_mut<T>(f: impl FnOnce(&mut MainSettings) -> T) -> T {
    SETTINGS.with(|g| f(g.borrow_mut().as_mut().expect("settings not initialised")))
}

pub fn patterns<T>(f: impl FnOnce(&PatternSet) -> T) -> T {
    PATTERNS.with(|g| f(g.borrow().as_ref().expect("patterns not initialised")))
}

pub fn storage<T>(f: impl FnOnce(&mut Storage) -> T) -> T {
    STORAGE.with(|g| f(&mut g.borrow_mut()))
}

pub fn reset_storage() {
    STORAGE.with(|g| *g.borrow_mut() = Storage::default());
}
