use std::path::PathBuf;

pub fn calendar_path(calendars_dir: &std::path::Path) -> PathBuf {
    calendars_dir.join("akademik_takvim.pdf")
}
