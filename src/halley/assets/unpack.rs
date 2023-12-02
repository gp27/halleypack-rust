use super::property_file;
use crate::halley::versions::common::hpk::{HalleyPack, HalleyPackData, HpkSection};
use anyhow::anyhow;
use indexmap::IndexMap;
use std::{
    fs::{create_dir_all, File},
    io::Write,
    path::Path,
};
use thiserror::Error;
use walkdir::WalkDir;

static SECTION_PREFIX: &str = "section_";

type SectionProps = IndexMap<String, i32>;

pub fn unpack_halley_pk(pack: &dyn HalleyPack, path: &Path) -> Result<(), anyhow::Error> {
    create_dir_all(path)?;

    if !path.is_dir() {
        return Err(anyhow!("Path is not a directory",));
    }

    for (i, section) in pack.sections().iter().enumerate() {
        let section_name = format!("{}{}", SECTION_PREFIX, i);
        let mut map = SectionProps::new();
        map.insert("asset_type".to_string(), section.asset_type());

        let section_path = &path.join(section_name);

        property_file::write(section_path, &map)?;
        create_dir_all(&section_path)?;

        for asset in section.assets().into_iter() {
            let filename = section.get_asset_filename(*asset);
            let file_path = section_path.join(&filename);
            asset.serialize_properties(&file_path)?;

            let data = pack.get_asset_data(asset);
            let data = section.modify_file_on_unpack(&data)?;

            let parent = file_path.parent().unwrap();

            if !parent.exists() {
                create_dir_all(parent)?;
            }

            let mut file = File::create(&file_path)?;
            file.write_all(&data)?;
        }
    }

    return Ok(());
}

#[derive(Error, Debug)]
pub enum PackError {
    #[error("Invalid file {0} in sections folder")]
    InvalidFileInSections(String),

    #[error("Missing asset type for section_{0}")]
    MissingAssetType(i32),
}

pub fn pack_halley_pk<Section: HpkSection + 'static>(
    path: &Path,
) -> Result<Box<dyn HalleyPack>, anyhow::Error> {
    let mut dir = path.read_dir()?;

    let non_section_entry = dir.find_map(|entry| {
        let entry = entry.unwrap();
        let is_dir = entry.file_type().unwrap().is_dir();

        let filename = entry.file_name().to_str().unwrap().to_owned();

        let is_section = filename.starts_with(SECTION_PREFIX)
            && (is_dir || filename.ends_with(property_file::EXT));
        if is_section {
            return None;
        }

        Some(filename)
    });

    if non_section_entry.is_some() {
        return Err(PackError::InvalidFileInSections(non_section_entry.unwrap()).into());
    }

    let mut pack = HalleyPackData::default();

    let mut index = 0;

    loop {
        let section_name = format!("{}{}", SECTION_PREFIX, index);
        let section_filename = path.join(section_name);

        if !section_filename.exists() {
            break;
        }

        let section_props: SectionProps = property_file::read(section_filename.as_path())?;
        let section_type = *section_props
            .get("asset_type")
            .ok_or(PackError::MissingAssetType(index))?;

        let mut section = Section::new(section_type)?;

        let paths = WalkDir::new(&section_filename)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| {
                let e = e.ok()?;
                let path = e.path();

                if e.file_type().is_file() && !path.to_str().unwrap().ends_with(property_file::EXT)
                {
                    return Some(path.to_path_buf());
                }
                None
            });

        for file_path in paths {
            if file_path.is_dir() {
                continue;
            }

            let relative_path = file_path.strip_prefix(&section_filename)?;
            section.add_asset(&mut pack, file_path.as_path(), relative_path)?;
        }
        pack.add_section(Box::new(section));

        index += 1;
    }
    Ok(Box::new(pack))
}
