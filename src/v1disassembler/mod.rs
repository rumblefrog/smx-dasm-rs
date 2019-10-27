use std::io::{Cursor, Seek, SeekFrom};
use byteorder::{ReadBytesExt, LittleEndian};
use std::convert::TryFrom;
use crate::errors::{Result, Error};
use crate::file::SMXFile;
use crate::v1opcodes::*;
use crate::sections::{SMXCodeV1Section};

#[derive(Clone)]
pub enum V1Param {
    Constant,
    Stack,
    Jump,
    Function,
    Native,
    Address,
}

#[derive(Clone, Default)]
pub struct V1OPCodeInfo{
    pub opcode: V1OPCode,
    pub name: String,
    pub params: Vec<V1Param>,
}

#[derive(Clone)]
pub struct V1Instruction {
    pub address: i32,
    pub info: V1OPCodeInfo,
    pub params: Vec<i32>,
}

lazy_static! {
    static ref opcode_list: Vec<V1OPCodeInfo> = {
        let mut m = Vec::with_capacity(V1OPCode::TOTAL_OPCODES as usize);

        let mut prep = |op: V1OPCode, params: &'static [V1Param]| {
            let name: String = (&op).to_string().replace("_", ".").to_lowercase();

            m.push(V1OPCodeInfo {
                opcode: op,
                name,
                params: params.to_vec(),
            });
        };

        prep(V1OPCode::ADD, &[]);
        prep(V1OPCode::ADD_C, &[V1Param::Constant]);
        prep(V1OPCode::ADDR_ALT, &[V1Param::Stack]);
        prep(V1OPCode::ADDR_PRI, &[V1Param::Stack]);
        prep(V1OPCode::AND, &[]);
        prep(V1OPCode::BOUNDS, &[V1Param::Constant]);
        prep(V1OPCode::BREAK, &[]);
        prep(V1OPCode::CALL, &[V1Param::Function]);
        prep(V1OPCode::CASETBL, &[V1Param::Constant, V1Param::Address]);
        prep(V1OPCode::CONST, &[V1Param::Address, V1Param::Constant]);
        prep(V1OPCode::CONST_ALT, &[V1Param::Constant]);
        prep(V1OPCode::CONST_PRI, &[V1Param::Constant]);
        prep(V1OPCode::CONST_S, &[V1Param::Stack, V1Param::Constant]);
        prep(V1OPCode::DEC, &[V1Param::Address]);
        prep(V1OPCode::DEC_ALT, &[]);
        prep(V1OPCode::DEC_I, &[]);
        prep(V1OPCode::DEC_PRI, &[]);
        prep(V1OPCode::DEC_S, &[V1Param::Stack]);
        prep(V1OPCode::EQ, &[]);
        prep(V1OPCode::EQ_C_ALT, &[V1Param::Constant]);
        prep(V1OPCode::EQ_C_PRI, &[V1Param::Constant]);
        prep(V1OPCode::FILL, &[V1Param::Constant]);
        prep(V1OPCode::GENARRAY, &[V1Param::Constant]);
        prep(V1OPCode::GENARRAY_Z, &[V1Param::Constant]);
        prep(V1OPCode::HALT, &[V1Param::Constant]);
        prep(V1OPCode::HEAP, &[V1Param::Constant]);
        prep(V1OPCode::IDXADDR, &[]);
        prep(V1OPCode::IDXADDR_B, &[V1Param::Constant]);
        prep(V1OPCode::INC, &[V1Param::Address]);
        prep(V1OPCode::INC_ALT, &[]);
        prep(V1OPCode::INC_I, &[]);
        prep(V1OPCode::INC_PRI, &[]);
        prep(V1OPCode::INC_S, &[V1Param::Stack]);
        prep(V1OPCode::INVERT, &[]);
        prep(V1OPCode::JEQ, &[V1Param::Jump]);
        prep(V1OPCode::JNEQ, &[V1Param::Jump]);
        prep(V1OPCode::JNZ, &[V1Param::Jump]);
        prep(V1OPCode::JSGEQ, &[V1Param::Jump]);
        prep(V1OPCode::JSGRTR, &[V1Param::Jump]);
        prep(V1OPCode::JSLEQ, &[V1Param::Jump]);
        prep(V1OPCode::JSLESS, &[V1Param::Jump]);
        prep(V1OPCode::JUMP, &[V1Param::Jump]);
        prep(V1OPCode::JZER, &[V1Param::Jump]);
        prep(V1OPCode::LIDX, &[]);
        prep(V1OPCode::LIDX_B, &[V1Param::Constant]);
        prep(V1OPCode::LOAD_ALT, &[V1Param::Constant]);
        prep(V1OPCode::LOAD_BOTH, &[V1Param::Constant, V1Param::Constant]);
        prep(V1OPCode::LOAD_I, &[]);
        prep(V1OPCode::LOAD_PRI, &[V1Param::Constant]);
        prep(V1OPCode::LOAD_S_ALT, &[V1Param::Stack]);
        prep(V1OPCode::LOAD_S_BOTH, &[V1Param::Stack, V1Param::Stack]);
        prep(V1OPCode::LOAD_S_PRI, &[V1Param::Stack]);
        prep(V1OPCode::LODB_I, &[V1Param::Constant]);
        prep(V1OPCode::LREF_S_ALT, &[V1Param::Stack]);
        prep(V1OPCode::LREF_S_PRI, &[V1Param::Stack]);
        prep(V1OPCode::MOVE_ALT, &[]);
        prep(V1OPCode::MOVE_PRI, &[]);
        prep(V1OPCode::MOVS, &[V1Param::Constant]);
        prep(V1OPCode::NEG, &[]);
        prep(V1OPCode::NEQ, &[]);
        prep(V1OPCode::NOP, &[]);
        prep(V1OPCode::NOT, &[]);
        prep(V1OPCode::OR, &[]);
        prep(V1OPCode::POP_ALT, &[]);
        prep(V1OPCode::POP_PRI, &[]);
        prep(V1OPCode::PROC, &[]);
        prep(V1OPCode::PUSH_ALT, &[]);
        prep(V1OPCode::PUSH_PRI, &[]);
        prep(V1OPCode::PUSH, &[V1Param::Address]);
        prep(V1OPCode::PUSH2, &[V1Param::Address, V1Param::Address]);
        prep(V1OPCode::PUSH3, &[V1Param::Address, V1Param::Address, V1Param::Address]);
        prep(V1OPCode::PUSH4, &[V1Param::Address, V1Param::Address, V1Param::Address, V1Param::Address]);
        prep(V1OPCode::PUSH5, &[V1Param::Address, V1Param::Address, V1Param::Address, V1Param::Address, V1Param::Address]);
        prep(V1OPCode::PUSH_C, &[V1Param::Constant]);
        prep(V1OPCode::PUSH2_C, &[V1Param::Constant, V1Param::Constant]);
        prep(V1OPCode::PUSH3_C, &[V1Param::Constant, V1Param::Constant, V1Param::Constant]);
        prep(V1OPCode::PUSH4_C, &[V1Param::Constant, V1Param::Constant, V1Param::Constant, V1Param::Constant]);
        prep(V1OPCode::PUSH5_C, &[V1Param::Constant, V1Param::Constant, V1Param::Constant, V1Param::Constant, V1Param::Constant]);
        prep(V1OPCode::PUSH_S, &[V1Param::Stack]);
        prep(V1OPCode::PUSH2_S, &[V1Param::Stack, V1Param::Stack]);
        prep(V1OPCode::PUSH3_S, &[V1Param::Stack, V1Param::Stack, V1Param::Stack]);
        prep(V1OPCode::PUSH4_S, &[V1Param::Stack, V1Param::Stack, V1Param::Stack, V1Param::Stack]);
        prep(V1OPCode::PUSH5_S, &[V1Param::Stack, V1Param::Stack, V1Param::Stack, V1Param::Stack, V1Param::Stack]);
        prep(V1OPCode::PUSH_ADR, &[V1Param::Stack]);
        prep(V1OPCode::PUSH2_ADR, &[V1Param::Stack, V1Param::Stack]);
        prep(V1OPCode::PUSH3_ADR, &[V1Param::Stack, V1Param::Stack, V1Param::Stack]);
        prep(V1OPCode::PUSH4_ADR, &[V1Param::Stack, V1Param::Stack, V1Param::Stack, V1Param::Stack]);
        prep(V1OPCode::PUSH5_ADR, &[V1Param::Stack, V1Param::Stack, V1Param::Stack, V1Param::Stack, V1Param::Stack]);
        prep(V1OPCode::RETN, &[]);
        prep(V1OPCode::SDIV, &[]);
        prep(V1OPCode::SDIV_ALT, &[]);
        prep(V1OPCode::SGEQ, &[]);
        prep(V1OPCode::SGRTR, &[]);
        prep(V1OPCode::SHL, &[]);
        prep(V1OPCode::SHL_C_ALT, &[V1Param::Constant]);
        prep(V1OPCode::SHL_C_PRI, &[V1Param::Constant]);
        prep(V1OPCode::SHR, &[]);
        prep(V1OPCode::SHR_C_ALT, &[V1Param::Constant]);
        prep(V1OPCode::SHR_C_PRI, &[V1Param::Constant]);
        prep(V1OPCode::SLEQ, &[]);
        prep(V1OPCode::SLESS, &[]);
        prep(V1OPCode::SMUL, &[]);
        prep(V1OPCode::SMUL_C, &[V1Param::Constant]);
        prep(V1OPCode::SREF_S_ALT, &[V1Param::Stack]);
        prep(V1OPCode::SREF_S_PRI, &[V1Param::Stack]);
        prep(V1OPCode::SSHR, &[]);
        prep(V1OPCode::STACK, &[V1Param::Constant]);
        prep(V1OPCode::STOR_ALT, &[V1Param::Constant]);
        prep(V1OPCode::STOR_I, &[]);
        prep(V1OPCode::STOR_PRI, &[V1Param::Constant]);
        prep(V1OPCode::STOR_S_ALT, &[V1Param::Stack]);
        prep(V1OPCode::STOR_S_PRI, &[V1Param::Stack]);
        prep(V1OPCode::STRADJUST_PRI, &[]);
        prep(V1OPCode::STRB_I, &[V1Param::Constant]);
        prep(V1OPCode::SUB, &[]);
        prep(V1OPCode::SUB_ALT, &[]);
        prep(V1OPCode::SWAP_ALT, &[]);
        prep(V1OPCode::SWAP_PRI, &[]);
        prep(V1OPCode::SWITCH, &[V1Param::Address]);
        prep(V1OPCode::SYSREQ_C, &[V1Param::Native]);
        prep(V1OPCode::SYSREQ_N, &[V1Param::Native, V1Param::Constant]);
        prep(V1OPCode::TRACKER_POP_SETHEAP, &[]);
        prep(V1OPCode::TRACKER_PUSH_C, &[V1Param::Constant]);
        prep(V1OPCode::XCHG, &[]);
        prep(V1OPCode::XOR, &[]);
        prep(V1OPCode::ZERO, &[V1Param::Address]);
        prep(V1OPCode::ZERO_ALT, &[]);
        prep(V1OPCode::ZERO_PRI, &[]);
        prep(V1OPCode::ZERO_S, &[V1Param::Stack]);
        prep(V1OPCode::REBASE, &[V1Param::Address, V1Param::Constant, V1Param::Constant]);

        m
    };
}

static mut populated: bool = false;

pub struct V1Disassembler<'a> {
    file: &'a SMXFile<'a>,
    data: Vec<u8>,
    code_start: i32,
    proc_offset: i32,
    cursor: i32,
    cursor_limit: i32,
}

impl<'a> V1Disassembler<'a> {
    pub fn new(file: &'a SMXFile<'a>, code: &'a  SMXCodeV1Section, proc_offset: i32) -> Self {
        Self {
            file,
            data: file.header.data.clone(),
            code_start: code.code_start(),
            proc_offset,
            cursor: proc_offset,
            cursor_limit: code.header().code_size,
        }
    }

    fn read_at(&self, offset: i32) -> Result<i32> {
        let mut cursor = Cursor::new(&self.data);

        cursor.seek(SeekFrom::Start((self.code_start + offset) as u64));

        Ok(cursor.read_i32::<LittleEndian>()?)
    }

    fn read_next(&mut self) -> Result<i32> {
        let value: i32 = self.read_at(self.cursor)?;
        self.cursor += 4;
        Ok(value)
    }

    fn read_next_op(&mut self) -> Result<V1OPCode> {
        Ok(V1OPCode::try_from(self.read_next()? as u8).unwrap())
    }

    fn diassemble_internal(&mut self) -> Result<Vec<V1Instruction>> {
        if self.read_next_op()? != V1OPCode::PROC {
            return Err(Error::Other("Function does not start with PROC"))
        }

        let mut insns: Vec<V1Instruction> = Vec::new();

        while self.cursor < self.cursor_limit {
            let address: i32 = self.cursor;

            let op: i32 = self.read_next()?;

            if op == V1OPCode::PROC as i32 || op == V1OPCode::ENDPROC as i32 {
                break;
            }

            let mut insn: V1Instruction = V1Instruction {
                address,
                info: opcode_list[op as usize].clone(),
                params: Vec::new(),
            };

            if op == V1OPCode::CASETBL as i32 {
                let ncases: i32 = self.read_next()?;

                insn.params.resize(((ncases + 1) * 2) as usize, 0);

                insn.params[0] = ncases;
                insn.params[1] = self.read_next()?;

                for i in 0..ncases {
                    insn.params[(2 + i * 2) as usize] = self.read_next()?;
                    insn.params[(2 + i * 2 + 1) as usize] = self.read_next()?;
                }

                insns.push(insn);

                continue;
            }

            insn.params.resize(insn.info.params.len(), 0);

            for i in 0..insn.info.params.len() {
                insn.params[i] = self.read_next()?;
            }

            if op == V1OPCode::CALL as i32 {
                let addr: i32 = insn.params[0];

                if !self.file.is_function_at_address(addr) {
                    self.file.called_functions.as_mut().unwrap().add_function(addr as u32);
                }
            }

            insns.push(insn);
        }

        Ok(insns)
    }

    pub fn diassemble(file: &'a SMXFile<'a>, code: &'a SMXCodeV1Section, proc_offset: i32) -> Result<Vec<V1Instruction>> {
        let mut disassembler: V1Disassembler = V1Disassembler::new(file, code, proc_offset);

        disassembler.diassemble_internal()
    }
}