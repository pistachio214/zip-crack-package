use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileConfig {
    pub name: String,      // 文件名
    pub path: String,      // 目标地址
    pub dir_name: String,  // 解压目标文件夹
    pub size: String,      // 文件大小
    pub extension: String, // 文件类型
}

impl FileConfig {
    pub fn new(
        name: String,
        path: String,
        dir_name: String,
        size: String,
        extension: String,
    ) -> FileConfig {
        FileConfig {
            name,
            path,
            dir_name,
            size,
            extension,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PasswordLengthConfig {
    pub min: i32, // 最小长度
    pub max: i32, // 最大长度
}

impl PasswordLengthConfig {
    pub fn new(min: i32, max: i32) -> PasswordLengthConfig {
        PasswordLengthConfig { min, max }
    }
}
