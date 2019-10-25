use std::io::{Cursor};
use byteorder::{ReadBytesExt, LittleEndian};
use crate::headers::{SectionEntry};
use crate::sections::{SMXNameTable};
use crate::errors::{Result, Error};

#[derive(Debug, Clone)]
pub enum CodeV1Flags {
    Debug,
}

impl CodeV1Flags {
    pub fn value(&self) -> u16 {
        match *self {
            CodeV1Flags::Debug => 0x0000_0001,
        }
    }
}

// The ".code" section.
#[derive(Debug, Clone)]
pub struct CodeV1Header {
    // Size of the code blob.
    pub code_size: i32,

    // Size of a cell in bytes (always 4).
    pub cell_size: u8,

    // Code version (see above constants).
    pub code_version: u8,

    // Flags (see above).
    pub flags: u16,

    // Offset within the code blob to the entry point function.
    pub main_offset: i32,

    // Offset to the code section.
    pub code_offset: i32,

    // Feature set.
    pub features: i32,
}

impl CodeV1Header {
    pub const SIZE: i32 = 16;

    pub const VERSION_JIT1: u8 = 9;
    pub const VERSION_JIT2: u8 = 10;

    pub fn new(data: Vec<u8>) -> Result<Self> {
        let mut cursor = Cursor::new(data);

        let mut read_features: bool = false;

        Ok(Self {
            code_size: cursor.read_i32::<LittleEndian>()?,
            cell_size: cursor.read_u8()?,
            code_version: {
                let code_version = cursor.read_u8()?;

                if code_version >= 13 {
                    read_features = true;
                }

                code_version
            },
            flags: cursor.read_u16::<LittleEndian>()?,
            main_offset: cursor.read_i32::<LittleEndian>()?,
            code_offset: cursor.read_i32::<LittleEndian>()?,
            features: {
                if read_features {
                    cursor.read_i32::<LittleEndian>()?;
                }

                0
            }
        })
    }
}

#[derive(Debug, Clone)]
pub struct DataHeader {
    // Size of the data blob.
    pub data_size: u32,

    // Amount of memory the plugin runtime requires.
    pub memory_size: u32,

    // Offset within this section to the data blob.
    pub data_offset: u32,
}

impl DataHeader {
    pub const SIZE: i32 = 12;

    pub fn new(data: Vec<u8>) -> Result<Self> {
        let mut cursor = Cursor::new(data);

        Ok(Self {
            data_size: cursor.read_u32::<LittleEndian>()?,
            memory_size: cursor.read_u32::<LittleEndian>()?,
            data_offset: cursor.read_u32::<LittleEndian>()?,
        })
    }
}

#[derive(Debug, Clone)]
pub struct PublicEntry {
    // Offset into the code section.
    pub address: u32,

    // Offset into the .names section.
    pub name_offset: i32,

    // Computed.
    pub name: String,
}

impl PublicEntry {
    pub const SIZE: i32 = 8;

    pub fn new(data: Vec<u8>, section: SectionEntry, mut names: SMXNameTable) -> Result<Vec<Self>> {
        if section.size & PublicEntry::SIZE != 0 {
            return Err(Error::InvalidSize)
        }

        let count: usize = (section.size / PublicEntry::SIZE) as usize;

        let mut entries: Vec<Self> = Vec::with_capacity(count);

        let mut cursor = Cursor::new(data);

        for _ in 0..count {
            let name_offset: i32;

            entries.push(PublicEntry {
                address: cursor.read_u32::<LittleEndian>()?,
                name_offset: {
                    name_offset = cursor.read_i32::<LittleEndian>()?;

                    name_offset
                },
                name: names.string_at(&name_offset)?,
            })
        }

        Ok(entries)
    }
}

pub struct CalledFunctionEntry {
    pub address: u32,

    pub name: String,
}
