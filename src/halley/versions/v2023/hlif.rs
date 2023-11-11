use nom::{
    bytes::complete::{tag, take},
    combinator::{map, rest},
    error::Error,
    number::complete::{le_u16, le_u32, u8},
    sequence::tuple,
    IResult,
};
use num_derive::{FromPrimitive, ToPrimitive};
use std::mem::size_of;

static IDENTIFIER: &str = "HLIFv01";

#[repr(u8)]
#[derive(Debug, FromPrimitive, ToPrimitive, PartialEq)]
pub enum ImageFormat {
    Undefined = 0,
    Indexed = 1,
    RGB = 2,
    RGBA = 3,
    RGBAPremultiplied = 4,
    SingleChannel = 5,
}

#[repr(u8)]
#[derive(Debug, FromPrimitive, ToPrimitive, PartialEq)]
enum LineEncoding {
    None = 0,
    Sub = 1,
    Up = 2,
    Average = 3,
    Paeth = 4,
}

pub struct Palette {
    end_pixel: u32,
    entries: [i32; 256],
}

#[derive(Debug)]
pub struct HLIFileInfo {
    pub size: (i32, i32),
    pub format: ImageFormat,
}

#[derive(Debug)]
pub struct HLIFFileHeader {
    pub width: usize,
    pub height: usize,
    pub compressed_size: u32,
    pub uncompressed_size: u32,
    pub format: ImageFormat,
    pub flags: u8,
    pub num_palettes: u8,
    pub reserved: u8,
    pub bpp: u32,
}

pub fn hlif_parser(i: &[u8]) -> IResult<&[u8], ()> {
    map(
        hlif_header_parser,
        |(header, _palette_data, line_data, pixel_data)| {
            let mut pixel_data = pixel_data.to_owned();

            decode_lines(
                (header.width, header.height),
                &line_data,
                &mut pixel_data,
                header.bpp,
            );
        },
    )(i)
}

fn hlif_header_parser(i: &[u8]) -> IResult<&[u8], (HLIFFileHeader, Vec<u8>, Vec<u8>, Vec<u8>)> {
    tuple((
        tag(IDENTIFIER),
        le_u16,
        le_u16,
        le_u32,
        le_u32,
        u8,
        u8,
        u8,
        u8,
    ))(i)
    .map(|(i, res)| {
        let (
            _id,
            width,
            height,
            compressed_size,
            uncompressed_size,
            format,
            flags,
            num_palettes,
            reserved,
        ) = res;

        let format: ImageFormat = num::FromPrimitive::from_u8(format).unwrap();
        let bpp = if num_palettes > 0 {
            1
        } else {
            get_bpp(&format)
        };

        let palettes_size = num_palettes as usize * size_of::<Palette>();

        // let size =
        //     width as usize * height as usize * bpp as usize + height as usize + palettes_size;

        let data: &[u8] = &lz4::block::decompress(i, Some(uncompressed_size as i32))
            .expect("Could not decompress LZ4 data!");

        let (_, (palette_data, line_data, pixel_data)) = tuple((
            take::<usize, &[u8], Error<_>>(palettes_size),
            take(height),
            rest,
        ))(data)
        .unwrap();

        (
            i,
            (
                HLIFFileHeader {
                    width: width as usize,
                    height: width as usize,
                    compressed_size,
                    uncompressed_size,
                    format,
                    flags,
                    num_palettes,
                    reserved,
                    bpp,
                },
                palette_data.to_vec(),
                line_data.to_vec(),
                pixel_data.to_vec(),
            ),
        )
    })
}

fn decode_lines(size: (usize, usize), line_data: &Vec<u8>, pixel_data: &mut Vec<u8>, bpp: u32) {
    let stride = bpp as usize * size.0 as usize;
    let blank_line: &[u8] = &vec![0 as u8; stride];

    let mut prev_line = blank_line;

    pixel_data
        .chunks_mut(stride)
        .enumerate()
        .for_each(|(y, cur_line)| {
            let line_encoding: LineEncoding = num::FromPrimitive::from_u8(line_data[y]).unwrap();
            decode_line(line_encoding, cur_line, prev_line, bpp);
            prev_line = cur_line;
        });
}

fn decode_line(line_encoding: LineEncoding, cur_line: &mut [u8], prev_line: &[u8], bpp: u32) {
    if bpp == 1 {
        do_decode_line::<1>(line_encoding, cur_line, prev_line);
    } else if bpp == 4 {
        do_decode_line::<4>(line_encoding, cur_line, prev_line);
    }
}

fn do_decode_line<const BPP: usize>(
    line_encoding: LineEncoding,
    cur_line: &mut [u8],
    prev_line: &[u8],
) {
    let n = cur_line.len();
    let mut prev = cur_line[0];

    match line_encoding {
        LineEncoding::None => {}
        LineEncoding::Sub => {
            for x in BPP..n {
                if BPP == 1 {
                    prev += cur_line[x];
                    cur_line[x] = prev;
                } else {
                    cur_line[x] += cur_line[x - BPP];
                }
            }
        }
        LineEncoding::Up => {
            for x in 0..n {
                cur_line[x] += prev_line[x];
            }
        }
        LineEncoding::Average => {
            for x in BPP..n {
                let a = cur_line[x - BPP];
                let b = prev_line[x];
                let avg = ((a as i16 + b as i16) / 2) as u8;
                cur_line[x] += avg;
            }
        }
        LineEncoding::Paeth => {
            for x in BPP..n {
                let a = cur_line[x - BPP];
                let b = prev_line[x];
                let c = prev_line[x - BPP];
                let p = a as i16 + b as i16 - c as i16;
                let pc = get_closest(a, b, c, p);
                cur_line[x] += pc;
            }
        }
    }
}

fn get_closest(a: u8, b: u8, c: u8, p: i16) -> u8 {
    let pa = (p - a as i16).abs();
    let pb = (p - b as i16).abs();
    let pc = (p - c as i16).abs();
    if pa <= pb && pa <= pc {
        a
    } else if pb <= pc {
        b
    } else {
        c
    }
}

fn get_bpp(format: &ImageFormat) -> u32 {
    if *format == ImageFormat::RGBA {
        4
    } else {
        1
    }
}
