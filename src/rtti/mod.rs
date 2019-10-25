use std::io::Cursor;
use byteorder::{ReadBytesExt, LittleEndian};
use crate::sections::BaseSection;
use crate::headers::{SMXHeader, SectionEntry};
use crate::errors::Result;

#[derive(Debug, Clone)]
pub struct SMXRTTIListTable<'b> {
    base: BaseSection<'b>,

    header_size: u32,

    row_size: u32,

    row_count: u32,
}

impl<'b> SMXRTTIListTable<'b> {
    pub fn new(header: &'b SMXHeader, section: &'b SectionEntry) -> Self {
        Self {
            base: BaseSection::new(header, section),
            header_size: 0,
            row_size: 0,
            row_count: 0,
        }
    }

    pub fn init<T>(&mut self, data: T) -> Result<&Self>
    where
        T: AsRef<[u8]>,
    {
        let mut cursor = Cursor::new(data);

        self.header_size = cursor.read_u32::<LittleEndian>()?;
        self.row_size = cursor.read_u32::<LittleEndian>()?;
        self.row_count = cursor.read_u32::<LittleEndian>()?;

        Ok(self)
    }

    pub fn header_size(&self) -> u32 {
        self.header_size
    }

    pub fn row_size(&self) -> u32 {
        self.row_size
    }

    pub fn row_count(&self) -> u32 {
        self.row_count
    }
}

pub struct CB;

impl CB {
    pub const BOOL: u8 = 0x01;
    pub const INT32: u8 = 0x06;
    pub const FLOAT32: u8 = 0x0c;
    pub const CHAR8: u8 = 0x0e;
    pub const ANY: u8 = 0x10;
    pub const TOPFUNCTION: u8 = 0x11;

    pub const FIXEDARRAY: u8 = 0x30;
    pub const ARRAY: u8 = 0x31;
    pub const FUNCTION: u8 = 0x32;

    pub const ENUM: u8 = 0x42;
    pub const TYPEDEF: u8 = 0x43;
    pub const TYPESET: u8 = 0x44;
    pub const STRUCT: u8 = 0x45;
    pub const ENUMSTRUCT: u8 = 0x46;

    pub const VOID: u8 = 0x70;
    pub const VARIADIC: u8 = 0x71;
    pub const BYREF: u8 = 0x72;
    pub const CONST: u8 = 0x73;

    pub const TYPEID_INLINE: u8 = 0x0;
    pub const TYPEID_COMPLEX: u8 = 0x1;

    pub fn decode_u32<T>(bytes: T, offset: &mut i32) -> i32
    where
        T: AsRef<[u8]>,
    {
        let bytes = Cursor::new(bytes);

        let mut value: u32 = 0;
        let mut shift: i32 = 0;

        loop {
            let b: u8 = bytes.get_ref().as_ref()[*offset as usize];
            *offset += 1;
            value |= ((b & 0x7f) << shift) as u32;
            if (b & 0x80) == 0 {
                break;
            }
            shift += 7;
        }

        value as i32
    }
}