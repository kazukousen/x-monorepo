use super::{
    CodeSection, CustomSection, Decoder, Error, ExportSection, FunctionSection, Read, Result,
    Section, TypeSection, Uint32, VarUint32,
};
use std::collections::HashMap;

const MAGIC_NUMBER: [u8; 4] = [0x00, 0x61, 0x73, 0x6d];
const VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq)]
pub struct Module {
    sections: Vec<Section>,
}

impl Module {
    pub fn sections(&self) -> &Vec<Section> {
        &self.sections
    }

    pub fn type_section(&self) -> Option<&TypeSection> {
        for section in self.sections() {
            if let Section::Type(ref s) = *section {
                return Some(s);
            }
        }
        None
    }

    pub fn function_section(&self) -> Option<&FunctionSection> {
        for section in self.sections() {
            if let Section::Function(ref s) = *section {
                return Some(s);
            }
        }
        None
    }

    pub fn export_section(&self) -> Option<&ExportSection> {
        for section in self.sections() {
            if let Section::Export(ref s) = *section {
                return Some(s);
            }
        }
        None
    }

    pub fn code_section(&self) -> Option<&CodeSection> {
        for section in self.sections() {
            if let Section::Code(ref s) = *section {
                return Some(s);
            }
        }
        None
    }

    pub fn function_names(&self) -> Option<HashMap<u32, String>> {
        let c = self.custom_function()?;
        c.function_names().ok()
    }

    fn custom_function(&self) -> Option<&CustomSection> {
        for section in self.sections() {
            if let Section::Custom(ref s) = *section {
                return Some(s);
            }
        }
        None
    }
}

impl Decoder for Module {
    fn decode<R: Read>(reader: &mut R) -> Result<Self> {
        let mut magic = [0u8; 4];
        reader.read(&mut magic)?;
        if magic != MAGIC_NUMBER {
            return Err(Error::InvalidMagic);
        };
        let version: u32 = Uint32::decode(reader)?.into();
        if version != VERSION {
            return Err(Error::UnsupportedVersion(version));
        };

        let mut sections = Vec::new();
        loop {
            match Section::decode(reader) {
                Ok(section) => {
                    sections.push(section);
                }
                Err(Error::UnexpectedEOF) => {
                    break;
                }
                Err(e) => return Err(e),
            }
        }

        Ok(Module { sections })
    }
}

#[cfg(test)]
mod tests {
    use super::{
        super::{decode_file, test},
        Module,
    };

    #[test]
    fn test_decode_file() {
        let module = decode_file("./fib.wasm").expect("should be decoded");
        assert_eq!(5, module.sections().len());
        assert!(module.type_section().is_some());
        assert!(module.function_section().is_some());
        assert!(module.export_section().is_some());
        assert!(module.code_section().is_some());
    }
}
