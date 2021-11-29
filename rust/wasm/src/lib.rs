mod exports;
mod func;
mod instance;
mod leb128;
mod module;
mod ops;
mod section;
mod types;

use exports::{Export, ExportDesc};
use func::{Func, Local};
use leb128::{List, Uint32, VarInt32, VarUint32, VarUint8};
use module::Module;
use section::{CodeSection, CustomSection, ExportSection, FunctionSection, Section, TypeSection};
use std::fmt::Formatter;
use std::io::Read as io_read;
use types::{BlockType, FuncType, Type, ValueType};

pub trait Decoder: Sized {
    fn decode<R: Read>(reader: &mut R) -> Result<Self>;
}

#[derive(Debug, Clone)]
pub enum Error {
    UnexpectedEOF,
    InvalidMagic,
    InvalidUint32,
    InvalidInt32,
    InvalidSectionId(u8),
    InvalidTypeSection(u8),
    InvalidValueType(u8),
    InvalidExportDesc(u8),
    InvalidExportSection(u8),
    UnknownBlockType(i32),
    Io(String),
    UnsupportedVersion(u32),
}

pub type Result<T> = std::result::Result<T, Error>;

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::UnexpectedEOF => write!(f, "unexpected end of input"),
            Self::InvalidMagic => write!(
                f,
                "Invalid magic number. the binary should start with '0x00 0x61 0x73 0x6d'"
            ),
            Self::InvalidUint32 => write!(f, "Not an uint32"),
            Self::InvalidInt32 => write!(f, "Not an int32"),
            Self::InvalidSectionId(id) => write!(f, "Invalid section id: {}", id),
            Self::InvalidTypeSection(invalid) => write!(f, "Invalid type section: {}", invalid),
            Self::InvalidValueType(invalid) => write!(f, "Invalid type: {}", invalid),
            Self::InvalidExportDesc(invalid) => {
                write!(f, "Invalid export description: {}", invalid)
            }
            Self::InvalidExportSection(invalid) => {
                write!(f, "Invalid export section: {}", invalid)
            }
            Self::UnknownBlockType(invalid) => {
                write!(f, "Invalid block type: {}", invalid)
            }
            Self::Io(ref msg) => write!(f, "{}", msg),
            Self::UnsupportedVersion(version) => write!(f, "Unsupported version: {}", version),
        }
    }
}

pub trait Read {
    fn read(&mut self, buf: &mut [u8]) -> Result<()>;
}

pub struct Cursor<T> {
    inner: T,
    pos: usize,
}

impl<T> Cursor<T> {
    pub fn new(inner: T) -> Self {
        Self { inner, pos: 0 }
    }
}

impl<T: AsRef<[u8]>> Read for Cursor<T> {
    fn read(&mut self, buf: &mut [u8]) -> Result<()> {
        let slice = self.inner.as_ref();
        let remain = slice.len() - self.pos;
        let requested = buf.len();
        if requested > remain {
            return Err(Error::UnexpectedEOF);
        }

        buf.copy_from_slice(&slice[self.pos..(self.pos + requested)]);
        self.pos += requested;

        Ok(())
    }
}

impl Read for ::std::fs::File {
    fn read(&mut self, buf: &mut [u8]) -> Result<()> {
        <::std::fs::File as ::std::io::Read>::read(self, buf)
            .map_err(|e| Error::Io(format!("{:?}", e)))?;
        Ok(())
    }
}

pub fn decode_file<P: AsRef<::std::path::Path>>(p: P) -> Result<Module> {
    let mut f = ::std::fs::File::open(p)
        .map_err(|e| Error::Io(format!("Can't read from the file: {:?}", e)))?;

    Module::decode(&mut f)
}

#[macro_export]
macro_rules! buffer_read {
    ($length: expr, $reader: expr) => {{
        let mut ret = Vec::new();
        let mut current_read = 0;
        let mut buf = [0u8; 1024];
        while current_read < $length {
            let try_read = if $length - current_read > 1024 {
                1024
            } else {
                $length - current_read
            };

            $reader.read(&mut buf[0..try_read])?;
            ret.extend_from_slice(&buf[0..try_read]);

            current_read += try_read
        }
        ret
    }};
}

// pair.0: an input value
// pair.1: an expected value
// pair.2: had error as boolean
#[macro_export]
macro_rules! test {
    ($fn_name: ident, $test_func: ident, $($pair: expr),*,) => {
        #[test]
        fn $fn_name() {
            $({
                let res = $test_func($pair.0);
                if $pair.2 { // error
                assert!(res.is_err());
                println!("ERROR: {}", res.err().unwrap());
                return;
                } else {
                    match res {
                        Ok(res) => {
                            assert_eq!(res, $pair.1);
                        }
                        Err(err) => {
                            panic!("{:?}", err);
                        }
                    }
                }
            })*
        }
    }
}
