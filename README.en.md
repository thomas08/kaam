> อ่านภาษาไทย: [README.md](README.md)

# Kaam (ก้าม)

A self-contained AI agent harness that runs entirely on an ESP32-C5 — a chip that costs a few dollars.

No Linux, no Node.js, no server. Rust compiled straight to bare metal. Talk to it over
Telegram or hold a button and speak. It remembers across reboots and can schedule its own
recurring work.

```
your machine. your agent. ก้ามเดียวก็พอ.
```

*(ก้าม — "kaam" — is Thai for a crab's claw. The tagline reads "one claw is enough.")*

## Status

**M0** — the skeleton is in place and the host test suite passes. Firmware is still a stub.
See [docs/ROADMAP.md](docs/ROADMAP.md).

| | |
|---|---|
| Host tests | 66 passing, no board required |
| CI | `fmt` + `clippy -D warnings` + tests, green |
| Firmware | compiles for `esp32c5`; nothing is wired up yet |
| Verified on hardware | not yet — no board has run this |

## Why it might interest you

- **Actually yours.** Code, conversations, and memory live on your own flash. No telemetry.
- **Readable and editable by hand.** All state is Markdown. Pull the flash, mount it, edit it.
- **Not tied to a vendor.** Switch between Anthropic and OpenAI at runtime.
- **Open standards.** MCP, `AGENTS.md`, `SKILL.md`.
- **Extend without reflashing.** Adding a skill means writing a file.

## Hardware

ESP32-C5-WROOM-1-**N16R8** module (16 MB flash, 8 MB PSRAM).
Voice is optional: an I2S microphone, a MAX98357A amplifier, and one button.

## Design constraints

These are contracts, not aspirations — every milestone measures against them:

| Budget | Limit |
|---|---|
| Free internal SRAM | ≥ 45 KB (a new turn is refused below this) |
| Largest free block | ≥ 32 KB |
| Free PSRAM | ≥ 2 MB |
| Binary size | ≤ 2.8 MB (over budget is a build failure) |
| Stack high-water mark | ≥ 20% headroom on every task |

The chip is single-core at 240 MHz, so exactly one turn runs at a time and nothing is
allowed to busy-wait. Both request and response are streamed — buffering a whole
conversation into RAM before sending it does not fit.

## Getting started

```bash
cargo test --workspace       # all logic, on your PC, no board needed
```

`crates/` is deliberately free of any hardware dependency — no `esp-idf-*`, and CI enforces
it. That is what makes the test suite above meaningful rather than decorative.

Building the firmware additionally needs ESP-IDF v5.5 and the RISC-V toolchain:

```bash
cd firmware/kaam-fw
cargo build --release
```

Note that ESP32-C5 is a **preview target** in ESP-IDF v5.5 (`PREVIEW_TARGETS`, not
`SUPPORTED_TARGETS`). It builds, but the platform support underneath is not yet stable.

Full architecture: [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md).
Rules for contributors (human and AI): [CLAUDE.md](CLAUDE.md) — written in Thai.

## Repository layout

```
crates/            pure logic, tested on host — no ESP-IDF allowed here
  kaam-types/        shared types and turn budgets
  kaam-provider/     sans-io: bytes in, events out (SSE decoder lives here)
  kaam-agent/        turn state machine + context manager
  kaam-tools/        tool registry, permission policy, SOUL.md edit rules
  kaam-store/        storage trait, path guard, in-memory store for tests
  kaam-skills/       SKILL.md reader (frontmatter only until triggered)
  kaam-mcp/          MCP client groundwork + circuit breaker
  kaam-chat/         Telegram allowlist + sentence splitter for TTS
firmware/kaam-fw/  the only place that knows about hardware
docs/              ARCHITECTURE.md is the source of truth
seed/              initial files flashed onto LittleFS
```

## A note on language

This project is written in Thai — documentation, code comments, and commit messages.
This English README exists so the project is legible to people outside that circle, but
the design document that actually governs the code is [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md),
and it is in Thai. Translating it faithfully is on the list; it is not done yet.

## Credits

Inspired by [thClaws](https://github.com/thClaws/thClaws) (harness architecture, `SKILL.md`,
MCP) and [MimiClaw](https://github.com/memovai/mimiclaw) (the bare-chip agent idea,
Markdown as memory). What Kaam changes from each is tabulated in Appendix A of the
architecture document.

## License

MIT or Apache-2.0, at your option.
