use super::super::common::{
    hpk::{Parsable, Writable},
    primitives::{h_bool, h_string, wh_bool, wh_string},
};
use cookie_factory::{
    bytes::{le_i32 as w_le_i32, le_u32 as w_le_u32},
    multi::all as wh_all,
    sequence::tuple as wh_tuple,
    SerializeFn,
};
use nom::{
    combinator::map,
    multi::length_count,
    number::complete::{le_i32, le_u32},
    sequence::tuple,
    IResult,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
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

impl Writable for Animation {
    fn write<'a>(&'a self) -> Box<dyn SerializeFn<Vec<u8>> + 'a> {
        let writer = wh_tuple((
            wh_string(&self.name),
            wh_string(&self.spritesheet),
            wh_string(&self.material),
            w_le_u32(self.sequences.len() as u32),
            wh_all(self.sequences.iter().map(|f| f.write())),
            w_le_u32(self.directions.len() as u32),
            wh_all(self.directions.iter().map(|f| f.write())),
        ));
        Box::new(writer)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Frame {
    pub image_name: String,
    pub frame_number: i32,
    pub duration: i32,
}

impl Parsable for Frame {
    fn parse(i: &[u8]) -> IResult<&[u8], Self> {
        map(
            tuple((h_string, le_i32, le_i32)),
            |(imagename, frame, duration)| Frame {
                image_name: imagename,
                frame_number: frame,
                duration,
            },
        )(i)
    }
}

impl Writable for Frame {
    fn write<'a>(&'a self) -> Box<dyn SerializeFn<Vec<u8>> + 'a> {
        let writer = wh_tuple((
            wh_string(&self.image_name),
            w_le_i32(self.frame_number),
            w_le_i32(self.duration),
        ));
        Box::new(writer)
    }
}

#[derive(Serialize, Deserialize, Debug)]
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

impl Writable for Sequence {
    fn write<'a>(&'a self) -> Box<dyn SerializeFn<Vec<u8>> + 'a> {
        let writer = wh_tuple((
            w_le_u32(self.frames.len() as u32),
            wh_all(self.frames.iter().map(|f| f.write())),
            wh_string(&self.name),
            wh_bool(self.is_loop),
            wh_bool(self.no_flip),
        ));
        Box::new(writer)
    }
}

#[derive(Serialize, Deserialize, Debug)]
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

impl Writable for Direction {
    fn write<'a>(&'a self) -> Box<dyn SerializeFn<Vec<u8>> + 'a> {
        let writer = wh_tuple((
            wh_string(&self.name),
            wh_string(&self.filename),
            w_le_i32(self.id),
            wh_bool(self.flip),
        ));
        Box::new(writer)
    }
}
