use serde::{Deserialize, Serialize};
use regex::Regex;
use std::collections::HashSet;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MTTrade {
    pub ticket: String,
    pub symbol: String,
    pub trade_type: String,     // "buy" or "sell"
    pub open_price: f64,
    pub close_price: Option<f64>,
    pub volume: f64,
    pub open_time: String,
    pub close_time: Option<String>,
    pub stop_loss: Option<f64>,
    pub take_profit: Option<f64>,
    pub commission: Option<f64>,
    pub swap: Option<f64>,
    pub profit: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ImportSummary {
    pub total_imported: usize,
    pub duplicates_skipped: usize,
    pub errors: Vec<String>,
}

/// Parse HTML statement from MT4/MT5
pub fn parse_mt_html(html_content: &str) -> Result<Vec<MTTrade>, String> {
    let mut trades = Vec::new();

    // Simple regex-based parsing for table rows
    // MT4/MT5 HTML statements typically have rows like:
    // <tr><td>ticket</td><td>symbol</td><td>type</td>...
    
    let row_pattern = Regex::new(r#"<tr[^>]*>(.*?)</tr>"#)
        .map_err(|e| format!("Regex error: {}", e))?;
    
    let cell_pattern = Regex::new(r#"<td[^>]*>(.*?)</td>"#)
        .map_err(|e| format!("Regex error: {}", e))?;

    for row_match in row_pattern.captures_iter(html_content) {
        let row_content = &row_match[1];
        let cells: Vec<String> = cell_pattern
            .captures_iter(row_content)
            .map(|m| {
                let text = m[1].to_string();
                // Remove HTML tags and entities
                text.replace("<br>", " ")
                    .replace("&nbsp;", " ")
                    .replace("<[^>]+>", "")
                    .trim()
                    .to_string()
            })
            .collect();

        if cells.len() < 8 {
            continue; // Skip incomplete rows
        }

        // Expected format (may vary):
        // [0] = Ticket, [1] = Symbol, [2] = Type, [3] = Open Price, 
        // [4] = Close Price, [5] = Volume, [6] = Open Time, [7] = Close Time, ...
        
        if let Ok(trade) = parse_mt_row(&cells) {
            trades.push(trade);
        }
    }

    if trades.is_empty() {
        return Err("No valid trades found in HTML file".to_string());
    }

    Ok(trades)
}

/// Parse CSV format (alternative to HTML)
pub fn parse_mt_csv(csv_content: &str) -> Result<Vec<MTTrade>, String> {
    let mut trades = Vec::new();
    let mut lines = csv_content.lines();
    
    // Skip header
    lines.next();

    for line in lines {
        let cells: Vec<&str> = line.split('\t').collect();
        if cells.len() < 8 {
            continue;
        }

        if let Ok(trade) = parse_mt_row(
            &cells.iter().map(|s| s.to_string()).collect::<Vec<_>>()
        ) {
            trades.push(trade);
        }
    }

    if trades.is_empty() {
        return Err("No valid trades found in CSV file".to_string());
    }

    Ok(trades)
}

fn parse_mt_row(cells: &[String]) -> Result<MTTrade, String> {
    if cells.len() < 8 {
        return Err("Insufficient cells in row".to_string());
    }

    let ticket = cells[0].trim().to_string();
    if ticket.is_empty() || ticket.to_lowercase().contains("ticket") {
        return Err("Invalid or header row".to_string());
    }

    let symbol = cells[1].trim().to_string();
    let trade_type = cells[2].trim().to_lowercase();
    
    if !trade_type.contains("buy") && !trade_type.contains("sell") {
        return Err("Invalid trade type".to_string());
    }

    let open_price: f64 = cells[3].trim().parse()
        .map_err(|_| "Invalid open price".to_string())?;
    
    let close_price: Option<f64> = cells[4].trim().parse().ok();
    let volume: f64 = cells[5].trim().parse()
        .map_err(|_| "Invalid volume".to_string())?;
    
    let open_time = cells[6].trim().to_string();
    let close_time = if cells.len() > 7 && !cells[7].trim().is_empty() {
        Some(cells[7].trim().to_string())
    } else {
        None
    };

    let stop_loss: Option<f64> = cells.get(8).and_then(|s| s.trim().parse().ok());
    let take_profit: Option<f64> = cells.get(9).and_then(|s| s.trim().parse().ok());
    let commission: Option<f64> = cells.get(10).and_then(|s| s.trim().parse().ok());
    let swap: Option<f64> = cells.get(11).and_then(|s| s.trim().parse().ok());
    let profit: Option<f64> = cells.get(12).and_then(|s| s.trim().parse().ok());

    Ok(MTTrade {
        ticket,
        symbol,
        trade_type: if trade_type.contains("buy") { "long".to_string() } else { "short".to_string() },
        open_price,
        close_price,
        volume,
        open_time,
        close_time,
        stop_loss,
        take_profit,
        commission,
        swap,
        profit,
    })
}

/// Check for duplicate trades based on ticket number
pub fn find_duplicates(tickets: &[String], existing_tickets: &HashSet<String>) -> HashSet<String> {
    tickets
        .iter()
        .filter(|t| existing_tickets.contains(*t))
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_mt_row_valid() {
        let cells = vec![
            "123456".to_string(),
            "EURUSD".to_string(),
            "buy".to_string(),
            "1.0850".to_string(),
            "1.0900".to_string(),
            "1.0".to_string(),
            "2024-01-15 10:30:00".to_string(),
            "2024-01-15 11:00:00".to_string(),
            "1.0800".to_string(),
            "1.0950".to_string(),
            "10.0".to_string(),
            "0.0".to_string(),
            "50.0".to_string(),
        ];

        let result = parse_mt_row(&cells);
        assert!(result.is_ok());
        let trade = result.unwrap();
        assert_eq!(trade.ticket, "123456");
        assert_eq!(trade.symbol, "EURUSD");
        assert_eq!(trade.trade_type, "long");
        assert_eq!(trade.open_price, 1.0850);
        assert_eq!(trade.close_price, Some(1.0900));
    }

    #[test]
    fn test_parse_mt_row_sell() {
        let cells = vec![
            "789012".to_string(),
            "GBPUSD".to_string(),
            "sell".to_string(),
            "1.2650".to_string(),
            "1.2600".to_string(),
            "0.5".to_string(),
            "2024-01-15 14:00:00".to_string(),
            "2024-01-15 14:30:00".to_string(),
            "1.2700".to_string(),
            "1.2550".to_string(),
            "5.0".to_string(),
            "0.0".to_string(),
            "25.0".to_string(),
        ];

        let result = parse_mt_row(&cells);
        assert!(result.is_ok());
        let trade = result.unwrap();
        assert_eq!(trade.trade_type, "short");
    }

    #[test]
    fn test_parse_mt_row_incomplete() {
        let cells = vec!["123456".to_string(), "EURUSD".to_string()];
        let result = parse_mt_row(&cells);
        assert!(result.is_err());
    }

    #[test]
    fn test_find_duplicates() {
        let new_tickets = vec!["111".to_string(), "222".to_string(), "333".to_string()];
        let mut existing = HashSet::new();
        existing.insert("222".to_string());

        let duplicates = find_duplicates(&new_tickets, &existing);
        assert_eq!(duplicates.len(), 1);
        assert!(duplicates.contains("222"));
    }

    #[test]
    fn test_find_no_duplicates() {
        let new_tickets = vec!["111".to_string(), "222".to_string()];
        let existing = HashSet::new();

        let duplicates = find_duplicates(&new_tickets, &existing);
        assert!(duplicates.is_empty());
    }
}
