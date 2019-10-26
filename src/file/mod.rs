use crate::headers::{SMXHeader, SectionEntry};
use crate::sections::*;
use crate::rtti::*;

#[derive(Debug)]
pub struct SMXFile<'a> {
    pub header: SMXHeader,
    pub unknown_sections: Vec<SectionEntry>,

    pub names: SMXNameTable<'a>,
    pub debug_names: SMXNameTable<'a>,
    pub natives: SMXNativeTable,
    pub publics: SMXPublicTable,
    pub pubvars: SMXPubvarTable,
    pub tags: SMXTagTable,
    pub data: SMXDataSection<'a>,
    pub codev1: SMXCodeV1Section<'a>,
    pub called_functions: SMXCalledFunctionsTable,

    pub debug_info: SMXDebugInfoSection,
    pub debug_files: SMXDebugFilesTable,
    pub debug_lines: SMXDebugLinesTable,
    pub debug_symbols: SMXDebugSymbols,

    pub rtti_data: SMXRTTIData<'a>,
    pub rtti_enums: SMXRTTIEnumTable,
    pub rtti_enum_structs: SMXRTTIEnumStructTable,
    pub rtti_enum_struct_fields: SMXRTTIEnumStructFieldTable,
    pub rtti_classdefs: SMXRTTIClassDefTable,
    pub rtti_fields:  SMXRTTIFieldTable,
    pub rtti_methods: SMXRTTIMethodTable,
    pub rtti_natives: SMXRTTINativeTable,
    pub rtti_typedefs: SMXRTTITypedefTable,
    pub rtti_typesets: SMXRTTITypesetTable,

    pub debug_methods: SMXDebugMethods,
    pub debug_globals: SMXDebugGlobals,
    // pub debug_locals
}