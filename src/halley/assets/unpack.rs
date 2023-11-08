use super::property_file;
use crate::halley::versions::common::hpk::{HalleyPack, HalleyPackData, HpkSection};
use std::{
    collections::HashMap,
    fs::{create_dir_all, File},
    io::Write,
    path::Path,
};
use walkdir::WalkDir;

static SECTION_PREFIX: &str = "section_";

type SectionProps = HashMap<String, u32>;

pub fn unpack_halley_pk(pack: &dyn HalleyPack, path: &Path) -> Result<(), std::io::Error> {
    create_dir_all(path)?;

    if !path.is_dir() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Path is not a directory",
        ));
    }

    for (i, section) in pack.sections().iter().enumerate() {
        let section_name = format!("{}{}", SECTION_PREFIX, i);
        let mut map = SectionProps::new();
        map.insert("asset_type".to_string(), section.asset_type());

        let section_path = &path.join(section_name);

        property_file::write(section_path, &map).unwrap();
        create_dir_all(&section_path)?;

        for asset in section.assets().into_iter() {
            let filename = section.get_asset_filename(*asset);
            let file_path = section_path.join(&filename);
            asset.serialize_properties(&file_path);

            let data = pack.get_asset_data(asset);
            let data = section.modify_file_on_unpack(&data);

            if !file_path.parent().unwrap().exists() {
                create_dir_all(file_path.parent().unwrap()).unwrap();
            }

            let mut file = File::create(&file_path).unwrap();
            file.write_all(&data).unwrap();
        }
    }

    return Ok(());
}

pub fn pack_halley_pk<Section: HpkSection + 'static>(
    path: &Path,
) -> Result<Box<dyn HalleyPack>, std::io::Error> {
    let mut dir = path.read_dir().unwrap();

    let has_sections_only = dir.all(|entry| {
        let entry = entry.unwrap();
        let is_dir = entry.file_type().unwrap().is_dir();

        let filename = entry.file_name().to_str().unwrap().to_owned();

        filename.starts_with(SECTION_PREFIX) && (is_dir || filename.ends_with(property_file::EXT))
    });

    if !has_sections_only {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Path does not contain only sections",
        ));
    }

    let mut pack = HalleyPackData::default();

    let mut index = 0;

    loop {
        let section_name = format!("{}{}", SECTION_PREFIX, index);
        let section_filename = path.join(section_name);

        if !section_filename.exists() {
            break;
        }

        let section_props: SectionProps = property_file::read(section_filename.as_path()).unwrap();
        let section_type = *section_props.get("asset_type").unwrap();

        let mut section = Section::new(section_type);

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

        let mut j = 0;

        paths.for_each(|file_path| {
            if file_path.is_dir() {
                return;
            }

            let relative_path = file_path.strip_prefix(&section_filename).unwrap();
            section.add_asset(&mut pack, file_path.as_path(), relative_path);

            j += 1;
        });
        pack.add_section(Box::new(section));

        index += 1;
    }
    Ok(Box::new(pack))
}
