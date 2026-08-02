//! State machine ของหนึ่งเทิร์น
//!
//! Idle → Assemble → Stream → (ToolWait → Dispatch → Stream)* → Finalize → Idle

use kaam_types::{BudgetError, StopReason};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TurnState {
    Idle,
    Assemble,
    Stream,
    ToolWait,
    Dispatch,
    Finalize(StopReason),
    Failed(TurnError),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TurnError {
    Budget(BudgetError),
    /// การเชื่อมต่อขาดกลาง stream — ไม่ retry อัตโนมัติเพราะอาจเสียเงินซ้ำ
    StreamInterrupted,
    ProviderRejected(String),
    /// ผู้ใช้แตะปุ่มเพื่อยกเลิก
    CancelledByUser,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Event {
    Started,
    PromptReady,
    ToolCallRequested,
    ToolResultReady,
    ProviderDone(StopReason),
    Cancelled,
}

impl TurnState {
    /// การเปลี่ยนสถานะ — ทางเดินที่ไม่ได้ระบุถือว่าผิดและถูกปฏิเสธ
    pub fn next(self, ev: Event) -> TurnState {
        use Event::*;
        use TurnState::*;

        // สถานะปลายทางดูดกลืนทุกเหตุการณ์ เทิร์นที่จบไปแล้วห้ามถูกเขียนทับ
        // โดยเฉพาะ Cancelled ที่มาช้ากว่าคำตอบเพียงเสี้ยววินาที
        if self.is_terminal() {
            return self;
        }

        match (self, ev) {
            (Idle, Started) => Assemble,
            (Assemble, PromptReady) => Stream,
            (Stream, ToolCallRequested) => ToolWait,
            (ToolWait, ToolResultReady) => Dispatch,
            (Dispatch, PromptReady) => Stream,
            (Stream, ProviderDone(r)) => Finalize(r),
            (_, Cancelled) => Failed(TurnError::CancelledByUser),
            (s, _) => s, // ไม่ยอมรับ transition ที่ไม่รู้จัก คงสถานะเดิม
        }
    }

    pub fn is_terminal(&self) -> bool {
        matches!(self, TurnState::Finalize(_) | TurnState::Failed(_))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn walks_the_happy_path() {
        let s = TurnState::Idle
            .next(Event::Started)
            .next(Event::PromptReady)
            .next(Event::ProviderDone(StopReason::EndTurn));
        assert_eq!(s, TurnState::Finalize(StopReason::EndTurn));
        assert!(s.is_terminal());
    }

    #[test]
    fn loops_through_tool_rounds() {
        let s = TurnState::Idle
            .next(Event::Started)
            .next(Event::PromptReady)
            .next(Event::ToolCallRequested)
            .next(Event::ToolResultReady)
            .next(Event::PromptReady);
        assert_eq!(s, TurnState::Stream);
    }

    /// ปุ่มต้องยกเลิกได้จากทุกสถานะ — ดู ARCHITECTURE.md §19.3
    #[test]
    fn cancel_works_from_any_state() {
        for s in [TurnState::Assemble, TurnState::Stream, TurnState::ToolWait] {
            assert_eq!(
                s.next(Event::Cancelled),
                TurnState::Failed(TurnError::CancelledByUser)
            );
        }
    }

    /// ปุ่มยกเลิกที่กดช้ากว่าคำตอบต้องไม่เปลี่ยนเทิร์นที่สำเร็จให้กลายเป็นล้มเหลว
    #[test]
    fn cancel_cannot_undo_a_finished_turn() {
        let done = TurnState::Idle
            .next(Event::Started)
            .next(Event::PromptReady)
            .next(Event::ProviderDone(StopReason::EndTurn));
        assert_eq!(
            done.clone().next(Event::Cancelled),
            TurnState::Finalize(StopReason::EndTurn)
        );
        assert_eq!(done.clone().next(Event::Started), done);
    }

    #[test]
    fn ignores_impossible_transitions() {
        assert_eq!(
            TurnState::Idle.next(Event::ToolResultReady),
            TurnState::Idle
        );
    }
}
