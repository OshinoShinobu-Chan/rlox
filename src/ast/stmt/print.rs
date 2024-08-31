use crate::ast::{Expr, Resolver, Stmt};
use crate::{Error, Scopes};
use rlox_macro::Expr;
use std::rc::Rc;

#[derive(Expr, Debug)]
pub struct Print {
    pub expression: Rc<dyn Expr>,
}

impl Stmt for Print {
    fn interpret(&self) -> Result<(), crate::error::Error> {
        let value = self.expression.eval()?;
        println!("{}", value);
        Ok(())
    }
}

impl Resolver for Print {
    fn resolve(self: Rc<Self>, scopes: &mut Scopes) -> Result<(), Error> {
        self.expression.clone().resolve(scopes)
    }
}
