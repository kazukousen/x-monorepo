use super::{buffer_read, Decoder, List, Read, Result, ValueType, VarUint32};

#[derive(Debug, Clone, PartialEq)]
pub struct Func {
    locals: Vec<Local>,
    body: Vec<u8>,
}

impl Func {
    pub fn locals(&self) -> &[Local] {
        &self.locals
    }

    pub fn body(&self) -> &[u8] {
        &self.body
    }
}

impl Decoder for Func {
    fn decode<R: Read>(reader: &mut R) -> Result<Self> {
        let length = u32::from(VarUint32::decode(reader)?) as usize;
        let locals = List::<Local>::decode(reader)?.into_inner();
        let body = if length > 0 {
            buffer_read!(length - (locals.len() * 2 + 1), reader)
        } else {
            vec![]
        };
        Ok(Self { locals, body })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Local {
    n: u32,
    value_type: ValueType,
}

impl Decoder for Local {
    fn decode<R: Read>(reader: &mut R) -> Result<Self> {
        let n = VarUint32::decode(reader)?.into();
        let value_type = ValueType::decode(reader)?;
        Ok(Self { n, value_type })
    }
}

#[cfg(test)]
mod tests {
    use super::{
        super::{test, Cursor, Decoder, Result, ValueType},
        Func, Local,
    };

    fn decode_code(buf: &[u8]) -> Result<Func> {
        let mut reader = Cursor::new(buf);
        Func::decode(&mut reader)
    }

    test!(
        test_decode_code,
        decode_code,
        (
            &vec![0x06, 0x01, 0x03, 0x7f, 0x66, 0x69, 0x62],
            Func {
                locals: vec![Local {
                    n: 3,
                    value_type: ValueType::Int32
                }],
                body: vec![0x66, 0x69, 0x62],
            },
            false,
        ),
    );
}
