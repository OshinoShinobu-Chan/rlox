//! expressions in AST
use crate::builtin::Builtin;
use crate::environment::Environment;
use crate::error::Error;
use crate::statement::{self, Function};
use crate::token::Token;
use crate::token_type::TokenType;
use once_cell::sync::Lazy;
use rlox_macro::Expr;
use std::cell::RefCell;
use std::rc::Rc;

pub trait Expr: std::fmt::Display + std::fmt::Debug + statement::Resolver {
    fn eval(&self) -> Result<Box<Value>, Error>;
    fn type_name(&self) -> String {
        std::any::type_name::<Self>().to_string()
    }
}

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

impl statement::Resolver for Literal {
    fn resolve(self: Rc<Self>, _scopes: &mut crate::Scopes) -> Result<(), Error> {
        Ok(())
    }
}

#[derive(Expr, Debug)]
pub struct Operator {
    pub operator: Token,
}

impl Expr for Operator {
    fn eval(&self) -> Result<Box<Value>, Error> {
        panic!("Evaluate an operator is not supported")
    }
}

impl statement::Resolver for Operator {
    fn resolve(self: Rc<Self>, _scopes: &mut crate::Scopes) -> Result<(), Error> {
        Ok(())
    }
}

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

impl statement::Resolver for Unary {
    fn resolve(self: Rc<Self>, scopes: &mut crate::Scopes) -> Result<(), Error> {
        self.right.clone().resolve(scopes)
    }
}

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
                        "Binary operator + only works with numbers or strings".to_string(),
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

impl statement::Resolver for Binary {
    fn resolve(self: Rc<Self>, scopes: &mut crate::Scopes) -> Result<(), Error> {
        self.left.clone().resolve(scopes)?;
        self.right.clone().resolve(scopes)
    }
}

#[derive(Expr, Debug)]
pub struct Grouping {
    #[wrapper]
    pub expression: Rc<dyn Expr>,
}

impl Expr for Grouping {
    fn eval(&self) -> Result<Box<Value>, Error> {
        self.expression.eval()
    }
}

impl statement::Resolver for Grouping {
    fn resolve(self: Rc<Self>, scopes: &mut crate::Scopes) -> Result<(), Error> {
        self.expression.clone().resolve(scopes)
    }
}

#[derive(Debug)]
/// Variable expression
pub struct Variable {
    pub name: Token,
}

impl std::fmt::Display for Variable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name.lexeme)
    }
}

impl Expr for Variable {
    fn eval(&self) -> Result<Box<Value>, Error> {
        let distance = unsafe { crate::LOCALS.get(&(self as *const dyn Expr)) };
        if let Some(distance) = distance {
            unsafe { crate::ENVIRONMENT.get_at(*distance, self.name.clone()) }
        } else {
            unsafe {
                crate::ENVIRONMENT
                    .get_global()
                    .borrow()
                    .get(self.name.clone())
            }
        }
    }
}

impl statement::Resolver for Variable {
    fn resolve(self: Rc<Self>, scopes: &mut crate::Scopes) -> Result<(), Error> {
        if !scopes.is_empty() && scopes.peek().unwrap().get(&self.name.lexeme) == Some(&false) {
            Err(Error::new(
                self.name.line,
                self.name.lexeme.clone(),
                "Can't read local variable in its own initializer".to_string(),
            ))
        } else {
            scopes.resolve_local(Rc::as_ptr(&self) as *const dyn Expr, &self.name);
            Ok(())
        }
    }
}

#[derive(Expr, Debug)]
pub struct Assignment {
    pub name: Token,
    pub value: Rc<dyn Expr>,
}

impl Expr for Assignment {
    fn eval(&self) -> Result<Box<Value>, Error> {
        let value = self.value.eval()?;
        let distance = unsafe { crate::LOCALS.get(&(self as *const dyn Expr)) };
        if let Some(distance) = distance {
            unsafe {
                crate::ENVIRONMENT.assign_at(*distance, self.name.clone(), value.clone())?;
            }
        } else {
            unsafe {
                crate::ENVIRONMENT
                    .get_global()
                    .borrow_mut()
                    .assign(self.name.clone(), value.clone())?;
            }
        }
        Ok(value)
    }
}

impl statement::Resolver for Assignment {
    fn resolve(self: Rc<Self>, scopes: &mut crate::Scopes) -> Result<(), Error> {
        self.value.clone().resolve(scopes)?;
        scopes.resolve_local(Rc::as_ptr(&self) as *const dyn Expr, &self.name);
        Ok(())
    }
}

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

impl statement::Resolver for Logic {
    fn resolve(self: Rc<Self>, scopes: &mut crate::Scopes) -> Result<(), Error> {
        self.left.clone().resolve(scopes)?;
        self.right.clone().resolve(scopes)
    }
}

#[derive(Debug)]
pub struct Call {
    pub callee: Rc<dyn Expr>,
    pub arguments: Vec<Rc<dyn Expr>>,
    pub paren: Token,
}

impl std::fmt::Display for Call {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut arguments = String::new();
        for arg in &self.arguments {
            arguments.push_str(&format!("{}, ", arg));
        }
        write!(f, "<c>({} {})", self.callee, arguments)
    }
}

impl Expr for Call {
    fn eval(&self) -> Result<Box<Value>, Error> {
        let callee = self.callee.eval()?;

        let mut arguements = Vec::new();
        for arg in &self.arguments {
            arguements.push(arg.eval()?);
        }

        if !callee.is_callable() {
            Err(Error::new(
                self.paren.line,
                self.paren.lexeme.clone(),
                "Can only call functions and classes".to_string(),
            ))
        } else if arguements.len() != callee.arity() {
            Err(Error::new(
                self.paren.line,
                self.paren.lexeme.clone(),
                format!(
                    "Expected {} arguments but got {}.",
                    callee.arity(),
                    arguements.len()
                ),
            ))
        } else {
            callee.call(arguements)
        }
    }
}

impl statement::Resolver for Call {
    fn resolve(self: Rc<Self>, scopes: &mut crate::Scopes) -> Result<(), Error> {
        self.callee.clone().resolve(scopes)?;
        for arg in &self.arguments {
            arg.clone().resolve(scopes)?;
        }
        Ok(())
    }
}

pub trait LoxCallable {
    fn call(&self, arguments: Vec<Box<Value>>) -> Result<Box<Value>, Error>;
    fn arity(&self) -> usize;
    fn is_callable(&self) -> bool;
}

#[derive(Clone, Debug)]
pub enum Value {
    Number(f64),
    String(String),
    Boolean(bool),
    Fun(Rc<Function>),
    Builtin(Rc<Builtin>),
    Nil,
}

impl LoxCallable for Value {
    fn arity(&self) -> usize {
        if let Value::Fun(fun) = self {
            fun.params.len()
        } else {
            0
        }
    }

    fn call(&self, arguments: Vec<Box<Value>>) -> Result<Box<Value>, Error> {
        if let Value::Fun(fun) = self {
            unsafe {
                // create a new environment for the function
                let previous = Rc::new(RefCell::new(crate::ENVIRONMENT.clone()));
                crate::ENVIRONMENT = Lazy::new(|| Environment::new(None));
                crate::ENVIRONMENT.set_enclosing(Some(previous.clone()));

                // pass the arguments to the function
                for (i, param) in fun.params.iter().enumerate() {
                    if let TokenType::Identifier(param_name) = param.token_type.clone() {
                        crate::ENVIRONMENT.define(param_name, arguments[i].clone());
                    }
                }

                let ret_val = {
                    // execute the function body
                    if let Err(e) = fun.body.excute() {
                        if e.message == "return".to_string() {
                            crate::ENVIRONMENT.get(Token {
                                token_type: TokenType::Identifier("return".to_string()),
                                lexeme: "return".to_string(),
                                line: 0,
                            })
                        } else {
                            Err(e)
                        }
                    } else {
                        Ok(Box::new(Value::Nil))
                    }
                };
                crate::ENVIRONMENT.from(previous);
                ret_val
            }
        } else if let Value::Builtin(builtin) = self {
            builtin.call(arguments)
        } else {
            Err(Error {
                line: 0,
                loc: "NoFun".to_string(),
                message: "Value that is not funciton can't be called".to_string(),
            })
        }
    }

    fn is_callable(&self) -> bool {
        if let Value::Fun(_) = self {
            true
        } else if let Value::Builtin(_) = self {
            true
        } else {
            false
        }
    }
}

impl std::ops::Add for Value {
    type Output = Result<Box<Value>, Error>;

    fn add(self, other: Self) -> Self::Output {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => Ok(Box::new(Value::Number(a + b))),
            (Value::String(a), Value::String(b)) => Ok(Box::new(Value::String(a + &b))),
            _ => Err(Error::new(0, "".to_string(), "".to_string())),
        }
    }
}

impl std::ops::Sub for Value {
    type Output = Result<Box<Value>, Error>;

    fn sub(self, other: Self) -> Self::Output {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => Ok(Box::new(Value::Number(a - b))),
            _ => Err(Error::new(0, "".to_string(), "".to_string())),
        }
    }
}

impl std::ops::Mul for Value {
    type Output = Result<Box<Value>, Error>;

    fn mul(self, other: Self) -> Self::Output {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => Ok(Box::new(Value::Number(a * b))),
            _ => Err(Error::new(0, "".to_string(), "".to_string())),
        }
    }
}

impl std::ops::Div for Value {
    type Output = Result<Box<Value>, Error>;

    fn div(self, other: Self) -> Self::Output {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => Ok(Box::new(Value::Number(a / b))),
            _ => Err(Error::new(0, "".to_string(), "".to_string())),
        }
    }
}

impl std::ops::Neg for Value {
    type Output = Result<Box<Value>, Error>;

    fn neg(self) -> Self::Output {
        match self {
            Value::Number(n) => Ok(Box::new(Value::Number(-n))),
            _ => Err(Error::new(0, "".to_string(), "".to_string())),
        }
    }
}

impl std::ops::Not for Value {
    type Output = Result<Box<Value>, Error>;

    fn not(self) -> Self::Output {
        match self {
            Value::Boolean(b) => Ok(Box::new(Value::Boolean(!b))),
            Value::Nil => Ok(Box::new(Value::Boolean(true))),
            _ => Err(Error::new(0, "".to_string(), "".to_string())),
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Boolean(a), Value::Boolean(b)) => a == b,
            (Value::Nil, Value::Nil) => true,
            _ => false,
        }
    }
}

impl Value {
    pub fn cmp(&self, other: &Self) -> Result<Option<std::cmp::Ordering>, Error> {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => Ok(a.partial_cmp(b)),
            _ => Err(Error::new(0, "".to_string(), "".to_string())),
        }
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Number(n) => write!(f, "{}", n),
            Value::String(s) => write!(f, "{}", s),
            Value::Boolean(b) => write!(f, "{}", b),
            Value::Nil => write!(f, "Nil"),
            Value::Fun(fun) => write!(f, "{}", fun),
            Value::Builtin(_) => write!(f, "<builtin fn>"),
        }
    }
}
