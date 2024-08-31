use crate::ast::{Expr, Resolver};
use crate::{Error, Token, TokenType, Value};
use rlox_macro::Expr;
use std::rc::Rc;

#[derive(Expr, Debug)]
pub struct Logic {
    pub operator: Token,
    pub left: Rc<dyn Expr>,
    pub right: Rc<dyn Expr>,
}

impl Expr for Logic {
    fn eval(&self) -> Result<Box<Value>, Error> {
        let left = self.left.eval()?;
        if self.operator.token_type == TokenType::Or {
            if *left == Value::Boolean(true) {
                return Ok(left);
            }
        } else {
            if *left == Value::Boolean(false) {
                return Ok(left);
            }
        }
        self.right.eval()
    }
}

impl Resolver for Logic {
    fn resolve(self: Rc<Self>, scopes: &mut crate::Scopes) -> Result<(), Error> {
        self.left.clone().resolve(scopes)?;
        self.right.clone().resolve(scopes)
    }
}
