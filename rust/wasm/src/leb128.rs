use super::{Decoder, Error, Read, Result};

#[derive(Debug, Clone, PartialEq)]
pub struct VarUint32(u32);

impl From<VarUint32> for u32 {
    fn from(v: VarUint32) -> Self {
        v.0
    }
}

impl From<u32> for VarUint32 {
    fn from(n: u32) -> Self {
        Self(n)
    }
}

impl Decoder for VarUint32 {
    fn decode<R: Read>(reader: &mut R) -> Result<Self> {
        let mut u8buf = [0u8; 1];
        let mut ret: u32 = 0;
        let mut shift = 0;
        loop {
            reader.read(&mut u8buf)?;
            let b = u8buf[0] as u32;
            ret |= (b & 0x7f).checked_shl(shift).ok_or(Error::InvalidUint32)?;
            if b & 0x80 == 0 {
                if shift >= 32 && (b as u8).leading_zeros() < 4 {
                    return Err(Error::InvalidUint32);
                }
                return Ok(VarUint32(ret | b << shift));
            }
            shift += 7;
        }
    }
}

pub struct VarInt32(i32);

impl From<VarInt32> for i32 {
    fn from(v: VarInt32) -> Self {
        v.0
    }
}

impl From<i32> for VarInt32 {
    fn from(n: i32) -> Self {
        Self(n)
    }
}

impl Decoder for VarInt32 {
    fn decode<R: Read>(reader: &mut R) -> Result<Self> {
        let mut u8buf = [0u8; 1];
        let mut ret = 0;

        for i in 0..=5 {
            reader.read(&mut u8buf)?;
            let b = u8buf[0] as i32;
            ret |= (b & 0x7f).checked_shl(i * 7).ok_or(Error::InvalidInt32)?;
            if b & 0x80 == 0 {
                if b & 0x40 == 0x40 {
                    // negative
                    ret |= (1i32 << (i + 1) * 7).wrapping_neg();
                }
                return Ok(ret.into());
            }
        }

        Err(Error::InvalidInt32)
    }
}

pub struct VarUint8(u8);

impl From<VarUint8> for u8 {
    fn from(v: VarUint8) -> Self {
        v.0
    }
}

impl From<u8> for VarUint8 {
    fn from(n: u8) -> Self {
        Self(n)
    }
}

impl Decoder for VarUint8 {
    fn decode<R: Read>(reader: &mut R) -> Result<Self> {
        let mut u8buf = [0u8; 1];
        reader.read(&mut u8buf)?;
        Ok(u8buf[0].into())
    }
}

pub struct Uint32(u32);

impl From<Uint32> for u32 {
    fn from(v: Uint32) -> Self {
        v.0
    }
}

impl From<u32> for Uint32 {
    fn from(n: u32) -> Self {
        Self(n)
    }
}

impl Decoder for Uint32 {
    fn decode<R: Read>(reader: &mut R) -> Result<Self> {
        let mut buf = [0u8; 4];
        reader.read(&mut buf)?;
        Ok(u32::from_le_bytes(buf).into())
    }
}

#[derive(Debug, Clone)]
pub struct List<T: Decoder>(Vec<T>);

impl<T: Decoder> List<T> {
    pub fn into_inner(self) -> Vec<T> {
        self.0
    }
}

impl<T: Decoder> Decoder for List<T> {
    fn decode<R: Read>(reader: &mut R) -> Result<Self> {
        let count = VarUint32::decode(reader)?.into();
        let mut list = Vec::new();
        for _ in 0..count {
            list.push(T::decode(reader)?);
        }
        Ok(List(list))
    }
}

#[cfg(test)]
mod tests {
    use super::{
        super::{test, Cursor, Decoder, Result},
        List, VarInt32, VarUint32, VarUint8,
    };

    fn uint32_decode(buf: &[u8]) -> Result<u32> {
        let mut reader = Cursor::new(buf);
        VarUint32::decode(&mut reader).map(|v| v.into())
    }

    fn int32_decode(buf: &[u8]) -> Result<i32> {
        let mut reader = Cursor::new(buf);
        VarInt32::decode(&mut reader).map(|v| v.into())
    }

    fn uint8_decode(buf: &[u8]) -> Result<u8> {
        let mut reader = Cursor::new(buf);
        VarUint8::decode(&mut reader).map(|v| v.into())
    }

    fn uint32_list_decode(buf: &[u8]) -> Result<Vec<VarUint32>> {
        let mut reader = Cursor::new(buf);
        let list = List::<VarUint32>::decode(&mut reader)?.into_inner();
        Ok(list)
    }

    test!(
        test_uint32,
        uint32_decode,
        (&vec![0x00], 0u32, false),
        (&vec![0x04], 4u32, false),
        (&vec![0x80, 0x7f], 16256u32, false),
        (&vec![0xe5, 0x8e, 0x26], 624485u32, false),
        (&vec![0x80, 0x80, 0x80, 0x4f], 165675008u32, false),
        (&vec![0x83, 0x80, 0x80, 0x80, 0x80, 0x00], 0u32, true),
    );

    test!(
        test_int32,
        int32_decode,
        (&vec![0x13], 19i32, false),
        (&vec![0x00], 0i32, false),
        (&vec![0xff, 0x00], 127i32, false),
        (&vec![0x7f], -1i32, false),
        (&vec![0x81, 0x7f], -127i32, false),
    );

    test!(
        test_uint8,
        uint8_decode,
        (&vec![0x7f], 127u8, false),
        (&Vec::<u8>::new(), 0u8, true),
    );

    test!(
        test_uint32_list,
        uint32_list_decode,
        (
            &vec![0x03, 0x01, 0x02, 0x03],
            vec![VarUint32(1), VarUint32(2), VarUint32(3)],
            false
        ),
    );
}
