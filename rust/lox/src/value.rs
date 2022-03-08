use crate::allocator::Reference;
use crate::function::{Closure, NativeFn};
use crate::Function;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Value {
    Bool(bool),
    Nil,
    Number(f64),
    String(Reference<String>),
    Function(Reference<Function>),
    Closure(Reference<Closure>),
    NativeFn(NativeFn),
}


impl Value {
    pub fn is_falsy(&self) -> bool {
        match self {
            Self::Bool(v) => !v.clone(),
            Self::Nil => true,
            _ => false,
        }
    }

    pub fn as_number(&self) -> f64 {
        match self {
            Self::Number(v) => v.clone(),
            _ => unreachable!(),
        }
    }

    pub fn as_bool(&self) -> bool {
        match self {
            Self::Bool(v) => v.clone(),
            _ => unreachable!(),
        }
    }

    pub fn as_string(&self) -> &Reference<String> {
        match self {
            Self::String(v) => v,
            _ => unreachable!(),
        }
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Nil => write!(f, "nil"),
            Self::Bool(v) => write!(f, "{}", v),
            Self::Number(v) => write!(f, "{}", v),
            Self::String(id) => write!(f, "<string {}>", id),
            Self::Function(id) => write!(f, "<fn {}>", id),
            Self::Closure(id) => write!(f, "<closure {}>", id),
            Self::NativeFn(_) => write!(f, "<native fn>"),
        }
    }
}
