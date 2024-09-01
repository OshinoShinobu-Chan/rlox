use crate::ast::{Expr, Resolver, Stmt};
use crate::{Error, Scopes, Token, Value};
use std::rc::Rc;

#[derive(Debug)]
/// Statement for variable declaration
pub struct VarDecl {
    pub name: Token,
    pub initializer: Option<Rc<dyn Expr>>,
}

impl std::fmt::Display for VarDecl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let initializer = match &self.initializer {
            Some(expr) => format!(" = {}", expr),
            None => "Nil".to_string(),
        };
        write!(f, "<v>({} {initializer})", self.name)
    }
}

impl Stmt for VarDecl {
    fn interpret(&self) -> Result<(), crate::error::Error> {
        let value = match &self.initializer {
            Some(expr) => expr.eval()?,
            None => Box::new(Value::Nil),
        };
        unsafe {
            crate::ENVIRONMENT
                .borrow_mut()
                .define(self.name.lexeme.clone(), value);
        }
        Ok(())
    }
}

impl Resolver for VarDecl {
    fn resolve(self: Rc<Self>, scopes: &mut Scopes) -> Result<(), Error> {
        scopes.declare(self.name.clone())?;
        if let Some(expr) = &self.initializer {
            expr.clone().resolve(scopes)?;
        }
        scopes.define(self.name.clone());
        Ok(())
    }
}
