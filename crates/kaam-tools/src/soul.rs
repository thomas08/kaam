//! การแก้ SOUL.md แบบมีคนอยู่ในลูป
//!
//! ดู ARCHITECTURE.md §18 — กลไกนี้ต้องกันสองเรื่อง:
//! 1. agent เขียนตัวเองออกจากข้อจำกัด → บล็อก immutable
//! 2. บุคลิกเลื่อนไหลทีละนิดจนคุมไม่ได้ → แสดงระยะห่างสะสมจาก baseline

pub const TOOL_NAME: &str = "propose_soul_edit";

pub const IMMUTABLE_BEGIN: &str = "<!-- kaam:immutable:begin -->";
pub const IMMUTABLE_END: &str = "<!-- kaam:immutable:end -->";

/// เพดานการแก้ต่อครั้ง คิดเป็นสัดส่วนของบรรทัดทั้งไฟล์
pub const MAX_CHANGE_RATIO: f32 = 0.20;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SoulEditError {
    /// พยายามแก้บล็อกที่ห้ามแตะ
    ImmutableBlockModified,
    /// บล็อก immutable หายไปจากไฟล์ใหม่
    ImmutableBlockMissing,
    /// แก้เกิน 20% ของไฟล์ในครั้งเดียว
    TooLarge,
    /// มี proposal ค้างอยู่แล้ว
    ProposalPending,
    /// เกินโควตาต่อวันหรือต่อเดือน
    RateLimited,
}

/// ดึงเนื้อหาในบล็อก immutable ออกมา
pub fn extract_immutable(content: &str) -> Option<&str> {
    let start = content.find(IMMUTABLE_BEGIN)? + IMMUTABLE_BEGIN.len();
    let end = content[start..].find(IMMUTABLE_END)? + start;
    Some(&content[start..end])
}

/// ตรวจว่าเนื้อหาใหม่ยอมรับได้หรือไม่
///
/// เปรียบเทียบบล็อก immutable แบบ byte-for-byte — ต่างแม้ตัวเดียวคือปฏิเสธ
pub fn validate(current: &str, proposed: &str) -> Result<(), SoulEditError> {
    let cur_block = extract_immutable(current);
    if cur_block.is_some() {
        let new_block = extract_immutable(proposed).ok_or(SoulEditError::ImmutableBlockMissing)?;
        if cur_block != Some(new_block) {
            return Err(SoulEditError::ImmutableBlockModified);
        }
    }

    if change_ratio(current, proposed) > MAX_CHANGE_RATIO {
        return Err(SoulEditError::TooLarge);
    }
    Ok(())
}

/// สัดส่วนบรรทัดที่เปลี่ยนไป เทียบกับไฟล์ที่ยาวกว่า
pub fn change_ratio(a: &str, b: &str) -> f32 {
    let av: Vec<&str> = a.lines().collect();
    let bv: Vec<&str> = b.lines().collect();
    let longest = av.len().max(bv.len());
    if longest == 0 {
        return 0.0;
    }
    let same = av.iter().zip(bv.iter()).filter(|(x, y)| x == y).count();
    let changed = longest - same;
    changed as f32 / longest as f32
}

/// ข้อความที่แสดงตอนขออนุมัติ
///
/// **ต้องแสดงระยะห่างสะสมจาก baseline เสมอ** — นี่คือมาตรการหลักที่กัน drift
/// ทำให้คนเห็นภาพรวมทุกครั้งที่ตัดสินใจ ไม่ใช่เห็นแค่การแก้ครั้งนี้
pub fn approval_summary(edit_number: u32, baseline: &str, proposed: &str) -> String {
    let ratio = change_ratio(baseline, proposed);
    let changed_lines = (ratio * baseline.lines().count().max(1) as f32).round() as u32;
    format!(
        "แก้ครั้งที่ {} · ต่างจากต้นฉบับสะสม {} บรรทัด ({:.0}%)",
        edit_number,
        changed_lines,
        ratio * 100.0
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    const CURRENT: &str = "คุณคือ Kaam\nพูดสั้น ตรงประเด็น\n<!-- kaam:immutable:begin -->\nตอบเฉพาะ chat_id ใน allowlist\nห้ามแก้บล็อกนี้\n<!-- kaam:immutable:end -->\nจบ\n";

    #[test]
    fn accepts_small_edit_outside_immutable_block() {
        let proposed = CURRENT.replace("พูดสั้น ตรงประเด็น", "พูดสั้น เป็นกันเอง");
        assert!(validate(CURRENT, &proposed).is_ok());
    }

    /// นี่คือเทสต์ที่สำคัญที่สุดในไฟล์นี้
    #[test]
    fn rejects_any_change_to_immutable_block() {
        let proposed = CURRENT.replace("ตอบเฉพาะ chat_id ใน allowlist", "ตอบทุกคน");
        assert_eq!(
            validate(CURRENT, &proposed),
            Err(SoulEditError::ImmutableBlockModified)
        );
    }

    #[test]
    fn rejects_removal_of_immutable_block() {
        let proposed = "คุณคือ Kaam\nพูดสั้น\n";
        assert_eq!(
            validate(CURRENT, proposed),
            Err(SoulEditError::ImmutableBlockMissing)
        );
    }

    #[test]
    fn rejects_rewrites_that_are_too_large() {
        let proposed = format!(
            "ก\nข\nค\nง\nจ\n{}\nฉ\nช\nซ\nฌ\n",
            &CURRENT[CURRENT.find(IMMUTABLE_BEGIN).unwrap()
                ..CURRENT.find(IMMUTABLE_END).unwrap() + IMMUTABLE_END.len()]
        );
        assert_eq!(validate(CURRENT, &proposed), Err(SoulEditError::TooLarge));
    }

    #[test]
    fn summary_always_reports_cumulative_drift() {
        let drifted = CURRENT.replace("พูดสั้น ตรงประเด็น", "พูดยาว อ้อมค้อม");
        let s = approval_summary(7, CURRENT, &drifted);
        assert!(s.contains("ครั้งที่ 7"));
        assert!(s.contains("สะสม"), "ต้องบอกระยะห่างสะสมเสมอ ไม่ใช่แค่การแก้ครั้งนี้");
    }
}
