use super::super::common::{
    hpk::{Parsable, Writable},
    primitives::{h_bool, h_string, wh_bool, wh_string},
};
use cookie_factory::{
    bytes::{le_f32 as w_le_f32, le_i16 as w_le_i16, le_i32 as w_le_i32, le_u32 as w_le_u32},
    multi::all as wh_all,
    sequence::tuple as wh_tuple,
    SerializeFn,
};
use indexmap::IndexMap;
use nom::{
    combinator::map,
    multi::length_count,
    number::complete::{le_f32, le_i16, le_i32, le_u32},
    sequence::tuple,
    IResult,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct SpriteSheet {
    pub name: String,
    pub sprites: Vec<Sprite>,
    pub sprite_idx: SpriteIdx,
    pub frame_tags: Vec<FrameTag>,
}

impl Parsable for SpriteSheet {
    fn parse(i: &[u8]) -> IResult<&[u8], Self> {
        map(
            tuple((
                h_string,
                length_count(le_u32, Sprite::parse),
                SpriteIdx::parse,
                length_count(le_u32, FrameTag::parse),
            )),
            |(name, sprites, sprite_idx, frame_tags)| SpriteSheet {
                name,
                sprites,
                sprite_idx,
                frame_tags,
            },
        )(i)
    }
}

impl Writable for SpriteSheet {
    fn write<'a>(&'a self) -> Box<dyn SerializeFn<Vec<u8>> + 'a> {
        let writer = wh_tuple((
            wh_string(&self.name),
            w_le_u32(self.sprites.len() as u32),
            wh_all(self.sprites.iter().map(|s| s.write())),
            self.sprite_idx.write(),
            w_le_u32(self.frame_tags.len() as u32),
            wh_all(self.frame_tags.iter().map(|t| t.write())),
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
    pub duration: i32,
    pub rotated: bool,
    pub trim_border: (i16, i16, i16, i16),
    pub slices: (i16, i16, i16, i16),
}

impl Parsable for Sprite {
    fn parse(i: &[u8]) -> IResult<&[u8], Self> {
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
}

impl Writable for Sprite {
    fn write<'a>(&'a self) -> Box<dyn SerializeFn<Vec<u8>> + 'a> {
        let writer = wh_tuple((
            wh_tuple((w_le_f32(self.pivot.0), w_le_f32(self.pivot.1))),
            wh_tuple((w_le_i32(self.orig_pivot.0), w_le_i32(self.orig_pivot.1))),
            wh_tuple((w_le_f32(self.size.0), w_le_f32(self.size.1))),
            wh_tuple((
                w_le_f32(self.coords.0),
                w_le_f32(self.coords.1),
                w_le_f32(self.coords.2),
                w_le_f32(self.coords.3),
            )),
            w_le_i32(self.duration),
            wh_bool(self.rotated),
            wh_tuple((
                w_le_i16(self.trim_border.0),
                w_le_i16(self.trim_border.1),
                w_le_i16(self.trim_border.2),
                w_le_i16(self.trim_border.3),
            )),
            wh_tuple((
                w_le_i16(self.slices.0),
                w_le_i16(self.slices.1),
                w_le_i16(self.slices.2),
                w_le_i16(self.slices.3),
            )),
        ));
        Box::new(writer)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SpriteIdx(IndexMap<String, i32>);

impl Parsable for SpriteIdx {
    fn parse(i: &[u8]) -> IResult<&[u8], Self> {
        map(length_count(le_u32, tuple((h_string, le_i32))), |entries| {
            let mut map = IndexMap::new();
            for (k, v) in entries {
                map.insert(k, v);
            }
            SpriteIdx(map)
        })(i)
    }
}

impl Writable for SpriteIdx {
    fn write<'a>(&'a self) -> Box<dyn SerializeFn<Vec<u8>> + 'a> {
        let writer = wh_tuple((
            w_le_u32(self.0.len() as u32),
            wh_all(
                self.0
                    .iter()
                    .map(|(k, v)| wh_tuple((wh_string(k), w_le_i32(*v)))),
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
        map(tuple((h_string, le_i32, le_i32)), |(name, to, from)| {
            FrameTag { name, to, from }
        })(i)
    }
}

impl Writable for FrameTag {
    fn write<'a>(&'a self) -> Box<dyn SerializeFn<Vec<u8>> + 'a> {
        let writer = wh_tuple((
            wh_string(&self.name),
            w_le_i32(self.to),
            w_le_i32(self.from),
        ));
        Box::new(writer)
    }
}
