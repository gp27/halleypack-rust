use crate::halley::versions::common::primitives::{h_var_i, h_var_u};

use super::super::common::primitives::{h_bool, h_var_string};
use nom::{
    combinator::{cond, flat_map, map, peek},
    multi::length_count,
    number::complete::{le_f32, u8},
    sequence::tuple,
    IResult,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
pub struct SpriteSheet {
    pub v: u8,
    pub name: String,
    pub sprites: Vec<Sprite>,
    pub sprite_idx: HashMap<String, i32>,
    pub frame_tags: Vec<FrameTag>,
    pub def_material_name: Option<String>,
    //pub palette_name: Option<String>,
}

pub fn spritesheet_parser(i: &[u8]) -> IResult<&[u8], SpriteSheet> {
    let versioned = flat_map(peek(u8), |v| {
        tuple((
            cond(v as i32 <= 255, u8),
            h_var_string,
            length_count(h_var_u, sprite_parser),
            sprite_idx_parser,
            length_count(h_var_u, frame_tag_parser),
            cond(v >= 1, h_var_string),
            //cond(v >= 2, h_short_string), // not yet implemented in this version
        ))
    });

    map(
        versioned,
        |(v, name, sprites, sprite_idx, frame_tags, def_material_name)| SpriteSheet {
            v: v.unwrap_or(0),
            name,
            sprites,
            sprite_idx,
            frame_tags,
            def_material_name,
        },
    )(i)
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Sprite {
    pub pivot: (f32, f32),
    pub orig_pivot: (i32, i32),
    pub size: (f32, f32),
    pub coords: (f32, f32, f32, f32),
    pub rotated: bool,
    pub trim_border: (i16, i16, i16, i16),
    pub slices: (i16, i16, i16, i16),
}

pub fn sprite_parser(i: &[u8]) -> IResult<&[u8], Sprite> {
    map(
        tuple((
            tuple((le_f32, le_f32)),
            tuple((h_var_i, h_var_i)),
            tuple((le_f32, le_f32)),
            tuple((le_f32, le_f32, le_f32, le_f32)),
            h_bool,
            tuple((h_var_i, h_var_i, h_var_i, h_var_i)),
            tuple((h_var_i, h_var_i, h_var_i, h_var_i)),
        )),
        |(pivot, orig_pivot, size, coords, rotated, trim_border, slices)| Sprite {
            pivot,
            orig_pivot: (orig_pivot.0 as i32, orig_pivot.1 as i32),
            size: (size.0 as f32, size.1 as f32),
            coords,
            rotated,
            trim_border: (
                trim_border.0 as i16,
                trim_border.1 as i16,
                trim_border.2 as i16,
                trim_border.3 as i16,
            ),
            slices: (
                slices.0 as i16,
                slices.1 as i16,
                slices.2 as i16,
                slices.3 as i16,
            ),
        },
    )(i)
}

pub fn sprite_idx_parser(i: &[u8]) -> IResult<&[u8], HashMap<String, i32>> {
    length_count(h_var_u, tuple((h_var_string, h_var_u)))(i).map(|(i, entries)| {
        let mut map = HashMap::new();
        for (k, v) in entries {
            map.insert(k, v as i32);
        }
        (i, map)
    })
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FrameTag {
    pub name: String,
    pub to: i32,
    pub from: i32,
}

pub fn frame_tag_parser(i: &[u8]) -> IResult<&[u8], FrameTag> {
    map(
        tuple((h_var_string, h_var_i, h_var_i)),
        |(name, to, from)| FrameTag {
            name,
            to: to as i32,
            from: from as i32,
        },
    )(i)
}
