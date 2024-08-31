use crate::ast::{Expr, Resolver};
use crate::{Error, Token, TokenType, Value};
use rlox_macro::Expr;
use std::rc::Rc;

#[derive(Expr, Debug)]
pub struct Unary {
    #[wrapper]
    pub operator: Token,
    pub right: Rc<dyn Expr>,
}

impl Expr for Unary {
    fn eval(&self) -> Result<Box<Value>, Error> {
        let right = self.right.eval()?;
        match self.operator.token_type {
            TokenType::Minus => {
                if let Ok(v) = -(*right) {
                    Ok(v)
                } else {
                    Err(Error::new(
                        self.operator.line,
                        "-".to_string(),
                        "Unary operator - only works with numbers".to_string(),
                    ))
                }
            }
            TokenType::Bang => {
                if let Ok(v) = !(*right) {
                    Ok(v)
                } else {
                    Err(Error::new(
                        self.operator.line,
                        "!".to_string(),
                        "Unary operator ! only works with boolean".to_string(),
                    ))
                }
            }
            _ => Err(Error::new(
                self.operator.line,
                self.operator.lexeme.clone(),
                "Unknown unary operator".to_string(),
            )),
        }
    }
}

impl Resolver for Unary {
    fn resolve(self: Rc<Self>, scopes: &mut crate::Scopes) -> Result<(), Error> {
        self.right.clone().resolve(scopes)
    }
}
