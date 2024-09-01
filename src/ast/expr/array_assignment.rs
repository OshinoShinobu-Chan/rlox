use crate::ast::{Expr, Resolver};
use crate::Token;
use std::rc::Rc;

#[derive(Debug)]
pub struct ArrayAssignment {
    pub name: Token,
    pub indeces: Vec<Rc<dyn Expr>>,
    pub value: Rc<dyn Expr>,
}

impl std::fmt::Display for ArrayAssignment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let indeces = self
            .indeces
            .iter()
            .map(|index| format!("[{}]", index))
            .collect::<Vec<String>>()
            .join("");
        write!(f, "<a>{}{} = {}", self.name, indeces, self.value)
    }
}

impl Resolver for ArrayAssignment {
    fn resolve(self: Rc<Self>, scopes: &mut crate::Scopes) -> Result<(), crate::error::Error> {
        self.value.clone().resolve(scopes)?;
        for index in self.indeces.clone() {
            index.clone().resolve(scopes)?;
        }
        scopes.resolve_local(Rc::as_ptr(&self) as *const dyn Expr, &self.name);
        Ok(())
    }
}

impl Expr for ArrayAssignment {
    fn eval(&self) -> Result<Box<crate::ast::Value>, crate::error::Error> {
        let value = self.value.eval()?;
        let indeces = self
            .indeces
            .iter()
            .map(|index| {
                let i = index.eval()?;
                if let crate::ast::Value::Number(i) = *i {
                    if i.fract() == 0f64 && i >= 0f64 && i.is_finite() {
                        Ok(i as usize)
                    } else {
                        Err(crate::error::Error::new(
                            self.name.line,
                            self.name.lexeme.clone(),
                            "Index must be non-negative integer".to_string(),
                        ))
                    }
                } else {
                    Err(crate::error::Error::new(
                        self.name.line,
                        self.name.lexeme.clone(),
                        "Index must be a number".to_string(),
                    ))
                }
            })
            .collect::<Result<Vec<usize>, crate::error::Error>>();
        let mut indeces_ = Vec::new();
        for index in indeces? {
            indeces_.push(index);
        }
        let distance = unsafe { crate::LOCALS.get(&(self as *const dyn Expr)) };
        if let Some(distance) = distance {
            unsafe {
                crate::Environment::assign_array_at(
                    crate::ENVIRONMENT.clone(),
                    *distance,
                    self.name.clone(),
                    indeces_,
                    value.clone(),
                )?;
            }
        } else {
            unsafe {
                crate::Environment::assign_array(
                    crate::Environment::get_global_mut(crate::ENVIRONMENT.clone()),
                    self.name.clone(),
                    indeces_,
                    value.clone(),
                )?;
            }
        }
        Ok(value)
    }
}
