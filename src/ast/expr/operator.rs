use crate::ast::{Expr, Resolver};
use crate::{Error, Scopes, Token, Value};
use rlox_macro::Expr;
use std::rc::Rc;

#[derive(Expr, Debug)]
pub struct Operator {
    pub operator: Token,
}

impl Expr for Operator {
    fn eval(&self) -> Result<Box<Value>, Error> {
        panic!("Evaluate an operator is not supported")
    }
}

impl Resolver for Operator {
    fn resolve(self: Rc<Self>, _scopes: &mut Scopes) -> Result<(), Error> {
        Ok(())
    }
}
