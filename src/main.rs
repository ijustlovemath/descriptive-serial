use json;
use std::fs::File;
use std::io::Result;
use std::io::Read;
use std::collections::HashMap;
use std::option::Option;
use std::vec::Vec;

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

macro_rules! insert_baud {
    ($rate:literal) => {
        map.insert(stringify!($rate), SerialOption::BaudRate(BaudRate::concat_idents!(Baud, $rate)));
    }
}

fn key_to_serial_config(key: &str) -> Option<SerialOption> {
    let mut map = HashMap::new();
    use serial::*;
    map.insert("xon/xoff", SerialOption::FlowControl(FlowControl::FlowSoftware));
    map.insert("rts/cts", SerialOption::FlowControl(FlowControl::FlowHardware));
    // There has to be a better way to do baud rates... macros?
    map.insert("110", SerialOption::BaudRate(BaudRate::Baud110));
//    insert_baud!(110);
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
struct SerialState {
    next: Option<usize>,
    name: String,
    kind: String, // TODO enum
    template: String, // TODO SerialStateTemplate struct
    format: String, // TODO enum
    contents: Option<String>, // TODO SerialStateContents struct
}

fn state_constructor(state_spec: &json::JsonValue) -> SerialState {
    // states have the following structure:
    // {
    //  "template" : {} // optional, not required
    //  "type" : "send" or "receive"
    //  "name" : user defined, should be equal to name parameter
    //  "format" : "header", "payload", or "header-then-payload"
    //  "contents" : specific to what "format" is
    // }
    // we're gonna ignore template states for now...
    //
    SerialState {
        next: None,
        name: state_spec["name"].to_string(),
        kind: state_spec["type"].to_string(),
        template: "unsupported".to_string(),
        format: state_spec["format"].to_string(),
        contents: None,
    }
}

fn test_state_constructor() {
    let jsono = fake_jsonobj();
    let actual = state_constructor(&jsono);
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

fn fake_serialstate () -> SerialState {
    SerialState {
         next: None,
         name: "foo".to_string(),
         kind: "send".to_string(),
         template: "unsupported".to_string(),
         format: "hello".to_string(),
         contents: None
    }
}

fn state_string(state_spec: &json::JsonValue, key: &str) -> String {
    match state_string_maybe(state_spec, key) {
        Some(value) => value,
        None => {
            panic!("unexpected type at '{}' key -> {:?}", key, state_spec);
        }
    }
}

fn state_string_maybe(state_spec: &json::JsonValue, key: &str) -> Option<String> {
    match &state_spec[key] {
        json::JsonValue::String(string) => Some(String::from(string)),
        json::JsonValue::Short(short) => Some(short.to_string()),
        _ => {
            None
        }
    }
}

fn state_name(state_spec: &json::JsonValue) -> String {
    state_string(state_spec, "name")
}

fn check_create_state(state_spec: &json::JsonValue, lookup: &HashMap<String, usize>) -> SerialState {
    let name = state_name(&state_spec);
    
    if lookup.contains_key(&name) {
        panic!("duplicate states not allowed in the specification, state with name '{}' is already defined somewhere else!", name);
    }

    state_constructor(&state_spec)
}

// TODO: the function signature for this feels wrong... state_spec contains name
// cause lookups can be done with a [], so what is this ~actually doing~? 
// needs a better name
fn state_lookup_build(state_spec: json::JsonValue, mut lookup: HashMap<String, SerialState>) 
    //-> HashMap<&'a str, SerialState<'a>> {
    -> (HashMap<String, SerialState>, SerialState) {

    let name = state_name(&state_spec);


    if lookup.contains_key(&name) {
        panic!("duplicate states not allowed in the specification, state with name '{}' is already defined somewhere else!", name);
    }

    lookup.insert(name.clone(), state_constructor(&state_spec));
    //lookup.entry(name).or_insert(state_constructor(&name.clone(), state_spec));
    // we have to do this because rust is stupid about rvalues (ok i know it's not but still)
    let state = lookup[&name].clone();
    (lookup, state)
}

fn get_type_string<T>(_: &T) -> String {
    format!("{:?}", std::any::type_name::<T>())
}

fn build_state_lookup(states_spec: json::Array) -> HashMap<String, SerialState> {
    let mut lookup = HashMap::new();
    for state in states_spec {
        // code smell here; why drop _? why get name when it's avail in state?
        let res = state_lookup_build(state, lookup.clone());
        lookup = res.0;
    }
    lookup
}

fn run_fsm(machine: &Vec<SerialState>) {
    if machine.len() == 0 {
        println!("no states to run");
        return;
    }
    let mut current = &machine[0];
    
    loop {
        //process_state(&current);
        println!("current state: {:?}", current);
        if let Some(next_id) = current.next {
            current = &machine[next_id];
            continue
        } else {
            println!("terminal state reached, breaking out...");
            break
        }
    }
}

fn test_run_fsm() -> std::io::Result<()> {
    let mut config_schema = File::open("schema.json")?;
    let mut contents = String::new();
    config_schema.read_to_string(&mut contents)?;
    let schema = json::parse(&contents).expect("unable to parse json");

    match &schema["states"] {
        json::JsonValue::Array(states) => {
            let (graph, _) = link_states(states.to_vec());
            //run_fsm(&graph);
            to_dot(&graph);
        },
        _ => {
            panic!("unexpected type at states key");
        }
    };
    Ok(())
}

fn alphabet(integer: usize) -> char {
    char::from(integer as u8 + b'a')
}

fn to_dot(graph: &Vec<SerialState>) {
    println!("digraph serial_state_machine {{");
    for (i, state) in graph.iter().enumerate() {
        let next = match state.next {
            Some(index) => alphabet(index).to_string(),
            None => "stop".to_string()
        };
        println!("\t{}[label=\"{}\"];", alphabet(i), state.name);
        println!("\t{} -> {};", alphabet(i), next); 
    }
    println!("}}");
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



fn test_link_states() -> std::io::Result<()> {
    let mut config_schema = File::open("schema.json")?;
    let mut contents = String::new();
    config_schema.read_to_string(&mut contents)?;
    let schema = json::parse(&contents).expect("unable to parse json");

    match &schema["states"] {
        json::JsonValue::Array(states) => {
            link_states(states.to_vec())
        },
        _ => {
            panic!("unexpected type at states key");
        }
    };
    Ok(())

}

fn link_states(states_spec: json::Array) -> (Vec<SerialState>, HashMap<String, usize>) {
    //for state in 
    //alarm bells are already ringing about this... the borrow checker will not like medoing
    //dynamic links between states. shit.
    //we can try doing this: https://rust-leipzig.github.io/architecture/2016/12/20/idiomatic-trees-in-rust/
    let arena = &mut Vec::new();
    let id_lookup = &mut HashMap::new();
    // TODO: just put nodeId's in map, not the actual states
    for state_spec in &states_spec {
        let name = state_name(&state_spec);
        let state = check_create_state(state_spec, id_lookup);

        // simple arena is a vector
        let id = arena.len(); // This is the index at which the state will be inserted
        arena.push(state);
        id_lookup.insert(name, id);
    }
    // once every state has been populated, we can define next states
    // all states should have an ID at this point
    for state_spec in &states_spec {
        let name = state_name(&state_spec);
        let id = id_lookup[&name];

        // The next state may be 'null', in which case we need to not do anything (since that's the
        // default)
        let next_name: String;
        match state_string_maybe(&state_spec, "next") {
            Some(string) => {
                next_name = string
            },
            None => {
                continue; // we've reached a stop state, nothing to do here
            }
        }
        if ! id_lookup.contains_key(&next_name) {
            panic!("There is no state named '{}' in the state machine, perhaps you made a typo?", next_name);
            // TODO: see which state name is closest and suggest it
        }
        let next_id = id_lookup[&next_name];
        
        let error_msg = format!("A state named {} with ID {} should exist, but doesn't", name, id);
        let state = arena.get_mut(id).expect(&error_msg);
        state.next = Some(next_id);
    }

    (arena.to_vec(), id_lookup.clone())
}

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
    //println!("{:?}", settings);
    //let mut port : SerialOptions;


    //test_state_constructor();
    //test_state_lookup_build();
    //test_build_state_lookup();
    //test_link_states();
    test_run_fsm();
    Ok(())
}
