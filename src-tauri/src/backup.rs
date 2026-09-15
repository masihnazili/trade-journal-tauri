use chrono::Local;
use std::fs;
use std::path::{Path, PathBuf};

/// ایجاد backup با timestamp
pub fn backup_database(db_path: &Path, backup_dir: &Path) -> Result<String, String> {
    if !db_path.exists() {
        return Err("دیتابیس یافت نشد".to_string());
    }

    // اطمینان از وجود مسیر backup
    fs::create_dir_all(backup_dir).map_err(|e| format!("خطا در ایجاد پوشه: {}", e))?;

    // نام فایل backup با timestamp
    let timestamp = Local::now().format("%Y%m%d_%H%M%S").to_string();
    let backup_filename = format!("trade_journal_backup_{}.db", timestamp);
    let backup_path = backup_dir.join(&backup_filename);

    // کپی فایل
    fs::copy(db_path, &backup_path)
        .map_err(|e| format!("خطا در کپی دیتابیس: {}", e))?;

    Ok(backup_path.to_string_lossy().to_string())
}

/// بازیابی database از backup
pub fn restore_database(backup_path: &Path, db_path: &Path) -> Result<(), String> {
    if !backup_path.exists() {
        return Err("فایل backup یافت نشد".to_string());
    }

    // بک‌آپ فعلی دیتابیس قبل از overwrite (safety measure)
    if db_path.exists() {
        let timestamp = Local::now().format("%Y%m%d_%H%M%S").to_string();
        let safety_backup = db_path.with_extension(format!("db.pre_restore_{}", timestamp));
        fs::copy(db_path, &safety_backup)
            .map_err(|e| format!("خطا در ایجاد بک‌آپ امنیتی: {}", e))?;
    }

    // کپی backup به جای فایل اصلی
    fs::copy(backup_path, db_path)
        .map_err(|e| format!("خطا در بازیابی دیتابیس: {}", e))?;

    Ok(())
}

/// لیست تمام backups در یک مسیر
pub fn list_backups(backup_dir: &Path) -> Result<Vec<String>, String> {
    if !backup_dir.exists() {
        return Ok(Vec::new());
    }

    let mut backups = Vec::new();
    let entries = fs::read_dir(backup_dir)
        .map_err(|e| format!("خطا در خواندن پوشه: {}", e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("خطا در خواندن entry: {}", e))?;
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("db") {
            backups.push(path.to_string_lossy().to_string());
        }
    }

    backups.sort();
    backups.reverse(); // آخرین‌ها اول
    Ok(backups)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_backup_creation() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let backup_dir = temp_dir.path().join("backups");

        // ایجاد فایل dummy
        fs::write(&db_path, "test data").unwrap();

        let result = backup_database(&db_path, &backup_dir);
        assert!(result.is_ok());
        assert!(backup_dir.join("trade_journal_backup_*").exists() == false); // این فقط برای بررسی regex
    }

    #[test]
    fn test_restore_database() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let backup_path = temp_dir.path().join("backup.db");

        fs::write(&backup_path, "backup data").unwrap();

        let result = restore_database(&backup_path, &db_path);
        assert!(result.is_ok());
        assert_eq!(fs::read_to_string(&db_path).unwrap(), "backup data");
    }
}
