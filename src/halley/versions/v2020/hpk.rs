use super::{
    super::common::{
        hpk::{HalleyPack, HpkAsset, HpkSection, HpkSectionUnpackable, Parsable, Writable},
        primitives::{h_map, h_pos_size, h_string},
        primitives::{wh_map, wh_pos_size, wh_string},
    },
    animation::Animation,
    spritesheet::SpriteSheet,
};
use crate::halley::{
    assets::{
        property_file,
        serialization::{get_format_from_ext, get_serialization_ext_from_path},
    },
    versions::common::{
        config::{ConfigFile, ConfigNode},
        hpk::{
            make_asset_type, pack_transform, unpack_transform, HalleyPackData, HalleyPackParseError,
        },
    },
};
use cookie_factory::{
    bytes::{le_i32 as w_le_i32, le_u32 as w_le_u32},
    multi::all as wh_all,
    sequence::tuple as wh_tuple,
    SerializeFn,
};
use indexmap::IndexMap;
use nom::{
    combinator::{map, map_res},
    multi::length_count,
    number::complete::{le_i32, le_u32},
    sequence::tuple,
    IResult,
};
use num_derive::{FromPrimitive, ToPrimitive};
use std::path::Path;

pub struct HalleyPackV2020 {}

impl HalleyPackV2020 {
    pub fn load(path: &Path, secret: Option<&str>) -> Result<Box<dyn HalleyPack>, std::io::Error> {
        HalleyPackData::load::<HpkSectionV2020>(path, secret)
    }
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
pub type HpkPropertiesV2020 = IndexMap<String, String>;

#[derive(Debug)]
pub struct HpkSectionV2020
where
    Self: HpkSection,
{
    pub asset_type: AssetTypeV2020,
    pub assets: Vec<HpkAssetV2020>,
}

impl HpkSection for HpkSectionV2020 {
    fn new(asset_type: i32) -> Result<Self, anyhow::Error> {
        let asset_type = make_asset_type(asset_type)?;
        Ok(HpkSectionV2020 {
            asset_type,
            assets: vec![],
        })
    }

    fn asset_type(&self) -> i32 {
        num_traits::ToPrimitive::to_i32(&self.asset_type).unwrap()
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
        path: &Path,
        relative_path: &Path,
    ) -> Result<(), anyhow::Error> {
        let (properties, data) = property_file::read_with_file_data::<HpkPropertiesV2020>(path)?;

        let serialization_ext = get_serialization_ext_from_path(path);
        let data = self.modify_data_on_repack(&data, serialization_ext)?;

        let name = self.get_asset_name(relative_path, serialization_ext);

        let mut asset = HpkAssetV2020 {
            name,
            pos: 0,
            size: 0,
            properties,
        };

        let compression = asset.get_asset_compression();

        let (pos, size) = pack.add_data(data, compression);

        asset.set_pos_size(pos, size);

        self.assets.push(asset);
        Ok(())
    }
}

impl HpkSectionUnpackable for HpkSectionV2020 {
    fn get_unknown_file_type_ending(&self) -> &str {
        match self.asset_type {
            AssetTypeV2020::SPRITESHEET => ".sheet",
            AssetTypeV2020::ANIMATION => ".anim",
            AssetTypeV2020::CONFIG => ".config",
            _ => "",
        }
    }

    fn modify_data_on_unpack(&self, i: &[u8]) -> Result<(Vec<u8>, &str), anyhow::Error> {
        match self.asset_type {
            AssetTypeV2020::SPRITESHEET => unpack_transform::<SpriteSheet, SpriteSheet>(i, None),
            AssetTypeV2020::ANIMATION => unpack_transform::<Animation, Animation>(i, None),
            AssetTypeV2020::CONFIG => {
                unpack_transform::<ConfigFile, ConfigNode>(i, Some(|c| c.root))
            }
            _ => Ok((i.into(), "")),
        }
    }

    fn modify_data_on_repack(&self, i: &[u8], ext: &str) -> Result<Vec<u8>, anyhow::Error> {
        let format = get_format_from_ext(ext);
        match self.asset_type {
            AssetTypeV2020::SPRITESHEET => {
                pack_transform::<SpriteSheet, SpriteSheet>(i, format, None)
            }
            AssetTypeV2020::ANIMATION => pack_transform::<Animation, Animation>(i, format, None),
            AssetTypeV2020::CONFIG => pack_transform::<ConfigFile, ConfigNode>(
                i,
                format,
                Some(|t| ConfigFile {
                    v: 2,
                    store_file_position: true,
                    root: t,
                }),
            ),
            _ => Ok(i.into()),
        }
    }
}

impl Parsable for HpkSectionV2020 {
    fn parse(i: &[u8]) -> IResult<&[u8], Self> {
        map_res(
            tuple((le_i32, length_count(le_u32, HpkAssetV2020::parse))),
            |(asset_type, assets)| {
                let asset_type = make_asset_type(asset_type)?;

                Ok::<HpkSectionV2020, HalleyPackParseError>(HpkSectionV2020 { asset_type, assets })
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

    fn serialize_properties(&self, filename: &std::path::Path) -> Result<(), anyhow::Error> {
        super::super::super::assets::property_file::write(filename, &self.properties)
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
            tuple((h_string, h_pos_size, h_map)),
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
            wh_map(&self.properties),
        ));
        Box::new(writer)
    }
}
