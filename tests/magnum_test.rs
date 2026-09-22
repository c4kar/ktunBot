use ktunbot::config::Config;
use ktunbot::db::Db;
use ktunbot::services::magnum::MagnumService;
use std::path::PathBuf;
use std::sync::Arc;

async fn init_test_db() -> (Arc<Db>, PathBuf) {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let temp_db_path = std::env::temp_dir().join(format!("test_ktun_magnum_{}.db", nanos));
    let mut config = Config::from_env();
    config.turso_database_url = format!("file:{}", temp_db_path.to_string_lossy());
    let db = Db::connect(&config).await.expect("Failed to initialize test Turso DB");
    (Arc::new(db), temp_db_path)
}

#[test]
fn test_normalize_course_id() {
    assert_eq!(MagnumService::normalize_course_id("fizik 1"), "FIZIK_1");
    assert_eq!(MagnumService::normalize_course_id("  Matematik 1  "), "MATEMATIK_1");
    assert_eq!(MagnumService::normalize_course_id("EEM-202"), "EEM_202");
    assert_eq!(MagnumService::normalize_course_id("eem/201"), "EEM_201");
    assert_eq!(MagnumService::normalize_course_id("İnşaat - Statik"), "INSAAT_STATIK");
    assert_eq!(MagnumService::normalize_course_id("çıkmış..sorular"), "CIKMIS_SORULAR");
    assert_eq!(MagnumService::normalize_course_id("diferansiyel denklemler"), "DIFERANSIYEL_DENKLEMLER");
}

#[tokio::test]
async fn test_catalog_courses_per_grade() {
    let (db, temp_path) = init_test_db().await;
    let magnum = MagnumService::new(
        PathBuf::from("storage/magnum"),
        PathBuf::from("ktunMagnum/magnum_compiler.py"),
        db,
    );

    let g1 = magnum.get_catalog_courses("EEM", 1);
    assert!(g1.iter().any(|(_, id)| *id == "FIZIK_1"));
    assert!(g1.iter().any(|(_, id)| *id == "MATEMATIK_1"));

    let eem_g2 = magnum.get_catalog_courses("EEM", 2);
    assert!(eem_g2.iter().any(|(_, id)| *id == "EEM_201"));
    assert!(eem_g2.iter().any(|(_, id)| *id == "DIFERANSIYEL"));

    let ceng_g2 = magnum.get_catalog_courses("CENG", 2);
    assert!(ceng_g2.iter().any(|(_, id)| *id == "CENG_201"));

    let mech_g2 = magnum.get_catalog_courses("MECH", 2);
    assert!(mech_g2.iter().any(|(_, id)| *id == "MECH_201"));

    let _ = std::fs::remove_file(temp_path);
}

#[test]
fn test_format_caption() {
    let caption = MagnumService::format_caption("FIZIK_1");
    assert!(caption.contains("<b>FIZIK_1 — MAGNUM OPUS KILAVUZU</b>"));
    assert!(caption.contains("KTÜN STEM Not Ağı"));
    assert!(caption.contains("Hoca tüyoları"));
    assert!(caption.contains("Sık yapılan hatalar"));
    assert!(caption.contains("Çıkmış soru analizleri"));
}

#[tokio::test]
async fn test_find_existing_pdf() {
    let (db, temp_path) = init_test_db().await;
    let base_dir = std::env::current_dir().unwrap();
    let magnum_dir = if base_dir.join("storage/magnum").exists() {
        base_dir.join("storage/magnum")
    } else if base_dir.join("../storage/magnum").exists() {
        base_dir.join("../storage/magnum")
    } else {
        base_dir.join("storage/magnum")
    };

    if magnum_dir.exists() {
        let magnum = MagnumService::new(
            magnum_dir,
            PathBuf::from("ktunMagnum/magnum_compiler.py"),
            db,
        );

        let fizik_pdf = magnum.find_existing_pdf("FIZIK_1");
        assert!(fizik_pdf.is_some(), "FIZIK_1_MAGNUM.pdf should be discovered");

        let mat_pdf = magnum.find_existing_pdf("MATEMATIK_1");
        assert!(mat_pdf.is_some(), "MATEMATIK_1_MAGNUM.pdf should be discovered");

        let nonexistent = magnum.find_existing_pdf("NONEXISTENT_COURSE_XYZ");
        assert!(nonexistent.is_none());
    }

    let _ = std::fs::remove_file(temp_path);
}

#[tokio::test]
async fn test_telegram_file_cache_persistence() {
    let (db, temp_path) = init_test_db().await;
    let magnum_svc = MagnumService::new(
        PathBuf::from("storage/magnum"),
        PathBuf::from("ktunMagnum/magnum_compiler.py"),
        db,
    );

    // Initial check: cache miss
    let initial_hit = magnum_svc.get_cached_telegram_file_id("FIZIK_1").await;
    assert_eq!(initial_hit, None);

    // Store fake telegram file_id
    let test_file_id = "BAACAgIAAxkBAAIBmGf...";
    magnum_svc
        .save_cached_telegram_file_id("FIZIK_1", test_file_id)
        .await
        .expect("Failed to cache telegram file id");

    // Second check: cache hit
    let cached_hit = magnum_svc.get_cached_telegram_file_id("FIZIK_1").await;
    assert_eq!(cached_hit, Some(test_file_id.to_string()));

    // Clean up temp DB
    let _ = std::fs::remove_file(temp_path);
}
