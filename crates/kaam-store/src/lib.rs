//! ชั้นเก็บข้อมูล — trait + implementation ในหน่วยความจำสำหรับทดสอบ
//!
//! implementation จริงบน LittleFS อยู่ใน firmware/kaam-fw
#![forbid(unsafe_code)]

pub mod guard;

use std::collections::BTreeMap;

pub use guard::{sandbox_check, PathError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StoreError {
    NotFound,
    Denied(PathError),
    Io(String),
}

pub trait Store {
    fn read(&self, path: &str) -> Result<String, StoreError>;
    /// ต้องเขียนแบบ atomic: tmp -> fsync -> rename
    fn write(&mut self, path: &str, content: &str) -> Result<(), StoreError>;
    fn append(&mut self, path: &str, content: &str) -> Result<(), StoreError>;
    fn list(&self, prefix: &str) -> Result<Vec<String>, StoreError>;
    fn delete(&mut self, path: &str) -> Result<(), StoreError>;
}

/// Store ในหน่วยความจำ ใช้ในเทสต์ทั้งหมดของ crates อื่น
#[derive(Debug, Default)]
pub struct MemStore {
    files: BTreeMap<String, String>,
}

impl MemStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_file(mut self, path: &str, content: &str) -> Self {
        self.files.insert(path.to_string(), content.to_string());
        self
    }
}

impl Store for MemStore {
    fn read(&self, path: &str) -> Result<String, StoreError> {
        sandbox_check(path).map_err(StoreError::Denied)?;
        self.files.get(path).cloned().ok_or(StoreError::NotFound)
    }

    fn write(&mut self, path: &str, content: &str) -> Result<(), StoreError> {
        sandbox_check(path).map_err(StoreError::Denied)?;
        self.files.insert(path.to_string(), content.to_string());
        Ok(())
    }

    fn append(&mut self, path: &str, content: &str) -> Result<(), StoreError> {
        sandbox_check(path).map_err(StoreError::Denied)?;
        self.files
            .entry(path.to_string())
            .or_default()
            .push_str(content);
        Ok(())
    }

    fn list(&self, prefix: &str) -> Result<Vec<String>, StoreError> {
        sandbox_check(prefix).map_err(StoreError::Denied)?;
        Ok(self
            .files
            .keys()
            .filter(|k| k.starts_with(prefix))
            .cloned()
            .collect())
    }

    fn delete(&mut self, path: &str) -> Result<(), StoreError> {
        sandbox_check(path).map_err(StoreError::Denied)?;
        self.files
            .remove(path)
            .map(|_| ())
            .ok_or(StoreError::NotFound)
    }
}

/// อ่านไฟล์ JSONL แบบทนความเสียหาย
///
/// ถ้าไฟดับกลางการเขียน บรรทัดสุดท้ายจะไม่สมบูรณ์ — ทิ้งไปเงียบ ๆ แล้วใช้ที่เหลือ
/// คืนค่า (บรรทัดที่ใช้ได้, จำนวนบรรทัดที่ทิ้ง)
pub fn parse_jsonl_lossy(raw: &str) -> (Vec<&str>, usize) {
    let mut good = Vec::new();
    let mut dropped = 0;
    for line in raw.lines() {
        let t = line.trim();
        if t.is_empty() {
            continue;
        }
        if t.starts_with('{') && t.ends_with('}') {
            good.push(t);
        } else {
            dropped += 1;
        }
    }
    (good, dropped)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_through_memstore() {
        let mut s = MemStore::new();
        s.write("/kaam/memory/MEMORY.md", "จำได้").unwrap();
        assert_eq!(s.read("/kaam/memory/MEMORY.md").unwrap(), "จำได้");
    }

    #[test]
    fn rejects_escape_attempts() {
        let mut s = MemStore::new();
        assert!(matches!(
            s.write("/kaam/../etc/passwd", "x"),
            Err(StoreError::Denied(_))
        ));
    }

    /// `list` เป็นทางรั่วได้เหมือน `read` — ต้องผ่าน guard เหมือนกันทุกเมธอด
    #[test]
    fn list_is_guarded_like_every_other_method() {
        let s = MemStore::new().with_file("/kaam/memory/a.md", "x");
        assert!(matches!(s.list("/etc/"), Err(StoreError::Denied(_))));
        assert!(matches!(
            s.list("/kaam/../etc/"),
            Err(StoreError::Denied(_))
        ));
        assert_eq!(s.list("/kaam/memory/").unwrap().len(), 1);
    }

    #[test]
    fn recovers_from_torn_final_line() {
        let raw = "{\"a\":1}\n{\"b\":2}\n{\"c\":";
        let (good, dropped) = parse_jsonl_lossy(raw);
        assert_eq!(good.len(), 2);
        assert_eq!(dropped, 1);
    }
}
