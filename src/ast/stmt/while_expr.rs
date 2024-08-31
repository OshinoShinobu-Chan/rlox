use crate::ast::{Expr, Resolver, Stmt};
use crate::{Error, Scopes, Value};
use std::rc::Rc;

#[derive(Debug)]
pub struct WhileExpr {
    pub condition: Rc<dyn Expr>,
    pub body: Rc<dyn Stmt>,
}

impl std::fmt::Display for WhileExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<while>({} {})", self.condition, self.body)
    }
}

impl Stmt for WhileExpr {
    fn interpret(&self) -> Result<(), crate::error::Error> {
        while let Value::Boolean(true) = self.condition.eval()?.as_ref() {
            self.body.interpret()?;
        }
        Ok(())
    }
}

impl Resolver for WhileExpr {
    fn resolve(self: Rc<Self>, scopes: &mut Scopes) -> Result<(), Error> {
        self.condition.clone().resolve(scopes)?;
        self.body.clone().resolve(scopes)
    }
}
