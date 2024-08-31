use crate::ast::value::LoxCallable;
use crate::ast::{Expr, Resolver};
use crate::{Error, Token, Value};
use std::rc::Rc;

#[derive(Debug)]
pub struct Call {
    pub callee: Rc<dyn Expr>,
    pub arguments: Vec<Rc<dyn Expr>>,
    pub paren: Token,
}

impl std::fmt::Display for Call {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut arguments = String::new();
        for arg in &self.arguments {
            arguments.push_str(&format!("{}, ", arg));
        }
        write!(f, "<c>({} {})", self.callee, arguments)
    }
}

impl Expr for Call {
    fn eval(&self) -> Result<Box<Value>, Error> {
        let callee = self.callee.eval()?;

        let mut arguements = Vec::new();
        for arg in &self.arguments {
            arguements.push(arg.eval()?);
        }

        if !callee.is_callable() {
            Err(Error::new(
                self.paren.line,
                self.paren.lexeme.clone(),
                "Can only call functions and classes".to_string(),
            ))
        } else if arguements.len() != callee.arity() {
            Err(Error::new(
                self.paren.line,
                self.paren.lexeme.clone(),
                format!(
                    "Expected {} arguments but got {}.",
                    callee.arity(),
                    arguements.len()
                ),
            ))
        } else {
            callee.call(arguements)
        }
    }
}

impl Resolver for Call {
    fn resolve(self: Rc<Self>, scopes: &mut crate::Scopes) -> Result<(), Error> {
        self.callee.clone().resolve(scopes)?;
        for arg in &self.arguments {
            arg.clone().resolve(scopes)?;
        }
        Ok(())
    }
}
