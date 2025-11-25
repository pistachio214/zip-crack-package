use crate::password_module::iter::PasswordPlusGenerator;
use crate::password_module::module::CharsetModeItem;
use crate::write;
use colored::Colorize;
use std::process;
use std::time::Instant;
use unrar::error::UnrarError;
use unrar::{Archive, UnrarResult};

/// TODO rar格式解压
pub fn try_extract_rar(input_path: &str) {
    if is_password_required(&input_path) {
        eprintln!(
            "\n{} => 目标文件 {}\n{}\n",
            "[ 🔔  Message ]".cyan(),
            input_path.cyan(),
            "需要密码解压,请选择模式进行后续!".cyan()
        );

        let crack_module = crate::zip_module::handle::select_crack_module();
        handle_rar_crack(crack_module, &input_path);
    } else {
        eprintln!(
            "\n{} => 目标文件 {}, 可直接解压,不需要额外的密码!\n",
            "[ ✅  Success ]".green(),
            input_path.green()
        );
    }
}

fn handle_rar_crack(crack_module: CharsetModeItem, input_path: &str) {
    let password_length_config = crate::zip_module::handle::scan_min_and_max_length();

    eprintln!("\n{} => 开始智能破解密码...\n", "[ 🔑  Trying ]".green());

    eprintln!(
        "{} => {}\n",
        "[ 🔑  Trying ]".green(),
        "开始尝试密码破解...".green()
    );

    let iter = PasswordPlusGenerator::from_mode(
        crack_module.charset,
        password_length_config.min as usize,
        password_length_config.max as usize,
    );

    // 获取当前时间点
    let start = Instant::now();

    handle_for_iter_plus_password(input_path, iter, start);

    eprintln!(
        "{} => {}\n",
        "[ ✅  Success ]".green(),
        "破解流程已完成！".green()
    );
}

/// 构建一个公共循环函数
fn handle_for_iter_plus_password(
    input_path: &str,
    iter_plus: PasswordPlusGenerator,
    start: Instant,
) {
    // 重新计算个数
    let mut num: i64 = 0;

    for pwd in iter_plus {
        num += 1;

        let message = format!(
            "[ 🔑  Trying ] => 正在尝试密码: {} [ 长度: {} 位 ] [ 总尝试: {} 个密码组合 ] [ 总耗时: {:?} ]",
            &pwd,
            &pwd.len(),
            &num,
            &start.elapsed()
        );
        write(&message);

        let archive = Archive::with_password(input_path, &pwd).open_for_listing();

        match archive {
            Ok(listing) => {
                let read = listing.read_header().is_ok();
                if read {
                    eprintln!(
                        "\n\n{} => 破解成功！密码是: {}\n",
                        "[ ✅  Success ]".green(),
                        &pwd.green()
                    );
                    process::exit(0);
                } else {
                    // 密码错误
                    continue;
                }
            }
            Err(_) => {
                // 如果文件本身损坏，直接停止
                process::exit(0);
            }
        }
    }
}

fn is_password_required(file_path: &str) -> bool {
    let list = Archive::new(file_path).open_for_listing();

    match list {
        Ok(archive) => {
            // 尝试读取第一个文件头
            match archive.read_header() {
                Ok(_) => false, // 读取成功 → 不需要密码
                Err(e) => true,
            }
        }
        Err(_) => false,
    }
}
