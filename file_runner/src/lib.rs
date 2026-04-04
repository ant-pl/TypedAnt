use std::{process::exit, sync::Arc};

use lexer::Lexer;
use name_resolver::Resolver;
use parser::{Parser, error::display_err};
use type_checker::{
    TypeChecker,
    module::TypedModule,
    ty_context::TypeContext,
    type_infer::{TypeInfer, infer_context::InferContext},
    typed_ast::{typed_node::TypedNode, typed_stmt::TypedStatement},
};

pub fn run_file(path: &str) {
    let filepath: Arc<str> = path.into();

    let mut ty_ctx = TypeContext::new();

    let code = std::fs::read_to_string(path).unwrap();

    let mut lexer = Lexer::new(code, filepath.clone());

    let tokens = lexer.get_tokens();

    if lexer.contains_error() {
        lexer.print_errors();
        exit(1);
    }

    let mut parser = Parser::new(tokens);

    let node = match parser.parse_program() {
        Ok(it) => {
            println!("ast: {it}");
            it
        }
        Err(err) => {
            display_err(&err);
            exit(1);
        }
    };

    let mut name_resolver = Resolver::new(0.into(), path);
    if let Err(it) = name_resolver.resolve(node.clone()) {
        println!("{it:#?}")
    };

    println!("name_resolver: {:#?}", name_resolver);

    let mut module = TypedModule::new(&mut ty_ctx);

    let mut checker = TypeChecker::new(&mut module);

    let typed_node;

    match checker.check_node(node) {
        Ok(it) => typed_node = it,
        Err(err) => {
            eprintln!("{err:#?}");
            exit(1)
        }
    }

    let constraints = checker.get_constraints().clone();

    let mut infer_ctx = InferContext::new(&mut module);

    let mut type_infer = TypeInfer::new(&mut infer_ctx);

    match type_infer.unify_all(constraints) {
        Ok(_) => (),
        Err(err) => {
            eprintln!("{err:#?}");
            exit(1);
        }
    }

    match type_infer.infer() {
        Ok(_) => (),
        Err(err) => {
            eprintln!("{err:#?}");
            exit(1);
        }
    }

    let TypedNode::Program { statements, .. } = typed_node;

    println!(
        "typed statements:\n{:#?}",
        statements
            .iter()
            .map(|it| module.get_stmt(*it).unwrap().clone())
            .collect::<Vec<TypedStatement>>()
    );

    println!("typed expressions:\n{:#?}", module.typed_exprs);

    println!("{:#?}", module.tcx_ref());
}
