extern crate byteorder;

use std::io::prelude::*;
use std::fs;
use std::fs::File;
use std::io::SeekFrom;
use std::process::Command;
use byteorder::{ByteOrder, BigEndian, ReadBytesExt};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <filename>", args[0]);
        std::process::exit(1);
    }
    let filename = &args[1];
    let mut mmfile = fs::OpenOptions::new().read(true).write(true).open(filename).expect("Unable to open the specified file!");
    mmfile.seek(SeekFrom::Start(0)).expect("Unable to seek to start!");
    mmfile.write_all(b"\x00\x00\x00\x01skip").expect("Unable to write skip atom header!");
    mmfile.seek(SeekFrom::Start(0x10)).expect("Unable to seek past header!");
    
    let (header, num_offsets) = if filename.to_lowercase().ends_with(".mmq") {
        let mut header = vec![0; 20];
        mmfile.read_exact(&mut header).expect("Unable to read header!");
        let num_offsets = BigEndian::read_u16(&header[18..20]);
        (header, num_offsets)
    } else {
        let mut header = vec![0; 12];
        mmfile.read_exact(&mut header).expect("Unable to read header!");
        let num_offsets = BigEndian::read_u16(&header[10..12]);
        (header, num_offsets)
    };

    if &header[0..2] != b"MM" {
        eprintln!("Invalid file format!");
        std::process::exit(1);
    }

    let version = BigEndian::read_u16(&header[2..4]);
    println!("Version: {}, Number of file offsets: {}", version, num_offsets);

    let mut offsets = Vec::new();
    for _ in 0..num_offsets {
        let offset = mmfile.read_u32::<BigEndian>().expect("Unable to read file offset!");
        offsets.push(offset);
    }
    let _filelength = mmfile.read_u32::<BigEndian>().expect("Unable to read file length!");
    if filename.to_lowercase().ends_with(".mmq") {
        mmfile.seek(SeekFrom::Current(0x4)).expect("Unable to seek past junk offset!");
    }
    let mut filenames = Vec::new();
    for _ in 0..num_offsets {
        let mut filename = [0; 0x20];
        mmfile.read_exact(&mut filename).expect("Unable to read filename!");
        let filename_str = String::from_utf8_lossy(&filename);
        let filename_str = match filename_str.find('\0') {
            Some(pos) => &filename_str[..pos],
            None => &filename_str,
        };
        filenames.push(filename_str.to_string());
    }

    // for (i, &offset) in offsets.iter().enumerate() {
    //     let next_offset = if i + 1 < offsets.len() {
    //         offsets[i + 1]
    //     } else {
    //         mmfile.metadata().expect("Unable to get file metadata!").len() as u32
    //     };

    //     let length = next_offset - offset;
    //     let mut buffer = vec![0; length as usize];
    //     mmfile.seek(SeekFrom::Start(offset as u64)).expect("Unable to seek to file offset!");
    //     mmfile.read_exact(&mut buffer).expect("Unable to read file data!");

    //     let output_filename = format!("extracted_file_{:03}.bin", i);
    //     let mut output_file = File::create(&output_filename).expect("Unable to create output file!");
    //     output_file.write_all(&buffer).expect("Unable to write to output file!");

    //     println!("Extracted file: {} (offset: {}, length: {})", output_filename, offset, length);
    // }
    for (i, &offset) in offsets.iter().enumerate() {
        convert(&mut mmfile, offset as u64, filename, &filenames[i]);
    }
}

fn length_to_header(length: u64) -> [u8; 8] {
    let mut lenbytes: [u8; 8] = [0; 8];
    BigEndian::write_u64(&mut lenbytes, length);
    lenbytes
}

fn convert(mmfile: &mut File, offset: u64, filename: &str, output_filename: &str) {
    println!("Converting {:?} at @{}", output_filename, offset);
    mmfile.seek(SeekFrom::Start(8)).expect("Unable to seek to write new header!");
    mmfile.write_all(&length_to_header(offset)).expect("Unable to write size to new header!");
    mmfile.sync_all().expect("Unable to sync!");
    Command::new("ffmpeg")
        .arg("-i")
        .arg(filename)
        .arg("-c")
        .arg("copy")
        .arg(format!("{}.mov", output_filename))
        .output().expect("Unable to call ffmpeg!");
}