use std::io::{Read, Cursor};
use byteorder::{ReadBytesExt, LittleEndian};
use flate2::read::GzDecoder;
use crate::errors::{Result, Error};

pub enum CompressionType {
    CompressionNone,
    CompressionGZ,
}

impl From<u8> for CompressionType {
    fn from(byte: u8) -> Self {
        match byte {
            1 => Self::CompressionGZ,
            _ => Self::CompressionNone,
        }
    }
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
pub const HEADER_SIZE: i32 = 24;

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

    pub disk_size: i32,

    pub image_size: i32,

    // Number of named file secctions.
    pub section_count: u8,

    // Offset to the string table. Each string is null-terminated. The string
    // table is only used for strings related to parsing the container itself.
    // For SourcePawn, a separate ".names" section exists for Pawn-specific data.
    pub string_table_offset: i32,

    // Offset to where compression begins (explained above).
    pub data_offset: i32,

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

            if buf[0] == 0 { 
                break; 
            } else {
                str_vec.push(buf[0]);
            }
        }

        Ok(String::from_utf8_lossy(&str_vec[..]).into_owned())
    }
}

impl SMXHeader {
    pub fn new(data: Vec<u8>) -> Result<SMXHeader> {
        let mut data = Cursor::new(data);

        let magic = data.read_u64::<LittleEndian>()?;

        if magic != FILE_MAGIC {
            return Err(Error::InvalidMagic)
        }

        let version = data.read_u16::<LittleEndian>()?;

        let compression_type = CompressionType::from(data.read_u8()?);

        let disk_size = data.read_i32::<LittleEndian>()?;

        if disk_size < HEADER_SIZE {
            return Err(Error::InvalidSize)
        }

        let image_size = data.read_i32::<LittleEndian>()?;

        if image_size < HEADER_SIZE {
            return Err(Error::InvalidSize)
        }

        let section_count = data.read_u8()?;

        let string_table_offset = data.read_i32::<LittleEndian>()?;

        if string_table_offset < HEADER_SIZE {
            return Err(Error::InvalidOffset)
        }

        let data_offset = data.read_i32::<LittleEndian>()?;

        if data_offset < HEADER_SIZE {
            return Err(Error::InvalidOffset)
        }

        let mut p_data: Vec<u8> = Vec::with_capacity(image_size as usize);

        p_data.extend(&data.get_ref()[..HEADER_SIZE as usize]);

        match compression_type {
            CompressionType::CompressionNone => {
                p_data.extend(&data.get_ref()[HEADER_SIZE as usize..(image_size - HEADER_SIZE) as usize]);
            },
            CompressionType::CompressionGZ => {
                p_data.extend(&data.get_ref()[HEADER_SIZE as usize..(data_offset - HEADER_SIZE) as usize]);

                let mut decoder = GzDecoder::new(&data.get_ref()[(disk_size - data_offset) as usize..]);

                decoder.read_to_end(&mut p_data)?;
            }
        }

        let cloned_data = p_data.clone();

        let mut new_data = Cursor::new(p_data);

        let mut sections: Vec<SectionEntry> = Vec::with_capacity(section_count as usize);

        let mut found_dbg_section: bool = false;

        for _ in 0..section_count {
            let name_offset: i32;

            sections.push(SectionEntry{
                name_offset: {
                    name_offset = new_data.read_i32::<LittleEndian>()?;

                    if name_offset < 0 {
                        return Err(Error::OffsetOverflow)
                    }

                    name_offset
                },
                data_offset: {
                    let offset = new_data.read_i32::<LittleEndian>()?;

                    if offset < HEADER_SIZE {
                        return Err(Error::OffsetOverflow)
                    }

                    offset
                },
                size: {
                    let size = new_data.read_i32::<LittleEndian>()?;

                    if size < 0 {
                        return Err(Error::SizeOverflow)
                    }

                    size
                },
                name: {
                    let name = Cursor::new(&cloned_data[string_table_offset as usize + name_offset as usize..]).read_cstring()?;

                    if name == ".dbg.natives" {
                        found_dbg_section = true;
                    }

                    name
                }
            })
        }

        Ok(SMXHeader{
            magic: FILE_MAGIC,
            version: version,
            compression_type: compression_type,
            disk_size: disk_size,
            image_size: image_size,
            section_count: section_count,
            string_table_offset: string_table_offset,
            data_offset: data_offset,
            data: cloned_data,
            sections: sections,
            debug_packed: (version == SP1_VERSION_1_0) && !found_dbg_section,
        })
    }

    // fn string_at(&self, index: usize) -> Result<String> {
    //     let mut data = Cursor::new(&self.data[self.string_table_offset as usize + index..]);

    //     return data.read_cstring();
    // }
}

pub struct SectionEntry {
    // Offset into the string table.
    pub name_offset: i32,

    // Offset into the file for section contents.
    pub data_offset: i32,

    // Size of this section's contents.
    pub size: i32,

    // Computed (not present on disk).
    pub name: String,
}