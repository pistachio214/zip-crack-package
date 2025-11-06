use crate::clear_terminal;
use crate::module::FileConfig;
use colored::Colorize;
use prettytable::{Table, format, row};
use std::io::stdin;
use std::path::Path;
use std::{fs, process};

fn is_target_string_case_insensitive(input: &str) -> bool {
    let lower_input = input.to_lowercase();
    matches!(lower_input.as_str(), "zip" | "rar" | "7z" | "tar" | "gz")
}

// 字节与其他单位的换算
fn format_size(bytes: u64) -> String {
    const UNITS: [&str; 6] = ["B", "KB", "MB", "GB", "TB", "PB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    format!("{:.2} {}", size, UNITS[unit_index])
}

pub fn get_current_compression_package() -> Vec<FileConfig> {
    #[cfg(debug_assertions)]
    let current_dir = match std::env::current_dir() {
        Ok(dir) => dir.join("zip_files"),
        Err(_) => {
            eprintln!("\n{} => {}\n", "[ ❌  Error ]".red(), "当前目录不合法！".red());
            process::exit(0);
        }
    };

    #[cfg(not(debug_assertions))]
    let current_dir = match std::env::current_dir() {
        Ok(dir) => dir,
        Err(_) => {
            eprintln!(
                "\n{} => {}\n",
                "[ ❌  Error ]".red(),
                "当前目录不合法！".red()
            );
            process::exit(0);
        }
    };

    let entries = match fs::read_dir(&current_dir) {
        Ok(entries) => entries,
        Err(_) => {
            eprintln!(
                "\n{} => {}\n",
                "[ ❌  Error ]".red(),
                "读取当前目录失败！".red()
            );
            process::exit(0);
        }
    };

    let mut lines: Vec<FileConfig> = Vec::new();

    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(_) => {
                continue; // 解构失败,跳过本次循环
            }
        };

        let path = entry.path();
        let metadata = match entry.metadata() {
            Ok(metadata) => metadata,
            Err(_) => {
                continue; // 解构失败,跳过本次循环
            }
        };

        let file_name = match path.file_name() {
            Some(file_name) => file_name.to_string_lossy(),
            None => continue,
        };

        if metadata.is_file() {
            let mut origin_url = current_dir.display().to_string();
            origin_url.push_str("/");
            origin_url.push_str(&file_name.to_string());

            // 获取文件扩展名
            let extension = match path.extension() {
                Some(extension) => extension.to_string_lossy(),
                None => continue,
            };

            // 判断是否为压缩文件
            if !is_target_string_case_insensitive(&extension.to_string()) {
                continue;
            }

            let size = metadata.len();

            // 将文件名去掉后缀名,用作解压时的文件夹
            let dir_name = get_clean_filename_without_extension(&origin_url);

            lines.push(FileConfig::new(
                format!("📄 {}", file_name.to_string()),
                origin_url,
                dir_name,
                format_size(size),
                extension.to_string(),
            ));
        }
    }

    lines
}

pub fn show_compression_table(lines: &Vec<FileConfig>) {
    // 创建表格
    let mut table = Table::new();
    let format = format::FormatBuilder::new()
        .column_separator('|')
        .borders('|')
        .separators(
            &[
                format::LinePosition::Top,
                format::LinePosition::Title,
                format::LinePosition::Intern,
                format::LinePosition::Bottom,
            ],
            format::LineSeparator::new('-', '+', '+', '+'),
        )
        .padding(2, 2)
        .build();

    table.set_format(format);
    // 设置标题
    table.set_titles(row!["ID", "Name", "Size", "Extension", "Origin Url"]);

    //添加行
    if !lines.is_empty() {
        for (index, line) in lines.iter().enumerate() {
            table.add_row(row![
                (index + 1),
                line.name,
                line.size,
                line.extension,
                line.path
            ]);
        }
    }

    // 清屏
    clear_terminal();

    // 打印表格到标准输出
    table.printstd();
}

// 获取不带后缀名的文件名称
pub fn get_clean_filename_without_extension(path: &str) -> String {
    let path_obj = Path::new(path);

    // 获取纯文件名（不含路径）
    let filename = match path_obj.file_name() {
        Some(name) => name.to_string_lossy().into_owned(),
        None => return path.to_string(), // 如果无法获取文件名，返回原路径
    };

    // 查找最后一个点号的位置
    if let Some(last_dot) = filename.rfind('.') {
        // 排除以下情况：
        // 1. 点号在开头（隐藏文件）
        // 2. 点号是最后一个字符
        // 3. 点号前面还是点号（特殊文件）
        if last_dot > 0 && last_dot < filename.len() - 1 {
            // 检查点号前面不是点号
            if !filename[..last_dot].ends_with('.') {
                return filename[..last_dot].to_string();
            }
        }
    }

    filename
}

pub fn select_compression(lines: &Vec<FileConfig>) -> &FileConfig {
    println!("请输入 {} 选择要解压的文件:", "序号".green());
    let num;
    loop {
        let mut guess = String::new();
        stdin().read_line(&mut guess).expect("读取输入错误");

        let guess: i32 = match guess.trim().parse() {
            Ok(num) => {
                if num < 1 {
                    eprintln!(
                        "\n{} => {} 请重新输入序号 {}:",
                        "[ ❌  Error ]".red(),
                        "序号必须大于 0 !".red(),
                        "序号".green()
                    );
                    continue;
                }

                if num > lines.len() as i32 {
                    eprintln!(
                        "\n{} => {} 请重新输入序号 {}:",
                        "[ ❌  Error ]".red(),
                        "您输入的序号超过了当前目录的文件数!".red(),
                        "序号".green()
                    );
                    continue;
                }

                num
            }
            Err(_) => {
                eprintln!(
                    "\n{} => {} 请重新输入序号 {}:",
                    "[ ❌  Error ]".red(),
                    "序号只能输入合法的整数!".red(),
                    "序号".green()
                );
                continue;
            }
        };

        num = guess;
        break;
    }

    let index = num - 1;
    let config = match lines.get(index as usize) {
        Some(config) => config,
        _ => {
            eprintln!(
                "\n{} => {}",
                "[ ❌  Error ]".red(),
                "获取压缩文件失败!".red(),
            );
            process::exit(0);
        }
    };

    config
}
