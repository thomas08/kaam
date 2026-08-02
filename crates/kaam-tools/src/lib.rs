//! ทะเบียน tool และนโยบายการอนุญาต
//!
//! ดู ARCHITECTURE.md §10
#![forbid(unsafe_code)]

pub mod soul;

use std::collections::BTreeMap;

/// สถานะการอนุญาตของ tool หนึ่งตัว
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Policy {
    Allow,
    /// ถามผู้ใช้ก่อน รอไม่เกิน 5 นาที ไม่ตอบถือว่าปฏิเสธ
    Confirm,
    Deny,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolSpec {
    pub name: String,
    pub description: String,
    /// JSON schema ของ argument
    pub schema: String,
    pub default_policy: Policy,
}

#[derive(Debug, Default)]
pub struct Registry {
    tools: BTreeMap<String, ToolSpec>,
    overrides: BTreeMap<String, Policy>,
}

impl Registry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, spec: ToolSpec) {
        self.tools.insert(spec.name.clone(), spec);
    }

    /// ผูก tool จาก MCP server เข้าทะเบียนเดียวกัน โดยตั้งชื่อแบบ namespace
    /// เพื่อไม่ให้ชนกับ built-in
    pub fn register_mcp(&mut self, server: &str, mut spec: ToolSpec) {
        spec.name = format!("mcp__{}__{}", server, spec.name);
        self.register(spec);
    }

    pub fn set_policy(&mut self, name: &str, policy: Policy) {
        self.overrides.insert(name.to_string(), policy);
    }

    pub fn policy_for(&self, name: &str) -> Policy {
        // propose_soul_edit ต้องอนุมัติเสมอ ปิดไม่ได้ ไม่ว่าตั้งค่าอย่างไร
        // ดู ARCHITECTURE.md §18
        if name == soul::TOOL_NAME {
            return Policy::Confirm;
        }
        self.overrides
            .get(name)
            .copied()
            .or_else(|| self.tools.get(name).map(|t| t.default_policy))
            .unwrap_or(Policy::Deny)
    }

    pub fn names(&self) -> Vec<&str> {
        self.tools.keys().map(|s| s.as_str()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spec(name: &str, p: Policy) -> ToolSpec {
        ToolSpec {
            name: name.into(),
            description: "d".into(),
            schema: "{}".into(),
            default_policy: p,
        }
    }

    #[test]
    fn unknown_tools_are_denied_by_default() {
        let r = Registry::new();
        assert_eq!(r.policy_for("anything"), Policy::Deny);
    }

    #[test]
    fn overrides_take_precedence() {
        let mut r = Registry::new();
        r.register(spec("http_get", Policy::Confirm));
        r.set_policy("http_get", Policy::Allow);
        assert_eq!(r.policy_for("http_get"), Policy::Allow);
    }

    #[test]
    fn mcp_tools_are_namespaced() {
        let mut r = Registry::new();
        r.register_mcp("github", spec("create_issue", Policy::Confirm));
        assert_eq!(r.names(), vec!["mcp__github__create_issue"]);
    }

    /// การอนุมัติแก้ SOUL.md ปิดไม่ได้แม้จะพยายาม override
    #[test]
    fn soul_edit_confirmation_cannot_be_disabled() {
        let mut r = Registry::new();
        r.register(spec(soul::TOOL_NAME, Policy::Confirm));
        r.set_policy(soul::TOOL_NAME, Policy::Allow);
        assert_eq!(r.policy_for(soul::TOOL_NAME), Policy::Confirm);
    }
}
