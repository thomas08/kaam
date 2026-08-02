//! รูปแบบข้อความกลาง — provider ทุกเจ้าต้องแปลงเข้า/ออกจากรูปนี้

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    System,
    User,
    Assistant,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContentBlock {
    Text(String),
    Thinking(String),
    ToolCall(ToolCall),
    ToolResult(ToolResult),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    /// argument ดิบเป็น JSON — เก็บเป็น String เพราะมันมาแบบ streaming ทีละชิ้น
    pub arguments: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolResult {
    pub call_id: String,
    pub content: String,
    pub is_error: bool,
    pub truncated: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Message {
    pub role: Role,
    pub content: Vec<ContentBlock>,
}

impl Message {
    pub fn user(text: impl Into<String>) -> Self {
        Self {
            role: Role::User,
            content: vec![ContentBlock::Text(text.into())],
        }
    }

    pub fn assistant(text: impl Into<String>) -> Self {
        Self {
            role: Role::Assistant,
            content: vec![ContentBlock::Text(text.into())],
        }
    }

    /// ประมาณจำนวน token แบบหยาบ (4 ไบต์ต่อ token)
    ///
    /// ตั้งใจให้ประเมินสูงกว่าจริงเล็กน้อย เพราะเผื่อไว้ปลอดภัยกว่าทะลุเพดาน
    pub fn approx_tokens(&self) -> usize {
        let bytes: usize = self
            .content
            .iter()
            .map(|b| match b {
                ContentBlock::Text(s) | ContentBlock::Thinking(s) => s.len(),
                ContentBlock::ToolCall(c) => c.name.len() + c.arguments.len(),
                ContentBlock::ToolResult(r) => r.content.len(),
            })
            .sum();
        bytes / 4 + 8
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StopReason {
    EndTurn,
    ToolUse,
    MaxTokens,
    Cancelled,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn approx_tokens_scales_with_length() {
        let short = Message::user("hi");
        let long = Message::user("x".repeat(400));
        assert!(long.approx_tokens() > short.approx_tokens());
        assert!(long.approx_tokens() >= 100);
    }
}
