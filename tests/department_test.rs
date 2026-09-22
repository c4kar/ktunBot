use ktunbot::handlers::department::{dept_name_tr, department_keyboard, grade_keyboard};

#[test]
fn test_dept_name_tr_mappings() {
    assert!(dept_name_tr("CENG").contains("Bilgisayar"));
    assert!(dept_name_tr("EEM").contains("Elektrik-Elektronik"));
    assert!(dept_name_tr("MECH").contains("Makine"));
    assert!(dept_name_tr("MKT").contains("Mekatronik"));
    assert!(dept_name_tr("CIVIL").contains("İnşaat"));
    assert!(dept_name_tr("CHEM").contains("Kimya"));
    assert!(dept_name_tr("IE").contains("Endüstri"));
    assert!(dept_name_tr("GEO").contains("Harita"));
    assert_eq!(dept_name_tr("UNKNOWN"), "🎓 Genel STEM");
}

#[test]
fn test_department_keyboard_structure() {
    let kb = department_keyboard();
    // 4 rows of 2 buttons = 8 departments
    assert_eq!(kb.inline_keyboard.len(), 4);
    assert_eq!(kb.inline_keyboard[0].len(), 2);
}

#[test]
fn test_grade_keyboard_callback_payload() {
    let kb = grade_keyboard("CENG");
    assert_eq!(kb.inline_keyboard.len(), 2);
    let first_btn = &kb.inline_keyboard[0][0];
    assert_eq!(first_btn.text, "1. Sınıf");
    if let teloxide::types::InlineKeyboardButtonKind::CallbackData(ref data) = first_btn.kind {
        assert_eq!(data, "grade:CENG:1");
    } else {
        panic!("Expected CallbackData button");
    }
}
