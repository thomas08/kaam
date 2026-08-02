//! ตัวกั้นขอบเขตระบบไฟล์
//!
//! agent เขียนไฟล์ได้เฉพาะใต้ `/kaam/` และห้ามแตะ identity/ กับ config/
//! ดู ARCHITECTURE.md §13.1 และ §18.4

pub const ROOT: &str = "/kaam/";

/// path ที่ agent ห้ามเขียนผ่าน `write_file` ไม่ว่ากรณีใด
///
/// `identity/` แก้ได้ทางเดียวคือผ่าน propose_soul_edit ที่มีคนอนุมัติ
/// `config/` แก้ได้ทางเดียวคือ serial console
const WRITE_DENY: &[&str] = &["/kaam/identity/", "/kaam/config/"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PathError {
    /// ไม่ได้อยู่ใต้ /kaam/
    OutsideRoot,
    /// มี .. อยู่ใน path
    Traversal,
    /// path ที่ห้ามเขียน
    Protected,
    /// อักขระที่ไม่อนุญาต เช่น NUL
    Invalid,
}

/// ตรวจว่า path อ่านได้ไหม
pub fn sandbox_check(path: &str) -> Result<(), PathError> {
    if path.contains('\0') {
        return Err(PathError::Invalid);
    }
    if path.split('/').any(|seg| seg == "..") {
        return Err(PathError::Traversal);
    }
    if !path.starts_with(ROOT) {
        return Err(PathError::OutsideRoot);
    }
    Ok(())
}

/// ตรวจว่า path เขียนได้ไหม — เข้มกว่า `sandbox_check`
pub fn write_check(path: &str) -> Result<(), PathError> {
    sandbox_check(path)?;
    if WRITE_DENY.iter().any(|d| path.starts_with(d)) {
        return Err(PathError::Protected);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allows_normal_paths() {
        assert!(sandbox_check("/kaam/memory/MEMORY.md").is_ok());
        assert!(write_check("/kaam/memory/MEMORY.md").is_ok());
    }

    #[test]
    fn blocks_traversal_in_any_position() {
        assert_eq!(
            sandbox_check("/kaam/../etc/passwd"),
            Err(PathError::Traversal)
        );
        assert_eq!(sandbox_check("/kaam/a/../../b"), Err(PathError::Traversal));
    }

    #[test]
    fn blocks_paths_outside_root() {
        assert_eq!(sandbox_check("/etc/passwd"), Err(PathError::OutsideRoot));
        assert_eq!(sandbox_check("kaam/relative"), Err(PathError::OutsideRoot));
    }

    /// SOUL.md ต้องแก้ผ่าน propose_soul_edit เท่านั้น ไม่ใช่ write_file
    #[test]
    fn protects_identity_and_config_from_direct_writes() {
        assert!(sandbox_check("/kaam/identity/SOUL.md").is_ok(), "อ่านได้");
        assert_eq!(
            write_check("/kaam/identity/SOUL.md"),
            Err(PathError::Protected)
        );
        assert_eq!(
            write_check("/kaam/config/settings.json"),
            Err(PathError::Protected)
        );
    }

    /// ชื่อไฟล์ที่ขึ้นต้นเหมือน .. แต่ไม่ใช่ ต้องผ่าน
    #[test]
    fn does_not_overblock_dotted_filenames() {
        assert!(sandbox_check("/kaam/memory/..hidden.md").is_ok());
    }
}
