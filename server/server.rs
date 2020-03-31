#![allow(non_snake_case, non_upper_case_globals, non_camel_case_types)]

extern crate netw;

use netw::*;
use std::net::{TcpListener, TcpStream};
use std::fs::File;
use std::io::prelude::*;
use std::str::FromStr;
use std::sync::{Mutex, Arc};
use std::thread;
use std::mem::drop;

struct splitQueue {
    files: Vec<(u32, u32)>,
    count: u32
}

impl splitQueue {
    fn getNext(&mut self) -> Result<(u32, u32), ()> {
        if self.count+1 <= self.files.len() as u32 {
            let res = self.files[self.count as usize];
            self.count+=1;
            Ok(res)
        } 
        else {
            Err(())
        }        
    }
}

fn main() {
    let mut coord: Vec<(u32, u32)> = Vec::new();
    createJsons(&mut coord);
    let mut queue = splitQueue{
        files: coord,
        count: 0,
    };
    let listener = startServer().expect("Failed to start server");
    acceptConnections(queue, listener);
}

fn startServer() -> std::io::Result<TcpListener> {
    let listener = TcpListener::bind("192.168.0.20:9999")?;
    println!("LISTENING");
    Ok(listener)
}

fn acceptConnections(files: splitQueue, listener: TcpListener) {
    let splitQueue = Arc::new(Mutex::new(files));
    'listener: for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let queue = Arc::clone(&splitQueue);
                let handle = thread::spawn(move || {    
                    println!("CONNECTED");
                    sendCode(netCode::sendFile.value(), &stream);
                    sendFile(&std::env::args().collect::<Vec<String>>()[2],&std::env::args().collect::<Vec<String>>()[2], &stream);
                    println!("sent!");
                    loop  {
                        let mut queue = queue.lock().unwrap();
                        if let Ok((y, x)) = queue.getNext() {
                            drop(queue);
                            render(y, x, &stream);
                        }
                        else {
                            drop(queue);
                            sendCode(netCode::DISC.value(), &stream);
                            break;
                        }
                    }
                });
            },
            Err(e) => {
                println!("Connection error : {:?}", e);
            }
        }
    }
}

fn render(y: u32, x: u32, stream: &TcpStream) {
    sendCode(netCode::sendFile.value(), &stream);
    sendFile(&format!("{}{}.json", &y.to_string(), &x.to_string()), &format!("{}{}.json", &y.to_string(), &x.to_string()), &stream);
    let code = getCode(&stream);
    getFile(&stream).unwrap();
}

fn getDimensions(numDiv: u32) -> (u32, u32, f32, f32, f32, f32){
    let reste = numDiv%2;
    let mut colX: u32 = 1;
    while (numDiv-reste)/colX%2==0{
        colX*=2;
    }
    let colY: u32 = ((numDiv-reste)/colX) as u32;
    if reste == 0 {
        (colX, colY, 1.0/colX as f32, 1.0/colY as f32, 0., 0.)
    }
    else {
        (colX, colY, 1.0/colX as f32, (numDiv-1) as f32/(numDiv*colY) as f32, 1.0, (numDiv-1) as f32/(numDiv*colY*colX) as f32)
    }
}

fn createJsonFile(xMin: f32, yMin: f32, xMax: f32, yMax: f32, x: u32, y:u32, name: &str) {
    let mut file = File::create(format!("{}{}.json", y, x)).expect("Couldn't create JSON file");
    file.write_all(format!("{{\"xMin\": {}, \"yMin\": {}, \"xMax\": {}, \"yMax\": {},\"x\": {}, \"y\": {}, \"name\": \"{}\"}}", 
    xMin, yMin, xMax, yMax, x, y, name).as_bytes()).expect("Couldnt write JSON data");
}

fn createJsons(coord: &mut Vec<(u32, u32)>) {
    let args: Vec<String> = std::env::args().collect();
    let dim = getDimensions(u32::from_str(&args[1]).expect("Please enter a number for division count"));
    let files: Vec<(u32, u32)> = Vec::new();
    for x in 0..dim.0 {
        for y in 0..dim.1 {
            createJsonFile(dim.2*x as f32, dim.3*y as f32, dim.2*(x+1) as f32, dim.3*(y+1) as f32, x, y, &args[2]);
            coord.push((y as u32,x as u32));
        }
    }
    if u32::from_str(&args[1]).unwrap()%2==1 {
        createJsonFile(0.0, 1.0-dim.5 as f32, 1.0 as f32, 1.0 as f32, dim.0, dim.1, &args[2]);
        coord.push((dim.1 as u32,dim.0 as u32));
    }
}