mod parse_functions;
pub mod tests;
use std::{collections::HashMap, rc::Rc};

use ast::{expr::Expression, node::Node, stmt::Statement};
use token::{token::Token, token_type::TokenType};

use crate::{
    error::{ParserError, ParserErrorKind},
    parse_functions::{
        parse_assign::parse_assign, parse_block::parse_block_expr, parse_bool::parse_bool, parse_build_struct::parse_build_struct, parse_call::parse_call, parse_extern::parse_extern, parse_field_access::parse_field_access, parse_func::parse_func, parse_ident::parse_ident, parse_if::parse_if, parse_infix::parse_infix, parse_let::parse_let, parse_num::{parse_i64, parse_u64}, parse_str::parse_str, parse_struct::parse_struct, parse_while::parse_while
    },
    precedence::{Precedence, get_token_precedence},
};

pub mod error;
pub mod precedence;

type ParseResult<T> = Result<T, ParserError>;

type PrefixParseFn = fn(&mut Parser) -> ParseResult<Expression>;
type InfixParseFn = fn(&mut Parser, Expression) -> ParseResult<Expression>;
type StmtParseFn = fn(&mut Parser) -> ParseResult<Statement>;

pub struct Parser {
    tokens: Vec<Token>,

    pos: usize,
    next_pos: usize,

    pub cur_token: Token,
    pub peek_token: Token,

    prefix_parse_fn_map: HashMap<TokenType, PrefixParseFn>,
    infix_parse_fn_map: HashMap<TokenType, InfixParseFn>,
    statement_parse_fn_map: HashMap<TokenType, StmtParseFn>,
}

impl Parser {
    fn init_prefix_parse_fn_map(m: &mut HashMap<TokenType, PrefixParseFn>) {
        m.insert(TokenType::BoolTrue, parse_bool);
        m.insert(TokenType::BoolFalse, parse_bool);
        m.insert(TokenType::Integer64, parse_i64);
        m.insert(TokenType::UInteger64, parse_u64);
        m.insert(TokenType::String, parse_str);

        m.insert(TokenType::Ident, parse_ident);
        m.insert(TokenType::LBrace, parse_block_expr);
        m.insert(TokenType::If, parse_if);
        m.insert(TokenType::Func, parse_func);
        m.insert(TokenType::New, parse_build_struct);
    }

    fn init_infix_parse_fn_map(m: &mut HashMap<TokenType, InfixParseFn>) {
        m.insert(TokenType::Plus, parse_infix); // a + b
        m.insert(TokenType::Asterisk, parse_infix); // a * b
        m.insert(TokenType::Minus, parse_infix); // a - b
        m.insert(TokenType::Slash, parse_infix); // a / b

        m.insert(TokenType::Eq, parse_infix); // ==
        m.insert(TokenType::NotEq, parse_infix); // !=

        m.insert(TokenType::Lt, parse_infix);
        m.insert(TokenType::Gt, parse_infix);

        m.insert(TokenType::LParen, parse_call);
        m.insert(TokenType::Assign, parse_assign);
        m.insert(TokenType::Dot, parse_field_access);
    }

    fn init_statement_parse_fn_map(m: &mut HashMap<TokenType, StmtParseFn>) {
        m.insert(TokenType::Let, parse_let); // let a = 1
        m.insert(TokenType::While, parse_while); // while 1 {}
        
        m.insert(TokenType::Struct, parse_struct);
        m.insert(TokenType::Extern, parse_extern);
    }

    pub fn new(tokens: Vec<Token>) -> Self {
        let mut prefix_parse_fn_map = HashMap::new();
        let mut infix_parse_fn_map = HashMap::new();
        let mut statement_parse_fn_map = HashMap::new();

        Self::init_prefix_parse_fn_map(&mut prefix_parse_fn_map);
        Self::init_infix_parse_fn_map(&mut infix_parse_fn_map);
        Self::init_statement_parse_fn_map(&mut statement_parse_fn_map);

        let mut parser = Self {
            tokens,
            pos: 0,
            next_pos: 0,
            cur_token: Token::new(
                "\0".into(),
                TokenType::Nonsense,
                "uninit_parser".into(),
                0,
                0,
            ),
            peek_token: Token::new(
                "\0".into(),
                TokenType::Nonsense,
                "uninit_parser".into(),
                0,
                0,
            ),
            infix_parse_fn_map,
            prefix_parse_fn_map,
            statement_parse_fn_map,
        };

        parser.next_token();

        parser
    }

    pub fn next_token(&mut self) {
        if self.next_pos < self.tokens.len() {
            self.pos = self.next_pos;
            self.next_pos += 1;

            self.cur_token = self.tokens[self.pos].clone();

            self.peek_token = if self.next_pos < self.tokens.len() {
                self.tokens[self.next_pos].clone()
            } else {
                Token::eof(
                    self.cur_token.file.clone(),
                    self.cur_token.line,
                    self.cur_token.column,
                )
            };
        } else {
            self.cur_token = Token::eof(
                self.cur_token.file.clone(),
                self.cur_token.line,
                self.cur_token.column,
            );
            self.peek_token = Token::eof(
                self.peek_token.file.clone(),
                self.peek_token.line,
                self.peek_token.column,
            );
        }
    }

    pub fn parse_expression_statement(parser: &mut Self) -> ParseResult<Statement> {
        let expr = parser.parse_expression(Precedence::Lowest)?;

        if parser.peek_token_is(TokenType::Semicolon) {
            parser.next_token();
        }

        Ok(Statement::ExpressionStatement(expr))
    }

    pub fn parse_statement(&mut self) -> ParseResult<Statement> {
        let stmt_parse_fn = self
            .statement_parse_fn_map
            .get(&self.cur_token.token_type)
            .map_or(Self::parse_expression_statement as StmtParseFn, |it| *it);

        stmt_parse_fn(self)
    }

    pub fn parse_program(&mut self) -> ParseResult<Node> {
        let token = self.cur_token.clone();

        let mut statements = vec![];

        while !self.cur_token_is(TokenType::Eof) {
            let statement = self.parse_statement()?;

            statements.push(statement);

            self.next_token();
        }

        Ok(Node::Program { token, statements })
    }

    // pratt parser 核心函数
    pub fn parse_expression(&mut self, precedence: Precedence) -> ParseResult<Expression> {
        let prefix_parse_fn = self
            .prefix_parse_fn_map
            .get(&self.cur_token.token_type)
            .map_or_else(
                || {
                    Err(self.make_error(
                        ParserErrorKind::PrefixParseFnNotFound,
                        Some(
                            format!(
                                "no prefix parse function for {:#?} found",
                                self.cur_token.token_type
                            )
                            .into(),
                        ),
                    ))
                },
                |it| Ok(it),
            )?;

        let mut left = prefix_parse_fn(self)?;

        while (
            self.peek_token.token_type != TokenType::Semicolon ||
            self.peek_token.token_type != TokenType::Eof
        )
            && precedence < get_token_precedence(self.peek_token.token_type)
        {
            let infix_parse_fn = *self
                .infix_parse_fn_map
                .get(&self.peek_token.token_type)
                .map_or_else(
                    || {
                        Err(self.make_error(
                            ParserErrorKind::InfixParseFnNotFound,
                            Some(
                                format!(
                                    "no infix parse function for {:#?} found",
                                    self.peek_token.token_type
                                )
                                .into(),
                            ),
                        ))
                    },
                    |it| Ok(it),
                )?;

            self.next_token();
            left = infix_parse_fn(self, left)?;
        }

        Ok(left)
    }

    /// 使用本函数前请确保你已经到达到指定开始 Token  
    ///   
    /// 例如 在使用该函数解析左括号到右括号的表达式时 请先前进到左括号  
    ///   
    /// 函数执行完后不会自动离开指定 结束Token 若要离开 请自行调用 next_token 方法  
    pub fn parse_expression_list(&mut self, end: TokenType) -> Result<Vec<Box<Expression>>, ParserError> {
        // 检查下一个词法单元是否为对应结束的词法单元
        if self.peek_token_is(end) {
            self.next_token();
            return Ok(Vec::new()); // 如果是，直接退出，跳过表达式解析
        }

        self.next_token(); // 前进到表达式

        let mut expressions = vec![];
        expressions.push(Box::new(self.parse_expression(get_token_precedence(end))?));

        while self.peek_token_is(TokenType::Comma) {
            self.next_token(); // 离开表达式

            if self.peek_token_is(end) {
                // 尾逗号
                self.next_token();
                break;
            }

            self.next_token(); // 离开逗号

            let expression = self.parse_expression(get_token_precedence(end))?;
            expressions.push(Box::new(expression));
        }

        // 前进到结束的词法单元
        self.next_token();

        // WARNING: 若想在调用后跳过结束的词法单元，请自行在使用后处理

        Ok(expressions)
    }

    pub fn make_error(&self, kind: ParserErrorKind, message: Option<Rc<str>>) -> ParserError {
        ParserError {
            token: self.cur_token.clone(),
            kind,
            message,
        }
    }

    #[inline]
    pub fn cur_token_is(&self, token_ty: TokenType) -> bool {
        self.cur_token.token_type == token_ty
    }

    #[inline]
    pub fn peek_token_is(&self, token_ty: TokenType) -> bool {
        self.peek_token.token_type == token_ty
    }

    pub fn expect_cur(&self, expected: TokenType) -> ParseResult<()> {
        if self.cur_token.token_type == expected {
            return Ok(());
        }

        Err(self.make_error(
            ParserErrorKind::NotExpectedTokenType,
            Some(
                format!(
                    "expected cur token: {:#?}, got: {:#?}",
                    expected, self.cur_token.token_type
                )
                .into(),
            ),
        ))
    }

    pub fn expect_peek(&self, expected: TokenType) -> ParseResult<()> {
        if self.peek_token.token_type == expected {
            return Ok(());
        }

        Err(self.make_error(
            ParserErrorKind::NotExpectedTokenType,
            Some(
                format!(
                    "expected peek token: {:#?}, got: {:#?}",
                    expected, self.peek_token.token_type
                )
                .into(),
            ),
        ))
    }
}
