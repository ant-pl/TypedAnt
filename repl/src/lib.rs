use std::{io::Write, sync::Arc};

use lexer::Lexer;
use parser::{Parser, error::display_err};
use type_checker::{
    TypeChecker,
    module::TypedModule,
    ty_context::TypeContext,
    type_infer::{TypeInfer, infer_context::InferContext},
    typed_ast::{typed_node::TypedNode, typed_stmt::TypedStatement},
};

pub fn repl() {
    let file: Arc<str> = "*repl".into();

    let mut ty_ctx = TypeContext::new();

    loop {
        print!(">>> ");

        std::io::stdout().flush().expect("flush failed");

        let mut code = String::new();

        std::io::stdin()
            .read_line(&mut code)
            .expect("read line failed");

        let dbg = code.starts_with("~debug:");
        let show_tcx = code.starts_with("~tcx");

        if dbg {
            code.drain(0..=7);
        } else if show_tcx {
            code.drain(0..=4);
        }

        let mut lexer = Lexer::new(code, file.clone());

        let tokens = lexer.get_tokens();

        if lexer.contains_error() {
            lexer.print_errors();
            continue;
        }

        if dbg {
            println!("tokens: {tokens:#?}");
        }

        let mut parser = Parser::new(tokens);

        let node = match parser.parse_program() {
            Ok(it) => {
                if dbg {
                    println!("~debug ast: {it:#?}")
                }
                println!("ast: {it}");
                it
            }
            Err(err) => {
                display_err(&err);
                continue;
            }
        };

        let mut module = TypedModule::new(&mut ty_ctx);

        let mut checker = TypeChecker::new(&mut module);

        // 不知道为什么明明有情况能使用到 rust analyzer 死活分析不出来
        #[allow(unused_assignments)]
        let mut typed_node = None;

        match checker.check_node(node) {
            Ok(it) => typed_node = Some(it),
            Err(err) => {
                eprintln!("{err:#?}");
                continue;
            }
        }

        let constraints = checker.get_constraints().clone();

        let mut infer_ctx = InferContext::new(&mut module);

        let mut type_infer = TypeInfer::new(&mut infer_ctx);

        match type_infer.unify_all(constraints) {
            Ok(_) => (),
            Err(err) => {
                eprintln!("{err:#?}");
                continue;
            }
        }

        match type_infer.infer() {
            Ok(_) => (),
            Err(err) => {
                eprintln!("{err:#?}");
                continue;
            }
        }

        if let Some(TypedNode::Program { statements, .. }) = typed_node {
            let module_cloned = module.cloned();
            println!(
                "typed statements:\n{:#?}",
                statements
                    .iter()
                    .map(|it| module_cloned.get_stmt(*it).unwrap().clone())
                    .collect::<Vec<TypedStatement>>()
            )
        } else {
            println!("no typed ast here.")
        }

        if show_tcx {
            println!("~tcx: {ty_ctx:#?}")
        }
    }
}
