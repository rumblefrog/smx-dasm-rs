use std::io::Cursor;
use std::collections::HashMap;
use crate::headers::{SMXHeader, SectionEntry};
use crate::errors::{Result, Error};

#[derive(Debug, Clone)]
pub struct BaseSection {
    header: SMXHeader,
    section: SectionEntry,
}

impl BaseSection {
    pub fn new(header: SMXHeader, section: SectionEntry) -> Self {
        BaseSection {
            header,
            section,
        }
    }

    // Read-only, cloned
    pub fn section(&self) -> SectionEntry {
        self.section.clone()
    }

    pub fn get_cursor(&self) -> Cursor<Vec<u8>> {
        Cursor::new(self.header.data[self.section.data_offset as usize..(self.section.data_offset + self.section.size) as usize].to_vec())
    }
}

#[derive(Debug, Clone)]
pub struct SMXNameTable {
    base: BaseSection,

    names: HashMap<i32, String>,

    extends: Vec<i32>,
}

impl SMXNameTable {
    pub fn new(header: SMXHeader, section: SectionEntry) -> Self {
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
        if self.extends.len() == 0 {
            self.compute_extends();
        }

        self.extends.clone()
    }

    // Returns a string at a given index.
    pub fn string_at(&mut self, index: &i32) -> Result<String> {
        if self.names.contains_key(index) {
            return Ok(self.names.get(index).unwrap().clone())
        }

        if index >= &self.base.section.size {
            return Err(Error::InvalidIndex)
        }

        let mut str_vec = Vec::with_capacity(256);

        for i in *index..self.base.section.size {
            if self.base.header.data[(self.base.section.data_offset + i) as usize] == 0 {
                break;
            }

            str_vec.push(self.base.header.data[(self.base.section.data_offset + i) as usize]);
        }

        Ok(String::from_utf8_lossy(&str_vec[..]).into_owned())
    }
}
