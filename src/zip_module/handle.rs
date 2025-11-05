use crate::module::PasswordLengthConfig;
use crate::password_module::iter::PasswordIter;
use crate::write;
use crate::zip_module::module::ParentFixStrategy;
use chrono::prelude::*;
use colored::Colorize;
use std::fs::File;
use std::io::{Read, stdin};
use std::path::Path;
use std::{fs, io, process};
use zip::ZipArchive;
use zip::read::ZipFile;
use zip::result::ZipError;

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

/// 尝试zip 解压
pub fn try_extract_zip(input_path: &str, output_path: &str) {
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

    match fs::create_dir_all(output_path) {
        Ok(_) => {
            eprintln!(
                "\n{} => 目标目录 {},创建成功!",
                "[ ✅  Success ]".green(),
                output_path.green()
            );
        }
        Err(e) => {
            eprintln!("\n{} => {}", "[ ❌  Error ]".red(), e.to_string().red());
            process::exit(0);
        }
    }

    for i in 0..archive.len() {
        let mut mut_file = match archive.by_index(i) {
            Ok(file) => file,
            Err(ZipError::UnsupportedArchive(_)) => {
                zip_password_decompression(input_path, output_path);
                process::exit(0);
            }
            Err(e) => {
                eprintln!("\n{} => {}", "[ ❌  Error ]".red(), e.to_string().red());
                process::exit(0);
            }
        };

        handle_dir_parent(output_path, &mut mut_file);
    }

    eprintln!(
        "\n{} => 目标文件 {},解压成功!\n",
        "[ ✅  Success ]".green(),
        input_path.green()
    );
}

/// TODO 密码解压
fn zip_password_decompression(input_path: &str, output_path: &str) {
    // 重新打开压缩包,避免所有权问题(无密码模式已打开过一次,可以确定能正常打开)
    let file = File::open(input_path).unwrap();
    let mut archive = ZipArchive::new(file).unwrap();

    let password_length_config = scan_min_and_max_length();

    eprintln!("\n{} => 开始智能破解密码...\n", "[ 🔑  Trying ]".green());

    let mut num = 0;


    eprintln!(
        "{} => {}\n",
        "[ 🔑  阶段1 ]".green(),
        "加载历史密码记录...".green()
    );
    eprintln!(
        "{} => {}\n",
        "[ ✅  Success ]".green(),
        "已加载 x 个失败密码记录".green()
    );

    eprintln!(
        "{} => {}\n",
        "[ 🔑  阶段2 ]".green(),
        "尝试常用密码列表...".green()
    );
    eprintln!(
        "{} => {}\n",
        "[ ✅  Success ]".green(),
        "尝试 x 个常用密码,跳过 x 个已知密码".green()
    );

    eprintln!(
        "{} => {}\n",
        "[ 🔑  阶段3 ]".green(),
        "尝试简单数字组合...".green()
    );
    let current_year = Utc::now().year();
    let years: Vec<String> = (0..=current_year)
        .map(|year| format!("{:04}", year))
        .collect();

    for year in &years {
        let mut count = 0;
        let mut current_password = "";
        num += 1;

        for i in 0..archive.len() {
            let message = format!(
                "[ 🔑  Trying ] => 正在尝试密码: {}  [总尝试: {} 个数字组合]",
                year, num
            );
            write(&message);

            let mut mut_file = match archive.by_index_decrypt(i, year.as_bytes()) {
                Ok(mut file) => {
                    // 这里触发真正的密码验证
                    let mut buffer = Vec::new();
                    match file.read_to_end(&mut buffer) {
                        Ok(_) => {
                            count += 1;
                            current_password = year;
                            file
                        }
                        Err(_) => {
                            break;
                        }
                    }
                }
                Err(ZipError::InvalidPassword) => {
                    // 密码不正确，跳出本次循环，进入下一个密码环节
                    break;
                }
                Err(e) => {
                    eprintln!("\n{} => {:?}", "[ ❌  Error ]".red(), e);
                    break;
                }
            };

            handle_dir_parent(output_path, &mut mut_file);
        }

        if !"".eq(current_password) && count == archive.len() {
            eprintln!(
                "\n\n{} => 破解成功！密码是: {}\n",
                "[ ✅  Success ]".green(),
                current_password.green()
            );
            process::exit(0);
        }
    }

    eprintln!(
        "\n\n{} => {}\n",
        "[ 🔑  阶段4 ]".green(),
        "尝试智能暴力破解...".green()
    );
    // 重新计算个数
    num = 0;
    let iter = PasswordIter::new(
        password_length_config.min as usize,
        password_length_config.max as usize,
    );

    for pwd in iter {
        let mut count = 0;
        let mut current_password = "";
        num += 1;
        for i in 0..archive.len() {
            let message = format!(
                "[ 🔑  Trying ] => 正在尝试密码: {}  [总尝试: {} 个密码组合]",
                pwd, num
            );
            write(&message);

            let mut mut_file = match archive.by_index_decrypt(i, pwd.as_bytes()) {
                Ok(mut file) => {
                    // 这里触发真正的密码验证
                    let mut buffer = Vec::new();
                    match file.read_to_end(&mut buffer) {
                        Ok(_) => {
                            count += 1;
                            current_password = &pwd;
                            file
                        }
                        Err(_) => {
                            break;
                        }
                    }
                }
                Err(ZipError::InvalidPassword) => {
                    // 密码不正确，跳出本次循环，进入下一个密码环节
                    break;
                }
                Err(e) => {
                    eprintln!("\n{} => {:?}", "[ ❌  Error ]".red(), e);
                    break;
                }
            };

            handle_dir_parent(output_path, &mut mut_file);
        }

        if !"".eq(current_password) && count == archive.len() {
            eprintln!(
                "\n\n{} => 破解成功！密码是: {}\n",
                "[ ✅  Success ]".green(),
                current_password.green()
            );
            process::exit(0);
        }
    }

    eprintln!(
        "{} => {}\n",
        "[ ✅  Success ]".green(),
        "破解流程已完成！".green()
    );
}

fn handle_dir_parent(output_path: &str, mut_file: &mut ZipFile<File>) {
    let out_path = Path::new(output_path)
        .join(mut_file.mangled_name())
        .canonicalize()
        .ok()
        .unwrap();

    if mut_file.is_dir() {
        ensure_dir(&out_path).unwrap();
    } else {
        // 确保父目录存在
        match ensure_parent_dir(&out_path, ParentFixStrategy::BackupFile) {
            Ok(_) => {}
            Err(_) => {
                eprintln!(
                    "\n{} => {}",
                    "[ ❌  Error ]".red(),
                    "目标目录创建失败！".red()
                );
                process::exit(0);
            }
        };

        // 创建并写入文件
        let mut out_file = match File::create(&out_path) {
            Ok(file) => file,
            Err(e) => {
                eprintln!(
                    "\n{} => {}",
                    "[ ❌  Error ]".red(),
                    "目录中创建解压文件失败！".red()
                );
                eprintln!("\n{} => {:?}", "[ ❌  Error ]".red(), e);
                process::exit(0);
            }
        };
        match io::copy(mut_file, &mut out_file) {
            Ok(_) => {}
            Err(e) => {
                eprintln!(
                    "\n{} => {}",
                    "[ ❌  Error ]".red(),
                    "写入解压文件失败！".red()
                );

                eprintln!(
                    "\n{} => {:?}",
                    "[ ❌  Error ]".red(),
                    mut_file.mangled_name()
                );
                eprintln!("\n{} => {:?}", "[ ❌  Error ]".red(), e);
                process::exit(0);
            }
        }

        // 在 Unix 系统上设置文件权限
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Some(mode) = mut_file.unix_mode() {
                match fs::set_permissions(&out_path, fs::Permissions::from_mode(mode)) {
                    Ok(_) => {}
                    Err(_) => {
                        eprintln!(
                            "\n{} => {}",
                            "[ ❌  Error ]".red(),
                            "设置文件权限失败！".red()
                        );
                        process::exit(0);
                    }
                }
            }
        }
    }
}

/// 若路径存在但不是目录，则删除后再创建
fn ensure_dir(path: &Path) -> io::Result<()> {
    if path.exists() && !path.is_dir() {
        fs::remove_file(path)?;
    }
    fs::create_dir_all(path)
}

pub fn ensure_parent_dir(path: &Path, strategy: ParentFixStrategy) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        if parent.exists() {
            if parent.is_dir() {
                // 一切正常
                return Ok(());
            } else {
                // 父路径存在但不是目录 —— 典型 ENOTDIR 场景
                match strategy {
                    ParentFixStrategy::Error => Err(io::Error::new(
                        io::ErrorKind::Other,
                        format!("parent is not directory: {}", parent.display()),
                    )),
                    ParentFixStrategy::RemoveFile => {
                        // 尝试删除冲突文件，然后创建目录
                        fs::remove_file(parent).map_err(|e| {
                            io::Error::new(
                                e.kind(),
                                format!(
                                    "failed to remove conflicting file '{}': {}",
                                    parent.display(),
                                    e
                                ),
                            )
                        })?;
                        fs::create_dir_all(parent)?;
                        Ok(())
                    }
                    ParentFixStrategy::BackupFile => {
                        // 尝试将冲突文件重命名为 .bak（如果已存在 .bak，追加时间戳）
                        let mut bak = parent.with_extension("bak");
                        if bak.exists() {
                            // 追加数字或时间戳以避免覆盖
                            let mut i = 1;
                            loop {
                                let candidate = parent.with_extension(format!("bak{}", i));
                                if !candidate.exists() {
                                    bak = candidate;
                                    break;
                                }
                                i += 1;
                            }
                        }
                        fs::rename(parent, &bak).map_err(|e| {
                            io::Error::new(
                                e.kind(),
                                format!(
                                    "failed to backup conflicting file '{}': {}",
                                    parent.display(),
                                    e
                                ),
                            )
                        })?;
                        fs::create_dir_all(parent)?;
                        Ok(())
                    }
                }
            }
        } else {
            // 父目录不存在，创建它
            fs::create_dir_all(parent)?;
            Ok(())
        }
    } else {
        // 没有父路径（path 是根或类似），直接 Ok 或报错视需求
        Ok(())
    }
}
