//! Token 表（对应 LatexIndent/Tokens.pm）。token_check 在处理前升级
//! 所有与正文冲突的 token 名。

#[derive(Clone)]
pub struct Tokens {
    pub filecontents: String,
    pub trailing_comment: String,
    pub blanklines: String,
    pub verbatim: String,
    pub verbatim_inline: String,
    pub preamble: String,
    pub begin_of_token: String,
    pub double_back_slash: String,
    pub alignment_block: String,
    pub paragraph: String,
    pub sentence: String,
    pub end_of_token: String,
    pub m_before_begin_line_break_add: String,
    pub m_before_begin_line_break_remove: String,
    pub m_after_end_remove: String,
    pub m_after_end_line_break: String,
    pub m_switch_comment: String,
    pub m_switch_comment_fi: String,
    pub special_begin_end_middle: String,
    pub m_else_trailing_space: String,
}

impl Default for Tokens {
    fn default() -> Self {
        Tokens {
            filecontents: "LTXIN-TK-FILECONTENTS".into(),
            trailing_comment: "tc".into(),
            blanklines: "LTXIN-TK-bl".into(),
            verbatim: "LTXIN-TK-VERBATIM".into(),
            verbatim_inline: "LTXIN-TK-VERBATIM-inline".into(),
            preamble: "LTXIN-TK-preamble".into(),
            begin_of_token: "LTXIN-TK-".into(),
            double_back_slash: "LTXIN-TK-DOUBLEBACKSLASH".into(),
            alignment_block: "LTXIN-TK-ALIGNMENTBLOCK".into(),
            paragraph: "LTXIN-TK-PARA".into(),
            sentence: "LTXIN-TK-SENTENCE".into(),
            end_of_token: "-END".into(),
            // 原版此处即为 mBeforeBeginLineADD（无 Break），保持一致
            m_before_begin_line_break_add: "mBeforeBeginLineADD".into(),
            m_before_begin_line_break_remove: "mBeforeBeginLineBreakREMOVE".into(),
            m_after_end_remove: "mAfterEndRemove".into(),
            m_after_end_line_break: "mAfterEndLineBreak".into(),
            m_switch_comment: "mSwitchComment".into(),
            m_switch_comment_fi: "mSwitchCommentFi".into(),
            special_begin_end_middle: "specialMiddle".into(),
            m_else_trailing_space: "mElseTrailingSpace".into(),
        }
    }
}

impl Tokens {
    /// 与正文冲突的 token 追加 "-LIN" 直到唯一（对应 token_check，忽略大小写）。
    pub fn check(&mut self, body: &str) {
        let hay = body.to_lowercase();
        let all = self.all_mut();
        for t in all {
            while hay.contains(&t.to_lowercase()) {
                t.push_str("-LIN");
            }
        }
    }

    fn all_mut(&mut self) -> Vec<&mut String> {
        vec![
            &mut self.filecontents,
            &mut self.trailing_comment,
            &mut self.blanklines,
            &mut self.verbatim,
            &mut self.verbatim_inline,
            &mut self.preamble,
            &mut self.begin_of_token,
            &mut self.double_back_slash,
            &mut self.alignment_block,
            &mut self.paragraph,
            &mut self.sentence,
            &mut self.end_of_token,
            &mut self.m_before_begin_line_break_add,
            &mut self.m_before_begin_line_break_remove,
            &mut self.m_after_end_remove,
            &mut self.m_after_end_line_break,
            &mut self.m_switch_comment,
            &mut self.m_switch_comment_fi,
            &mut self.special_begin_end_middle,
            &mut self.m_else_trailing_space,
        ]
    }
}

use std::cell::RefCell;

thread_local! {
    pub static TOKENS: RefCell<Tokens> = RefCell::new(Tokens::default());
}

/// 读取 token 字符串快照（作用域内避免长期持有借用）。
pub fn tok<T>(f: impl FnOnce(&Tokens) -> T) -> T {
    TOKENS.with(|t| f(&t.borrow()))
}

pub fn token_check(body: &str) {
    TOKENS.with(|t| t.borrow_mut().check(body))
}
