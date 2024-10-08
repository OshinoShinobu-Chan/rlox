use crate::ast::stmt::{Function, Instance};
use crate::{Builtin, Environment, Error, Token, TokenType};
use once_cell::sync::Lazy;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

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
    Fun(
        Rc<Function>,
        Option<Rc<RefCell<Instance>>>,
        Option<Box<Value>>,
    ),
    Builtin(Rc<Builtin>),
    Class {
        class: String,
        methods: HashMap<String, Box<Value>>,
        super_class: Option<Box<Value>>,
    },
    Instance(Rc<RefCell<Instance>>),
    Array(Vec<Box<Value>>),
    ArrayObject {
        array: Vec<Box<Value>>,
        index: usize,
    },
    Nil,
}

impl Value {
    pub fn bind(&mut self, instance: Rc<RefCell<Instance>>) {
        match self {
            Value::Fun(fun, _, super_) => {
                *self = Value::Fun(fun.clone(), Some(instance), super_.clone());
            }
            _ => {}
        }
    }
    pub fn is_class(&self) -> bool {
        match self {
            Value::Class {
                class: _,
                methods: _,
                super_class: _,
            } => true,
            _ => false,
        }
    }
    pub fn is_nil(&self) -> bool {
        match self {
            Value::Nil => true,
            _ => false,
        }
    }
    pub fn get_method(&self, name: &str) -> Option<Box<Value>> {
        match self {
            Value::Class {
                class: _,
                methods,
                super_class,
            } => {
                if methods.contains_key(name) {
                    methods.get(name).cloned()
                } else if let Some(super_class) = super_class {
                    super_class.get_method(name)
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

impl LoxCallable for Value {
    fn arity(&self) -> usize {
        if let Value::Fun(fun, _, _) = self {
            fun.params.len()
        } else if let Value::Class {
            class: _,
            methods,
            super_class: _,
        } = self
        {
            let initializer = methods.get("init");
            if let Some(initializer) = initializer {
                initializer.arity()
            } else {
                0
            }
        } else if let Value::Builtin(builtin) = self {
            builtin.arity
        } else {
            0
        }
    }

    fn call(&self, arguments: Vec<Box<Value>>) -> Result<Box<Value>, Error> {
        if let Value::Fun(fun, this, super_) = self {
            unsafe {
                // create a new environment for the function
                let previous = Rc::new(RefCell::new(crate::ENVIRONMENT.borrow().clone()));

                // bind super to the environment
                if let Some(super_) = super_ {
                    crate::ENVIRONMENT =
                        Lazy::new(|| Rc::new(RefCell::new(Environment::new(None))));
                    crate::ENVIRONMENT
                        .borrow_mut()
                        .set_enclosing(Some(previous.clone()));
                    crate::ENVIRONMENT
                        .borrow_mut()
                        .define("super".to_string(), super_.clone());
                }

                let super_outer = crate::ENVIRONMENT.borrow().clone();
                // bind this to the environment
                if this.is_some() {
                    crate::ENVIRONMENT =
                        Lazy::new(|| Rc::new(RefCell::new(Environment::new(None))));
                    crate::ENVIRONMENT
                        .borrow_mut()
                        .set_enclosing(Some(Rc::new(RefCell::new(super_outer.clone()))));
                    crate::ENVIRONMENT.borrow_mut().define(
                        "this".to_string(),
                        Box::new(Value::Instance(this.clone().unwrap())),
                    );
                }

                let outer = Rc::new(RefCell::new(crate::ENVIRONMENT.borrow().clone()));
                crate::ENVIRONMENT = Lazy::new(|| Rc::new(RefCell::new(Environment::new(None))));
                crate::ENVIRONMENT
                    .borrow_mut()
                    .set_enclosing(Some(outer.clone()));

                // pass the arguments to the function
                for (i, param) in fun.params.iter().enumerate() {
                    if let TokenType::Identifier(param_name) = param.token_type.clone() {
                        crate::ENVIRONMENT
                            .borrow_mut()
                            .define(param_name, arguments[i].clone());
                    }
                }

                let mut ret_val = {
                    // execute the function body
                    if let Err(e) = fun.body.excute() {
                        if e.message == "return".to_string() {
                            crate::ENVIRONMENT.borrow().get(Token {
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
                if fun.is_initializer {
                    ret_val = crate::ENVIRONMENT.borrow().get(Token {
                        token_type: TokenType::Identifier("this".to_string()),
                        lexeme: "this".to_string(),
                        line: 0,
                    });
                }
                crate::ENVIRONMENT.borrow_mut().from(previous);
                ret_val
            }
        } else if let Value::Builtin(builtin) = self {
            builtin.call(arguments)
        } else if let Value::Class {
            class,
            methods,
            super_class,
        } = self
        {
            let mut instance = Instance {
                class: class.clone(),
                fields: HashMap::new(),
                methods: methods.clone(),
                super_class: super_class.clone(),
            };
            let initializer = instance.methods.get_mut("init").cloned();
            let instance = Rc::new(RefCell::new(instance));
            if let Some(mut initializer) = initializer {
                initializer.bind(instance.clone());
                initializer.call(arguments)
            } else {
                Ok(Box::new(Value::Instance(instance)))
            }
        } else {
            Err(Error {
                line: 0,
                loc: "NoFun".to_string(),
                message: "Value that is not funciton can't be called".to_string(),
            })
        }
    }

    fn is_callable(&self) -> bool {
        if let Value::Fun(_, _, _) = self {
            true
        } else if let Value::Builtin(_) = self {
            true
        } else if let Value::Class {
            class: _,
            methods: _,
            super_class: _,
        } = self
        {
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
            (Value::Array(a), Value::Array(b)) => {
                let mut array = a.clone();
                array.extend(b);
                Ok(Box::new(Value::Array(array)))
            }
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
            Value::Fun(fun, _, _) => write!(f, "{}", fun),
            Value::Builtin(_) => write!(f, "<builtin fn>"),
            Value::Class {
                class,
                methods: _,
                super_class: _,
            } => write!(f, "{}", class),
            Value::Instance(instance) => write!(f, "{}", instance.borrow()),
            Value::Array(a) => write!(f, "{:?}", a),
            Value::ArrayObject { array, index } => write!(f, "{:?}[{}]", array, index),
        }
    }
}
