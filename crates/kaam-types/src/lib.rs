//! ชนิดข้อมูลกลางของ Kaam
//!
//! กฎ: crate นี้ห้าม depend อะไรที่แตะ IO ทั้งสิ้น
#![forbid(unsafe_code)]

pub mod budget;
pub mod message;

pub use budget::{Budget, BudgetError, BudgetTracker, Usage};
pub use message::{ContentBlock, Message, Role, StopReason, ToolCall, ToolResult};

/// ที่มาของข้อความขาเข้า — ทั้งสามทางเข้า `inbox_q` เดียวกันและเดินผ่าน code path เดียวกัน
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    Telegram,
    Voice,
    Scheduler,
}

/// ปลายทางของคำตอบ
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sink {
    Text,
    /// อ่านออกลำโพง — ต้องตัดตามขอบประโยคก่อนส่งเข้า TTS
    Speech,
}

impl Source {
    pub fn default_sink(self) -> Sink {
        match self {
            Source::Voice => Sink::Speech,
            _ => Sink::Text,
        }
    }
}
