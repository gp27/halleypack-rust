use nom::{
    combinator::map,
    multi::length_count,
    number::complete::{le_i32, le_u32},
    sequence::tuple,
    IResult,
};
use serde::{Deserialize, Serialize};

use crate::halley::versions::common::hpk::Parsable;

use super::super::common::primitives::{h_bool, h_string};

#[derive(Serialize, Deserialize)]
pub struct Animation {
    pub name: String,
    pub spritesheet: String,
    pub material: String,
    pub sequences: Vec<Sequence>,
    pub directions: Vec<Direction>,
}

impl Parsable for Animation {
    fn parse(i: &[u8]) -> IResult<&[u8], Self> {
        animation_parser(i).map(|(i, a)| (i, a))
    }
}

pub fn animation_parser(i: &[u8]) -> IResult<&[u8], Animation> {
    map(
        tuple((
            h_string,
            h_string,
            h_string,
            length_count(le_u32, sequence_parser),
            length_count(le_u32, direction_parser),
        )),
        |(name, spritesheet, material, sequences, directions)| Animation {
            name,
            spritesheet,
            material,
            sequences,
            directions,
        },
    )(i)
}

#[derive(Serialize, Deserialize)]
pub struct Frame {
    pub imagename: String,
    pub frame: i32,
    pub duration: i32,
}

pub fn frame_parser(i: &[u8]) -> IResult<&[u8], Frame> {
    map(
        tuple((h_string, le_i32, le_i32)),
        |(imagename, frame, duration)| Frame {
            imagename,
            frame,
            duration,
        },
    )(i)
}

#[derive(Serialize, Deserialize)]
pub struct Sequence {
    pub frames: Vec<Frame>,
    pub name: String,
    pub is_loop: bool,
    pub no_flip: bool,
}

pub fn sequence_parser(i: &[u8]) -> IResult<&[u8], Sequence> {
    map(
        tuple((length_count(le_u32, frame_parser), h_string, h_bool, h_bool)),
        |(frames, name, is_loop, no_flip)| Sequence {
            frames,
            name,
            is_loop,
            no_flip,
        },
    )(i)
}

#[derive(Serialize, Deserialize)]
pub struct Direction {
    pub name: String,
    pub filename: String,
    pub id: i32,
    pub flip: bool,
}

pub fn direction_parser(i: &[u8]) -> IResult<&[u8], Direction> {
    map(
        tuple((h_string, h_string, le_i32, h_bool)),
        |(name, filename, id, flip)| Direction {
            name,
            filename,
            id,
            flip,
        },
    )(i)
}
