import React, { useEffect, useState } from 'react';
import { api } from '../lib/api';

const MONTH_NAMES = ['ژانویه','فوریه','مارس','آوریل','مه','ژوئن','ژوئیه','اوت','سپتامبر','اکتبر','نوامبر','دسامبر'];

export default function CalendarView() {
  const now = new Date();
  const [year, setYear] = useState(now.getFullYear());
  const [month, setMonth] = useState(now.getMonth() + 1);
  const [data, setData] = useState([]);

  useEffect(() => { api.stats.calendar(year, month).then(setData); }, [year, month]);

  const dayMap = {};
  data.forEach(d => { dayMap[d.entry_date] = d; });

  const daysInMonth = new Date(year, month, 0).getDate();
  const firstDayOfWeek = new Date(year, month - 1, 1).getDay();
  const cells = [];
  for (let i = 0; i < firstDayOfWeek; i++) cells.push(null);
  for (let d = 1; d <= daysInMonth; d++) cells.push(d);

  const changeMonth = (delta) => {
    let newMonth = month + delta, newYear = year;
    if (newMonth > 12) { newMonth = 1; newYear += 1; }
    if (newMonth < 1) { newMonth = 12; newYear -= 1; }
    setMonth(newMonth); setYear(newYear);
  };

  const monthPnl = data.reduce((sum, d) => sum + (d.day_pnl || 0), 0);

  return (
    <div className="page">
      <div className="page-header">
        <h1>تقویم معاملاتی</h1>
        <div className="header-actions">
          <button className="btn-secondary" onClick={() => changeMonth(-1)}>ماه قبل</button>
          <span className="month-label">{MONTH_NAMES[month - 1]} {year}</span>
          <button className="btn-secondary" onClick={() => changeMonth(1)}>ماه بعد</button>
        </div>
      </div>
      <p className={monthPnl >= 0 ? 'text-green' : 'text-red'} style={{ marginBottom: '1rem' }}>سود/زیان این ماه: {monthPnl.toFixed(2)}</p>
      <div className="calendar-grid">
        {['یک','دو','سه','چهار','پنج','جمعه','شنبه'].map(d => (<div key={d} className="calendar-day-header">{d}</div>))}
        {cells.map((day, idx) => {
          if (day === null) return <div key={idx} className="calendar-cell empty" />;
          const dateStr = `${year}-${String(month).padStart(2, '0')}-${String(day).padStart(2, '0')}`;
          const info = dayMap[dateStr];
          const pnlClass = info ? (info.day_pnl >= 0 ? 'day-positive' : 'day-negative') : '';
          return (
            <div key={idx} className={`calendar-cell ${pnlClass}`}>
              <div className="cell-date">{day}</div>
              {info && (<><div className="cell-pnl">{info.day_pnl.toFixed(1)}</div><div className="cell-count">{info.trade_count} معامله</div></>)}
            </div>
          );
        })}
      </div>
    </div>
  );
}
