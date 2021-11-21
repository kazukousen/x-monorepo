use super::{BlockType, Decoder, Error, Read, Result, VarInt32, VarUint32, VarUint8};

#[derive(Debug, Clone, PartialEq)]
pub struct Instructions(Vec<Instruction>);

impl Instructions {
    pub fn entries(&self) -> &[Instruction] {
        &self.0
    }
}

impl Decoder for Instructions {
    fn decode<R: Read>(reader: &mut R) -> Result<Self> {
        let mut ret = Vec::new();
        let mut block_count: usize = 1;
        while block_count != 0 {
            let inst = Instruction::decode(reader)?;
            if inst.is_terminal() {
                block_count -= 1;
            } else if inst.is_block() {
                block_count = block_count.checked_add(1).ok_or(Error::InvalidUint32)?;
                // TODO
            }

            ret.push(inst);
        }

        Ok(Self(ret))
    }
}

// https://webassembly.github.io/spec/core/binary/instructions.html
#[derive(Debug, Clone, PartialEq)]
pub enum Instruction {
    // control instructions
    Nop,
    Unreachable,
    Block(BlockType),
    Loop(BlockType),
    If(BlockType),
    Else,
    End,
    Br(u32),   // label idx
    BrIf(u32), // label idx
    BrTable,
    Return,
    Call(u32),              // func idx
    CallIndirect(u32, u32), // type idx, table idx

    // parametric instructions

    // variable instructions
    LocalGet(u32),  // local idx
    LocalSet(u32),  // local idx
    LocalTee(u32),  // local idx
    GlobalGet(u32), // global idx
    GlobalSet(u32), // global idx

    // table instructions

    // memory instructions

    // numeric instructions
    I32Const(i32),
    I64Const(i64),
    F32Const(f32),
    F64Const(f64),

    I32Eqz,
    I32Eq,
    I32Ne,
    I32LtS,
    I32LtU,
    I32GtS,
    I32GtU,
    I32LeS,
    I32LeU,
    I32GeS,
    I32GeU,

    I64Eqz,
    I64Eq,
    I64Ne,
    I64LtS,
    I64LtU,
    I64GtS,
    I64GtU,
    I64LeS,
    I64LeU,
    I64GeS,
    I64GeU,

    I32Clz,
    I32Ctz,
    I32PopCnt,
    I32Add,
    I32Sub,
}

impl Instruction {
    fn is_block(&self) -> bool {
        matches!(self, &Self::Block(_) | &Self::Loop(_) | &Self::If(_))
    }

    fn is_terminal(&self) -> bool {
        matches!(self, &Self::End)
    }
}

impl Decoder for Instruction {
    fn decode<R: Read>(reader: &mut R) -> Result<Self> {
        // An opcode is represented by a single byte.
        let op: u8 = VarUint8::decode(reader)?.into();
        Ok(match op {
            // control instructions
            0x00 => Self::Unreachable,
            0x01 => Self::Nop,
            0x02 => Self::Block(BlockType::decode(reader)?),
            0x03 => Self::Loop(BlockType::decode(reader)?),
            0x04 => Self::If(BlockType::decode(reader)?),
            0x05 => Self::Else,
            0x0b => Self::End,
            0x0c => Self::Br(VarUint32::decode(reader)?.into()),
            0x0d => Self::BrIf(VarUint32::decode(reader)?.into()),
            // 0x0e => Self::BrTable
            0x0f => Self::Return,
            0x10 => Self::Call(VarUint32::decode(reader)?.into()),
            0x11 => Self::CallIndirect(
                VarUint32::decode(reader)?.into(),
                VarUint32::decode(reader)?.into(),
            ),

            // parametric instructions

            // variable instructions
            0x20 => Self::LocalGet(VarUint32::decode(reader)?.into()),
            0x21 => Self::LocalSet(VarUint32::decode(reader)?.into()),
            0x22 => Self::LocalTee(VarUint32::decode(reader)?.into()),
            0x23 => Self::GlobalGet(VarUint32::decode(reader)?.into()),
            0x24 => Self::GlobalSet(VarUint32::decode(reader)?.into()),

            // table instructions

            // memory instructions

            // numeric instructions
            0x41 => Self::I32Const(VarInt32::decode(reader)?.into()),
            // 0x42
            // 0x43
            // 0x44
            0x45 => Self::I32Eqz,
            0x46 => Self::I32Eq,
            0x47 => Self::I32Ne,
            0x48 => Self::I32LtS,
            0x49 => Self::I32LtU,
            0x4a => Self::I32GtS,
            0x4b => Self::I32GtU,
            0x4c => Self::I32LeS,
            0x4d => Self::I32LeU,
            0x4e => Self::I32GeS,
            0x4f => Self::I32GeU,

            0x6a => Self::I32Add,

            op => return Err(Error::InvalidSectionId(op)),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{
        super::{BlockType, Cursor, Decoder, ValueType},
        Instruction, Instructions,
    };
    use std::fs::read;

    #[test]
    fn test_fib() {
        let buf: &[u8] = &[
            0x20, 0x00, 0x41, 0x02, 0x4f, 0x04, 0x40, 0x20, 0x00, 0x41, 0x7f, 0x6a, 0x21, 0x01,
            0x41, 0x01, 0x21, 0x00, 0x03, 0x40, 0x20, 0x00, 0x22, 0x03, 0x20, 0x02, 0x6a, 0x21,
            0x00, 0x20, 0x03, 0x21, 0x02, 0x20, 0x01, 0x41, 0x7f, 0x6a, 0x22, 0x01, 0x0d, 0x00,
            0x0b, 0x0b, 0x20, 0x00, 0x0b,
        ];

        let mut reader = Cursor::new(buf);

        let instructions = Instructions::decode(&mut reader).unwrap();

        assert_eq!(27usize, instructions.entries().len());
        assert_eq!(
            &[
                // 0x20 0x00 => local.get 0
                Instruction::LocalGet(0u32),
                // 0x41, 0x02 => i32.const 2
                Instruction::I32Const(2i32),
                // 0x4f => i32.ge_u
                Instruction::I32GeU,
                // 0x04 0x40 => if
                Instruction::If(BlockType::Empty),
                // 0x20 0x00 => local.get 0
                Instruction::LocalGet(0u32),
                // 0x41, 0x7f => i32.const -1
                Instruction::I32Const(-1i32),
                // 0x6a => i32.add
                Instruction::I32Add,
                // 0x21 0x01 => local.set 1
                Instruction::LocalSet(1u32),
                // 0x41, 0x01 => i32.const 1
                Instruction::I32Const(1i32),
                // 0x21 0x00 => local.set 0
                Instruction::LocalSet(0u32),
                // 0x03 0x40 => loop
                Instruction::Loop(BlockType::Empty),
                // 0x20, 0x00 => local.get 0
                Instruction::LocalGet(0u32),
                // 0x22 0x03 => local.tee 3
                Instruction::LocalTee(3u32),
                // 0x20, 0x02 => local.get 2
                Instruction::LocalGet(2u32),
                // 0x6a => i32.add
                Instruction::I32Add,
                // 0x21 0x00 => local.set 0
                Instruction::LocalSet(0u32),
                // 0x20, 0x03 => local.get 3
                Instruction::LocalGet(3u32),
                // 0x21 0x02 => local.set 2
                Instruction::LocalSet(2u32),
                // 0x20, 0x01 => local.get 1
                Instruction::LocalGet(1u32),
                // 0x41, 0x7f => i32.const -1
                Instruction::I32Const(-1i32),
                // 0x6a => i32.add
                Instruction::I32Add,
                // 0x22 0x01 => local.tee 1
                Instruction::LocalTee(1u32),
                // 0x0d, 0x00 => br_if 0
                Instruction::BrIf(0u32),
                // 0x0b => end
                Instruction::End,
                // 0x0b => end
                Instruction::End,
                // 0x20, 0x00 => local.get 0
                Instruction::LocalGet(0u32),
                // 0x0b => end
                Instruction::End,
            ],
            instructions.entries(),
        );
    }
}
