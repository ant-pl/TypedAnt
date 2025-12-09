use std::{cell::RefCell, io::Write, rc::Rc};

use lexer::Lexer;
use parser::{Parser, error::display_err};
use type_checker::{TypeChecker, table::TypeTable};

pub fn repl() {
    let file: Rc<str> = "*repl".into();

    let table = Rc::new(RefCell::new(TypeTable::new()));
    table.borrow_mut().init_table();

    loop {
        print!(">>> ");

        std::io::stdout().flush().expect("flush failed");

        let mut code = String::new();

        std::io::stdin()
            .read_line(&mut code)
            .expect("read line failed");

        let mut lexer = Lexer::new(code, file.clone());

        let tokens = lexer.get_tokens();

        if lexer.contains_error() {
            lexer.print_errors();
            continue;
        }

        #[cfg(debug_assertions)]
        println!("tokens: {tokens:#?}");

        let mut parser = Parser::new(tokens);

        let node = match parser.parse_program() {
            Ok(it) => { println!("ast: {it}"); it },
            Err(err) => { display_err(&err); continue; },
        };

        let mut checker = TypeChecker::new(table.clone());

        match checker.check_node(node) {
            Ok(it) => println!("typed_ast: {it}"),
            Err(err) => println!("{err:#?}")
        }
    }
}
