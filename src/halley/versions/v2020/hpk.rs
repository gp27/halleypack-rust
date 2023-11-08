use crate::halley::versions::common::{
    config::ConfigFile,
    hpk::{pack_transform, unpack_transform},
};

use super::{
    super::common::{
        hpk::{HalleyPack, HpkAsset, HpkSection, HpkSectionUnpackable, Parsable, Writable},
        hpk_parse::parse_hpk,
        primitives::{h_hashmap, h_pos_size, h_string},
        primitives::{wh_hashmap, wh_pos_size, wh_string},
    },
    animation::Animation,
    spritesheet::SpriteSheet,
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
use num_derive::{FromPrimitive, ToPrimitive};
use std::collections::HashMap;

// pub trait HalleyPackV2020: ParsablePack {
//     fn parse<'a>(i: &'a [u8], secret: &str) -> IResult<&'a [u8], Box<dyn HalleyPack>> {
//         let (_, pack) = parse_hpk::<HpkSectionV2020>(i, secret).unwrap();
//         Ok((i, Box::new(pack)))
//     }
// }

pub fn halley_pack_v2020_parse<'a>(
    i: &'a [u8],
    secret: &str,
) -> IResult<&'a [u8], Box<dyn HalleyPack>> {
    let (_, pack) = parse_hpk::<HpkSectionV2020>(i, secret).unwrap();
    Ok((i, Box::new(pack)))
}

#[derive(Debug, FromPrimitive, ToPrimitive)]
pub enum AssetTypeV2020 {
    BINARY = 0,
    TEXT = 1,
    CONFIG = 2,
    TEXTURE,
    SHADER,
    MATERIAL,
    IMAGE,
    SPRITE,
    SPRITESHEET,
    ANIMATION,
    FONT,
    AUDIOCLIP,
    AUDIOEVENT,
    MESH,
    MESHANIMATION,
    VARIABLETABLE,
}

pub type HpkPropertiesV2020 = HashMap<String, String>;

#[derive(Debug)]
pub struct HpkSectionV2020
where
    Self: HpkSection,
{
    pub asset_type: AssetTypeV2020,
    pub assets: Vec<HpkAssetV2020>,
}

impl HpkSection for HpkSectionV2020 {
    fn new(asset_type: u32) -> Self {
        HpkSectionV2020 {
            asset_type: num::FromPrimitive::from_u32(asset_type).unwrap(),
            assets: vec![],
        }
    }

    fn asset_type(&self) -> u32 {
        num_traits::ToPrimitive::to_u32(&self.asset_type).unwrap()
    }

    fn assets(&self) -> Vec<Box<&dyn HpkAsset>> {
        self.assets
            .iter()
            .map(|a| Box::new(a as &dyn HpkAsset))
            .collect()
    }

    fn add_asset(
        &mut self,
        pack: &mut dyn HalleyPack,
        name: String,
        props_data: &[u8],
        asset_data: &[u8],
    ) {
        let props_data = std::str::from_utf8(props_data).unwrap();
        let props: HpkPropertiesV2020 = toml::from_str(props_data).unwrap();

        let mut asset = HpkAssetV2020 {
            name,
            pos: 0,
            size: 0,
            properties: props,
        };

        let compression = asset.get_asset_compression();
        let (pos, size) = pack.add_data(asset_data, compression);

        asset.set_pos_size(pos, size);

        self.assets.push(asset);
    }
}

impl HpkSectionUnpackable for HpkSectionV2020 {
    fn get_unknown_file_type_ending(&self) -> &str {
        match self.asset_type {
            AssetTypeV2020::SPRITESHEET => ".sheet.json",
            AssetTypeV2020::ANIMATION => ".anim.json",
            AssetTypeV2020::CONFIG => ".config.json",
            _ => ".ukn",
        }
    }

    fn get_file_name_extension(&self, asset_index: usize) -> &str {
        match self.assets[asset_index].get_compression() {
            Some(compression) => match compression.as_str() {
                "png" => ".png",
                _ => "",
            },
            None => "",
        }
    }

    fn modify_file_on_unpack<'a>(&self, i: &'a [u8]) -> Vec<u8> {
        match self.asset_type {
            AssetTypeV2020::SPRITESHEET => unpack_transform::<SpriteSheet>(i),
            AssetTypeV2020::ANIMATION => unpack_transform::<Animation>(i),
            AssetTypeV2020::CONFIG => unpack_transform::<ConfigFile>(i),
            _ => return i.to_owned(),
        }
    }

    fn modify_file_on_repack(&self, i: &[u8]) -> Vec<u8> {
        match self.asset_type {
            AssetTypeV2020::SPRITESHEET => pack_transform::<SpriteSheet>(i),
            _ => i.to_owned(),
        }
    }
}

impl Parsable for HpkSectionV2020 {
    fn parse(i: &[u8]) -> IResult<&[u8], Self> {
        map(
            tuple((le_i32, length_count(le_u32, HpkAssetV2020::parse))),
            |(asset_type, assets)| HpkSectionV2020 {
                asset_type: num::FromPrimitive::from_i32(asset_type).unwrap(),
                assets,
            },
        )(i)
    }
}

impl Writable for HpkSectionV2020 {
    fn write<'a>(&'a self) -> Box<dyn SerializeFn<Vec<u8>> + 'a> {
        let writer = wh_tuple((
            w_le_i32(num_traits::ToPrimitive::to_i32(&self.asset_type).unwrap()),
            w_le_u32(self.assets.len() as u32),
            wh_all(self.assets.iter().map(|a| a.write())),
        ));

        Box::new(writer)
    }
}

#[derive(Debug)]
pub struct HpkAssetV2020
where
    Self: HpkAsset,
{
    pub name: String,
    pub pos: usize,
    pub size: usize,
    pub properties: HpkPropertiesV2020,
}

impl HpkAsset for HpkAssetV2020 {
    fn name(&self) -> &String {
        &self.name
    }

    fn pos(&self) -> usize {
        self.pos
    }

    fn size(&self) -> usize {
        self.size
    }

    fn set_pos_size(&mut self, pos: usize, size: usize) {
        self.pos = pos;
        self.size = size;
    }

    fn get_serialized_properties(&self) -> Vec<u8> {
        toml::to_string_pretty(&self.properties)
            .unwrap()
            .into_bytes()
    }

    fn get_asset_compression(&self) -> Option<String> {
        self.properties
            .get("asset_compression")
            .map(|s| s.to_owned())
    }

    fn get_compression(&self) -> Option<String> {
        self.properties.get("compression").map(|s| s.to_owned())
    }
}

impl Parsable for HpkAssetV2020 {
    fn parse(i: &[u8]) -> IResult<&[u8], Self> {
        map(
            tuple((h_string, h_pos_size, h_hashmap)),
            |(name, (pos, size), properties)| HpkAssetV2020 {
                name,
                pos,
                size,
                properties,
            },
        )(i)
    }
}

impl Writable for HpkAssetV2020 {
    fn write<'a>(&'a self) -> Box<dyn SerializeFn<Vec<u8>> + 'a> {
        let writer = wh_tuple((
            wh_string(&self.name),
            wh_pos_size(self.pos, self.size),
            wh_hashmap(&self.properties),
        ));
        Box::new(writer)
    }
}
