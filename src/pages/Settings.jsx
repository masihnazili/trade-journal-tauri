import React, { useEffect, useState } from 'react';
import { api } from '../lib/api';

export default function Settings() {
  const [settings, setSettings] = useState({ starting_balance: '10000', currency: 'USD' });
  const [saved, setSaved] = useState(false);

  useEffect(() => { api.settings.get().then(setSettings); }, []);

  const handleSave = async () => {
    await api.settings.update(settings);
    setSaved(true);
    setTimeout(() => setSaved(false), 2000);
  };

  return (
    <div className="page">
      <h1>تنظیمات</h1>
      <div className="form-section" style={{ maxWidth: 480 }}>
        <div className="form-grid">
          <div className="field">
            <label>سرمایه اولیه</label>
            <input type="number" value={settings.starting_balance} onChange={e => setSettings({ ...settings, starting_balance: e.target.value })} />
          </div>
          <div className="field">
            <label>واحد پول</label>
            <input value={settings.currency} onChange={e => setSettings({ ...settings, currency: e.target.value })} />
          </div>
        </div>
        <button className="btn-primary" onClick={handleSave}>ذخیره تنظیمات</button>
        {saved && <span className="text-green" style={{ marginRight: 10 }}>ذخیره شد ✓</span>}
      </div>
      <p className="muted" style={{ marginTop: '2rem' }}>داده‌های شما به‌صورت کامل و محلی روی همین سیستم ذخیره می‌شوند و به هیچ سروری ارسال نمی‌شوند.</p>
    </div>
  );
}
