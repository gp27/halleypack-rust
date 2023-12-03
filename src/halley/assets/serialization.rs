use anyhow::anyhow;
use serde::{de::DeserializeOwned, Serialize};
use std::path::Path;

#[derive(Copy, Clone)]
pub enum Format {
    Json5,
    Toml5,
    Yaml,
}

static SERIALIZATION_FORMAT: Format = Format::Json5;

pub fn serialize<T: Serialize>(t: &T, f: Option<Format>) -> Result<String, anyhow::Error> {
    match f.unwrap_or(SERIALIZATION_FORMAT) {
        Format::Json5 => {
            let json = json5::to_string(t)?;
            jsonxf::pretty_print(&json).map_err(|e| anyhow!(e))
        }
        Format::Toml5 => toml::to_string(t).map_err(|e| e.into()),
        Format::Yaml => serde_yaml::to_string(t).map_err(|e| e.into()),
    }
}

pub fn deserialize<T: DeserializeOwned>(i: &str, f: Option<Format>) -> Result<T, anyhow::Error> {
    match f.unwrap_or(SERIALIZATION_FORMAT) {
        Format::Json5 => json5::from_str(i).map_err(|e| e.into()),
        Format::Toml5 => toml::from_str(i).map_err(|e| e.into()),
        Format::Yaml => serde_yaml::from_str(i).map_err(|e| e.into()),
    }
}

pub fn get_serialization_ext(f: Option<Format>) -> &'static str {
    match f.unwrap_or(SERIALIZATION_FORMAT) {
        Format::Json5 => "json5",
        Format::Toml5 => "toml",
        Format::Yaml => "yaml",
    }
}

pub fn get_format_from_ext(ext: &str) -> Option<Format> {
    match ext {
        "json5" => Some(Format::Json5),
        "toml" => Some(Format::Toml5),
        "yaml" => Some(Format::Yaml),
        "yml" => Some(Format::Yaml),
        _ => None,
    }
}

pub fn get_serialization_ext_from_path(path: &Path) -> &'static str {
    let ext = path
        .extension()
        .unwrap_or_default()
        .to_str()
        .unwrap_or_default();
    match ext {
        "json5" => "json5",
        "toml" => "toml",
        "yaml" => "yaml",
        "yml" => "yaml",
        _ => "",
    }
}
