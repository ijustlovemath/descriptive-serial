use json;
//use json::JsonValue;
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
#[derive(Debug)]
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
    let key = &serial_config[subkey];
    option = match key {
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
            println!("[WARNING] Unexpected type found for {:?}", key);
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
                },
                _ => None
            }
        },
        None => {
            println!("[WARNING] Unrecognized option {:?} for setting {}, using default...", key, subkey);
            None
        }
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

#[derive(Debug, PartialEq, Clone)]
struct SerialState<'a> {
    next: Option<&'a SerialState<'a>>,
    name: String,
    kind: String, // TODO enum
    template: String, // TODO SerialStateTemplate struct
    format: String, // TODO enum
    contents: Option<String>, // TODO SerialStateContents struct
}

fn state_constructor(name: &str, state_spec: json::JsonValue) -> SerialState {
    // states have the following structure:
    // {
    //  "template" : {} // optional, not required
    //  "type" : "send" or "receive"
    //  "name" : user defined, should be equal to name parameter
    //  "format" : "header", "payload", or "header-then-payload"
    //  "contents" : specific to what "format" is
    // }
    assert_eq!(name, state_spec["name"]);
    // we're gonna ignore template states for now...
    //
    SerialState {
        next: None,
        name: name.to_string(),
        kind: state_spec["type"].to_string(),
        template: "unsupported".to_string(),
        format: state_spec["format"].to_string(),
        contents: None,
    }
}

fn test_state_constructor() {
    let jsono = fake_jsonobj();
    let actual = state_constructor("foo", jsono);
    let expected = fake_serialstate();
    assert_eq!(actual, expected);

}

fn fake_jsonobj() -> json::JsonValue {
    json::parse(r#"
            {
                "name":"foo",
                "type":"send",
                "format":"hello"
            }"#).unwrap()
}

fn fake_serialstate<'a> () -> SerialState<'a> {
    SerialState {
         next: None,
         name: "foo".to_string(),
         kind: "send".to_string(),
         template: "unsupported".to_string(),
         format: "hello".to_string(),
         contents: None
    }
}

// TODO: the function signature for this feels wrong... state_spec contains name
// cause lookups can be done with a [], so what is this ~actually doing~? 
// needs a better name
fn state_lookup_build<'a>(state_spec: json::JsonValue, mut lookup: HashMap<String, SerialState<'a>>) 
    //-> HashMap<&'a str, SerialState<'a>> {
    -> (HashMap<String, SerialState<'a>>, SerialState<'a>) {

    let name = match &state_spec["name"] {
        json::JsonValue::String(string) => *string,
        json::JsonValue::Short(short) => short.to_string(),
        _ => {
            panic!("unexpected type at 'name' key -> {:?}", state_spec);
        }
    }.to_string();


    if lookup.contains_key(&name) {
        panic!("duplicate states not allowed in the specification, state with name '{}' is already defined somewhere else!", name);
    }
    lookup.entry(name).or_insert(state_constructor(&name.clone(), state_spec));
    // we have to do this because rust is stupid about rvalues (ok i know it's not but still)
    let state = lookup[&name].clone();
    (lookup, state)
}

fn get_type_string<T>(_: &T) -> String {
    format!("{:?}", std::any::type_name::<T>())
}

fn build_state_lookup<'a>(states_spec: json::Array) -> HashMap<String, SerialState<'a>> {
    let mut lookup = HashMap::new();
    for state in states_spec {
        // code smell here; why drop _? why get name when it's avail in state?
        let res = state_lookup_build(state, lookup.clone());
        lookup = res.0;
    }
    lookup
}

fn test_build_state_lookup() -> std::io::Result<()> {
    let mut config_schema = File::open("schema.json")?;
    let mut contents = String::new();
    config_schema.read_to_string(&mut contents)?;
    let schema = json::parse(&contents).expect("unable to parse json");

    let lookup = match &schema["states"] {
        json::JsonValue::Array(states) => {
            build_state_lookup(states.to_vec())
        },
        _ => {
            panic!("unexpected type at states key");
        }
    };
    println!("states: {:?}", lookup);
    Ok(())
}

//fn link_states<'a>(mut lookup: HashMap<

fn test_state_lookup_build() {
    let map = HashMap::new();
    let name = "foo";
    let spec = fake_jsonobj();
    let (map, state) = state_lookup_build(spec, map);
    assert_eq!(map[name].name, "foo".to_string());
    let expected = fake_serialstate();
    assert_eq!(state, expected);
}

fn main() -> std::io::Result<()> {
    let mut config_schema = File::open("schema.json")?;
    let mut contents = String::new();
    config_schema.read_to_string(&mut contents)?;
    let schema = json::parse(&contents).expect("unable to parse json");

    let mut settings = serial::PortSettings {
        baud_rate: serial::BaudRate::Baud9600,
        char_size: serial::CharSize::Bits8,
        parity: serial::Parity::ParityNone,
        stop_bits: serial::StopBits::Stop1,
        flow_control: serial::FlowControl::FlowNone
    };
    for key in &["parity", "flow-control", "baud", "data-bits", "stop-bits"] {
        settings = set_option(&schema, key, settings);
    }
    println!("{:?}", settings);
    //let mut port : SerialOptions;


    test_state_constructor();
    test_state_lookup_build();
    test_build_state_lookup();
    Ok(())
}
