use crate::ast::{Expr, Resolver};
use crate::{Error, Token, Value};
use std::rc::Rc;

#[derive(Debug)]
/// Variable expression
pub struct VarExpr {
    pub name: Token,
}

impl std::fmt::Display for VarExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name.lexeme)
    }
}

impl Expr for VarExpr {
    fn eval(&self) -> Result<Box<Value>, Error> {
        unsafe { crate::ENVIRONMENT.look_up_variable(self.name.clone(), self) }
    }
}

impl Resolver for VarExpr {
    fn resolve(self: Rc<Self>, scopes: &mut crate::Scopes) -> Result<(), Error> {
        if !scopes.is_empty() && scopes.peek().unwrap().get(&self.name.lexeme) == Some(&false) {
            Err(Error::new(
                self.name.line,
                self.name.lexeme.clone(),
                "Can't read local variable in its own initializer".to_string(),
            ))
        } else {
            scopes.resolve_local(Rc::as_ptr(&self) as *const dyn Expr, &self.name);
            Ok(())
        }
    }
}
