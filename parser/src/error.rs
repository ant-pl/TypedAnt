use std::{fmt::Display, rc::Rc};

use token::token::Token;

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Ord, Eq)]
pub enum ParseIntErrorKind {
    Empty,
    InvalidDigit,
    PosOverflow,
    NegOverflow,
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Ord, Eq)]
pub enum ParserErrorKind {
    ParseIntError(ParseIntErrorKind),
    MissingExpression,
    NotExpectedTokenType,
    PrefixParseFnNotFound,
    InfixParseFnNotFound,
    StmtParseFnNotFound,
    ExpectedType,
    Other
}

impl Display for ParserErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Other => "unknown parser error kind",
            Self::MissingExpression => "missing expression",
            Self::NotExpectedTokenType => "not expected token type",
            Self::PrefixParseFnNotFound => "prefix parse fn not found",
            Self::InfixParseFnNotFound => "infix parse fn not found",
            Self::StmtParseFnNotFound => "statement parse fn not found",
            Self::ExpectedType => "expected type",
            Self::ParseIntError(err) => match err {
                ParseIntErrorKind::Empty => "cannot parse integer from empty string",
                ParseIntErrorKind::InvalidDigit => "invalid digit found in string",
                ParseIntErrorKind::NegOverflow => "number too small to fit in target type",
                ParseIntErrorKind::PosOverflow => "number too large to fit in target type",
            },
        };

        write!(f, "{s}")
    }
}

#[derive(Clone, Debug)]
pub struct ParserError {
    pub token: Token,
    pub kind: ParserErrorKind,
    pub message: Option<Rc<str>>,
}

impl Display for ParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use colored::Colorize;

        if let Some(msg) = &self.message {
            write!(
                f,
                "{} '{}', {} {}, {} {}\n{}. {}",
                "at file".purple().bold(),
                self.token.file.purple(),
                "line".purple().bold(),
                self.token.line.to_string().purple(),
                "column".purple().bold(),
                self.token.column.to_string().purple(),
                self.kind.to_string().purple(),
                msg.purple()
            )
        } else {
            write!(
                f,
                "{} '{}', {} {}, {} {}\n{}.",
                "at file".purple().bold(),
                self.token.file.purple(),
                "line".purple().bold(),
                self.token.line.to_string().purple(),
                "column".purple().bold(),
                self.token.column.to_string().purple(),
                self.kind.to_string().purple(),
            )
        }
    }
}

fn try_get_content(file: &str) -> Option<String> {
    if file == "*repl" {
        return None;
    }

    std::fs::read_to_string(file).map_or_else(|_| None, |it| Some(it))
}

fn display_source_code(err: &ParserError) {
    use colored::Colorize;

    let code = try_get_content(&err.token.file);

    if code.is_none() {
        println!("    {}", err.token.value.red());

        println!(
            "    {}",
            "^".repeat(err.token.value.chars().count()).purple()
        );

        return;
    }

    let code = unsafe { code.unwrap_unchecked() };

    let line = code
        .split("\n")
        .nth(err.token.line - 1)
        .unwrap()
        .chars()
        .map(|it| it.to_string())
        .collect::<Vec<String>>();

    let err_token = &err.token;

    let last_char = line[err_token.column - 1].red();

    print!("    {}", line[..err_token.column - 1].join(""));
    print!("{}", last_char);
    println!("{}", line[err_token.column..].join(""));

    print!("{}", " ".repeat(3 + err_token.column));
    println!("{}", "^".purple());
}

pub fn display_err(err: &ParserError) {
    use colored::Colorize;

    println!("{err} \n{}: {{", "code".purple());

    display_source_code(err);

    println!("}}")
}
