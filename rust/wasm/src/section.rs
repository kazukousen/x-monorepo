use super::{
    buffer_read, Cursor, Decoder, Error, Export, ExportDesc, Func, FuncType, List, Read, Result,
    Type, VarUint32, VarUint8,
};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum Section {
    Custom(CustomSection),
    Type(TypeSection),
    Function(FunctionSection),
    Export(ExportSection),
    Code(CodeSection),
}

impl Decoder for Section {
    fn decode<R: Read>(reader: &mut R) -> Result<Self> {
        let id = match VarUint8::decode(reader) {
            Err(_) => return Err(Error::UnexpectedEOF),
            Ok(id) => id,
        };

        Ok(match id.into() {
            0 => Section::Custom(CustomSection::decode(reader)?),
            1 => Section::Type(TypeSection::decode(reader)?),
            3 => Section::Function(FunctionSection::decode(reader)?),
            7 => Section::Export(ExportSection::decode(reader)?),
            10 => Section::Code(CodeSection::decode(reader)?),
            invalid => return Err(Error::InvalidSectionId(invalid)),
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CustomSection {
    name: String,
    payload: Vec<u8>,
}

impl CustomSection {
    pub fn function_names(&self) -> Result<HashMap<u32, String>> {
        let mut m = HashMap::new();

        let mut reader = Cursor::new(&self.payload);

        loop {
            let id: u8 = VarUint8::decode(&mut reader)?.into();

            let _size: u32 = VarUint32::decode(&mut reader)?.into();

            if id == 1 {
                break;
            }
        }

        let n: u32 = VarUint32::decode(&mut reader)?.into();
        for _ in 0..n {
            let idx: u32 = VarUint32::decode(&mut reader)?.into();
            let size: usize = u32::from(VarUint32::decode(&mut reader)?) as usize;
            let func_name = String::from_utf8(buffer_read!(size, &mut reader)).expect("hoge");
            m.insert(idx, func_name);
        }

        Ok(m)
    }
}

impl Decoder for CustomSection {
    fn decode<R: Read>(reader: &mut R) -> Result<Self> {
        let size = u32::from(VarUint32::decode(reader)?) as usize;
        let name_len = u32::from(VarUint32::decode(reader)?) as usize;

        if size == 0 {
            return Err(Error::UnexpectedEOF);
        }

        let name = if name_len > 0 {
            String::from_utf8(buffer_read!(name_len, reader)).expect("hoge")
        } else {
            String::new()
        };

        let payload_len = size - (name_len + 1);

        let payload = if payload_len > 0 {
            buffer_read!(payload_len, reader)
        } else {
            Vec::new()
        };

        Ok(Self { name, payload })
    }
}

// signature.
#[derive(Debug, Clone, PartialEq)]
pub struct TypeSection(Vec<Type>);

impl TypeSection {
    pub fn get_func_type(&self, idx: u32) -> &FuncType {
        let Type::Func(ref func_type) = self
            .0
            .get(idx as usize)
            .expect("Due to validation functions should have valid types");
        func_type
    }
}

impl Decoder for TypeSection {
    fn decode<R: Read>(reader: &mut R) -> Result<Self> {
        let _length = u32::from(VarUint32::decode(reader)?) as usize;
        let list = List::<Type>::decode(reader)?.into_inner();
        Ok(Self(list))
    }
}

// its index is to be code index, its element is to be type index.
#[derive(Debug, Clone, PartialEq)]
pub struct FunctionSection(Vec<u32>);

impl FunctionSection {
    pub fn entries(&self) -> &[u32] {
        &self.0
    }
}

impl Decoder for FunctionSection {
    fn decode<R: Read>(reader: &mut R) -> Result<Self> {
        let _length = u32::from(VarUint32::decode(reader)?) as usize;
        let list = List::<VarUint32>::decode(reader)?.into_inner();
        Ok(Self(list.into_iter().map(|v| v.into()).collect()))
    }
}

// have pairs that are exported function name and function index.
#[derive(Debug, Clone, PartialEq)]
pub struct ExportSection(Vec<Export>);

impl ExportSection {
    pub fn entries(&self) -> &[Export] {
        &self.0
    }
}

impl Decoder for ExportSection {
    fn decode<R: Read>(reader: &mut R) -> Result<Self> {
        let _length = u32::from(VarUint32::decode(reader)?) as usize;
        let list = List::<Export>::decode(reader)?.into_inner();
        Ok(Self(list))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CodeSection(Vec<Func>);

impl CodeSection {
    pub fn entries(&self) -> &[Func] {
        return &self.0;
    }
}

impl Decoder for CodeSection {
    fn decode<R: Read>(reader: &mut R) -> Result<Self> {
        let _length = u32::from(VarUint32::decode(reader)?) as usize;
        let list = List::<Func>::decode(reader)?.into_inner();
        Ok(Self(list))
    }
}

#[cfg(test)]
mod tests {
    use super::{
        super::{test, Cursor, Decoder, Result},
        CustomSection,
    };
    use std::collections::HashMap;

    fn decode_custom(buf: &[u8]) -> Result<CustomSection> {
        let mut reader = Cursor::new(buf);
        CustomSection::decode(&mut reader)
    }

    fn decode_custom_hashmap(buf: &[u8]) -> Result<HashMap<u32, String>> {
        let mut reader = Cursor::new(buf);
        let custom = CustomSection::decode(&mut reader)?;
        custom.function_names()
    }

    const VEC: [u8; 35] = [
        0x22, 0x04, 0x6e, 0x61, 0x6d, 0x65, 0x01, 0x06, 0x01, 0x00, 0x03, 0x66, 0x69, 0x62, 0x02,
        0x13, 0x01, 0x00, 0x04, 0x00, 0x02, 0x70, 0x30, 0x01, 0x02, 0x6c, 0x30, 0x02, 0x02, 0x6c,
        0x31, 0x03, 0x02, 0x6c, 0x32,
    ];

    macro_rules! hashmap {
        ($($key: expr => $val: expr),*,) => {{
            let mut m = ::std::collections::HashMap::new();
            $( m.insert($key, $val); )*
            m
        }}
    }

    test!(
        test_decode_custom,
        decode_custom,
        (
            &VEC,
            CustomSection {
                name: "name".to_string(),
                payload: vec![
                    0x01, 0x06, 0x01, 0x00, 0x03, 0x66, 0x69, 0x62, 0x02, 0x13, 0x01, 0x00, 0x04,
                    0x00, 0x02, 0x70, 0x30, 0x01, 0x02, 0x6c, 0x30, 0x02, 0x02, 0x6c, 0x31, 0x03,
                    0x02, 0x6c, 0x32
                ]
            },
            false,
        ),
    );

    test!(
        test_decode_custom_hashmap,
        decode_custom_hashmap,
        (
            &VEC,
            hashmap![
                0u32 => "fib".to_string(),
            ],
            false,
        ),
    );
}
