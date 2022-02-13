mod models;

extern crate core;

use std::{env, str};
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};
use std::process::exit;
use arcode::bitbit::{BitReader, BitWriter, MSB};
use arcode::decode::decoder::ArithmeticDecoder;
use arcode::encode::encoder::ArithmeticEncoder;
use arcode::util::source_model_builder::EOFKind::EndAddOne;

use arcode::util::source_model_builder::SourceModelBuilder;

use image::GenericImageView;

use image::io::Reader as ImageReader;

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use crate::models::get_model;

const VERSION: u32 = 1;
const MAGIC_BYTES: &str = "ELLA";
const BIT_PRECISION: u64 = 48;


fn main() {
    let args: Vec<String> = env::args().collect();

    let mode = args[1].as_str();
    let input_fname = args[2].as_str();
    let output_fname = args[3].as_str();

    match mode {
        "e" => {
            let model_idx: u16 = args[4].as_str().parse().unwrap();
            encode(input_fname, output_fname, model_idx);
        }
        "d" => {
            decode(input_fname, output_fname);
        }
        _ => { help() }
    }
}

fn help() {
    println!("ella e <in> <out> <model>");
    println!("ella d <in> <out>");
    exit(1);
}

fn decode(input_fname: &str, output_fname: &str) {
    let mut in_file = BufReader::new(File::open(input_fname).expect("input io"));

    let mut buf = [0u8; 4];
    in_file.read_exact(&mut buf).expect("magic word bytes");
    assert_eq!(str::from_utf8(&buf).expect("magic word parse"), MAGIC_BYTES);

    let version = in_file.read_u32::<BigEndian>().unwrap();
    assert!(version <= VERSION);

    let h = in_file.read_u32::<BigEndian>().unwrap();
    let w = in_file.read_u32::<BigEndian>().unwrap();
    let c = in_file.read_u8().unwrap();
    let m_idx = in_file.read_u16::<BigEndian>().unwrap();
    let model = get_model(m_idx);

    let mut sm = SourceModelBuilder::new().num_bits(8).eof(EndAddOne).build();
    let mut decoder = ArithmeticDecoder::new(BIT_PRECISION);

    let mut br: BitReader<_, MSB> = BitReader::new(in_file);

    let mut bytes: Vec<u8> = Vec::new();
    while !decoder.finished() {
        let d = decoder.decode(&sm, &mut br).unwrap();
        sm.update_symbol(d);
        bytes.push(d as u8);
    }
    assert_eq!(sm.eof() as u8, bytes.pop().unwrap());
    model.decode(w, h, bytes).save(output_fname).expect("save err");
}

fn encode(input_fname: &str, output_fname: &str, model_idx: u16) {
    let img = ImageReader::open(input_fname)
        .unwrap()
        .decode()
        .expect("input io");

    let model = get_model(model_idx);

    let (w, h) = img.dimensions();
    let c = img.color().channel_count();

    let img = img.into_rgb8();
    let bytes = model.encode(&img);

    let mut file = BufWriter::new(File::create(output_fname).expect("err making output file"));
    file.write(MAGIC_BYTES.as_bytes()).expect("magic bytes");
    file.write_u32::<BigEndian>(VERSION).expect("version");
    file.write_u32::<BigEndian>(h).expect("could not write h");
    file.write_u32::<BigEndian>(w).expect("could not write w");
    file.write_u8(c).expect("could not write c");
    file.write_u16::<BigEndian>(model_idx).expect("model");

    let mut model = SourceModelBuilder::new().num_bits(8).eof(EndAddOne).build();
    let mut encoder = ArithmeticEncoder::new(BIT_PRECISION);
    let mut bw = BitWriter::new(file);
    for byte in bytes {
        encoder.encode(byte as u32, &model, &mut bw).expect("encode err");
        model.update_symbol(byte as u32);
    }
    encoder.encode(model.eof(), &model, &mut bw).unwrap();
    encoder.finish_encode(&mut bw).unwrap();
    bw.pad_to_byte().unwrap();
}