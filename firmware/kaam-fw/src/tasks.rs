//! ตาราง task — ดู ARCHITECTURE.md §5.2
//!
//! กฎที่ห้ามละเมิดบน single core:
//! - ห้าม busy-wait ทุกกรณี ต้อง block บน queue/semaphore เท่านั้น
//! - งานยาวต้องหั่นเป็นชิ้นแล้ว yield ระหว่างชิ้น
//! - มีเทิร์นที่กำลังทำงานได้ทีละหนึ่ง ข้อความที่เข้ามาระหว่างนั้นเข้าคิว

/// ความลึกของคิวขาเข้า — เต็มแล้วต้องตอบผู้ใช้ว่ากำลังยุ่ง อย่าเงียบ
pub const INBOX_DEPTH: usize = 8;

pub struct TaskSpec {
    pub name: &'static str,
    pub priority: u8,
    pub stack_bytes: usize,
}

/// ตารางนี้ต้องตรงกับ ARCHITECTURE.md §5.2 เสมอ
pub const TASKS: &[TaskSpec] = &[
    TaskSpec { name: "net_in", priority: 6, stack_bytes: 8 * 1024 },
    TaskSpec { name: "net_out", priority: 6, stack_bytes: 6 * 1024 },
    TaskSpec { name: "agent", priority: 5, stack_bytes: 12 * 1024 },
    TaskSpec { name: "sched", priority: 3, stack_bytes: 4 * 1024 },
    TaskSpec { name: "sys", priority: 2, stack_bytes: 4 * 1024 },
    #[cfg(feature = "voice")]
    TaskSpec { name: "audio_in", priority: 7, stack_bytes: 6 * 1024 },
    #[cfg(feature = "voice")]
    TaskSpec { name: "audio_out", priority: 7, stack_bytes: 6 * 1024 },
];

pub fn spawn_all() -> anyhow::Result<()> {
    // TODO(M1): สร้าง task จริงตามตารางด้านบน
    // stack ของ agent ให้ใช้ xTaskCreateStatic กับ buffer ใน PSRAM
    for t in TASKS {
        log::info!("task {} prio={} stack={} B (ยังไม่ได้สร้าง)", t.name, t.priority, t.stack_bytes);
    }
    Ok(())
}
