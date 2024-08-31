use crate::ast::{Expr, Resolver, Stmt};
use crate::{Error, Scopes, Value};
use std::rc::Rc;

#[derive(Debug)]
pub struct IfExpr {
    pub condition: Rc<dyn Expr>,
    pub then_branch: Rc<dyn Stmt>,
    pub else_branch: Option<Rc<dyn Stmt>>,
}

impl std::fmt::Display for IfExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let else_branch = match &self.else_branch {
            Some(stmt) => format!(" else {}", stmt),
            None => "".to_string(),
        };
        write!(
            f,
            "<if>({} {}{})",
            self.condition, self.then_branch, else_branch
        )
    }
}

impl Stmt for IfExpr {
    fn interpret(&self) -> Result<(), crate::error::Error> {
        let condition = self.condition.eval()?;
        if let Value::Boolean(true) = condition.as_ref() {
            self.then_branch.interpret()
        } else if let Value::Boolean(false) = condition.as_ref() {
            if let Some(stmt) = &self.else_branch {
                stmt.interpret()
            } else {
                Ok(())
            }
        } else {
            Err(crate::error::Error::new(
                0,
                self.condition.to_string(),
                "Expect boolean condition".to_string(),
            ))
        }
    }
}

impl Resolver for IfExpr {
    fn resolve(self: Rc<Self>, scopes: &mut Scopes) -> Result<(), Error> {
        self.condition.clone().resolve(scopes)?;
        self.then_branch.clone().resolve(scopes)?;
        if let Some(stmt) = &self.else_branch {
            stmt.clone().resolve(scopes)?;
        }
        Ok(())
    }
}
