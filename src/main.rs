use json;
use std::fs::File;
use std::io::Result;
use std::io::Read;
use std::collections::HashMap;
use std::option::Option;

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

fn key_to_serial_config(key: &str) -> Option<SerialOption> {
    let mut map = HashMap::new();
    use serial::*;
    map.insert("xon/xoff", SerialOption::FlowControl(FlowControl::FlowSoftware));
    map.insert("rts/cts", SerialOption::FlowControl(FlowControl::FlowHardware));
    map.insert("110", SerialOption::BaudRate(BaudRate::Baud110));
    map.insert("300", SerialOption::BaudRate(BaudRate::Baud300));
    map.insert("600", SerialOption::BaudRate(BaudRate::Baud600));
    map.insert("1200", SerialOption::BaudRate(BaudRate::Baud1200));
    map.insert("2400", SerialOption::BaudRate(BaudRate::Baud2400));
    map.insert("4800", SerialOption::BaudRate(BaudRate::Baud4800));
    map.insert("9600", SerialOption::BaudRate(BaudRate::Baud9600));
    map.insert("19200", SerialOption::BaudRate(BaudRate::Baud19200));
    map.insert("38400", SerialOption::BaudRate(BaudRate::Baud38400));
    map.insert("57600", SerialOption::BaudRate(BaudRate::Baud57600));
    map.insert("115200", SerialOption::BaudRate(BaudRate::Baud115200));
    map.insert("5", SerialOption::CharSize(CharSize::Bits5));
    map.insert("6", SerialOption::CharSize(CharSize::Bits6));
    map.insert("7", SerialOption::CharSize(CharSize::Bits7));
    map.insert("8", SerialOption::CharSize(CharSize::Bits8));
    map.insert("odd", SerialOption::Parity(Parity::ParityOdd));
    map.insert("even", SerialOption::Parity(Parity::ParityEven));
    map.insert("1", SerialOption::StopBits(StopBits::Stop1));
    map.insert("2", SerialOption::StopBits(StopBits::Stop2));

    map.remove(key)
}

fn maybe_set_option(spec: json::JsonValue, subkey: &str, mut settings: serial::PortSettings) -> Option<serial::PortSettings> {//, mut options: SerialOptions) {
    let serial_config = &spec["serial-config"];//.expect("Invalid serial config spec, missing the 'serial-config' key");
    let option: Option<SerialOption>;
    option = match &serial_config[subkey] {
        json::JsonValue::String(string) => {
            key_to_serial_config(&string)
        },
        json::JsonValue::Number(number) => {
            key_to_serial_config(&number.to_string())
        },
        _ => {
            None
        }
    };
    match option {
        Some(thing) => {
            match thing {
                SerialOption::FlowControl(flow) => {
                    settings.set_flow_control(flow);
                    Some(settings)
                },
                _ => None
            }
        },
        None => None
    }
}

fn main() -> std::io::Result<()> {
    let mut config_schema = File::open("schema.json")?;
    let mut contents = String::new();
    config_schema.read_to_string(&mut contents)?;
    let schema = json::parse(&contents).expect("unable to parse json");

    //let mut port : SerialOptions;


    Ok(())
}
