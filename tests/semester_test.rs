use chrono::NaiveDate;
use ktunbot::services::semester::current_semester;

#[test]
fn test_semester_boundaries() {
    assert_eq!(
        current_semester(NaiveDate::from_ymd_opt(2026, 1, 15).unwrap()),
        "2025-2026 Güz"
    );
    assert_eq!(
        current_semester(NaiveDate::from_ymd_opt(2026, 2, 1).unwrap()),
        "2025-2026 Bahar"
    );
    assert_eq!(
        current_semester(NaiveDate::from_ymd_opt(2026, 6, 30).unwrap()),
        "2025-2026 Bahar"
    );
    assert_eq!(
        current_semester(NaiveDate::from_ymd_opt(2026, 7, 1).unwrap()),
        "2026 Yaz"
    );
    assert_eq!(
        current_semester(NaiveDate::from_ymd_opt(2026, 8, 31).unwrap()),
        "2026 Yaz"
    );
    assert_eq!(
        current_semester(NaiveDate::from_ymd_opt(2026, 9, 1).unwrap()),
        "2026-2027 Güz"
    );
    assert_eq!(
        current_semester(NaiveDate::from_ymd_opt(2026, 12, 31).unwrap()),
        "2026-2027 Güz"
    );
}
