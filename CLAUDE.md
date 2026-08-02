# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

Kaam (ก้าม) คือ AI agent harness ที่รันครบวงจรบน ESP32-C5 เขียนด้วย Rust
รับคำสั่งผ่าน Telegram และเสียง (push-to-talk) เรียก LLM ผ่านคลาวด์ จำเรื่องข้ามรีบูตได้

**อ่าน `docs/ARCHITECTURE.md` ก่อนเริ่มงานใด ๆ** เอกสารนั้นคือแหล่งความจริง
ไฟล์นี้เป็นบทสรุปของกฎที่ต้องเคารพ คำสั่งที่ใช้บ่อย และสิ่งที่ต้องอ่านหลายไฟล์ถึงจะเข้าใจ

สถานะ: **M0** — `crates/` ครบและเทสต์ผ่าน 66 ตัวบน host, `firmware/` ยังเป็นโครงเปล่าที่ไม่เคยคอมไพล์จริง

---

## คำสั่ง

```bash
# ทดสอบ logic ทั้งหมด — บนพีซี ไม่ต้องมีบอร์ด ไม่ต้องมี ESP-IDF
cargo test --workspace

# เทสต์ตัวเดียว (ตัวกรองคือชื่อ path เต็มของโมดูล)
cargo test -p kaam-provider sse::tests::survives_split_anywhere
cargo test -p kaam-store guard::            # ทั้งโมดูล

# ตรวจสไตล์ — ต้องผ่านทั้งสองก่อน push (CI รัน -D warnings)
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all --check

# กฎข้อ 1: crates/ ห้ามแตะ ESP-IDF
./scripts/check-no-esp-idf.sh

# ---- ต้องมี ESP-IDF v5.5 + toolchain ของ RISC-V ----
cd firmware/kaam-fw
cargo build --release              # ต้องดูขนาด binary ด้วย เพดาน 2.8 MB
cargo run --release                # flash + monitor ผ่าน espflash (runner ตั้งไว้ใน .cargo/config.toml)
cargo build --release --features voice
```

`cargo test --workspace` และ `cargo fmt --all` **ไม่แตะ `firmware/`** เพราะมันถูก `exclude` จาก workspace
ตั้งใจให้เป็นแบบนี้ — ถ้าแก้ firmware ต้องเข้าไปรันคำสั่งในโฟลเดอร์นั้นเอง

---

## สถาปัตยกรรม

### เส้นแบ่งเดียวที่สำคัญที่สุด

โปรเจกต์นี้ถูกออกแบบรอบเส้นแบ่งเดียว: **logic ที่ทดสอบบนพีซีได้** กับ **โค้ดที่ต้องรันบนชิป**

```
crates/           sans-io ล้วน — ไม่มี dependency ภายนอกแม้แต่ตัวเดียว (ยังไม่มีแม้แต่ serde)
  kaam-types/       Message/ContentBlock/Budget — รากของ dependency graph ห้าม depend อะไรทั้งสิ้น
  kaam-provider/    byte เข้า → ProviderEvent ออก (SseDecoder อยู่นี่) ไม่รู้จัก socket/TLS
  kaam-agent/       TurnState machine + ContextManager (ตัดสินใจว่าเมื่อไหร่ต้อง compact)
  kaam-tools/       Registry + Policy + กลไก validate การแก้ SOUL.md
  kaam-store/       trait Store + path guard + MemStore ที่ crate อื่นใช้ในเทสต์
  kaam-skills/      parser ของ SKILL.md แบบแยก header ออกจาก body
  kaam-mcp/         CircuitBreaker (ตัว client จริงยังไม่ได้เขียน)
  kaam-chat/        allowlist + SentenceSplitter สำหรับ TTS
firmware/kaam-fw/  ที่เดียวที่ `use esp_idf_*` ได้ — อยู่นอก workspace โดยตั้งใจ
seed/              ไฟล์ตั้งต้นที่จะ flash ลง LittleFS (SOUL.md, settings.json, skill ตัวอย่าง)
```

ผลจากการเป็น sans-io ที่ต้องเข้าใจตอนเขียนโค้ดใหม่: **สถานะที่มาจากฮาร์ดแวร์ถูกส่งเข้ามาเป็น parameter เสมอ**
เช่น `ContextManager::should_compact(msgs, free_internal)` รับตัวเลข heap เข้ามาตรง ๆ
แทนที่จะไปเรียก `esp_get_free_internal_heap_size()` เอง — นี่คือสิ่งที่ทำให้เทสต์ low-memory เขียนได้บนพีซี
โค้ดใหม่ต้องรักษารูปแบบนี้ ถ้าอยากรู้เวลา/heap/uptime ให้รับเข้ามา ไม่ใช่ไปหาเอง

### เรื่องที่ต้องอ่านหลายไฟล์ถึงจะเห็น

- **`SseDecoder` แบ่งบรรทัดที่ระดับไบต์ ไม่ใช่ char** — ปลอดภัยเพราะ `\n` (0x0A) ไม่มีวันเป็นส่วนหนึ่ง
  ของอักขระ UTF-8 หลายไบต์ นี่คือเหตุผลที่ chunk ขาดกลางคำไทยแล้ว decoder ไม่พัง อย่าเผลอเปลี่ยนไปใช้ `str`

- **`Store` ไม่บังคับ deny-list ให้** — `MemStore::write` เรียกแค่ `sandbox_check` (กัน `..` และนอก `/kaam/`)
  ส่วน `guard::write_check` ที่กัน `identity/` กับ `config/` เป็นฟังก์ชันแยกที่ **ชั้น tool ต้องเรียกเอง**
  ตอนต่อ `write_file` จริงต้องเรียก `write_check` ไม่ใช่พึ่ง `Store` ไม่งั้นกฎข้อ 6 หลุดเงียบ ๆ

- **`Registry::policy_for` hard-code ให้ `propose_soul_edit` เป็น `Confirm` เสมอ** ก่อนดู override
  ตั้งใจให้ปิดไม่ได้ ต่อให้ `settings.json` สั่ง `allow` (มีเทสต์ล็อกไว้)

- **tool ที่ไม่รู้จัก = `Deny`** ไม่ใช่ `Allow` — `Registry` ที่ว่างเปล่าปฏิเสธทุกอย่าง

- **`ToolCall.arguments` เป็น `String` ดิบ ไม่ใช่ค่าที่ parse แล้ว** เพราะมันไหลมาทีละชิ้นจาก stream
  ทั้ง repo ยังไม่มี JSON library ที่ `crates/` — การเลือก parser ถือเป็นการตัดสินใจที่ยังไม่ได้ทำ

- **Source ทั้งสามทาง (Telegram / Voice / Scheduler) เดินผ่าน code path เดียวกัน** เข้า `inbox_q` เหมือนกันหมด
  งาน cron ที่ถึงเวลาถูกโยนเข้าคิวเหมือนข้อความจากคน อย่าสร้าง path แยก

- **`firmware/kaam-fw/src/tasks.rs::TASKS` ต้องตรงกับตารางใน ARCHITECTURE.md §5.2 เสมอ**
  แก้ที่หนึ่งต้องแก้อีกที่ เช่นเดียวกับ `partitions.csv` (§6) และ `sdkconfig.defaults` (§3)

- **`[profile.release]` ของ firmware ต้องมิเรอร์ของ workspace** — `kaam-fw` อยู่นอก
  workspace จึงไม่สืบทอด profile ถ้าสองที่ไม่ตรงกัน เลขคณิตชุดเดียวกันจะมีพฤติกรรม
  ต่างกันระหว่างที่ทดสอบกับที่รันจริง ซึ่งลบล้างเหตุผลทั้งหมดของกฎข้อ 1
  (ตอนนี้ตรงกันแล้วที่ `overflow-checks = true` ทั้งสองฝั่ง)

---

## กฎเหล็ก

กฎพวกนี้ไม่ใช่คำแนะนำ ถ้าจะละเมิดต้องถามเจ้าของโปรเจกต์ก่อน

1. **`crates/` ห้ามแตะ ESP-IDF** — ห้าม `esp-idf-*`, `embuild`, `esp-hal` ทั้งใน manifest และใน `use`
   บังคับด้วย `scripts/check-no-esp-idf.sh` ใน CI เหตุผลคือทดสอบ logic บนพีซีได้ 85–90%

2. **Stream ทั้งขาส่งและขารับ** — ห้าม buffer ทั้ง request หรือทั้ง response ไว้ใน RAM
   ขาส่งมักถูกลืม: conversation 60 KB กลายเป็น JSON 80 KB ที่ต้องต่อเนื่องกันใน heap ที่ fragment แล้ว
   ใช้ HTTP chunked encoding แล้วสร้าง JSON ทีละ message

3. **งบประมาณคือสัญญา** — free internal SRAM ≥ 45 KB, largest free block ≥ 32 KB, free PSRAM ≥ 2 MB,
   binary ≤ 2.8 MB (เกิน = build failure), stack high-water เหลือ ≥ 20% ทุก task
   ถ้าการเปลี่ยนแปลงทำให้ตัวเลขพวกนี้แย่ลง ต้องบอกในสรุปงานเสมอ

4. **มันคือ single core 240 MHz** — ห้าม busy-wait ต้อง block บน queue/semaphore, งานยาวต้องหั่นแล้ว yield,
   มีเทิร์นที่กำลังทำงานได้ทีละหนึ่งเท่านั้น

5. **ทุกอย่างเป็นไฟล์ที่มนุษย์อ่านได้** — Markdown/JSONL ใต้ `/kaam/` บน LittleFS
   เขียนแบบ atomic เสมอ (tmp → fsync → rename) session เป็น append-only JSONL เพื่อให้ไฟดับเสียแค่บรรทัดสุดท้าย

6. **ความปลอดภัยที่ห้ามอ่อนข้อ**
   - allowlist `chat_id` — ข้อความจาก id อื่นทิ้งเงียบ ๆ ไม่ตอบแม้แต่ error (การตอบ error ก็ยืนยันว่ามีบอทอยู่)
   - `write_file` ต้อง deny กับ `/kaam/identity/` และ `/kaam/config/` เสมอ
   - `SOUL.md` แก้ได้ทางเดียวคือ `propose_soul_edit` ที่มีคนอนุมัติ (§18)
   - บล็อก `kaam:immutable` เทียบ byte-for-byte ต่างแม้ตัวเดียวคือปฏิเสธ
   - `settings.json`, allowlist, เพดาน token แก้ได้ทาง serial console เท่านั้น
   - API key อยู่ใน NVS ที่เข้ารหัส ห้ามอยู่ในซอร์สหรือไฟล์ header และห้ามโผล่ใน log

---

## วิธีเขียนเทสต์ในโปรเจกต์นี้

เทสต์ที่มีค่าที่สุดคือเทสต์ที่จำลองสิ่งที่เกิดยากบนชิป — และเกือบทุกตัวในนี้เป็นแบบ exhaustive loop
ไม่ใช่ตัวอย่างเดียว:

- **chunk ขาดตรงไหนก็ได้** — `sse::tests::survives_split_anywhere` วนทุกจุดตัดตั้งแต่ไบต์ 1 ถึงท้าย
- **ขาดกลางอักขระ UTF-8** — `survives_split_mid_utf8_character` สำคัญมากกับภาษาไทย
- **ไฟดับกลางเขียน** — `recovers_from_torn_final_line`
- **หน่วยความจำต่ำ** — ส่งค่า `free_internal` จำลองเข้าไปตรง ๆ
- **การพยายามข้ามขอบเขต** — `blocks_traversal_in_any_position` + `does_not_overblock_dotted_filenames`

ตั้งชื่อเทสต์เป็นประโยคที่บอกพฤติกรรม ไม่ใช่ชื่อฟังก์ชันที่ทดสอบ
comment และข้อความ assert เขียนเป็นภาษาไทยตามโค้ดที่มีอยู่
เทสต์ที่ล็อก invariant ด้านความปลอดภัยให้เขียน doc comment บอกว่ามันกันอะไร

## สิ่งที่ห้ามทำ

- อย่าเพิ่ม async runtime ตอนนี้ — ESP-IDF ให้ pthreads มา ใช้ thread ปกติก่อน
- อย่าใช้ `unwrap()` ในโค้ดที่จะรันบนชิป
- อย่าเพิ่ม dependency ที่ไม่จำเป็น — `crates/` ตอนนี้มี **ศูนย์** dependency ภายนอก ทุกตัวที่เพิ่มกินพื้นที่ใน partition 3.5 MB
- อย่าเก็บไฟล์เสียงลงแฟลช — stream ขึ้น STT ระหว่างที่ยังกดปุ่มอยู่
- อย่าใช้ vector embedding — ใช้ grep + read
- อย่า commit `chat_id` จริงหรือ key ลง `seed/config/settings.json`

## ลำดับงานและของที่ยังไม่ยืนยัน

`docs/ROADMAP.md` มีเกณฑ์ผ่านของแต่ละ milestone — สรุป: **M1 กับ M2 จะกินเวลามากกว่าที่คิด 2–3 เท่า**
เพราะปัญหา heap และ TLS ทั้งหมดจะโผล่ตรงนั้น

`firmware/` ทั้งโฟลเดอร์ยังไม่เคยคอมไพล์จริง ค่าที่ยังเป็นการเดาและมี `TODO(M0)` กำกับไว้:

1. ชื่อ target ของ C5 — ตั้งไว้ `riscv32imac-esp-espidf` ตาม C6 (`firmware/kaam-fw/.cargo/config.toml`)
2. เวอร์ชัน ESP-IDF ที่รองรับ C5 — ตั้งไว้ v5.5
3. เวอร์ชัน `esp-idf-svc` 0.51 / `esp-idf-hal` 0.45 ที่เข้ากับ IDF นั้น
4. ว่ายังต้อง nightly เพราะ `build-std` หรือ stable พอแล้ว (`firmware/kaam-fw/rust-toolchain.toml`)
5. job `firmware` ใน CI ยังปิดอยู่ด้วย `if: false` — เปิดเมื่อยืนยันสี่ข้อบน
