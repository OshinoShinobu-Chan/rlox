use crate::ast::{Expr, Resolver};
use crate::{Error, Value};
use rlox_macro::Expr;
use std::rc::Rc;

#[derive(Expr, Debug)]
pub struct Grouping {
    #[wrapper]
    pub expression: Rc<dyn Expr>,
}

impl Expr for Grouping {
    fn eval(&self) -> Result<Box<Value>, Error> {
        self.expression.eval()
    }
}

impl Resolver for Grouping {
    fn resolve(self: Rc<Self>, scopes: &mut crate::Scopes) -> Result<(), Error> {
        self.expression.clone().resolve(scopes)
    }
}
