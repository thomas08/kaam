//! ช่องทางสนทนา — Telegram และเสียง ใช้ trait เดียวกัน
#![forbid(unsafe_code)]

pub mod sentence;

use kaam_types::Source;

pub use sentence::SentenceSplitter;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Incoming {
    pub source: Source,
    pub text: String,
    /// ใช้ตรวจ allowlist — ดู ARCHITECTURE.md §13.1
    pub chat_id: i64,
}

/// ตรวจว่าผู้ส่งอยู่ใน allowlist หรือไม่
///
/// ข้อความจาก id อื่นถูกทิ้งเงียบ ๆ ไม่ตอบแม้แต่ error
/// เพราะการตอบ error ก็ยืนยันว่ามีบอทอยู่จริง
pub fn is_allowed(chat_id: i64, allowlist: &[i64]) -> bool {
    allowlist.contains(&chat_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allowlist_is_exact_match_only() {
        let allow = [12345i64];
        assert!(is_allowed(12345, &allow));
        assert!(!is_allowed(12346, &allow));
        assert!(!is_allowed(12345, &[]));
    }
}
