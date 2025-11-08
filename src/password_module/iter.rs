// iter.rs
// 可配置的密码枚举器：支持多种字符集模式（纯数字、纯字母、字母+数字、以及自定义）
// 提供一个简单的 Iterator `PasswordGenerator`，按长度从 min_len..=max_len 逐一枚举

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum CharsetMode {
    Digits,         // 0-9
    Lowercase,      // a-z
    Uppercase,      // A-Z
    Letters,        // a-z + A-Z
    LettersDigits,  // 0-9 + a-z + A-Z
    LowerDigits,    // 0-9 + a-z
    UpperDigits,    // 0-9 + A-Z
    Symbols,        // 常见符号: !@#$%^&*() 等
    AllPrintable,   // 所有可打印 ASCII（33..=126）包括数字/字母/符号
    Custom(Vec<char>), // 自定义字符集
}

impl CharsetMode {
    pub fn charset(&self) -> Vec<char> {
        match self {
            CharsetMode::Digits => ('0'..='9').collect(),
            CharsetMode::Lowercase => ('a'..='z').collect(),
            CharsetMode::Uppercase => ('A'..='Z').collect(),
            CharsetMode::Letters => {
                let mut v: Vec<char> = ('a'..='z').collect();
                v.extend(('A'..='Z').collect::<Vec<char>>());
                v
            }
            CharsetMode::LettersDigits => {
                let mut v: Vec<char> = ('0'..='9').collect();
                v.extend(('a'..='z').collect::<Vec<char>>());
                v.extend(('A'..='Z').collect::<Vec<char>>());
                v
            }
            CharsetMode::LowerDigits => {
                let mut v: Vec<char> = ('0'..='9').collect();
                v.extend(('a'..='z').collect::<Vec<char>>());
                v
            }
            CharsetMode::UpperDigits => {
                let mut v: Vec<char> = ('0'..='9').collect();
                v.extend(('A'..='Z').collect::<Vec<char>>());
                v
            }
            CharsetMode::Symbols => {
                // 常见符号集合（可按需扩展）
                vec![
                    '!', '@', '#', '$', '%', '^', '&', '*', '(', ')',
                    '-', '_', '+', '=', '{', '}', '[', ']', ':', ';',
                    '"', '\'', '<', '>', ',', '.', '?', '/', '\'', '|', '~', '`'
                ]
            }
            CharsetMode::AllPrintable => {
                // ASCII 可打印字符范围 33..=126
                (33u8..=126u8).map(|b| b as char).collect()
            }
            CharsetMode::Custom(c) => c.clone(),
        }
    }
}

/// PasswordGenerator 按照指定字符集和长度范围，逐一生成字符串
///
/// 使用方法：
/// ```ignore
/// let gen = PasswordGenerator::new(charset, 1, 3);/// for p in gen { println!("{}", p); }
/// ```
#[derive(Debug, Clone)]
pub struct PasswordPlusGenerator {
    charset: Vec<char>,
    max_len: usize,
    // current state uses "digits" indices into charset
    state: Option<Vec<usize>>,
    current_len: usize,
}

impl PasswordPlusGenerator {
    /// 创建一个新的生成器
    /// charset: 非空字符集
    /// min_len, max_len: 1..= ..
    pub fn new(charset: Vec<char>, min_len: usize, max_len: usize) -> Self {
        assert!(!charset.is_empty(), "charset must not be empty");
        assert!(min_len >= 1 && max_len >= min_len, "invalid min/max length");

        let state = Some(vec![0; min_len]);

        PasswordPlusGenerator {
            charset,
            max_len,
            state,
            current_len: min_len,
        }
    }

    /// 快速创建基于 CharsetMode 的生成器
    pub fn from_mode(mode: CharsetMode, min_len: usize, max_len: usize) -> Self {
        Self::new(mode.charset(), min_len, max_len)
    }
}

impl Iterator for PasswordPlusGenerator {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        let charset_len = self.charset.len();
        if charset_len == 0 {
            return None;
        }

        let state = match &mut self.state {
            Some(s) => s,
            None => return None,
        };

        // 构建当前字符串
        let s: String = state.iter().map(|&i| self.charset[i]).collect();

        // 增量：把当前 state 当作 base-N 的计数器（最低位为最后一个元素）
        let mut pos = state.len();
        loop {
            if pos == 0 {
                // 整个位都进位溢出
                if self.current_len < self.max_len {
                    // 增长长度，重置 state
                    self.current_len += 1;
                    *state = vec![0; self.current_len];
                } else {
                    // 到达最大长度且溢出 -> 结束迭代
                    self.state = None;
                }
                break;
            }

            pos -= 1; // 处理最后一位
            if state[pos] + 1 < charset_len {
                state[pos] += 1;
                break;
            } else {
                // 进位，当前位归零，继续向高位
                state[pos] = 0;
                continue;
            }
        }

        Some(s)
    }
}
