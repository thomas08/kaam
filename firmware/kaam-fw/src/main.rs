//! Kaam firmware — จุดเชื่อมระหว่าง logic ที่ทดสอบได้กับฮาร์ดแวร์จริง
//!
//! นี่คือที่เดียวในโปรเจกต์ที่รู้จัก ESP-IDF
//! logic ทั้งหมดอยู่ใน crates/ ซึ่งทดสอบบนพีซีได้
//!
//! สถานะ: M0 — โครงเปล่า ยังไม่ได้ต่อ WiFi
//! ดู docs/ROADMAP.md ว่าอะไรมาก่อนหลัง

// hal 0.46 ถอดโมดูล `prelude` ออกแล้ว Peripherals ย้ายมาอยู่ที่ `peripherals`
use esp_idf_svc::hal::peripherals::Peripherals;
use esp_idf_svc::log::EspLogger;
use esp_idf_svc::sys::link_patches;

mod tasks;

fn main() -> anyhow::Result<()> {
    // จำเป็นสำหรับ ESP-IDF ทุกครั้ง ห้ามลบ
    link_patches();
    EspLogger::initialize_default();

    log::info!("kaam v{} เริ่มทำงาน", env!("CARGO_PKG_VERSION"));
    report_memory("บูต");

    let _peripherals = Peripherals::take()?;

    // TODO(M1): WiFi + SNTP + TLS + Telegram long-poll
    // TODO(M2): provider streaming
    // TODO(M3): LittleFS mount ที่ /kaam
    tasks::spawn_all()?;

    loop {
        std::thread::sleep(std::time::Duration::from_secs(60));
        report_memory("รายคาบ");
    }
}

/// ตัวเลขที่ต้องเฝ้าตั้งแต่วันแรก — ดู ARCHITECTURE.md ภาคผนวก B
///
/// เกณฑ์: free_internal ต้อง >= 45 KB, largest_block >= 32 KB
fn report_memory(tag: &str) {
    unsafe {
        let internal = esp_idf_svc::sys::heap_caps_get_free_size(
            esp_idf_svc::sys::MALLOC_CAP_INTERNAL,
        );
        let largest = esp_idf_svc::sys::heap_caps_get_largest_free_block(
            esp_idf_svc::sys::MALLOC_CAP_INTERNAL,
        );
        let psram = esp_idf_svc::sys::heap_caps_get_free_size(
            esp_idf_svc::sys::MALLOC_CAP_SPIRAM,
        );
        log::info!(
            "[{}] internal={} KB largest={} KB psram={} KB",
            tag,
            internal / 1024,
            largest / 1024,
            psram / 1024
        );
        if internal < 45 * 1024 {
            log::warn!("internal heap ต่ำกว่าเกณฑ์ 45 KB — ดู ARCHITECTURE.md §3.1");
        }
    }
}
