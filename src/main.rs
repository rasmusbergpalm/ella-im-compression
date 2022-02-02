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

use image::Pixel;
use image::Rgb;
use image::io::Reader as ImageReader;

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};


const VERSION: u32 = 1;
const MAGIC_BYTES: &str = "ELLA";
const BIT_PRECISION: u64 = 48;

fn main() {
    let args: Vec<String> = env::args().collect();
    println!("{:?}", args);
    if args.len() != 4 {
        help()
    }

    let mode = args[1].as_str();
    let input_fname = args[2].as_str();
    let output_fname = args[3].as_str();

    match mode {
        "e" => {
            encode(input_fname, output_fname);
        }
        "d" => {
            decode(input_fname, output_fname)
        }
        _ => { help() }
    }
}

fn help() {
    println!("ella e|d input output");
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

    let mut model = SourceModelBuilder::new().num_bits(8).eof(EndAddOne).build();
    let mut decoder = ArithmeticDecoder::new(BIT_PRECISION);

    let mut br: BitReader<_, MSB> = BitReader::new(in_file);

    let mut imgbuf = image::ImageBuffer::new(w, h);
    let mut prev = image::Rgb([0u8, 0u8, 0u8]);
    for (_, _, pixel) in imgbuf.enumerate_pixels_mut() {
        let deltas = (0..3).map(|_| {
            let d = decoder.decode(&model, &mut br).unwrap() as u8;
            model.update_symbol(d as u32);
            return d;
        }).collect::<Vec<u8>>();

        *pixel = image::Rgb([
            prev.0[0].wrapping_add(deltas[0]),
            prev.0[1].wrapping_add(deltas[1]),
            prev.0[2].wrapping_add(deltas[2])
        ]);
        prev = *pixel;
    }
    assert_eq!(model.eof(), decoder.decode(&model, &mut br).unwrap());

    imgbuf.save(output_fname).unwrap();
}

fn encode(input_fname: &str, output_fname: &str) {
    let img = ImageReader::open(input_fname)
        .unwrap()
        .decode()
        .expect("input io");

    let (w, h) = img.dimensions();
    let c = img.color().channel_count();

    let img = img.into_rgb8();
    let mut prev = Rgb([0u8, 0u8, 0u8]);
    let deltas: Vec<Rgb<u8>> = img.pixels().map(|p| {
        let delta = p.map2(&prev, |it, ot| { it.wrapping_sub(ot) });
        prev = *p;
        return delta;
    }).collect();

    let mut file = BufWriter::new(File::create(output_fname).expect("err making output file"));
    file.write(MAGIC_BYTES.as_bytes()).expect("magic bytes");
    file.write_u32::<BigEndian>(VERSION).expect("version");
    file.write_u32::<BigEndian>(h).expect("could not write h");
    file.write_u32::<BigEndian>(w).expect("could not write w");
    file.write_u8(c).expect("could not write c");

    let mut model = SourceModelBuilder::new().num_bits(8).eof(EndAddOne).build();
    let mut encoder = ArithmeticEncoder::new(BIT_PRECISION);
    let mut bw = BitWriter::new(file);
    for delta in deltas {
        for v in delta.0 {
            encoder.encode(v as u32, &model, &mut bw).expect("encode err");
            model.update_symbol(v as u32);
        }
    }
    encoder.encode(model.eof(), &model, &mut bw).unwrap();
    encoder.finish_encode(&mut bw).unwrap();
    bw.pad_to_byte().unwrap();
}