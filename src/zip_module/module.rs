/// 冲突处理策略
#[derive(Clone, Copy)]
#[allow(dead_code)]
pub enum ParentFixStrategy {
    /// 报错返回 Err（默认保守策略）
    Error,
    /// 删除冲突的文件（危险，会丢失原文件）
    RemoveFile,
    /// 把冲突的文件重命名为 "<name>.bak"（更安全）
    BackupFile,
}
