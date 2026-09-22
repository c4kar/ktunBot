# Deploying ktunBot

1. Build the release binary:
   ```bash
   cargo build --release
   ```

2. Copy files to `/opt/ktunbot/` on the target server.
   The directory should contain:
   - `target/release/ktunbot`
   - `data/` (menus, calendars, schedules)
   - `migrations/`

3. Create the configuration file at `/etc/ktunbot/ktunbot.env`:
   ```bash
   sudo mkdir -p /etc/ktunbot
   sudo nano /etc/ktunbot/ktunbot.env
   ```
   Add your env vars: `TELEGRAM_BOT_TOKEN`, `TURSO_DATABASE_URL`, `RUST_LOG=info`, etc.

4. Setup systemd unit:
   ```bash
   sudo cp deploy/ktunbot.service /etc/systemd/system/
   sudo systemctl daemon-reload
   sudo systemctl enable --now ktunbot
   ```

5. Check logs:
   ```bash
   sudo journalctl -u ktunbot -f
   ```
