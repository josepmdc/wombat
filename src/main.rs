mod codegen;
mod error;
mod lex;
mod parser;

use std::{
    env, fs,
    io::{self, Write},
};

use error::CompilerError;
use inkwell::{context::Context, module::Module, OptimizationLevel};
use lex::Scanner;
use parser::stmt::Function;

use crate::{
    codegen::Compiler,
    parser::{stmt::{Stmt, Prototype}, Parser},
};

#[macro_use]
extern crate log;

pub const PROGRAM_STARTING_POINT: &str = "main";

fn main() {
    env_logger::init();

    let args: Vec<String> = env::args().collect();
    if args.len() > 2 {
        println!("Usage: craft [script]");
    } else if args.len() == 2 {
        run_file(&args[1]);
    } else {
        run_repl();
    }
}

fn run_file(path: &String) {
    let source = fs::read_to_string(path).expect("Could not read file");
    if let Err(err) = run(source) {
        println!("{}", err);
    }
}

fn run_repl() {
    loop {
        let mut input = String::new();

        print_prompt();

        io::stdin()
            .read_line(&mut input)
            .expect("Could not read line");

        if input.starts_with("exit") || input.starts_with("quit") {
            break;
        } else if input.chars().all(char::is_whitespace) {
            continue;
        }

        if let Err(err) = run(input) {
            error!("{}", err);
        }
    }
}

fn run(source: String) -> Result<(), CompilerError> {
    trace!("\n{}", source);
    let mut scanner = Scanner::new(source);
    let tokens = scanner.scan_tokens();
    let mut parser = Parser::new(tokens.to_vec());
    let func = parser.parse()?;

    // TODO This is only for testing, it will be properly implemented later
    let func = match &func[0] {
        Stmt::Function(func) => func.clone(),
        Stmt::Expr(expr) => Function {
            prototype: Prototype {
                name: PROGRAM_STARTING_POINT.to_string(),
                args: vec![],
            },
            body: vec![Stmt::Expr(expr.clone())],
            is_anon: true,
        },
        _ => panic!("Unexpected statement, {:#?}", func[0]),
    };

    compile(func)?;
    Ok(())
}

fn compile(func: Function) -> Result<(), CompilerError> {
    let context = Context::create();
    let module = context.create_module("repl");
    let builder = context.create_builder();

    match Compiler::compile(&context, &builder, &module, &func) {
        Ok(function) => {
            println!("--------------------------------");
            function.print_to_stderr();
            println!("--------------------------------");
        }
        Err(err) => return Err(CompilerError::CodegenError(err)),
    }

    run_jit(&module);
    Ok(())
}

fn run_jit(module: &Module) {
    let ee = module
        .create_jit_execution_engine(OptimizationLevel::None)
        .unwrap();

    let maybe_fn =
        unsafe { ee.get_function::<unsafe extern "C" fn() -> f64>(PROGRAM_STARTING_POINT) };

    let compiled_fn = match maybe_fn {
        Ok(f) => f,
        Err(err) => {
            println!("!> Error during execution: {:?}", err);
            return;
        }
    };

    unsafe {
        println!("=> {}", compiled_fn.call());
    }
}

fn print_prompt() {
    print!(">> ");
    io::stdout().flush().unwrap();
}
