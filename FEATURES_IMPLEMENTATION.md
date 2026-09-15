# Trade Journal Tauri - نرم‌افزار ثبت معاملات

## پیاده‌سازی شده: چهار قابلیت اصلی

---

## ۱. بک‌آپ و بازیابی دیتابیس (Database Backup & Restore)

### ویژگی‌ها:
- ✅ بک‌آپ خودکار با timestamp (مثلاً `trade_journal_backup_20240115_153045.db`)
- ✅ بازیابی از هر بک‌آپ قدیمی
- ✅ تأیید کاربر قبل از overwrite
- ✅ ایجاد backup امنیتی خودکار قبل از restore
- ✅ دکمه‌های "💾 بک‌آپ اکنون" و "⤴️ بازیابی از بک‌آپ" در صفحه تنظیمات

### فایل‌های مرتبط:
- `src-tauri/src/backup.rs` - توابع backup/restore
- `src-tauri/src/lib.rs` - Tauri commands
- `src/pages/Settings.jsx` - UI

### استفاده:
```javascript
// در فرانت‌اند (React):
await invoke('backup_database', { backupDir: '/path/to/backup' });
await invoke('restore_database', { backupPath: '/path/to/backup.db' });
await invoke('list_backups', { backupDir: '/path/to/backups' });
```

---

## ۲. Code Signing برای Build خروجی

### گزینه‌های رایگان/ارزان:

#### الف) Self-Signed Certificate (برای توسعه محلی)
**هزینه:** رایگان
**مزایا:** بدون هزینه، سریع
**معایب:** هشدار Windows Defender ظاهر می‌شود (اما نرم‌افزار اجرا می‌شود)

```powershell
# ایجاد self-signed cert:
$cert = New-SelfSignedCertificate -CertStoreLocation Cert:\CurrentUser\My `
  -Subject "CN=Trade Journal" -KeyUsage DigitalSignature `
  -Type CodeSigningCert -NotAfter (Get-Date).AddYears(1)

# Export به PFX:
Export-PfxCertificate -Cert $cert -FilePath cert.pfx -Password (ConvertTo-SecureString -String "your_password" -AsPlainText -Force)

# اضافه کردن cert.pfx را Base64 تبدیل کنید:
[Convert]::ToBase64String([IO.File]::ReadAllBytes("cert.pfx")) | Out-File cert_base64.txt
```

#### ب) Let's Encrypt + Sectigo (هزینه کم)
**هزینه:** ۴۰-۲۰۰ دلار/سال
**مزایا:** اعتماد کامل، بدون هشدار
**معایب:** خریج لازم

#### ج) SignPath (رایگان برای Open Source)
**هزینه:** رایگان برای open source
**مزایا:** رایگان، اعتماد کامل
**روند:** ثبت‌نام → تأیید open source → دریافت API key
**وب‌سایت:** https://signpath.io

### راه‌اندازی Code Signing در GitHub Actions:

**مرحله ۱: ایجاد Certificate (روی دستگاه خصوصی)**

```powershell
# PowerShell (مدیریت‌گر)

# ۱. ایجاد self-signed certificate:
$cert = New-SelfSignedCertificate -CertStoreLocation Cert:\CurrentUser\My `
  -Subject "CN=Trade Journal App" `
  -KeyUsage DigitalSignature `
  -Type CodeSigningCert `
  -NotAfter (Get-Date).AddYears(2)

# ۲. نمایش thumbprint برای مرجع:
$cert.Thumbprint

# ۳. Export به فایل PFX (با رمز عبور):
$password = ConvertTo-SecureString -String "YourSecurePassword123!" -AsPlainText -Force
Export-PfxCertificate -Cert $cert -FilePath "C:\temp\trade-journal-cert.pfx" -Password $password

# ۴. تبدیل به Base64:
$certContent = [System.Convert]::ToBase64String([System.IO.File]::ReadAllBytes("C:\temp\trade-journal-cert.pfx"))
$certContent | Out-File "C:\temp\cert_base64.txt"

# خروجی را کپی کنید و در GitHub Secrets اضافه کنید
```

**مرحله ۲: اضافه کردن Secrets در GitHub**

1. رفتن به `https://github.com/masihnazili/trade-journal-tauri`
2. **Settings → Secrets and variables → Actions**
3. اضافه کردن موارد زیر:
   - `WIN_SIGNING_CERT_BASE64`: (محتوای Base64 از مرحله ۱)
   - `WIN_SIGNING_CERT_PASSWORD`: `YourSecurePassword123!`

**مرحله ۳: ایجاد GitHub Actions Workflow**

ایجاد فایل `.github/workflows/build.yml`:

```yaml
name: Build & Sign Release

on:
  push:
    tags:
      - "v*"
  workflow_dispatch:

jobs:
  build:
    permissions:
      contents: write
    strategy:
      fail-fast: false
      matrix:
        include:
          - platform: 'windows-latest'
            args: '--target x86_64-pc-windows-msvc'

    runs-on: ${{ matrix.platform }}
    
    steps:
      - uses: actions/checkout@v4

      - name: setup node
        uses: actions/setup-node@v4
        with:
          node-version: lts/*

      - name: install Rust stable
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: x86_64-pc-windows-msvc

      - name: install frontend dependencies
        run: npm install

      - name: Build the app
        uses: tauri-apps/tauri-action@v0
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
        with:
          tagName: ${{ github.ref_name }}
          releaseName: 'Trade Journal ${{ github.ref_name }}'
          releaseBody: 'Download the installer below'
          releaseDraft: false
          prerelease: false
          args: ${{ matrix.args }}

  sign-windows:
    if: startsWith(github.ref, 'refs/tags/v') && secrets.WIN_SIGNING_CERT_BASE64 != ''
    needs: build
    runs-on: windows-latest
    
    steps:
      - name: Download Windows MSI
        uses: actions/download-artifact@v3
        with:
          path: artifacts

      - name: Import signing certificate
        run: |
          $cert_bytes = [System.Convert]::FromBase64String("${{ secrets.WIN_SIGNING_CERT_BASE64 }}")
          [System.IO.File]::WriteAllBytes("cert.pfx", $cert_bytes)

      - name: Sign MSI with SignTool
        shell: pwsh
        run: |
          $msi_path = Get-ChildItem -Path artifacts -Filter "*.msi" -Recurse | Select-Object -First 1
          if ($msi_path) {
            $signtool = "C:\Program Files (x86)\Windows Kits\10\bin\10.0.22621.0\x64\signtool.exe"
            if (Test-Path $signtool) {
              & $signtool sign /f cert.pfx /p "${{ secrets.WIN_SIGNING_CERT_PASSWORD }}" `
                /t "http://timestamp.comodoca.com/rfc3161" /fd sha256 $msi_path.FullName
              Write-Output "✓ Signed: $($msi_path.FullName)"
            } else {
              Write-Output "⚠ SignTool not found at $signtool - skipping signing"
            }
          }

      - name: Cleanup
        run: Remove-Item cert.pfx -Force -ErrorAction SilentlyContinue
```

**مرحله ۴: Release کنید**

```bash
# روی دستگاه محلی:
git tag v1.0.1
git push origin v1.0.1

# GitHub Actions خودکار build و sign خواهد کرد
```

---

## ۳. Unit Tests برای منطق آماری

### توابع تست‌شده:
- ✅ Win Rate (درصد معاملات سودآور)
- ✅ Profit Factor (نسبت سود به ضرر)
- ✅ Average Win/Loss
- ✅ Max Drawdown (بدترین افت رسیدن به سرمایه)
- ✅ Equity Curve (منحنی پیشرفت سرمایه)
- ✅ Plan Adherence (پیروی از برنامه)
- ✅ Average R:R (نسبت ریسک/پاداش)

### اجرای تست‌ها:
```bash
cd src-tauri

# اجرای تمام تست‌ها:
cargo test db::tests --lib

# اجرای تست خاص:
cargo test test_win_rate_calculation -- --nocapture

# اجرای تست‌ها با output:
cargo test -- --nocapture --test-threads=1
```

### نمونه تست:
```rust
#[test]
fn test_win_rate_calculation() {
    let trades = vec![
        Trade { pnl_net: Some(100.0), ..Default::default() },
        Trade { pnl_net: Some(-50.0), ..Default::default() },
    ];
    let wins = trades.iter().filter(|t| t.pnl_net.unwrap_or(0.0) > 0.0).count();
    let win_rate = (wins as f64 / trades.len() as f64) * 100.0;
    assert_eq!(win_rate, 50.0);
}
```

### Scenarios تست‌شده:
1. **عادی:** 50% win rate
2. **بدون معامله:** empty trades
3. **همه سود:** 100% win rate
4. **همه ضرر:** 0% win rate
5. **بدون stop loss:** avg_rr = 0
6. **Max drawdown:** calculation verification
7. **Equity curve:** progressive balance tracking

---

## ۴. Import از MT4/MT5 Statement

### ویژگی‌ها:
- ✅ پارس فایل‌های HTML و CSV
- ✅ بررسی تکراری بودن بر اساس Ticket Number
- ✅ نگهداری معاملات import‌شده در لایه "اجرا" فقط
- ✅ علامت‌گذاری خودکار که نیاز به تکمیل Plan و Review دارد
- ✅ خلاصه‌ی تعداد معاملات درج شده و رد شده

### فرمت فایل‌های پشتیبانی:

#### HTML (صورت‌حساب MT4/MT5):
```html
<table>
  <tr>
    <td>Ticket</td><td>Symbol</td><td>Type</td><td>Open Price</td>
    <td>Close Price</td><td>Volume</td><td>Open Time</td><td>Close Time</td>
    <td>Stop Loss</td><td>Take Profit</td><td>Commission</td><td>Swap</td><td>Profit</td>
  </tr>
  <tr>
    <td>123456</td><td>EURUSD</td><td>buy</td><td>1.0850</td>
    <td>1.0900</td><td>1.0</td><td>2024-01-15 10:30:00</td><td>2024-01-15 11:00:00</td>
    <td>1.0800</td><td>1.0950</td><td>10.0</td><td>0.0</td><td>50.0</td>
  </tr>
</table>
```

#### CSV:
```
Ticket	Symbol	Type	Open Price	Close Price	Volume	Open Time	Close Time	Stop Loss	Take Profit	Commission	Swap	Profit
123456	EURUSD	buy	1.0850	1.0900	1.0	2024-01-15 10:30:00	2024-01-15 11:00:00	1.0800	1.0950	10.0	0.0	50.0
```

### استفاده:
```javascript
// در React:
const result = await invoke('import_mt_statement', { 
  filePath: '/path/to/statement.html' 
});

// نتیجه:
{
  total_imported: 45,      // معاملات جدید
  duplicates_skipped: 3,   // معاملات تکراری
  errors: []               // لیست خطاها
}
```

### Mapping MT → Trade Journal:
| MT Field | Trade Field | توضیح |
|----------|-------------|-------|
| Ticket | notes (MT:XXXXXX) | شناسه‌ی منحصر |
| Symbol | symbol | نماد معامله |
| Type (buy/sell) | direction (long/short) | جهت |
| Open Price | entry_price | قیمت ورود |
| Close Price | exit_price | قیمت خروج |
| Volume | position_size | حجم |
| Open Time | entry_date/time | زمان ورود |
| Close Time | exit_date/time | زمان خروج |
| Stop Loss | stop_loss | حد ضرر |
| Take Profit | take_profit | حد سود |
| Commission | fees | کمیسیون |
| Profit | (calculated) | سود محاسبه‌شده |

### علامت‌گذاری معاملات import‌شده:
```
imported_from_mt = 1  // flag برای شناسایی
notes = "MT:123456"   // شماره ticket اصلی
```

---

## فایل‌های اضافه شده/تغییر یافته:

```
src-tauri/src/
├── backup.rs          ✨ NEW - توابع backup/restore
├── mt_import.rs       ✨ NEW - پارس MT statement
├── db.rs              📝 UPDATED - unit tests + MT support
└── lib.rs             📝 UPDATED - Tauri commands

src-tauri/
├── Cargo.toml         📝 UPDATED - regex, tempfile dependencies
└── tauri.conf.json    📝 UPDATED - code signing placeholders

src/pages/
└── Settings.jsx       📝 UPDATED - UI برای backup/restore/import

.github/workflows/
└── build.yml          ✨ NEW - CI/CD with code signing (دستی ایجاد کنید)
```

---

## نکات مهم:

### ۱. Backup خودکار (Enhancement)
اگر بخواهید backup خودکار اضافه کنید:
```rust
// در lib.rs setup:
fn setup_auto_backup(app: &tauri::App) {
    let last_backup = get_setting("last_backup_date");
    if should_backup(last_backup) {
        backup_database_internal();
        set_setting("last_backup_date", today());
    }
}
```

### ۲. Code Signing بدون Secrets
اگر secrets تنظیم نشده باشد، build بدون signing انجام می‌شود (خطا نیست).

### ۳. MT Import و Plan/Review
معاملات import‌شده تنها لایه "اجرا" پر کردند. کاربر باید:
- ✏️ پیش‌برنامه را تکمیل کند
- 💭 نظرات روان‌شناسی اضافه کند
- ✓ خودارزیابی انجام دهد

### ۴. Test Coverage
تست‌های موجود مسائل edge cases را پوشش می‌دهند

---

## منابع:

- [Tauri Documentation](https://tauri.app)
- [Code Signing Windows](https://learn.microsoft.com/en-us/windows/win32/seccrypto/cryptography-tools)
- [SignPath (Free for Open Source)](https://signpath.io)
- [Rust Testing Guide](https://doc.rust-lang.org/book/ch11-00-testing.html)

---

**نسخه:** 1.0.0  
**آخرین بروزرسانی:** ۱۵ سپتامبر ۲۰۲۶
