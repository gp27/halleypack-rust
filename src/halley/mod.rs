use self::{
    assets::{
        unpack::{pack_halley_pk, unpack_halley_pk},
        utils::{get_dat_files, get_dat_folders},
    },
    versions::{
        common::hpk::HalleyPack,
        v2020::hpk::{HalleyPackV2020, HpkSectionV2020},
        v2023::hpk::{HalleyPackV2023, HpkSectionV2023},
    },
};
use clap::ValueEnum;
use cookie_factory::WriteContext;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::{BufWriter, Write},
    path::Path,
};

pub mod assets;
pub mod versions;

#[derive(Debug, Serialize, Deserialize, Clone, Copy, ValueEnum)]
pub enum PackVersion {
    V2020,
    V2023,
}

pub fn unpack_assets(src: &Path, dst: &Path, pack_version: PackVersion, secret: Option<&str>) {
    let dat_files = get_dat_files(src);
    if !dst.exists() && !dat_files.is_empty() {
        fs::create_dir_all(dst).unwrap();
    }

    dat_files.par_iter().for_each(|dat_file| {
        let filename = dat_file.file_name().unwrap().to_str().unwrap();
        let dst_file = dst.join(filename);
        if dst_file.exists() {
            fs::remove_dir_all(&dst_file).unwrap();
        }
        fs::create_dir_all(&dst_file).unwrap();
        let pack = read_pack(dat_file, pack_version, secret);
        unpack_halley_pk(&*pack, &dst_file).unwrap();
    });
}

pub fn pack_assets(src: &Path, dst: &Path, pack_version: PackVersion, secret: Option<&str>) {
    let dat_folders = get_dat_folders(src);
    if !dst.exists() {
        panic!("Destination folder does not exist");
    }
    dat_folders.par_iter().for_each(|dat_folder| {
        let filename = dat_folder.file_name().unwrap().to_str().unwrap();
        let dst_file = dst.join(filename);
        // if dst_file.exists() {
        //     fs::remove_file(&dst_file).unwrap();
        // }
        let pack = pack_asset(dat_folder, pack_version);
        write_pack(pack, &dst_file, secret);
    });
}

pub fn read_pack(
    path: &Path,
    pack_version: PackVersion,
    secret: Option<&str>,
) -> Box<dyn HalleyPack> {
    match pack_version {
        PackVersion::V2023 => HalleyPackV2023::load(path, secret).unwrap(),
        PackVersion::V2020 => HalleyPackV2020::load(path, secret).unwrap(),
    }
}

pub fn pack_asset(path: &Path, pack_version: PackVersion) -> Box<dyn HalleyPack> {
    match pack_version {
        PackVersion::V2023 => pack_halley_pk::<HpkSectionV2023>(path).unwrap(),
        PackVersion::V2020 => pack_halley_pk::<HpkSectionV2020>(path).unwrap(),
    }
}

pub fn write_pack(pack: Box<dyn HalleyPack>, path: &Path, secret: Option<&str>) {
    let mut writer = BufWriter::new(fs::File::create(path).unwrap());
    let buf = vec![];
    let res = pack.write()(WriteContext {
        write: buf,
        position: 0,
    })
    .unwrap();
    writer.write_all(&res.write).unwrap();
}
