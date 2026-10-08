# Proteus Sovereign BOS — Pilot Shop Onboarding & Operational Manual

> **Scope:** Field Onboarding, Daily Workflows & Autonomous Maintenance for 2–3 Local Repair & Retail Shops (Greece)  
> **Target Audience:** Shop Owners, Head Technicians, Cash Desk Operators  
> **Statutory Compliance:** Greek Law 4808/2021 (Ergani II), AADE myDATA (A.1155/2023), GDPR & Greek Law 4624/2019  

---

## 1. Zero-Friction System Deployment (Hardware & Software)

### A. System Requirements & Zero-Privilege Execution
- **Operating System:** Windows 10/11 (64-bit), Linux (Ubuntu 22.04+), or macOS (Intel/Apple Silicon).
- **User Privileges:** Standard user account (Zero administrator rights required).
- **Data Location:** All state, schemas, and event queues reside in `%APPDATA%\Proteus\data\store.db`.
- **Zero Cloud Lock-in:** The system runs fully autonomous offline. No mandatory internet connection is required to create tickets, ring sales, or print receipts.

### B. Hardware Interconnection Setup
1. **Thermal Receipt Printer (58mm / 80mm):**
   - Connect via standard USB.
   - Configure as the default thermal receipt spooler (`RAW` or generic text driver).
   - Test print via Proteus Settings: verify Greek codepage (CP737 / CP869) and 2D QR code barcode rendering.
2. **Barcode & 2D QR Scanner:**
   - Plug into USB; runs in standard USB-HID keyboard emulation mode.
   - Scans serial numbers, IMEIs, and intake receipt QR codes with instant acoustic confirmation.
3. **EFT-POS Cash Register Interlock (AADE A.1155/2023):**
   - Connect LAN/USB cable from the certified payment terminal to the shop PC.
   - Payment lock activates automatically upon card receipt issuance.

---

## 2. Daily Operational Runbook

### Workflow 1: Customer Intake (Νέα Παραλαβή)
1. Navigate to **Service Bench $\rightarrow$ Νέα Παραλαβή**.
2. Scan or enter customer details: Full Name, Phone Number, and Optional Email (GDPR consent toggle).
3. Record device information: Category, Brand, Model, Serial/IMEI, and Fault Description.
4. Set Estimated Cost and Customer Approval limit.
5. Click **«Έκδοση Δελτίου Παραλαβής»**:
   - Ticket UUIDv7 is generated with monotonic sequence.
   - Thermal receipt prints with shop header, ticket number, device barcode, and customer tear-off slip.
   - Initial lane set to `Received` (Παραλήφθηκε).

### Workflow 2: Workshop Diagnostics & Repair Execution
1. Open the visual 6-lane Kanban board:
   - `Received` $\rightarrow$ `In Progress` $\rightarrow$ `Waiting Parts` $\rightarrow$ `Ready` $\rightarrow$ `Delivered` $\rightarrow$ `Cancelled`.
2. Move ticket card into `In Progress` upon opening the device.
3. Add parts consumed from local inventory and labor hours.
4. Mark internal technician diagnostic notes and customer-facing repair report.
5. When repair is verified, drag card to `Ready` (Έτοιμο προς Παράδοση). Automated SMS/Viber alert queued.

### Workflow 3: Delivery & 1-Click Fiscal Settlement (myDATA)
1. Customer presents intake ticket or phone number.
2. Scan intake QR code to summon ticket details.
3. Click **«1-Click Ολοκλήρωση & Έκδοση Απόδειξης»**:
   - Settlement calculates Net + 24% VAT automatically.
   - Payment method selected (Cash or Card).
   - If Card: triggers AADE A.1155 EFT-POS lock.
   - Digital voucher generated with cryptographic MARK and UID.
   - ESC/POS thermal printer produces official tax receipt with verification QR code.
   - Ticket transitions to `Delivered`.

### Workflow 4: Digital Work Card Check-In (Ergani II)
1. Counter staff touches the PIN pad or scans employee QR badge on arrival.
2. Select action: **«Έναρξη Βάρδιας» (Clock In)**.
3. System logs timestamp in UTC and local time, hashes event into Merkle chain, and queues transmission to SEPE.
4. Breaks are logged via **«Έναρξη Διαλείμματος»** and **«Λήξη Διαλείμματος»**.
5. Shift ends with **«Λήξη Βάρδιας» (Clock Out)**. Overtime (>8h) is calculated live.

---

## 3. Incident Playbook & Offline Resilience

### Scenario A: Store Internet / Fiber Drop (Ανωτέρα Βία)
- **Immediate Action:** Keep operating as normal.
- **Ergani II Compliance:** Flip the **«Κατάσταση Ανωτέρας Βίας (Telecom Outage)»** toggle in Work Card view.
  - Events are stamped with the statutory outage flag under Law 4808/2021 & Circular 47319/2023.
  - Merkle hash chain guarantees sequential integrity without timestamps drifting.
- **myDATA Fiscal Invoicing:** Invoices and receipts buffer safely in `mydata_transmissions` table.
- **Recovery:** Once internet reconnects, the resilient Outbox Worker flushes all pending records with full jitter exponential backoff.

### Scenario B: Emergency Database Backup & Rollback
- Proteus automatically creates a `.bak` snapshot prior to any package mount or schema update.
- **Manual Snapshot:** Run `scripts/launcher.ps1` and select option `[6]` or click **«Εκτέλεση Άμεσου Backup στο Cloudflare R2»** in Settings.
- **Database Restoration:** In case of hardware swap, copy `%APPDATA%\Proteus\data\store.db` or import the latest zero-knowledge `.crmb` export.

---

## 4. Pilot Feedback & Telemetry Checklist

During the 6-month pilot, report any operational findings using the direct log file:
- **Log Location:** `%APPDATA%\Proteus\logs\proteus-client.log`
- **Weekly Checkpoints:**
  1. Intake ticket thermal print speed (< 1 second).
  2. Offline sync outbox drain rate upon network restoration.
  3. Barcode scanner accuracy on damaged serial labels.
  4. myDATA transmission status (all receipts marked with verified MARK).
