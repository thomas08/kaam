//! MCP client — รองรับเฉพาะ HTTP + SSE
//!
//! stdio transport ทำไม่ได้บน MCU เพราะไม่มี process ให้ spawn
//!
//! ดู ARCHITECTURE.md §10.2
#![forbid(unsafe_code)]

/// server ที่ล้มติดกันเกินจำนวนนี้จะถูกปิดชั่วคราว
pub const FAILURE_THRESHOLD: u8 = 3;
/// ระยะเวลาที่ปิด (วินาที)
pub const COOLDOWN_SECONDS: u32 = 600;
/// timeout ต่อการเรียกหนึ่งครั้ง — server ที่ตายห้ามทำให้ทั้งเครื่องค้าง
pub const CALL_TIMEOUT_SECONDS: u32 = 15;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BreakerState {
    Closed,
    Open { until_uptime: u32 },
}

/// ตัวตัดวงจรต่อ server หนึ่งตัว
#[derive(Debug, Clone, Copy)]
pub struct CircuitBreaker {
    failures: u8,
    state: BreakerState,
}

impl Default for CircuitBreaker {
    fn default() -> Self {
        Self {
            failures: 0,
            state: BreakerState::Closed,
        }
    }
}

impl CircuitBreaker {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn on_success(&mut self) {
        self.failures = 0;
        self.state = BreakerState::Closed;
    }

    pub fn on_failure(&mut self, now: u32) {
        self.failures = self.failures.saturating_add(1);
        if self.failures >= FAILURE_THRESHOLD {
            self.state = BreakerState::Open {
                until_uptime: now + COOLDOWN_SECONDS,
            };
        }
    }

    pub fn is_available(&self, now: u32) -> bool {
        match self.state {
            BreakerState::Closed => true,
            BreakerState::Open { until_uptime } => now >= until_uptime,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opens_after_three_consecutive_failures() {
        let mut b = CircuitBreaker::new();
        b.on_failure(0);
        b.on_failure(1);
        assert!(b.is_available(2));
        b.on_failure(2);
        assert!(!b.is_available(3), "ต้องปิดหลังล้ม 3 ครั้ง");
    }

    #[test]
    fn reopens_after_cooldown() {
        let mut b = CircuitBreaker::new();
        for i in 0..3 {
            b.on_failure(i);
        }
        assert!(!b.is_available(100));
        assert!(b.is_available(2 + COOLDOWN_SECONDS));
    }

    #[test]
    fn success_resets_the_counter() {
        let mut b = CircuitBreaker::new();
        b.on_failure(0);
        b.on_failure(1);
        b.on_success();
        b.on_failure(2);
        assert!(b.is_available(3), "สำเร็จหนึ่งครั้งต้องล้างประวัติ");
    }
}
