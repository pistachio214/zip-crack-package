mod folder_module;
mod gz_module;
mod module;
mod password_module;
mod rar_module;
mod seven_module;
mod tar_module;
mod zip_module;

use crate::folder_module::handle::{
    get_current_compression_package, select_compression, show_compression_table,
};
use crate::gz_module::handle::try_extract_gz;
use crate::rar_module::handle::try_extract_rar;
use crate::seven_module::handle::try_extract_7z;
use crate::tar_module::handle::try_extract_tar;
use crate::zip_module::handle::try_extract_zip;

use clap::{ArgMatches, Command};
use colored::Colorize;
use std::io::Write;
use std::{io, process};

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

        match file_config.extension.as_str() {
            "zip" => try_extract_zip(&file_config.path),
            "rar" => try_extract_rar(&file_config.path),
            "7z" => try_extract_7z(&file_config.path),
            "tar" => try_extract_tar(&file_config.path),
            "gz" => try_extract_gz(&file_config.path),
            &_ => {
                eprintln!(
                    "\n{} => {} 类型暂时不支持,请期待后续更新～ \n",
                    "[ ❌  Error ]".red(),
                    file_config.extension.red()
                );
            }
        }
    }
}

fn error_action() {
    eprintln!("\n{} => {} \n", "[ ❌  Error ]".red(), "非法指令".red());
    process::exit(0);
}

//清屏
fn clear_terminal() {
    print!("\x1b[2J");
    print!("\x1b[H");
}

pub fn write(message: &str) {
    let stderr = io::stderr();
    let mut handle = stderr.lock();

    write!(handle, "\r{:3}", message).unwrap();
    handle.flush().unwrap();
    // std::thread::sleep(Duration::from_millis(1));
}
