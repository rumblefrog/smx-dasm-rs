use std::io::{Cursor, Seek, SeekFrom};
use byteorder::{ReadBytesExt, LittleEndian};
use crate::sections::{BaseSection, SMXNameTable};
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

#[derive(Debug, Clone)]
pub struct SMXRTTIEnumTable {
    enums: Vec<String>,
}

impl SMXRTTIEnumTable {
    pub fn new(header: &SMXHeader, section: &SectionEntry, names: &mut SMXNameTable) -> Result<Self> {
        let base = BaseSection::new(header, section);    
        let mut rtti = SMXRTTIListTable::new(header, section);

        let data = base.get_data();

        rtti.init(&data)?;

        let mut enums: Vec<String> = Vec::with_capacity(rtti.row_count() as usize);

        let mut data = Cursor::new(data);

        for _ in 0..rtti.row_count() {
            let index = data.read_i32::<LittleEndian>()?;

            enums.push(names.string_at(index)?);

            // reserved0-2.
            data.seek(SeekFrom::Current(3 * 4))?;
        }

        Ok(Self {
            enums,
        })
    }

    pub fn enums(&self) -> Vec<String> {
        self.enums.clone()
    }
}

#[derive(Debug, Clone)]
pub struct RTTIMethod {
    name: String,

    pcode_start: i32,

    pcode_end: i32,

    signature: i32,
}

#[derive(Debug, Clone)]
pub struct SMXRTTIMethodTable {
    methods: Vec<RTTIMethod>,
}

impl SMXRTTIMethodTable {
    pub fn new(header: &SMXHeader, section: &SectionEntry, names: &mut SMXNameTable) -> Result<Self> {
        let base = BaseSection::new(header, section);    
        let mut rtti = SMXRTTIListTable::new(header, section);

        let data = base.get_data();

        rtti.init(&data)?;

        let mut methods: Vec<RTTIMethod> = Vec::with_capacity(rtti.row_count() as usize);

        let mut data = Cursor::new(data);

        for _ in 0..rtti.row_count() {
            let index = data.read_i32::<LittleEndian>()?;

            methods.push(RTTIMethod {
                name: names.string_at(index)?,
                pcode_start: data.read_i32::<LittleEndian>()?,
                pcode_end: data.read_i32::<LittleEndian>()?,
                signature: data.read_i32::<LittleEndian>()?,
            });
        }

        Ok(Self {
            methods,
        })
    }

    pub fn methods(&self) -> Vec<RTTIMethod> {
        self.methods.clone()
    }
}

#[derive(Debug, Clone)]
pub struct RTTINative {
    name: String,

    signature: i32,
}

#[derive(Debug, Clone)]
pub struct SMXRTTINativeTable {
    natives: Vec<RTTINative>,
}

impl SMXRTTINativeTable {
    pub fn new(header: &SMXHeader, section: &SectionEntry, names: &mut SMXNameTable) -> Result<Self> {
        let base = BaseSection::new(header, section);    
        let mut rtti = SMXRTTIListTable::new(header, section);

        let data = base.get_data();

        rtti.init(&data)?;

        let mut natives: Vec<RTTINative> = Vec::with_capacity(rtti.row_count() as usize);

        let mut data = Cursor::new(data);

        for _ in 0..rtti.row_count() {
            let index = data.read_i32::<LittleEndian>()?;

            natives.push(RTTINative {
                name: names.string_at(index)?,
                signature: data.read_i32::<LittleEndian>()?,
            });
        }

        Ok(Self {
            natives,
        })
    }

    pub fn natives(&self) -> Vec<RTTINative> {
        self.natives.clone()
    }
}

#[derive(Debug, Clone)]
pub struct RTTITypedef {
    name: String,

    type_id: i32,
}

#[derive(Debug, Clone)]
pub struct SMXRTTITypedefTable {
    typedefs: Vec<RTTITypedef>,
}

impl SMXRTTITypedefTable {
    pub fn new(header: &SMXHeader, section: &SectionEntry, names: &mut SMXNameTable) -> Result<Self> {
        let base = BaseSection::new(header, section);    
        let mut rtti = SMXRTTIListTable::new(header, section);

        let data = base.get_data();

        rtti.init(&data)?;

        let mut typedefs: Vec<RTTITypedef> = Vec::with_capacity(rtti.row_count() as usize);

        let mut data = Cursor::new(data);

        for _ in 0..rtti.row_count() {
            let index = data.read_i32::<LittleEndian>()?;

            typedefs.push(RTTITypedef {
                name: names.string_at(index)?,
                type_id: data.read_i32::<LittleEndian>()?,
            });
        }

        Ok(Self {
            typedefs,
        })
    }

    pub fn typedefs(&self) -> Vec<RTTITypedef> {
        self.typedefs.clone()
    }
}

#[derive(Debug, Clone)]
pub struct RTTITypeset {
    name: String,

    signature: i32,
}

#[derive(Debug, Clone)]
pub struct SMXRTTITypesetTable {
    typesets: Vec<RTTITypeset>,
}

impl SMXRTTITypesetTable {
    pub fn new(header: &SMXHeader, section: &SectionEntry, names: &mut SMXNameTable) -> Result<Self> {
        let base = BaseSection::new(header, section);    
        let mut rtti = SMXRTTIListTable::new(header, section);

        let data = base.get_data();

        rtti.init(&data)?;

        let mut typesets: Vec<RTTITypeset> = Vec::with_capacity(rtti.row_count() as usize);

        let mut data = Cursor::new(data);

        for _ in 0..rtti.row_count() {
            let index = data.read_i32::<LittleEndian>()?;

            typesets.push(RTTITypeset {
                name: names.string_at(index)?,
                signature: data.read_i32::<LittleEndian>()?,
            });
        }

        Ok(Self {
            typesets,
        })
    }

    pub fn typesets(&self) -> Vec<RTTITypeset> {
        self.typesets.clone()
    }
}

#[derive(Debug, Clone)]
pub struct RTTIEnumStruct {
    name_offset: i32,

    first_field: i32,

    size: i32,

    name: String,
}

#[derive(Debug, Clone)]
pub struct SMXRTTIEnumStructTable {
    entries: Vec<RTTIEnumStruct>,
}

impl SMXRTTIEnumStructTable {
    pub fn new(header: &SMXHeader, section: &SectionEntry, names: &mut SMXNameTable) -> Result<Self> {
        let base = BaseSection::new(header, section);    
        let mut rtti = SMXRTTIListTable::new(header, section);

        let data = base.get_data();

        rtti.init(&data)?;

        let mut entries: Vec<RTTIEnumStruct> = Vec::with_capacity(rtti.row_count() as usize);

        let mut data = Cursor::new(data);

        for _ in 0..rtti.row_count() {
            let name_offset = data.read_i32::<LittleEndian>()?;
            let first_field = data.read_i32::<LittleEndian>()?;
            let size = data.read_i32::<LittleEndian>()?;
            let name = names.string_at(name_offset)?;

            entries.push(RTTIEnumStruct {
                name_offset,
                first_field,
                size,
                name,
            })
        }

        Ok(Self {
            entries,
        })
    }

    pub fn entries(&self) -> Vec<RTTIEnumStruct> {
        self.entries.clone()
    }
}

#[derive(Debug, Clone)]
pub struct RTTIEnumStructField {
    name_offset: i32,

    type_id: i32,

    offset: i32,

    name: String,
}

#[derive(Debug, Clone)]
pub struct SMXRTTIEnumStructFieldTable {
    entries: Vec<RTTIEnumStructField>,
}

impl SMXRTTIEnumStructFieldTable {
    pub fn new(header: &SMXHeader, section: &SectionEntry, names: &mut SMXNameTable) -> Result<Self> {
        let base = BaseSection::new(header, section);    
        let mut rtti = SMXRTTIListTable::new(header, section);

        let data = base.get_data();

        rtti.init(&data)?;

        let mut entries: Vec<RTTIEnumStructField> = Vec::with_capacity(rtti.row_count() as usize);

        let mut data = Cursor::new(data);

        for _ in 0..rtti.row_count() {
            let name_offset = data.read_i32::<LittleEndian>()?;
            let type_id = data.read_i32::<LittleEndian>()?;
            let offset = data.read_i32::<LittleEndian>()?;
            let name = names.string_at(name_offset)?;

            entries.push(RTTIEnumStructField {
                name_offset,
                type_id,
                offset,
                name,
            })
        }

        Ok(Self {
            entries,
        })
    }

    pub fn entries(&self) -> Vec<RTTIEnumStructField> {
        self.entries.clone()
    }
}

#[derive(Debug, Clone)]
pub struct RTTIClassDef {
    flags: i32,

    name_offset: i32,

    first_field: i32,

    name: String,
}

#[derive(Debug, Clone)]
pub struct SMXRTTIClassDefTable {
    defs: Vec<RTTIClassDef>,
}

impl SMXRTTIClassDefTable {
    pub fn new(header: &SMXHeader, section: &SectionEntry, names: &mut SMXNameTable) -> Result<Self> {
        let base = BaseSection::new(header, section);    
        let mut rtti = SMXRTTIListTable::new(header, section);

        let data = base.get_data();

        rtti.init(&data)?;

        let mut defs: Vec<RTTIClassDef> = Vec::with_capacity(rtti.row_count() as usize);

        let mut data = Cursor::new(data);

        for _ in 0..rtti.row_count() {
            let flags = data.read_i32::<LittleEndian>()?;
            let name_offset = data.read_i32::<LittleEndian>()?;
            let first_field = data.read_i32::<LittleEndian>()?;
            let name = names.string_at(name_offset)?;

            defs.push(RTTIClassDef {
                flags,
                name_offset,
                first_field,
                name,
            });

            // reserved0-3
            data.seek(SeekFrom::Current(4 * 4))?;
        }

        Ok(Self {
            defs,
        })
    }

    pub fn defs(&self) -> Vec<RTTIClassDef> {
        self.defs.clone()
    }
}

#[derive(Debug, Clone)]
pub struct RTTIField {
    flags: i32,

    name_offset: i32,

    type_id: i32,

    name: String,
}

#[derive(Debug, Clone)]
pub struct SMXRTTIFieldTable {
    fields: Vec<RTTIField>,
}

impl SMXRTTIFieldTable {
    pub fn new(header: &SMXHeader, section: &SectionEntry, names: &mut SMXNameTable) -> Result<Self> {
        let base = BaseSection::new(header, section);    
        let mut rtti = SMXRTTIListTable::new(header, section);

        let data = base.get_data();

        rtti.init(&data)?;

        let mut fields: Vec<RTTIField> = Vec::with_capacity(rtti.row_count() as usize);

        let mut data = Cursor::new(data);

        for _ in 0..rtti.row_count() {
            let flags = data.read_i32::<LittleEndian>()?;
            let name_offset = data.read_i32::<LittleEndian>()?;
            let type_id = data.read_i32::<LittleEndian>()?;
            let name = names.string_at(name_offset)?;

            fields.push(RTTIField {
                flags,
                name_offset,
                type_id,
                name,
            });
        }

        Ok(Self {
            fields,
        })
    }

    pub fn fields(&self) -> Vec<RTTIField> {
        self.fields.clone()
    }
}