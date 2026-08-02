#!/usr/bin/env bash
# บังคับกฎข้อ 1: crates/ ห้ามแตะ ESP-IDF
# กฎนี้คือสิ่งที่ทำให้ทดสอบบนพีซีได้ ถ้าหลุดเมื่อไหร่โปรเจกต์จะ debug ยากขึ้นทันที
set -euo pipefail

fail=0
for manifest in crates/*/Cargo.toml; do
  if grep -qE '^\s*(esp-idf|embuild|esp-hal)' "$manifest"; then
    echo "ผิดกฎ: $manifest depend ESP-IDF — logic ต้องทดสอบบน host ได้"
    fail=1
  fi
done

if grep -rqE 'use\s+esp_idf' crates/ 2>/dev/null; then
  echo "ผิดกฎ: พบ 'use esp_idf' ใน crates/"
  grep -rnE 'use\s+esp_idf' crates/ || true
  fail=1
fi

if [ "$fail" -eq 0 ]; then
  echo "ผ่าน: crates/ สะอาด ไม่มี dependency ฝั่งฮาร์ดแวร์"
fi
exit "$fail"
