use ktunbot::commands::Command;

#[test]
fn test_parse_count() {
    assert_eq!(Command::parse_count("".to_string()), 10);
    assert_eq!(Command::parse_count("abc".to_string()), 10);
    assert_eq!(Command::parse_count("5".to_string()), 5);
    assert_eq!(Command::parse_count("50".to_string()), 50);
    assert_eq!(Command::parse_count("51".to_string()), 50);
    assert_eq!(Command::parse_count("0".to_string()), 1);
    assert_eq!(Command::parse_count("-5".to_string()), 10); // fails parse
}
