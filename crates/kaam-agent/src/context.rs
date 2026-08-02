//! การจัดการ context และ compaction
//!
//! กลไกนี้คือสิ่งที่ทำให้เป้าหมาย "อยู่ได้ 30 วันโดยไม่ต้องรีบูต" เป็นจริง
//! ถ้าไม่มี เครื่องจะพังภายในไม่กี่วัน
//!
//! ดู ARCHITECTURE.md §9.4

use kaam_types::Message;

/// เหตุที่ทำให้ต้อง compact
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompactionTrigger {
    /// token เกิน 70% ของเพดาน
    TokenPressure,
    /// ข้อความเกิน 40 ข้อความ
    MessageCount,
    /// SRAM ภายในเหลือน้อยกว่า 45 KB
    LowMemory,
}

pub struct ContextManager {
    pub max_tokens: usize,
    pub max_messages: usize,
    pub min_free_internal_bytes: usize,
}

impl Default for ContextManager {
    fn default() -> Self {
        Self {
            max_tokens: 60_000,
            max_messages: 40,
            min_free_internal_bytes: 45 * 1024,
        }
    }
}

impl ContextManager {
    /// ตรวจว่าถึงเวลา compact หรือยัง
    ///
    /// `free_internal` มาจาก `esp_get_free_internal_heap_size()` บนเครื่องจริง
    /// ในเทสต์ส่งค่าจำลองเข้ามาได้ตรง ๆ — นี่คือประโยชน์ของ sans-io
    pub fn should_compact(
        &self,
        msgs: &[Message],
        free_internal: usize,
    ) -> Option<CompactionTrigger> {
        if free_internal < self.min_free_internal_bytes {
            return Some(CompactionTrigger::LowMemory);
        }
        if msgs.len() > self.max_messages {
            return Some(CompactionTrigger::MessageCount);
        }
        let tokens: usize = msgs.iter().map(|m| m.approx_tokens()).sum();
        if tokens * 10 > self.max_tokens * 7 {
            return Some(CompactionTrigger::TokenPressure);
        }
        None
    }

    /// เลือกว่าข้อความไหนจะถูกยุบเป็นบทสรุป
    ///
    /// เอาครึ่งแรกไปสรุป เก็บครึ่งหลังไว้เต็ม ๆ
    pub fn split_for_compaction<'a>(&self, msgs: &'a [Message]) -> (&'a [Message], &'a [Message]) {
        msgs.split_at(msgs.len() / 2)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn msgs(n: usize, size: usize) -> Vec<Message> {
        (0..n).map(|_| Message::user("x".repeat(size))).collect()
    }

    const PLENTY: usize = 100 * 1024;

    #[test]
    fn quiet_when_everything_is_fine() {
        let cm = ContextManager::default();
        assert_eq!(cm.should_compact(&msgs(5, 100), PLENTY), None);
    }

    /// หน่วยความจำต่ำต้องชนะทุกเงื่อนไข เพราะเป็นเรื่องที่ทำให้เครื่องตายทันที
    #[test]
    fn low_memory_wins_over_everything() {
        let cm = ContextManager::default();
        assert_eq!(
            cm.should_compact(&msgs(1, 10), 30 * 1024),
            Some(CompactionTrigger::LowMemory)
        );
    }

    #[test]
    fn triggers_on_message_count() {
        let cm = ContextManager::default();
        assert_eq!(
            cm.should_compact(&msgs(41, 10), PLENTY),
            Some(CompactionTrigger::MessageCount)
        );
    }

    #[test]
    fn triggers_on_token_pressure() {
        let cm = ContextManager::default();
        // 20 ข้อความ ข้อความละ ~2500 token = 50k ซึ่งเกิน 70% ของ 60k
        assert_eq!(
            cm.should_compact(&msgs(20, 10_000), PLENTY),
            Some(CompactionTrigger::TokenPressure)
        );
    }

    #[test]
    fn keeps_recent_half_intact() {
        let cm = ContextManager::default();
        let m = msgs(10, 10);
        let (old, recent) = cm.split_for_compaction(&m);
        assert_eq!(old.len(), 5);
        assert_eq!(recent.len(), 5);
    }
}
