use std::io::Cursor;
use crate::headers::{SMXHeader, SectionEntry};

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