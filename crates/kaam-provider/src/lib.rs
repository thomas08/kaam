//! ชั้น provider แบบ sans-io
//!
//! หลักการ: crate นี้ไม่รู้จัก socket, ไม่รู้จัก TLS, ไม่รู้จัก ESP-IDF
//! มันรับไบต์เข้ามาแล้วคายเหตุการณ์ออกไป เท่านั้น
//!
//! ผลคือทดสอบ parser ได้ครบทุกกรณีบนพีซี รวมถึงกรณีที่จำลองบนชิปยาก:
//! chunk ขาดกลาง UTF-8, SSE event มาครึ่งบรรทัด, การเชื่อมต่อหลุดกลาง stream
#![forbid(unsafe_code)]

pub mod sse;

use kaam_types::{StopReason, Usage};

pub use sse::{SseDecoder, SseEvent};

/// เหตุการณ์กลางที่ provider ทุกเจ้าต้องแปลงมาให้เหมือนกัน
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProviderEvent {
    TextDelta(String),
    ThinkingDelta(String),
    ToolCallStart {
        id: String,
        name: String,
    },
    ToolCallArgDelta(String),
    ToolCallEnd,
    TurnEnd {
        stop_reason: StopReason,
        usage: Usage,
    },
    Error {
        retryable: bool,
        message: String,
    },
}

/// ตัวแปลง byte stream ของ provider หนึ่งเจ้า
///
/// `feed` ต้องทนต่อ chunk ที่ขาดตรงไหนก็ได้ รวมถึงกลางอักขระ UTF-8
pub trait StreamParser {
    fn feed(&mut self, chunk: &[u8]) -> Vec<ProviderEvent>;

    /// เรียกเมื่อ stream จบ — ใช้ตรวจว่าค้างกลางทางหรือไม่
    fn finish(&mut self) -> Vec<ProviderEvent>;
}

/// provider ที่รองรับ
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderKind {
    Anthropic,
    OpenAi,
}

impl ProviderKind {
    pub fn from_model_name(model: &str) -> Option<Self> {
        if model.starts_with("claude") {
            Some(Self::Anthropic)
        } else if model.starts_with("gpt") || model.starts_with("o1") || model.starts_with("o3") {
            Some(Self::OpenAi)
        } else {
            None
        }
    }

    pub fn host(self) -> &'static str {
        match self {
            Self::Anthropic => "api.anthropic.com",
            Self::OpenAi => "api.openai.com",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_provider_from_model_name() {
        assert_eq!(
            ProviderKind::from_model_name("claude-sonnet-4-6"),
            Some(ProviderKind::Anthropic)
        );
        assert_eq!(
            ProviderKind::from_model_name("gpt-4.1"),
            Some(ProviderKind::OpenAi)
        );
        assert_eq!(ProviderKind::from_model_name("llama3"), None);
    }
}
