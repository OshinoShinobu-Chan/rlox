//! statement in AST
use crate::environment::Environment;
use crate::error::Error;
use crate::syntax;
use crate::syntax::Value;
use crate::token::Token;
use crate::FunctionType;
use crate::Scopes;
use once_cell::sync::Lazy;
use rlox_macro::Expr;
use std::cell::RefCell;
use std::rc::Rc;

pub trait Resolver {
    fn resolve(self: Rc<Self>, scopes: &mut Scopes) -> Result<(), Error>;
}

pub trait Stmt: std::fmt::Display + std::fmt::Debug + Resolver {
    fn interpret(&self) -> Result<(), crate::error::Error>;
    fn type_name(&self) -> String {
        std::any::type_name::<Self>().to_string()
    }
}

#[derive(Expr, Debug)]
pub struct Expression {
    pub expression: Rc<dyn syntax::Expr>,
}

impl Stmt for Expression {
    fn interpret(&self) -> Result<(), crate::error::Error> {
        self.expression.eval()?;
        Ok(())
    }
}

impl Resolver for Expression {
    fn resolve(self: Rc<Self>, scopes: &mut Scopes) -> Result<(), Error> {
        self.expression.clone().resolve(scopes)
    }
}

#[derive(Expr, Debug)]
pub struct Print {
    pub expression: Rc<dyn syntax::Expr>,
}

impl Stmt for Print {
    fn interpret(&self) -> Result<(), crate::error::Error> {
        let value = self.expression.eval()?;
        println!("{}", value);
        Ok(())
    }
}

impl Resolver for Print {
    fn resolve(self: Rc<Self>, scopes: &mut Scopes) -> Result<(), Error> {
        self.expression.clone().resolve(scopes)
    }
}

#[derive(Debug)]
/// Statement for variable declaration
pub struct Variable {
    pub name: Token,
    pub initializer: Option<Rc<dyn syntax::Expr>>,
}

impl std::fmt::Display for Variable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let initializer = match &self.initializer {
            Some(expr) => format!(" = {}", expr),
            None => "Nil".to_string(),
        };
        write!(f, "<v>({} {initializer})", self.name)
    }
}

impl Stmt for Variable {
    fn interpret(&self) -> Result<(), crate::error::Error> {
        let value = match &self.initializer {
            Some(expr) => expr.eval()?,
            None => Box::new(crate::syntax::Value::Nil),
        };
        unsafe {
            crate::ENVIRONMENT.define(self.name.lexeme.clone(), value);
        }
        Ok(())
    }
}

impl Resolver for Variable {
    fn resolve(self: Rc<Self>, scopes: &mut Scopes) -> Result<(), Error> {
        scopes.declare(self.name.clone())?;
        if let Some(expr) = &self.initializer {
            expr.clone().resolve(scopes)?;
        }
        scopes.define(self.name.clone());
        Ok(())
    }
}

#[derive(Debug)]
pub struct Block {
    pub statements: Vec<Rc<dyn Stmt>>,
}

impl std::fmt::Display for Block {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut statements = String::new();
        statements.push_str("<b>{");
        for stmt in &self.statements {
            statements.push_str(&format!("{}\n", stmt));
        }
        write!(f, "{}}}", statements)
    }
}

impl Stmt for Block {
    fn interpret(&self) -> Result<(), crate::error::Error> {
        unsafe {
            let previous = Rc::new(RefCell::new(crate::ENVIRONMENT.clone()));

            crate::ENVIRONMENT = Lazy::new(|| Environment::new(None));
            crate::ENVIRONMENT.set_enclosing(Some(previous.clone()));
            for statement in &self.statements {
                if let Err(e) = statement.interpret() {
                    let mut ret_val = None;
                    if e.message == "return" {
                        ret_val = Some(crate::ENVIRONMENT.get(Token {
                            token_type: crate::token_type::TokenType::Identifier(
                                "return".to_string(),
                            ),
                            lexeme: "return".to_string(),
                            line: 0,
                        })?);
                    }
                    crate::ENVIRONMENT.from(crate::ENVIRONMENT.get_enclosing().unwrap());
                    if let Some(value) = ret_val {
                        crate::ENVIRONMENT.define("return".to_string(), value);
                    }
                    return Err(e);
                }
            }
            crate::ENVIRONMENT.from(crate::ENVIRONMENT.get_enclosing().unwrap());
            Ok(())
        }
    }
}

impl Resolver for Block {
    fn resolve(self: Rc<Self>, scopes: &mut Scopes) -> Result<(), Error> {
        scopes.begin_scope();
        for statement in self.statements.clone() {
            statement.resolve(scopes)?;
        }
        scopes.end_scope();
        Ok(())
    }
}

impl Block {
    pub fn excute(&self) -> Result<(), Error> {
        for statement in &self.statements {
            statement.interpret()?;
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct If {
    pub condition: Rc<dyn syntax::Expr>,
    pub then_branch: Rc<dyn Stmt>,
    pub else_branch: Option<Rc<dyn Stmt>>,
}

impl std::fmt::Display for If {
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

impl Stmt for If {
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

impl Resolver for If {
    fn resolve(self: Rc<Self>, scopes: &mut Scopes) -> Result<(), Error> {
        self.condition.clone().resolve(scopes)?;
        self.then_branch.clone().resolve(scopes)?;
        if let Some(stmt) = &self.else_branch {
            stmt.clone().resolve(scopes)?;
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct While {
    pub condition: Rc<dyn syntax::Expr>,
    pub body: Rc<dyn Stmt>,
}

impl std::fmt::Display for While {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<while>({} {})", self.condition, self.body)
    }
}

impl Stmt for While {
    fn interpret(&self) -> Result<(), crate::error::Error> {
        while let Value::Boolean(true) = self.condition.eval()?.as_ref() {
            self.body.interpret()?;
        }
        Ok(())
    }
}

impl Resolver for While {
    fn resolve(self: Rc<Self>, scopes: &mut Scopes) -> Result<(), Error> {
        self.condition.clone().resolve(scopes)?;
        self.body.clone().resolve(scopes)
    }
}

#[derive(Debug, Clone)]
pub struct Function {
    pub name: Token,
    pub params: Vec<Token>,
    pub body: Rc<Block>,
}

impl std::fmt::Display for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut params = String::new();
        for param in &self.params {
            params.push_str(&format!("{} ", param.lexeme));
        }
        let mut body = String::new();
        for stmt in &self.body.statements {
            body.push_str(&format!("{}\n", stmt));
        }
        write!(f, "<fn>({} {})", self.name.lexeme, params)
    }
}

impl Stmt for Function {
    fn interpret(&self) -> Result<(), crate::error::Error> {
        unsafe {
            let function = crate::syntax::Value::Fun(Rc::new(self.clone()));

            crate::ENVIRONMENT.define(self.name.lexeme.clone(), Box::new(function));
        }
        Ok(())
    }
}

impl Resolver for Function {
    fn resolve(self: Rc<Self>, scopes: &mut Scopes) -> Result<(), Error> {
        scopes.declare(self.name.clone())?;
        scopes.define(self.name.clone());
        resolve_function(self, scopes, FunctionType::Function)
    }
}

#[derive(Debug)]
pub struct Return {
    pub keyword: Token,
    pub value: Option<Rc<dyn syntax::Expr>>,
}

impl std::fmt::Display for Return {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = match &self.value {
            Some(expr) => format!("{}", expr),
            None => "Nil".to_string(),
        };
        write!(f, "<re>({} {})", self.keyword, value)
    }
}

impl Stmt for Return {
    fn interpret(&self) -> Result<(), crate::error::Error> {
        if let Some(expr) = &self.value {
            let value = expr.eval()?;
            unsafe {
                crate::ENVIRONMENT.define("return".to_string(), value);
            }
        }
        Err(Error {
            line: 0,
            loc: "".to_string(),
            message: "return".to_string(),
        })
    }
}

impl Resolver for Return {
    fn resolve(self: Rc<Self>, scopes: &mut Scopes) -> Result<(), Error> {
        if scopes.get_current_function().is_none() {
            return Err(Error::report(
                self.keyword.clone(),
                "Can't return from top-level code".to_string(),
            ));
        }
        if let Some(expr) = &self.value {
            expr.clone().resolve(scopes)?;
        }
        Ok(())
    }
}

fn resolve_statements(statements: Vec<Rc<dyn Stmt>>, scopes: &mut Scopes) -> Result<(), Error> {
    for statement in statements {
        statement.resolve(scopes)?;
    }
    Ok(())
}

fn resolve_function(
    function: Rc<Function>,
    scopes: &mut Scopes,
    function_type: FunctionType,
) -> Result<(), Error> {
    let enclosing_function_type = scopes.1.clone();
    scopes.1 = Some(function_type);
    scopes.begin_scope();
    for param in &function.params {
        scopes.declare(param.clone())?;
        scopes.define(param.clone());
    }
    resolve_statements(function.body.statements.clone(), scopes)?;
    scopes.end_scope();
    scopes.1 = enclosing_function_type;
    Ok(())
}
