use std::io;
use std::time::Duration;

use std::io::prelude::*;
use serde::{Serialize, Deserialize};
use serial::prelude::*;

const RELAY_A04_PATH: &str = "/dev/serial/by-id/usb-deciphe_it_LucidIo_AO4_11001001FA480000-if00";

// Request and response frames for the LucidControl protocol.
// See https://www.lucid-control.com/wp-content/uploads/2013/07/User-Manual-LucidControl-AO4.pdf
#[derive(Serialize, Deserialize, Debug)]
struct SetIOFrame {
    opcode: u8,
    channel: u8,
    value_type: u8,
    length: u8,
    value: [u8; 2],
}

#[derive(Serialize, Deserialize, Debug)]
struct SetIOResponse {
    status: u8,
    length: u8,
}

#[derive(Serialize, Deserialize, Debug)]
struct GetIOFrame {
    opcode: u8,
    channel: u8,
    value_type: u8,
    length: u8,
}

#[derive(Serialize, Deserialize, Debug)]
struct GetIOResponse {
    status: u8,
    length: u8,
    value: [u8; 2],
}

fn read() {
    // User input for channel number.
    println!("Please provide channel number:");
    print!("> ");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    match io::stdin().read_line(&mut input) {
        Ok(_) => {},
        Err(error) => {println!("Error reading channel number: {}", error); return},
    }
    let channel: i32 = match input.trim().parse() {
        Ok(num) => num,
        Err(error) => {println!("Error parsing input to number: {}", error); return},
    };

    // Construct frame.
    let frame: GetIOFrame = GetIOFrame {
        opcode: 0x46,
        channel: channel as u8,
        value_type: 0x1C,
        length: 0x00,
    };

    // Encode frame into binary data and send over serial pipe.
    let encoded: Vec<u8> = bincode::serialize(&frame).unwrap();
    let mut port = serial::open(RELAY_A04_PATH).unwrap();
    port.reconfigure(&|settings| {
        settings.set_baud_rate(serial::Baud9600)?;
        settings.set_char_size(serial::Bits8);
        settings.set_parity(serial::ParityNone);
        settings.set_stop_bits(serial::Stop1);
        settings.set_flow_control(serial::FlowNone);
        Ok(())
    }).unwrap();
    port.set_timeout(Duration::from_millis(1000)).unwrap();
    match port.write(&encoded) {
        Ok(_) => {},
        Err(error) => {println!("Error writing to serial port: {}", error); return},
    }

    // Read frame back and output mV value.
    let mut reply: [u8; 4]= [0; 4];
    match port.read(&mut reply) {
        Ok(_) => {},
        Err(error) => {println!("Error reading from serial port: {}", error); return},
    }
    let response: GetIOResponse = bincode::deserialize(&reply).unwrap();

    println!("Voltage from channel {} is {} mV", channel, i16::from_le_bytes(response.value));
}

fn write() {
    // Read user input for channel number and mV value.
    println!("Please provide channel number: ");
    print!("> ");
    io::stdout().flush().unwrap();
    let mut input = String::new();
    match io::stdin().read_line(&mut input) {
        Ok(_) => {},
        Err(error) => {println!("Error reading channel number: {}", error); return},
    }
    let channel: i32 = match input.trim().parse() {
        Ok(num) => num,
        Err(error) => {println!("Error parsing input to number: {}", error); return},
    };

    println!("Please provide voltage to set, in mV: ");
    print!("> ");
    io::stdout().flush().unwrap();
    let mut input = String::new();
    match io::stdin().read_line(&mut input) {
        Ok(_) => {},
        Err(error) => {println!("Error reading voltage: {}", error); return},
    }
    let voltage: i16 = match input.trim().parse() {
        Ok(num) => num,
        Err(error) => {println!("Error parsing input to number: {}", error); return},
    };

    // Construct frame.
    let frame: SetIOFrame = SetIOFrame {
        opcode: 0x40,
        channel: channel as u8,
        value_type: 0x1C,
        length: 2,
        value: voltage.to_le_bytes(),
    };

    // Encode frame into binary data and send over serial port.
    let encoded: Vec<u8> = bincode::serialize(&frame).unwrap();
    let mut port = serial::open(RELAY_A04_PATH).unwrap();
    port.reconfigure(&|settings| {
        settings.set_baud_rate(serial::Baud9600)?;
        settings.set_char_size(serial::Bits8);
        settings.set_parity(serial::ParityNone);
        settings.set_stop_bits(serial::Stop1);
        settings.set_flow_control(serial::FlowNone);
        Ok(())
    }).unwrap();
    port.set_timeout(Duration::from_millis(1000)).unwrap();
    match port.write(&encoded) {
        Ok(_) => {},
        Err(error) => {println!("Error writing to serial port: {}", error); return},
    }

    // Read response frame. If error code is anything but zero, make sure to print error code.
    let mut reply: [u8; 2]= [0; 2];
    match port.read(&mut reply) {
        Ok(_) => {},
        Err(error) => {println!("Error reading from serial port: {}", error); return},
    }
    let response: SetIOResponse = bincode::deserialize(&reply).unwrap();
    if response.status == 0 {
        println!("Voltage set successfully.");
    } else {
        println!("Voltage set failed with error code {:x?}. Read LucidControl manual to decipher.", response.status);
    }
}

fn main() {
    println!("Home Relay Read/Writer");
    println!("----------------------");
    loop {
        println!("Please select operation.");
        println!("[1] Read voltage");
        println!("[2] Set voltage");
        print!("> ");
        io::stdout().flush().unwrap();
        
        // Read a number from standard input
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(_) => {},
            Err(error) => {println!("Error reading operation: {}", error); continue},
        }
        let operation: i32 = match input.trim().parse() {
            Ok(num) => num,
            Err(error) => {println!("Error parsing input to number: {}", error); continue},
        };
        match operation {
            1 => read(),
            2 => write(),
            _ => println!("Invalid operation."),
        }
    }
    
}
