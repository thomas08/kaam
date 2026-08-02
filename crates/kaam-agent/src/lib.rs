//! แกนของ agent — state machine ของเทิร์น และการจัดการ context
//!
//! ดู ARCHITECTURE.md §9
#![forbid(unsafe_code)]

pub mod context;
pub mod turn;

pub use context::{CompactionTrigger, ContextManager};
pub use turn::TurnState;
