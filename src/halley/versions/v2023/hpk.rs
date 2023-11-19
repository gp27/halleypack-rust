use super::{
    super::common::{
        hpk::{HalleyPack, HpkAsset, HpkSection, HpkSectionUnpackable, Parsable},
        {
            config::{h_config_file, h_confignode, wh_confignode, ConfigNode},
            primitives::{h_pos_size, h_string},
        },
    },
    animation::animation_parser,
    hlif::hlif_parser,
    spritesheet::spritesheet_parser,
};
use crate::halley::versions::common::{
    hpk::{HalleyPackData, Writable},
    primitives::{wh_pos_size, wh_string},
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
use std::path::Path;

pub struct HalleyPackV2023 {}

impl HalleyPackV2023 {
    pub fn load(path: &Path, secret: Option<&str>) -> Result<Box<dyn HalleyPack>, std::io::Error> {
        HalleyPackData::load::<HpkSectionV2023>(path, secret)
    }
}

#[derive(Debug, FromPrimitive, ToPrimitive)]
pub enum AssetTypeV2023 {
    BINARY = 0,
    TEXT = 1,
    CONFIG = 2,
    GAMEPROPERTIES = 3,
    TEXTURE = 4,
    SHADER = 5,
    MATERIAL = 6,
    IMAGE = 7,
    SPRITESHEET = 8,
    SPRITE = 9,
    ANIMATION = 10,
    FONT = 11,
    AUDIOCLIP = 12,
    AUDIOOBJECT = 13,
    AUDIOEVENT = 14,
    MESH = 15,
    MESHANIMATION = 16,
    VARIABLETABLE = 17,
    RENDERGHRAPHDEFINITION,
    SCRIPTGHRAPH,
    NAVMESHSET,
    PREFAB,
    SCENE,
    UIDDEFINITION,
}

#[derive(Debug)]
pub struct HpkSectionV2023
where
    Self: HpkSection,
{
    pub asset_type: AssetTypeV2023,
    pub section_index: i32,
    pub assets: Vec<HpkAssetV2023>,
}

impl HpkSection for HpkSectionV2023 {
    fn new(asset_type: u32) -> Self {
        HpkSectionV2023 {
            asset_type: num::FromPrimitive::from_u32(asset_type).unwrap(),
            section_index: asset_type as i32,
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

    fn add_asset(&mut self, pack: &mut dyn HalleyPack, path: &Path, relative_path: &Path) {
        let (config, data) =
            super::super::super::assets::property_file::read_with_file_data::<ConfigNode>(path)
                .unwrap();

        let data = self.modify_file_on_repack(&data);

        let name = self.get_asset_name(relative_path.to_str().unwrap(), get_compression(&config));

        let mut asset = HpkAssetV2023 {
            name,
            pos: 0,
            size: 0,
            config,
        };

        let compression = asset.get_asset_compression();
        let (pos, size) = pack.add_data(&data, compression);

        asset.set_pos_size(pos, size);

        self.assets.push(asset);
    }
}

impl HpkSectionUnpackable for HpkSectionV2023 {
    fn get_unknown_file_type_ending(&self) -> &str {
        match self.asset_type {
            AssetTypeV2023::SPRITESHEET => ".sheet.json",
            AssetTypeV2023::ANIMATION => ".anim.json",
            AssetTypeV2023::CONFIG => ".config.json",
            AssetTypeV2023::GAMEPROPERTIES => ".game.json",
            AssetTypeV2023::AUDIOOBJECT => ".audioobject.json",
            AssetTypeV2023::AUDIOEVENT => ".audioevent.json",
            _ => ".ukn",
        }
    }

    fn modify_file_on_unpack<'a>(&self, i: &'a [u8]) -> Vec<u8> {
        // match self.asset_type {
        //     AssetTypeV2023::SPRITESHEET | AssetTypeV2023::ANIMATION | AssetTypeV2023::CONFIG => {
        //         //println!("asset_type: {:?}", self.asset_type);
        //         //println!("i: {:x?}", &i[0..min(3000, i.len())]);
        //     }
        //     _ => {}
        // }

        let j = match self.asset_type {
            AssetTypeV2023::SPRITESHEET => match spritesheet_parser(i) {
                Ok((_, spritesheet)) => json5::to_string(&spritesheet).unwrap(),
                Err(err) => {
                    println!("parse error: {:#?}", err);
                    return i.to_owned();
                }
            },
            AssetTypeV2023::ANIMATION => {
                let (_, animation) = animation_parser(i).unwrap();
                json5::to_string(&animation).unwrap()
            }
            AssetTypeV2023::CONFIG => {
                let (_, config) = h_config_file(i).unwrap();
                json5::to_string(&config).unwrap()
            }
            // AssetTypeV2023::TEXTURE => {
            //     let (_, texture) = hlif_parser(i).unwrap();
            //     texture.to_vec()
            // }
            _ => return i.to_owned(),
        };

        j.into_bytes()
    }
}

impl Parsable for HpkSectionV2023 {
    fn parse(i: &[u8]) -> IResult<&[u8], Self> {
        map(
            tuple((le_i32, le_i32, length_count(le_u32, HpkAssetV2023::parse))),
            |(asset_type, section_index, assets)| HpkSectionV2023 {
                asset_type: num::FromPrimitive::from_i32(asset_type).unwrap(),
                section_index,
                assets,
            },
        )(i)
    }
}

impl Writable for HpkSectionV2023 {
    fn write<'a>(&'a self) -> Box<dyn SerializeFn<Vec<u8>> + 'a> {
        let writer = wh_tuple((
            w_le_i32(num_traits::ToPrimitive::to_i32(&self.asset_type).unwrap()),
            w_le_i32(self.section_index),
            w_le_u32(self.assets.len() as u32),
            wh_all(self.assets.iter().map(|a| a.write())),
        ));

        Box::new(writer)
    }
}

#[derive(Debug)]
pub struct HpkAssetV2023 {
    pub name: String,
    pub pos: usize,
    pub size: usize,
    pub config: ConfigNode,
}

impl HpkAsset for HpkAssetV2023 {
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

    fn serialize_properties(&self, filename: &std::path::Path) -> Option<std::io::Error> {
        super::super::super::assets::property_file::write(filename, &self.config).err()
    }

    fn get_asset_compression(&self) -> Option<String> {
        match &self.config {
            ConfigNode::Map(map) => match map.get("asset_compression") {
                Some(ConfigNode::String(s)) => Some(s.to_owned()),
                _ => None,
            },
            _ => None,
        }
    }

    fn get_compression(&self) -> Option<String> {
        get_compression(&self.config)
    }
}

impl Parsable for HpkAssetV2023 {
    fn parse(i: &[u8]) -> IResult<&[u8], Self> {
        map(
            tuple((h_string, h_pos_size, h_confignode)),
            |(name, (pos, size), config)| HpkAssetV2023 {
                name,
                pos,
                size,
                config,
            },
        )(i)
    }
}

impl Writable for HpkAssetV2023 {
    fn write<'a>(&'a self) -> Box<dyn SerializeFn<Vec<u8>> + 'a> {
        let writer = wh_tuple((
            wh_string(&self.name),
            wh_pos_size(self.pos, self.size),
            wh_confignode(&self.config),
        ));
        Box::new(writer)
    }
}

fn get_compression(config: &ConfigNode) -> Option<String> {
    match config {
        ConfigNode::Map(map) => match map.get("compression") {
            Some(ConfigNode::String(s)) => Some(s.to_owned()),
            _ => None,
        },
        _ => None,
    }
}
