use crate::ast::{Expr, Resolver};
use std::rc::Rc;

#[derive(Debug)]
pub struct Array {
    pub values: Vec<Rc<dyn Expr>>,
}

impl std::fmt::Display for Array {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut values = String::new();
        values.push_str("<a>[");
        for value in &self.values {
            values.push_str(&format!("{}, ", value));
        }
        write!(f, "{}]", values)
    }
}

impl Resolver for Array {
    fn resolve(
        self: std::rc::Rc<Self>,
        scopes: &mut crate::Scopes,
    ) -> Result<(), crate::error::Error> {
        for value in self.values.clone() {
            value.resolve(scopes)?;
        }
        Ok(())
    }
}

impl Expr for Array {
    fn eval(&self) -> Result<Box<crate::ast::Value>, crate::error::Error> {
        let mut values = Vec::new();
        for value in &self.values {
            values.push(value.eval()?);
        }
        Ok(Box::new(crate::ast::Value::Array(values)))
    }
}
