use rusqlite::{Connection, params, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use uuid::Uuid;

pub struct DbState(pub Mutex<Connection>);

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct TradeInput {
    pub symbol: String,
    pub direction: String,
    pub strategy_tag: Option<String>,
    pub entry_date: String,
    pub entry_time: Option<String>,
    pub exit_date: Option<String>,
    pub exit_time: Option<String>,
    pub entry_price: f64,
    pub exit_price: Option<f64>,
    pub stop_loss: Option<f64>,
    pub take_profit: Option<f64>,
    pub position_size: f64,
    pub fees: Option<f64>,
    pub planned_rr: Option<f64>,
    pub plan_setup_reason: Option<String>,
    pub plan_market_condition: Option<String>,
    pub plan_confidence: Option<i64>,
    pub emotion_entry: Option<String>,
    pub emotion_exit: Option<String>,
    pub followed_plan: Option<bool>,
    pub mistake_tags: Option<String>,
    pub lesson_learned: Option<String>,
    pub notes: Option<String>,
    pub screenshot_entry_path: Option<String>,
    pub screenshot_exit_path: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Trade {
    pub id: String,
    pub symbol: String,
    pub direction: String,
    pub strategy_tag: Option<String>,
    pub entry_date: String,
    pub entry_time: Option<String>,
    pub exit_date: Option<String>,
    pub exit_time: Option<String>,
    pub entry_price: f64,
    pub exit_price: Option<f64>,
    pub stop_loss: Option<f64>,
    pub take_profit: Option<f64>,
    pub position_size: f64,
    pub fees: f64,
    pub planned_rr: Option<f64>,
    pub actual_rr: Option<f64>,
    pub pnl_gross: Option<f64>,
    pub pnl_net: Option<f64>,
    pub pnl_percent: Option<f64>,
    pub plan_setup_reason: Option<String>,
    pub plan_market_condition: Option<String>,
    pub plan_confidence: Option<i64>,
    pub emotion_entry: Option<String>,
    pub emotion_exit: Option<String>,
    pub followed_plan: i64,
    pub mistake_tags: Option<String>,
    pub lesson_learned: Option<String>,
    pub notes: Option<String>,
    pub screenshot_entry_path: Option<String>,
    pub screenshot_exit_path: Option<String>,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct SummaryStats {
    pub total_trades: i64,
    pub win_rate: f64,
    pub avg_win: f64,
    pub avg_loss: f64,
    pub profit_factor: f64,
    pub total_pnl: f64,
    pub max_drawdown: f64,
    pub avg_rr: f64,
    pub plan_adherence: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EquityPoint {
    pub date: String,
    pub balance: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CalendarDay {
    pub entry_date: String,
    pub day_pnl: f64,
    pub trade_count: i64,
}

pub fn init_db(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS trades (
            id TEXT PRIMARY KEY,
            symbol TEXT NOT NULL,
            direction TEXT NOT NULL,
            strategy_tag TEXT,
            entry_date TEXT NOT NULL,
            entry_time TEXT,
            exit_date TEXT,
            exit_time TEXT,
            entry_price REAL NOT NULL,
            exit_price REAL,
            stop_loss REAL,
            take_profit REAL,
            position_size REAL NOT NULL,
            fees REAL DEFAULT 0,
            planned_rr REAL,
            actual_rr REAL,
            pnl_gross REAL,
            pnl_net REAL,
            pnl_percent REAL,
            plan_setup_reason TEXT,
            plan_market_condition TEXT,
            plan_confidence INTEGER,
            emotion_entry TEXT,
            emotion_exit TEXT,
            followed_plan INTEGER DEFAULT 1,
            mistake_tags TEXT,
            lesson_learned TEXT,
            notes TEXT,
            screenshot_entry_path TEXT,
            screenshot_exit_path TEXT,
            status TEXT DEFAULT 'closed',
            created_at TEXT DEFAULT (datetime('now')),
            updated_at TEXT DEFAULT (datetime('now'))
        );
        CREATE TABLE IF NOT EXISTS settings (
            key TEXT PRIMARY KEY,
            value TEXT
        );
        CREATE INDEX IF NOT EXISTS idx_trades_entry_date ON trades(entry_date);
        CREATE INDEX IF NOT EXISTS idx_trades_symbol ON trades(symbol);
        CREATE INDEX IF NOT EXISTS idx_trades_strategy ON trades(strategy_tag);",
    )?;

    conn.execute(
        "INSERT OR IGNORE INTO settings (key, value) VALUES ('starting_balance', '10000')",
        [],
    )?;
    conn.execute(
        "INSERT OR IGNORE INTO settings (key, value) VALUES ('currency', 'USD')",
        [],
    )?;

    Ok(())
}

fn compute_pnl(
    direction: &str,
    entry_price: f64,
    exit_price: Option<f64>,
    position_size: f64,
    fees: f64,
    stop_loss: Option<f64>,
) -> (Option<f64>, Option<f64>, Option<f64>, Option<f64>) {
    if let Some(exit_p) = exit_price {
        let diff = if direction == "long" {
            exit_p - entry_price
        } else {
            entry_price - exit_p
        };
        let pnl_gross = diff * position_size;
        let pnl_net = pnl_gross - fees;
        let pnl_percent = (diff / entry_price) * 100.0;

        let actual_rr = if let Some(sl) = stop_loss {
            if (entry_price - sl).abs() > f64::EPSILON {
                let risk_per_unit = (entry_price - sl).abs();
                if diff != 0.0 {
                    let sign = if pnl_net >= 0.0 { 1.0 } else { -1.0 };
                    Some((diff.abs() / risk_per_unit) * sign)
                } else {
                    Some(0.0)
                }
            } else {
                None
            }
        } else {
            None
        };

        (Some(pnl_gross), Some(pnl_net), Some(pnl_percent), actual_rr)
    } else {
        (None, None, None, None)
    }
}

fn row_to_trade(row: &rusqlite::Row) -> rusqlite::Result<Trade> {
    Ok(Trade {
        id: row.get("id")?,
        symbol: row.get("symbol")?,
        direction: row.get("direction")?,
        strategy_tag: row.get("strategy_tag")?,
        entry_date: row.get("entry_date")?,
        entry_time: row.get("entry_time")?,
        exit_date: row.get("exit_date")?,
        exit_time: row.get("exit_time")?,
        entry_price: row.get("entry_price")?,
        exit_price: row.get("exit_price")?,
        stop_loss: row.get("stop_loss")?,
        take_profit: row.get("take_profit")?,
        position_size: row.get("position_size")?,
        fees: row.get("fees")?,
        planned_rr: row.get("planned_rr")?,
        actual_rr: row.get("actual_rr")?,
        pnl_gross: row.get("pnl_gross")?,
        pnl_net: row.get("pnl_net")?,
        pnl_percent: row.get("pnl_percent")?,
        plan_setup_reason: row.get("plan_setup_reason")?,
        plan_market_condition: row.get("plan_market_condition")?,
        plan_confidence: row.get("plan_confidence")?,
        emotion_entry: row.get("emotion_entry")?,
        emotion_exit: row.get("emotion_exit")?,
        followed_plan: row.get("followed_plan")?,
        mistake_tags: row.get("mistake_tags")?,
        lesson_learned: row.get("lesson_learned")?,
        notes: row.get("notes")?,
        screenshot_entry_path: row.get("screenshot_entry_path")?,
        screenshot_exit_path: row.get("screenshot_exit_path")?,
        status: row.get("status")?,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
    })
}

pub fn get_trade_by_id(conn: &Connection, id: &str) -> rusqlite::Result<Option<Trade>> {
    conn.query_row("SELECT * FROM trades WHERE id = ?1", params![id], row_to_trade)
        .optional()
}

pub fn get_all_trades(conn: &Connection) -> rusqlite::Result<Vec<Trade>> {
    let mut stmt = conn.prepare("SELECT * FROM trades ORDER BY entry_date DESC, entry_time DESC")?;
    let rows = stmt.query_map([], row_to_trade)?;
    rows.collect()
}

pub fn create_trade(conn: &Connection, t: TradeInput) -> rusqlite::Result<Trade> {
    let id = Uuid::new_v4().to_string();
    let fees = t.fees.unwrap_or(0.0);
    let followed_plan = if t.followed_plan.unwrap_or(true) { 1 } else { 0 };
    let status = if t.exit_price.is_some() { "closed" } else { "open" };

    let (pnl_gross, pnl_net, pnl_percent, actual_rr) =
        compute_pnl(&t.direction, t.entry_price, t.exit_price, t.position_size, fees, t.stop_loss);

    conn.execute(
        "INSERT INTO trades (
            id, symbol, direction, strategy_tag, entry_date, entry_time, exit_date, exit_time,
            entry_price, exit_price, stop_loss, take_profit, position_size, fees,
            planned_rr, actual_rr, pnl_gross, pnl_net, pnl_percent,
            plan_setup_reason, plan_market_condition, plan_confidence,
            emotion_entry, emotion_exit, followed_plan, mistake_tags,
            lesson_learned, notes, screenshot_entry_path, screenshot_exit_path, status
        ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19,?20,?21,?22,?23,?24,?25,?26,?27,?28,?29,?30,?31)",
        params![
            id, t.symbol, t.direction, t.strategy_tag, t.entry_date, t.entry_time, t.exit_date, t.exit_time,
            t.entry_price, t.exit_price, t.stop_loss, t.take_profit, t.position_size, fees,
            t.planned_rr, actual_rr, pnl_gross, pnl_net, pnl_percent,
            t.plan_setup_reason, t.plan_market_condition, t.plan_confidence,
            t.emotion_entry, t.emotion_exit, followed_plan, t.mistake_tags,
            t.lesson_learned, t.notes, t.screenshot_entry_path, t.screenshot_exit_path, status
        ],
    )?;

    Ok(get_trade_by_id(conn, &id)?.unwrap())
}

pub fn update_trade(conn: &Connection, id: &str, t: TradeInput) -> rusqlite::Result<Option<Trade>> {
    let fees = t.fees.unwrap_or(0.0);
    let followed_plan = if t.followed_plan.unwrap_or(true) { 1 } else { 0 };
    let status = if t.exit_price.is_some() { "closed" } else { "open" };

    let (pnl_gross, pnl_net, pnl_percent, actual_rr) =
        compute_pnl(&t.direction, t.entry_price, t.exit_price, t.position_size, fees, t.stop_loss);

    conn.execute(
        "UPDATE trades SET
            symbol=?1, direction=?2, strategy_tag=?3, entry_date=?4, entry_time=?5, exit_date=?6, exit_time=?7,
            entry_price=?8, exit_price=?9, stop_loss=?10, take_profit=?11, position_size=?12, fees=?13,
            planned_rr=?14, actual_rr=?15, pnl_gross=?16, pnl_net=?17, pnl_percent=?18,
            plan_setup_reason=?19, plan_market_condition=?20, plan_confidence=?21,
            emotion_entry=?22, emotion_exit=?23, followed_plan=?24, mistake_tags=?25,
            lesson_learned=?26, notes=?27, screenshot_entry_path=?28, screenshot_exit_path=?29,
            status=?30, updated_at=datetime('now')
        WHERE id=?31",
        params![
            t.symbol, t.direction, t.strategy_tag, t.entry_date, t.entry_time, t.exit_date, t.exit_time,
            t.entry_price, t.exit_price, t.stop_loss, t.take_profit, t.position_size, fees,
            t.planned_rr, actual_rr, pnl_gross, pnl_net, pnl_percent,
            t.plan_setup_reason, t.plan_market_condition, t.plan_confidence,
            t.emotion_entry, t.emotion_exit, followed_plan, t.mistake_tags,
            t.lesson_learned, t.notes, t.screenshot_entry_path, t.screenshot_exit_path,
            status, id
        ],
    )?;

    get_trade_by_id(conn, id)
}

pub fn delete_trade(conn: &Connection, id: &str) -> rusqlite::Result<()> {
    conn.execute("DELETE FROM trades WHERE id = ?1", params![id])?;
    Ok(())
}

pub fn get_settings(conn: &Connection) -> rusqlite::Result<std::collections::HashMap<String, String>> {
    let mut stmt = conn.prepare("SELECT key, value FROM settings")?;
    let rows = stmt.query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)))?;
    let mut map = std::collections::HashMap::new();
    for r in rows {
        let (k, v) = r?;
        map.insert(k, v);
    }
    Ok(map)
}

pub fn update_settings(conn: &Connection, settings: std::collections::HashMap<String, String>) -> rusqlite::Result<std::collections::HashMap<String, String>> {
    for (k, v) in settings {
        conn.execute(
            "INSERT INTO settings (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![k, v],
        )?;
    }
    get_settings(conn)
}

pub fn get_summary_stats(conn: &Connection) -> rusqlite::Result<SummaryStats> {
    let trades: Vec<Trade> = {
        let mut stmt = conn.prepare("SELECT * FROM trades WHERE status = 'closed'")?;
        let rows = stmt.query_map([], row_to_trade)?;
        rows.collect::<rusqlite::Result<Vec<_>>>()?
    };

    let total = trades.len() as i64;
    if total == 0 {
        return Ok(SummaryStats::default());
    }

    let wins: Vec<&Trade> = trades.iter().filter(|t| t.pnl_net.unwrap_or(0.0) > 0.0).collect();
    let losses: Vec<&Trade> = trades.iter().filter(|t| t.pnl_net.unwrap_or(0.0) < 0.0).collect();
    let total_pnl: f64 = trades.iter().map(|t| t.pnl_net.unwrap_or(0.0)).sum();
    let gross_profit: f64 = wins.iter().map(|t| t.pnl_net.unwrap_or(0.0)).sum();
    let gross_loss: f64 = losses.iter().map(|t| t.pnl_net.unwrap_or(0.0)).sum::<f64>().abs();
    let avg_win = if !wins.is_empty() { gross_profit / wins.len() as f64 } else { 0.0 };
    let avg_loss = if !losses.is_empty() { gross_loss / losses.len() as f64 } else { 0.0 };
    let profit_factor = if gross_loss > 0.0 {
        gross_profit / gross_loss
    } else if gross_profit > 0.0 {
        f64::INFINITY
    } else {
        0.0
    };

    let rr_trades: Vec<&Trade> = trades.iter().filter(|t| t.actual_rr.is_some()).collect();
    let avg_rr = if !rr_trades.is_empty() {
        rr_trades.iter().map(|t| t.actual_rr.unwrap()).sum::<f64>() / rr_trades.len() as f64
    } else {
        0.0
    };

    let followed_count = trades.iter().filter(|t| t.followed_plan == 1).count() as f64;
    let plan_adherence = (followed_count / total as f64) * 100.0;

    let mut sorted = trades.clone();
    sorted.sort_by(|a, b| a.entry_date.cmp(&b.entry_date));
    let mut running = 0.0;
    let mut peak = 0.0;
    let mut max_drawdown = 0.0;
    for t in &sorted {
        running += t.pnl_net.unwrap_or(0.0);
        if running > peak {
            peak = running;
        }
        let dd = peak - running;
        if dd > max_drawdown {
            max_drawdown = dd;
        }
    }

    Ok(SummaryStats {
        total_trades: total,
        win_rate: (wins.len() as f64 / total as f64) * 100.0,
        avg_win,
        avg_loss,
        profit_factor,
        total_pnl,
        max_drawdown,
        avg_rr,
        plan_adherence,
    })
}

pub fn get_equity_curve(conn: &Connection) -> rusqlite::Result<Vec<EquityPoint>> {
    let settings = get_settings(conn)?;
    let starting_balance: f64 = settings
        .get("starting_balance")
        .and_then(|v| v.parse().ok())
        .unwrap_or(10000.0);

    let mut stmt = conn.prepare(
        "SELECT entry_date, pnl_net FROM trades WHERE status = 'closed' ORDER BY entry_date ASC",
    )?;
    let rows = stmt.query_map([], |row| {
        let date: String = row.get(0)?;
        let pnl: Option<f64> = row.get(1)?;
        Ok((date, pnl.unwrap_or(0.0)))
    })?;

    let mut balance = starting_balance;
    let mut result = Vec::new();
    for r in rows {
        let (date, pnl) = r?;
        balance += pnl;
        result.push(EquityPoint {
            date,
            balance: (balance * 100.0).round() / 100.0,
        });
    }
    Ok(result)
}

pub fn get_calendar_data(conn: &Connection, year: i64, month: i64) -> rusqlite::Result<Vec<CalendarDay>> {
    let prefix = format!("{:04}-{:02}", year, month);
    let mut stmt = conn.prepare(
        "SELECT entry_date, SUM(pnl_net) as day_pnl, COUNT(*) as trade_count
         FROM trades WHERE status = 'closed' AND entry_date LIKE ?1
         GROUP BY entry_date",
    )?;
    let pattern = format!("{}%", prefix);
    let rows = stmt.query_map(params![pattern], |row| {
        Ok(CalendarDay {
            entry_date: row.get(0)?,
            day_pnl: row.get::<_, Option<f64>>(1)?.unwrap_or(0.0),
            trade_count: row.get(2)?,
        })
    })?;
    rows.collect()
}

pub fn export_trades_csv(conn: &Connection) -> rusqlite::Result<String> {
    let trades = get_all_trades(conn)?;
    if trades.is_empty() {
        return Ok(String::new());
    }

    let headers = vec![
        "id", "symbol", "direction", "strategy_tag", "entry_date", "entry_time", "exit_date", "exit_time",
        "entry_price", "exit_price", "stop_loss", "take_profit", "position_size", "fees",
        "planned_rr", "actual_rr", "pnl_gross", "pnl_net", "pnl_percent",
        "plan_setup_reason", "plan_market_condition", "plan_confidence",
        "emotion_entry", "emotion_exit", "followed_plan", "mistake_tags",
        "lesson_learned", "notes", "screenshot_entry_path", "screenshot_exit_path",
        "status", "created_at", "updated_at",
    ];

    let mut csv = headers.join(",") + "\n";
    for t in trades {
        let row = vec![
            t.id, t.symbol, t.direction,
            t.strategy_tag.unwrap_or_default(),
            t.entry_date,
            t.entry_time.unwrap_or_default(),
            t.exit_date.unwrap_or_default(),
            t.exit_time.unwrap_or_default(),
            t.entry_price.to_string(),
            t.exit_price.map(|v| v.to_string()).unwrap_or_default(),
            t.stop_loss.map(|v| v.to_string()).unwrap_or_default(),
            t.take_profit.map(|v| v.to_string()).unwrap_or_default(),
            t.position_size.to_string(),
            t.fees.to_string(),
            t.planned_rr.map(|v| v.to_string()).unwrap_or_default(),
            t.actual_rr.map(|v| v.to_string()).unwrap_or_default(),
            t.pnl_gross.map(|v| v.to_string()).unwrap_or_default(),
            t.pnl_net.map(|v| v.to_string()).unwrap_or_default(),
            t.pnl_percent.map(|v| v.to_string()).unwrap_or_default(),
            t.plan_setup_reason.unwrap_or_default(),
            t.plan_market_condition.unwrap_or_default(),
            t.plan_confidence.map(|v| v.to_string()).unwrap_or_default(),
            t.emotion_entry.unwrap_or_default(),
            t.emotion_exit.unwrap_or_default(),
            t.followed_plan.to_string(),
            t.mistake_tags.unwrap_or_default(),
            t.lesson_learned.unwrap_or_default(),
            t.notes.unwrap_or_default(),
            t.screenshot_entry_path.unwrap_or_default(),
            t.screenshot_exit_path.unwrap_or_default(),
            t.status,
            t.created_at,
            t.updated_at,
        ];
        let escaped: Vec<String> = row.iter().map(|v| format!("\"{}\"", v.replace('"', "\"\""))).collect();
        csv.push_str(&escaped.join(","));
        csv.push('\n');
    }

    Ok(csv)
}
