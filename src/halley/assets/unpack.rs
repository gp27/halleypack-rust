use crate::halley::versions::common::hpk::{HalleyPack, HalleyPackData, HpkSection};
use std::{
    collections::HashMap,
    ffi::{OsStr, OsString},
    fs::{self, create_dir_all, File},
    io::{BufReader, Read, Write},
    path::{Path, PathBuf},
};

static SECTION_PREFIX: &str = "section_";
static PROPERTIES_FILE_EXT: &str = ".pro.json";

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
        let mut map: SectionProps = HashMap::new();
        map.insert("asset_type".to_string(), section.asset_type());

        let map_data = serde_json::to_vec(&map).unwrap();

        let section_path = &path.join(section_name);

        write_property_file(&map_data, &section_path);
        create_dir_all(&section_path)?;

        let u_ext = section.get_unknown_file_type_ending();
        for (j, asset) in section.assets().into_iter().enumerate() {
            let ext = section.get_file_name_extension(j);
            let name = asset.name();
            let filename = format!("{}{}", pathify(name, u_ext), ext);
            let file_path = section_path.join(&filename);
            write_property_file(&asset.get_serialized_properties(), &file_path);

            let data = pack.get_asset_data(asset);
            let data = section.modify_file_on_unpack(&data);

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

        filename.starts_with(SECTION_PREFIX) && (is_dir || filename.ends_with(PROPERTIES_FILE_EXT))
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

        let section_props_data = read_property_file(section_filename.as_path()).unwrap();
        let section_props = serde_json::from_slice::<SectionProps>(&section_props_data).unwrap();
        let section_type = *section_props.get("asset_type").unwrap();

        let mut section = Section::new(section_type);

        let paths = glob::glob(
            section_filename
                .join(format!("**/*{}", PROPERTIES_FILE_EXT))
                .to_str()
                .unwrap(),
        )
        .unwrap();

        let mut j = 0;

        paths.for_each(|entry| {
            let entry = entry.unwrap();
            let entry_path = entry.to_str().unwrap();

            let u_ext = section.get_unknown_file_type_ending();
            let ext = ""; //section.get_file_name_extension(j);

            let file_path = Path::new(entry_path.strip_suffix(PROPERTIES_FILE_EXT).unwrap());

            let file_props_data = read_property_file(file_path).unwrap();
            let file_data = fs::read(file_path).unwrap();
            let relative_path = file_path.strip_prefix(&section_filename).unwrap();
            let name = unpathify(
                relative_path.to_str().unwrap().strip_suffix(ext).unwrap(),
                u_ext,
            );

            section.add_asset(&mut pack, name, &file_props_data, &file_data);

            j += 1;
        });
        pack.add_section(Box::new(section));

        index += 1;
    }
    Ok(Box::new(pack))
}

fn write_property_file(properties_data: &[u8], filename: &Path) -> Option<std::io::Error> {
    let prop_filename = append_to_path(filename, PROPERTIES_FILE_EXT);
    if !prop_filename.parent()?.exists() {
        create_dir_all(prop_filename.parent()?).unwrap();
    }

    let mut file = File::create(prop_filename).unwrap();
    file.write_all(properties_data).unwrap();
    None
}

fn read_property_file(filename: &Path) -> Result<Vec<u8>, std::io::Error> {
    let prop_filename = append_to_path(filename, PROPERTIES_FILE_EXT);
    let file = File::open(prop_filename)?;
    let mut buf = vec![];
    BufReader::new(file).read_to_end(&mut buf);
    Ok(buf)
}

// fn read_property_file<T: ?Sized + DeserializeOwned>(
//     filename: &Path,
// ) -> Result<T, serde_json::Error> {
//     let prop_filename = filename.with_extension(PROPERTIES_FILE_EXT);
//     let file = File::open(prop_filename).unwrap();
//     serde_json::from_reader(BufReader::new(file))
// }

fn pathify(name: &str, u_ext: &str) -> String {
    let filename = if !name.contains('.') {
        format!("{}{}", name, u_ext).to_string()
    } else {
        name.to_string()
    };
    filename.replace(":", "___..___")
}

fn unpathify(name: &str, u_ext: &str) -> String {
    let mut filename = name.to_string();
    filename = filename.replace("___..___", ":");
    if filename.ends_with(u_ext) {
        filename = filename[0..filename.len() - u_ext.len()].to_string();
    }
    filename
}

fn append_to_path(p: impl Into<OsString>, s: impl AsRef<OsStr>) -> PathBuf {
    let mut p = p.into();
    p.push(s);
    p.into()
}
