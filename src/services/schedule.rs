use std::path::PathBuf;

pub fn schedule_path(schedules_dir: &std::path::Path) -> PathBuf {
    schedules_dir.join("ders_programi.pdf")
}
