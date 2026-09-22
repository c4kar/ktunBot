use crate::db::Db;
use crate::error::{BotError, Result};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Semaphore;
use tracing::{info, warn};

pub type SharedMagnumService = Arc<MagnumService>;

pub struct MagnumService {
    pub magnum_dir: PathBuf,
    pub compiler_script: PathBuf,
    pub db: Arc<Db>,
    compile_semaphore: Arc<Semaphore>,
}

impl MagnumService {
    pub fn new(magnum_dir: PathBuf, compiler_script: PathBuf, db: Arc<Db>) -> Self {
        // Enforce max 2 concurrent Python compilation jobs to prevent resource starvation
        let compile_semaphore = Arc::new(Semaphore::new(2));
        Self {
            magnum_dir,
            compiler_script,
            db,
            compile_semaphore,
        }
    }

    /// Normalizes raw course queries into standardized uppercase slug format.
    /// E.g.: "fizik 1" -> "FIZIK_1", "eem-202" -> "EEM_202", "  Matematik 1 " -> "MATEMATIK_1"
    pub fn normalize_course_id(query: &str) -> String {
        let mut cleaned = String::with_capacity(query.len());
        for c in query.chars() {
            match c {
                'ı' | 'i' | 'İ' | 'I' => cleaned.push('I'),
                'ö' | 'Ö' => cleaned.push('O'),
                'ü' | 'Ü' => cleaned.push('U'),
                'ç' | 'Ç' => cleaned.push('C'),
                'ş' | 'Ş' => cleaned.push('S'),
                'ğ' | 'Ğ' => cleaned.push('G'),
                c if c.is_ascii_alphanumeric() => cleaned.push(c.to_ascii_uppercase()),
                ' ' | '-' | '/' | '.' | '_' => cleaned.push('_'),
                _ => {}
            }
        }

        // Collapse multiple underscores and trim
        let parts: Vec<&str> = cleaned.split('_').filter(|s| !s.is_empty()).collect();
        parts.join("_")
    }

    /// Locates an existing compiled PDF in magnum_dir.
    pub fn find_existing_pdf(&self, normalized_course: &str) -> Option<PathBuf> {
        let candidates = [
            self.magnum_dir.join(format!("{}_MAGNUM.pdf", normalized_course)),
            self.magnum_dir.join(format!("{}.pdf", normalized_course)),
        ];

        for path in &candidates {
            if path.exists() && path.is_file() {
                return Some(path.clone());
            }
        }

        // Also try case-insensitive scan of magnum_dir if direct matches fail
        if let Ok(entries) = std::fs::read_dir(&self.magnum_dir) {
            let target_sub = normalized_course.replace('_', "").to_uppercase();
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) == Some("pdf") {
                    if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                        let clean_stem = stem.replace(['_', '-'], "").to_uppercase();
                        if clean_stem.starts_with(&target_sub) || target_sub.starts_with(&clean_stem) {
                            return Some(path);
                        }
                    }
                }
            }
        }

        None
    }

    /// Check if we already have a cached Telegram file_id for instant delivery.
    pub async fn get_cached_telegram_file_id(&self, normalized_course: &str) -> Option<String> {
        let key = format!("magnum_{}", normalized_course);
        self.db.get_cached_telegram_file_id(&key).await.ok().flatten()
    }

    /// Store the Telegram file_id returned after successful document delivery.
    pub async fn save_cached_telegram_file_id(&self, normalized_course: &str, file_id: &str) -> Result<()> {
        let key = format!("magnum_{}", normalized_course);
        self.db.set_cached_telegram_file_id(&key, file_id).await
    }

    /// Triggers on-demand compilation via ktunMagnum compiler script under concurrency lock.
    pub async fn compile_course(&self, course_query: &str) -> Result<PathBuf> {
        let normalized = Self::normalize_course_id(course_query);

        // Check again if file was compiled by a concurrent task
        if let Some(existing) = self.find_existing_pdf(&normalized) {
            return Ok(existing);
        }

        info!("Acquiring compilation lock for course '{}'...", course_query);
        let _permit = self
            .compile_semaphore
            .acquire()
            .await
            .map_err(|e| BotError::Other(format!("Derleme kuyruğu hatası: {}", e)))?;

        // Re-check in case it finished while waiting for semaphore
        if let Some(existing) = self.find_existing_pdf(&normalized) {
            return Ok(existing);
        }

        let python_bin = std::env::var("PYTHON_BIN").unwrap_or_else(|_| "python3".to_string());
        info!(
            "Running compiler: {} {:?} '{}' --output-dir {:?}",
            python_bin, self.compiler_script, course_query, self.magnum_dir
        );

        let work_dir = self.compiler_script.parent().unwrap_or_else(|| Path::new("."));

        let output = tokio::process::Command::new(&python_bin)
            .arg(&self.compiler_script)
            .arg(course_query)
            .arg("--output-dir")
            .arg(&self.magnum_dir)
            .current_dir(work_dir)
            .output()
            .await
            .map_err(|e| BotError::Other(format!("Python derleyici alt süreci başlatılamadı: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            warn!("Compiler failed: {}\nstdout: {}", stderr, stdout);
            return Err(BotError::Other(format!(
                "'{}' için Magnum notları derlenemedi. Yeterli ders materyali bulunamamış olabilir.",
                course_query
            )));
        }

        // Verify that target PDF was generated
        if let Some(path) = self.find_existing_pdf(&normalized) {
            info!("Successfully compiled on-demand Magnum package at {:?}", path);
            Ok(path)
        } else {
            Err(BotError::NotFound(format!(
                "Derleme tamamlandı ancak beklenen PDF ({}) diskte bulunamadı.",
                normalized
            )))
        }
    }

    /// Returns recommended Magnum Opus courses based on student profile.
    pub fn get_catalog_courses(&self, department: &str, grade: u8) -> Vec<(&'static str, &'static str)> {
        match (department, grade) {
            // 1st Grade (Common across almost all STEM departments)
            (_, 1) => vec![
                ("⚡ Fizik 1", "FIZIK_1"),
                ("📐 Matematik 1", "MATEMATIK_1"),
                ("🧪 Genel Kimya", "KIMYA"),
                ("💻 Bilgisayar Prog. 1", "BILGISAYAR_1"),
                ("📊 Lineer Cebir", "LINEER_CEBIR"),
            ],
            ("EEM", 2) => vec![
                ("🔌 Devre Teorisi 1", "EEM_201"),
                ("📈 Diferansiyel Denk.", "DIFERANSIYEL"),
                ("⚡ Elektromanyetik", "EEM_204"),
                ("💡 Sayısal Mantık", "EEM_205"),
            ],
            ("CENG", 2) => vec![
                ("🌲 Veri Yapıları", "CENG_201"),
                ("📈 Diferansiyel Denk.", "DIFERANSIYEL"),
                ("🔢 Ayrık Matematik", "CENG_205"),
                ("☕ Nesne Yönelimli Prog.", "CENG_203"),
            ],
            ("MECH", 2) => vec![
                ("⚖️ Statik", "MECH_201"),
                ("🔥 Termodinamik 1", "MECH_203"),
                ("📈 Diferansiyel Denk.", "DIFERANSIYEL"),
                ("🔩 Mukavemet 1", "MECH_205"),
            ],
            ("CIVIL", 2) => vec![
                ("⚖️ Statik", "CIVIL_201"),
                ("🔩 Mukavemet 1", "CIVIL_203"),
                ("📈 Diferansiyel Denk.", "DIFERANSIYEL"),
                ("🌊 Akışkanlar Mekaniği", "CIVIL_205"),
            ],
            // Default / fallback courses
            _ => vec![
                ("⚡ Fizik 1", "FIZIK_1"),
                ("📐 Matematik 1", "MATEMATIK_1"),
                ("📈 Diferansiyel Denk.", "DIFERANSIYEL"),
                ("📊 Lineer Cebir", "LINEER_CEBIR"),
            ],
        }
    }

    /// Formats the official Telegram document caption.
    pub fn format_caption(course_name: &str) -> String {
        format!(
            "📖 <b>{} — MAGNUM OPUS KILAVUZU</b>\n\
            ━━━━━━━━━━━━━━━━━━━━━━\n\
            💡 <i>Bu rehber, KTÜN STEM Not Ağı'ndaki öğrenci notlarının filtrelenip sentezlenmesiyle hazırlanmıştır.</i>\n\n\
            ✨ <b>Paket İçeriği:</b>\n\
            • 📌 Hoca tüyoları ve sınav püf noktaları\n\
            • ⚠️ Sık yapılan hatalar ve sınav tuzakları\n\
            • 📝 Çıkmış soru analizleri ve çözüm adımları\n\
            • 📐 Kritik formül ve tanım kartı\n\n\
            🎓 <i>Konya Teknik Üniversitesi · STEM Not Ağı</i>",
            course_name.to_uppercase()
        )
    }
}
