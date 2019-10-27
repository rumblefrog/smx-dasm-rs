use std::rc::Rc;
use std::cell::RefCell;
use crate::headers::{SMXHeader, SectionEntry};
use crate::sections::*;
use crate::rtti::*;
use crate::v1disassembler::V1Disassembler;
use crate::errors::Result;

#[derive(Debug, Default)]
pub struct SMXFile {
    pub header: Rc<SMXHeader>,
    pub unknown_sections: Vec<Rc<SectionEntry>>,

    pub names: Option<Rc<RefCell<SMXNameTable>>>,
    pub debug_names: Option<Rc<RefCell<SMXNameTable>>>,
    pub natives: Option<Rc<SMXNativeTable>>,
    pub publics: Option<Rc<SMXPublicTable>>,
    pub pubvars: Option<Rc<SMXPubvarTable>>,
    pub tags: Option<Rc<SMXTagTable>>,
    pub data: Option<Rc<SMXDataSection>>,
    pub codev1: Option<Rc<SMXCodeV1Section>>,
    pub called_functions: Option<Rc<RefCell<SMXCalledFunctionsTable>>>,

    pub debug_info: Option<Rc<SMXDebugInfoSection>>,
    pub debug_files: Option<Rc<SMXDebugFilesTable>>,
    pub debug_lines: Option<Rc<SMXDebugLinesTable>>,

    pub rtti_data: Option<Rc<SMXRTTIData>>,
    pub rtti_enums: Option<Rc<SMXRTTIEnumTable>>,
    pub rtti_enum_structs: Option<Rc<SMXRTTIEnumStructTable>>,
    pub rtti_enum_struct_fields: Option<Rc<SMXRTTIEnumStructFieldTable>>,
    pub rtti_classdefs: Option<Rc<SMXRTTIClassDefTable>>,
    pub rtti_fields:  Option<Rc<SMXRTTIFieldTable>>,
    pub rtti_methods: Option<Rc<SMXRTTIMethodTable>>,
    pub rtti_natives: Option<Rc<SMXRTTINativeTable>>,
    pub rtti_typedefs: Option<Rc<SMXRTTITypedefTable>>,
    pub rtti_typesets: Option<Rc<SMXRTTITypesetTable>>,

    pub debug_methods: Option<Rc<SMXDebugMethods>>,
    pub debug_globals: Option<Rc<RefCell<SMXDebugGlobals>>>,
    pub debug_locals: Option<Rc<SMXDebugLocals>>,
}

impl SMXFile {
    pub fn new<T>(data: T) -> Result<Rc<RefCell<SMXFile>>>
    where
        T: AsRef<[u8]>,
    {
        let file: Rc<RefCell<SMXFile>> = Rc::new(RefCell::new(Default::default()));

        {
            let file_mut = &mut *file.borrow_mut();

            file_mut.header = Rc::new(SMXHeader::new(&data)?);
            file_mut.unknown_sections = Vec::new();
            file_mut.called_functions = Some(Rc::new(RefCell::new(SMXCalledFunctionsTable::new())));

            for section in &file_mut.header.sections {
                match section.name.as_ref() {
                    ".names"  => file_mut.names = Some(Rc::new(RefCell::new(SMXNameTable::new(Rc::clone(&file_mut.header), Rc::clone(&section))))),
                    ".dbg.strings" => file_mut.debug_names = Some(Rc::new(RefCell::new(SMXNameTable::new(Rc::clone(&file_mut.header), Rc::clone(&section))))),
                    ".dbg.info" => file_mut.debug_info = Some(Rc::new(SMXDebugInfoSection::new(Rc::clone(&file_mut.header), Rc::clone(&section))?)),
                    _ => (),
                }
            }

            if file_mut.debug_names.is_none() {
                file_mut.debug_names = file_mut.names.clone();
            }

            // After first pass, we have the name tables
            for section in &file_mut.header.sections {
                match section.name.as_ref() {
                    ".names" | ".dbg.strings" | ".dbg.info" => (),
                    ".natives" => file_mut.natives = Some(Rc::new(SMXNativeTable::new(Rc::clone(&file_mut.header), Rc::clone(&section), Rc::clone(file_mut.names.as_ref().unwrap()))?)),
                    ".publics" => file_mut.publics = Some(Rc::new(SMXPublicTable::new(Rc::clone(&file_mut.header), Rc::clone(&section), Rc::clone(file_mut.names.as_ref().unwrap()))?)),
                    ".pubvars" => file_mut.pubvars = Some(Rc::new(SMXPubvarTable::new(Rc::clone(&file_mut.header), Rc::clone(&section), Rc::clone(file_mut.names.as_ref().unwrap()))?)),
                    ".tags" => file_mut.tags = Some(Rc::new(SMXTagTable::new(Rc::clone(&file_mut.header), Rc::clone(&section), Rc::clone(file_mut.names.as_ref().unwrap()))?)),
                    ".data" => file_mut.data = Some(Rc::new(SMXDataSection::new(Rc::clone(&file_mut.header), Rc::clone(&section))?)),
                    ".code" => file_mut.codev1 = Some(Rc::new(SMXCodeV1Section::new(Rc::clone(&file_mut.header), Rc::clone(&section))?)),
                    ".dbg.files" => file_mut.debug_files = Some(Rc::new(SMXDebugFilesTable::new(Rc::clone(&file_mut.header), Rc::clone(&section), Rc::clone(file_mut.names.as_ref().unwrap()))?)),
                    ".dbg.lines" => file_mut.debug_lines = Some(Rc::new(SMXDebugLinesTable::new(Rc::clone(&file_mut.header), Rc::clone(&section))?)),
                    // .dbg.natives and .dbg.symbols is unimplemented due to being legacy
                    ".dbg.methods" => file_mut.debug_methods = Some(Rc::new(SMXDebugMethods::new(Rc::clone(&file_mut.header), Rc::clone(&section))?)), // names param is excluded as it's not used
                    ".dbg.globals" => file_mut.debug_globals = Some(Rc::new(RefCell::new(SMXDebugGlobals::new(Rc::clone(&file_mut.header), Rc::clone(&section))?))),
                    ".dbg.locals" => file_mut.debug_locals = Some(Rc::new(SMXDebugLocals::new(Rc::clone(&file), Rc::clone(&file_mut.header), Rc::clone(&section))?)),
                    "rtti.data" => file_mut.rtti_data = Some(Rc::new(SMXRTTIData::new(Rc::clone(&file), Rc::clone(&file_mut.header), Rc::clone(&section)))),
                    "rtti.classdefs" => file.borrow_mut().rtti_classdefs = Some(Rc::new(SMXRTTIClassDefTable::new(Rc::clone(&file_mut.header), Rc::clone(&section), Rc::clone(file_mut.names.as_ref().unwrap()))?)),
                    "rtti.enumstructs" => file.borrow_mut().rtti_enum_structs = Some(Rc::new(SMXRTTIEnumStructTable::new(Rc::clone(&file_mut.header), Rc::clone(&section), Rc::clone(file_mut.names.as_ref().unwrap()))?)),
                    "rtti.enumstruct_fields" => file.borrow_mut().rtti_enum_struct_fields = Some(Rc::new(SMXRTTIEnumStructFieldTable::new(Rc::clone(&file_mut.header), Rc::clone(&section), Rc::clone(file_mut.names.as_ref().unwrap()))?)),
                    "rtti.fields" => file.borrow_mut().rtti_fields = Some(Rc::new(SMXRTTIFieldTable::new(Rc::clone(&file_mut.header), Rc::clone(&section), Rc::clone(file_mut.names.as_ref().unwrap()))?)),
                    "rtti.methods" => file.borrow_mut().rtti_methods = Some(Rc::new(SMXRTTIMethodTable::new(Rc::clone(&file_mut.header), Rc::clone(&section), Rc::clone(file_mut.names.as_ref().unwrap()))?)),
                    "rtti.natives" => file.borrow_mut().rtti_natives = Some(Rc::new(SMXRTTINativeTable::new(Rc::clone(&file_mut.header), Rc::clone(&section), Rc::clone(file_mut.names.as_ref().unwrap()))?)),
                    "rtti.enums" => file.borrow_mut().rtti_enums = Some(Rc::new(SMXRTTIEnumTable::new(Rc::clone(&file_mut.header), Rc::clone(&section), Rc::clone(file_mut.names.as_ref().unwrap()))?)),
                    "rtti.typedefs" => file.borrow_mut().rtti_typedefs = Some(Rc::new(SMXRTTITypedefTable::new(Rc::clone(&file_mut.header), Rc::clone(&section), Rc::clone(file_mut.names.as_ref().unwrap()))?)),
                    "rtti.typesets" => file.borrow_mut().rtti_typesets = Some(Rc::new(SMXRTTITypesetTable::new(Rc::clone(&file_mut.header), Rc::clone(&section), Rc::clone(file_mut.names.as_ref().unwrap()))?)),
                    _ =>  file.borrow_mut().unknown_sections.push(Rc::clone(&section)),
                }
            }

            // Legacy debug symbols table is skipped

            if file_mut.publics.is_some() {
                for pubfun in file_mut.publics.as_ref().unwrap().entries_ref() {
                    V1Disassembler::diassemble(Rc::clone(&file), Rc::clone(file.borrow().codev1.as_ref().unwrap()), pubfun.address as i32)?;
                }
            }

            if file_mut.called_functions.is_some() {
                for fun in file_mut.called_functions.as_ref().unwrap().borrow().entries_ref() {
                    V1Disassembler::diassemble(Rc::clone(&file), Rc::clone(file.borrow().codev1.as_ref().unwrap()), fun.address as i32)?;
                }
            }
        }

        Ok(file)
    }

    pub fn find_global_name(&mut self, addr: i32) -> Option<String> {
        if self.debug_globals.is_some() {
            let sym = self.debug_globals.as_mut().unwrap().borrow_mut().find_global(addr);

            if sym.is_some() {
                return Some(self.names.as_mut().unwrap().borrow_mut().string_at(sym.unwrap().name_offset).unwrap());
            }
        }

        None
    }

    pub fn find_local_name(&mut self, code_addr: i32, addr: i32) -> Option<String> {
        if self.debug_locals.is_some() {
            let entry = self.debug_locals.as_ref().unwrap().find_local(code_addr, addr);

            if entry.is_some() {
                return Some(self.names.as_mut().unwrap().borrow_mut().string_at(entry.unwrap().name_offset).unwrap());
            }
        }

        None
    }

    pub fn find_function_name(&self, addr: i32) -> String {
        if self.publics.is_some() {
            for pubfun in self.publics.as_ref().unwrap().entries_ref() {
                if pubfun.address == addr as u32 {
                    return pubfun.name.clone();
                }
            }
        }

        if self.called_functions.is_some() {
            for fun in self.called_functions.as_ref().unwrap().borrow().entries_ref() {
                if fun.address == addr as u32 {
                    return fun.name.clone();
                }
            }
        }

        "unknown".into()
    }

    pub fn is_function_at_address(&self, addr: i32) -> bool {
        // Legacy debug symbols is unimplemented

        if self.publics.is_some() {
            for pubfun in self.publics.as_ref().unwrap().entries_ref() {
                if pubfun.address == addr as u32 {
                    return true;
                }
            }
        }

        if self.called_functions.is_some() {
            for fun in self.called_functions.as_ref().unwrap().borrow().entries_ref() {
                if fun.address == addr as u32 {
                    return true;
                }
            }
        }

        false
    }
}