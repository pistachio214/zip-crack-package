use crate::password_module::iter::CharsetMode;

#[derive(Debug, Clone)]
pub struct CharsetModeItem {
    pub charset: CharsetMode,
    pub explain: String,
}

impl CharsetModeItem {
    pub fn new(charset: CharsetMode, explain: String) -> CharsetModeItem {
        CharsetModeItem { charset, explain }
    }
}
