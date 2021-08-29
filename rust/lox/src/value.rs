
#[derive(Clone, Debug)]
pub struct Value {
    typ: ValueType,
}

#[derive(Clone, Debug)]
pub enum ValueType {
    Bool(bool),
    Nil,
    Number(f64),
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
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.typ {
            ValueType::Nil => write!(f, "nil"),
            ValueType::Bool(v) => write!(f, "{}", v),
            ValueType::Number(v) => write!(f, "{}", v),
        }
    }
}
