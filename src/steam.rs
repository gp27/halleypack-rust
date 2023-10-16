use std::env::consts::OS;
use std::path::PathBuf;

pub fn find_steam_folder() -> Option<PathBuf> {
    let mut path = PathBuf::new();

    let home = std::env::var("HOME").unwrap();

    if OS == "windows" {
        path.push(std::env::var("ProgramFiles(x86)").unwrap());
        path.push("Steam");
    } else if OS == "macos" {
        path.push(home + "/Library/Application Support/Steam");
    } else if OS == "linux" {
        path.push(home + "/.steam/steam");
    } else {
        return None;
    }

    if !path.exists() {
        return None;
    }

    path.push("steamapps");
    path.push("common");

    return Some(path);
}

pub fn find_wargroove_assets_folder(steam_folder: Option<PathBuf>) -> Option<PathBuf> {
    let mut path = steam_folder.unwrap_or(find_steam_folder().unwrap_or(PathBuf::new()));
    if !path.exists() {
        return None;
    }
    path.push("Wargroove/assets");
    if !path.exists() {
        return None;
    }
    return Some(path);
}

pub fn find_wargroove2_assets_folder(steam_folder: Option<PathBuf>) -> Option<PathBuf> {
    let mut path = steam_folder.unwrap_or(find_steam_folder().unwrap_or(PathBuf::new()));
    if !path.exists() {
        return None;
    }
    path.push("Wargroove 2/assets");
    if !path.exists() {
        return None;
    }
    return Some(path);
}
