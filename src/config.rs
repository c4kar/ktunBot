use std::path::PathBuf;

fn env_or(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}

fn env_opt(key: &str) -> Option<String> {
    std::env::var(key).ok().filter(|s| !s.is_empty())
}

#[derive(Clone, Debug)]
pub struct Config {
    pub telegram_bot_token: Option<String>,
    pub announcements_url: Option<String>,
    pub turso_database_url: String,
    pub turso_auth_token: Option<String>,
    pub base_dir: PathBuf,
    pub data_dir: PathBuf,
    pub schedules_dir: PathBuf,
    pub calendars_dir: PathBuf,
    pub menus_dir: PathBuf,
    pub magnum_dir: PathBuf,
    pub magnum_compiler_script: PathBuf,
    pub incoming_dir: PathBuf,
}

impl Config {
    pub fn from_env() -> Self {
        let base_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let data_dir = base_dir.join("data");
        let magnum_dir = if let Some(d) = env_opt("MAGNUM_DIR") {
            PathBuf::from(d)
        } else if base_dir.join("storage/magnum").exists() {
            base_dir.join("storage/magnum")
        } else if base_dir.join("../storage/magnum").exists() {
            base_dir.join("../storage/magnum")
        } else {
            base_dir.join("storage/magnum")
        };
        let magnum_compiler_script = if let Some(s) = env_opt("MAGNUM_COMPILER_SCRIPT") {
            PathBuf::from(s)
        } else if base_dir.join("ktunMagnum/magnum_compiler.py").exists() {
            base_dir.join("ktunMagnum/magnum_compiler.py")
        } else if base_dir.join("../ktunMagnum/magnum_compiler.py").exists() {
            base_dir.join("../ktunMagnum/magnum_compiler.py")
        } else {
            base_dir.join("ktunMagnum/magnum_compiler.py")
        };
        let incoming_dir = if let Some(d) = env_opt("INCOMING_DIR") {
            PathBuf::from(d)
        } else if base_dir.join("storage/incoming").exists() {
            base_dir.join("storage/incoming")
        } else if base_dir.join("../storage/incoming").exists() {
            base_dir.join("../storage/incoming")
        } else {
            base_dir.join("storage/incoming")
        };
        Self {
            telegram_bot_token: env_opt("TELEGRAM_BOT_TOKEN"),
            announcements_url: env_opt("UNIVERSITY_ANNOUNCEMENTS_URL"),
            turso_database_url: env_or("TURSO_DATABASE_URL", "file:./data/ktunbot.db"),
            turso_auth_token: env_opt("TURSO_AUTH_TOKEN"),
            base_dir,
            schedules_dir: data_dir.join("schedules"),
            calendars_dir: data_dir.join("calendars"),
            menus_dir: data_dir.join("menus"),
            magnum_dir,
            magnum_compiler_script,
            incoming_dir,
            data_dir,
        }
    }
}
