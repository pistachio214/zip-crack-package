use std::io::Write;
mod folder_module;
mod module;
mod zip_module;
mod pasword_module;

use crate::folder_module::handle::{
    get_current_compression_package, select_compression, show_compression_table,
};
use crate::zip_module::handle::try_extract_zip;
use clap::{ArgMatches, Command};
use colored::Colorize;
use std::time::Duration;
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

        // 目标目录,以文件名为文件夹
        let extract_to = format!("{}{}/", "./output/", file_config.dir_name);
        match file_config.extension.as_str() {
            "zip" => try_extract_zip(&file_config.path, &extract_to),
            &_ => {
                eprintln!(
                    "\n{} => {} 类型暂时不支持,请期待后续更新～ \n",
                    "[ ❌  Error]".red(),
                    file_config.extension.red()
                );
            }
        }
    }
}

fn error_action() {
    eprintln!("\n{} => {} \n", "[ ❌  Error]".red(), "非法指令".red());
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
