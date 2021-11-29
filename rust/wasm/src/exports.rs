use super::{buffer_read, Decoder, Error, List, Read, Result, VarUint32, VarUint8};

#[derive(Debug, Clone, PartialEq)]
pub struct Export {
    name: String,
    desc: ExportDesc,
}

impl Export {
    pub fn name(&self) -> &str {
        return &self.name;
    }

    pub fn desc(&self) -> &ExportDesc {
        return &self.desc;
    }
}

impl Decoder for Export {
    fn decode<R: Read>(reader: &mut R) -> Result<Self> {
        let length = u32::from(VarUint32::decode(reader)?) as usize;

        let name = if length > 0 {
            String::from_utf8(buffer_read!(length, reader)).expect("hoge")
        } else {
            String::new()
        };
        let desc = ExportDesc::decode(reader)?;
        Ok(Export { name, desc })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExportDesc {
    Func(u32),
    Table(u32),
    Memory(u32),
    Global(u32),
}

impl Decoder for ExportDesc {
    fn decode<R: Read>(reader: &mut R) -> Result<Self> {
        let id = VarUint8::decode(reader)?.into();
        Ok(match id {
            0 => Self::Func(VarUint32::decode(reader)?.into()),
            1 => Self::Table(VarUint32::decode(reader)?.into()),
            2 => Self::Memory(VarUint32::decode(reader)?.into()),
            3 => Self::Global(VarUint32::decode(reader)?.into()),
            invalid => return Err(Error::InvalidExportDesc(invalid)),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{
        super::{test, Cursor, Decoder, Result},
        Export, ExportDesc,
    };

    fn decode_export(buf: &[u8]) -> Result<Export> {
        let mut reader = Cursor::new(buf);
        Export::decode(&mut reader)
    }

    test!(
        test_decode_export,
        decode_export,
        (
            &vec![0x03, 0x66, 0x69, 0x62, 0x00, 0x00],
            Export {
                name: "fib".to_string(),
                desc: ExportDesc::Func(0u32)
            },
            false,
        ),
    );
}
