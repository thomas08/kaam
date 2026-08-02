//! ตัวอ่าน SKILL.md
//!
//! กลยุทธ์หน่วยความจำ: ตอนบูตอ่านเฉพาะ frontmatter ของทุก skill (~100 ไบต์/อัน)
//! เนื้อหาเต็มโหลดจากแฟลชเฉพาะตอนที่ LLM ตัดสินใจใช้
//! มี 50 skill ก็ยังกิน RAM แค่ ~5 KB
#![forbid(unsafe_code)]

/// หัวข้อของ skill ที่อยู่ใน RAM ตลอดเวลา
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkillHeader {
    pub name: String,
    pub description: String,
    /// path ของไฟล์ ใช้โหลดเนื้อหาเต็มตอนถูกเรียก
    pub path: String,
}

impl SkillHeader {
    /// บรรทัดที่จะใส่ในระบบ prompt — สั้นที่สุดเท่าที่ยังพอให้ LLM ตัดสินใจได้
    pub fn prompt_line(&self) -> String {
        format!("- {}: {}", self.name, self.description)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkillError {
    MissingFrontmatter,
    MissingName,
    MissingDescription,
}

/// อ่าน frontmatter โดยไม่แตะเนื้อหาส่วนที่เหลือ
///
/// รับเฉพาะ `name` และ `description` — ฟิลด์อื่นถูกละเว้นอย่างเงียบ ๆ
/// เพื่อให้ไฟล์ที่เขียนสำหรับ harness อื่นยังใช้ได้
pub fn parse_header(path: &str, raw: &str) -> Result<SkillHeader, SkillError> {
    let rest = raw
        .strip_prefix("---")
        .ok_or(SkillError::MissingFrontmatter)?;
    let end = rest.find("\n---").ok_or(SkillError::MissingFrontmatter)?;
    let front = &rest[..end];

    let mut name = None;
    let mut description = None;
    for line in front.lines() {
        let Some((k, v)) = line.split_once(':') else {
            continue;
        };
        let v = v.trim().trim_matches('"').trim_matches('\'');
        match k.trim() {
            "name" => name = Some(v.to_string()),
            "description" => description = Some(v.to_string()),
            _ => {}
        }
    }

    Ok(SkillHeader {
        name: name
            .filter(|s| !s.is_empty())
            .ok_or(SkillError::MissingName)?,
        description: description
            .filter(|s| !s.is_empty())
            .ok_or(SkillError::MissingDescription)?,
        path: path.to_string(),
    })
}

/// ดึงเนื้อหาหลัง frontmatter — เรียกเฉพาะตอน skill ถูก trigger
pub fn parse_body(raw: &str) -> &str {
    let Some(rest) = raw.strip_prefix("---") else {
        return raw;
    };
    match rest.find("\n---") {
        Some(end) => rest[end + 4..].trim_start(),
        None => raw,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "---\nname: morning-brief\ndescription: สรุปข่าวและตารางงานตอนเช้า\n---\nเนื้อหาเต็มอยู่ตรงนี้\nบรรทัดสอง\n";

    #[test]
    fn reads_header_without_touching_body() {
        let h = parse_header("/kaam/skills/morning-brief/SKILL.md", SAMPLE).unwrap();
        assert_eq!(h.name, "morning-brief");
        assert_eq!(h.description, "สรุปข่าวและตารางงานตอนเช้า");
    }

    #[test]
    fn body_is_separate_from_header() {
        assert!(parse_body(SAMPLE).starts_with("เนื้อหาเต็ม"));
    }

    #[test]
    fn tolerates_unknown_frontmatter_fields() {
        let raw = "---\nname: x\nlicense: MIT\ndescription: y\nversion: 2\n---\nbody";
        assert!(parse_header("p", raw).is_ok());
    }

    #[test]
    fn rejects_missing_required_fields() {
        assert_eq!(
            parse_header("p", "---\nname: x\n---\nbody"),
            Err(SkillError::MissingDescription)
        );
        assert_eq!(
            parse_header("p", "no frontmatter"),
            Err(SkillError::MissingFrontmatter)
        );
    }

    #[test]
    fn header_line_is_compact() {
        let h = parse_header("p", SAMPLE).unwrap();
        assert!(h.prompt_line().len() < 200, "หัวข้อ skill ต้องสั้นเพื่อประหยัด RAM");
    }
}
