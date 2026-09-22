# ktunBot → Teloxide (Rust) Rewrite Plan (Revised)

> Revised via brainstorming self-review on 2026-07-10. Supersedes the first plan
> discussed in chat. This document is the source of truth for execution.

## Understanding Summary

- **What:** Rewrite the ktunBot Telegram bot from Python (`python-telegram-bot`
  + Flask) to Rust using the **Teloxide 0.17** framework.
- **Why:** Fix latent bugs (event-loop-bound session crash under Flask webhook,
  naive-UTC timezone wrong-day bug, async-blocking file reads, swallowed errors,
  FakeMessage/FakeUpdate callback shim), drop PythonAnywhere (no Rust support),
  and align with the planned `ktunPlatform` Turso-based data system so the bot
  lifts into the platform workspace later with near-zero cost.
- **Who:** Konya Teknik Üniversitesi students; the bot serves announcements,
  schedule/calendar PDFs, and cafeteria menus.
- **Constraints:** Runs on a single low-cost VPS via long-polling + systemd.
  Existing `data/` files (PDFs, `2025-12.json`, `menu_2026-01.jpg`) MUST work
  unchanged. Europe/Istanbul timezone for date math. Turkish user-facing copy
  preserved.
- **Explicit non-goals:** Webhook mode (can be added later behind a feature
  flag), dialogue/FSM state, RAG/AI chat, automatic scrapers, rewriting
  `update_files.py` (kept as-is), multi-crate workspace now (single crate,
  structured for easy extraction).

## Assumptions

- Rust toolchain ≥1.85 is available on the deploy host (dev machine has 1.96).
- A single Telegram bot token is provided via env (`TELEGRAM_BOT_TOKEN`).
- TursoDB is NOT yet provisioned; the bot runs against a **local libSQL file**
  (`file:./data/ktunbot.db`) today and switches to remote/embedded-replica later
  by changing env vars only.
- `update_files.py` is run manually on a machine with Python; the Rust bot
  itself has zero Python dependency.
- The university announcements page keeps its current `tbody tr` table layout;
  the parser degrades gracefully if selectors miss.
- PythonAnywhere / Flask / webhook deployment is retired; polling replaces it.
- A systemd-managed VPS captures logs via journald (stdout), so no rotating
  file logger is needed.

## Decision Log

| # | Decision | Alternatives considered | Rationale |
|---|----------|------------------------|-----------|
| D1 | Teloxide 0.17 long-polling, single Tokio runtime | axum webhooks; keep Python | User chose VPS/polling; polling needs no public HTTPS/TLS and no Flask event-loop wart. |
| D2 | Full Python removal; keep `data/` + `update_files.py` | keep Python alongside; rewrite updater too | User chose full replace, keep data files. |
| D3 | `chrono-tz Europe::Istanbul` for all date math | system local time | Fixes the UTC-server wrong-day bug; Turkey is UTC+3 with no DST since 2016. |
| D4 | moka L1 + Turso/libSQL L2 (local file now, remote-ready) | moka only; SQLite only | User wants unified Turso layer across `ktun*` projects; L1 keeps latency identical to today, L2 survives restarts and is platform-shareable. |
| D5 | Drop calendar term keyboard; add 🍽 Menü → `[Dün][Bugün][Yarın]` row | wire up term selector; leave dead branches | User confirmed; only one calendar PDF today; makes `/dun` & `/yarin` reachable via buttons. |
| D6 | `Option<String>` for `/duyurular [n]` + manual clamp 1..=50 | `Option<u8>`; `Option<u32>` | u8 rejects `/duyurular 999`; manual parse preserves Python's "non-numeric → 10" behavior exactly. |
| D7 | Menu source priority: "JSON if it exists for the month, else image" | mtime comparison; always image | Deterministic; preserves 100% of current behavior (Dec→JSON, Jan→image); aligns with RAG-future (JSON is the structured target). |
| D8 | Day buttons answer callback + send a NEW menu message | edit the existing message | Editing is messy when today=image (photo) vs tomorrow=text. New message is uniform and simple. |
| D9 | `tracing` → stdout (journald), drop rotating file logger | keep file logger | Idiomatic for systemd; less code, less disk I/O. |
| D10 | `bot.set_my_commands(Command::descriptions())` at startup | omit | Telegram command menu auto-shows Turkish commands; better UX. |
| D11 | Inject `Arc<Services>` via `dptree::deps!` (teloxide idiom) | module-level globals | Cleaner than the Python singletons; testable. |
| D12 | `libsql::Builder::new_local` when URL starts with `file:`; tiny migration runner in `db/mod.rs` | sqlx migrations | libsql crate has no sqlx-style migrator; tiny runner is trivial and dependency-light. |
| D13 | Single Cargo crate (not a workspace) structured so `db/turso.rs` mirrors the `ktunPlatform/crates/indexer-service/src/storage/turso.rs` interface | workspace now | YAGNI; easy to fold later. |

## Risks & Mitigations

| Risk | Likelihood | Mitigation |
|------|-----------|------------|
| Teloxide command parser + Turkish lowercase rename_rule edge cases | Low | `rename_rule="lowercase"` maps ASCII variant names; Telegram command matching is case-insensitive. Verified against Teloxide example. |
| CSS selector for KTUN site drifts | Medium | Degrade gracefully: empty result → "❌ Duyuru bulunamadı." (matches current Python behavior). |
| `libsql` crate API churn (pre-1.0) | Medium | Pin a known-good version; wrap all DB calls behind `db/turso.rs` so a swap is one file. |
| Menu JSON shape changes | Low | `serde` structs tolerate missing fields via `#[serde(default)]`; verified against `2025-12.json`. |
| Build fails on VPS (glibc/libssl) | Low | `rustls-tls` feature avoids OpenSSL; `cargo build --release` on the VPS itself avoids ABI issues. |
| Existing users lose `/menu` muscle memory | Low | `/menu` is NOT a real command today (only a `help.py` typo); new `Command::descriptions()` is the source of truth. |

## Bug Fixes Baked Into The Rewrite

1. `help` text now auto-generated from the typed `Command` enum (no phantom `/menu`).
2. All file reads via `tokio::fs` (no async-blocking `open()`).
3. No Flask / no per-request `asyncio.run()` → no loop-bound session crash.
4. Europe/Istanbul timezone for today/yesterday/tomorrow.
5. `teloxide::utils::html::escape` on scraped titles before building `<a href>`.
6. Dead `calendar_fall/_spring` removed; `cafeteria_yesterday/_tomorrow` now reachable via the new day-row.
7. Single `Config.menus_dir`; deterministic JSON-or-image priority (D7).
8. `tracing::error!` with error chain on all error paths (no swallowed `e`).
9. `moka::future::Cache` (lock-free) + Turso for persistence (replaces `threading::Lock`-in-async).
10. Shared `(&Bot, ChatId)` logic fns called by both commands + callbacks (kills the `FakeMessage`/`FakeUpdate` shim).

## Revised Design

### Dependencies (`Cargo.toml`)

```toml
[package]
name = "ktunbot"
version = "2.0.0"
edition = "2021"

[dependencies]
teloxide = { version = "0.17", features = ["macros"] }
tokio = { version = "1", features = ["rt-multi-thread", "macros", "signal", "fs"] }
reqwest = { version = "0.12", default-features = false, features = ["rustls-tls"] }
scraper = "0.22"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
libsql = "0.9"
moka = { version = "0.12", features = ["future"] }
chrono = { version = "0.4", features = ["clock"] }
chrono-tz = "0.10"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
dotenvy = "0.15"
thiserror = "2"
anyhow = "1"

[profile.release]
lto = "thin"
```

### File Layout

```
ktunBot/
├── Cargo.toml                 # new
├── .env.example               # new
├── src/
│   ├── main.rs                # tokio + Dispatcher + ctrlc + set_my_commands
│   ├── config.rs              # dotenvy + env
│   ├── error.rs               # thiserror::Error
│   ├── services.rs            # Arc<Services> container (moka + db + config)
│   ├── bot.rs                 # dptree branches: messages → Command, callbacks → callbacks::handle
│   ├── commands.rs            # #[derive(BotCommands)] enum Command
│   ├── callbacks.rs           # CallbackQuery dispatcher → shared logic fns
│   ├── keyboards.rs            # main menu + menu day-row
│   ├── handlers/
│   │   ├── mod.rs
│   │   ├── announcements.rs   # send_announcements(&Bot, ChatId, count)
│   │   ├── schedule.rs        # send_schedule(&Bot, ChatId)
│   │   ├── calendar.rs        # send_calendar(&Bot, ChatId)
│   │   ├── cafeteria.rs       # send_menu(&Bot, ChatId, date) + day-row
│   │   ├── about.rs
│   │   └── start.rs
│   ├── services/
│   │   ├── mod.rs
│   │   ├── announcements.rs   # reqwest + scraper crate
│   │   ├── cafeteria.rs        # serde model + JSON-or-image (D7)
│   │   ├── schedule.rs
│   │   ├── calendar.rs
│   │   └── semester.rs
│   ├── db/
│   │   ├── mod.rs             # pool init + tiny migration runner
│   │   └── turso.rs            # libSQL client (interface-shaped to ktunPlatform)
│   ├── cache.rs               # moka L1 wrappers (announcements 900s, menus 7200s)
│   ├── scraper.rs             # reqwest fetch + scraper HTML parse
│   ├── formatter.rs           # html escape + food-emoji table
│   └── tz.rs                  # Europe/Istanbul now/yesterday/tomorrow helpers
├── migrations/
│   └── 0001_init.sql          # cache_blob(key, value, expires_at) + bot_state
├── tests/
│   ├── semester_test.rs
│   ├── formatter_test.rs
│   ├── cafeteria_test.rs      # loads data/menus/2025-12.json fixture
│   └── commands_test.rs       # count clamp
├── deploy/
│   ├── ktunbot.service        # systemd unit
│   └── README.deploy.md
├── data/                      # UNCHANGED
├── update_files.py            # KEPT as-is
└── README.md                  # updated for Rust
```

### Command Enum (single source of truth)

```rust
#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "ktünBot komutları:")]
enum Command {
    #[command(description = "Botu başlatır.")]
    Start,
    #[command(description = "Güncel duyuruları listeler. /duyurular [sayı]")]
    Duyurular(Option<String>),   // manual clamp 1..=50, default 10
    #[command(description = "Ders programını gönderir.")]
    Program,
    #[command(description = "Akademik takvimi gönderir.")]
    Takvim,
    #[command(description = "Bugünün yemekhane menüsü.")]
    Bugun,
    #[command(description = "Dünün yemekhane menüsü.")]
    Dun,
    #[command(description = "Yarının yemekhane menüsü.")]
    Yarin,
    #[command(description = "Bot hakkında.")]
    Hakkinda,
    #[command(description = "Bu yardım metni.")]
    Yardim,
}
```

### Dispatch Shape (bot.rs)

- `Update::filter_message()` → `Command::filter()` → branch per variant → handler
  receives `(Bot, Message, Command, Arc<Services>)`.
- `Update::filter_callback_query()` → `callbacks::handle` `match` on data:
  - `announcements` → `send_announcements`
  - `schedule` → `send_schedule`
  - `calendar` → `send_calendar`
  - `cafeteria` → `send_menu(today)` + send day-row keyboard
  - `cafeteria_yesterday` / `cafeteria_tomorrow` → answer query + `send_menu(date)`
  - `about` → `send_about`
- All shared logic fns take `(&Bot, ChatId, ...)` — no fake-update shim.

### Menu Source Priority (D7)

For a given date `d`:
1. If `data/menus/{d.year}-{d.month:02}.json` exists → load JSON, find entry by
   `"DD MonthTR"` key (e.g. `"01 Aralık"`).
2. Else if a `menu_{year}-{month:02}.{jpg,jpeg,png}` exists → send image.
3. Else → `None` (no menu).

### Turso Local Mode (D12)

`Config::turso_db`:
- Starts with `file:` → `libsql::Builder::new_local(path.strip_prefix("file:"))`.
- Else → `libsql::Builder::new_remote(url, token)` (token optional).
- `db/mod.rs` reads `migrations/*.sql` on boot, idempotently applies via
  `CREATE TABLE IF NOT EXISTS`.

## Execution Checklist

- [ ] **1.** Scaffold `Cargo.toml`, `.env.example`, `src/main.rs` skeleton.
- [ ] **2.** `config.rs` + `error.rs` + `services.rs` container.
- [ ] **3.** `db/mod.rs` + `db/turso.rs` + `migrations/0001_init.sql` + `cache.rs` (moka L1).
- [ ] **4.** `tz.rs` + `services/semester.rs` + tests.
- [ ] **5.** `services/schedule.rs` + `services/calendar.rs`.
- [ ] **6.** `services/cafeteria.rs` + `tests/cafeteria_test.rs` (real `2025-12.json`).
- [ ] **7.** `services/announcements.rs` + `scraper.rs`.
- [ ] **8.** `formatter.rs` + `tests/formatter_test.rs`.
- [ ] **9.** `keyboards.rs` (main menu + menu day-row).
- [ ] **10.** `commands.rs` + `tests/commands_test.rs` (count clamp).
- [ ] **11.** `handlers/*` (shared logic fns).
- [ ] **12.** `callbacks.rs`.
- [ ] **13.** `bot.rs` + `main.rs` (dispatcher, deps, set_my_commands, ctrlc).
- [ ] **14.** `deploy/ktunbot.service` + `deploy/README.deploy.md`.
- [ ] **15.** Update `README.md`; remove Python files (`bot/`, `config.py`, `flask_app.py`, `run_polling.py`, `requirements.txt`, `package-lock.json`, `__pycache__/`).
- [ ] **16.** Verification: `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test`, then `cargo build --release`.
- [ ] **17.** Manual smoke test against real Telegram using existing `data/` files.