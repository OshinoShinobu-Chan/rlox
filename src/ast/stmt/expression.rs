use crate::ast::expr::Expr;
use crate::ast::{Resolver, Stmt};
use crate::{Error, Scopes};
use rlox_macro::Expr;
use std::rc::Rc;

#[derive(Expr, Debug)]
pub struct Expression {
    pub expression: Rc<dyn Expr>,
}

impl Stmt for Expression {
    fn interpret(&self) -> Result<(), Error> {
        self.expression.eval()?;
        Ok(())
    }
}

impl Resolver for Expression {
    fn resolve(self: Rc<Self>, scopes: &mut Scopes) -> Result<(), Error> {
        self.expression.clone().resolve(scopes)
    }
}
