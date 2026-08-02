//! ตัวถอด Server-Sent Events แบบ incremental
//!
//! นี่คือชิ้นส่วนที่พังบ่อยที่สุดบน MCU จึงต้องทดสอบให้หนักที่สุด
//!
//! หมายเหตุสำคัญเรื่อง UTF-8: เราแบ่งบรรทัดที่ระดับ **ไบต์** โดยใช้ `\n`
//! ซึ่งปลอดภัยเพราะ `\n` (0x0A) ไม่มีวันปรากฏเป็นส่วนหนึ่งของอักขระ UTF-8 หลายไบต์
//! ทำให้ chunk ที่ขาดกลางคำภาษาไทยไม่ทำให้ decoder พัง

/// เพดานความยาวหนึ่ง event กันหน่วยความจำโตไม่จำกัดจาก server ที่ประพฤติผิดปกติ
const MAX_EVENT_BYTES: usize = 256 * 1024;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SseEvent {
    /// ค่าจากฟิลด์ `event:` — ว่างถ้าไม่ได้ระบุ
    pub name: String,
    /// ค่าจากฟิลด์ `data:` ต่อกันด้วย `\n` ถ้ามีหลายบรรทัด
    pub data: String,
}

#[derive(Debug, Default)]
pub struct SseDecoder {
    line_buf: Vec<u8>,
    event_name: String,
    data: String,
    overflowed: bool,
}

impl SseDecoder {
    pub fn new() -> Self {
        Self::default()
    }

    /// ป้อนไบต์เข้าไป แล้วรับ event ที่ครบสมบูรณ์ออกมา
    pub fn feed(&mut self, chunk: &[u8]) -> Vec<SseEvent> {
        let mut out = Vec::new();
        for &byte in chunk {
            if byte == b'\n' {
                let line = std::mem::take(&mut self.line_buf);
                if let Some(ev) = self.handle_line(&line) {
                    out.push(ev);
                }
            } else if byte != b'\r' {
                if self.line_buf.len() < MAX_EVENT_BYTES {
                    self.line_buf.push(byte);
                } else {
                    self.overflowed = true;
                }
            }
        }
        out
    }

    /// true ถ้าเคยมี event ที่ยาวเกินเพดานจนต้องตัดทิ้ง
    pub fn overflowed(&self) -> bool {
        self.overflowed
    }

    fn handle_line(&mut self, line: &[u8]) -> Option<SseEvent> {
        // บรรทัดว่าง = จบหนึ่ง event
        if line.is_empty() {
            if self.data.is_empty() && self.event_name.is_empty() {
                return None;
            }
            return Some(SseEvent {
                name: std::mem::take(&mut self.event_name),
                data: std::mem::take(&mut self.data),
            });
        }

        // บรรทัดที่ขึ้นต้นด้วย ':' คือ comment ใช้ keep-alive
        if line[0] == b':' {
            return None;
        }

        let text = String::from_utf8_lossy(line);
        let (field, value) = match text.find(':') {
            Some(i) => (
                &text[..i],
                text[i + 1..].strip_prefix(' ').unwrap_or(&text[i + 1..]),
            ),
            None => (&text[..], ""),
        };

        match field {
            "event" => self.event_name = value.to_string(),
            "data" => {
                if !self.data.is_empty() {
                    self.data.push('\n');
                }
                self.data.push_str(value);
            }
            _ => {} // id / retry — ยังไม่ใช้
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_single_event() {
        let mut d = SseDecoder::new();
        let events = d.feed(b"event: message\ndata: hello\n\n");
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].name, "message");
        assert_eq!(events[0].data, "hello");
    }

    /// นี่คือกรณีที่สำคัญที่สุด: TCP แบ่ง chunk ตรงไหนก็ได้
    #[test]
    fn survives_split_anywhere() {
        let full = b"event: delta\ndata: {\"text\":\"hi\"}\n\nevent: done\ndata: [DONE]\n\n";
        for split in 1..full.len() {
            let mut d = SseDecoder::new();
            let mut events = d.feed(&full[..split]);
            events.extend(d.feed(&full[split..]));
            assert_eq!(events.len(), 2, "พังเมื่อแบ่งที่ไบต์ {}", split);
            assert_eq!(events[0].data, "{\"text\":\"hi\"}");
            assert_eq!(events[1].name, "done");
        }
    }

    /// chunk ที่ขาดกลางอักขระไทย ต้องไม่ทำให้ decoder เสียหาย
    #[test]
    fn survives_split_mid_utf8_character() {
        let payload = "data: สวัสดีครับ\n\n".as_bytes().to_vec();
        for split in 1..payload.len() {
            let mut d = SseDecoder::new();
            let mut events = d.feed(&payload[..split]);
            events.extend(d.feed(&payload[split..]));
            assert_eq!(events.len(), 1, "พังเมื่อแบ่งที่ไบต์ {}", split);
            assert_eq!(events[0].data, "สวัสดีครับ");
        }
    }

    #[test]
    fn ignores_keepalive_comments() {
        let mut d = SseDecoder::new();
        assert!(d.feed(b": ping\n\n").is_empty());
    }

    #[test]
    fn handles_crlf_line_endings() {
        let mut d = SseDecoder::new();
        let events = d.feed(b"data: x\r\n\r\n");
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].data, "x");
    }

    #[test]
    fn joins_multiline_data() {
        let mut d = SseDecoder::new();
        let events = d.feed(b"data: line1\ndata: line2\n\n");
        assert_eq!(events[0].data, "line1\nline2");
    }
}
