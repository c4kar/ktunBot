use chrono::Datelike;

pub fn current_semester(date: chrono::NaiveDate) -> String {
    let y = date.year();
    let m = date.month();

    match m {
        1 => format!("{}-{} Güz", y - 1, y),
        2..=6 => format!("{}-{} Bahar", y - 1, y),
        7..=8 => format!("{} Yaz", y),
        9..=12 => format!("{}-{} Güz", y, y + 1),
        _ => unreachable!(),
    }
}
