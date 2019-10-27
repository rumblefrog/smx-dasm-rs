use crate::headers::{SMXHeader, SectionEntry};
use crate::sections::*;
use crate::rtti::*;
use crate::v1disassembler::V1Disassembler;
use crate::errors::Result;

#[derive(Debug, Default)]
pub struct SMXFile<'a> {
    pub header: SMXHeader,
    pub unknown_sections: Vec<SectionEntry>,

    pub names: Option<SMXNameTable<'a>>,
    pub debug_names: Option<SMXNameTable<'a>>,
    pub natives: Option<SMXNativeTable>,
    pub publics: Option<SMXPublicTable>,
    pub pubvars: Option<SMXPubvarTable>,
    pub tags: Option<SMXTagTable>,
    pub data: Option<SMXDataSection<'a>>,
    pub codev1: Option<SMXCodeV1Section<'a>>,
    pub called_functions: Option<&'a SMXCalledFunctionsTable>,

    pub debug_info: Option<SMXDebugInfoSection>,
    pub debug_files: Option<SMXDebugFilesTable>,
    pub debug_lines: Option<SMXDebugLinesTable>,

    pub rtti_data: Option<SMXRTTIData<'a>>,
    pub rtti_enums: Option<SMXRTTIEnumTable>,
    pub rtti_enum_structs: Option<SMXRTTIEnumStructTable>,
    pub rtti_enum_struct_fields: Option<SMXRTTIEnumStructFieldTable>,
    pub rtti_classdefs: Option<SMXRTTIClassDefTable>,
    pub rtti_fields:  Option<SMXRTTIFieldTable>,
    pub rtti_methods: Option<SMXRTTIMethodTable>,
    pub rtti_natives: Option<SMXRTTINativeTable>,
    pub rtti_typedefs: Option<SMXRTTITypedefTable>,
    pub rtti_typesets: Option<SMXRTTITypesetTable>,

    pub debug_methods: Option<SMXDebugMethods>,
    pub debug_globals: Option<SMXDebugGlobals>,
    pub debug_locals: Option<SMXDebugLocals<'a>>,
}

impl<'a> SMXFile<'a> {
    pub fn new<T>(data: T) -> Result<Self>
    where
        T: AsRef<[u8]>,
    {
        let mut file: SMXFile<'a> = Default::default();

        let cf = &SMXCalledFunctionsTable::new();

        file.header = SMXHeader::new(&data)?;
        file.unknown_sections = Vec::new();
        file.called_functions = Some(cf);

        for section in file.header.sections {
            match section.name.as_ref() {
                ".names"  => file.names = Some(SMXNameTable::new(&file.header, &section)),
                ".dbg.strings" => file.debug_names = Some(SMXNameTable::new(&file.header, &section)),
                ".dbg.info" => file.debug_info = Some(SMXDebugInfoSection::new(&file.header, &section)?),
                _ => (),
            }
        }

        if file.debug_names.is_none() {
            file.debug_names = file.names;
        }

        // After first pass, we have the name table
        for section in file.header.sections {
            match section.name.as_ref() {
                ".names" | ".dbg.strings" | ".dbg.info" => (),
                ".natives" => file.natives = Some(SMXNativeTable::new(&file.header, &section, file.names.as_mut().unwrap())?),
                ".publics" => file.publics = Some(SMXPublicTable::new(&file.header, &section, file.names.as_mut().unwrap())?),
                ".pubvars" => file.pubvars = Some(SMXPubvarTable::new(&file.header, &section, file.names.as_mut().unwrap())?),
                ".tags" => file.tags = Some(SMXTagTable::new(&file.header, &section, file.names.as_mut().unwrap())?),
                ".data" => file.data = Some(SMXDataSection::new(&file.header, &section)?),
                ".code" => file.codev1 = Some(SMXCodeV1Section::new(&file.header, &section)?),
                ".dbg.files" => file.debug_files = Some(SMXDebugFilesTable::new(&file.header, &section, file.debug_names.as_mut().unwrap())?),
                ".dbg.lines" => file.debug_lines = Some(SMXDebugLinesTable::new(&file.header, &section)?),
                // .dbg.natives and .dbg.symbols is unimplemented due to being legacy
                ".dbg.methods" => file.debug_methods = Some(SMXDebugMethods::new(&file.header, &section)?), // names param is excluded as it's not used
                ".dbg.globals" => file.debug_globals = Some(SMXDebugGlobals::new(&file.header, &section)?),
                ".dbg.locals" => file.debug_locals = Some(SMXDebugLocals::new(&file, &file.header, &section)?),
                "rtti.data" => file.rtti_data = Some(SMXRTTIData::new(&file, &file.header, &section)),
                "rtti.classdefs" => file.rtti_classdefs = Some(SMXRTTIClassDefTable::new(&file.header, &section, file.names.as_mut().unwrap())?),
                "rtti.enumstructs" => file.rtti_enum_structs = Some(SMXRTTIEnumStructTable::new(&file.header, &section, file.names.as_mut().unwrap())?),
                "rtti.enumstruct_fields" => file.rtti_enum_struct_fields = Some(SMXRTTIEnumStructFieldTable::new(&file.header, &section, file.names.as_mut().unwrap())?),
                "rtti.fields" => file.rtti_fields = Some(SMXRTTIFieldTable::new(&file.header, &section, file.names.as_mut().unwrap())?),
                "rtti.methods" => file.rtti_methods = Some(SMXRTTIMethodTable::new(&file.header, &section, file.names.as_mut().unwrap())?),
                "rtti.natives" => file.rtti_natives = Some(SMXRTTINativeTable::new(&file.header, &section, file.names.as_mut().unwrap())?),
                "rtti.enums" => file.rtti_enums = Some(SMXRTTIEnumTable::new(&file.header, &section, file.names.as_mut().unwrap())?),
                "rtti.typedefs" => file.rtti_typedefs = Some(SMXRTTITypedefTable::new(&file.header, &section, file.names.as_mut().unwrap())?),
                "rtti.typesets" => file.rtti_typesets = Some(SMXRTTITypesetTable::new(&file.header, &section, file.names.as_mut().unwrap())?),
                _ =>  file.unknown_sections.push(section),
            }
        }

        // Legacy debug symbols table is skipped

        if file.publics.is_some() {
            for pubfun in file.publics.as_ref().unwrap().entries_ref() {
                V1Disassembler::diassemble(&file, file.codev1.as_ref().unwrap(), pubfun.address as i32);
            }
        }

        if file.called_functions.is_some() {
            for fun in file.called_functions.unwrap().entries_ref() {
                V1Disassembler::diassemble(&file, file.codev1.as_ref().unwrap(), fun.address as i32);
            }
        }

        Ok(file)
    }

    pub fn find_global_name(&mut self, addr: i32) -> Option<String> {
        if self.debug_globals.is_some() {
            let sym = self.debug_globals.as_mut().unwrap().find_global(addr);

            if sym.is_some() {
                return Some(self.names.as_mut().unwrap().string_at(sym.unwrap().name_offset).unwrap());
            }
        }

        None
    }

    pub fn find_local_name(&mut self, code_addr: i32, addr: i32) -> Option<String> {
        if self.debug_locals.is_some() {
            let entry = self.debug_locals.as_ref().unwrap().find_local(code_addr, addr);

            if entry.is_some() {
                return Some(self.names.as_mut().unwrap().string_at(entry.unwrap().name_offset).unwrap());
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
            for fun in self.called_functions.as_ref().unwrap().entries_ref() {
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
            for fun in self.called_functions.as_ref().unwrap().entries_ref() {
                if fun.address == addr as u32 {
                    return true;
                }
            }
        }

        false
    }
}