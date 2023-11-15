use std::{
    fs,
    io::{BufWriter, Write},
    path::Path,
};

use cookie_factory::WriteContext;
use serde::{Deserialize, Serialize};

use self::{
    assets::{unpack::pack_halley_pk, utils::get_dat_files},
    versions::{common::hpk::HalleyPack, v2020::hpk::HpkSectionV2020, v2023::hpk::HpkSectionV2023},
};

pub mod assets;
pub mod versions;

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum PackVersion {
    V2020,
    V2023,
}

pub fn unpack_assets(src: &Path, dst: &Path, pack_version: PackVersion) {
    let dat_files = get_dat_files(src);
    if !dst.exists() && !dat_files.is_empty() {
        fs::create_dir_all(dst).unwrap();
    }
    for dat_file in dat_files {
        let filename = dat_file.file_name().unwrap().to_str().unwrap();
        let dst_file = dst.join(filename);
        if dst_file.exists() {
            fs::remove_dir_all(&dst_file).unwrap();
        }
        fs::create_dir_all(&dst_file).unwrap();
        let pack = unpack_asset(&dat_file, pack_version);
        write_pack(pack, &dst_file);
    }
}

pub fn unpack_asset(path: &Path, pack_version: PackVersion) -> Box<dyn HalleyPack> {
    match pack_version {
        PackVersion::V2023 => pack_halley_pk::<HpkSectionV2023>(path).unwrap(),
        PackVersion::V2020 => pack_halley_pk::<HpkSectionV2020>(path).unwrap(),
    }
}

pub fn write_pack(pack: Box<dyn HalleyPack>, path: &Path) {
    let mut writer = BufWriter::new(fs::File::create(path).unwrap());
    let buf = vec![];
    let res = pack.write()(WriteContext {
        write: buf,
        position: 0,
    })
    .unwrap();
    writer.write_all(&res.write).unwrap();
}
