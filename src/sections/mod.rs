use std::collections::HashMap;
use crate::headers::{SMXHeader, SectionEntry};
use crate::v1types::*;
use crate::errors::{Result, Error};

#[derive(Debug, Clone)]
pub struct BaseSection<'a> {
    header: &'a SMXHeader,
    section: &'a SectionEntry,
}

impl<'a> BaseSection<'a> {
    pub fn new(header: &'a SMXHeader, section: &'a SectionEntry) -> Self {
        BaseSection {
            header,
            section,
        }
    }

    // Read-only, cloned
    pub fn section(&self) -> SectionEntry {
        self.section.clone()
    }

    pub fn get_data(&self) -> Vec<u8> {
        self.header.data[self.section.data_offset as usize..(self.section.data_offset + self.section.size) as usize].to_vec()
    }
}

// The following tables conform to a nametable:
//   .names
//   .dbg.names
#[derive(Debug, Clone)]
pub struct SMXNameTable<'b> {
    base: BaseSection<'b>,

    names: HashMap<i32, String>,

    extends: Vec<i32>,
}

impl<'b> SMXNameTable<'b> {
    pub fn new(header: &'b SMXHeader, section: &'b SectionEntry) -> Self {
        Self {
            base: BaseSection::new(header, section),
            names: HashMap::new(),
            extends: Vec::new(),
        }
    }

    fn compute_extends(&mut self) -> &Self {
        let mut last_index: i32 = 0;

        for i in 0..self.base.section.size {
            if self.base.header.data[(self.base.section.data_offset + i) as usize] == 0 {
                self.extends.push(last_index);
                last_index = i + 1;
            }
        }

        self
    }

    // Returns a list of all root indexes that map to strings.
    pub fn get_extends(&mut self) -> Vec<i32> {
        if self.extends.is_empty() {
            self.compute_extends();
        }

        self.extends.clone()
    }

    // Returns a string at a given index.
    pub fn string_at(&mut self, index: i32) -> Result<String> {
        if self.names.contains_key(&index) {
            return Ok(self.names.get(&index).unwrap().clone())
        }

        if index >= self.base.section.size {
            return Err(Error::InvalidIndex)
        }

        let mut str_vec = Vec::with_capacity(256);

        for i in index..self.base.section.size {
            if self.base.header.data[(self.base.section.data_offset + i) as usize] == 0 {
                break;
            }

            str_vec.push(self.base.header.data[(self.base.section.data_offset + i) as usize]);
        }

        Ok(String::from_utf8_lossy(&str_vec[..]).into_owned())
    }
}

// The .natives table.
#[derive(Debug, Clone)]
pub struct SMXNativeTable {
    natives: Vec<NativeEntry>,
}

impl SMXNativeTable {
    pub fn new(header: &SMXHeader, section: &SectionEntry, names: &mut SMXNameTable) -> Result<Self> {
        let base = BaseSection::new(header, section);
        let natives = NativeEntry::new(&base.get_data(), section, names)?;

        Ok(Self {
            natives,
        })
    }

    // Return a copy of the natives vector
    pub fn entries(&self) -> Vec<NativeEntry> {
        self.natives.clone()
    }

    // Return immutable cloned copy at index
    pub fn get_entry(&self, index: usize) -> NativeEntry {
        self.natives[index].clone()
    }

    pub fn size(&self) -> usize {
        self.natives.len()
    }
}

// The .publics table.
#[derive(Debug, Clone)]
pub struct SMXPublicTable {
    publics: Vec<PublicEntry>,
}

impl SMXPublicTable {
    pub fn new(header: &SMXHeader, section: &SectionEntry, names: &mut SMXNameTable) -> Result<Self> {
        let base = BaseSection::new(header, section);
        let publics = PublicEntry::new(base.get_data(), section, names)?;

        Ok(Self {
            publics,
        })
    }

    // Return a copy of the publics vector
    pub fn entries(&self) -> Vec<PublicEntry> {
        self.publics.clone()
    }

    // Return immutable cloned copy at index
    pub fn get_entry(&self, index: usize) -> PublicEntry {
        self.publics[index].clone()
    }

    pub fn size(&self) -> usize {
        self.publics.len()
    }
}

#[derive(Debug, Clone, Default)]
pub struct SMXCalledFunctionsTable {
    functions: Vec<CalledFunctionEntry>,
}

impl SMXCalledFunctionsTable {
    pub fn new() -> Self {
        Self {
            functions: Vec::new(),
        }
    }

    pub fn add_function(&mut self, addr: u32) {
        self.functions.push(CalledFunctionEntry {
            address: addr,
            name: format!("sub_{:x}", addr),
        })
    }

    // Return a copy of the publics vector
    pub fn entries(&self) -> Vec<CalledFunctionEntry> {
        self.functions.clone()
    }

    // Return immutable cloned copy at index
    pub fn get_entry(&self, index: usize) -> CalledFunctionEntry {
        self.functions[index].clone()
    }

    pub fn size(&self) -> usize {
        self.functions.len()
    }
}

// The .pubvars table.
#[derive(Debug, Clone)]
pub struct SMXPubvarTable {
    public_variables: Vec<PubvarEntry>,
}

impl SMXPubvarTable {
    pub fn new(header: &SMXHeader, section: &SectionEntry, names: &mut SMXNameTable) -> Result<Self> {
        let base = BaseSection::new(header, section);
        let public_variables = PubvarEntry::new(base.get_data(), section, names)?;

        Ok(Self {
            public_variables,
        })
    }

    // Return a copy of the publics vector
    pub fn entries(&self) -> Vec<PubvarEntry> {
        self.public_variables.clone()
    }

    // Return immutable cloned copy at index
    pub fn get_entry(&self, index: usize) -> PubvarEntry {
        self.public_variables[index].clone()
    }

    pub fn size(&self) -> usize {
        self.public_variables.len()
    }
}

#[derive(Debug, Clone)]
pub enum TagFlags {
    Fixed,
    Function,
    Object,
    Enum,
    MethodMap,
    Struct,
}

impl TagFlags {
    pub fn value(&self) -> u32 {
        match *self {
            TagFlags::Fixed => 0x4000_0000,
            TagFlags::Function => 0x2000_0000,
            TagFlags::Object => 0x1000_0000,
            TagFlags::Enum => 0x0800_0000,
            TagFlags::MethodMap => 0x0400_0000,
            TagFlags::Struct => 0x0200_0000,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Tag {
    entry: TagEntry,
}

impl Tag {
    pub fn new(entry: TagEntry) -> Self {
        Self {
            entry,
        }
    }

    pub fn id(&self) -> u32 {
        self.entry.tag & !TagEntry::FLAGMASK
    }

    pub fn value(&self) -> u32 {
        self.entry.tag
    }

    pub fn flags(&self) -> u32 {
        self.entry.tag & TagEntry::FLAGMASK
    }

    pub fn name(&self) -> String {
        self.entry.name.clone()
    }

    pub fn entry(&self) -> TagEntry {
        self.entry.clone()
    }
}

// The .tags table.
#[derive(Debug, Clone)]
pub struct SMXTagTable {
    tags: Vec<Tag>,

    cache: HashMap<u16, Tag>,
}

impl SMXTagTable {
    pub fn new(header: &SMXHeader, section: &SectionEntry, names: &mut SMXNameTable) -> Result<Self> {
        let base = BaseSection::new(header, section);
        let tags = TagEntry::new(base.get_data(), section, names)?;

        let mut tt = Self {
            tags: Vec::new(),
            cache: HashMap::new(),
        };

        for tag in tags {
            tt.tags.push(Tag::new(tag.to_owned()))
        }

        Ok(tt)
    }

    pub fn find_tag(&mut self, tag: u16) -> Option<Tag> {
        if self.cache.contains_key(&tag) {
            return Some(self.cache.get(&tag).unwrap().clone());
        }

        let mut found: Option<Tag> = None;

        for i in 0..self.tags.len() {
            if self.tags[i].id() as u16 == tag {
                found = Some(self.tags[i].clone());
                break;
            }
        }

        if let Some(v) = &found {
            self.cache.insert(tag, v.clone());
        }

        found
    }


    // Return a copy of the tag vector
    pub fn entries(&self) -> Vec<Tag> {
        self.tags.clone()
    }

    // Return immutable cloned copy at index
    pub fn get_entry(&self, index: usize) -> Tag {
        self.tags[index].clone()
    }

    pub fn len(&self) -> usize {
        self.tags.len()
    }

    pub fn is_empty(&self) -> bool {
        self.tags.is_empty()
    }
}

// The .data section.
#[derive(Debug, Clone)]
pub struct SMXDataSection<'b> {
    base: BaseSection<'b>,

    data_header: DataHeader,
}

impl<'b> SMXDataSection<'b> {
    pub fn new(header: &'b SMXHeader, section: &'b SectionEntry) -> Result<Self> {
        let base = BaseSection::new(header, section);
        let data_header = DataHeader::new(base.get_data())?;

        Ok(Self {
            base,
            data_header,
        })
    }

    pub fn get_data_vec(&self) -> Vec<u8> {
        let start = self.base.section.data_offset as u32 + self.data_header.data_offset;

        Vec::from(&self.base.header.data[start as usize..(start + self.data_header.data_size) as usize])
    }

    pub fn header(&self) -> DataHeader {
        self.data_header.clone()
    }
}

// The .code section.
#[derive(Debug, Clone)]
pub struct SMXCodeV1Section<'b> {
    base: BaseSection<'b>,

    code_header: CodeV1Header,
}

impl<'b> SMXCodeV1Section<'b> {
    pub fn new(header: &'b SMXHeader, section: &'b SectionEntry) -> Result<Self> {
        let base = BaseSection::new(header, section);
        let code_header = CodeV1Header::new(base.get_data())?;

        Ok(Self {
            base,
            code_header,
        })
    }

    pub fn get_data_vec(&self) -> Vec<u8> {
        let start = self.base.section.data_offset + self.code_header.code_offset;

        Vec::from(&self.base.header.data[start as usize..(start + self.code_header.code_size) as usize])
    }

    pub fn header(&self) -> CodeV1Header {
        self.code_header.clone()
    }

    pub fn code_start(&self) -> i32 {
        self.base.header.data_offset + self.code_header.code_offset
    }
}

// The .dbg.info section.
#[derive(Debug, Clone)]
pub struct SMXDebugInfoSection {
    info: DebugInfoHeader,
}

impl SMXDebugInfoSection {
    pub fn new(header: &SMXHeader, section: &SectionEntry) -> Result<Self> {
        let base = BaseSection::new(header, section);
        let info = DebugInfoHeader::new(base.get_data())?;

        Ok(Self {
            info,
        })
    }

    pub fn file_count(&self) -> i32 {
        self.info.file_count
    }

    pub fn line_count(&self) -> i32 {
        self.info.line_count
    }

    pub fn symbol_count(&self) -> i32 {
        self.info.symbol_count
    }

    pub fn array_count(&self) -> i32 {
        self.info.array_count
    }
}

// The .dbg.files table.
#[derive(Debug, Clone)]
pub struct SMXDebugFilesTable {
    entries: Vec<DebugFileEntry>,
}

impl SMXDebugFilesTable {
    pub fn new(header: &SMXHeader, section: &SectionEntry, names: &mut SMXNameTable) -> Result<Self> {
        let base = BaseSection::new(header, section);
        let entries = DebugFileEntry::new(base.get_data(), section, names)?;

        Ok(Self {
            entries,
        })
    }

    pub fn find_file(&self, addr: u32) -> Option<String> {
        let mut high = self.len() as i32;
        let mut low = -1;

        while high - low > 1 {
            let mid = (low + high) / 2;

            if self.entries[mid as usize].address <= addr {
                low = mid;
            } else {
                high = mid;
            }
        }

        if low == -1 {
            return None;
        }

        Some(self.entries[low as usize].name.clone())
    }

    // Return a copy of the tag vector
    pub fn entries(&self) -> Vec<DebugFileEntry> {
        self.entries.clone()
    }

    // Return immutable cloned copy at index
    pub fn get_entry(&self, index: usize) -> DebugFileEntry {
        self.entries[index].clone()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

// The .dbg.lines table.
#[derive(Debug, Clone)]
pub struct SMXDebugLinesTable {
    entries: Vec<DebugLineEntry>,
}

impl SMXDebugLinesTable {
    pub fn new(header: &SMXHeader, section: &SectionEntry) -> Result<Self> {
        let base = BaseSection::new(header, section);
        let entries = DebugLineEntry::new(base.get_data(), section)?;

        Ok(Self {
            entries,
        })
    }

    pub fn find_file(&self, addr: u32) -> Option<u32> {
        let mut high = self.len() as i32;
        let mut low = -1;

        while high - low > 1 {
            let mid = (low + high) / 2;

            if self.entries[mid as usize].address <= addr {
                low = mid;
            } else {
                high = mid;
            }
        }

        if low == -1 {
            return None;
        }

        Some(self.entries[low as usize].line + 1)
    }

    // Return a copy of the tag vector
    pub fn entries(&self) -> Vec<DebugLineEntry> {
        self.entries.clone()
    }

    // Return immutable cloned copy at index
    pub fn get_entry(&self, index: usize) -> DebugLineEntry {
        self.entries[index].clone()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}
