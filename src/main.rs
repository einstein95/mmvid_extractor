extern crate byteorder;

use std::io::prelude::*;
use std::fs;
use std::fs::File;
use std::io::SeekFrom;
use std::process::Command;
use byteorder::{ByteOrder, BigEndian, ReadBytesExt, WriteBytesExt};

const START: u64 = 0x08EDB7; // First moov, based on manual inspection.

fn main() {
    let mut SCIGUY = fs::OpenOptions::new().read(true).write(true).open("SCIGUY.MOV").expect("Unable to open SCIGUY.MOV!");
    SCIGUY.seek(SeekFrom::Start(0)).expect("Unable to seek to start!");
    SCIGUY.write_all(b"\x00\x00\x00\x01skip").expect("Unable to write skip atom header!");
    let mut current: u64 = START;
    loop {
        SCIGUY.seek(SeekFrom::Start(current)).expect("Unable to seek to current!");
        let atom_size = SCIGUY.read_u32::<BigEndian>().expect("Unable to read atom size!");
        let mut atom_type: [u8; 4] = [0; 4];
        SCIGUY.read_exact(&mut atom_type).expect("Unable to read atom type!");
        if &atom_type == b"moov" {
            convert(&mut SCIGUY, current);
        }
        current += atom_size as u64;
    }
    
}

fn length_to_header(length: u64) -> [u8; 8] {
    let mut lenbytes: [u8; 8] = [0; 8];
    BigEndian::write_u64(&mut lenbytes, length);
    lenbytes
}

fn convert(SCIGUY: &mut File, offset: u64) {
    SCIGUY.seek(SeekFrom::Start(8)).expect("Unable to seek to write new header!");
    SCIGUY.write_all(&length_to_header(offset)).expect("Unable to write size to new header!");
    SCIGUY.sync_all().expect("Unable to sync!");
    Command::new("ffmpeg")
        .arg("-i")
        .arg("SCIGUY.MOV")
        .arg("-vcodec")
        .arg("copy")
        .arg("-acodec")
        .arg("copy")
        .arg(format!("{}.mov", offset))
        .output().expect("Unable to call ffmpeg!");
}