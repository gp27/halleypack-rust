use base64::{engine::general_purpose, Engine as _};
use derivative::Derivative;
use flate2::{read::ZlibDecoder, write::ZlibEncoder};
use libaes::Cipher;
use nom::{
    bytes::complete::{tag, take},
    combinator::map,
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
};

static IDENTIFIER: &str = "HLLYSAVE";

pub struct SDLSaveHeaderV0 {
    pub version: u32,
    pub reserved: u32,
    pub iv: [u8; 16],
    pub filename_hash: u64,
}

pub struct SDLSaveHeaderV1 {
    pub data_hash: u64,
}

pub struct SDLSaveHeader {
    pub v0: SDLSaveHeaderV0,
    pub v1: Option<SDLSaveHeaderV1>,
}

// fn parse_hsave_header<I, E>(
//     i: &[u8],
// ) -> IResult<&[u8], SDLSaveHeaderV0, nom::error::VerboseError<I>>
// where
//     I: Slice<RangeFrom<usize>> + InputIter<Item = u8> + InputLength,
// {
//     let (i, header) = map(
//         tuple((
//             tag(IDENTIFIER),
//             le_u32,
//             le_u32,
//             map(take(16usize), |iv: &[u8]| iv.try_into().unwrap()),
//             le_u64,
//         )),
//         |(_, version, reserved, iv, filename_hash)| SDLSaveHeaderV0 {
//             version,
//             reserved,
//             iv,
//             filename_hash,
//         },
//     )(i)?;

//     if header.version < 0 {
//         Err::Failure("Invalid version")
//     } else {
//         Ok((i, header))
//     }
// }
