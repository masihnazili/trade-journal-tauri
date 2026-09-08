import React, { useEffect, useState } from 'react';
import { Link, useNavigate } from 'react-router-dom';
import { api } from '../lib/api';

export default function TradesList() {
  const [trades, setTrades] = useState([]);
  const [filter, setFilter] = useState('');
  const navigate = useNavigate();
  const load = () => api.trades.getAll().then(setTrades);
  useEffect(() => { load(); }, []);

  const handleDelete = async (id) => {
    if (!confirm('این معامله حذف شود؟')) return;
    await api.trades.delete(id);
    load();
  };

  const handleExport = async () => {
    const csv = await api.trades.exportCsv();
    if (!csv) { alert('معامله‌ای برای خروجی وجود ندارد.'); return; }
    const blob = new Blob([csv], { type: 'text/csv;charset=utf-8;' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `trades_export_${Date.now()}.csv`;
    a.click();
    URL.revokeObjectURL(url);
  };

  const filtered = trades.filter(t =>
    t.symbol.toLowerCase().includes(filter.toLowerCase()) ||
    (t.strategy_tag || '').toLowerCase().includes(filter.toLowerCase())
  );

  return (
    <div className="page">
      <div className="page-header">
        <h1>لیست معاملات</h1>
        <div className="header-actions">
          <input className="input-search" placeholder="جستجو بر اساس نماد یا استراتژی..." value={filter} onChange={e => setFilter(e.target.value)} />
          <button className="btn-secondary" onClick={handleExport}>خروجی CSV</button>
          <button className="btn-primary" onClick={() => navigate('/trades/new')}>+ معامله جدید</button>
        </div>
      </div>
      <div className="table-wrapper">
        <table>
          <thead>
            <tr>
              <th>تاریخ</th><th>نماد</th><th>جهت</th><th>استراتژی</th><th>ورود</th><th>خروج</th>
              <th>سود/زیان خالص</th><th>R:R واقعی</th><th>پایبند به پلن</th><th>عملیات</th>
            </tr>
          </thead>
          <tbody>
            {filtered.map(t => (
              <tr key={t.id}>
                <td>{t.entry_date}</td>
                <td>{t.symbol}</td>
                <td><span className={`badge ${t.direction === 'long' ? 'badge-long' : 'badge-short'}`}>{t.direction === 'long' ? 'خرید' : 'فروش'}</span></td>
                <td>{t.strategy_tag || '-'}</td>
                <td>{t.entry_price}</td>
                <td>{t.exit_price ?? '-'}</td>
                <td className={t.pnl_net > 0 ? 'text-green' : t.pnl_net < 0 ? 'text-red' : ''}>{t.pnl_net != null ? t.pnl_net.toFixed(2) : '-'}</td>
                <td>{t.actual_rr != null ? t.actual_rr.toFixed(2) : '-'}</td>
                <td>{t.followed_plan ? '✅' : '❌'}</td>
                <td className="row-actions">
                  <Link to={`/trades/${t.id}/edit`}>ویرایش</Link>
                  <button className="link-danger" onClick={() => handleDelete(t.id)}>حذف</button>
                </td>
              </tr>
            ))}
            {filtered.length === 0 && (<tr><td colSpan="10" className="muted center">معامله‌ای یافت نشد.</td></tr>)}
          </tbody>
        </table>
      </div>
    </div>
  );
}
