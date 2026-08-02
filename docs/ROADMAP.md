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

ยืนยันแล้วด้วยการ build จริงบน ESP-IDF v5.5:

1. ✅ target คือ `riscv32imac-esp-espidf` — `toolchain-esp32c5.cmake` ใช้
   `-march=rv32imac_zicsr_zifencei` เหมือน C6 และ `esp-idf-sys` แมป target นี้
   เป็น `[C6, C5, H2]` โดยมี **C6 เป็นค่าเริ่มต้น** — `MCU=esp32c5` จึงห้ามลบ
2. ✅ ESP-IDF v5.5 build C5 ได้ แต่ C5 อยู่ใน `PREVIEW_TARGETS` ไม่ใช่
   `SUPPORTED_TARGETS` (`idf.py --list-targets` ไม่แสดง ต้องใส่ `--preview`)
3. ✅ ต้องใช้ `esp-idf-svc 0.52` / `esp-idf-hal 0.46` / `esp-idf-sys 0.37`
   — ชุด 0.51/0.45/0.36 คอมไพล์ไม่ผ่านเพราะ binding ไม่ตรงกับ IDF v5.5
4. ✅ **ยังต้องใช้ nightly** — `rustup target add riscv32imac-esp-espidf` บน stable
   ตอบว่า "no prebuilt artifacts available" จึงต้อง `build-std` ซึ่งเป็น nightly-only
5. ⬜ เปิด job `firmware` ใน CI

### ตัวเลขฐานแรก (M0, main.rs ยังเป็นโครงเปล่า)

| ตัวชี้วัด | ค่า |
|---|---|
| binary | 443,712 ไบต์ (0.42 MB) — 15% ของเพดาน 2.8 MB |
| ใช้ ota_0 | 12% ของ 3.5 MB |

**อย่าเอาไปอ้างว่าเหลืองบเยอะ** — ยังไม่มี WiFi, TLS, mbedTLS, Telegram หรือ
provider เลย §16 ประเมินว่าพอมี TLS จะขึ้นไป 1.5–2.5 MB ตัวเลขนี้เป็นพื้น ไม่ใช่ตัวแทน
