use crate::ast::{Expr, Resolver};
use crate::Token;
use std::rc::Rc;

#[derive(Debug)]
pub struct Get {
    pub object: Rc<dyn Expr>,
    pub name: Token,
}

impl Resolver for Get {
    fn resolve(self: Rc<Self>, scopes: &mut crate::Scopes) -> Result<(), crate::error::Error> {
        self.object.clone().resolve(scopes)
    }
}

impl std::fmt::Display for Get {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<get>({} {})", self.object, self.name.lexeme)
    }
}

impl Expr for Get {
    fn eval(&self) -> Result<Box<crate::ast::value::Value>, crate::error::Error> {
        let obj = self.object.eval()?;
        match obj.as_ref() {
            crate::ast::value::Value::Instance(instance) => {
                let instance = instance.clone();
                let value = instance.borrow().get(&self.name.lexeme, instance.clone());
                match value {
                    Some(value) => Ok(value),
                    None => Err(crate::error::Error::new(
                        self.name.line,
                        format!("'{}'", self.name.lexeme),
                        "Undefined property".to_string(),
                    )),
                }
            }
            _ => Err(crate::error::Error::new(
                self.name.line,
                self.name.lexeme.clone(),
                "Only instance have properties".to_string(),
            )),
        }
    }
}
