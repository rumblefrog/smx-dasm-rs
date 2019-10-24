#[derive(Debug, Clone)]
pub enum CodeV1Flags {
    Debug,
}

impl CodeV1Flags {
    pub fn value(&self) -> u16 {
        match *self {
            CodeV1Flags::Debug => 0x0000_0001,
        }
    }
}

// The ".code" section.
#[derive(Debug, Clone)]
pub struct CodeV1Header {
    // Size of the code blob.
    pub code_size: i32,

    // Size of a cell in bytes (always 4).
    pub cell_size: u8,

    // Code version (see above constants).
    pub code_version: u8,

    // Flags (see above).
    pub flags: u16,

    // Offset within the code blob to the entry point function.
    pub main_offset: i32,

    // Offset to the code section.
    pub code_offset: i32,

    // Feature set.
    pub features: i32,
}

impl CodeV1Header {
    pub const SIZE: i32 = 16;

    pub const VERSION_JIT1: u8 = 9;
    pub const VERSION_JIT2: u8 = 10;
}