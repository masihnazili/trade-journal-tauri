mod db;
mod backup;
mod mt_import;

use db::{DbState, TradeInput, Trade, SummaryStats, EquityPoint, CalendarDay};
use mt_import::{parse_mt_html, parse_mt_csv, find_duplicates, ImportSummary};
use rusqlite::Connection;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::Manager;
use std::fs;

#[tauri::command]
fn get_all_trades(state: tauri::State<DbState>) -> Result<Vec<Trade>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    db::get_all_trades(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_trade_by_id(state: tauri::State<DbState>, id: String) -> Result<Option<Trade>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    db::get_trade_by_id(&conn, &id).map_err(|e| e.to_string())
}

#[tauri::command]
fn create_trade(state: tauri::State<DbState>, trade: TradeInput) -> Result<Trade, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    db::create_trade(&conn, trade).map_err(|e| e.to_string())
}

#[tauri::command]
fn update_trade(state: tauri::State<DbState>, id: String, trade: TradeInput) -> Result<Option<Trade>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    db::update_trade(&conn, &id, trade).map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_trade(state: tauri::State<DbState>, id: String) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    db::delete_trade(&conn, &id).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_summary_stats(state: tauri::State<DbState>) -> Result<SummaryStats, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    db::get_summary_stats(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_equity_curve(state: tauri::State<DbState>) -> Result<Vec<EquityPoint>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    db::get_equity_curve(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_calendar_data(state: tauri::State<DbState>, year: i64, month: i64) -> Result<Vec<CalendarDay>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    db::get_calendar_data(&conn, year, month).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_settings(state: tauri::State<DbState>) -> Result<HashMap<String, String>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    db::get_settings(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
fn update_settings(state: tauri::State<DbState>, settings: HashMap<String, String>) -> Result<HashMap<String, String>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    db::update_settings(&conn, settings).map_err(|e| e.to_string())
}

#[tauri::command]
fn export_trades_csv(state: tauri::State<DbState>) -> Result<String, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    db::export_trades_csv(&conn).map_err(|e| e.to_string())
}

/// Backup database to user-selected directory
#[tauri::command]
fn backup_database(state: tauri::State<DbState>, backup_dir: String) -> Result<String, String> {
    let _conn = state.0.lock().map_err(|e| e.to_string())?;
    
    // Get database path from connection (we need to know it)
    // For now, we'll use app data dir + known filename
    let app_dir = std::env::var("APPDATA")
        .unwrap_or_else(|_| std::env::var("HOME").unwrap_or_default());
    let db_path = PathBuf::from(&app_dir)
        .join("trade-journal-tauri")
        .join("trade_journal.db");

    let backup_path = backup::backup_database(&db_path, backup_dir.as_ref())
        .map_err(|e| e.to_string())?;

    Ok(backup_path)
}

/// Restore database from backup
#[tauri::command]
fn restore_database(backup_path: String) -> Result<(), String> {
    // Get app data dir
    let app_dir = std::env::var("APPDATA")
        .unwrap_or_else(|_| std::env::var("HOME").unwrap_or_default());
    let db_path = PathBuf::from(&app_dir)
        .join("trade-journal-tauri")
        .join("trade_journal.db");

    backup::restore_database(backup_path.as_ref(), &db_path)
        .map_err(|e| e.to_string())?;

    Ok(())
}

/// List available backups
#[tauri::command]
fn list_backups(backup_dir: String) -> Result<Vec<String>, String> {
    backup::list_backups(backup_dir.as_ref())
        .map_err(|e| e.to_string())
}

/// Import trades from MT4/MT5 statement (HTML or CSV)
#[tauri::command]
fn import_mt_statement(
    state: tauri::State<DbState>,
    file_path: String,
) -> Result<ImportSummary, String> {
    let file_content = fs::read_to_string(&file_path)
        .map_err(|e| format!("خطا در خواندن فایل: {}", e))?;

    // Determine format (HTML or CSV)
    let mt_trades = if file_path.ends_with(".html") {
        parse_mt_html(&file_content)
    } else if file_path.ends_with(".csv") {
        parse_mt_csv(&file_content)
    } else {
        return Err("فرمت فایل پشتیبانی نمی‌شود. لطفاً HTML یا CSV استفاده کنید.".to_string());
    }?;

    let conn = state.0.lock().map_err(|e| e.to_string())?;

    // Get existing tickets to check for duplicates
    let existing_trades = db::get_all_trades(&conn).map_err(|e| e.to_string())?;
    let existing_tickets: HashSet<String> = existing_trades
        .iter()
        .filter_map(|t| {
            if t.imported_from_mt == 1 {
                // Extract ticket from notes if available
                t.notes.as_ref().and_then(|n| {
                    if n.starts_with("MT:") {
                        Some(n[3..].to_string())
                    } else {
                        None
                    }
                })
            } else {
                None
            }
        })
        .collect();

    let new_tickets: Vec<String> = mt_trades.iter().map(|t| t.ticket.clone()).collect();
    let duplicates = find_duplicates(&new_tickets, &existing_tickets);

    let mut imported_count = 0;
    let mut errors = Vec::new();

    // Insert non-duplicate trades
    for mt_trade in mt_trades {
        if duplicates.contains(&mt_trade.ticket) {
            continue; // Skip duplicates
        }

        let trade_input = TradeInput {
            symbol: mt_trade.symbol.clone(),
            direction: mt_trade.trade_type.clone(),
            strategy_tag: None,
            entry_date: mt_trade.open_time.split_whitespace().next().unwrap_or("").to_string(),
            entry_time: Some(mt_trade.open_time.clone()),
            exit_date: mt_trade.close_time.as_ref().map(|t| t.split_whitespace().next().unwrap_or("").to_string()),
            exit_time: mt_trade.close_time.clone(),
            entry_price: mt_trade.open_price,
            exit_price: mt_trade.close_price,
            stop_loss: mt_trade.stop_loss,
            take_profit: mt_trade.take_profit,
            position_size: mt_trade.volume,
            fees: mt_trade.commission,
            planned_rr: None,
            plan_setup_reason: None,
            plan_market_condition: None,
            plan_confidence: None,
            emotion_entry: None,
            emotion_exit: None,
            followed_plan: Some(true),
            mistake_tags: None,
            lesson_learned: None,
            notes: Some(format!("MT:{}", mt_trade.ticket)),
            screenshot_entry_path: None,
            screenshot_exit_path: None,
            imported_from_mt: Some(true),
        };

        match db::create_trade(&conn, trade_input) {
            Ok(_) => imported_count += 1,
            Err(e) => errors.push(format!("خطا در وارد کردن معامله {}: {}", mt_trade.ticket, e)),
        }
    }

    Ok(ImportSummary {
        total_imported: imported_count,
        duplicates_skipped: duplicates.len(),
        errors,
    })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let app_dir = app.path().app_data_dir().expect("failed to get app data dir");
            std::fs::create_dir_all(&app_dir).expect("failed to create app data dir");
            let db_path = app_dir.join("trade_journal.db");

            let conn = Connection::open(db_path).expect("failed to open database");
            conn.execute_batch("PRAGMA journal_mode = WAL;").ok();
            db::init_db(&conn).expect("failed to init database");

            app.manage(DbState(Mutex::new(conn)));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_all_trades,
            get_trade_by_id,
            create_trade,
            update_trade,
            delete_trade,
            get_summary_stats,
            get_equity_curve,
            get_calendar_data,
            get_settings,
            update_settings,
            export_trades_csv,
            backup_database,
            restore_database,
            list_backups,
            import_mt_statement,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
