-- 创建课程表
CREATE TABLE IF NOT EXISTS classes (
    class_lesson_id TEXT PRIMARY KEY,
    lesson_name TEXT NOT NULL,
    last_check_id TEXT,
    last_site_id TEXT,
    last_create_time TEXT,
    is_expired BOOLEAN DEFAULT 0
);

-- 创建索引
CREATE INDEX IF NOT EXISTS idx_class_lesson_id ON classes(class_lesson_id);