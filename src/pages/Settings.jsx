import React, { useEffect, useState } from 'react';
import { api } from '../lib/api';
import { open } from '@tauri-apps/plugin-dialog';
import { invoke } from '@tauri-apps/api/core';

export default function Settings() {
  const [settings, setSettings] = useState({ starting_balance: '10000', currency: 'USD' });
  const [saved, setSaved] = useState(false);
  const [backupLoading, setBackupLoading] = useState(false);
  const [restoreLoading, setRestoreLoading] = useState(false);
  const [importLoading, setImportLoading] = useState(false);
  const [message, setMessage] = useState('');
  const [messageType, setMessageType] = useState(''); // 'success', 'error'

  useEffect(() => { 
    api.settings.get().then(setSettings); 
  }, []);

  const handleSave = async () => {
    await api.settings.update(settings);
    setSaved(true);
    setTimeout(() => setSaved(false), 2000);
  };

  const handleBackup = async () => {
    try {
      setBackupLoading(true);
      const backupDir = await open({
        directory: true,
        title: 'انتخاب مسیر ذخیره بک‌آپ',
      });

      if (backupDir) {
        const result = await invoke('backup_database', { 
          backupDir: backupDir 
        });
        setMessageType('success');
        setMessage(`✓ بک‌آپ با موفقیت ایجاد شد: ${result}`);
      }
    } catch (error) {
      setMessageType('error');
      setMessage(`✗ خطا در بک‌آپ: ${error}`);
    } finally {
      setBackupLoading(false);
    }
  };

  const handleRestore = async () => {
    try {
      setRestoreLoading(true);
      const file = await open({
        filters: [
          { name: 'Database Backup', extensions: ['db'] },
        ],
        title: 'انتخاب فایل بک‌آپ برای بازیابی',
      });

      if (file) {
        if (window.confirm('آیا مطمئن هستید؟ داده‌های فعلی با بک‌آپ جایگزین خواهند شد.')) {
          await invoke('restore_database', { 
            backupPath: file 
          });
          setMessageType('success');
          setMessage('✓ دیتابیس با موفقیت بازیابی شد. لطفاً اپلیکیشن را دوباره راه‌اندازی کنید.');
        }
      }
    } catch (error) {
      setMessageType('error');
      setMessage(`✗ خطا در بازیابی: ${error}`);
    } finally {
      setRestoreLoading(false);
    }
  };

  const handleImportMT = async () => {
    try {
      setImportLoading(true);
      const file = await open({
        filters: [
          { name: 'MT Statement', extensions: ['html', 'csv'] },
        ],
        title: 'انتخاب فایل صورت‌حساب MT4/MT5',
      });

      if (file) {
        const result = await invoke('import_mt_statement', { 
          filePath: file 
        });
        
        setMessageType('success');
        setMessage(
          `✓ وارد شده: ${result.total_imported} معامله\n` +
          (result.duplicates_skipped > 0 ? `تکراری رد شده: ${result.duplicates_skipped}\n` : '') +
          (result.errors.length > 0 ? `خطاها: ${result.errors.join(', ')}` : '')
        );
      }
    } catch (error) {
      setMessageType('error');
      setMessage(`✗ خطا در import: ${error}`);
    } finally {
      setImportLoading(false);
    }
  };

  return (
    <div className="page">
      <h1>تنظیمات</h1>
      
      {message && (
        <div className={`message ${messageType}`} style={{
          padding: '12px',
          marginBottom: '20px',
          borderRadius: '4px',
          backgroundColor: messageType === 'success' ? '#d4edda' : '#f8d7da',
          color: messageType === 'success' ? '#155724' : '#721c24',
          whiteSpace: 'pre-wrap',
        }}>
          {message}
        </div>
      )}

      <div className="form-section" style={{ maxWidth: 480 }}>
        <h2>تنظیمات اصلی</h2>
        <div className="form-grid">
          <div className="field">
            <label>سرمایه اولیه</label>
            <input 
              type="number" 
              value={settings.starting_balance} 
              onChange={e => setSettings({ ...settings, starting_balance: e.target.value })} 
            />
          </div>
          <div className="field">
            <label>واحد پول</label>
            <input 
              value={settings.currency} 
              onChange={e => setSettings({ ...settings, currency: e.target.value })} 
            />
          </div>
        </div>
        <button className="btn-primary" onClick={handleSave}>ذخیره تنظیمات</button>
        {saved && <span className="text-green" style={{ marginRight: 10 }}>ذخیره شد ✓</span>}
      </div>

      <div className="form-section" style={{ maxWidth: 480, marginTop: '2rem' }}>
        <h2>بک‌آپ و بازیابی</h2>
        <p className="muted">داده‌های شما را محفوظ نگه دارید:</p>
        <div style={{ display: 'flex', gap: '10px', flexDirection: 'column' }}>
          <button 
            className="btn-primary" 
            onClick={handleBackup}
            disabled={backupLoading}
            style={{ opacity: backupLoading ? 0.6 : 1 }}
          >
            {backupLoading ? 'درحال بک‌آپ...' : '💾 بک‌آپ اکنون'}
          </button>
          <button 
            className="btn-secondary" 
            onClick={handleRestore}
            disabled={restoreLoading}
            style={{ opacity: restoreLoading ? 0.6 : 1 }}
          >
            {restoreLoading ? 'درحال بازیابی...' : '⤴️ بازیابی از بک‌آپ'}
          </button>
        </div>
      </div>

      <div className="form-section" style={{ maxWidth: 480, marginTop: '2rem' }}>
        <h2>Import از MT4/MT5</h2>
        <p className="muted">معاملات را از صورت‌حساب MT وارد کنید:</p>
        <button 
          className="btn-primary" 
          onClick={handleImportMT}
          disabled={importLoading}
          style={{ opacity: importLoading ? 0.6 : 1 }}
        >
          {importLoading ? 'درحال Import...' : '📤 Import از MT Statement'}
        </button>
      </div>

      <p className="muted" style={{ marginTop: '2rem' }}>
        داده‌های شما به‌صورت کامل و محلی روی همین سیستم ذخیره می‌شوند و به هیچ سرور خارجی ارسال نمی‌شود.
      </p>
    </div>
  );
}
