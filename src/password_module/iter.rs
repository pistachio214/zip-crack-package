use std::iter::Iterator;

pub struct PasswordIter {
    charset: &'static [u8],
    base: usize,
    min_len: usize,
    max_len: usize,
    indices: Vec<usize>,
    started: bool,
}

impl PasswordIter {
    pub fn new(min_len: usize, max_len: usize) -> Self {
        let charset: &'static [u8] =
            b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789!@#$%^&*()";
        let base = charset.len();

        PasswordIter {
            charset,
            base,
            min_len,
            max_len,
            indices: Vec::new(),
            started: false,
        }
    }
}

impl Iterator for PasswordIter {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.started {
            self.started = true;
            if self.min_len == 0 {
                return Some(String::new());
            }
            // 初始化长度为 min_len
            self.indices = vec![0; self.min_len];
            // 返回第一个密码
            return Some(
                self.indices
                    .iter()
                    .map(|&i| self.charset[i] as char)
                    .collect(),
            );
        }

        loop {
            if self.indices.is_empty() {
                return None;
            }

            // 进位逻辑
            let mut pos = self.indices.len() - 1;
            loop {
                if self.indices[pos] + 1 < self.base {
                    self.indices[pos] += 1;
                    break;
                } else {
                    self.indices[pos] = 0;
                    if pos == 0 {
                        // 当前长度已穷尽
                        if self.indices.len() < self.max_len {
                            // 进入下一长度（初始化为全零）
                            self.indices = vec![0; self.indices.len() + 1];
                            break;
                        } else {
                            // 所有长度都走完
                            self.indices.clear();
                            return None;
                        }
                    }
                    pos -= 1;
                }
            }

            return Some(
                self.indices
                    .iter()
                    .map(|&i| self.charset[i] as char)
                    .collect(),
            );
        }
    }
}
