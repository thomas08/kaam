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
///
/// **บล็อกที่ใช้เทียบต้องมาจาก `SOUL.baseline.md` ไม่ใช่จาก `SOUL.md` ปัจจุบัน**
/// (ดู ARCHITECTURE.md §18.3 — baseline เก็บถาวรลบไม่ได้) ถ้าดึงจากไฟล์ปัจจุบัน
/// ที่แก้ได้ พอไฟล์นั้นเสียหรือถูก revert จนบล็อกหาย การตรวจจะถูกข้ามไปตลอดกาล
pub fn validate(baseline: &str, current: &str, proposed: &str) -> Result<(), SoulEditError> {
    if let Some(base_block) = extract_immutable(baseline) {
        let new_block = extract_immutable(proposed).ok_or(SoulEditError::ImmutableBlockMissing)?;
        if base_block != new_block {
            return Err(SoulEditError::ImmutableBlockModified);
        }
    }

    if change_ratio(current, proposed) > MAX_CHANGE_RATIO {
        return Err(SoulEditError::TooLarge);
    }
    Ok(())
}

/// สัดส่วนบรรทัดที่เปลี่ยนไป เทียบกับไฟล์ที่ยาวกว่า
///
/// ใช้ LCS ไม่ใช่การเทียบทีละตำแหน่ง — ไม่งั้นการแทรกบรรทัดเดียวที่หัวไฟล์
/// จะทำให้ทุกบรรทัดเลื่อนแล้วนับเป็นเปลี่ยน 100% ซึ่งปฏิเสธการแก้ที่ถูกต้อง
/// และผู้ใช้ซอยให้เล็กลงกว่านั้นไม่ได้อีกแล้ว
pub fn change_ratio(a: &str, b: &str) -> f32 {
    let av: Vec<&str> = a.lines().collect();
    let bv: Vec<&str> = b.lines().collect();
    let longest = av.len().max(bv.len());
    if longest == 0 {
        return 0.0;
    }
    let common = lcs_len(&av, &bv);
    (longest - common) as f32 / longest as f32
}

/// ความยาว LCS ของสองชุดบรรทัด ใช้แถวหมุนสองแถวเพื่อไม่กิน RAM เป็น n×m
fn lcs_len(a: &[&str], b: &[&str]) -> usize {
    let mut prev = vec![0usize; b.len() + 1];
    let mut cur = vec![0usize; b.len() + 1];
    for i in 1..=a.len() {
        for j in 1..=b.len() {
            cur[j] = if a[i - 1] == b[j - 1] {
                prev[j - 1] + 1
            } else {
                prev[j].max(cur[j - 1])
            };
        }
        std::mem::swap(&mut prev, &mut cur);
        cur.iter_mut().for_each(|v| *v = 0);
    }
    prev[b.len()]
}

/// ข้อความที่แสดงตอนขออนุมัติ
///
/// **ต้องแสดงระยะห่างสะสมจาก baseline เสมอ** — นี่คือมาตรการหลักที่กัน drift
/// ทำให้คนเห็นภาพรวมทุกครั้งที่ตัดสินใจ ไม่ใช่เห็นแค่การแก้ครั้งนี้
pub fn approval_summary(edit_number: u32, baseline: &str, proposed: &str) -> String {
    let ratio = change_ratio(baseline, proposed);
    // ตัวหารต้องเป็นตัวเดียวกับที่ change_ratio ใช้ ไม่งั้นจำนวนบรรทัดที่รายงาน
    // จะต่ำกว่าจริงเมื่อ proposed ยาวกว่า baseline
    let total = baseline
        .lines()
        .count()
        .max(proposed.lines().count())
        .max(1);
    let changed_lines = (ratio * total as f32).round() as u32;
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
        assert!(validate(CURRENT, CURRENT, &proposed).is_ok());
    }

    /// นี่คือเทสต์ที่สำคัญที่สุดในไฟล์นี้
    #[test]
    fn rejects_any_change_to_immutable_block() {
        let proposed = CURRENT.replace("ตอบเฉพาะ chat_id ใน allowlist", "ตอบทุกคน");
        assert_eq!(
            validate(CURRENT, CURRENT, &proposed),
            Err(SoulEditError::ImmutableBlockModified)
        );
    }

    #[test]
    fn rejects_removal_of_immutable_block() {
        let proposed = "คุณคือ Kaam\nพูดสั้น\n";
        assert_eq!(
            validate(CURRENT, CURRENT, proposed),
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
        assert_eq!(
            validate(CURRENT, CURRENT, &proposed),
            Err(SoulEditError::TooLarge)
        );
    }

    /// ถ้า SOUL.md ปัจจุบันเสียจนบล็อกหาย baseline ต้องยังบังคับให้เอากลับมา
    /// ไม่ใช่ปล่อยผ่านเพราะ "ไฟล์ปัจจุบันไม่มีอะไรให้ตรวจ"
    #[test]
    fn baseline_still_enforces_when_current_lost_the_block() {
        let broken = "คุณคือ Kaam\nพูดสั้น\nจบ\n";
        let proposed = "คุณคือ Kaam\nตอบทุกคน\nจบ\n";
        assert_eq!(
            validate(CURRENT, broken, proposed),
            Err(SoulEditError::ImmutableBlockMissing)
        );
    }

    /// แทรกบรรทัดเดียวต้องไม่ถูกนับเป็นเขียนใหม่ทั้งไฟล์
    #[test]
    fn inserting_one_line_is_a_small_edit() {
        let proposed = format!("บรรทัดใหม่ที่หัวไฟล์\n{CURRENT}");
        let r = change_ratio(CURRENT, &proposed);
        assert!(r < MAX_CHANGE_RATIO, "ratio={r} ไม่ควรถูกมองว่าแก้ทั้งไฟล์");
        assert!(validate(CURRENT, CURRENT, &proposed).is_ok());
    }

    /// จำนวนบรรทัดที่รายงานต้องคิดจากตัวหารเดียวกับเปอร์เซ็นต์
    #[test]
    fn summary_line_count_matches_the_percentage() {
        let baseline = "ก\nข\nค\n";
        let proposed = "ก\n1\n2\n3\n4\n5\n6\n7\n";
        let s = approval_summary(1, baseline, proposed);
        let ratio = change_ratio(baseline, proposed);
        let expected = (ratio * 8.0).round() as u32;
        assert!(s.contains(&format!("{expected} บรรทัด")), "ได้: {s}");
    }

    #[test]
    fn summary_always_reports_cumulative_drift() {
        let drifted = CURRENT.replace("พูดสั้น ตรงประเด็น", "พูดยาว อ้อมค้อม");
        let s = approval_summary(7, CURRENT, &drifted);
        assert!(s.contains("ครั้งที่ 7"));
        assert!(s.contains("สะสม"), "ต้องบอกระยะห่างสะสมเสมอ ไม่ใช่แค่การแก้ครั้งนี้");
    }
}
