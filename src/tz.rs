use chrono::{DateTime, Duration, NaiveDate, Utc};
use chrono_tz::{Europe::Istanbul, Tz};

pub fn now_istanbul() -> DateTime<Tz> {
    Utc::now().with_timezone(&Istanbul)
}

pub fn today() -> NaiveDate {
    now_istanbul().date_naive()
}

pub fn yesterday() -> NaiveDate {
    today() - Duration::days(1)
}

pub fn tomorrow() -> NaiveDate {
    today() + Duration::days(1)
}

pub fn tr_month_name(month: u32) -> &'static str {
    match month {
        1 => "Ocak",
        2 => "Şubat",
        3 => "Mart",
        4 => "Nisan",
        5 => "Mayıs",
        6 => "Haziran",
        7 => "Temmuz",
        8 => "Ağustos",
        9 => "Eylül",
        10 => "Ekim",
        11 => "Kasım",
        12 => "Aralık",
        _ => "",
    }
}

pub fn format_tr_date(date: NaiveDate) -> String {
    use chrono::Datelike;
    format!("{:02} {}", date.day(), tr_month_name(date.month()))
}
