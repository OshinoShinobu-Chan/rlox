use crate::ast::{Expr, Resolver};
use crate::{Error, Token, Value};
use rlox_macro::Expr;
use std::rc::Rc;

#[derive(Expr, Debug)]
pub struct Literal {
    pub value: Token,
}

impl Expr for Literal {
    fn eval(&self) -> Result<Box<Value>, Error> {
        let value = self.value.token_type.value();
        match value {
            Some(v) => Ok(v),
            None => Err(Error::new(
                self.value.line,
                self.value.lexeme.clone(),
                "This litral can't be evaluate".to_string(),
            )),
        }
    }
}

impl Resolver for Literal {
    fn resolve(self: Rc<Self>, _scopes: &mut crate::Scopes) -> Result<(), Error> {
        Ok(())
    }
}
