# Kaam (ก้าม)

AI agent harness ที่รันเองได้ครบบนชิป ESP32-C5 ราคาไม่กี่ร้อยบาท

ไม่มี Linux ไม่มี Node.js ไม่ต้องมีเซิร์ฟเวอร์ — Rust คอมไพล์ลง bare metal
ทักผ่าน Telegram หรือกดปุ่มพูด แล้วมันจัดการให้ จำเรื่องได้ข้ามการรีบูต
และตั้งงานประจำเองได้

```
your machine. your agent. ก้ามเดียวก็พอ.
```

## สถานะ

**M0** — โครงพร้อม เทสต์บน host ผ่าน firmware ยังเป็นโครงเปล่า
ดู [docs/ROADMAP.md](docs/ROADMAP.md)

## ทำไมถึงน่าสนใจ

- **ของคุณจริง ๆ** — โค้ด บทสนทนา ความจำ อยู่บนแฟลชของคุณเอง ไม่มี telemetry
- **อ่านและแก้ได้ด้วยมือ** — state ทุกอย่างเป็น Markdown ถอดแฟลชมา mount แล้วแก้ได้เลย
- **ไม่ผูกกับผู้ให้บริการ** — สลับ Anthropic กับ OpenAI ได้ตอนรันไทม์
- **มาตรฐานเปิด** — MCP, `AGENTS.md`, `SKILL.md`
- **ต่อยอดได้โดยไม่ต้อง reflash** — เพิ่ม skill ด้วยการเขียนไฟล์

## ฮาร์ดแวร์

โมดูล ESP32-C5-WROOM-1-**N16R8** (แฟลช 16 MB + PSRAM 8 MB)
เสียงเป็นตัวเลือกเสริม: ไมค์ I2S + MAX98357A + ปุ่มหนึ่งตัว

## เริ่มพัฒนา

```bash
cargo test --workspace          # ทดสอบ logic ทั้งหมด ไม่ต้องมีบอร์ด
```

สถาปัตยกรรมทั้งหมดอยู่ใน [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md)
กฎสำหรับผู้ร่วมพัฒนา (และ AI) อยู่ใน [CLAUDE.md](CLAUDE.md)

## ที่มา

ได้แรงบันดาลใจจาก [thClaws](https://github.com/thClaws/thClaws) (สถาปัตยกรรม harness,
`SKILL.md`, MCP) และ [MimiClaw](https://github.com/memovai/mimiclaw)
(แนวคิด agent บนชิปเปล่า, ความจำเป็น Markdown)

## ใบอนุญาต

MIT หรือ Apache-2.0 แล้วแต่คุณเลือก
