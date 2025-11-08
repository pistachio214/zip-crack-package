use crate::module::PasswordLengthConfig;
use crate::password_module::iter::{CharsetMode, PasswordPlusGenerator};
use crate::write;
use crate::zip_module::module::PasswordCheckResult;
use chrono::prelude::*;
use colored::Colorize;
use crc32fast::Hasher;
use std::fs::File;
use std::io::{Read, stdin};
use std::process;
use std::time::Instant;
use zip::ZipArchive;

/// 尝试zip 解压
pub fn try_extract_zip(input_path: &str) {
    // 判断是否需要密码解压
    if zip_requires_password(input_path) {
        eprintln!(
            "\n{} => 目标文件 {}, 需要密码解压,请准备好设置后进行破解!\n",
            "[ 🔔  Message ]".cyan(),
            input_path.cyan()
        );
        // TODO 选择特定的模式或者默认模式

        zip_password_decompression(input_path);
    } else {
        eprintln!(
            "\n{} => 目标文件 {}, 可直接解压,不需要额外的密码!\n",
            "[ ✅  Success ]".green(),
            input_path.green()
        );
    }
}

/// zip是否需要密码
fn zip_requires_password(input_path: &str) -> bool {
    let file = match File::open(input_path) {
        Ok(file) => file,
        Err(_) => {
            eprintln!(
                "\n{} => {}",
                "[ ❌  Error ]".red(),
                "目标文件定位失败!".red(),
            );
            process::exit(0);
        }
    };

    let mut archive = match ZipArchive::new(file) {
        Ok(archive) => archive,
        Err(_) => {
            eprintln!(
                "\n{} => {}",
                "[ ❌  Error ]".red(),
                "zip打开文件失败!".red(),
            );
            process::exit(0);
        }
    };

    for i in 0..archive.len() {
        if is_entry_encrypted(&mut archive, i) {
            return true;
        }
    }
    false
}

fn is_entry_encrypted(archive: &mut ZipArchive<File>, i: usize) -> bool {
    match archive.by_index_decrypt(i, b"") {
        Err(_) => true, // AES 或 header 有 flag
        Ok(mut file) => {
            // 对 ZipCrypto：读取时报错才算加密
            let mut buf = [0u8; 1];
            file.read_exact(&mut buf).is_err()
        }
    }
}

/// 复杂密码解压
fn zip_password_decompression(input_path: &str) {
    let password_length_config = scan_min_and_max_length();

    eprintln!("\n{} => 开始智能破解密码...\n", "[ 🔑  Trying ]".green());

    eprintln!(
        "{} => {}\n",
        "[ 🔑  阶段1 ]".green(),
        "尝试简单数字组合...".green()
    );
    handle_simple_number_decompression(input_path);

    eprintln!(
        "{} => {}\n",
        "[ 🔑  阶段2 ]".green(),
        "尝试复杂数字组合...".green()
    );
    handle_complex_number_decompression(input_path, &password_length_config);

    eprintln!(
        "{} => {}\n",
        "[ 🔑  阶段3 ]".green(),
        "尝试字母加数字组合...".green()
    );
    handle_complex_number_alphabet_decompression(input_path, &password_length_config);

    eprintln!(
        "\n{} => {}\n",
        "[ 🔑  阶段4 ]".green(),
        "尝试智能暴力破解...".green()
    );
    handle_complex_decompression(input_path, &password_length_config);

    eprintln!(
        "{} => {}\n",
        "[ ✅  Success ]".green(),
        "破解流程已完成！".green()
    );
}

/// 字母加数字组合破解
fn handle_complex_number_alphabet_decompression(
    input_path: &str,
    password_length_config: &PasswordLengthConfig,
) {

    let iter = PasswordPlusGenerator::from_mode(
        CharsetMode::LettersDigits,
        password_length_config.min as usize,
        password_length_config.max as usize,
    );

    // 获取当前时间点
    let start = Instant::now();

    handle_for_iter_plus_password(input_path, iter, start);

    eprintln!(
        "\n\n{} => 字母加数字组合密码不正确！\n",
        "[ 🔔  Message ]".cyan()
    );
}

/// 复杂破解
fn handle_complex_decompression(input_path: &str, password_length_config: &PasswordLengthConfig) {
    let iter = PasswordPlusGenerator::from_mode(
        CharsetMode::AllPrintable,
        password_length_config.min as usize,
        password_length_config.max as usize,
    );

    // 获取当前时间点
    let start = Instant::now();

    handle_for_iter_plus_password(input_path, iter, start);

    eprintln!(
        "\n\n{} => 智能暴力破解密码不正确！\n",
        "[ 🔔  Message ]".cyan()
    );
}

/// 复杂数字破解
fn handle_complex_number_decompression(
    input_path: &str,
    password_length_config: &PasswordLengthConfig,
) {
    let iter = PasswordPlusGenerator::from_mode(
        CharsetMode::Digits,
        password_length_config.min as usize,
        password_length_config.max as usize,
    );

    // 获取当前时间点
    let start = Instant::now();

    handle_for_iter_plus_password(input_path, iter, start);

    eprintln!(
        "\n\n{} => 复杂数字密码破解失败！\n",
        "[ 🔔  Message ]".cyan()
    );
}

/// 简单数字破解
fn handle_simple_number_decompression(input_path: &str) {
    let current_year = Utc::now().year();
    let years: Vec<String> = (0..=current_year)
        .map(|year| format!("{:04}", year))
        .collect();

    // 获取当前时间点
    let start = Instant::now();

    let mut num = 0;

    for year in &years {
        num += 1;

        let message = format!(
            "[ 🔑  Trying ] => 正在尝试密码: {}  [ 长度: {} 位 ] [ 总尝试: {} 个数字组合 ] [ 总耗时: {:?} ]",
            &year,
            &year.len(),
            &num,
            &start.elapsed()
        );
        write(&message);

        if handle_verify_zip_password(input_path, year) {
            continue;
        } else {
            process::exit(0);
        }
    }

    eprintln!(
        "\n\n{} => 简单数字组合密码破解失败！\n",
        "[ 🔔  Message ]".cyan(),
    );
}

/// 构建一个公共循环函数
fn handle_for_iter_plus_password(
    input_path: &str,
    iter_plus: PasswordPlusGenerator,
    start: Instant,
) {
    // 重新计算个数
    let mut num = 0;

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

        if handle_verify_zip_password(input_path, &pwd) {
            continue;
        } else {
            process::exit(0);
        }
    }
}

/// 构建成一个共用方法
fn handle_verify_zip_password(input_path: &str, password: &str) -> bool {
    match verify_zip_password(input_path, &password) {
        PasswordCheckResult::Correct => {
            eprintln!(
                "\n\n{} => 破解成功！密码是: {}\n",
                "[ ✅  Success ]".green(),
                &password.green()
            );
            false
        }
        PasswordCheckResult::WrongPassword => {
            // ❌ 密码错误
            true
        }
        PasswordCheckResult::CorruptFile => {
            // ⚠️ 压缩包损坏或格式不支持
            eprintln!(
                "\n{} => {}",
                "[ ❌  Error ]".red(),
                "压缩包损坏或格式不支持".red()
            );
            false
        }
        PasswordCheckResult::IoError(e) => {
            // ⚠️ 压缩包损坏或格式不支持
            eprintln!(
                "\n{} => {} {:?}",
                "[ ❌  Error ]".red(),
                "I/O 错误:".red(),
                e
            );
            false
        }
    }
}

/// 验证密码是否正确
fn verify_zip_password(path: &str, password: &str) -> PasswordCheckResult {
    let file = match File::open(path) {
        Ok(f) => f,
        Err(e) => return PasswordCheckResult::IoError(e),
    };

    let mut archive = match ZipArchive::new(file) {
        Ok(a) => a,
        Err(_) => return PasswordCheckResult::CorruptFile,
    };

    for i in 0..archive.len() {
        let mut entry = match archive.by_index_decrypt(i, password.as_bytes()) {
            Ok(f) => f,
            Err(_) => return PasswordCheckResult::WrongPassword,
        };

        if !entry.encrypted() {
            continue;
        }

        let mut hasher = Hasher::new();
        let mut total_read: u64 = 0;
        let expected_crc = entry.crc32();
        let expected_size = entry.size();

        let mut buf = [0u8; 8192];
        loop {
            match entry.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    total_read += n as u64;
                    hasher.update(&buf[..n]);
                }
                Err(_) => return PasswordCheckResult::WrongPassword,
            }
        }

        if total_read != expected_size {
            return PasswordCheckResult::WrongPassword;
        }

        if hasher.finalize() != expected_crc {
            return PasswordCheckResult::WrongPassword;
        }
    }

    PasswordCheckResult::Correct
}

/// 密码最大长度和最小长度设定
fn scan_min_and_max_length() -> PasswordLengthConfig {
    println!("{}", "请输入最小密码长度:".green());
    let min_num;
    loop {
        let mut guess = String::new();
        stdin().read_line(&mut guess).expect("读取输入错误");

        let guess: i32 = match guess.trim().parse() {
            Ok(num) => {
                if num < 1 {
                    eprintln!(
                        "\n{} => {} {}",
                        "[ ⚠️  Warning ]".yellow(),
                        "最小长度必须大于 0 !".yellow(),
                        "重新输入最小密码长度:".green()
                    );
                    continue;
                }
                num
            }
            Err(_) => {
                eprintln!(
                    "\n{} => {} {}",
                    "[ ⚠️  Warning ]".yellow(),
                    "只能输入合法的整数!".yellow(),
                    "重新输入最小密码长度:".green()
                );
                continue;
            }
        };

        min_num = guess;
        break;
    }

    println!("{}", "请输入最大密码长度:".green());
    let max_num;
    loop {
        let mut guess = String::new();
        stdin().read_line(&mut guess).expect("读取输入错误");

        let guess: i32 = match guess.trim().parse() {
            Ok(num) => {
                if num < min_num {
                    eprintln!(
                        "\n{} => {} {}",
                        "[ ⚠️  Warning ]".yellow(),
                        "最大长度不能比最小长度小!".yellow(),
                        "重新输入最大密码长度:".green()
                    );
                    continue;
                }
                num
            }
            Err(_) => {
                eprintln!(
                    "\n{} => {} {}",
                    "[ ⚠️  Warning ]".yellow(),
                    "只能输入合法的整数!".yellow(),
                    "重新输入最大密码长度:".green()
                );
                continue;
            }
        };

        max_num = guess;
        break;
    }

    PasswordLengthConfig::new(min_num, max_num)
}
