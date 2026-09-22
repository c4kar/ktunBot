# ktunBot → Teloxide Rewrite — Session Handoff

> Created 2026-07-10 mid-execution. Read this + `docs/REWRITE_PLAN.md` to resume.

## TL;DR

Rewriting the KTÜN Telegram bot from Python (`python-telegram-bot` + Flask,
PythonAnywhere) to Rust **Teloxide 0.17** long-polling on a VPS. All user
decisions are LOCKED. Implementation is ~30% done. Resume from "Next Steps"
below.

## Locked Decisions (confirmed by user)

| Decision | Choice |
|---|---|
| Runtime | Rust/Teloxide 0.17, long-polling Dispatcher, single Tokio runtime, VPS+systemd |
| Scope | Full replace of Python; keep `data/` files + `update_files.py` as-is |
| Timezone | `chrono-tz Europe::Istanbul` for today/yesterday/tomorrow |
| Cache | `moka` L1 (announcements 900s, menus 7200s) + TursoDB/libSQL L2 |
| Turso mode | Local file `file:./data/ktunbot.db` now; remote-ready code (accepts `TURSO_DATABASE_URL` + optional `TURSO_AUTH_TOKEN`) |
| UI | Drop calendar term keyboard; add 🍽 Menü → `[Dün][Bugün][Yarın]` inline row |
| Cache strategy rationale | User plans to unify `ktun*` projects on TursoDB; moka keeps latency, Turso is the shared platform layer |
| Menu source priority | JSON if it exists for the month, else image (deterministic; aligns with RAG future) |
| Dead branches | Drop `calendar_fall/_spring`; make `cafeteria_yesterday/_tomorrow` reachable via buttons |

## Exit of brainstorming skill

I ran the `brainstorming` superpowers skill as a critical self-review of my
first plan. Surfaced 7 revisions (all adopted):

1. Count arg → `Option<String>` + manual clamp 1..=50 (u8 would reject `/duyurular 999`; preserves Python's "non-numeric → 10").
2. Image-vs-JSON priority → "JSON if exists for the month, else image" (deterministic; drops fragile mtime comparison).
3. Menu day buttons → tapping 🍽 Menü sends today's menu + day-row; day buttons answer + send a NEW message.
4. Logging → `tracing` → stdout (journald captures under systemd); drop file logger.
5. `bot.set_my_commands(Command::descriptions())` at startup so Telegram UI shows Turkish commands.
6. DI via `OnceLock<Arc<Services>>` + `dptree::deps!` (teloxide idiom) instead of globals.
7. Turso local mode: `libsql::Builder::new_local` when URL starts with `file:`; tiny migration runner in `db/mod.rs` (libsql has no sqlx-style migrations).

## Bugs In The Original Python Bot (being fixed by the rewrite)

1. `help.py` lists nonexistent `/menu` & omits `/start /yardim /hakkinda`.
2. `callback_handler.py:74` uses sync `open()` inside async fn (blocks event loop).
3. `flask_app.py:47` calls `asyncio.run()` per webhook → loop-bound aiohttp session crashes on 2nd+ request.
4. Naive `datetime.now()` → UTC server returns wrong day for today/yesterday/tomorrow menus.
5. Unescaped scraped announcement titles interpolated into HTML Telegram messages (`formatter.py:3`).
6. Dead code: `get_calendar_keyboard` imported but unused; `calendar_fall/_spring` callbacks not handled in `button_handler`; `cafeteria_yesterday/_tomorrow` callbacks unreachable.
7. `Config.MENUS_DIR` defined but `CafeteriaService` recomputes its own path; image-vs-JSON priority nondeterministic.
8. Swallowed errors in `schedule.py`/`calendar.py` (`except Exception as e:` then `e` never logged).
9. `threading.Lock` used in async cache (should be async lock).
10. `FakeMessage`/`FakeUpdate` hack in `callback_handler.py:90` to invoke command handlers from callbacks.

## Current Implementation State

### Files already written (verified on disk)

- `Cargo.toml` — teloxide 0.17, tokio, reqwest(rustls), scraper, serde, libsql 0.9, moka, chrono, chrono-tz, tracing, dotenvy, thiserror, anyhow. ✓
- `.env.example` — TELEGRAM_BOT_TOKEN, UNIVERSITY_ANNOUNCEMENTS_URL, TURSO_DATABASE_URL, TURSO_AUTH_TOKEN, RUST_LOG. ✓
- `docs/REWRITE_PLAN.md` — full revised plan with Decision Log, Risks, Assumptions, Execution Checklist. ✓ READ THIS.
- `src/config.rs` — `Config::from_env()` with telegram_bot_token, announcements_url, turso_database_url, turso_auth_token, base_dir, data_dir, schedules_dir, calendars_dir, menus_dir. ✓
- `src/error.rs` — `BotError` with Telegram/Request/Io/Json/Db/NotFound/Other variants, `Result<T>` alias. ✓
- `src/services.rs` — `Services { config: Arc<Config>, cache: AppCache, db: Arc<Db> }` (cloneable, injected via deps). ✓
- `migrations/0001_init.sql` — `cache_blob(key, value, expires_at)` + `bot_state(chat_id, key, value, updated_at)` + `_migrations` table. ✓
- `src/db/migrations.rs` — tiny migration runner: reads `migrations/*.sql`, applies via `execute_batch`, tracks in `_migrations`. Has `is_local_file()`, `local_path()`, `ensure_parent_dir()`. ✓
- `src/db/turso.rs` — `Db { conn: Arc<Connection> }` with `connect()`, `cache_get()`, `cache_set()`, `cache_evict()`. ⚠️ HAS A TYPO: line ~30 `b = b读ductor();` — DELETE THIS LINE, the remote branch should just be `Builder::new_remote(url, token)`. Interface mirrors `ktunPlatform/crates/indexer-service/src/storage/turso.rs`.

### Files NOT yet written (resume here — IN ORDER)

- `src/cache.rs` — `AppCache`: moka `Cache<String, Arc<Vec<u8>>>` with per-key TTL via `Expiry` impl or `insert_with_expiry`. Two named instances: announcements (900s), menus (7200s). Methods `get_announcements(n) -> Option<Vec<u8>>`, `set_announcements(n, bytes)`, `get_menu(month)`, `set_menu(month, bytes)`.
- `src/db/mod.rs` — `pub use turso::Db; pub mod migrations;`
- `src/tz.rs` — `NOW_ISTANBUL()`, `today()`, `yesterday()`, `tomorrow()` returning `chrono::DateTime<chrono_tz::Tz>` or `chrono::NaiveDate`. Use `chrono_tz::Europe::Istanbul`. Turkish month names map for menu JSON keys (`01 Aralık` etc — map month int→TR string).
- `src/services/mod.rs` — re-exports.
- `src/services/semester.rs` — `current_semester() -> String`. Logic: Jan→"{y-1}-{y} Güz", Feb-Jun→"{y-1}-{y} Bahar", Jul-Aug→"{y} Yaz", Sep-Dec→"{y}-{y+1} Güz". Write tests for Jan/Feb/Jul/Sep boundaries.
- `src/services/schedule.rs` — path = `config.schedules_dir/ders_programi.pdf`; semester via semester::current_semester().
- `src/services/calendar.rs` — path = `config.calendars_dir/akademik_takvim.pdf`.
- `src/services/cafeteria.rs` — serde structs matching `data/menus/2025-12.json` EXACTLY: `{ year: u32, month: String, generatedAt: String, menu: [MenuItem] }` where MenuItem = `{ date: String, dayOfWeek: String, mealType: String, foods: Vec<String>, totalcalorie: String, #[serde(default)] closed: bool, #[serde(default)] closedReason: String }`. Priority: JSON if `menus/{y}-{m:02}.json` exists, else image if `menu_{y}-{m:02}.{jpg,jpeg,png}` exists. Date key format `"DD MonthTR"` e.g. `"01 Aralık"`. `get_available_months()` lists `*.json` files. All date math uses `src/tz.rs`.
- `src/services/announcements.rs` — reqwest GET of announcements_url, `scraper` crate parse `tbody tr`, for each row extract title (`a.text`), href, date cell. Build full URL `https://www.ktun.edu.tr{href}` if relative. Return `Vec<Announcement{title, date, link}>`.
- `src/scraper.rs` — thin reqwest + scraper wrapper (session with custom UA "KTUN-Bot/2.0", 15s timeout).
- `src/formatter.rs` — `format_announcement(ann)` using `teloxide::utils::html::escape` on title; `format_menu(menu, date_str)` with closed-branch, mealType emoji, foods with `get_food_emoji(food)`. Port `get_food_emoji` verbatim from `bot/utils/formatter.py:60-104` (soups🍲, pilav🍚, makarna🍝, meat🍖, veg🥗, dessert🍰, dairy🥛, salad🥬, fruit🍎, drink🥤, default•).
- `src/keyboards.rs` — `main_menu()`: [[📢Duyurular,📚Program],[📅Takvim,🍽Menü],[ℹ️Hakkında]] with callback_data "announcements"/"schedule"/"calendar"/"cafeteria"/"about". `menu_day_row()`: [[Dün,Bugün,Yarın]] data "cafeteria_yesterday"/"cafeteria"/"cafeteria_tomorrow". NO calendar term keyboard (dropped).
- `src/commands.rs` — `#[derive(BotCommands, Clone)]` enum Command with `#[command(rename_rule="lowercase", description="ktünBot komutları:")]`. Variants: Start, Duyurular(Option<String>), Program, Takvim, Bugun, Dun, Yarin, Hakkinda, Yardim. Each with Turkish `#[command(description=...)]`.
- `tests/commands_test.rs` — `clamp_count: Option<String> -> usize`: None→10, parse ok clamp 1..=50, parse fail→10.
- `tests/semester_test.rs` — Jan/Feb/Jul/Sep boundary cases.
- `tests/formatter_test.rs` — closed-menu branch, open-menu with foods, food-emoji matching (çorba🍲, köfte🍖, etc.), HTML escaping of title with `<`/`&`.
- `tests/cafeteria_test.rs` — load real `data/menus/2025-12.json`, find "01 Aralık" → Mercimek Çorbası etc., closed:false. Test date-key format `"01 Aralık"`.
- `src/handlers/mod.rs` + `src/handlers/{start,announcements,schedule,calendar,cafeteria,about}.rs` — shared logic fns taking `(&Bot, ChatId, &Services)`: `send_announcements(bot, chat, svc, count)`, `send_schedule(bot, chat, svc)`, `send_calendar(bot, chat, svc)`, `send_menu(bot, chat, svc, date)` (sends today + day-row keyboard when called without explicit date intent; for yesterday/tomorrow just send menu), `send_about(bot, chat)`, `send_welcome(bot, chat, user)` with main menu, `send_help(bot, chat)` = `Command::descriptions().to_string()`. All use `tokio::fs::read` for PDFs/images. All errors: `tracing::error!` + reply "❌ Hata oluştu.".
- `src/callbacks.rs` — `async fn handle(bot, q, svc)`: `q.answer()`, match `q.data`: announcements→send_announcements, schedule→send_schedule, calendar→send_calendar, cafeteria→send_menu(today) (with day-row), cafeteria_yesterday→send_menu(yesterday), cafeteria_tomorrow→send_menu(tomorrow), about→send_about.
- `src/bot.rs` — `pub fn build_dispatcher(bot, svc) -> Dispatcher`: dependencies `dptree::deps![svc]`; branch `Update::filter_message().filter_command::<Command>().branch(dptree::entry().endpoint(commands_router))`; branch `Update::filter_callback_query().endpoint(callbacks::handle)`.
- `src/main.rs` — `#[tokio::main]`, `dotenvy::dotenv()`, `tracing_subscriber::fmt().with_env_filter(RUST_LOG).init()`, `Config::from_env()`, init moka cache, `Db::connect(&config).await?`, `Services::new(...)`, `Bot::from_env()` (requires TELEGRAM_BOT_TOKEN), `bot.set_my_commands(Command::descriptions()).await?`, `build_dispatcher(bot, svc).dispatch().await` with `.enable_ctrlc_handler()`.
- `deploy/ktunbot.service` — systemd unit: `[Service] ExecStart=/opt/ktunbot/target/release/ktunbot EnvironmentFile=/etc/ktunbot/ktunbot.env Restart=on-failure User=ktunbot WorkingDirectory=/opt/ktunbot`, `[Install] WantedBy=multi-user.target`.
- `deploy/README.deploy.md` — quick VPS deploy steps: cargo build --release, copy binary, set env, systemctl enable --now.
- `README.md` — rewrite for Rust (cargo run, cargo test, .env, deploy). Remove Python references.
- Remove: `bot/`, `config.py`, `flask_app.py`, `run_polling.py`, `requirements.txt`, `package-lock.json`, `__pycache__/`.

## Data Files (UNCHANGED, verified)

- `data/schedules/ders_programi.pdf` (161KB)
- `data/calendars/akademik_takvim.pdf` (108KB)
- `data/menus/2025-12.json` (11KB) — shape verified: `{year:2025, month:"Aralık", generatedAt:..., menu:[{date:"01 Aralık", dayOfWeek:"Pazartesi", mealType:"öğle", foods:[...], totalcalorie:"1100 KKAL", closed:false, closedReason:""}], ...}`
- `data/menus/menu_2026-01.jpg` (191KB)
- `update_files.py` — kept as-is (manual interactive CLI, not runtime)

## Verified API Idioms (from docs I fetched)

### Teloxide 0.17
- `#[derive(BotCommands, Clone)]` + `#[command(rename_rule="lowercase", description=...)]`
- `Command::repl(bot, answer)` for simple; for deps use `Dispatcher::builder(bot, handler).dependencies(dptree::deps![...]).enable_ctrlc_handler().build().dispatch().await`
- Handler signature: `async fn h(bot: Bot, msg: Message, cmd: Command, svc: Arc<Services>) -> ResponseResult<()>`
- `ResponseResult = Result<(), Box<dyn Error + Send + Sync>>`
- `bot.set_my_commands(Command::descriptions()).await?` to register Turkish command menu
- `Update::filter_message().filter_command::<Command>()` for message branch
- `Update::filter_callback_query()` for callback branch
- `teloxide::utils::html::escape(text)` for HTML-safe inline/HTML mode
- Requires rustc ≥1.85 (have 1.96) ✓
- Webhooks available in `dispatching::update_listeners::webhooks` (NOT using — polling)

### libsql 0.9 (0.9.30 latest)
- `libsql::Builder::new_local(":memory:")` / `new_local(path)` → `.build().await?` → `db.connect()?` → `Connection`
- `libsql::Builder::new_remote(url, token)` for remote Turso
- `conn.execute_batch(sql).await?` for migrations
- `conn.execute(sql, params![...]).await?` — `params!` macro
- `conn.query(sql, [params]).await?` → `Rows`; `rows.next().await` → `Result<Option<Row>>`
- `row.get_value(0)` → `Result<Value>`; `Value::Blob(Vec<u8>)`, `Value::Integer`, `Value::Text(String)`
- `row.get::<T>(idx)` via `FromValue`
- No built-in migrator → tiny runner in `db/migrations.rs`
- Local-file mode = embedded SQLite (no network); remote = HTTP to Turso

### Dependencies chosen
- `reqwest 0.12` with `default-features=false, features=["rustls-tls"]` (avoid OpenSSL native dep)
- `scraper 0.22` = CSS-selector HTML parser (replaces BeautifulSoup+lxml)
- `moka 0.12` with `features=["future"]` for async cache
- `chrono 0.4` (+chrono-tz 0.10) for Europe/Istanbul
- `tracing`+`tracing-subscriber` with `env-filter` (replaces Python logging)

## codegraph findings (use for verification after rewrite)

- `codegraph status` → 29 files, 232 nodes, 380 edges, up-to-date index
- `codegraph explore "bot handler"` shows: `setup_handlers` (bot/main.py:15) called by flask_app.py + run_polling.py; `button_handler` (callback_handler.py:15) called by bot/main.py. After rewrite re-run `codegraph index` to refresh.
- Original bugs confirmed via codegraph: `handle_callback_as_command`/`handle_cafeteria_callback`/`button_handler` all flagged "⚠️ no covering tests found"
- The rewrite eliminates `FakeMessage`/`FakeUpdate` (callback_handler.py:99-119) entirely via shared logic fns

## Ecosystem Context (do NOT lose)

- `/home/c4kar/Projects/ktunEcoSystem/` contains: ktunBot (this), ktunDepo, ktunot (Quartz site), ktunOto
- `docs/superpowers/plans/2026-07-10-veri-platformu-implementation-plan.md` lays out `ktunPlatform/` Rust workspace using Turso/NATS/Qdrant/Meilisearch + Axum/Tokio/SQLx
- The `db/turso.rs` interface deliberately mirrors `ktunPlatform/crates/indexer-service/src/storage/turso.rs` so this bot lifts into the platform workspace later with a path-dep swap
- User's long-term vision: unify all `ktun*` on shared Turso; this bot is the first to use it

## Verification Gates (run before declaring done)

1. `cargo fmt --check`
2. `cargo clippy -- -D warnings`
3. `cargo test` — semester boundaries, formatter (closed+open+emoji+escape), cafeteria (real JSON), count clamp
4. `cargo build --release`
5. Manual: set TELEGRAM_BOT_TOKEN + `file:./data/ktunbot.db`; `cargo run`; test `/start /duyurular /program /takvim /bugun /dun /yarin /hakkinda /yardim` + each inline button against real Telegram using existing data/ files

## First thing to do when resuming

1. Open `docs/REWRITE_PLAN.md` (the authoritative revised plan with Decision Log).
2. Open this file (`docs/HANDOFF.md`).
3. **FIX the typo in `src/db/turso.rs`** — find `b读ductor()` (around line 30) and DELETE that line; the remote branch should just be `let b = Builder::new_remote(url, token);` then `b.build().await?`.
4. Create `src/db/mod.rs` with: `pub mod migrations; pub use turso::Db;`
5. Create `src/cache.rs` (moka L1 — see spec above).
6. Continue through the "Files NOT yet written" list in order.
7. The todo list was last at: step 3 in_progress. Steps 1 (Cargo.toml/.env/main skeleton — note main.rs NOT actually written yet, only Cargo.toml + .env) and 2 (config/error/services — DONE) were marked in_progress/completed in the live todo list.

Paste that into `docs/HANDOFF.md`. When you switch sessions, tell the new session: *"Read `docs/HANDOFF.md` and `docs/REWRITE_PLAN.md`, then continue the Teloxide rewrite from where it left off."*
