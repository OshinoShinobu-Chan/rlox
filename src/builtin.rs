use crate::error::Error;
use crate::syntax::Value;

#[derive(Clone)]
pub struct Builtin {
    pub arity: usize,
    pub call: fn(Vec<Box<Value>>) -> Result<Box<Value>, Error>,
}

impl std::fmt::Debug for Builtin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<builtin fn>")
    }
}

impl Builtin {
    pub fn call(&self, args: Vec<Box<Value>>) -> Result<Box<Value>, Error> {
        (self.call)(args)
    }
}
