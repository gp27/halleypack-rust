use super::super::common::{
    hpk::Parsable,
    primitives::{h_bool, h_string},
};
use nom::{
    combinator::map,
    multi::length_count,
    number::complete::{le_i32, le_u32},
    sequence::tuple,
    IResult,
};
use serde::{Deserialize, Serialize};

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
        map(
            tuple((
                h_string,
                h_string,
                h_string,
                length_count(le_u32, Sequence::parse),
                length_count(le_u32, Direction::parse),
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
}

#[derive(Serialize, Deserialize)]
pub struct Frame {
    pub imagename: String,
    pub frame: i32,
    pub duration: i32,
}

impl Parsable for Frame {
    fn parse(i: &[u8]) -> IResult<&[u8], Self> {
        map(
            tuple((h_string, le_i32, le_i32)),
            |(imagename, frame, duration)| Frame {
                imagename,
                frame,
                duration,
            },
        )(i)
    }
}

#[derive(Serialize, Deserialize)]
pub struct Sequence {
    pub frames: Vec<Frame>,
    pub name: String,
    pub is_loop: bool,
    pub no_flip: bool,
}

impl Parsable for Sequence {
    fn parse(i: &[u8]) -> IResult<&[u8], Self> {
        map(
            tuple((length_count(le_u32, Frame::parse), h_string, h_bool, h_bool)),
            |(frames, name, is_loop, no_flip)| Sequence {
                frames,
                name,
                is_loop,
                no_flip,
            },
        )(i)
    }
}

#[derive(Serialize, Deserialize)]
pub struct Direction {
    pub name: String,
    pub filename: String,
    pub id: i32,
    pub flip: bool,
}

impl Parsable for Direction {
    fn parse(i: &[u8]) -> IResult<&[u8], Self> {
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
}
