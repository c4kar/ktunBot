use ktunbot::handlers::search::{KTUNOT_SEARCH_URL, KTUNOT_WEB_URL};

#[test]
fn test_ktunot_web_bridge_urls() {
    assert_eq!(KTUNOT_WEB_URL, "https://ktunot.net.tr");
    assert_eq!(KTUNOT_SEARCH_URL, "https://ktunot.net.tr/ara/");

    // Verify valid URL parsing
    let search_url = reqwest::Url::parse(KTUNOT_SEARCH_URL);
    assert!(search_url.is_ok());
    let parsed = search_url.unwrap();
    assert_eq!(parsed.host_str(), Some("ktunot.net.tr"));
    assert_eq!(parsed.path(), "/ara/");
}
