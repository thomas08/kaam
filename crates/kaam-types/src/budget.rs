//! งบประมาณของเทิร์น — บังคับใช้ในโค้ด ไม่ใช่แค่เขียนในเอกสาร
//!
//! ดู docs/ARCHITECTURE.md §9.2

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Usage {
    pub input_tokens: u32,
    pub output_tokens: u32,
}

impl Usage {
    pub fn total(&self) -> u32 {
        self.input_tokens.saturating_add(self.output_tokens)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Budget {
    pub max_tool_rounds: u8,
    pub max_turn_seconds: u16,
    pub max_turn_tokens: u32,
    pub max_daily_tokens: u32,
    pub max_tool_result_bytes: usize,
}

impl Default for Budget {
    fn default() -> Self {
        Self {
            max_tool_rounds: 12,
            max_turn_seconds: 180,
            max_turn_tokens: 60_000,
            max_daily_tokens: 500_000,
            max_tool_result_bytes: 8 * 1024,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BudgetError {
    ToolRoundsExceeded,
    TurnTimeout,
    TurnTokensExceeded,
    DailyTokensExceeded,
}

/// ตัวนับที่เดินไปพร้อมกับเทิร์นหนึ่งเทิร์น
#[derive(Debug, Clone, Copy, Default)]
pub struct BudgetTracker {
    pub tool_rounds: u8,
    pub elapsed_seconds: u16,
    pub turn_tokens: u32,
    pub daily_tokens: u32,
}

impl BudgetTracker {
    /// ตรวจก่อนเริ่มรอบใหม่ — เรียกทุกครั้งก่อนยิง request
    pub fn check(&self, b: &Budget) -> Result<(), BudgetError> {
        if self.tool_rounds >= b.max_tool_rounds {
            return Err(BudgetError::ToolRoundsExceeded);
        }
        if self.elapsed_seconds >= b.max_turn_seconds {
            return Err(BudgetError::TurnTimeout);
        }
        if self.turn_tokens >= b.max_turn_tokens {
            return Err(BudgetError::TurnTokensExceeded);
        }
        if self.daily_tokens >= b.max_daily_tokens {
            return Err(BudgetError::DailyTokensExceeded);
        }
        Ok(())
    }

    /// บวกแบบอิ่มตัว — ถ้าล้นแล้ว wrap ตัวนับจะย้อนกลับไปศูนย์และเพดานรายวันหลุด
    /// ค้างที่ `u32::MAX` แทน ทำให้ `check` ปฏิเสธเทิร์นใหม่ ซึ่งเป็นฝั่งที่ปลอดภัยกว่า
    pub fn record(&mut self, usage: Usage) {
        let total = usage.total();
        self.turn_tokens = self.turn_tokens.saturating_add(total);
        self.daily_tokens = self.daily_tokens.saturating_add(total);
    }
}

/// ตัดผลลัพธ์ tool ที่ยาวเกินงบ พร้อมหมายเหตุกำกับ
///
/// ตัดที่ขอบ char เสมอ ไม่งั้น UTF-8 พัง — ซึ่งกับภาษาไทยเกิดง่ายมาก
pub fn truncate_tool_result(s: &str, max_bytes: usize) -> (String, bool) {
    if s.len() <= max_bytes {
        return (s.to_string(), false);
    }
    const NOTE: &str = "\n\n[ถูกตัดเพราะเกินงบผลลัพธ์]";
    // ถ้างบเล็กกว่าหมายเหตุ ใส่หมายเหตุไม่ได้เลย — ตัดดิบ ๆ ดีกว่าคืนค่าเกินงบ
    let (room, note) = if max_bytes >= NOTE.len() {
        (max_bytes - NOTE.len(), NOTE)
    } else {
        (max_bytes, "")
    };
    let mut end = 0;
    for (i, _) in s.char_indices() {
        if i > room {
            break;
        }
        end = i;
    }
    (format!("{}{}", &s[..end], note), true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_when_tool_rounds_exhausted() {
        let b = Budget::default();
        let t = BudgetTracker {
            tool_rounds: 12,
            ..Default::default()
        };
        assert_eq!(t.check(&b), Err(BudgetError::ToolRoundsExceeded));
    }

    #[test]
    fn daily_cap_survives_across_turns() {
        let b = Budget {
            max_daily_tokens: 100,
            ..Budget::default()
        };
        let mut t = BudgetTracker::default();
        t.record(Usage {
            input_tokens: 60,
            output_tokens: 50,
        });
        assert_eq!(t.check(&b), Err(BudgetError::DailyTokensExceeded));
    }

    #[test]
    fn truncation_never_splits_utf8() {
        let thai = "ก้ามปูทะเล".repeat(200);
        let (out, truncated) = truncate_tool_result(&thai, 512);
        assert!(truncated);
        assert!(out.len() <= 512);
        assert!(std::str::from_utf8(out.as_bytes()).is_ok());
    }

    #[test]
    fn short_results_pass_through_untouched() {
        let (out, truncated) = truncate_tool_result("สั้น", 512);
        assert_eq!(out, "สั้น");
        assert!(!truncated);
    }

    /// งบเล็กกว่าข้อความหมายเหตุก็ยังห้ามคืนค่าเกินงบ — ค่านี้ตั้งได้ใน settings.json
    #[test]
    fn never_exceeds_the_budget_even_when_tiny() {
        let thai = "ก้ามปูทะเล".repeat(200);
        for max_bytes in [0usize, 8, 32, 64, 75, 76, 512] {
            let (out, truncated) = truncate_tool_result(&thai, max_bytes);
            assert!(truncated);
            assert!(
                out.len() <= max_bytes,
                "max_bytes={max_bytes} แต่คืน {} ไบต์",
                out.len()
            );
        }
    }

    /// ตัวนับ token ห้าม wrap เพราะจะทำให้เพดานรายวันหลุดเงียบ ๆ
    #[test]
    fn token_counters_saturate_instead_of_wrapping() {
        let mut t = BudgetTracker {
            daily_tokens: u32::MAX - 1,
            ..Default::default()
        };
        t.record(Usage {
            input_tokens: 100,
            output_tokens: 100,
        });
        assert_eq!(t.daily_tokens, u32::MAX);
        assert_eq!(
            t.check(&Budget::default()),
            Err(BudgetError::DailyTokensExceeded)
        );
    }
}
