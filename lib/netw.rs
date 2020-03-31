#![allow(non_snake_case, non_upper_case_globals, non_camel_case_types)]

use std::io::prelude::*;
use std::io::{BufReader, BufRead, SeekFrom};
use std::net::TcpStream;
use std::fs::File;

const packetSize: usize = 128;

#[derive(Debug)]
pub enum netCode {
    sendFile,
    sendCode,
    sendPacket,
	gotPacket,
    OK,
    FINISHED,
    DISC
}

#[derive(Debug)]
pub enum netCodeError {
    netCodeError
}

impl netCode {
    pub fn value(&self) -> &[u8; 2] {
        use self::netCode::*;
        match self {
            sendFile => &[255,8],
            sendCode => &[255, 32],
            sendPacket => &[255, 16],
			gotPacket => &[32,16],
            OK => &[255, 255],
            FINISHED => &[255, 128],
            DISC => &[63,63]
        }
    }
}

pub fn codeFromValue(value: &[u8;2]) -> Result<netCode, netCodeError> {
    use netCode::*;
    match value {
        &[255,8] => Ok(sendFile),
        &[255, 32] => Ok(sendCode),
        &[255,16] => Ok(sendPacket),
		&[32,16] => Ok(gotPacket),
        &[255, 255] => Ok(OK),
        &[255, 128] => Ok(FINISHED),
        &[63,63] => Ok(DISC),
        _ => Err(netCodeError::netCodeError)
    }
}

pub fn sendCode(code: &[u8;2],mut stream: &TcpStream) {
    stream.write(code).unwrap();
    //if code != netCode::sendPacket.value() {
        println!("Code sent : {:?}", codeFromValue(&code).unwrap());
    //}
    //dont wait for a response if you sent OK
    if code != netCode::OK.value() {
        let mut resp = [0;2];
        stream.read(&mut resp).unwrap();
        if &resp == netCode::OK.value() {
            return ()
        }
        else {
            println!("Failed to send code (no confirmation) : return code {:?}", &resp);
            return ()
        }
    }
}

fn sendPacket(packet: &[u8],withBlanck: bool, mut stream: &TcpStream) {
    sendCode(netCode::sendPacket.value(), &stream);
    //sendCode waits for confirmation (netCode::OK)
	if withBlanck {
        sendWithBlanck(packet, &stream);
    }
    else {
        stream.write(&packet).unwrap();
    }
	match getCode(&stream) {
		netCode::gotPacket => {},
		_ => {panic!("Packet not received");}
	}
    
}

fn getPacket(buf : &mut [u8], mut stream: &TcpStream) {
    match getCode(&stream) {
        netCode::sendPacket => {
            stream.read(buf).unwrap();
        },
        _ => {}
    }
}

pub fn sendFile(name: &str, path: &str, mut stream: &TcpStream) {
    let file = File::open(path).expect("Could not open the file");
    println!("Sending file {}", name);
    //sending file name length as le_bytes (little endian) then send file name : little endian allows us to send a  x\00 padding with no need to format 
    sendPacket(&name.as_bytes().len().to_le_bytes(), false, &stream);
    sendPacket(&name.as_bytes(), false, &stream);
    //send file size
    let fSize = std::fs::metadata(path).unwrap().len();
    sendPacket(&fSize.to_le_bytes(), false, &stream);
    println!("Sending file size : {}b", fSize);
    sendPacket(b"[[FILE_SIZE_FINISHED]]", true, &stream);
    //send file content
    let mut reader = BufReader::with_capacity(packetSize, &file);
    let max = ((fSize-fSize%packetSize as u64)/packetSize as u64)+1;
    println!("{}", max*packetSize as u64); 
    'data: for x in 0..max {
        reader.seek(SeekFrom::Start(x*packetSize as u64)).unwrap();
		if x==max-1 {
			let mut reader = BufReader::with_capacity((fSize-x*packetSize as u64) as usize, &file);
			sendPacket(reader.fill_buf().unwrap(), true, &stream);
			break 'data;
		}
        sendPacket(reader.fill_buf().unwrap(), true, &stream);
    }
    sendPacket(b"[[FILE_DATA_FINISHED]]", true, &stream);
    //check for answer code (OK)
    if getCode(&stream).value() != netCode::OK.value() {
        println!("There was an error sending the file");
    }
}

pub fn sendWithBlanck(value: &[u8], mut stream: &TcpStream) {
    //if value bigger than a packet size split it in multiple packet
    if value.len()>packetSize {
        println!("{}", value.len());
        let val = value.len();
        let max = ((val-val%packetSize)/packetSize)+1;
        for x in 0..max {
            sendWithBlanck(&value[x*packetSize .. ((x+1)*packetSize)], &stream);
        }
        return
    }
    //else add x\00 padding
    let arr: Vec<u8> = vec![0u8; packetSize-value.len()];
    stream.write(&[value, &arr].concat()).unwrap();
}

pub fn notEmptyPacket(data: &[u8]) -> bool {
    for x in 0..data.len() {
        if data[x] != 0 {
            return true
        }
    }
    false
}

pub fn getFile(mut stream: &TcpStream) -> std::io::Result<()> {
    //get file name size then file name (to get the extension)
    let mut message = [0;8];
    getPacket(&mut message, &stream);
    let fileNameSize = i64::from_le_bytes(message);
    let mut fileName = vec![0; fileNameSize as usize];
    getPacket(&mut fileName, &stream);
    let mut fileName = std::str::from_utf8(&fileName).unwrap();
    println!("FILENAME : {}", fileName);
    if(fileName.len() >= 5) && (&fileName[fileName.len()-4..]) == "json" {
        fileName = "coord.json";
    }
    let mut file = File::create(fileName).expect("Couldn't create file");
    //get file size
    let fileSize;         
    let mut message = [0;8];
    getPacket(&mut message, &stream);
    fileSize = i64::from_le_bytes(message);
    //get file data
    let mut count = 0;
    'outer: loop {
        let mut message = [0;packetSize];
        getPacket(&mut message, &stream);
        if &message[0..22]==b"[[FILE_DATA_FINISHED]]" {
            println!("FILE DATA RECEIVED");
                break 'outer;
        }
        else if &message[0..22]==b"[[FILE_SIZE_FINISHED]]" {
            println!("FILE SIZE : {}b", fileSize);
        }
        else {
            file.write_all(&message).unwrap();
			//println!("{:?}", &message[0..31]);	        
		}
		sendCode(netCode::gotPacket.value(), &stream);
	/*else {
	    if count<500 {
		file.write_all(&message).unwrap();
		count+=1;
		println!("{}", count);
	    }
	    else {
		println!("FILE DATA RECEIVED");
                break 'outer;
	    }
	}*/	
    }
    //truncate the fill to its real size, removing x\00 padding
    file.set_len(fileSize as u64).unwrap();
    sendCode(netCode::OK.value(), &stream);
    Ok(())
}

pub fn getCode(mut stream: &TcpStream) -> netCode {
    let mut code = [0;2];
    stream.read(&mut code).unwrap();
    // if received smthing else than OK answer OK
    if &code != netCode::OK.value() {
        stream.write(netCode::OK.value()).unwrap();
    }
    //if &code != netCode::sendPacket.value() {
        println!("Received code : {:?}", codeFromValue(&code).unwrap());
    //}
    codeFromValue(&code).unwrap()
}
