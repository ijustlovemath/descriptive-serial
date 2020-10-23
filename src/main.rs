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
    DataBits(serial::CharSize),
    Parity(serial::Parity),
    StopBits(serial::StopBits),
    FlowControl(serial::FlowControl)
}

fn key_to_serial_config(key: &str) -> Option<SerialOption> {
    let mut map = HashMap::new();
    use serial::*;
    map.insert("xon/xoff", SerialOption::FlowControl(FlowControl::FlowSoftware));
    map.insert("rts/cts", SerialOption::FlowControl(FlowControl::FlowHardware));
    // There has to be a better way to do baud rates... macros?
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
    map.insert("B110", SerialOption::BaudRate(BaudRate::Baud110));
    map.insert("B300", SerialOption::BaudRate(BaudRate::Baud300));
    map.insert("B600", SerialOption::BaudRate(BaudRate::Baud600));
    map.insert("B1200", SerialOption::BaudRate(BaudRate::Baud1200));
    map.insert("B2400", SerialOption::BaudRate(BaudRate::Baud2400));
    map.insert("B4800", SerialOption::BaudRate(BaudRate::Baud4800));
    map.insert("B9600", SerialOption::BaudRate(BaudRate::Baud9600));
    map.insert("B19200", SerialOption::BaudRate(BaudRate::Baud19200));
    map.insert("B38400", SerialOption::BaudRate(BaudRate::Baud38400));
    map.insert("B57600", SerialOption::BaudRate(BaudRate::Baud57600));
    map.insert("B115200", SerialOption::BaudRate(BaudRate::Baud115200));
    map.insert("5", SerialOption::DataBits(CharSize::Bits5));
    map.insert("6", SerialOption::DataBits(CharSize::Bits6));
    map.insert("7", SerialOption::DataBits(CharSize::Bits7));
    map.insert("8", SerialOption::DataBits(CharSize::Bits8));
    map.insert("odd", SerialOption::Parity(Parity::ParityOdd));
    map.insert("even", SerialOption::Parity(Parity::ParityEven));
    map.insert("1", SerialOption::StopBits(StopBits::Stop1));
    map.insert("2", SerialOption::StopBits(StopBits::Stop2));

    map.remove(key)
}

fn maybe_set_option(spec: &json::JsonValue, subkey: &str, mut settings: serial::PortSettings) -> Option<serial::PortSettings> {//, mut options: SerialOptions) {
    let serial_config = &spec["serial-config"];//.expect("Invalid serial config spec, missing the 'serial-config' key");
    let option: Option<SerialOption>;
    option = match &serial_config[subkey] {
        json::JsonValue::String(string) => {
            key_to_serial_config(&string)
        },
        // TODO: use | here to make sure strings and shorts do the same thing
        json::JsonValue::Short(string) => {
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
                    Some(settings) // TODO: reduce this boilerplate?
                },
                SerialOption::BaudRate(speed) => {
                    settings.set_baud_rate(speed).expect("Problem setting baud rate");
                    Some(settings)
                },
                SerialOption::StopBits(stop) => {
                    settings.set_stop_bits(stop);
                    Some(settings)
                },
                SerialOption::DataBits(data) => {
                    settings.set_char_size(data);
                    Some(settings)
                },
                SerialOption::Parity(parity) => {
                    settings.set_parity(parity);
                    Some(settings)
                }
                _ => None
            }
        },
        None => None
    }
}

fn set_option(spec: &json::JsonValue, subkey: &str, settings: serial::PortSettings) -> serial::PortSettings {
    match maybe_set_option(spec, subkey, settings) {
        Some(new) => { 
            new
        },
        None => {
            settings
        }
    }
}

fn main() -> std::io::Result<()> {
    let mut config_schema = File::open("schema.json")?;
    let mut contents = String::new();
    config_schema.read_to_string(&mut contents)?;
    let schema = json::parse(&contents).expect("unable to parse json");
    println!("{:?}", schema);

    let mut settings = serial::PortSettings {
        baud_rate: serial::BaudRate::Baud9600,
        char_size: serial::CharSize::Bits8,
        parity: serial::Parity::ParityNone,
        stop_bits: serial::StopBits::Stop1,
        flow_control: serial::FlowControl::FlowNone
    };
    settings = set_option(&schema, "baud", settings);
    settings = set_option(&schema, "data-bits", settings);
    //settings = set_option(schema, "stop-bits", 
    for key in &["parity", "flow-control"] {
        settings = set_option(&schema, key, settings);
    }
    println!("{:?}", settings);
    //let mut port : SerialOptions;


    Ok(())
}
