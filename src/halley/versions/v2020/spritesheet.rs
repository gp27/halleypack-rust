use crate::halley::versions::common::hpk::{Parsable, Writable};

use super::super::common::primitives::{h_bool, h_string, wh_bool, wh_string};
use cookie_factory::{
    bytes::{le_f32 as w_le_f32, le_i16 as w_le_i16, le_i32 as w_le_i32},
    multi::all as wh_all,
    sequence::tuple as wh_tuple,
    SerializeFn,
};
use nom::{
    combinator::map,
    multi::length_count,
    number::complete::{le_f32, le_i16, le_i32, le_u32},
    sequence::tuple,
    IResult,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
pub struct SpriteSheet {
    pub name: String,
    pub sprites: Vec<Sprite>,
    pub sprite_idx: HashMap<String, i32>,
    pub frame_tags: Vec<FrameTag>,
}

impl Parsable for SpriteSheet {
    fn parse(i: &[u8]) -> IResult<&[u8], Self> {
        spritesheet_parser(i).map(|(i, ss)| (i, ss))
    }
}

impl Writable for SpriteSheet {
    fn write<'a>(&'a self) -> Box<dyn SerializeFn<Vec<u8>> + 'a> {
        spritesheet_writer(self)
    }
}

pub fn spritesheet_parser(i: &[u8]) -> IResult<&[u8], SpriteSheet> {
    map(
        tuple((
            h_string,
            length_count(le_u32, sprite_parser),
            sprite_idx_parser,
            length_count(le_u32, frame_tag_parser),
        )),
        |(name, sprites, sprite_idx, frame_tags)| SpriteSheet {
            name,
            sprites,
            sprite_idx,
            frame_tags,
        },
    )(i)
}

fn spritesheet_writer<'a>(spritesheet: &'a SpriteSheet) -> Box<dyn SerializeFn<Vec<u8>> + 'a> {
    let writer = wh_tuple((
        wh_string(&spritesheet.name),
        w_le_i32(spritesheet.sprites.len() as i32),
        wh_all(spritesheet.sprites.iter().map(sprite_writer)),
        sprite_idx_writer(&spritesheet.sprite_idx),
        w_le_i32(spritesheet.frame_tags.len() as i32),
        wh_all(spritesheet.frame_tags.iter().map(frame_tag_writer)),
    ));
    Box::new(writer)
}

#[derive(Serialize, Deserialize)]
pub struct Sprite {
    pub pivot: (f32, f32),
    pub orig_pivot: (i32, i32),
    pub size: (f32, f32),
    pub coords: (f32, f32, f32, f32),
    pub duration: i32,
    pub rotated: bool,
    pub trim_border: (i16, i16, i16, i16),
    pub slices: (i16, i16, i16, i16),
}

fn sprite_parser(i: &[u8]) -> IResult<&[u8], Sprite> {
    map(
        tuple((
            tuple((le_f32, le_f32)),
            tuple((le_i32, le_i32)),
            tuple((le_f32, le_f32)),
            tuple((le_f32, le_f32, le_f32, le_f32)),
            le_i32,
            h_bool,
            tuple((le_i16, le_i16, le_i16, le_i16)),
            tuple((le_i16, le_i16, le_i16, le_i16)),
        )),
        |(pivot, orig_pivot, size, coords, duration, rotated, trim_border, slices)| Sprite {
            pivot,
            orig_pivot,
            size,
            coords,
            duration,
            rotated,
            trim_border,
            slices,
        },
    )(i)
}

fn sprite_writer(sprite: &Sprite) -> Box<dyn SerializeFn<Vec<u8>>> {
    let writer = wh_tuple((
        wh_tuple((w_le_f32(sprite.pivot.0), w_le_f32(sprite.pivot.1))),
        wh_tuple((w_le_i32(sprite.orig_pivot.0), w_le_i32(sprite.orig_pivot.1))),
        wh_tuple((w_le_f32(sprite.size.0), w_le_f32(sprite.size.1))),
        wh_tuple((
            w_le_f32(sprite.coords.0),
            w_le_f32(sprite.coords.1),
            w_le_f32(sprite.coords.2),
            w_le_f32(sprite.coords.3),
        )),
        w_le_i32(sprite.duration),
        wh_bool(sprite.rotated),
        wh_tuple((
            w_le_i16(sprite.trim_border.0),
            w_le_i16(sprite.trim_border.1),
            w_le_i16(sprite.trim_border.2),
            w_le_i16(sprite.trim_border.3),
        )),
        wh_tuple((
            w_le_i16(sprite.slices.0),
            w_le_i16(sprite.slices.1),
            w_le_i16(sprite.slices.2),
            w_le_i16(sprite.slices.3),
        )),
    ));
    Box::new(writer)
}

fn sprite_idx_parser(i: &[u8]) -> IResult<&[u8], HashMap<String, i32>> {
    length_count(le_u32, tuple((h_string, le_i32)))(i).map(|(i, entries)| {
        let mut map = HashMap::new();
        for (k, v) in entries {
            map.insert(k, v);
        }
        (i, map)
    })
}

fn sprite_idx_writer<'a>(
    sprite_idx: &'a HashMap<String, i32>,
) -> Box<dyn SerializeFn<Vec<u8>> + 'a> {
    let writer = wh_tuple((
        w_le_i32(sprite_idx.len() as i32),
        wh_all(
            sprite_idx
                .iter()
                .map(|(k, v)| wh_tuple((wh_string(k), w_le_i32(*v)))),
        ),
    ));
    Box::new(writer)
}

#[derive(Serialize, Deserialize)]
pub struct FrameTag {
    pub name: String,
    pub to: i32,
    pub from: i32,
}

fn frame_tag_parser(i: &[u8]) -> IResult<&[u8], FrameTag> {
    map(tuple((h_string, le_i32, le_i32)), |(name, to, from)| {
        FrameTag { name, to, from }
    })(i)
}

fn frame_tag_writer<'a>(tag: &'a FrameTag) -> Box<dyn SerializeFn<Vec<u8>> + 'a> {
    let writer = wh_tuple((wh_string(&tag.name), w_le_i32(tag.to), w_le_i32(tag.from)));
    Box::new(writer)
}
