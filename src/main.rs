use json;
use std::fs::File;
use std::io::Result;
use std::io::Read;
use std::collections::HashMap;

use serial::prelude::*;

struct SerialOptions {
    port: dyn SerialPort
}

//impl dd
enum SerialOption {
    BaudRate(serial::BaudRate),
    CharSize(serial::CharSize),
    Parity(serial::Parity),
    StopBits(serial::StopBits),
    FlowControl(serial::FlowControl)
}

fn key_to_serial_config(key: &str) -> SerialOption {
    let mut map = HashMap::new();
    use serial::*;
    map.insert("xon/xoff", SerialOption::FlowControl(FlowControl::FlowSoftware));
    map.insert("8", SerialOption::CharSize(CharSize::Bits8));
    SerialOption::StopBits(StopBits::Stop1)
}

fn main() -> std::io::Result<()> {
    let mut config_schema = File::open("schema.json")?;
    let mut contents = String::new();
    config_schema.read_to_string(&mut contents)?;
    let schema = json::parse(&contents).expect("unable to parse json");

    Ok(())
}
