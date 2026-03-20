mod parse_functions;
pub mod tests;
use std::{collections::HashMap, sync::Arc};

use ast::{expr::Expression, node::Node, stmt::Statement};
use token::{token::Token, token_type::TokenType};

use crate::{
    error::{ParserError, ParserErrorKind},
    parse_functions::{
        parse_assign::parse_assign,
        parse_block::parse_block_expr,
        parse_bool::parse_bool,
        parse_bool_and_or::{parse_bool_and, parse_bool_or},
        parse_build_struct::parse_build_struct,
        parse_call::parse_call,
        parse_cast::parse_cast,
        parse_const::parse_const,
        parse_extern::parse_extern,
        parse_field_access::parse_field_access,
        parse_func::parse_func,
        parse_grouped_expr::parse_grouped_expr,
        parse_ident::parse_ident,
        parse_if::parse_if,
        parse_impl::parse_impl,
        parse_infix::parse_infix,
        parse_let::parse_let,
        parse_num::{
            parse_i8, parse_i16, parse_i32, parse_i64, parse_isize, parse_u8, parse_u16, parse_u32,
            parse_u64, parse_usize,
        },
        parse_prefix::parse_prefix,
        parse_return::parse_return,
        parse_sizeof::parse_sizeof,
        parse_str::parse_str,
        parse_struct::parse_struct,
        parse_trait::parse_trait,
        parse_turbo_fish::parse_turbo_fish,
        parse_type_hint::parse_type_hint,
        parse_type_path::parse_type_path,
        parse_while::parse_while,
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
        m.insert(TokenType::Integer64, parse_i64);
        m.insert(TokenType::Integer32, parse_i32);
        m.insert(TokenType::Integer16, parse_i16);
        m.insert(TokenType::Integer8, parse_i8);
        m.insert(TokenType::UInteger64, parse_u64);
        m.insert(TokenType::UInteger32, parse_u32);
        m.insert(TokenType::UInteger16, parse_u16);
        m.insert(TokenType::UInteger8, parse_u8);
        m.insert(TokenType::USize, parse_usize);
        m.insert(TokenType::ISize, parse_isize);

        m.insert(TokenType::BoolTrue, parse_bool);
        m.insert(TokenType::BoolFalse, parse_bool);
        m.insert(TokenType::String, parse_str);

        m.insert(TokenType::Ident, parse_ident);
        m.insert(TokenType::LBrace, parse_block_expr);
        m.insert(TokenType::If, parse_if);
        m.insert(TokenType::Func, parse_func);
        m.insert(TokenType::New, parse_build_struct);
        m.insert(TokenType::Sizeof, parse_sizeof);

        m.insert(TokenType::Asterisk, parse_prefix);
        m.insert(TokenType::AddrOf, parse_prefix);

        m.insert(TokenType::LParen, parse_grouped_expr);
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

        m.insert(TokenType::BoolAnd, parse_bool_and);
        m.insert(TokenType::BoolOr, parse_bool_or);

        m.insert(TokenType::LParen, parse_call);
        m.insert(TokenType::Assign, parse_assign);
        m.insert(TokenType::Dot, parse_field_access);
        m.insert(TokenType::As, parse_cast);
        m.insert(TokenType::TwoColon, parse_turbo_fish);
    }

    fn init_statement_parse_fn_map(m: &mut HashMap<TokenType, StmtParseFn>) {
        m.insert(TokenType::Const, parse_const); // const a = 1
        m.insert(TokenType::Let, parse_let); // let a = 1
        m.insert(TokenType::While, parse_while); // while 1 {}

        m.insert(TokenType::Impl, parse_impl);
        m.insert(TokenType::Trait, parse_trait);
        m.insert(TokenType::Struct, parse_struct);
        m.insert(TokenType::Return, parse_return);
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
            .map_or(
                Err(self.make_error(
                    ParserErrorKind::PrefixParseFnNotFound,
                    Some(
                        format!(
                            "no prefix parse function for {:#?} found",
                            self.cur_token.token_type
                        )
                        .into(),
                    ),
                )),
                |it| Ok(it),
            )?;

        let mut left = prefix_parse_fn(self)?;

        while (self.peek_token.token_type != TokenType::Semicolon
            && self.peek_token.token_type != TokenType::Eof)
            && precedence < get_token_precedence(self.peek_token.token_type)
        {
            let infix_parse_fn = *self
                .infix_parse_fn_map
                .get(&self.peek_token.token_type)
                .map_or(
                    Err(self.make_error_with_token(
                        ParserErrorKind::InfixParseFnNotFound,
                        Some(
                            format!(
                                "no infix parse function for {:#?} found",
                                self.peek_token.token_type
                            )
                            .into(),
                        ),
                        self.peek_token.clone()
                    )),
                    |it| Ok(it),
                )?;

            self.next_token();
            left = infix_parse_fn(self, left)?;
        }

        Ok(left)
    }

    pub fn parse_type_expression(&mut self, precedence: Precedence) -> ParseResult<Expression> {
        // 保存状态
        let old_lt_handler = self.infix_parse_fn_map.remove(&TokenType::Lt);

        // 注入 TypePath 解析函数
        self.infix_parse_fn_map
            .insert(TokenType::Lt, parse_type_path);

        let r = self.parse_expression(precedence);

        // 恢复
        if let Some(old_lt) = old_lt_handler {
            self.infix_parse_fn_map.insert(TokenType::Lt, old_lt);
        }

        r
    }

    /// 使用本函数前请确保你已经到达到指定开始 Token  
    ///   
    /// 例如 在使用该函数解析左括号到右括号的表达式时 请先前进到左括号  
    ///   
    /// 函数执行完后不会自动离开指定 结束Token 若要离开 请自行调用 next_token 方法  
    pub fn parse_type_expression_list(
        &mut self,
        end: TokenType,
    ) -> Result<Vec<Box<Expression>>, ParserError> {
        // 保存当前 Lt 处理函数
        let old_lt_handler = self.infix_parse_fn_map.remove(&TokenType::Lt);
        // 保存当前 Colon 处理函数
        let old_colon_handler = self.infix_parse_fn_map.remove(&TokenType::Colon);

        // 注入 TypeHint 和 TypePath 解析函数
        self.infix_parse_fn_map
            .insert(TokenType::Colon, parse_type_hint);
        self.infix_parse_fn_map
            .insert(TokenType::Lt, parse_type_path);

        let exprs = self.parse_expression_list(end)?;

        // 恢复原来的处理函数
        if let Some(old_lt) = old_lt_handler {
            self.infix_parse_fn_map.insert(TokenType::Lt, old_lt);
        }

        if let Some(old_colon) = old_colon_handler {
            self.infix_parse_fn_map.insert(TokenType::Colon, old_colon);
        }

        Ok(exprs)
    }

    /// 使用本函数前请确保你已经到达到指定开始 Token  
    ///   
    /// 例如 在使用该函数解析左括号到右括号的表达式时 请先前进到左括号  
    ///   
    /// 函数执行完后不会自动离开指定 结束Token 若要离开 请自行调用 next_token 方法  
    pub fn parse_expression_list(
        &mut self,
        end: TokenType,
    ) -> Result<Vec<Box<Expression>>, ParserError> {
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

    pub fn make_error(&self, kind: ParserErrorKind, message: Option<Arc<str>>) -> ParserError {
        ParserError {
            token: self.cur_token.clone(),
            kind,
            message,
        }
    }

    pub fn make_error_with_token(
        &self,
        kind: ParserErrorKind,
        message: Option<Arc<str>>,
        token: Token,
    ) -> ParserError {
        ParserError {
            token,
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

    pub fn unexpect_token_err(&self, expected: TokenType, got: Token) -> ParseResult<()> {
        Err(ParserError {
            token: got,
            kind: ParserErrorKind::NotExpectedTokenType,
            message: Some(
                format!(
                    "expected token: {:#?}, got: {:#?}",
                    expected, self.peek_token.token_type
                )
                .into(),
            ),
        })
    }
}
