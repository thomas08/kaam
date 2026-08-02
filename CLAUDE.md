# Kaam — คู่มือสำหรับ Claude Code

Kaam (ก้าม) คือ AI agent harness ที่รันเองได้ครบบน ESP32-C5 เขียนด้วย Rust
รับคำสั่งผ่าน Telegram และเสียง (push-to-talk) เรียก LLM ผ่านคลาวด์ จำเรื่องข้ามรีบูตได้

**อ่าน `docs/ARCHITECTURE.md` ก่อนเริ่มงานใด ๆ** เอกสารนั้นคือแหล่งความจริง
ไฟล์นี้เป็นเพียงบทสรุปของกฎที่ต้องเคารพและคำสั่งที่ใช้บ่อย

สถานะปัจจุบัน: **M0** — โครง repo พร้อม เทสต์บน host ผ่าน 52 ตัว firmware ยังเป็นโครงเปล่า

---

## กฎเหล็ก

กฎพวกนี้ไม่ใช่คำแนะนำ ถ้าจะละเมิดต้องถามเจ้าของโปรเจกต์ก่อนเสมอ

### 1. `crates/` ห้ามแตะ ESP-IDF

crate ทุกตัวใน `crates/` ห้าม depend `esp-idf-*`, `embuild` หรืออะไรที่ต้องมีฮาร์ดแวร์
เหตุผลคือทดสอบบนพีซีได้ 85–90% ของ logic ซึ่งถูกกว่าการ debug บนชิปมาก

โค้ดที่รู้จักฮาร์ดแวร์อยู่ใน `firmware/kaam-fw` ที่เดียว
มี `scripts/check-no-esp-idf.sh` บังคับใช้ใน CI

### 2. Stream ทุกอย่าง ทั้งขาส่งและขารับ

ห้าม buffer ทั้ง request หรือทั้ง response ไว้ใน RAM

ขารับใช้ SSE incremental parsing (มีแล้วใน `kaam-provider::sse`)
**ขาส่งก็ต้อง stream ด้วย** — conversation 60 KB พอ serialize เป็น JSON กลายเป็นก้อน 80 KB
ที่ต้องต่อเนื่องกันใน heap ที่ fragment แล้ว ใช้ HTTP chunked encoding แล้วสร้าง JSON ทีละ message

### 3. งบประมาณคือสัญญา

| ตัวชี้วัด | เกณฑ์ |
|---|---|
| free internal SRAM | ≥ 45 KB (ปฏิเสธเทิร์นใหม่ถ้าต่ำกว่านี้) |
| largest free block | ≥ 32 KB |
| free PSRAM | ≥ 2 MB |
| binary size | ≤ 2.8 MB (เกินถือเป็น build failure) |
| stack high-water mark | เหลือ ≥ 20% ทุก task |

ถ้าการเปลี่ยนแปลงทำให้ตัวเลขพวกนี้แย่ลง ต้องบอกในสรุปงานเสมอ

### 4. อย่าลืมว่ามันคือ single core

ESP32-C5 มีคอร์เดียว 240 MHz (LP core 40 MHz ใช้ทำงาน agent ไม่ได้)

- ห้าม busy-wait ทุกกรณี ต้อง block บน queue หรือ semaphore
- งานยาวต้องหั่นเป็นชิ้นแล้ว yield ระหว่างชิ้น
- TLS handshake คือ CPU spike ที่ทำให้ task อื่นหยุดหายใจ
- มีเทิร์นที่กำลังทำงานได้ทีละหนึ่งเท่านั้น

### 5. ทุกอย่างเป็นไฟล์ที่มนุษย์อ่านได้

state ทั้งหมดเป็น Markdown หรือ JSONL ใต้ `/kaam/` บน LittleFS
เขียนแบบ atomic เสมอ (tmp → fsync → rename) และ session เป็น append-only JSONL
เพื่อให้ไฟดับกลางทางเสียแค่บรรทัดสุดท้าย

### 6. ความปลอดภัยที่ห้ามอ่อนข้อ

- **allowlist `chat_id`** — ข้อความจาก id อื่นทิ้งเงียบ ๆ ไม่ตอบแม้แต่ error
- `write_file` ต้อง `deny` กับ `/kaam/identity/` และ `/kaam/config/` เสมอ
- `SOUL.md` แก้ได้ทางเดียวคือ `propose_soul_edit` ที่มีคนอนุมัติ (ดู §18 ในเอกสาร)
- บล็อก `kaam:immutable` เทียบแบบ byte-for-byte ต่างแม้ตัวเดียวคือปฏิเสธ
- `settings.json`, allowlist, เพดาน token แก้ได้ทาง serial console เท่านั้น
- API key อยู่ใน NVS ที่เข้ารหัส **ห้ามอยู่ในซอร์สหรือไฟล์ header** และห้ามโผล่ใน log

---

## คำสั่งที่ใช้บ่อย

```bash
# ทดสอบ logic ทั้งหมด — ทำบนพีซี ไม่ต้องมีบอร์ด
cargo test --workspace

# ตรวจสไตล์
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all --check

# ตรวจว่าไม่มี crate ไหนแอบ depend ESP-IDF
./scripts/check-no-esp-idf.sh

# ---- ต้องมีฮาร์ดแวร์และ ESP-IDF ----
cd firmware/kaam-fw
cargo build --release              # ตรวจขนาด binary ด้วย
cargo run --release                # flash + monitor ผ่าน espflash
cargo build --release --features voice
```

## โครงสร้าง

```
crates/            logic ล้วน ทดสอบบน host — ห้ามมี esp-idf
  kaam-types/      ชนิดข้อมูลกลาง + งบประมาณ
  kaam-provider/   sans-io: byte เข้า → event ออก (SSE decoder อยู่นี่)
  kaam-agent/      state machine ของเทิร์น + context manager
  kaam-tools/      ทะเบียน tool + นโยบายอนุญาต + กลไกแก้ SOUL.md
  kaam-store/      trait storage + path guard + MemStore สำหรับเทสต์
  kaam-skills/     ตัวอ่าน SKILL.md แบบโหลด frontmatter อย่างเดียว
  kaam-mcp/        MCP client (HTTP+SSE) + circuit breaker
  kaam-chat/       Telegram + allowlist + ตัวตัดประโยคสำหรับ TTS
firmware/kaam-fw/  ที่เดียวที่รู้จักฮาร์ดแวร์ (อยู่นอก workspace โดยตั้งใจ)
docs/              ARCHITECTURE.md คือแหล่งความจริง
seed/              ไฟล์ตั้งต้นที่จะ flash ลง LittleFS
```

## วิธีเขียนเทสต์ในโปรเจกต์นี้

เทสต์ที่มีค่าที่สุดคือเทสต์ที่จำลองสิ่งที่เกิดยากบนชิป:

- **chunk ขาดตรงไหนก็ได้** — `survives_split_anywhere` วนทดสอบทุกจุดตัด
- **ขาดกลางอักขระ UTF-8** — สำคัญมากกับภาษาไทย
- **ไฟดับกลางเขียน** — `recovers_from_torn_final_line`
- **หน่วยความจำต่ำ** — ส่งค่า `free_internal` จำลองเข้าไปตรง ๆ
- **การพยายามข้ามขอบเขต** — `blocks_traversal_in_any_position`

ตั้งชื่อเทสต์เป็นประโยคที่บอกพฤติกรรม ไม่ใช่ชื่อฟังก์ชันที่ทดสอบ

## สิ่งที่ห้ามทำ

- อย่าเพิ่ม async runtime ตอนนี้ — ESP-IDF ให้ pthreads มา ใช้ thread ปกติก่อน
- อย่าใช้ `unwrap()` ในโค้ดที่จะรันบนชิป
- อย่าเพิ่ม dependency ที่ไม่จำเป็น ทุกตัวกินพื้นที่ใน partition 3.5 MB
- อย่าเก็บไฟล์เสียงลงแฟลช — stream ขึ้น STT ระหว่างที่ยังกดปุ่มอยู่
- อย่าใช้ vector embedding — ใช้ grep + read ตามแนวทาง LLM-wiki

## ลำดับงาน

ดู `docs/ROADMAP.md` — สรุป: **M1 กับ M2 จะกินเวลามากกว่าที่คิด 2–3 เท่า**
เพราะปัญหา heap และ TLS ทั้งหมดจะโผล่ตรงนั้น ถ้าผ่านสองด่านนี้ได้
ที่เหลือเป็นงานเชิงตรรกะซึ่งทดสอบบนพีซีได้เกือบหมด

## ของที่ยังไม่ได้ยืนยัน

`firmware/` ทั้งโฟลเดอร์ยังไม่เคยคอมไพล์จริง สิ่งที่ต้องตรวจใน M0:

1. ชื่อ target ของ C5 — ตั้งไว้เป็น `riscv32imac-esp-espidf` ตาม C6 แต่ต้องยืนยัน
2. เวอร์ชัน ESP-IDF ที่รองรับ C5 — ตั้งไว้ v5.5
3. เวอร์ชันของ `esp-idf-svc` / `esp-idf-hal` ที่เข้ากับ IDF นั้น
4. ว่ายังต้องใช้ nightly เพราะ `build-std` หรือ stable พอแล้ว
