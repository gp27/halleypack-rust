use std::collections::HashMap;

use crate::halley::versions::common::{
    hpk::{Parsable, Writable},
    primitives::{
        h_bool, h_var_i, h_var_string, h_var_u, wh_bool, wh_var_i, wh_var_string, wh_var_u,
    },
};
use cookie_factory::{
    bytes::{le_f32 as w_le_f32, le_u8 as w_le_u8},
    combinator::cond as wh_cond,
    multi::all as wh_all,
    sequence::tuple as wh_tuple,
    SerializeFn,
};
use nom::{
    combinator::{cond, flat_map, map, peek},
    multi::length_count,
    number::complete::{le_f32, u8},
    sequence::tuple,
    IResult,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct SpriteSheet {
    pub v: u8,
    pub name: String,
    pub sprites: Vec<Sprite>,
    pub sprite_idx: SpriteIdx,
    pub frame_tags: Vec<FrameTag>,
    pub def_material_name: Option<String>,
    pub palette_name: Option<String>,
}

impl Parsable for SpriteSheet {
    fn parse(i: &[u8]) -> IResult<&[u8], Self> {
        let versioned = flat_map(peek(u8), |v| {
            tuple((
                cond(v as i32 <= 255, u8),
                h_var_string,
                length_count(h_var_u, Sprite::parse),
                SpriteIdx::parse,
                length_count(h_var_u, FrameTag::parse),
                cond(v >= 1, h_var_string),
                cond(v >= 2, h_var_string),
            ))
        });

        map(
            versioned,
            |(v, name, sprites, sprite_idx, frame_tags, def_material_name, palette_name)| {
                SpriteSheet {
                    v: v.unwrap_or(0),
                    name,
                    sprites,
                    sprite_idx,
                    frame_tags,
                    def_material_name,
                    palette_name,
                }
            },
        )(i)
    }
}

impl Writable for SpriteSheet {
    fn write<'a>(&'a self) -> Box<dyn cookie_factory::SerializeFn<Vec<u8>> + 'a> {
        let writer = wh_tuple((
            w_le_u8(self.v), //wh_cond(self.v as i32 <= 255, w_le_u8(self.v)),
            wh_var_string(&self.name),
            wh_var_u(self.sprites.len() as u64),
            wh_all(self.sprites.iter().map(|s| s.write())),
            self.sprite_idx.write(),
            wh_var_u(self.frame_tags.len() as u64),
            wh_all(self.frame_tags.iter().map(|t| t.write())),
            wh_cond(
                self.v >= 1,
                wh_var_string(&self.def_material_name.clone().unwrap()),
            ),
            wh_cond(
                self.v >= 2,
                wh_var_string(&self.palette_name.clone().unwrap()),
            ),
        ));
        Box::new(writer)
    }
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

impl Parsable for Sprite {
    fn parse(i: &[u8]) -> IResult<&[u8], Self> {
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
}

impl Writable for Sprite {
    fn write<'a>(&'a self) -> Box<dyn SerializeFn<Vec<u8>> + 'a> {
        let writer = wh_tuple((
            wh_tuple((w_le_f32(self.pivot.0), w_le_f32(self.pivot.1))),
            wh_tuple((
                wh_var_i(self.orig_pivot.0 as i64),
                wh_var_i(self.orig_pivot.1 as i64),
            )),
            wh_tuple((w_le_f32(self.size.0), w_le_f32(self.size.1))),
            wh_tuple((
                w_le_f32(self.coords.0),
                w_le_f32(self.coords.1),
                w_le_f32(self.coords.2),
                w_le_f32(self.coords.3),
            )),
            wh_bool(self.rotated),
            wh_tuple((
                wh_var_i(self.trim_border.0 as i64),
                wh_var_i(self.trim_border.1 as i64),
                wh_var_i(self.trim_border.2 as i64),
                wh_var_i(self.trim_border.3 as i64),
            )),
            wh_tuple((
                wh_var_i(self.slices.0 as i64),
                wh_var_i(self.slices.1 as i64),
                wh_var_i(self.slices.2 as i64),
                wh_var_i(self.slices.3 as i64),
            )),
        ));
        Box::new(writer)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SpriteIdx(HashMap<String, i32>);

impl Parsable for SpriteIdx {
    fn parse(i: &[u8]) -> IResult<&[u8], Self> {
        map(
            length_count(h_var_u, tuple((h_var_string, h_var_i))),
            |entries| {
                let mut map = HashMap::new();
                for (k, v) in entries {
                    map.insert(k, v as i32);
                }
                SpriteIdx(map)
            },
        )(i)
    }
}

impl Writable for SpriteIdx {
    fn write<'a>(&'a self) -> Box<dyn SerializeFn<Vec<u8>> + 'a> {
        let writer = wh_tuple((
            wh_var_u(self.0.len() as u64),
            wh_all(
                self.0
                    .iter()
                    .map(|(k, v)| wh_tuple((wh_var_string(k), wh_var_i(*v as i64)))),
            ),
        ));
        Box::new(writer)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FrameTag {
    pub name: String,
    pub to: i32,
    pub from: i32,
}

impl Parsable for FrameTag {
    fn parse(i: &[u8]) -> IResult<&[u8], Self> {
        map(
            tuple((h_var_string, h_var_i, h_var_i)),
            |(name, to, from)| FrameTag {
                name,
                to: to as i32,
                from: from as i32,
            },
        )(i)
    }
}

impl Writable for FrameTag {
    fn write<'a>(&'a self) -> Box<dyn SerializeFn<Vec<u8>> + 'a> {
        let writer = wh_tuple((
            wh_var_string(&self.name),
            wh_var_i(self.to as i64),
            wh_var_i(self.from as i64),
        ));
        Box::new(writer)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SpriteResource {
    pub name: String,
    pub idx: u64,
}

impl Parsable for SpriteResource {
    fn parse(i: &[u8]) -> IResult<&[u8], Self> {
        map(tuple((h_var_string, h_var_u)), |(name, idx)| {
            SpriteResource { name, idx }
        })(i)
    }
}

impl Writable for SpriteResource {
    fn write<'a>(&'a self) -> Box<dyn SerializeFn<Vec<u8>> + 'a> {
        let writer = wh_tuple((wh_var_string(&self.name), wh_var_u(self.idx)));
        Box::new(writer)
    }
}
