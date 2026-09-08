mod db;

use db::{DbState, TradeInput, Trade, SummaryStats, EquityPoint, CalendarDay};
use rusqlite::Connection;
use std::collections::HashMap;
use std::sync::Mutex;
use tauri::Manager;

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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
