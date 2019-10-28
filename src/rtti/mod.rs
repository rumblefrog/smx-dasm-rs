use std::rc::Rc;
use std::cell::RefCell;
use std::io::{Cursor, Seek, SeekFrom};
use byteorder::{ReadBytesExt, LittleEndian};
use crate::sections::{BaseSection, SMXNameTable};
use crate::headers::{SMXHeader, SectionEntry};
use crate::file::SMXFile;
use crate::errors::Result;

#[derive(Debug, Clone)]
pub struct SMXRTTIListTable {
    base: BaseSection,

    header_size: u32,

    row_size: u32,

    row_count: u32,
}

impl SMXRTTIListTable {
    pub fn new(header: Rc<SMXHeader>, section: Rc<SectionEntry>) -> Self {
        Self {
            base: BaseSection::new(header, section),
            header_size: 0,
            row_size: 0,
            row_count: 0,
        }
    }

    pub fn init(&mut self, data: &mut Cursor<Vec<u8>>) -> Result<&Self> {
        self.header_size = data.read_u32::<LittleEndian>()?;
        self.row_size = data.read_u32::<LittleEndian>()?;
        self.row_count =data.read_u32::<LittleEndian>()?;

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

#[derive(Clone)]
pub struct SMXRTTIData {
    smx_file: Rc<RefCell<SMXFile>>,

    bytes: Vec<u8>,
}

impl SMXRTTIData {
    pub fn new(file: Rc<RefCell<SMXFile>>, header: Rc<SMXHeader>, section: Rc<SectionEntry>) -> Self {
        let base = BaseSection::new(Rc::clone(&header), Rc::clone(&section));
        
        Self {
            smx_file: Rc::clone(&file),
            bytes: base.get_data(),
        }
    }

    pub fn type_from_id(&self, type_id: i32) -> String {
        let kind: i32 = type_id & 0xf;
        let mut payload: i32 = (type_id >> 4) & 0x0fff_ffff;

        if kind == CB::TYPEID_INLINE as i32 {
            let temp: [u8; 4] = [
                (payload & 0xff) as u8,
                (payload >> 8) as u8,
                (payload >> 16) as u8,
                (payload >> 24) as u8,
            ];

            let mut builder: TypeBuilder = TypeBuilder::new(Rc::clone(&self.smx_file), temp.to_vec(), 0);

            return builder.decode_new()
        }

        //TODO: Consider convert to Result<String>
        if kind != CB::TYPEID_COMPLEX as i32 {
            return format!("Unknown type_id kind: {}", kind);
        }

        self.build_type_name(&mut payload)
    }

    pub fn function_type_from_offset(&self, offset: i32) -> String {
        let mut builder: TypeBuilder = TypeBuilder::new(Rc::clone(&self.smx_file), self.bytes.clone(), offset);

        builder.decode_function()
    }

    pub fn typeset_types_from_offset(&self, offset: i32) -> Vec<String> {
        let count: i32 = CB::decode_u32(&self.bytes, &mut offset.clone());

        let mut types: Vec<String> = Vec::with_capacity(count as usize);

        let mut builder: TypeBuilder = TypeBuilder::new(Rc::clone(&self.smx_file), self.bytes.clone(), offset);

        for _ in 0..count {
            types.push(builder.decode_new())
        }

        types
    }

    fn build_type_name(&self, offset: &mut i32) -> String {
        let mut builder: TypeBuilder = TypeBuilder::new(Rc::clone(&self.smx_file), self.bytes.clone(), *offset);

        let text: String = builder.decode_new();

        *offset = builder.offset;

        text
    }
}

struct TypeBuilder {
    file: Rc<RefCell<SMXFile>>,
    bytes: Vec<u8>,
    offset: i32,
    is_const: bool,
}

impl TypeBuilder{
    pub fn new(file: Rc<RefCell<SMXFile>>, bytes: Vec<u8>, offset: i32) -> Self {
        Self {
            file,
            bytes,
            offset,
            is_const: false,
        }
    }

    // Decode a type, but reset the |is_const| indicator for non-
    // dependent type.
    pub fn decode_new(&mut self) -> String {
        let was_const: bool = self.is_const;
        self.is_const = false;

        let mut result: String = self.decode();

        if self.is_const {
            result = format!("const {}", result);
        }

        self.is_const = was_const;

        result
    }

    pub fn decode(&mut self) -> String {
        self.is_const |= self.r#match(CB::CONST);
        let b: u8 = self.bytes[self.offset as usize];
        self.offset += 1;

        match b {
            CB::BOOL => "bool".into(),
            CB::INT32 => "int".into(),
            CB::FLOAT32 => "float".into(),
            CB::CHAR8 => "char".into(),
            CB::ANY => "any".into(),
            CB::TOPFUNCTION => "Function".into(),
            CB::FIXEDARRAY => {
                let index = CB::decode_u32(&self.bytes, &mut self.offset);
                let inner: String = self.decode();

                format!("{}[{}]", inner, index)
            },
            CB::ARRAY => {
                let inner: String = self.decode();
                
                format!("{}[]", inner)
            },
            CB::ENUM => {
                let index = CB::decode_u32(&self.bytes, &mut self.offset);

                self.file.borrow_mut().rtti_enums.as_ref().unwrap().enums()[index as usize].clone()
            },
            CB::TYPEDEF => {
                let index = CB::decode_u32(&self.bytes, &mut self.offset);

                self.file.borrow_mut().rtti_typedefs.as_ref().unwrap().typedefs()[index as usize].name.clone()
            }
            CB::TYPESET => {
                let index = CB::decode_u32(&self.bytes, &mut self.offset);

                self.file.borrow_mut().rtti_typesets.as_ref().unwrap().typesets()[index as usize].name.clone()
            },
            CB::STRUCT => {
                let index = CB::decode_u32(&self.bytes, &mut self.offset);

                self.file.borrow_mut().rtti_classdefs.as_ref().unwrap().defs()[index as usize].name.clone()
            },
            CB::FUNCTION => self.decode_function(),
            CB::ENUMSTRUCT => {
                let index = CB::decode_u32(&self.bytes, &mut self.offset);

                self.file.borrow_mut().rtti_enum_structs.as_ref().unwrap().entries()[index as usize].name.clone()
            },
            _ => format!("unknown type code: {}", b),
        }
    }

    pub fn decode_function(&mut self) -> String {
        let argc: u32 = self.bytes[self.offset as usize] as u32;
        self.offset += 1;

        let mut variadic: bool = false;

        if self.bytes[self.offset as usize] == CB::VARIADIC {
            variadic = true;
            self.offset += 1;
        }

        let return_type: String;

        if self.bytes[self.offset as usize] == CB::VOID {
            return_type = "void".into();
            self.offset += 1;
        } else {
            return_type = self.decode_new();
        }

        let mut argv: Vec<String> = Vec::with_capacity(argc as usize);

        for _ in 0..argc {
            let is_byref: bool = self.r#match(CB::BYREF);
            let mut text: String = self.decode_new();

            if is_byref {
                text += "&";
            }

            argv.push(text);
        }

        let mut signature: String = format!("function {} ({}", return_type, argv.join(", "));

        if variadic {
            signature += "...";
        }

        signature += ")";

        signature
    }

    fn r#match(&mut self, b: u8) -> bool {
        if self.bytes[self.offset as usize] != b {
            return false
        }

        self.offset += 1;

        true
    }
}

#[derive(Debug, Clone)]
pub struct SMXRTTIEnumTable {
    enums: Vec<String>,
}

impl SMXRTTIEnumTable {
    pub fn new(header: Rc<SMXHeader>, section: Rc<SectionEntry>, names: Rc<RefCell<SMXNameTable>>) -> Result<Self> {
        let base = BaseSection::new(header.clone(), section.clone());    
        let mut rtti = SMXRTTIListTable::new(header, section);

        let mut data = Cursor::new(base.get_data());

        rtti.init(&mut data)?;

        let mut enums: Vec<String> = Vec::with_capacity(rtti.row_count() as usize);

        for _ in 0..rtti.row_count() {
            let index = data.read_i32::<LittleEndian>()?;

            enums.push(names.borrow_mut().string_at(index)?);

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
    pub name: String,

    pub pcode_start: i32,

    pub pcode_end: i32,

    pub signature: i32,
}

#[derive(Debug, Clone)]
pub struct SMXRTTIMethodTable {
    methods: Vec<RTTIMethod>,
}

impl SMXRTTIMethodTable {
    pub fn new(header: Rc<SMXHeader>, section: Rc<SectionEntry>, names: Rc<RefCell<SMXNameTable>>) -> Result<Self> {
        let base = BaseSection::new(Rc::clone(&header), Rc::clone(&section));    
        let mut rtti = SMXRTTIListTable::new(Rc::clone(&header), Rc::clone(&section));

        let mut data = Cursor::new(base.get_data());

        rtti.init(&mut data)?;

        let mut methods: Vec<RTTIMethod> = Vec::with_capacity(rtti.row_count() as usize);

        for _ in 0..rtti.row_count() {
            let index = data.read_i32::<LittleEndian>()?;

            methods.push(RTTIMethod {
                name: names.borrow_mut().string_at(index)?,
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

    pub fn methods_ref(&self) -> &Vec<RTTIMethod> {
        self.methods.as_ref()
    }
}

#[derive(Debug, Clone)]
pub struct RTTINative {
    pub name: String,

    pub signature: i32,
}

#[derive(Debug, Clone)]
pub struct SMXRTTINativeTable {
    natives: Vec<RTTINative>,
}

impl SMXRTTINativeTable {
    pub fn new(header: Rc<SMXHeader>, section: Rc<SectionEntry>, names: Rc<RefCell<SMXNameTable>>) -> Result<Self> {
        let base = BaseSection::new(Rc::clone(&header), Rc::clone(&section));    
        let mut rtti = SMXRTTIListTable::new(Rc::clone(&header), Rc::clone(&section));

        let mut data = Cursor::new(base.get_data());

        rtti.init(&mut data)?;

        let mut natives: Vec<RTTINative> = Vec::with_capacity(rtti.row_count() as usize);

        for _ in 0..rtti.row_count() {
            let index = data.read_i32::<LittleEndian>()?;

            natives.push(RTTINative {
                name: names.borrow_mut().string_at(index)?,
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
    pub name: String,

    pub type_id: i32,
}

#[derive(Debug, Clone)]
pub struct SMXRTTITypedefTable {
    typedefs: Vec<RTTITypedef>,
}

impl SMXRTTITypedefTable {
    pub fn new(header: Rc<SMXHeader>, section: Rc<SectionEntry>, names: Rc<RefCell<SMXNameTable>>) -> Result<Self> {
        let base = BaseSection::new(Rc::clone(&header), Rc::clone(&section));    
        let mut rtti = SMXRTTIListTable::new(Rc::clone(&header), Rc::clone(&section));

        let mut data = Cursor::new(base.get_data());

        rtti.init(&mut data)?;

        let mut typedefs: Vec<RTTITypedef> = Vec::with_capacity(rtti.row_count() as usize);

        for _ in 0..rtti.row_count() {
            let index = data.read_i32::<LittleEndian>()?;

            typedefs.push(RTTITypedef {
                name: names.borrow_mut().string_at(index)?,
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
    pub name: String,

    pub signature: i32,
}

#[derive(Debug, Clone)]
pub struct SMXRTTITypesetTable {
    typesets: Vec<RTTITypeset>,
}

impl SMXRTTITypesetTable {
    pub fn new(header: Rc<SMXHeader>, section: Rc<SectionEntry>, names: Rc<RefCell<SMXNameTable>>) -> Result<Self> {
        let base = BaseSection::new(Rc::clone(&header), Rc::clone(&section));    
        let mut rtti = SMXRTTIListTable::new(Rc::clone(&header), Rc::clone(&section));

        let mut data = Cursor::new(base.get_data());

        rtti.init(&mut data)?;

        let mut typesets: Vec<RTTITypeset> = Vec::with_capacity(rtti.row_count() as usize);

        for _ in 0..rtti.row_count() {
            let index = data.read_i32::<LittleEndian>()?;

            typesets.push(RTTITypeset {
                name: names.borrow_mut().string_at(index)?,
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
    pub name_offset: i32,

    pub first_field: i32,

    pub size: i32,

    pub name: String,
}

#[derive(Debug, Clone)]
pub struct SMXRTTIEnumStructTable {
    entries: Vec<RTTIEnumStruct>,
}

impl SMXRTTIEnumStructTable {
    pub fn new(header: Rc<SMXHeader>, section: Rc<SectionEntry>, names: Rc<RefCell<SMXNameTable>>) -> Result<Self> {
        let base = BaseSection::new(Rc::clone(&header), Rc::clone(&section));    
        let mut rtti = SMXRTTIListTable::new(Rc::clone(&header), Rc::clone(&section));

        let mut data = Cursor::new(base.get_data());

        rtti.init(&mut data)?;

        let mut entries: Vec<RTTIEnumStruct> = Vec::with_capacity(rtti.row_count() as usize);

        for _ in 0..rtti.row_count() {
            let name_offset = data.read_i32::<LittleEndian>()?;
            let first_field = data.read_i32::<LittleEndian>()?;
            let size = data.read_i32::<LittleEndian>()?;
            let name = names.borrow_mut().string_at(name_offset)?;

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
    pub name_offset: i32,

    pub type_id: i32,

    pub offset: i32,

    pub name: String,
}

#[derive(Debug, Clone)]
pub struct SMXRTTIEnumStructFieldTable {
    entries: Vec<RTTIEnumStructField>,
}

impl SMXRTTIEnumStructFieldTable {
    pub fn new(header: Rc<SMXHeader>, section: Rc<SectionEntry>, names: Rc<RefCell<SMXNameTable>>) -> Result<Self> {
        let base = BaseSection::new(Rc::clone(&header), Rc::clone(&section));    
        let mut rtti = SMXRTTIListTable::new(Rc::clone(&header), Rc::clone(&section));

        let mut data = Cursor::new(base.get_data());

        rtti.init(&mut data)?;

        let mut entries: Vec<RTTIEnumStructField> = Vec::with_capacity(rtti.row_count() as usize);

        for _ in 0..rtti.row_count() {
            let name_offset = data.read_i32::<LittleEndian>()?;
            let type_id = data.read_i32::<LittleEndian>()?;
            let offset = data.read_i32::<LittleEndian>()?;
            let name = names.borrow_mut().string_at(name_offset)?;

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
    pub flags: i32,

    pub name_offset: i32,

    pub first_field: i32,

    pub name: String,
}

#[derive(Debug, Clone)]
pub struct SMXRTTIClassDefTable {
    defs: Vec<RTTIClassDef>,
}

impl SMXRTTIClassDefTable {
    pub fn new(header: Rc<SMXHeader>, section: Rc<SectionEntry>, names: Rc<RefCell<SMXNameTable>>) -> Result<Self> {
        let base = BaseSection::new(Rc::clone(&header), Rc::clone(&section));    
        let mut rtti = SMXRTTIListTable::new(Rc::clone(&header), Rc::clone(&section));

        let mut data = Cursor::new(base.get_data());

        rtti.init(&mut data)?;

        let mut defs: Vec<RTTIClassDef> = Vec::with_capacity(rtti.row_count() as usize);

        for _ in 0..rtti.row_count() {
            let flags = data.read_i32::<LittleEndian>()?;
            let name_offset = data.read_i32::<LittleEndian>()?;
            let first_field = data.read_i32::<LittleEndian>()?;
            let name = names.borrow_mut().string_at(name_offset)?;

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
    pub flags: i16,

    pub name_offset: i32,

    pub type_id: i32,

    pub name: String,
}

#[derive(Debug, Clone)]
pub struct SMXRTTIFieldTable {
    fields: Vec<RTTIField>,
}

impl SMXRTTIFieldTable {
    pub fn new(header: Rc<SMXHeader>, section: Rc<SectionEntry>, names: Rc<RefCell<SMXNameTable>>) -> Result<Self> {
        let base = BaseSection::new(Rc::clone(&header), Rc::clone(&section));    
        let mut rtti = SMXRTTIListTable::new(Rc::clone(&header), Rc::clone(&section));

        let mut data = Cursor::new(base.get_data());

        rtti.init(&mut data)?;

        let mut fields: Vec<RTTIField> = Vec::with_capacity(rtti.row_count() as usize);

        for _ in 0..rtti.row_count() {
            let flags = data.read_i16::<LittleEndian>()?;
            let name_offset = data.read_i32::<LittleEndian>()?;
            let type_id = data.read_i32::<LittleEndian>()?;
            let name = names.borrow_mut().string_at(name_offset)?;

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