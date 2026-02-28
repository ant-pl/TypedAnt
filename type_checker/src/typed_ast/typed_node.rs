use ast::StmtId;
use token::token::Token;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TypedNode {
    Program {
        token: Token,
        statements: Vec<StmtId>,
    }
}