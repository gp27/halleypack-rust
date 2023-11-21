use crate::halley::versions::{
    common::{
        hpk::{Parsable, Writable},
        primitives::{h_bool, h_string, wh_bool, wh_string},
    },
    v2020::animation::{Direction, Frame},
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
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
pub struct Animation {
    pub name: String,
    pub spritesheet: String,
    pub material: String,
    pub sequences: Vec<Sequence>,
    pub directions: Vec<Direction>,
    pub action_points: Vec<ActionPoint>,
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
                length_count(le_u32, ActionPoint::parse),
            )),
            |(name, spritesheet, material, sequences, directions, action_points)| Animation {
                name,
                spritesheet,
                material,
                sequences,
                directions,
                action_points,
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
            w_le_u32(self.action_points.len() as u32),
            wh_all(self.action_points.iter().map(|f| f.write())),
        ));
        Box::new(writer)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Sequence {
    pub frames: Vec<Frame>,
    pub name: String,
    pub id: i32,
    pub is_loop: bool,
    pub no_flip: bool,
    pub fallback: bool,
}

impl Parsable for Sequence {
    fn parse(i: &[u8]) -> IResult<&[u8], Self> {
        map(
            tuple((
                length_count(le_u32, Frame::parse),
                h_string,
                le_i32,
                h_bool,
                h_bool,
                h_bool,
            )),
            |(frames, name, id, is_loop, no_flip, fallback)| Sequence {
                frames,
                name,
                id: id as i32,
                is_loop,
                no_flip,
                fallback,
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
            w_le_i32(self.id),
            wh_bool(self.is_loop),
            wh_bool(self.no_flip),
            wh_bool(self.fallback),
        ));
        Box::new(writer)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ActionPoint {
    pub name: String,
    pub id: i32,
    pub points: HashMap<(i32, i32, i32), (i32, i32)>,
}

impl Parsable for ActionPoint {
    fn parse(i: &[u8]) -> IResult<&[u8], Self> {
        map(
            tuple((
                h_string,
                le_i32,
                length_count(
                    le_u32,
                    tuple((tuple((le_i32, le_i32, le_i32)), tuple((le_i32, le_i32)))),
                ),
            )),
            |(name, id, points)| {
                let mut map = HashMap::new();
                for ((x, y, z), (px, py)) in points {
                    map.insert((x, y, z), (px, py));
                }
                ActionPoint {
                    name,
                    id,
                    points: map,
                }
            },
        )(i)
    }
}
impl Writable for ActionPoint {
    fn write<'a>(&'a self) -> Box<dyn SerializeFn<Vec<u8>> + 'a> {
        let writer = wh_tuple((
            wh_string(&self.name),
            w_le_i32(self.id),
            w_le_u32(self.points.len() as u32),
            wh_all(self.points.iter().map(|((x, y, z), (px, py))| {
                wh_tuple((wh_tuple((
                    w_le_i32(*x),
                    w_le_i32(*y),
                    w_le_i32(*z),
                    w_le_i32(*px),
                    w_le_i32(*py),
                )),))
            })),
        ));
        Box::new(writer)
    }
}
