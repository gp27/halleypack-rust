use std::{
    fs,
    path::{Path, PathBuf},
};

pub fn pathify(name: &str, u_ext: &str) -> String {
    let filename = if !name.contains('.') {
        format!("{}{}", name, u_ext).to_string()
    } else {
        name.to_string()
    };
    filename.replace(":", "___..___")
}

pub fn unpathify(name: &str, u_ext: &str) -> String {
    let mut filename = name.to_string();
    filename = filename.replace("___..___", ":");
    if filename.ends_with(u_ext) {
        filename = filename[0..filename.len() - u_ext.len()].to_string();
    }
    filename
}

pub fn get_dat_files(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    for entry in fs::read_dir(dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_file() && path.extension().unwrap() == "dat" {
            files.push(path);
        }
    }
    files
}

pub fn get_dat_folders(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    for entry in fs::read_dir(dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_dir() && path.extension().unwrap() == "dat" {
            files.push(path);
        }
    }
    files
}

pub fn copy_assets(src: &Path, dst: &Path, force: Option<bool>) {
    let dat_files = get_dat_files(src);
    if !dst.exists() && !dat_files.is_empty() {
        fs::create_dir_all(dst).unwrap();
    }
    let force = force.unwrap_or(false);
    for dat_file in dat_files {
        let filename = dat_file.file_name().unwrap().to_str().unwrap();
        let dst_file = dst.join(filename);
        if force || !dst_file.exists() {
            fs::copy(dat_file, dst_file).unwrap();
        }
    }
}
