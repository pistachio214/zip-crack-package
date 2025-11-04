use crate::module::PasswordLengthConfig;
use colored::Colorize;
use std::fs::File;
use std::io::stdin;
use std::path::Path;
use std::{fs, io, process};
use zip::ZipArchive;
use zip::result::ZipError;

/// TODO 密码解压
fn zip_password_decompression(input_path: &str, output_path: &str) {
    eprintln!("\n执行破解方案, {} , {} \n", input_path, output_path);

    let password_config = scan_min_and_max_length();

    eprintln!("{:?}", password_config);
}

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
                        "[ ⚠️  Warning]".yellow(),
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
                    "[ ⚠️  Warning]".yellow(),
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
                        "[ ⚠️  Warning]".yellow(),
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
                    "[ ⚠️  Warning]".yellow(),
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

// zip 解压
pub fn decompress_zip(input_path: &str, output_path: &str) {
    let file = match File::open(input_path) {
        Ok(file) => file,
        Err(_) => {
            eprintln!("\n{} => {}", "[ ❌  Error]".red(), "目标文件定位失败!".red(),);
            process::exit(0);
        }
    };

    let mut archive = match ZipArchive::new(file) {
        Ok(archive) => archive,
        Err(_) => {
            eprintln!("\n{} => {}", "[ ❌  Error]".red(), "zip打开文件失败!".red(),);
            process::exit(0);
        }
    };

    match fs::create_dir_all(output_path) {
        Ok(_) => {
            eprintln!(
                "\n{} => 目标目录 {},创建成功!",
                "[ ✅  Success ]".green(),
                output_path.green()
            );
        }
        Err(e) => {
            eprintln!("\n{} => {}", "[ ❌  Error]".red(), e.to_string().red());
            process::exit(0);
        }
    }

    for i in 0..archive.len() {
        let mut file = match archive.by_index(i) {
            Ok(file) => file,
            Err(ZipError::UnsupportedArchive(_)) => {
                // 需要密码解压
                zip_password_decompression(input_path, output_path);
                process::exit(0);
            }
            Err(e) => {
                eprintln!("\n{} => {}", "[ ❌  Error]".red(), e.to_string().red());
                process::exit(0);
            }
        };

        let out_path = Path::new(output_path).join(file.mangled_name());

        // 确保父目录存在
        if let Some(parent) = out_path.parent() {
            if !parent.exists() {
                match fs::create_dir_all(parent) {
                    Ok(_) => {}
                    Err(_) => {
                        eprintln!("\n{} => {}", "[ ❌  Error]".red(), "目标目录创建失败！".red());
                        process::exit(0);
                    }
                };
            }
        }

        // 创建并写入文件
        let mut out_file = match File::create(&out_path) {
            Ok(file) => file,
            Err(_) => {
                eprintln!(
                    "\n{} => {}",
                    "[ ❌  Error]".red(),
                    "目录中创建解压文件失败！".red()
                );
                process::exit(0);
            }
        };
        match io::copy(&mut file, &mut out_file) {
            Ok(_) => {}
            Err(_) => {
                eprintln!("\n{} => {}", "[ ❌  Error]".red(), "写入解压文件失败！".red());
                process::exit(0);
            }
        }

        // 在 Unix 系统上设置文件权限
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Some(mode) = file.unix_mode() {
                match fs::set_permissions(&out_path, fs::Permissions::from_mode(mode)) {
                    Ok(_) => {}
                    Err(_) => {
                        eprintln!("\n{} => {}", "[ ❌  Error]".red(), "设置文件权限失败！".red());
                        process::exit(0);
                    }
                }
            }
        }
    }

    eprintln!(
        "\n{} => 目标文件 {},解压成功!\n",
        "[ ✅  Success ]".green(),
        input_path.green()
    );
}
