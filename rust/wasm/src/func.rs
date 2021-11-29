use super::{buffer_read, Cursor, Decoder, Instructions, List, Read, Result, ValueType, VarUint32};

#[derive(Debug, Clone, PartialEq)]
pub struct Func {
    locals: Vec<Local>,
    body: Instructions,
}

impl Func {
    pub fn locals(&self) -> &[Local] {
        &self.locals
    }

    pub fn body(&self) -> &Instructions {
        &self.body
    }
}

impl Decoder for Func {
    fn decode<R: Read>(reader: &mut R) -> Result<Self> {
        let length = u32::from(VarUint32::decode(reader)?) as usize;
        let locals = List::<Local>::decode(reader)?.into_inner();
        let body = if length > 0 {
            let buf = buffer_read!(length - (locals.len() * 2 + 1), reader);
            let mut reader = Cursor::new(buf);
            Instructions::decode(&mut reader)?
        } else {
            Instructions::new()
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
        super::{test, Cursor, Decoder, Instruction, Instructions, Result, ValueType},
        Func, Local,
    };

    fn decode_code_local(buf: &[u8]) -> Result<Vec<Local>> {
        let mut reader = Cursor::new(buf);
        Func::decode(&mut reader).map(|f| f.locals().iter().cloned().collect())
    }

    test!(
        test_decode_code_local,
        decode_code_local,
        (
            &[0x04, 0x01, 0x03, 0x7f, 0x0b],
            &[Local {
                n: 3,
                value_type: ValueType::Int32
            }],
            false,
        ),
    );
}
