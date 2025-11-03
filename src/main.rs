use clap::{ArgMatches, Command};
use colored::Colorize;
use prettytable::{Table, format, row};
use serde::{Deserialize, Serialize};
use std::io::stdin;
use std::{fs, process};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileConfig {
    pub title: String,      // 文件名
    pub origin_url: String, // 目标地址
    pub size: String,       // 文件大小
    pub extension: String,  // 文件类型
}

impl FileConfig {
    fn new(title: String, origin_url: String, size: String, extension: String) -> FileConfig {
        FileConfig {
            title,
            origin_url,
            size,
            extension,
        }
    }
}

fn main() {
    // 构建命令详情
    let app = build_cli();
    // 获取命令集合
    let matches = app.get_matches();

    match matches.subcommand() {
        Some(("list", sub_matches)) => impl_compression_package_table_action(sub_matches),
        _ => error_action(),
    }
}

/**
 * 构建命令
 */
fn build_cli() -> Command {
    // 以Cargo.toml的版本为命令的版本号
    let version = env!("CARGO_PKG_VERSION");

    Command::new("aspen")
        .name("pengsy")
        .version(version)
        .author("pengsy<songyang410@outlook.com>")
        .about("解压工具箱")
        .subcommand_required(true)
        .arg_required_else_help(true)
        // 查看当前文件夹的压缩包列表
        .subcommand(build_ssh_compression_package_table_toolbox())
}

// 构建查看当前文件加的压缩文件列表命令
fn build_ssh_compression_package_table_toolbox() -> Command {
    Command::new("list").about("查看当前文件夹的压缩包列表")
}

fn impl_compression_package_table_action(_: &ArgMatches) {
    let lines = get_current_compression_package();
    show_compression_table(&lines);

    if !&lines.is_empty() {
        let file_config = select_compression(&lines);
        println!("{:?}", file_config)
    }
}

fn select_compression(lines: &Vec<FileConfig>) -> &FileConfig {
    println!("请输入 {} 选择要解压的文件:", "序号".green());
    let num;
    loop {
        let mut guess = String::new();
        stdin().read_line(&mut guess).expect("读取输入错误");

        let guess: i32 = match guess.trim().parse() {
            Ok(num) => {
                if num < 1 {
                    eprintln!(
                        "\n[Error] => {} 请重新输入序号 {}:",
                        "序号必须大于 0 !".red(),
                        "序号".green()
                    );
                    continue;
                }

                if num > lines.len() as i32 {
                    eprintln!(
                        "\n[Error] => {} 请重新输入序号 {}:",
                        "您输入的序号超过了当前目录的文件数!".red(),
                        "序号".green()
                    );
                    continue;
                }

                num
            }
            Err(_) => {
                eprintln!(
                    "\n[Error] => {} 请重新输入序号 {}:",
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
            eprintln!("\n[Error] => {}", "获取压缩文件失败!".red(),);
            process::exit(0);
        }
    };

    config
}

fn show_compression_table(lines: &Vec<FileConfig>) {
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
                line.title,
                line.size,
                line.origin_url,
                line.extension
            ]);
        }
    }

    // 清屏
    clear_terminal();

    // 打印表格到标准输出
    table.printstd();
}

fn get_current_compression_package() -> Vec<FileConfig> {
    let current_dir = match std::env::current_dir() {
        Ok(dir) => dir,
        Err(_) => {
            eprintln!("\n[Error] => {}\n", "当前目录不合法！".red());
            process::exit(0);
        }
    };

    let entries = match fs::read_dir(&current_dir) {
        Ok(entries) => entries,
        Err(_) => {
            eprintln!("\n[Error] => {}\n", "读取当前目录失败！".red());
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

            lines.push(FileConfig::new(
                format!("📄 {}", file_name.to_string()),
                extension.to_string(),
                format_size(size),
                origin_url,
            ));
        }
    }

    lines
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

fn is_target_string_case_insensitive(input: &str) -> bool {
    let lower_input = input.to_lowercase();
    matches!(lower_input.as_str(), "zip" | "rar" | "7z" | "tar" | "gz")
}

fn error_action() {
    eprintln!("\n[Error] => {} \n", "非法指令".red(),);
    process::exit(0);
}

//清屏
fn clear_terminal() {
    print!("\x1b[2J");
    print!("\x1b[H");
}
