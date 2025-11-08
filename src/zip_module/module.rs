#[allow(dead_code)]
#[derive(Debug)]
pub enum PasswordCheckResult {
    Correct,
    WrongPassword,
    CorruptFile,
    IoError(std::io::Error),
}