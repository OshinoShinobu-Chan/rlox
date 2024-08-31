#![deny(unused_must_use)]
use ast::expr::Expr;
use ast::stmt::function::Builtin;
use ast::value::Value;
use clap::Parser;
use environment::Environment;
use error::Error;
use once_cell::sync::Lazy;
use scanner::Scanner;
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::rc::Rc;
use token::Token;
use token_type::TokenType;

mod ast;
mod environment;
mod error;
mod parser;
mod scanner;
mod token;
mod token_type;
extern crate rlox_macro;

pub static mut ENVIRONMENT: Lazy<Environment> = Lazy::new(|| Environment::new(None));
pub static mut LOCALS: Lazy<HashMap<*const dyn Expr, usize>> = Lazy::new(|| HashMap::new());
pub static mut CLOCK: Builtin = Builtin {
    arity: 0,
    call: |_: Vec<Box<Value>>| {
        let time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs_f64();
        Ok(Box::new(Value::Number(time)))
    },
};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Arg {
    pub script: Option<String>,
}

fn main() {
    let args = Arg::parse();
    if let Some(script) = args.script {
        run_file(script);
    } else {
        run_prompt();
    }
}

fn run_file(script: String) {
    let contents = fs::read_to_string(script).expect("Something went wrong reading the file");
    if let Err(e) = run(contents) {
        println!("{e}");
    }
}

fn run_prompt() {
    let handle_in = std::io::stdin();
    let mut handle_out = std::io::stdout();
    loop {
        print!(">>> ");
        handle_out.flush().unwrap();
        let mut input = String::new();
        handle_in.read_line(&mut input).unwrap();
        if input.is_empty() {
            break;
        }
        if let Err(e) = run(input) {
            println!("{e}");
        }
    }
}

fn run(source: String) -> Result<(), Error> {
    unsafe {
        ENVIRONMENT.define(
            "clock".to_string(),
            Box::new(Value::Builtin(Rc::new(CLOCK.clone()))),
        );
    }
    let mut scanner = Scanner::new(&source);
    let tokens = scanner.scan_tokens()?;

    let mut parse = parser::Parser::new(tokens);
    let ast = parse.parse()?;
    let mut scopes = Scopes::new();
    for stmt in ast.clone() {
        stmt.resolve(&mut scopes)?;
    }
    for stmt in ast {
        stmt.interpret()?;
        // println!("{}", stmt);
    }
    Ok(())
}

#[derive(Clone, Copy)]
pub enum FunctionType {
    Function,
    Method,
    Initializer,
}

#[derive(Clone, Copy)]
pub enum ClassType {
    Class,
}

pub struct Scopes(
    Vec<HashMap<String, bool>>,
    Option<FunctionType>,
    Option<ClassType>,
);

impl Scopes {
    pub fn new() -> Self {
        Self(Vec::new(), None, None)
    }

    pub fn begin_scope(&mut self) {
        self.0.push(HashMap::new());
    }

    pub fn end_scope(&mut self) {
        self.0.pop();
    }

    pub fn declare(&mut self, name: Token) -> Result<(), Error> {
        if !self.0.is_empty() {
            let scope = self.0.last_mut().unwrap();
            if scope.contains_key(&name.lexeme) {
                return Err(Error::new(
                    name.line,
                    name.lexeme.clone(),
                    "Variable with this name already declared in this scope".to_string(),
                ));
            }
            scope.insert(name.lexeme, false);
        }
        Ok(())
    }

    pub fn define(&mut self, name: Token) {
        if !self.0.is_empty() {
            let scope = self.0.last_mut().unwrap();
            scope.insert(name.lexeme, true);
        }
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn peek(&self) -> Option<&HashMap<String, bool>> {
        self.0.last()
    }

    pub fn peek_mut(&mut self) -> Option<&mut HashMap<String, bool>> {
        self.0.last_mut()
    }

    pub fn resolve_local(&self, expr: *const dyn Expr, name: &Token) {
        for i in (0..self.0.len()).rev() {
            if self.0[i].contains_key(&name.lexeme) {
                unsafe {
                    LOCALS.insert(expr, self.0.len() - 1 - i);
                }
                return;
            }
        }
    }

    pub fn get_current_function(&self) -> Option<FunctionType> {
        self.1
    }

    pub fn set_current_function(&mut self, function_type: Option<FunctionType>) {
        self.1 = function_type;
    }

    pub fn get_current_class(&self) -> Option<ClassType> {
        self.2
    }

    pub fn set_current_class(&mut self, class_type: Option<ClassType>) {
        self.2 = class_type;
    }
}
