use crate::ast::{Expr, Resolver};
use crate::{Error, Token, TokenType, Value};
use rlox_macro::Expr;
use std::rc::Rc;

#[derive(Expr, Debug)]
pub struct Binary {
    #[wrapper]
    pub operator: Token,
    pub left: Rc<dyn Expr>,
    pub right: Rc<dyn Expr>,
}

impl Expr for Binary {
    fn eval(&self) -> Result<Box<Value>, Error> {
        let left = self.left.eval()?;
        let right = self.right.eval()?;
        match self.operator.token_type {
            TokenType::Minus => {
                if let Ok(v) = (*left) - (*right) {
                    Ok(v)
                } else {
                    Err(Error::new(
                        self.operator.line,
                        "-".to_string(),
                        "Binary operator - only works with numbers".to_string(),
                    ))
                }
            }
            TokenType::Plus => {
                if let Ok(v) = (*left) + (*right) {
                    Ok(v)
                } else {
                    Err(Error::new(
                        self.operator.line,
                        "+".to_string(),
                        "Binary operator + only works with numbers , strings or arrays".to_string(),
                    ))
                }
            }
            TokenType::Slash => {
                if let Ok(v) = (*left) / (*right) {
                    Ok(v)
                } else {
                    Err(Error::new(
                        self.operator.line,
                        "/".to_string(),
                        "Binary operator / only works with numbers".to_string(),
                    ))
                }
            }
            TokenType::Star => {
                if let Ok(v) = (*left) * (*right) {
                    Ok(v)
                } else {
                    Err(Error::new(
                        self.operator.line,
                        "*".to_string(),
                        "Binary operator * only works with numbers".to_string(),
                    ))
                }
            }
            TokenType::Greater => match left.cmp(&right) {
                Ok(Some(std::cmp::Ordering::Greater)) => Ok(Box::new(Value::Boolean(true))),
                Ok(_) => Ok(Box::new(Value::Boolean(false))),
                _ => Err(Error::new(
                    self.operator.line,
                    ">".to_string(),
                    "Binary operator > only works with numbers".to_string(),
                )),
            },
            TokenType::GreaterEqual => match left.cmp(&right) {
                Ok(Some(std::cmp::Ordering::Greater)) | Ok(Some(std::cmp::Ordering::Equal)) => {
                    Ok(Box::new(Value::Boolean(true)))
                }
                Ok(_) => Ok(Box::new(Value::Boolean(false))),
                _ => Err(Error::new(
                    self.operator.line,
                    ">=".to_string(),
                    "Binary operator >= only works with numbers".to_string(),
                )),
            },
            TokenType::Less => match left.cmp(&right) {
                Ok(Some(std::cmp::Ordering::Less)) => Ok(Box::new(Value::Boolean(true))),
                Ok(_) => Ok(Box::new(Value::Boolean(false))),
                _ => Err(Error::new(
                    self.operator.line,
                    "<".to_string(),
                    "Binary operator < only works with numbers".to_string(),
                )),
            },
            TokenType::LessEqual => match left.cmp(&right) {
                Ok(Some(std::cmp::Ordering::Less)) | Ok(Some(std::cmp::Ordering::Equal)) => {
                    Ok(Box::new(Value::Boolean(true)))
                }
                Ok(_) => Ok(Box::new(Value::Boolean(false))),
                _ => Err(Error::new(
                    self.operator.line,
                    "<=".to_string(),
                    "Binary operator <= only works with numbers".to_string(),
                )),
            },
            TokenType::EqualEqual => Ok(Box::new(Value::Boolean(left == right))),
            TokenType::BangEqual => Ok(Box::new(Value::Boolean(left != right))),
            _ => Err(Error::new(
                self.operator.line,
                self.operator.lexeme.clone(),
                "Unknown binary operator".to_string(),
            )),
        }
    }
}

impl Resolver for Binary {
    fn resolve(self: Rc<Self>, scopes: &mut crate::Scopes) -> Result<(), Error> {
        self.left.clone().resolve(scopes)?;
        self.right.clone().resolve(scopes)
    }
}
