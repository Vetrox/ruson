use crate::services::parser::Parser;
use std::fmt::Display;

#[derive(Debug)]
pub struct ErrorWithContext {
    pub error: SoNError,
    pub line: usize,
    pub col: usize,
}

impl Display for ErrorWithContext {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Error@{}:{} {:?}", self.line, self.col, self.error)
    }
}

#[derive(Clone, Debug)]
pub enum SoNError {
    NodeIdNotExisting,
    NumberCannotStartWith0,
    SyntaxExpected { expected: String, but_got: String },
    TypTransitionNotAllowed,
    VariableRedefinition { variable: String },
    VariableUndefined { variable: String },
    DebugPropagateControlFlowUpward,
}

impl SoNError {
    pub fn attach_context(&self, parser: &Parser) -> ErrorWithContext {
        let (line, col) = parser.lexer.dbg_position().unwrap_or((0, 0));
        ErrorWithContext { error: self.clone(), line, col }
    }
}