use crate::halley::assets::{compression, utils::pathify, utils::unpathify};
use cookie_factory::{SerializeFn, WriteContext};
use derivative::Derivative;
use derive_new::new;
use nom::IResult;
use serde::{Deserialize, Serialize};
use std::{fmt::Debug, path::Path};

pub trait HalleyPackParser {
    fn parse<'a>(i: &'a [u8], secret: &str) -> IResult<&'a [u8], &'a dyn HalleyPack>;
}

pub trait HalleyPack: Writable + Debug {
    fn sections(&self) -> &Vec<Box<dyn HpkSection>>;
    fn add_section(&mut self, section: Box<dyn HpkSection>);
    fn get_asset_data(&self, asset: Box<&dyn HpkAsset>) -> Vec<u8>;
    fn data(&self) -> &[u8];
    fn add_data(&mut self, data: &[u8], compression: Option<String>) -> (usize, usize);
    // fn get_boxed(&self) -> Box<Self>;
}

#[derive(Derivative, new)]
#[derivative(Debug, Default)]
pub struct HalleyPackData {
    //iv: [u8; 16],
    //asset_db_start_pos: u64,
    #[derivative(Default(value = "vec![]"))]
    asset_db: Vec<Box<dyn HpkSection>>,

    #[derivative(Debug = "ignore")]
    #[derivative(Default(value = "vec![]"))]
    data: Vec<u8>,
}

impl HalleyPack for HalleyPackData {
    fn sections(&self) -> &Vec<Box<dyn HpkSection>> {
        &self.asset_db
    }

    fn add_section(&mut self, section: Box<dyn HpkSection>) {
        self.asset_db.push(section);
    }

    fn get_asset_data(&self, asset: Box<&dyn HpkAsset>) -> Vec<u8> {
        let pos = asset.pos();
        let data = &self.data[pos..pos + asset.size()];
        match asset.get_asset_compression() {
            Some(compression) => compression::decompress(data, &compression),
            None => data.to_vec(),
        }
    }

    fn data(&self) -> &[u8] {
        &self.data
    }

    fn add_data(&mut self, data: &[u8], compression: Option<String>) -> (usize, usize) {
        let data = match compression {
            Some(compression) => compression::compress(data, &compression),
            None => data.to_vec(),
        };
        let pos = self.data.len();
        self.data.extend_from_slice(&data);
        (pos, data.len())
    }
}

pub trait HpkSection
where
    Self: HpkSectionUnpackable + Writable + Debug,
{
    fn new(asset_type: u32) -> Self
    where
        Self: Sized;
    fn asset_type(&self) -> u32;
    fn assets(&self) -> Vec<Box<&dyn HpkAsset>>;
    fn add_asset(&mut self, pack: &mut dyn HalleyPack, path: &Path, relative_path: &Path);
}

pub trait HpkAsset: Writable + Debug {
    fn name(&self) -> &String;
    fn pos(&self) -> usize;
    fn size(&self) -> usize;
    fn set_pos_size(&mut self, pos: usize, size: usize);
    fn serialize_properties(&self, filaname: &Path) -> Option<std::io::Error>;
    fn get_asset_compression(&self) -> Option<String>;
    fn get_compression(&self) -> Option<String>;
}

pub trait HpkSectionUnpackable {
    fn get_unknown_file_type_ending(&self) -> &str {
        ".ukn"
    }

    fn get_file_name_extension(&self, compression: Option<String>) -> &str {
        match compression {
            Some(compression) => match compression.as_str() {
                "png" => ".png",
                _ => "",
            },
            None => "",
        }
    }

    fn modify_file_on_unpack(&self, i: &[u8]) -> Vec<u8> {
        i.to_owned()
    }

    fn modify_file_on_repack(&self, i: &[u8]) -> Vec<u8> {
        i.to_owned()
    }

    fn get_asset_filename(&self, asset: &dyn HpkAsset) -> String {
        let name = asset.name();
        let u_ext = self.get_unknown_file_type_ending();
        let ext = self.get_file_name_extension(asset.get_compression());
        format!("{}{}", pathify(name, u_ext), ext)
    }

    fn get_asset_name(&self, filename: &str, compression: Option<String>) -> String {
        let u_ext = self.get_unknown_file_type_ending();
        let ext = self.get_file_name_extension(compression);
        unpathify(filename, u_ext)
            .strip_suffix(ext)
            .unwrap()
            .to_string()
    }
}

// pub trait ParsablePack {
//     fn parse<'a>(i: &'a [u8], secret: &str) -> IResult<&'a [u8], Box<Self>>;
// }

pub trait Parsable
where
    Self: Sized,
{
    fn parse(i: &[u8]) -> IResult<&[u8], Self>;
}

pub trait Writable {
    fn write<'a>(&'a self) -> Box<dyn SerializeFn<Vec<u8>> + 'a>;
}

pub fn unpack_transform<T: Parsable + Serialize>(i: &[u8]) -> Vec<u8> {
    let (_, t) = T::parse(i).unwrap();
    serde_json::to_string_pretty(&t).unwrap().into_bytes()
}

pub fn pack_transform<'a, T: Writable + Deserialize<'a>>(i: &'a [u8]) -> Vec<u8> {
    let t: T = serde_json::from_slice(i).unwrap();
    let writer = t.write();
    let w = WriteContext::from(Vec::new());
    writer(w).unwrap().write
}
