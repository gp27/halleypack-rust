use std::collections::HashMap;

use nom::{
    combinator::map,
    multi::length_count,
    number::complete::{le_i32, le_u32},
    sequence::tuple,
    IResult,
};
use serde::{Deserialize, Serialize};

use super::super::common::primitives::{h_bool, h_string};

#[derive(Serialize, Deserialize)]
pub struct Animation {
    pub name: String,
    pub spritesheet: String,
    pub material: String,
    pub sequences: Vec<Sequence>,
    pub directions: Vec<Direction>,
    pub action_points: Vec<ActionPoint>,
}

pub fn animation_parser(i: &[u8]) -> IResult<&[u8], Animation> {
    map(
        tuple((
            h_string,
            h_string,
            h_string,
            length_count(le_u32, sequence_parser),
            length_count(le_u32, direction_parser),
            length_count(le_u32, action_point_parser),
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

#[derive(Serialize, Deserialize)]
pub struct Frame {
    pub image_name: String,
    pub frame_number: i32,
    pub duration: i32,
}

pub fn frame_parser(i: &[u8]) -> IResult<&[u8], Frame> {
    map(
        tuple((h_string, le_i32, le_i32)),
        |(image_name, frame_number, duration)| Frame {
            image_name,
            frame_number,
            duration,
        },
    )(i)
}

#[derive(Serialize, Deserialize)]
pub struct Sequence {
    pub frames: Vec<Frame>,
    pub name: String,
    pub id: i32,
    pub is_loop: bool,
    pub no_flip: bool,
    pub fallback: bool,
}

pub fn sequence_parser(i: &[u8]) -> IResult<&[u8], Sequence> {
    map(
        tuple((
            length_count(le_u32, frame_parser),
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

#[derive(Serialize, Deserialize)]
pub struct ActionPoint {
    pub name: String,
    pub id: i32,
    pub points: HashMap<(i32, i32, i32), (i32, i32)>,
}

pub fn action_point_parser(i: &[u8]) -> IResult<&[u8], ActionPoint> {
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
