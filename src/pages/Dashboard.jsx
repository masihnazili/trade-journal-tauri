import React, { useEffect, useState } from 'react';
import { LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer } from 'recharts';
import { api } from '../lib/api';

export default function Dashboard() {
  const [stats, setStats] = useState(null);
  const [equity, setEquity] = useState([]);

  useEffect(() => {
    api.stats.summary().then(setStats);
    api.stats.equityCurve().then(setEquity);
  }, []);

  if (!stats) return <div className="page">در حال بارگذاری...</div>;
  const fmt = (n) => (n == null || isNaN(n) ? '0' : n.toLocaleString('en-US', { maximumFractionDigits: 2 }));

  return (
    <div className="page">
      <h1>داشبورد</h1>
      <div className="stat-grid">
        <StatCard label="تعداد کل معاملات" value={stats.total_trades} />
        <StatCard label="نرخ برد" value={`${fmt(stats.win_rate)}%`} highlight={stats.win_rate >= 50} />
        <StatCard label="سود خالص کل" value={fmt(stats.total_pnl)} highlight={stats.total_pnl >= 0} />
        <StatCard label="میانگین سود" value={fmt(stats.avg_win)} />
        <StatCard label="میانگین زیان" value={fmt(stats.avg_loss)} />
        <StatCard label="ضریب سود (Profit Factor)" value={fmt(stats.profit_factor)} />
        <StatCard label="میانگین R:R واقعی" value={fmt(stats.avg_rr)} />
        <StatCard label="بیشترین افت سرمایه" value={fmt(stats.max_drawdown)} />
        <StatCard label="پایبندی به پلن" value={`${fmt(stats.plan_adherence)}%`} highlight={stats.plan_adherence >= 80} />
      </div>
      <div className="chart-card">
        <h2>منحنی رشد سرمایه (Equity Curve)</h2>
        {equity.length === 0 ? (
          <p className="muted">هنوز معامله‌ای ثبت نشده است.</p>
        ) : (
          <ResponsiveContainer width="100%" height={320}>
            <LineChart data={equity}>
              <CartesianGrid strokeDasharray="3 3" stroke="#2a2f3a" />
              <XAxis dataKey="date" tick={{ fontSize: 11 }} />
              <YAxis tick={{ fontSize: 11 }} />
              <Tooltip />
              <Line type="monotone" dataKey="balance" stroke="#4f8cff" strokeWidth={2} dot={false} />
            </LineChart>
          </ResponsiveContainer>
        )}
      </div>
    </div>
  );
}

function StatCard({ label, value, highlight }) {
  return (<div className={`stat-card ${highlight ? 'positive' : ''}`}><div className="stat-label">{label}</div><div className="stat-value">{value}</div></div>);
}
