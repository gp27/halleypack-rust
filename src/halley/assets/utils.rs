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
