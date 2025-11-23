use unrar::Archive;

/// TODO rar格式解压
pub fn try_extract_rar(input_path: &str) {

}

fn is_password_required(file_path: &str) -> bool {
    let archive = Archive::new(file_path);

    false
}
