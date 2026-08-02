//! ตัวตัดประโยคสำหรับ TTS
//!
//! เหตุผล: ถ้ารอ LLM พูดจบทั้งย่อหน้าก่อนเริ่มสังเคราะห์เสียง
//! ผู้ใช้จะรอ ~8 วินาที การตัดตามขอบประโยคลดเหลือ ~2.5 วินาที
//! ซึ่งเป็นความต่างระหว่าง "ใช้ได้" กับ "ทนไม่ไหว"
//!
//! ภาษาไทยไม่เว้นวรรคระหว่างคำ จึงพึ่ง `.` อย่างเดียวไม่ได้ —
//! ต้องใช้เครื่องหมายที่ปรากฏจริงบวกกับความยาวสูงสุดเป็นตัวบังคับ

/// ถ้ายังไม่เจอขอบประโยคภายในความยาวนี้ ให้ตัดที่ช่องว่างล่าสุด
const MAX_CHUNK_CHARS: usize = 180;

const TERMINATORS: &[char] = &['.', '!', '?', '\n', '。', '！', '？'];

#[derive(Debug, Default)]
pub struct SentenceSplitter {
    buf: String,
}

impl SentenceSplitter {
    pub fn new() -> Self {
        Self::default()
    }

    /// ป้อน delta จาก LLM แล้วรับประโยคที่พร้อมส่งเข้า TTS
    pub fn feed(&mut self, delta: &str) -> Vec<String> {
        self.buf.push_str(delta);
        let mut out = Vec::new();

        while let Some(idx) = self.find_boundary() {
            let rest = self.buf.split_off(idx);
            let chunk = std::mem::replace(&mut self.buf, rest);
            let trimmed = chunk.trim().to_string();
            if !trimmed.is_empty() {
                out.push(trimmed);
            }
        }
        out
    }

    /// เรียกเมื่อ LLM พูดจบ — คายส่วนที่ค้างอยู่
    pub fn finish(&mut self) -> Option<String> {
        let rest = std::mem::take(&mut self.buf);
        let t = rest.trim();
        (!t.is_empty()).then(|| t.to_string())
    }

    /// คืน byte index ที่ควรตัด (หลังเครื่องหมายจบประโยค)
    fn find_boundary(&self) -> Option<usize> {
        for (i, c) in self.buf.char_indices() {
            if TERMINATORS.contains(&c) {
                if c == '.' {
                    // ตัวเลขทศนิยม เช่น 3.14 ไม่ใช่จุดจบประโยค
                    if self.is_inside_number(i) {
                        continue;
                    }
                    // จุดอยู่ท้ายบัฟเฟอร์พอดีและมีเลขนำหน้า — ยังแยกไม่ออกว่า
                    // เป็น "3.14" หรือจบประโยคจริง ต้องรอ delta ถัดไปก่อน
                    if self.is_pending_decimal_point(i) {
                        break;
                    }
                }
                return Some(i + c.len_utf8());
            }
        }
        // ยาวเกินเพดานแล้วยังไม่เจอขอบ — ตัดที่ช่องว่างล่าสุด **ภายในเพดาน**
        // (ค้นทั้งบัฟเฟอร์จะได้ชิ้นยาวเกิน MAX_CHUNK_CHARS ซึ่งลบล้างเหตุผลของเพดาน)
        if self.buf.chars().count() > MAX_CHUNK_CHARS {
            let limit = self
                .buf
                .char_indices()
                .nth(MAX_CHUNK_CHARS)
                .map_or(self.buf.len(), |(i, _)| i);
            if let Some(sp) = self.buf[..limit].rfind(' ') {
                return Some(sp + 1);
            }
            return Some(limit);
        }
        None
    }

    fn is_inside_number(&self, dot: usize) -> bool {
        let before = self.buf[..dot].chars().next_back();
        let after = self.buf[dot + 1..].chars().next();
        matches!((before, after), (Some(b), Some(a)) if b.is_ascii_digit() && a.is_ascii_digit())
    }

    /// จุดที่เป็นตัวอักษรสุดท้ายของบัฟเฟอร์และมีเลขนำหน้า — ยังตัดสินไม่ได้
    fn is_pending_decimal_point(&self, dot: usize) -> bool {
        self.buf[dot + 1..].chars().next().is_none()
            && matches!(self.buf[..dot].chars().next_back(), Some(b) if b.is_ascii_digit())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn emits_each_sentence_as_soon_as_it_completes() {
        let mut s = SentenceSplitter::new();
        assert!(s.feed("ยังไม่จบ").is_empty());
        let out = s.feed("ประโยค. ต่อไป");
        assert_eq!(out, vec!["ยังไม่จบประโยค."]);
    }

    #[test]
    fn streams_incrementally_one_char_at_a_time() {
        let mut s = SentenceSplitter::new();
        let mut all = Vec::new();
        for c in "หนึ่ง. สอง! สาม?".chars() {
            all.extend(s.feed(&c.to_string()));
        }
        all.extend(s.finish());
        assert_eq!(all, vec!["หนึ่ง.", "สอง!", "สาม?"]);
    }

    #[test]
    fn does_not_split_decimal_numbers() {
        let mut s = SentenceSplitter::new();
        let out = s.feed("ราคา 3.14 บาท. จบ");
        assert_eq!(out, vec!["ราคา 3.14 บาท."]);
    }

    /// ภาษาไทยที่ไม่มีเครื่องหมายเลย ต้องยังถูกตัดเพื่อไม่ให้ TTS รอนาน
    #[test]
    fn force_splits_long_run_without_punctuation() {
        let mut s = SentenceSplitter::new();
        let long: String = "ก".repeat(400);
        let out = s.feed(&long);
        assert!(!out.is_empty(), "ข้อความยาวไม่มีเครื่องหมายต้องถูกบังคับตัด");
    }

    /// ช่องว่างที่อยู่นอกเพดานต้องไม่ถูกใช้เป็นจุดตัด ไม่งั้นชิ้นจะยาวเกินงบดีเลย์
    #[test]
    fn force_split_never_exceeds_the_cap() {
        let mut s = SentenceSplitter::new();
        let out = s.feed(&format!("{}{}{}", "ก".repeat(390), " ", "ข".repeat(9)));
        assert!(!out.is_empty());
        assert!(
            out[0].chars().count() <= MAX_CHUNK_CHARS,
            "ได้ชิ้นยาว {} ตัวอักษร ทั้งที่เพดานคือ {MAX_CHUNK_CHARS}",
            out[0].chars().count()
        );
    }

    /// delta ขาดหลังจุดทศนิยมพอดี ต้องรอ ไม่ใช่ตัดเลขครึ่ง
    #[test]
    fn waits_when_chunk_ends_on_a_decimal_point() {
        let mut s = SentenceSplitter::new();
        assert!(s.feed("ราคา 3.").is_empty(), "ต้องรอ delta ถัดไป");
        assert_eq!(s.feed("14 บาท. จบ"), vec!["ราคา 3.14 บาท."]);
    }

    /// จุดจบประโยคปกติที่ไม่มีเลขนำหน้า ต้องคายทันที ไม่ต้องรอ
    #[test]
    fn does_not_delay_ordinary_sentence_end() {
        let mut s = SentenceSplitter::new();
        assert_eq!(s.feed("จบแล้ว."), vec!["จบแล้ว."]);
    }

    #[test]
    fn finish_flushes_trailing_text() {
        let mut s = SentenceSplitter::new();
        s.feed("ค้างอยู่");
        assert_eq!(s.finish(), Some("ค้างอยู่".to_string()));
        assert_eq!(s.finish(), None);
    }
}
