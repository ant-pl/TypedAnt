pub mod error;
pub mod test;

use std::rc::Rc;

use unicode_properties::UnicodeEmoji;

use token::token::Token;
use token::token_type::{TOKEN_TYPE_MAP, TokenNumType, TokenType};
use token::{NEW_LINE, NULL_CHAR};

use crate::error::{LexerError, LexerErrorKind};

pub struct Lexer {
    cur_char: char,
    pos: usize,
    next_pos: usize,
    line: usize,
    column: usize,
    file: Rc<str>,
    code_vec: Rc<[char]>,
    errors: Vec<LexerError>,
}

impl Lexer {
    pub fn new(code: String, file: Rc<str>) -> Lexer {
        let mut lexer = Lexer {
            file,
            errors: vec![],
            cur_char: NULL_CHAR,
            pos: 0,
            next_pos: 0,
            line: 1,
            column: 1,
            code_vec: code.chars().collect::<Vec<_>>().into(),
        };

        lexer.read_char(); // 初始化
        lexer
    }

    fn get_ident_token_type(&self, ident: &str) -> TokenType {
        *TOKEN_TYPE_MAP
            .get(&ident.to_uppercase())
            .unwrap_or(&TokenType::Ident)
    }

    fn peek_char(&self) -> char {
        match self.code_vec.get(self.next_pos) {
            Some(it) => it.clone(),
            None => NULL_CHAR,
        }
    }

    fn get_char(&self, pos: usize) -> char {
        match self.code_vec.get(pos) {
            Some(it) => it.clone(),
            None => NULL_CHAR,
        }
    }

    fn is_valid_char(&self, c: char) -> bool {
        (c.is_alphabetic() || c == '_' || c.is_emoji_char()) && c != '#' && c != '*'
    }

    fn read_char(&mut self) -> char {
        if self.next_pos < self.code_vec.len() {
            self.cur_char = self.code_vec[self.next_pos]
        } else {
            self.cur_char = NULL_CHAR;
        }

        if self.cur_char == NEW_LINE {
            self.line += 1;
            self.column = 0;
        }

        self.column += 1;

        self.pos = self.next_pos;
        self.next_pos += 1;

        self.cur_char
    }

    fn skip_whitespace(&mut self) {
        while self.cur_char == ' '
            || self.cur_char == '\t'
            || self.cur_char == '\n'
            || self.cur_char == '\r'
        {
            self.read_char();
        }
    }

    fn read_ident(&mut self) -> String {
        let start = self.pos;

        while self.is_valid_char(self.cur_char) && !self.eof() {
            self.read_char();
        }

        self.code_vec[start..self.pos]
            .iter()
            .map(|ch| ch.to_string())
            .collect::<Vec<String>>()
            .concat()
    }

    fn read_number(&mut self) -> TokenNumType {
        let start = self.pos;

        while self.cur_char.is_ascii_digit() {
            self.read_char();
        }

        let code = self.code_vec[start..self.pos]
            .iter()
            .map(|ch| ch.to_string())
            .collect::<Vec<String>>()
            .concat();

        if self.peek_char() == '6' && self.get_char(self.next_pos + 1) == '4' {
            self.read_char();
            self.read_char();
            self.read_char();

            TokenNumType::Int64(code)
        } else {
            TokenNumType::Big(code)
        }
    }

    fn push_err(&mut self, kind: LexerErrorKind, message: Option<Rc<str>>) {
        self.errors.push(LexerError {
            file: self.file.clone(),
            line: self.line,
            column: self.column,
            kind,
            message,
        });
    }

    fn push_error(
        &mut self,
        kind: LexerErrorKind,
        message: Option<Rc<str>>,
        line: usize,
        column: usize,
    ) {
        self.errors.push(LexerError {
            file: self.file.clone(),
            line,
            column,
            kind,
            message,
        });
    }

    fn read_string(&mut self) -> String {
        let start_line = self.line;
        let start_column = self.column;

        let mut result = String::new();

        // 跳过起始双引号
        self.read_char();

        loop {
            if self.cur_char == '"' {
                self.read_char(); // 跳过结束双引号
                return result;
            }

            if self.eof() {
                self.push_error(
                    LexerErrorKind::UnClosedString,
                    None,
                    start_line,
                    start_column - 1,
                );
                break;
            }

            if self.cur_char == '\\' {
                // 处理转义字符
                self.read_char();
                match self.cur_char {
                    'n' => result.push('\n'),
                    't' => result.push('\t'),
                    'r' => result.push('\r'),
                    '\\' => result.push('\\'),
                    '"' => result.push('"'),
                    '0' => result.push('\0'),
                    'b' => result.push('\u{0008}'), // backspace
                    'f' => result.push('\u{000C}'), // form feed
                    'u' => {
                        // Unicode转义: \u{XXXX}
                        if self.peek_char() != '{' {
                            self.push_err(
                                LexerErrorKind::InvalidUnicodeEscapeSequence,
                                Some("expected '{{' after '\\u'.".into()),
                            );
                            return "".to_string();
                        }

                        self.read_char(); // 跳过 {
                        let mut hex_digits = String::new();

                        loop {
                            self.read_char();

                            if self.cur_char == '}' {
                                break;
                            }

                            if self.eof() || !self.cur_char.is_ascii_hexdigit() {
                                self.push_err(LexerErrorKind::InvalidUnicodeEscapeSequence, None);
                                return "".to_string();
                            }

                            hex_digits.push(self.cur_char);
                        }

                        match u32::from_str_radix(&hex_digits, 16) {
                            Ok(code_point) => match char::from_u32(code_point) {
                                Some(ch) => result.push(ch),
                                None => {
                                    self.push_err(
                                        LexerErrorKind::InvalidUnicodeEscapeSequence,
                                        Some("invalid unicode code point.".into()),
                                    );
                                    return "".to_string();
                                }
                            },
                            Err(_) => {
                                self.push_err(
                                    LexerErrorKind::InvalidUnicodeEscapeSequence,
                                    Some("invalid hex digits in unicode escape.".into()),
                                );

                                return "".to_string();
                            }
                        }
                    }
                    _ => {
                        // 未知的转义序列，原样输出
                        result.push('\\');
                        result.push(self.cur_char);
                    }
                }
            } else {
                // 普通字符，直接添加
                result.push(self.cur_char);
            }

            self.read_char();
        }

        "".to_string()
    }

    fn read_comment(&mut self) -> String {
        let start = self.pos + 2; // 跳过 "//"

        loop {
            self.read_char();

            if self.cur_char == NEW_LINE || self.eof() {
                let s = self.code_vec[start..self.pos]
                    .iter()
                    .map(|ch| ch.to_string())
                    .collect::<Vec<String>>()
                    .join("");

                return s;
            }
        }
    }

    fn next_token(&mut self) -> Token {
        self.skip_whitespace();

        let mut token = Token::new(
            self.cur_char.to_string().into(),
            TokenType::Illegal,
            self.file.clone(),
            self.line,
            self.column - 1,
        );

        if TOKEN_TYPE_MAP.contains_key(&self.cur_char.to_string()) {
            token.token_type = TOKEN_TYPE_MAP[&self.cur_char.to_string()].clone();
            token.value = self.cur_char.to_string().into();
        }

        match self.cur_char {
            '=' => {
                let peek_char = self.peek_char();

                if peek_char == '=' {
                    token.token_type = TokenType::Eq;
                    token.value = format!("{}{}", self.cur_char, peek_char).into();

                    self.read_char();
                }
            }

            '!' => {
                let peek_char = self.peek_char();
                if peek_char == '=' {
                    token.token_type = TokenType::Eq;
                    token.value = format!("{}{}", self.cur_char, peek_char).into();

                    self.read_char();
                }
            }

            ':' => {
                let peek_char = self.peek_char();
                if peek_char == ':' {
                    token.token_type = TokenType::GetClassMember;
                    token.value = format!("{}{}", self.cur_char, peek_char).into();

                    self.read_char();
                }
            }

            '"' => {
                if self.cur_char == '"' {
                    let s = self.read_string();
                    token.value = s.into();
                    token.token_type = TokenType::String;

                    return token;
                }
            }

            '/' => {
                let peek_char = self.peek_char();
                if peek_char == '/' {
                    // 读取注释内容并跳过
                    self.read_comment();
                    // 递归调用 next_token 跳过注释，获取下一个有效token
                    return self.next_token();
                }
            }

            _ => {
                if self.is_valid_char(self.cur_char) && !self.cur_char.is_ascii_digit() {
                    let ident = self.read_ident();
                    token.token_type = self.get_ident_token_type(&ident);
                    token.value = ident.into();

                    return token;
                } else if self.cur_char.is_ascii_digit() {
                    let num = self.read_number();
                    match num {
                        TokenNumType::Big(num) => {
                            token.token_type = TokenType::IntegerBig;
                            token.value = num.into();
                        }

                        TokenNumType::Int64(num) => {
                            token.token_type = TokenType::Integer64;
                            token.value = num.into();
                        }

                        TokenNumType::Int32(num) => {
                            token.token_type = TokenType::Integer32;
                            token.value = num.into();
                        }

                        TokenNumType::Int16(num) => {
                            token.token_type = TokenType::Integer16;
                            token.value = num.into();
                        }

                        TokenNumType::Int8(num) => {
                            token.token_type = TokenType::Integer8;
                            token.value = num.into();
                        }

                        TokenNumType::UInt64(num) => {
                            token.token_type = TokenType::UInteger64;
                            token.value = num.into();
                        }

                        TokenNumType::UInt32(num) => {
                            token.token_type = TokenType::UInteger32;
                            token.value = num.into();
                        }

                        TokenNumType::UInt16(num) => {
                            token.token_type = TokenType::UInteger16;
                            token.value = num.into();
                        }

                        TokenNumType::UInt8(num) => {
                            token.token_type = TokenType::UInteger8;
                            token.value = num.into();
                        }
                    }

                    return token;
                }
            }
        }

        self.read_char();

        token
    }

    fn eof(&self) -> bool {
        self.cur_char == NULL_CHAR
    }

    pub fn get_tokens(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();

        while !self.eof() {
            tokens.push(self.next_token());
        }

        tokens
    }

    pub fn contains_error(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn errors(&self) -> &Vec<LexerError> {
        &self.errors
    }

    pub fn print_errors(&self) {
        use colored::Colorize;

        println!(
            "{}",
            format!(
                "lexer {}:",
                if self.errors.len() > 1 {
                    "errors"
                } else {
                    "error"
                }
            )
            .purple()
        );

        for error in self.errors.clone() {
            println!("{}", "--> error here!".purple());

            let line = &self
                .code_vec
                .split(|it| it == &NEW_LINE)
                .map(|it| it.iter().map(|it| it.to_string()).collect::<Vec<_>>())
                .collect::<Vec<_>>()[error.line - 1];

            let last_char = line[error.column - 1].red();

            print!("    {}", line[..error.column - 1].join(""));
            print!("{}", last_char);
            println!("{}", line[error.column..].join(""));

            print!("{}", " ".repeat(3 + error.column));
            println!("{}", "^".purple());

            println!("    {}", error.to_string().purple());
        }
    }
}
