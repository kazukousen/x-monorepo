use crate::allocator::Reference;
use crate::function::{Closure, NativeFn};
use crate::Function;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Value {
    typ: ValueType,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ValueType {
    Bool(bool),
    Nil,
    Number(f64),
    String(Reference<String>),
    Function(Reference<Function>),
    Closure(Reference<Closure>),
    NativeFn(NativeFn),
}

impl Value {
    pub fn new_bool(v: bool) -> Self {
        Self {
            typ: ValueType::Bool(v),
        }
    }

    pub fn new_nil() -> Self {
        Self {
            typ: ValueType::Nil,
        }
    }

    pub fn new_number(v: f64) -> Self {
        Self {
            typ: ValueType::Number(v),
        }
    }

    pub fn new_string(s: Reference<String>) -> Self {
        Self {
            typ: ValueType::String(s),
        }
    }

    pub fn new_function(id: Reference<Function>) -> Self {
        Self {
            typ: ValueType::Function(id),
        }
    }

    pub fn new_closure(id: Reference<Closure>) -> Self {
        Self {
            typ: ValueType::Closure(id),
        }
    }

    pub fn new_native_fn(f: NativeFn) -> Self {
        Self {
            typ: ValueType::NativeFn(f),
        }
    }

    pub fn is_nil(&self) -> bool {
        match self.typ {
            ValueType::Nil => true,
            _ => false,
        }
    }

    pub fn is_number(&self) -> bool {
        match self.typ {
            ValueType::Number(_) => true,
            _ => false,
        }
    }

    pub fn is_bool(&self) -> bool {
        match self.typ {
            ValueType::Bool(_) => true,
            _ => false,
        }
    }

    pub fn is_string(&self) -> bool {
        match &self.typ {
            ValueType::String(_) => true,
            _ => false,
        }
    }

    pub fn is_fun(&self) -> bool {
        match &self.typ {
            ValueType::Function(_) => true,
            _ => false,
        }
    }

    pub fn is_closure(&self) -> bool {
        match &self.typ {
            ValueType::Closure(_) => true,
            _ => false,
        }
    }

    pub fn is_native_fn(&self) -> bool {
        match &self.typ {
            ValueType::NativeFn(_) => true,
            _ => false,
        }
    }

    pub fn is_falsy(&self) -> bool {
        match self.typ {
            ValueType::Bool(v) => !v,
            ValueType::Nil => true,
            _ => false,
        }
    }

    pub fn as_number(&self) -> f64 {
        match self.typ {
            ValueType::Number(v) => v,
            _ => unreachable!(),
        }
    }

    pub fn as_bool(&self) -> bool {
        match self.typ {
            ValueType::Bool(v) => v,
            _ => unreachable!(),
        }
    }

    pub fn as_string(&self) -> &Reference<String> {
        match &self.typ {
            ValueType::String(v) => v,
            _ => unreachable!(),
        }
    }

    pub fn as_fun(&self) -> &Reference<Function> {
        match &self.typ {
            ValueType::Function(id) => id,
            _ => unreachable!(),
        }
    }

    pub fn as_closure(&self) -> &Reference<Closure> {
        match &self.typ {
            ValueType::Closure(id) => id,
            _ => unreachable!(),
        }
    }

    pub fn as_native_fn(&self) -> &NativeFn {
        match &self.typ {
            ValueType::NativeFn(f) => f,
            _ => unreachable!(),
        }
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.typ {
            ValueType::Nil => write!(f, "nil"),
            ValueType::Bool(v) => write!(f, "{}", v),
            ValueType::Number(v) => write!(f, "{}", v),
            ValueType::String(id) => write!(f, "<string {}>", id),
            ValueType::Function(id) => write!(f, "<fn {}>", id),
            ValueType::Closure(id) => write!(f, "<closure {}>", id),
            ValueType::NativeFn(_) => write!(f, "<native fn>"),
        }
    }
}
