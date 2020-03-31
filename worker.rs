#![allow(non_snake_case, non_upper_case_globals, non_camel_case_types)]

extern crate netw;

use netw::*;
use std::net::TcpStream;
use std::fs::File;
use std::io::prelude::*;

fn main() {
    let server = "192.168.0.20:9999";
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
                .args(&["/C", "script.bat", &args[1]])
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
        let mut f = File::open("num.txt").unwrap_or_else(|e| {
                match e.kind() {
                        std::io::ErrorKind::NotFound => File::create("num.txt").unwrap(),
                        _ => {panic!("Error file (num.txt)")}
                }
        });
        let mut num = String::new();
        f.read_to_string(&mut num).unwrap();
        sendFile(&format!("{}.png", &num), &format!("{}.png", &num), &stream);
        
    }
    println!("Render finished");
}