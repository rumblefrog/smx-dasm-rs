use std::fmt;
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

    pub fn new<T>(data: T) -> Result<Self>
    where
        T: AsRef<[u8]>
    {
        let mut cursor = Cursor::new(data);

        let code_size = cursor.read_i32::<LittleEndian>()?;
        let cell_size = cursor.read_u8()?;
        let code_version = cursor.read_u8()?;
        let flags = cursor.read_u16::<LittleEndian>()?;
        let main_offset = cursor.read_i32::<LittleEndian>()?;
        let code_offset = cursor.read_i32::<LittleEndian>()?;

        Ok(Self {
            code_size,
            cell_size,
            code_version,
            flags,
            main_offset,
            code_offset,
            features: {
                if code_version >= 13 {
                    cursor.read_i32::<LittleEndian>()?;
                }

                0
            }
        })
    }
}

// The ".data" section.
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

    pub fn new<T>(data: T) -> Result<Self>
    where
        T: AsRef<[u8]>,
    {
        let mut cursor = Cursor::new(data);

        Ok(Self {
            data_size: cursor.read_u32::<LittleEndian>()?,
            memory_size: cursor.read_u32::<LittleEndian>()?,
            data_offset: cursor.read_u32::<LittleEndian>()?,
        })
    }
}

// The ".publics" section.
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

    pub fn new<T>(data: T, section: SectionEntry, mut names: SMXNameTable) -> Result<Vec<Self>>
    where
        T: AsRef<[u8]>,
    {
        if section.size & Self::SIZE != 0 {
            return Err(Error::InvalidSize)
        }

        let count: usize = (section.size / Self::SIZE) as usize;

        let mut entries: Vec<Self> = Vec::with_capacity(count);

        let mut cursor = Cursor::new(data);

        for _ in 0..count {
            let address = cursor.read_u32::<LittleEndian>()?;
            let name_offset = cursor.read_i32::<LittleEndian>()?;

            entries.push(Self {
                address,
                name_offset,
                name: names.string_at(name_offset)?,
            })
        }

        Ok(entries)
    }
}

pub struct CalledFunctionEntry {
    pub address: u32,

    pub name: String,
}

// The ".natives" section.
#[derive(Debug, Clone)]
pub struct NativeEntry {
    // Offset into the .names section.
    pub name_offset: i32,

    // Computed name.
    pub name: String,
}

impl NativeEntry {
    pub const SIZE: i32 = 4;

    pub fn new<T>(data: T, section: SectionEntry, mut names: SMXNameTable) -> Result<Vec<Self>>
    where
        T: AsRef<[u8]>,
    {
        if section.size & Self::SIZE != 0 {
            return Err(Error::InvalidSize)
        }

        let count: usize = (section.size / Self::SIZE) as usize;

        let mut entries: Vec<Self> = Vec::with_capacity(count);

        let mut cursor = Cursor::new(data);

        for _ in 0..count {
            let name_offset = cursor.read_i32::<LittleEndian>()?;

            entries.push(Self {
                name_offset,
                name: names.string_at(name_offset)?,
            })
        }

        Ok(entries)
    }
}

// The ".pubvars" section.
#[derive(Debug, Clone)]
pub struct PubvarEntry {
    // Offset into the data section.
    pub address: u32,

    // Offset into the .names section.
    pub name_offset: i32,

    // Computed name.
    pub name: String,
}

impl PubvarEntry {
    pub const SIZE: i32 = 8;

    pub fn new<T>(data: T, section: SectionEntry, mut names: SMXNameTable) -> Result<Vec<Self>>
    where
        T: AsRef<[u8]>,
    {
        if section.size & Self::SIZE != 0 {
            return Err(Error::InvalidSize)
        }

        let count: usize = (section.size / Self::SIZE) as usize;

        let mut entries: Vec<Self> = Vec::with_capacity(count);

        let mut cursor = Cursor::new(data);

        for _ in 0..count {
            let address = cursor.read_u32::<LittleEndian>()?;
            let name_offset = cursor.read_i32::<LittleEndian>()?;

            entries.push(Self {
                address,
                name_offset,
                name: names.string_at(name_offset)?,
            })
        }

        Ok(entries)
    }
}

// The ".tags" section.
#[derive(Debug, Clone)]
pub struct TagEntry {
    // Tag ID from the compiler.
    pub tag: u32,

    // Offset into the .names section.
    pub name_offset: i32,

    // Computed name.
    pub name: String,
}

impl TagEntry {
    pub const SIZE: i32 = 8;

    // Various tags that can be on a tag id.
    pub const FIXED: u32 = 0x40000000;
    pub const FUNC: u32 = 0x20000000;
    pub const OBJECT: u32 = 0x10000000;
    pub const ENUM: u32 = 0x08000000;
    pub const METHODMAP: u32 = 0x04000000;
    pub const STRUCT: u32 = 0x02000000;
    pub const FLAGMASK: u32 = 
        (Self::FIXED |
        Self:: FUNC |
        Self::OBJECT |
        Self::ENUM |
        Self::METHODMAP |
        Self::STRUCT);

    pub fn new<T>(data: T, section: SectionEntry, mut names: SMXNameTable) -> Result<Vec<Self>>
    where
        T: AsRef<[u8]>,
    {
        if section.size & NativeEntry::SIZE != 0 {
            return Err(Error::InvalidSize)
        }

        let count: usize = (section.size / NativeEntry::SIZE) as usize;

        let mut entries: Vec<Self> = Vec::with_capacity(count);

        let mut cursor = Cursor::new(data);

        for _ in 0..count {
            let tag = cursor.read_u32::<LittleEndian>()?;
            let name_offset = cursor.read_i32::<LittleEndian>()?;

            entries.push(Self {
                tag,
                name_offset,
                name: names.string_at(name_offset)?,
            })
        }

        Ok(entries)
    }
}

// The ".dbg.info" section.
#[derive(Debug, Clone)]
pub struct DebugInfoHeader {
    pub file_count: i32,

    pub line_count: i32,

    pub symbol_count: i32,

    pub array_count: i32,
}

impl DebugInfoHeader {
    pub fn new<T>(data: T) -> Result<Self>
    where
        T: AsRef<[u8]>,
    {
        let mut cursor = Cursor::new(data);

        Ok(Self {
            file_count: cursor.read_i32::<LittleEndian>()?,
            line_count: cursor.read_i32::<LittleEndian>()?,
            symbol_count: cursor.read_i32::<LittleEndian>()?,
            array_count: cursor.read_i32::<LittleEndian>()?,
        })
    }
}

// The ".dbg.files" section.
#[derive(Debug, Clone)]
pub struct DebugFileEntry {
    // Offset into the data section.
    pub address: u32,

    // Offset into the .names section.
    pub name_offset: i32,

    // Computed name.
    pub name: String,
}

impl DebugFileEntry {
    pub const SIZE: i32 = 8;

    pub fn new<T>(data: T, section: SectionEntry, mut names: SMXNameTable) -> Result<Vec<Self>>
    where
        T: AsRef<[u8]>,
    {
        if section.size & Self::SIZE != 0 {
            return Err(Error::InvalidSize)
        }

        let count: usize = (section.size / Self::SIZE) as usize;

        let mut entries: Vec<Self> = Vec::with_capacity(count);

        let mut cursor = Cursor::new(data);

        for _ in 0..count {
            let address = cursor.read_u32::<LittleEndian>()?;
            let name_offset = cursor.read_i32::<LittleEndian>()?;

            entries.push(Self {
                address,
                name_offset,
                name: names.string_at(name_offset)?,
            })
        }

        Ok(entries)
    }
}

// The ".dbg.lines" section.
#[derive(Debug, Clone)]
pub struct DebugLineEntry {
    // Offset into the data section.
    pub address: u32,

    // Line number.
    pub line: u32,
}

impl DebugLineEntry {
    pub const SIZE: i32 = 8;

    pub fn new<T>(data: T, section: SectionEntry) -> Result<Vec<Self>>
    where
        T: AsRef<[u8]>,
    {
        if section.size & Self::SIZE != 0 {
            return Err(Error::InvalidSize)
        }

        let count: usize = (section.size / Self::SIZE) as usize;

        let mut entries: Vec<Self> = Vec::with_capacity(count);

        let mut cursor = Cursor::new(data);

        for _ in 0..count {
            let address = cursor.read_u32::<LittleEndian>()?;
            let line = cursor.read_u32::<LittleEndian>()?;

            entries.push(Self {
                address,
                line,
            })
        }

        Ok(entries)
    }
}

#[derive(Debug, Clone)]
pub enum SymbolScope {
    Global,
    Local,
    Static,
    Arg,
    Unknown,
}

impl From<u8> for SymbolScope {
    fn from(s: u8) -> Self {
        match s {
            0 => Self::Global,
            1 => Self::Local,
            2 => Self::Static,
            3 => Self::Arg,
            _ => Self::Unknown,
        }
    }
}

impl fmt::Display for SymbolScope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SymbolScope::Global => writeln!(f, "Global"),
            SymbolScope::Local => writeln!(f, "Local"),
            SymbolScope::Static => writeln!(f, "Static"),
            SymbolScope::Arg => writeln!(f, "Arg"),
            SymbolScope::Unknown => writeln!(f, "Unknown"),
        }
    }
}

// The ".dbg.methods" section.
#[derive(Debug, Clone)]
pub struct DebugMethodEntry {
    pub method_index: i32,

    pub first_local: i32,
}

impl DebugMethodEntry {
    pub fn new<T>(data: T) -> Result<Self>
    where
        T: AsRef<[u8]>,
    {
        let mut cursor = Cursor::new(data);

        Ok(Self {
            method_index: cursor.read_i32::<LittleEndian>()?,
            first_local: cursor.read_i32::<LittleEndian>()?,
        })
    }
}

// The ".dbg.globals"  and ".dbg.locals" section.
#[derive(Debug, Clone)]
pub struct DebugVarEntry {
    pub address: i32,

    pub scope: SymbolScope,

    pub name_offset: i32,

    pub code_start: i32,

    pub code_end: i32,

    pub type_id: i32,
}

impl DebugVarEntry {
    pub fn new<T>(data: T) -> Result<Self>
    where
        T: AsRef<[u8]>,
    {
        let mut cursor = Cursor::new(data);

        Ok(Self {
            address: cursor.read_i32::<LittleEndian>()?,
            scope: SymbolScope::from(cursor.read_u8()?),
            name_offset: cursor.read_i32::<LittleEndian>()?,
            code_start: cursor.read_i32::<LittleEndian>()?,
            code_end: cursor.read_i32::<LittleEndian>()?,
            type_id: cursor.read_i32::<LittleEndian>()?,
        })
    }
}
