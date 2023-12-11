use anyhow::anyhow;
use image::{DynamicImage, ImageBuffer};
use nom::{
    bytes::complete::{tag, take},
    combinator::{map, map_res, rest},
    multi::count,
    number::complete::{le_f32, le_u16, le_u32, u8},
    sequence::tuple,
    IResult,
};
use num_derive::{FromPrimitive, ToPrimitive};
static IDENTIFIER: &str = "HLIFv01\0";

#[repr(u8)]
#[derive(Debug, FromPrimitive, ToPrimitive, PartialEq, Copy, Clone)]
pub enum Format {
    RGBA = 0,
    SingleChannel = 1,
    Indexed = 2,
}

pub enum Flags {
    Premultiplied = 1,
}

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
    entries: Vec<f32>, //[f32; 256],
}

impl Palette {
    fn entries_buffer(&mut self) -> &mut [u8] {
        unsafe {
            std::slice::from_raw_parts_mut(
                self.entries.as_mut_ptr() as *mut u8,
                self.entries.len() * std::mem::size_of::<f32>(),
            )
        }
    }
}

#[derive(Debug)]
pub struct HLIFileInfo {
    pub size: (i32, i32),
    pub format: Format,
}

#[derive(Debug)]
pub struct HLIFFileHeader {
    pub width: u32,
    pub height: u32,
    pub compressed_size: u32,
    pub uncompressed_size: u32,
    pub format: Format,
    pub flags: u8,
    pub num_palettes: u8,
    pub reserved: u8,
    pub bpp: u32,
}

pub fn hlif_decoder(i: &[u8]) -> Result<DynamicImage, anyhow::Error> {
    let (_, (header, mut palettes, line_data, pixel_data)) =
        hlif_header_parser(i).map_err(|e| anyhow!(e.to_string()))?;
    let mut pixel_data = pixel_data.to_owned();

    let img_format = get_img_format(&header);

    decode_lines(
        (header.width, header.height),
        &line_data,
        &mut pixel_data,
        header.bpp,
    )?;

    let image = if palettes.len() > 0 {
        delta_decode_palettes(&mut palettes);
        let mut buf = vec![0.0 as f32; header.width as usize * header.height as usize];
        apply_palettes(&palettes, &pixel_data, &mut buf);
        let img_buf = ImageBuffer::from_raw(header.width, header.height, buf)
            .ok_or(anyhow!("Could not make ImageBuffer from pixel_data"))?;
        DynamicImage::ImageRgba32F(img_buf)
    } else {
        let pixel_data: Vec<f32> = unsafe {
            std::slice::from_raw_parts(
                pixel_data.as_ptr() as *const f32,
                pixel_data.len() * std::mem::size_of::<f32>(),
            )
            .to_vec()
        };
        let img_buf = ImageBuffer::from_raw(header.width, header.height, pixel_data).unwrap();
        DynamicImage::ImageRgba32F(img_buf)
    };
    Ok::<DynamicImage, anyhow::Error>(image)
}

fn hlif_header_parser(
    i: &[u8],
) -> IResult<&[u8], (HLIFFileHeader, Vec<Palette>, Vec<u8>, Vec<u8>)> {
    let (i, res) = tuple((
        tag(IDENTIFIER),
        le_u16,
        le_u16,
        le_u32,
        le_u32,
        map_res(u8, |f| {
            let format: Format =
                num::FromPrimitive::from_u8(f).ok_or(anyhow!("Could not parse HLIF format"))?;
            Ok::<Format, anyhow::Error>(format)
        }),
        u8,
        u8,
        u8,
    ))(i)?;

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

    let bpp = if num_palettes > 0 {
        1
    } else {
        get_bpp(&format)
    };

    //let palettes_size = num_palettes as usize * size_of::<Palette>();

    // if uncompressed_size
    //     != width as u32 * height as u32 * bpp as u32 + height as u32 + palettes_size as u32
    // {
    //     // Invalid HLIF file encoding.
    //     println!(
    //         "Invalid HLIF file encoding: {} != {} * {} * {} + {} + {}",
    //         uncompressed_size, width, height, bpp, height, palettes_size
    //     );
    //     return Err(nom::Err::Failure(nom::error::Error::new(
    //         i,
    //         nom::error::ErrorKind::Fail,
    //     )));
    // }

    // let data = lz4::block::decompress(i, Some(uncompressed_size as i32)).map_err(|_err| {
    //     nom::Err::Failure(nom::error::Error::new(i, nom::error::ErrorKind::Fail))
    // })?;

    let (i, data) = lz4_decompress(Some(uncompressed_size as i32))(i)?;

    let res = map(
        tuple((
            count(parse_palette, num_palettes as usize),
            take(height as usize),
            rest,
        )),
        |(palettes, line_data, pixel_data)| {
            (
                HLIFFileHeader {
                    width: width as u32,
                    height: width as u32,
                    compressed_size,
                    uncompressed_size,
                    format,
                    flags,
                    num_palettes,
                    reserved,
                    bpp,
                },
                palettes,
                line_data.to_vec(),
                pixel_data.to_vec(),
            )
        },
    )(&data);
    let (_, t) = res.unwrap();
    Ok((i, t))
}

fn parse_palette(i: &[u8]) -> IResult<&[u8], Palette> {
    map(
        tuple((le_u32, count(le_f32, 256))),
        |(end_pixel, entries)| Palette { end_pixel, entries },
    )(i)
}

fn decode_lines(
    size: (u32, u32),
    line_data: &[u8],
    pixel_data: &mut [u8],
    bpp: u32,
) -> Result<(), anyhow::Error> {
    let stride = bpp as usize * size.0 as usize;
    let blank_line: &[u8] = &vec![0 as u8; stride];

    let mut prev_line = blank_line;

    for (y, cur_line) in pixel_data.chunks_mut(stride).enumerate() {
        let line_encoding: LineEncoding =
            num::FromPrimitive::from_u8(line_data[y]).ok_or(anyhow!("Invalid line encoding"))?;
        decode_line(line_encoding, cur_line, prev_line, bpp);
        prev_line = cur_line;
    }
    Ok(())
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
                    prev = prev.wrapping_add(cur_line[x]);
                    cur_line[x] = prev;
                } else {
                    cur_line[x] = cur_line[x].wrapping_add(cur_line[x - BPP]);
                }
            }
        }
        LineEncoding::Up => {
            for x in 0..n {
                cur_line[x] = cur_line[x].wrapping_add(prev_line[x]);
            }
        }
        LineEncoding::Average => {
            for x in BPP..n {
                let a = cur_line[x - BPP];
                let b = prev_line[x];
                let avg = ((a as i16 + b as i16) / 2) as u8;
                cur_line[x] = cur_line[x].wrapping_add(avg);
            }
        }
        LineEncoding::Paeth => {
            for x in BPP..n {
                let a = cur_line[x - BPP];
                let b = prev_line[x];
                let c = prev_line[x - BPP];
                let p = a as i16 + b as i16 - c as i16;
                let pc = get_closest(a, b, c, p);
                cur_line[x] = cur_line[x].wrapping_add(pc);
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

fn get_bpp(format: &Format) -> u32 {
    if *format == Format::RGBA {
        4
    } else {
        1
    }
}

fn get_img_format(header: &HLIFFileHeader) -> ImageFormat {
    if header.format == Format::RGBA {
        if (header.flags & (Flags::Premultiplied as u8)) == 1 {
            ImageFormat::RGBAPremultiplied
        } else {
            ImageFormat::RGBA
        }
    } else {
        if header.format == Format::SingleChannel {
            ImageFormat::SingleChannel
        } else {
            ImageFormat::Indexed
        }
    }
}

fn delta_decode_palettes(palettes: &mut [Palette]) {
    for i in 1..palettes.len() {
        for j in 0..1024 {
            palettes[i].entries_buffer()[j] =
                palettes[i].entries_buffer()[j].wrapping_add(palettes[i - 1].entries_buffer()[j]);
        }
    }
}

fn apply_palettes(palettes: &[Palette], pixel_data: &[u8], dst: &mut [f32]) {
    let mut start_pos = 0;
    for palette in palettes {
        let end_pos = palette.end_pixel as usize;
        for i in start_pos..end_pos {
            let pixel = pixel_data[i] as usize;
            let color = palette.entries[pixel];
            dst[i] = color;
        }
        start_pos = end_pos;
    }
}

pub fn lz4_decompress(
    uncompressed_size: Option<i32>,
) -> impl FnMut(&[u8]) -> IResult<&[u8], Vec<u8>> {
    move |i| {
        let data = lz4::block::decompress(i, uncompressed_size).expect("Could not decompress lz4");
        Ok((i, data))
    }
}
