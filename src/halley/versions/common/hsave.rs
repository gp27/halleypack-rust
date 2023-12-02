use base64::{engine::general_purpose, Engine as _};
use derivative::Derivative;
use flate2::{read::ZlibDecoder, write::ZlibEncoder};
use indexmap::IndexSet;
use libaes::Cipher;
use nom::{
    bytes::complete::{tag, take},
    combinator::{map, verify},
    error::ParseError,
    multi::length_count,
    number::complete::{le_i32, le_u32, le_u64},
    sequence::tuple,
    Err, IResult, InputIter, InputLength, Parser, Slice,
};
use num::iter::RangeFrom;
use std::{
    cmp::min,
    io::{Read, Seek, Write},
    path::{Path, PathBuf},
};

use super::hpk_parse::get_decrypted_data;

static IDENTIFIER: &str = "HLLYSAVE";

#[derive(Debug)]
struct SDLSaveHeaderV0 {
    pub version: u32,
    pub reserved: u32,
    pub iv: [u8; 16],
    pub filename_hash: u64,
}

struct SDLSaveHeaderV1 {
    pub data_hash: u64,
}

struct SDLSaveHeader {
    pub v0: SDLSaveHeaderV0,
    pub v1: Option<SDLSaveHeaderV1>,
}

enum SaveDataType {}

pub struct SDLSaveData {
    save_type: SaveDataType,
    dir: PathBuf,
    key: Option<String>,
    corrupted_files: IndexSet<String>,
}

impl SDLSaveData {}

pub fn load_save_data(path: &Path, key: Option<&str>) -> Vec<u8> {
    let i = std::fs::read(path).unwrap();
    parse_save(&i, key)
}

pub fn parse_save<'a>(i: &'a [u8], key: Option<&str>) -> Vec<u8> {
    let (encrypted, header) = parse_hsave_header(i).unwrap();
    println!("save header -> {:?}", header);
    get_decrypted_data(encrypted, key, Some(&header.iv))
}

fn parse_hsave_header(i: &[u8]) -> IResult<&[u8], SDLSaveHeaderV0> {
    map(
        tuple((
            tag(IDENTIFIER),
            verify(le_u32, |version| *version == 1),
            verify(le_u32, |reserved| *reserved == 0),
            map(take(16usize), |iv: &[u8]| iv.try_into().unwrap()),
            le_u64,
        )),
        |(_, version, reserved, iv, filename_hash)| SDLSaveHeaderV0 {
            version,
            reserved,
            iv,
            filename_hash,
        },
    )(i)
}
