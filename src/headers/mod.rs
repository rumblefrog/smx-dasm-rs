use std::io::{Read, Cursor};
use crate::errors::{Result};

pub enum CompressionType {
    CompressionNone,
    CompressionGZ,
}

pub const FILE_MAGIC: u64 = 0x53504646;

// File format version number.
//
// The major version bits (8-15) indicate a product number. Consumers
// should reject any version for a different product.
//
// The minor version bits (0-7) indicate a compatibility revision. Any
// version higher than the current version should be rejected.
pub const SP1_VERSION_1_0: u16 = 0x0101;
pub const SP1_VERSION_1_1: u16 = 0x0102;
pub const SP1_VERSION_MIN: u16 = SP1_VERSION_1_0;
pub const SP1_VERSION_MAX: u16 = SP1_VERSION_1_1;

// Size of the header.
pub const HEADER_SIZE: u8 = 24;

pub struct SMXHeader {
    pub magic: u64,

    pub version: u16,

    // Compression algorithm. If the file is not compressed, then imagesize and
    // disksize are the same value, and dataoffs is 0.
    //
    // The start of the compressed region is indicated by dataoffs. The length
    // of the compressed region is (disksize - dataoffs). The amount of memory
    // required to hold the decompressed bytes is (imagesize - dataoffs). The
    // compressed region should be expanded in-place. That is, bytes before
    // "dataoffs" should be retained, and the decompressed region should be
    // appended.
    //
    // |imagesize| is the amount of memory required to hold the entire container
    // in memory.
    //
    // Note: This scheme may seem odd. It's a combination of historical debt and
    // previously unspecified behavior. The original .amx file format contained
    // an on-disk structure that supported an endian-agnostic variable-length 
    // encoding of its data section, and this structure was loaded directly into
    // memory and used as the VM context. AMX Mod X later developed a container
    // format called ".amxx" as a "universal binary" for 32-bit and 64-bit
    // plugins. This format dropped compact encoding, but supported gzip. The
    // disksize/imagesize oddness made its way to this file format. When .smx
    // was created for SourceMod, it persisted even though AMX was dropped
    // entirely. So it goes.
    pub compression_type: CompressionType,

    pub disk_size: u64,

    pub image_size: u64,

    // Number of named file secctions.
    pub section_count: u64,

    // Offset to the string table. Each string is null-terminated. The string
    // table is only used for strings related to parsing the container itself.
    // For SourcePawn, a separate ".names" section exists for Pawn-specific data.
    pub string_table_offset: u64,

    // Offset to where compression begins (explained above).
    pub data_offset: u64,

    // The computed data buffer (which contains the header).
    pub data: Vec<u8>,

    pub sections: Vec<SectionEntry>,

    pub debug_packed: bool,
}

trait ReadCString {
    fn read_cstring(&mut self) -> Result<String>;
}

impl ReadCString for Cursor<&[u8]> {
    fn read_cstring(&mut self) -> Result<String> {
        let mut buf = [0; 1];
        let mut str_vec = Vec::with_capacity(256);
        loop {
            self.read(&mut buf)?;
            if buf[0] == 0 { break; } else { str_vec.push(buf[0]); }
        }
        Ok(String::from_utf8_lossy(&str_vec[..]).into_owned())
    }
}

impl SMXHeader {
    fn string_at(&self, index: usize) -> Result<String> {
        let mut data = Cursor::new(&self.data[self.string_table_offset as usize + index..]);

        return data.read_cstring();
    }
}

pub struct SectionEntry {
    // Offset into the string table.
    pub name_offset: u64,

    // Offset into the file for section contents.
    pub data_offset: u64,

    // Size of this section's contents.
    pub size: u64,

    // Computed (not present on disk).
    pub name: String,
}