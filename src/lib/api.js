import { invoke } from '@tauri-apps/api/core';

export const api = {
  trades: {
    getAll: () => invoke('get_all_trades'),
    getById: (id) => invoke('get_trade_by_id', { id }),
    create: (trade) => invoke('create_trade', { trade }),
    update: (id, trade) => invoke('update_trade', { id, trade }),
    delete: (id) => invoke('delete_trade', { id }),
    exportCsv: () => invoke('export_trades_csv'),
  },
  stats: {
    summary: () => invoke('get_summary_stats'),
    equityCurve: () => invoke('get_equity_curve'),
    calendar: (year, month) => invoke('get_calendar_data', { year, month }),
  },
  settings: {
    get: () => invoke('get_settings'),
    update: (settings) => invoke('update_settings', { settings }),
  },
};
