-- migrations/0002_ecosystem_expansion.sql
-- ktunEcoSystem STEM Genişlemesi ve Çoklu Bölüm Öğrenci Profili

-- 1. Öğrenci Profili Tablosu
CREATE TABLE IF NOT EXISTS student_profiles (
    chat_id INTEGER PRIMARY KEY,
    department TEXT NOT NULL DEFAULT 'EEM', -- CENG, EEM, MECH, MECHATRONICS, CIVIL, CHEM, INDUSTRIAL, ARCH
    grade INTEGER NOT NULL DEFAULT 1,       -- 1, 2, 3, 4
    term TEXT NOT NULL DEFAULT 'Guz',       -- Guz, Bahar
    magnum_credits INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- 2. Materyal Alım Kuyruğu (Rust Bot -> Python Engine)
CREATE TABLE IF NOT EXISTS intake_queue (
    id TEXT PRIMARY KEY,                    -- UUID
    chat_id INTEGER NOT NULL,
    file_name TEXT NOT NULL,
    file_size INTEGER NOT NULL,
    storage_path TEXT NOT NULL,
    department TEXT,
    course_hint TEXT,
    status TEXT NOT NULL DEFAULT 'QUEUED',  -- QUEUED, PROCESSING, ACCEPTED, REJECTED
    quality_score INTEGER,
    rejection_reason TEXT,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    processed_at TEXT
);

-- 3. Telegram file_id Önbelleği (Bant genişliği tasarrufu)
CREATE TABLE IF NOT EXISTS telegram_file_cache (
    cache_key TEXT PRIMARY KEY,             -- e.g. "schedule_EEM_1", "magnum_EEM_201"
    file_id TEXT NOT NULL,
    file_unique_id TEXT,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_student_profiles_dept ON student_profiles(department);
CREATE INDEX IF NOT EXISTS idx_intake_queue_status ON intake_queue(status);
