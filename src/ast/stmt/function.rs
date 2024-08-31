use crate::ast::stmt::Block;
use crate::ast::{Resolver, Stmt};
use crate::{Error, FunctionType, Scopes, Token, Value};
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct Function {
    pub name: Token,
    pub params: Vec<Token>,
    pub body: Rc<Block>,
    pub is_initializer: bool,
}

impl std::fmt::Display for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut params = String::new();
        for param in &self.params {
            params.push_str(&format!("{} ", param.lexeme));
        }
        let mut body = String::new();
        for stmt in &self.body.statements {
            body.push_str(&format!("{}\n", stmt));
        }
        write!(f, "<fn>({} {})", self.name.lexeme, params)
    }
}

impl Stmt for Function {
    fn interpret(&self) -> Result<(), crate::error::Error> {
        unsafe {
            let function = Value::Fun(Rc::new(self.clone()), None);

            crate::ENVIRONMENT.define(self.name.lexeme.clone(), Box::new(function));
        }
        Ok(())
    }
}

impl Resolver for Function {
    fn resolve(self: Rc<Self>, scopes: &mut Scopes) -> Result<(), Error> {
        scopes.declare(self.name.clone())?;
        scopes.define(self.name.clone());
        resolve_function(self, scopes, FunctionType::Function)
    }
}

pub fn resolve_function(
    function: Rc<Function>,
    scopes: &mut Scopes,
    function_type: FunctionType,
) -> Result<(), Error> {
    let enclosing_function_type = scopes.get_current_function();
    scopes.set_current_function(Some(function_type));
    scopes.begin_scope();
    for param in &function.params {
        scopes.declare(param.clone())?;
        scopes.define(param.clone());
    }
    resolve_statements(function.body.statements.clone(), scopes)?;
    scopes.end_scope();
    scopes.set_current_function(enclosing_function_type);
    Ok(())
}

fn resolve_statements(statements: Vec<Rc<dyn Stmt>>, scopes: &mut Scopes) -> Result<(), Error> {
    for statement in statements {
        statement.resolve(scopes)?;
    }
    Ok(())
}

#[derive(Clone)]
pub struct Builtin {
    pub arity: usize,
    pub call: fn(Vec<Box<Value>>) -> Result<Box<Value>, Error>,
}

impl std::fmt::Debug for Builtin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<builtin fn>")
    }
}

impl Builtin {
    pub fn call(&self, args: Vec<Box<Value>>) -> Result<Box<Value>, Error> {
        (self.call)(args)
    }
}
