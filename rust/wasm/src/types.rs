use super::{Decoder, Error, List, Read, Result, VarInt32, VarUint8};

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Func(FuncType),
}

impl Type {
    pub fn func_type(&self) -> Option<&FuncType> {
        if let Type::Func(f) = self {
            return Some(f);
        }
        None
    }
}

impl Decoder for Type {
    fn decode<R: Read>(reader: &mut R) -> Result<Self> {
        let header = VarUint8::decode(reader)?.into();
        Ok(match header {
            0x60 => Type::Func(FuncType::decode(reader)?),
            invalid => return Err(Error::InvalidTypeSection(invalid)),
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ValueType {
    Int32,
    Int64,
    Float32,
    Float64,
}

impl Decoder for ValueType {
    fn decode<R: Read>(reader: &mut R) -> Result<Self> {
        let n = VarUint8::decode(reader)?.into();
        match n {
            0x7f => Ok(ValueType::Int32),
            0x7e => Ok(ValueType::Int64),
            0x7d => Ok(ValueType::Float32),
            0x7c => Ok(ValueType::Float64),
            0x70 => unimplemented!("funcref"),
            0x6F => unimplemented!("externref"),
            invalid => Err(Error::InvalidValueType(invalid)),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FuncType {
    params: Vec<ValueType>,
    results: Vec<ValueType>,
}

impl FuncType {
    pub fn params(&self) -> &[ValueType] {
        &self.params
    }

    pub fn results(&self) -> &[ValueType] {
        &self.results
    }
}

impl Decoder for FuncType {
    fn decode<R: Read>(reader: &mut R) -> Result<Self> {
        let params = List::<ValueType>::decode(reader)?.into_inner();
        let results = List::<ValueType>::decode(reader)?.into_inner();

        Ok(FuncType { params, results })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum BlockType {
    Empty,
    ValueType(ValueType),
    TypeIndex(u32),
}

impl Decoder for BlockType {
    fn decode<R: Read>(reader: &mut R) -> Result<Self> {
        let val = VarInt32::decode(reader)?;
        Ok(match val.into() {
            -64 => Self::Empty,                        // 0x40
            -1 => Self::ValueType(ValueType::Int32),   // 0x7f
            -2 => Self::ValueType(ValueType::Int64),   // 0x7e
            -3 => Self::ValueType(ValueType::Float32), // 0x7d
            -4 => Self::ValueType(ValueType::Float64), // 0x7c
            val => {
                let val = val.try_into().map_err(|_| Error::UnknownBlockType(val))?;
                Self::TypeIndex(val)
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{
        super::{test, Cursor, Decoder, Error, Result},
        FuncType, Type, ValueType,
    };

    fn value_type_decode(buf: &[u8]) -> Result<ValueType> {
        let mut reader = Cursor::new(buf);
        ValueType::decode(&mut reader)
    }

    fn func_type_decode(buf: &[u8]) -> Result<FuncType> {
        let mut reader = Cursor::new(buf);
        FuncType::decode(&mut reader)
    }

    fn type_decode(buf: &[u8]) -> Result<Type> {
        let mut reader = Cursor::new(buf);
        Type::decode(&mut reader)
    }

    test!(
        test_value_type,
        value_type_decode,
        (&vec![0x7f], ValueType::Int32, false),
        (&vec![0x7e], ValueType::Int64, false),
        (&vec![0x00], ValueType::Int32, true),
    );

    test!(
        test_func_type,
        func_type_decode,
        (
            &vec![0x02, 0x7f, 0x7f, 0x01, 0x7e],
            FuncType {
                params: vec![ValueType::Int32, ValueType::Int32],
                results: vec![ValueType::Int64],
            },
            false,
        ),
        (
            &vec![0x02, 0x7f, 0x01, 0x7e],
            FuncType {
                params: vec![],
                results: vec![],
            },
            true,
        ),
    );

    test!(
        test_type,
        type_decode,
        (
            &vec![0x05],
            Type::Func(FuncType {
                params: vec![],
                results: vec![]
            }),
            true
        ),
        (
            &vec![0x60, 0x01, 0x7e, 0x01, 0x7f],
            Type::Func(FuncType {
                params: vec![ValueType::Int64],
                results: vec![ValueType::Int32]
            }),
            false
        ),
    );
}
