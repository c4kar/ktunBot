use ktunbot::config::Config;
use ktunbot::db::Db;

#[tokio::test]
async fn test_db_migrations_and_profile() -> anyhow::Result<()> {
    // Use a unique temp db path for the test
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let temp_db_path = format!("/tmp/test_ktunbot_{}.db", nanos);
    let config = Config {
        telegram_bot_token: Some("test_token".to_string()),
        announcements_url: None,
        turso_database_url: format!("file:{}", temp_db_path),
        turso_auth_token: None,
        base_dir: std::path::PathBuf::from("."),
        data_dir: std::path::PathBuf::from("data"),
        menus_dir: std::path::PathBuf::from("data/menus"),
        schedules_dir: std::path::PathBuf::from("data/schedules"),
        calendars_dir: std::path::PathBuf::from("data/calendars"),
        magnum_dir: std::path::PathBuf::from("storage/magnum"),
        magnum_compiler_script: std::path::PathBuf::from("ktunMagnum/magnum_compiler.py"),
        incoming_dir: std::path::PathBuf::from("storage/incoming"),
    };

    let db = Db::connect(&config).await?;

    // 1. Initial profile query should be None
    let profile = db.get_student_profile(12345678).await?;
    assert!(profile.is_none());

    // 2. Upsert profile
    db.upsert_student_profile(12345678, "CENG", 2, "Guz").await?;

    // 3. Verify retrieved profile
    let profile = db.get_student_profile(12345678).await?.expect("profile should exist");
    assert_eq!(profile.chat_id, 12345678);
    assert_eq!(profile.department, "CENG");
    assert_eq!(profile.grade, 2);
    assert_eq!(profile.term, "Guz");
    assert_eq!(profile.magnum_credits, 1);

    // 4. Update profile to 3rd grade and add credits
    db.upsert_student_profile(12345678, "CENG", 3, "Bahar").await?;
    db.add_student_credits(12345678, 5).await?;
    let updated = db.get_student_profile(12345678).await?.expect("updated profile");
    assert_eq!(updated.grade, 3);
    assert_eq!(updated.term, "Bahar");
    assert_eq!(updated.magnum_credits, 6);

    // 5. Test telegram file_id cache
    let cached = db.get_cached_telegram_file_id("schedule_CENG_3").await?;
    assert!(cached.is_none());

    db.set_cached_telegram_file_id("schedule_CENG_3", "FILE_ID_ABC123").await?;
    let cached = db.get_cached_telegram_file_id("schedule_CENG_3").await?.expect("file_id cached");
    assert_eq!(cached, "FILE_ID_ABC123");

    // 6. Test intake queue enqueue and processing
    let intake_item = ktunbot::db::IntakeQueueItem {
        id: "intake_test_123".to_string(),
        chat_id: 12345678,
        file_name: "devre_teorisi_not.pdf".to_string(),
        file_size: 1048576,
        storage_path: "/tmp/devre_teorisi_not.pdf".to_string(),
        department: Some("EEM".to_string()),
        course_hint: Some("Devre Teorisi 1".to_string()),
        status: "QUEUED".to_string(),
        quality_score: None,
        rejection_reason: None,
        created_at: None,
        processed_at: None,
    };

    db.enqueue_intake(&intake_item).await?;

    let pending = db.get_pending_intake_items(10).await?;
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].file_name, "devre_teorisi_not.pdf");
    assert_eq!(pending[0].status, "QUEUED");

    // Update status to ACCEPTED with quality score 85
    db.update_intake_status("intake_test_123", "ACCEPTED", Some(85), None).await?;

    let pending_after = db.get_pending_intake_items(10).await?;
    assert_eq!(pending_after.len(), 0);

    // Clean up temp file
    let _ = tokio::fs::remove_file(&temp_db_path).await;
    Ok(())
}
