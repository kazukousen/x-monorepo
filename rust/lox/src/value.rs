
#[derive(Clone, Debug, PartialEq)]
pub struct Value<'a> {
    typ: ValueType<'a>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ValueType<'a> {
    Bool(bool),
    Nil,
    Number(f64),
    Obj(Obj<'a>),
}

impl<'a> Value<'a> {

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
    
    pub fn new_string(s: &'a str) -> Self {
        Self {
            typ: ValueType::Obj(Obj{ typ: ObjType::String(s) })
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
            ValueType::Obj(v) => match v.typ {
                ObjType::String(_) => true,
            },
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

    pub fn as_obj(&self) -> &Obj<'a> {
        match &self.typ {
            ValueType::Obj(v) => v,
            _ => unreachable!(),
        }
    }

    pub fn as_string(&self) -> &'a str {
        match &self.typ {
            ValueType::Obj(v) => match v.typ {
                ObjType::String(v) => v,
            }
            _ => unreachable!(),
        }
    }
}

impl<'a> std::fmt::Display for Value<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.typ {
            ValueType::Nil => write!(f, "nil"),
            ValueType::Bool(v) => write!(f, "{}", v),
            ValueType::Number(v) => write!(f, "{}", v),
            ValueType::Obj(v) => match v.typ {
                ObjType::String(v) => write!(f, "{}", v),
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Obj<'a> {
    typ: ObjType<'a>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ObjType<'a> {
    String(&'a str),
}
