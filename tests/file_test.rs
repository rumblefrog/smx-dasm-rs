use std::fs::File;
use std::io::Read;

extern crate smxdasm;

#[test]
fn test_file() {
    let mut file = File::open("F:\\Github\\smx-dasm-rs\\tests\\Source-Chat-Relay.smx").unwrap();

    let mut data = Vec::new();

    file.read_to_end(&mut data).unwrap();

    let c = smxdasm::file::SMXFile::new(data).unwrap();
    let f = c.borrow();

    println!("========== HEADER ==========");
    println!("Magic: {}", f.header.magic);
    println!("Version: {}", f.header.version);
    println!("Compression Type: {}", f.header.compression_type);
    println!("Disk Size: {}", f.header.disk_size);
    println!("Image Size: {}", f.header.image_size);
    println!("Section Count: {}", f.header.section_count);
    println!("String Table Offset: {}", f.header.string_table_offset);
    println!("Data Offset: {}", f.header.data_offset);
    println!("Debug Packed: {}", f.header.debug_packed);
    println!("========== HEADER ==========");

    if let Some(opt) = &f.names {
        let names = opt.borrow();
        println!("========== Name Table Names ==========");
        for (_, name) in &names.names() {
            println!("{}", name);
        }
        println!("========== Name Table Names ==========");
    }

    if let Some(opt) = &f.debug_names {
        let names = opt.borrow();
        println!("========== Debug Name Table Names ==========");
        for (_, name) in &names.names() {
            println!("{}", name);
        }
        println!("========== Debug Name Table Names ==========");
    }

    if let Some(opt) = &f.natives {
        println!("========== Native Entries ==========");
        for native in &opt.entries() {
            println!("======");
            println!("Name: {}", native.name);
            println!("Name Offset: {}", native.name_offset);
            println!("======");
        }
        println!("========== Native Entries ==========");
    }

    if let Some(opt) = &f.publics {
        println!("========== Public Entries ==========");
        for native in &opt.entries() {
            println!("======");
            println!("Name: {}", native.name);
            println!("Name Offset: {}", native.name_offset);
            println!("Address: {}", native.address);
            println!("======");
        }
        println!("========== Public Entries ==========");
    }

    if let Some(opt) = &f.pubvars {
        println!("========== Public Variables Entries ==========");
        for native in &opt.entries() {
            println!("======");
            println!("Name: {}", native.name);
            println!("Name Offset: {}", native.name_offset);
            println!("Address: {}", native.address);
            println!("======");
        }
        println!("========== Public Variables Entries ==========");
    }

    if let Some(opt) = &f.tags {
        println!("========== Tag Entries ==========");
        for tag in &opt.entries() {
            println!("======");
            println!("Name: {}", tag.entry().name);
            println!("Name Offset: {}", tag.entry().name_offset);
            println!("Tag: {}", tag.entry().tag);
            println!("ID: {}", tag.id());
            println!("Value: {}", tag.value());
            println!("Flags: {}", tag.flags());
            println!("======");
        }
        println!("========== Tag Entries ==========");
    }

    if let Some(opt) = &f.data {
        println!("========== Data Header ==========");
        let header = opt.header();
        println!("Data Size: {}", header.data_size);
        println!("Memory Size: {}", header.memory_size);
        println!("Data Offset: {}", header.data_offset);
        println!("========== Data Header ==========");
    }

    if let Some(opt) = &f.codev1 {
        println!("========== CodeV1 Section ==========");
        let header = opt.header();
        println!("Code Size: {}", header.code_size);
        println!("Cell Size: {}", header.cell_size);
        println!("Code Version: {}", header.code_version);
        println!("Flags: {:?}", header.flags);
        println!("Main Offset: {}", header.main_offset);
        println!("Code Offset: {}", header.code_offset);
        println!("Features: {}", header.features);
        println!("========== CodeV1 Section ==========");
    }

    if let Some(opt) = &f.called_functions {
        let functions = opt.borrow();
        println!("========== Called Function Entries ==========");
        for func in &functions.entries() {
            println!("======");
            println!("Address: {}", func.address);
            println!("Name: {}", func.name);
            println!("======");
        }
        println!("========== Called Function Entries ==========");
    }

    if let Some(opt) = &f.debug_info {
        println!("========== Debug Info Section ==========");
        println!("File Count: {}", opt.file_count());
        println!("Line Count: {}", opt.line_count());
        println!("Symbol Count: {}", opt.symbol_count());
        println!("Array Count: {}", opt.array_count());
        println!("========== Debug Info Section ==========");
    }

    if let Some(opt) = &f.debug_files {
        println!("========== Debug Files ==========");
        for file in &opt.entries() {
            println!("======");
            println!("Address: {}", file.address);
            println!("Name Offset: {}", file.name_offset);
            println!("Name: {}", file.name);
            println!("======");
        }
        println!("========== Debug Files ==========");
    }

    if let Some(opt) = &f.debug_lines {
        println!("========== Debug Lines ==========");
        for l in &opt.entries() {
            println!("======");
            println!("Address: {}", l.address);
            println!("Line: {}", l.line);
            println!("======");
        }
        println!("========== Debug Lines ==========");
    }

    if let Some(opt) = &f.rtti_enums {
        println!("========== RTTI Enums ==========");
        for e in &opt.enums() {
            println!("{}", e);
        }
        println!("========== RTTI Enums ==========");
    }

    if let Some(opt) = &f.rtti_enum_structs {
        println!("========== RTTI Enum Structs ==========");
        for e in &opt.entries() {
            println!("======");
            println!("Name Offset: {}", e.name_offset);
            println!("First Field: {}", e.first_field);
            println!("Size: {}", e.size);
            println!("Name: {}", e.name);
            println!("======");
        }
        println!("========== RTTI Enum Structs ==========");
    }

    if let Some(opt) = &f.rtti_enum_struct_fields {
        println!("========== RTTI Enum Struct Fields ==========");
        for e in &opt.entries() {
            println!("======");
            println!("Name Offset: {}", e.name_offset);
            println!("Type ID: {}", e.type_id);
            println!("Offset: {}", e.offset);
            println!("Name: {}", e.name);
            println!("======");
        }
        println!("========== RTTI Enum Struct Fields ==========");
    }

    if let Some(opt) = &f.rtti_classdefs {
        println!("========== RTTI Class Definitions ==========");
        for c in &opt.defs() {
            println!("======");
            println!("Flags: {}", c.flags);
            println!("Name Offset: {}", c.name_offset);
            println!("First Field: {}", c.first_field);
            println!("Name: {}", c.name);
            println!("======");
        }
        println!("========== RTTI Class Definitions ==========");
    }

    if let Some(opt) = &f.rtti_fields {
        println!("========== RTTI Fields ==========");
        for f in &opt.fields() {
            println!("======");
            println!("Flags: {}", f.flags);
            println!("Name Offset: {}", f.name_offset);
            println!("Type ID: {}", f.type_id);
            println!("Name: {}", f.name);
            println!("======");
        }
        println!("========== RTTI Fields ==========");
    }

    if let Some(opt) = &f.rtti_methods {
        println!("========== RTTI Methods ==========");
        for m in &opt.methods() {
            println!("======");
            println!("Name: {}", m.name);
            println!("PCode Start: {}", m.pcode_start);
            println!("PCode End: {}", m.pcode_end);
            println!("Signature: {}", m.signature);
            println!("======");
        }
        println!("========== RTTI Methods ==========");
    }

    if let Some(opt) = &f.rtti_natives {
        println!("========== RTTI Natives ==========");
        for n in &opt.natives() {
            println!("======");
            println!("Name: {}", n.name);
            println!("Signature: {}", n.signature);
            println!("======");
        }
        println!("========== RTTI Natives ==========");
    }

    if let Some(opt) = &f.rtti_typedefs {
        println!("========== RTTI Type Definitions ==========");
        for t in &opt.typedefs() {
            println!("======");
            println!("Name: {}", t.name);
            println!("Type ID: {}", t.type_id);
            println!("======");
        }
        println!("========== RTTI Type Definitions ==========");
    }

    if let Some(opt) = &f.rtti_typesets {
        println!("========== RTTI Type Sets ==========");
        for t in &opt.typesets() {
            println!("======");
            println!("Name: {}", t.name);
            println!("Signature: {}", t.signature);
            println!("======");
        }
        println!("========== RTTI Type Sets ==========");
    }

    if let Some(opt) = &f.debug_methods {
        println!("========== Debug Methods ==========");
        for m in &opt.entries() {
            println!("======");
            println!("Method Index: {}", m.method_index);
            println!("First Local: {}", m.first_local);
            println!("======");
        }
        println!("========== Debug Methods ==========");
    }

    if let Some(opt) = &f.debug_globals {
        let globals = opt.borrow();
        println!("========== Debug Globals ==========");
        for g in &globals.symbol_entries() {
            println!("======");
            println!("Address: {}", g.address);
            println!("Scope: {}", g.scope);
            println!("Name Offset: {}", g.name_offset);
            println!("Code Start: {}", g.code_start);
            println!("Code End: {}", g.code_end);
            println!("Type ID: {}", g.type_id);
            println!("======");
        }
        println!("========== Debug Globals ==========");
    }

    if let Some(opt) = &f.debug_locals {
        println!("========== Debug Locals ==========");
        for l in &opt.symbol_entries() {
            println!("======");
            println!("Address: {}", l.address);
            println!("Scope: {}", l.scope);
            println!("Name Offset: {}", l.name_offset);
            println!("Code Start: {}", l.code_start);
            println!("Code End: {}", l.code_end);
            println!("Type ID: {}", l.type_id);
            println!("======");
        }
        println!("========== Debug Locals ==========");
    }
}