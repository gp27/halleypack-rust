use serde::{de::DeserializeOwned, Serialize};
use std::{
    ffi::{OsStr, OsString},
    fs::create_dir_all,
    path::{Path, PathBuf},
};

pub static EXT: &str = ".pro.toml";

pub fn read_with_file_data<'a, T: DeserializeOwned + std::fmt::Debug>(
    asset_path: &Path,
) -> Result<(T, Vec<u8>), std::io::Error> {
    let props: T = read(asset_path).unwrap();
    let file_data = std::fs::read(asset_path)?;
    Ok((props, file_data))
}

pub fn read<T: DeserializeOwned>(asset_path: &Path) -> Result<T, std::io::Error> {
    let filename = append_to_path(asset_path, EXT);
    let data_str = if filename.exists() {
        std::fs::read_to_string(filename).ok().unwrap()
    } else {
        "".to_string()
    };

    let data: T = toml::from_str(&data_str).unwrap();
    Ok(data)
}

pub fn write<T: Serialize>(asset_path: &Path, data: &T) -> Result<(), std::io::Error> {
    let filename = append_to_path(asset_path, EXT);
    let data_str = toml::to_string_pretty(data).unwrap();
    if data_str.is_empty() {
        return Ok(());
    }
    if !filename.parent().unwrap().exists() {
        create_dir_all(filename.parent().unwrap()).unwrap();
    }
    std::fs::write(filename, data_str)?;
    Ok(())
}

fn append_to_path(p: impl Into<OsString>, s: impl AsRef<OsStr>) -> PathBuf {
    let mut p = p.into();
    p.push(s);
    p.into()
}
