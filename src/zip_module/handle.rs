use crate::folder_module::handle::table_format_builder;
use crate::module::PasswordLengthConfig;
use crate::password_module::iter::{CharsetMode, PasswordPlusGenerator};
use crate::password_module::module::CharsetModeItem;
use crate::write;
use crate::zip_module::module::PasswordCheckResult;
use colored::Colorize;
use crc32fast::Hasher;
use prettytable::{Table, row};
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
            "\n{} => 目标文件 {}\n{}\n",
            "[ 🔔  Message ]".cyan(),
            input_path.cyan(),
            "需要密码解压,请选择模式进行后续!".cyan()
        );
        let crack_module = select_crack_module();
        handle_zip_crack(crack_module, input_path);
    } else {
        eprintln!(
            "\n{} => 目标文件 {}, 可直接解压,不需要额外的密码!\n",
            "[ ✅  Success ]".green(),
            input_path.green()
        );
    }
}

fn handle_zip_crack(crack_module: CharsetModeItem, input_path: &str) {
    let password_length_config = scan_min_and_max_length();

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
        "\n\n{} => 字母加数字组合密码不正确！\n",
        "[ 🔔  Message ]".cyan()
    );

    eprintln!(
        "{} => {}\n",
        "[ ✅  Success ]".green(),
        "破解流程已完成！".green()
    );
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

/// 选择特定的模式或者默认模式
fn select_crack_module() -> CharsetModeItem {
    // 创建表格
    let mut table = Table::new();
    let format = table_format_builder();

    table.set_format(format);
    // 设置标题
    table.set_titles(row!["ID", "Name", "explain"]);

    let lines = vec![
        CharsetModeItem::new(CharsetMode::Digits, "0 - 9".to_string()),
        CharsetModeItem::new(CharsetMode::Lowercase, "a - z".to_string()),
        CharsetModeItem::new(CharsetMode::Uppercase, "A - Z".to_string()),
        CharsetModeItem::new(CharsetMode::Letters, "a -z + A - Z".to_string()),
        CharsetModeItem::new(
            CharsetMode::LettersDigits,
            "0 - 9 + a -z + A - Z".to_string(),
        ),
        CharsetModeItem::new(CharsetMode::LowerDigits, "0 - 9 + a -z".to_string()),
        CharsetModeItem::new(CharsetMode::UpperDigits, "0 - 9 + A - Z".to_string()),
        CharsetModeItem::new(CharsetMode::Symbols, "!@#$%^&*() 等".to_string()),
        CharsetModeItem::new(
            CharsetMode::AllPrintable,
            "所有可打印ASCII（33..=126）包括数字/字母/符号".to_string(),
        ),
    ];

    for (index, line) in lines.iter().enumerate() {
        table.add_row(row![
            (index + 1),
            format!("模式 {}", index + 1),
            line.explain,
        ]);
    }

    // 打印表格到标准输出
    table.printstd();

    println!("请输入 {} 选择对应的破解模式(默认全模式):", "序号".green());
    let num;
    loop {
        let mut guess = String::new();
        stdin().read_line(&mut guess).expect("读取输入错误");

        if guess.trim().is_empty() {
            return match lines.last() {
                Some(value) => value.clone(),
                None => {
                    eprintln!("\n{} => {}:", "[ ❌  Error ]".red(), "向量为空!".red());
                    process::exit(0);
                }
            };
        }

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
                        "非法模式!".red(),
                        "序号".green()
                    );
                    continue;
                }
                num
            }
            Err(e) => {
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

    match lines.get(index as usize) {
        Some(config) => config.clone(),
        _ => {
            eprintln!("\n{} => {}:", "[ ❌  Error ]".red(), "向量为空!".red());
            process::exit(0);
        }
    }
}
