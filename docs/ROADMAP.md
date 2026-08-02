# Roadmap

เกณฑ์ผ่านคือสิ่งที่วัดได้ ไม่ใช่ความรู้สึกว่าเสร็จแล้ว

| M | เนื้อหา | เกณฑ์ผ่าน | สถานะ |
|---|---|---|---|
| M0 | โครง repo, CI, เทสต์บน host, ยืนยัน toolchain ของ C5 | CI เขียว + flash blink ขึ้นบอร์ดได้ | 🟡 เทสต์ผ่านแล้ว ยังไม่ยืนยัน toolchain |
| M1 | WiFi + SNTP + TLS + Telegram long-poll | echo bot อยู่ได้ 24 ชม. โดย heap ไม่ลด | ⬜ |
| M2 | Anthropic streaming + งบ token | คุยได้ ตอบทยอยออก เพดานรายวันทำงานจริง | ⬜ |
| M3 | LittleFS + session JSONL + SOUL/USER/MEMORY | จำข้ามรีบูตได้ ตัดไฟกลางเทิร์นแล้วไม่เสียหาย | ⬜ |
| M4 | Tool loop + built-in tools + นโยบายอนุญาต | เรียก tool ต่อกัน 3 ชั้นสำเร็จ | ⬜ |
| M5 | Compaction + journal | คุย 200 เทิร์นติดกันโดยไม่ล้ม | ⬜ |
| M6 | Skills + OpenAI provider | สลับ provider ได้ตอนรันไทม์ | ⬜ |
| M7 | MCP client | ต่อ MCP server จริงได้หนึ่งเจ้า | ⬜ |
| M8 | Scheduler + OTA + secure boot | อัปเดตข้ามเน็ตแล้ว rollback ได้จริง | ⬜ |
| M9 | `propose_soul_edit` + history + `/soul` | เสนอ อนุมัติ ย้อนกลับ ครบวง และ immutable block กันได้จริง | ⬜ |
| M10 | เสียง push-to-talk เต็มรูปแบบ | กดปุ่มพูดแล้วได้ยินคำตอบใน 3 วินาที แตะหยุดได้ทันที | ⬜ |

## สิ่งที่ต้องรู้ก่อนเริ่ม

**M1 กับ M2 จะกินเวลามากกว่าที่คิด 2–3 เท่า** เพราะปัญหา heap และ TLS ทั้งหมดจะโผล่ตรงนี้
ถ้าผ่านสองด่านนี้ได้ ที่เหลือเป็นงานเชิงตรรกะซึ่งทดสอบบนพีซีได้เกือบหมด

**ทำ mock provider ตั้งแต่ M2** ที่เล่น SSE จากไฟล์ที่บันทึกไว้ แล้วใช้ตลอด M3–M9
จะได้ไม่ต้องจ่ายค่า API ทุกครั้งที่รันเทสต์

## งานที่เหลือใน M0

1. ยืนยันชื่อ target ของ ESP32-C5 (ตั้งไว้ `riscv32imac-esp-espidf`)
2. ยืนยันเวอร์ชัน ESP-IDF ที่รองรับ C5 (ตั้งไว้ v5.5)
3. ยืนยันเวอร์ชัน `esp-idf-svc` / `esp-idf-hal` ที่เข้ากัน
4. ตรวจว่ายังต้อง nightly เพราะ `build-std` หรือ stable พอแล้ว
5. เปิด job `firmware` ใน CI (ตอนนี้ `if: false`)
