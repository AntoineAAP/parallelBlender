#![allow(non_snake_case, non_upper_case_globals, non_camel_case_types)]

extern crate netw;

use netw::*;
use std::net::TcpStream;
use std::fs::File;
use std::io::prelude::*;

fn main() {
    let args: Vec<String> = std::env::args().collect();   
    let server = &args[1];
    let stream = TcpStream::connect(server).unwrap();
    let code = getCode(&stream);
    match code {
        netCode::sendFile => {
                getFile(&stream).unwrap();
        },
        _ => ()
    }
    let args: Vec<String> = std::env::args().collect();
    
    loop {
        let code = getCode(&stream);
        match code {
                netCode::sendFile => {
                    getFile(&stream).unwrap();
                },
                netCode::DISC => {break;}
                _ => ()
        }
        
        if cfg!(windows) {
                std::process::Command::new("cmd")
                .args(&["/C", "script.bat", &args[2]])
                .output()
                .expect("Failed to execute process");
        }
        else {
                std::process::Command::new("bash")
                .args(&["script.sh", &args[1]])
                .output()
                .expect("Failed to execute process");
        }
        sendCode(netCode::FINISHED.value(), &stream);
        let mut f = File::open("num.txt").unwrap();
        let mut num = String::new();
        f.read_to_string(&mut num).unwrap();
        sendFile(&format!("{}.png", &num), &format!("{}.png", &num), &stream);
        
    }
    println!("Render finished");
}