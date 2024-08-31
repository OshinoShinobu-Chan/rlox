use crate::ast::{Expr, Resolver};
use crate::Token;
use std::rc::Rc;

#[derive(Debug)]
pub struct Set {
    pub object: Rc<dyn Expr>,
    pub name: Token,
    pub value: Rc<dyn Expr>,
}

impl std::fmt::Display for Set {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "<set>({} {} {})",
            self.object, self.name.lexeme, self.value
        )
    }
}

impl Resolver for Set {
    fn resolve(self: Rc<Self>, scopes: &mut crate::Scopes) -> Result<(), crate::error::Error> {
        self.object.clone().resolve(scopes)?;
        self.value.clone().resolve(scopes)
    }
}

impl Expr for Set {
    fn eval(&self) -> Result<Box<crate::ast::value::Value>, crate::error::Error> {
        let obj = self.object.eval()?;
        if let crate::ast::value::Value::Instance(instance) = *obj {
            let value = self.value.eval()?;
            instance.borrow_mut().set(&self.name.lexeme, value.clone());
            Ok(value)
        } else {
            return Err(crate::error::Error::new(
                self.name.line,
                self.name.lexeme.clone(),
                "Only instance have fields".to_string(),
            ));
        }
    }
}
