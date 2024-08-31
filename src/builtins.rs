use crate::ast::stmt::function::Builtin;
use crate::ast::Value;

pub static CLOCK: Builtin = Builtin {
    arity: 0,
    call: |_: Vec<Box<Value>>| {
        let time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs_f64();
        Ok(Box::new(Value::Number(time)))
    },
};

pub static STR: Builtin = Builtin {
    arity: 1,
    call: |args: Vec<Box<Value>>| Ok(Box::new(Value::String(args[0].to_string()))),
};
