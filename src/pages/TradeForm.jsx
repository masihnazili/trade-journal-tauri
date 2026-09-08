import React, { useEffect, useState } from 'react';
import { useNavigate, useParams } from 'react-router-dom';
import { api } from '../lib/api';

const initialState = {
  symbol: '', direction: 'long', strategy_tag: '',
  entry_date: new Date().toISOString().slice(0, 10), entry_time: '',
  exit_date: '', exit_time: '', entry_price: '', exit_price: '',
  stop_loss: '', take_profit: '', position_size: '', fees: '0',
  planned_rr: '', plan_setup_reason: '', plan_market_condition: '', plan_confidence: 5,
  emotion_entry: 'آرام', emotion_exit: 'آرام', followed_plan: true,
  mistake_tags: '', lesson_learned: '', notes: '',
};

const EMOTIONS = ['آرام', 'مطمئن', 'مضطرب', 'ترسیده', 'طمع‌کار', 'بی‌حوصله', 'خسته', 'عصبانی'];
const MARKET_CONDITIONS = ['روند صعودی', 'روند نزولی', 'رنج/بدون روند', 'پرنوسان', 'کم‌نوسان'];

export default function TradeForm() {
  const [form, setForm] = useState(initialState);
  const navigate = useNavigate();
  const { id } = useParams();
  const isEdit = Boolean(id);

  useEffect(() => {
    if (isEdit) {
      api.trades.getById(id).then(trade => {
        if (trade) setForm({ ...trade, followed_plan: Boolean(trade.followed_plan), fees: String(trade.fees ?? '0') });
      });
    }
  }, [id]);

  const update = (field, value) => setForm(prev => ({ ...prev, [field]: value }));

  const handleSubmit = async (e) => {
    e.preventDefault();
    if (!form.symbol || !form.entry_price || !form.position_size) {
      alert('نماد، قیمت ورود و حجم پوزیشن الزامی هستند.');
      return;
    }
    const payload = {
      ...form,
      entry_price: parseFloat(form.entry_price),
      exit_price: form.exit_price ? parseFloat(form.exit_price) : null,
      stop_loss: form.stop_loss ? parseFloat(form.stop_loss) : null,
      take_profit: form.take_profit ? parseFloat(form.take_profit) : null,
      position_size: parseFloat(form.position_size),
      fees: parseFloat(form.fees || 0),
      planned_rr: form.planned_rr ? parseFloat(form.planned_rr) : null,
      plan_confidence: parseInt(form.plan_confidence, 10),
      followed_plan: Boolean(form.followed_plan),
    };
    if (isEdit) await api.trades.update(id, payload);
    else await api.trades.create(payload);
    navigate('/trades');
  };

  return (
    <div className="page">
      <h1>{isEdit ? 'ویرایش معامله' : 'ثبت معامله جدید'}</h1>
      <form className="trade-form" onSubmit={handleSubmit}>
        <section className="form-section">
          <h2>۱. اطلاعات پایه</h2>
          <div className="form-grid">
            <Field label="نماد *"><input value={form.symbol} onChange={e => update('symbol', e.target.value.toUpperCase())} placeholder="مثلاً EURUSD, BTCUSDT" /></Field>
            <Field label="جهت *">
              <select value={form.direction} onChange={e => update('direction', e.target.value)}>
                <option value="long">خرید (Long)</option>
                <option value="short">فروش (Short)</option>
              </select>
            </Field>
            <Field label="تگ استراتژی"><input value={form.strategy_tag} onChange={e => update('strategy_tag', e.target.value)} placeholder="مثلاً Break & Retest" /></Field>
            <Field label="حجم پوزیشن *"><input type="number" step="any" value={form.position_size} onChange={e => update('position_size', e.target.value)} /></Field>
          </div>
        </section>

        <section className="form-section">
          <h2>۲. پلن پیش از معامله</h2>
          <div className="form-grid">
            <Field label="دلیل ورود / ستاپ"><textarea value={form.plan_setup_reason} onChange={e => update('plan_setup_reason', e.target.value)} rows={2} /></Field>
            <Field label="وضعیت بازار">
              <select value={form.plan_market_condition} onChange={e => update('plan_market_condition', e.target.value)}>
                <option value="">انتخاب کنید</option>
                {MARKET_CONDITIONS.map(m => <option key={m} value={m}>{m}</option>)}
              </select>
            </Field>
            <Field label="سطح اطمینان (۱ تا ۱۰)">
              <input type="range" min="1" max="10" value={form.plan_confidence} onChange={e => update('plan_confidence', e.target.value)} />
              <span className="range-value">{form.plan_confidence}</span>
            </Field>
            <Field label="R:R برنامه‌ریزی‌شده"><input type="number" step="any" value={form.planned_rr} onChange={e => update('planned_rr', e.target.value)} /></Field>
          </div>
        </section>

        <section className="form-section">
          <h2>۳. اجرا و قیمت‌گذاری</h2>
          <div className="form-grid">
            <Field label="تاریخ ورود *"><input type="date" value={form.entry_date} onChange={e => update('entry_date', e.target.value)} /></Field>
            <Field label="ساعت ورود"><input type="time" value={form.entry_time} onChange={e => update('entry_time', e.target.value)} /></Field>
            <Field label="قیمت ورود *"><input type="number" step="any" value={form.entry_price} onChange={e => update('entry_price', e.target.value)} /></Field>
            <Field label="حد ضرر (Stop Loss)"><input type="number" step="any" value={form.stop_loss} onChange={e => update('stop_loss', e.target.value)} /></Field>
            <Field label="حد سود (Take Profit)"><input type="number" step="any" value={form.take_profit} onChange={e => update('take_profit', e.target.value)} /></Field>
            <Field label="تاریخ خروج"><input type="date" value={form.exit_date || ''} onChange={e => update('exit_date', e.target.value)} /></Field>
            <Field label="ساعت خروج"><input type="time" value={form.exit_time || ''} onChange={e => update('exit_time', e.target.value)} /></Field>
            <Field label="قیمت خروج"><input type="number" step="any" value={form.exit_price || ''} onChange={e => update('exit_price', e.target.value)} /></Field>
            <Field label="کارمزد"><input type="number" step="any" value={form.fees} onChange={e => update('fees', e.target.value)} /></Field>
          </div>
        </section>

        <section className="form-section">
          <h2>۴. روانشناسی و پایبندی به پلن</h2>
          <div className="form-grid">
            <Field label="احساس هنگام ورود">
              <select value={form.emotion_entry} onChange={e => update('emotion_entry', e.target.value)}>
                {EMOTIONS.map(em => <option key={em} value={em}>{em}</option>)}
              </select>
            </Field>
            <Field label="احساس هنگام خروج">
              <select value={form.emotion_exit} onChange={e => update('emotion_exit', e.target.value)}>
                {EMOTIONS.map(em => <option key={em} value={em}>{em}</option>)}
              </select>
            </Field>
            <Field label="آیا طبق پلن عمل کردی؟">
              <label className="checkbox-label">
                <input type="checkbox" checked={form.followed_plan} onChange={e => update('followed_plan', e.target.checked)} />
                بله، طبق پلن عمل کردم
              </label>
            </Field>
            <Field label="تگ اشتباه (در صورت وجود)"><input value={form.mistake_tags} onChange={e => update('mistake_tags', e.target.value)} placeholder="مثلاً ورود زودهنگام، حجم بیش‌ازحد" /></Field>
          </div>
        </section>

        <section className="form-section">
          <h2>۵. ریویو و یادداشت</h2>
          <div className="form-grid">
            <Field label="درس گرفته‌شده"><textarea value={form.lesson_learned} onChange={e => update('lesson_learned', e.target.value)} rows={2} /></Field>
            <Field label="یادداشت آزاد"><textarea value={form.notes} onChange={e => update('notes', e.target.value)} rows={2} /></Field>
          </div>
        </section>

        <div className="form-actions">
          <button type="button" className="btn-secondary" onClick={() => navigate('/trades')}>انصراف</button>
          <button type="submit" className="btn-primary">{isEdit ? 'ذخیره تغییرات' : 'ثبت معامله'}</button>
        </div>
      </form>
    </div>
  );
}

function Field({ label, children }) {
  return (<div className="field"><label>{label}</label>{children}</div>);
}
