use std::fs::File;
use std::io::Read;

extern crate smxdasm;

#[test]
fn test_header() {
    let mut file = File::open("F:\\Github\\smx-dasm-rs\\tests\\Source-Chat-Relay.smx").unwrap();

    let mut data = Vec::new();

    file.read_to_end(&mut data).unwrap();

    let d = smxdasm::headers::SMXHeader::new(data).unwrap();

    println!("{:?}", d);
}