use std::io::{Read, Seek, SeekFrom, Cursor};
use byteorder::{ReadBytesExt, LittleEndian};
use flate2::read::ZlibDecoder;
use std::fmt;
use crate::errors::{Result, Error};

#[derive(Debug, Clone)]
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

impl fmt::Display for CompressionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CompressionType::CompressionGZ => writeln!(f, "GZip"),
            CompressionType::CompressionNone => writeln!(f, "None"),
        }
    }
}

#[derive(Clone)]
pub struct SMXHeader {
    pub magic: u32,

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
            self.read_exact(&mut buf)?;

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
    // SourcePawn File Format magic number.
    pub const FILE_MAGIC: u32 = 0x5350_4646;

    // File format version number.
    //
    // The major version bits (8-15) indicate a product number. Consumers
    // should reject any version for a different product.
    //
    // The minor version bits (0-7) indicate a compatibility revision. Any
    // version higher than the current version should be rejected.
    pub const SP1_VERSION_1_0: u16 = 0x0101;
    pub const SP1_VERSION_1_1: u16 = 0x0102;
    pub const SP1_VERSION_MIN: u16 = SMXHeader::SP1_VERSION_1_0;
    pub const SP1_VERSION_MAX: u16 = SMXHeader::SP1_VERSION_1_1;

    // Size of the header.
    const HEADER_SIZE: i32 = 24;

    pub fn new<T>(data: T) -> Result<SMXHeader>
    where
        T: AsRef<[u8]>,
    {
        let mut data = Cursor::new(data);

        let magic = data.read_u32::<LittleEndian>()?;

        if magic != SMXHeader::FILE_MAGIC {
            return Err(Error::InvalidMagic)
        }

        let version = data.read_u16::<LittleEndian>()?;

        let compression_type = CompressionType::from(data.read_u8()?);

        let disk_size = data.read_i32::<LittleEndian>()?;

        if disk_size < SMXHeader::HEADER_SIZE {
            return Err(Error::InvalidSize)
        }

        let image_size = data.read_i32::<LittleEndian>()?;

        if image_size < SMXHeader::HEADER_SIZE {
            return Err(Error::InvalidSize)
        }

        let section_count = data.read_u8()?;

        let string_table_offset = data.read_i32::<LittleEndian>()?;

        if string_table_offset < SMXHeader::HEADER_SIZE {
            return Err(Error::InvalidOffset)
        }

        let data_offset = data.read_i32::<LittleEndian>()?;

        if data_offset < SMXHeader::HEADER_SIZE {
            return Err(Error::InvalidOffset)
        }

        let mut p_data: Vec<u8> = Vec::with_capacity(image_size as usize);

        p_data.extend(&data.get_ref().as_ref()[..SMXHeader::HEADER_SIZE as usize]);

        match compression_type {
            CompressionType::CompressionNone => {
                p_data.extend(&data.get_ref().as_ref()[SMXHeader::HEADER_SIZE as usize..image_size as usize]);
            },
            CompressionType::CompressionGZ => {
                p_data.extend(&data.get_ref().as_ref()[SMXHeader::HEADER_SIZE as usize..data_offset as usize]);

                let mut decoder = ZlibDecoder::new(&data.get_ref().as_ref()[data_offset as usize..]);

                decoder.read_to_end(&mut p_data)?;
            }
        }

        let cloned_data = p_data.clone();

        let mut new_data = Cursor::new(p_data);

        new_data.seek(SeekFrom::Start(SMXHeader::HEADER_SIZE as u64))?;

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

                    if offset < SMXHeader::HEADER_SIZE {
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
                    let mut cursor = Cursor::new(&cloned_data[string_table_offset as usize + name_offset as usize..]);

                    let name = cursor.read_cstring()?;

                    if name == ".dbg.natives" {
                        found_dbg_section = true;
                    }

                    name
                }
            })
        }

        Ok(SMXHeader{
            magic: SMXHeader::FILE_MAGIC,
            version,
            compression_type,
            disk_size,
            image_size,
            section_count,
            string_table_offset,
            data_offset,
            data: cloned_data,
            sections,
            debug_packed: (version == SMXHeader::SP1_VERSION_1_0) && !found_dbg_section,
        })
    }

    // fn string_at(&self, index: usize) -> Result<String> {
    //     let mut data = Cursor::new(&self.data[self.string_table_offset as usize + index..]);

    //     return data.read_cstring();
    // }
}

impl fmt::Debug for SMXHeader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Magic: {}", self.magic)?;
        writeln!(f, "Version: {}", self.version)?;
        writeln!(f, "Compression Type: {}", self.compression_type)?;
        writeln!(f, "Disk Size: {}", self.disk_size)?;
        writeln!(f, "Image Size: {}", self.image_size)?;
        writeln!(f, "Section Count: {}", self.section_count)?;
        writeln!(f, "String Table Offset: {}", self.string_table_offset)?;
        writeln!(f, "Data Offset: {}", self.data_offset)?;
        writeln!(f, "Sections: {:?}", self.sections)?;
        writeln!(f, "Debug Packed: {}", self.debug_packed)
    }
}

#[derive(Debug, Clone)]
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
