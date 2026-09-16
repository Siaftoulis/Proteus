# 20 - The Anti-SAP Manifesto & Hierarchical Multi-Store Architecture

---

## 1. Το Όραμα της Ανατροπής (The Anti-SAP Disruptor)

### 1.1 Η Παθογένεια του Παραδοσιακού ERP Μονοπωλίου
Για περισσότερες από τέσσερις δεκαετίες, η παγκόσμια αγορά του επιχειρησιακού λογισμικού (Enterprise Resource Planning - ERP) κυριαρχείται από μια ολιγοπωλιακή ελίτ με επικεφαλής τη **SAP** και τη **Microsoft/Oracle**. Ενώ ξεκίνησαν ως τεχνολογικοί πρωτοπόροι, έχουν μετατραπεί σε **μονοπωλιακούς φοροεισπράκτορες (rent-seekers)** που εκμεταλλεύονται το μέγεθός τους εις βάρος των επιχειρήσεων:

1. **Αστρονομικό Κόστος & Εκβιαστικό Licensing ("The Seat Tax"):**
   - Η προσθήκη ενός απλού υπαλλήλου ταμείου ή αποθήκης επιβαρύνεται με ετήσια κόστη αδειών από 1.500€ έως 4.000€ ανά θέση, συν 22% ετήσια "τέλη συντήρησης".
   - Εάν μια λιανεμπορική αλυσίδα (π.χ. τύπου **Public**, με 30 πολυκαταστήματα) θελήσει να προσλάβει 50 εποχιακούς πωλητές για την περίοδο των εορτών ή του Black Friday, η SAP την εκβιάζει με ετήσια συμβόλαια δέσμευσης και τεράστια penalties.
2. **Δυσνόητη, Αποκρουστική Εμπειρία Χρήστη (1990s Cognitive Fatigue):**
   - Το περιβάλλον του SAP GUI παραμένει δαιδαλώδες, αργό, γκρίζο και γεμάτο αδιαπέραστους κωδικούς συναλλαγών (T-Codes όπως `VA01`, `ME21N`, `MIGO`).
   - Για να εκτελέσει ένας εργαζόμενος μια απλή παραλαβή ή έκδοση δελτίου, απαιτούνται εβδομάδες εκπαίδευσης και δεκάδες περιττά κλικ. Η έννοια της αφής ή της σύγχρονης ταχείας εξυπηρέτησης είναι ανύπαρκτη.
3. **Ομηρία μέσω των "Big-4" Συμβούλων (Vendor Lock-in):**
   - Για να αλλάξει ένα απλό πεδίο σε μια φόρμα ή να προστεθεί μια στήλη σε μια αναφορά, η επιχείρηση αναγκάζεται να πληρώνει εξωτερικές συμβουλευτικές εταιρείες (Deloitte, Accenture, PwC) με ωρομίσθια 150€–300€/ώρα.
   - Τα projects υλοποίησης διαρκούν από 12 έως 24 μήνες, συχνά ξεπερνούν τον προϋπολογισμό κατά 200% και καταλήγουν σε αποτυχία ή παρατεταμένη δυσλειτουργία.

### 1.2 Η Αποστολή του Proteus BOS
Το **Proteus Business Operating System (BOS)** δημιουργήθηκε για να σπάσει αυτό το μονοπώλιο:
> **Στόχος:** Να προσφέρει την πλήρη αρχιτεκτονική ισχύ, τον έλεγχο, την ασφάλεια και την ιχνηλασιμότητα ενός Tier-1 ERP, συνδυασμένη με την αστραπιαία ταχύτητα, την κομψότητα και την ευκολία χρήσης του Apple UX — σε ένα κλάσμα του κόστους, με ελεύθερο πνεύμα (free-spirit) και ανοιχτό, δίκαιο οικοσύστημα.

---

## 2. Αναλυτική Σύγκριση: SAP vs. Proteus BOS

| Άξονας Αξιολόγησης | SAP Enterprise ERP | Proteus BOS |
| :--- | :--- | :--- |
| **Τεχνολογική Βάση** | Legacy ABAP / Java / NetWeaver (βαριά runtimes, τεράστια κατανάλωση RAM) | **100% Native Rust** (egui GPU-accelerated rendering, 60fps, μνήμη < 45MB) |
| **Μέγεθος Εγκατάστασης** | Gigabytes από runtimes, dependencies, SAP GUI, database clients | **Αυτόνομο εκτελέσιμο 5.0 MB (`proteus-client.exe`)**, μηδενικά external dependencies |
| **Χρόνος Υλοποίησης** | 6 έως 24 μήνες με στρατιές συμβούλων | **One-Click Setup**: Έτοιμο για συναλλαγές σε **5 λεπτά** |
| **Ευκολία Χρήστη (UX)** | Χιλιάδες T-Codes, πυκνά inputs, δυσνόητα μενού | **"Average Joe" Principle**: Luxury dark, touch-friendly, ολοκλήρωση εργασίας σε <30s |
| **Κόστος Αδειών (Licensing)** | 2.500€–5.000€ ανά χρήστη upfront + 22% ετήσια συντήρηση | **Δίκαιο Cost-Plus Cloud**: 0€ άσκοπο seat tax, κλιμάκωση μόνο βάσει πραγματικού κόστους servers (1€–1.50€/χρήστη) |
| **Πρόσβαση εκτός δικτύου (Offline)** | Αδύνατη. Απαιτείται μόνιμη σύνδεση με HANA servers | **100% Offline-First**: Τοπική κρυπτογραφημένη SQLite (`store.db`), αυτόματο background P2P sync |
| **Ασφάλεια Δεδομένων** | Εξαρτάται από πολύπλοκα domain roles & Windows network stack | **Zero-Privilege & Client-Side Crypto** (XChaCha20-Poly1305 + Argon2id + Merkle Hash Chain) |
| **Οικοσύστημα & Εργασία** | Κλειστό καρτέλ πολυεθνικών συμβουλευτικών | **Ανοικτό Marketplace & 10-Tier Escrow**: Χιλιάδες θέσεις εργασίας για τοπικούς IT integrators |

### 2.1 Μαθηματικό Μοντέλο TCO: 30 Καταστήματα, 150 Χρήστες (Το Σενάριο "Public")

Ας εξετάσουμε μια αλυσίδα 30 καταστημάτων λιανικής/τεχνολογίας με 150 ταυτόχρονους χειριστές σε βάθος 3 ετών:

```
[SAP ENTERPRISE 3-YEAR TCO]
├─ Initial Software Licenses (150 users × 2.800€) ........... : 420.000 €
├─ Implementation & Customization Consulting (14 μήνες) ..... : 380.000 €
├─ Annual Maintenance Fees (22% × 420.000€ × 3 έτη) ......... : 277.200 €
├─ Dedicated Certified SAP Admin Salary (60.000€/έτος × 3) .. : 180.000 €
├─ Heavy Server Farm & Infrastructure ....................... :  95.000 €
└─ ΣΥΝΟΛΙΚΟ ΚΟΣΤΟΣ 3ΕΤΙΑΣ .................................... : 1.352.200 €

[PROTEUS BOS 3-YEAR TCO]
├─ Core Enterprise Deployment License ....................... :   1.990 €
├─ One-Click Managed Cloud Sync & Relays (150 users × 1.20€)  :   6.480 € (3 έτη)
├─ Local Certified Integrator Setup & Templates (1 εβδομάδα) :   4.500 €
├─ Maintenance & Update Subscriptions ....................... :   1.080 € (3 έτη)
├─ Dedicated Server / Hardware Waste ........................ :       0 € (Standard POS PC)
└─ ΣΥΝΟΛΙΚΟ ΚΟΣΤΟΣ 3ΕΤΙΑΣ .................................... :  14.050 €

>> ΚΕΡΔΟΣ ΓΙΑ ΤΗΝ ΕΠΙΧΕΙΡΗΣΗ: 1.338.150 € (98.9% Μείωση Κόστους)
>> ΧΩΡΙΣ ΚΑΜΙΑ ΥΠΟΧΩΡΗΣΗ ΣΕ ΤΑΧΥΤΗΤΑ, ΑΣΦΑΛΕΙΑ Ή ΕΛΕΓΧΟ!
```

---

## 3. Ιεραρχική Αρχιτεκτονική Πολλαπλών Καταστημάτων & Delegated Access

Το θεμελιώδες πλεονέκτημα του Proteus απέναντι στη SAP είναι η **αποκεντρωμένη διαχείριση προσβάσεων (Hierarchical Delegated Provisioning)**. Ο ιδιοκτήτης δεν χρειάζεται να είναι διαχειριστής συστήματος για κάθε κατάστημα.

```
                           ┌───────────────────────────────┐
                           │    ENTERPRISE OWNER / HQ      │
                           │   (Κεντρική Διοίκηση / CEO)   │
                           │  - Καθορισμός Quotas Αδειών   │
                           │  - Global Inventory & P&L     │
                           │  - Master Tamper-Proof Audit  │
                           └───────────────┬───────────────┘
                                           │
         ┌─────────────────────────────────┴─────────────────────────────────┐
         ▼                                                                   ▼
┌─────────────────────────────────┐                         ┌─────────────────────────────────┐
│   STORE 01: ΣΥΝΤΑΓΜΑ (PUBLIC)   │                         │     STORE 02: ΤΣΙΜΙΣΚΗ ΘΕΣ/ΚΗ   │
│   Store Director (Διευθυντής)   │                         │   Store Director (Διευθυντής)   │
│   Allocated Quota: 25 Seats     │                         │   Allocated Quota: 18 Seats     │
└────────────────┬────────────────┘                         └────────────────┬────────────────┘
                 │                                                           │
   ┌─────────────┼─────────────┬─────────────┐                 ┌─────────────┼─────────────┐
   ▼             ▼             ▼             ▼                 ▼             ▼             ▼
[ΑΠΟΘΗΚΗ]   [ΤΗΛΕΦΩΝΙΑ]   [ΒΙΒΛΙΑ/POS]  [SERVICE]         [ΑΠΟΘΗΚΗ]     [ΤΑΜΕΙΟ]      [SERVICE]
Barcodes    IMEI Check    ISBN Scan     Kanban            Barcodes      POS Speed     Kanban
Stock-In    Carrier Plans Quick Pay     Repairs           Stock-In      Receipts      Repairs
4 Seats     8 Seats       10 Seats      3 Seats           3 Seats       10 Seats      5 Seats
```

### 3.1 Τα 3 Επίπεδα Ιεραρχίας (The 3 Governance Tiers)

#### Επίπεδο 1: Enterprise Owner & Κεντρικά (HQ Tier)
- **Πλήρης Εποπτεία (Global Visibility):** Ενιαία εικόνα για όλα τα καταστήματα του δικτύου (από 1 έως 500+ σημεία).
- **Κατανομή Αδειών (Seat Quota Distribution):** Ο Owner αγοράζει ένα συνολικό capacity (π.χ. 200 χρήστες) και κατανέμει όρια σε κάθε κατάστημα (π.χ. Κατάστημα Α: 25, Κατάστημα Β: 15, Κεντρική Αποθήκη: 40).
- **Master Consolidated Audit Stream:** Παρακολούθηση όλων των κινήσεων σε πραγματικό χρόνο.

#### Επίπεδο 2: Διευθυντής Καταστήματος (Store Director Tier)
- **Αυτόνομο Role Provisioning:** Ο διευθυντής του καταστήματος έχει το δικό του απλό UI. Μπορεί άμεσα να δημιουργήσει ή να απενεργοποιήσει λογαριασμούς για το προσωπικό του, μέσα στα όρια του quota που του έχουν δοθεί.
- **Μηδενική Εμπλοκή Κεντρικού IT:** Δεν απαιτείται ticket στο IT department για να ξεκινήσει δουλειά ένας νέος πωλητής. Ο διευθυντής πατάει «+ Νέος Υπάλληλος», επιλέγει το τμήμα (π.χ. *Πωλητής Τηλεφωνίας*) και το σύστημα παράγει αυτόματα τα κατάλληλα δικαιώματα.

#### Επίπεδο 3: Εξειδικευμένοι Χειριστές Τμημάτων (Departmental Operators)
Κάθε τμήμα του καταστήματος αντικρίζει ένα απόλυτα εξειδικευμένο, καθαρό περιβάλλον εργασίας, απαλλαγμένο από περιττά δεδομένα:

1. **📦 Αποθήκη & Logistics (Warehouse Operator):**
   - Παραλαβές εμπορευμάτων με ασύρματο Barcode scanner.
   - Ενδοδιακινήσεις μεταξύ καταστημάτων (Inter-store Transfers).
   - Αυτόματες ειδοποιήσεις χαμηλού αποθέματος (Reorder Points).
2. **📱 Πωλητής Κινητής & Τεχνολογίας (Mobile & Tech Specialist):**
   - Σκανάρισμα σειριακών αριθμών IMEI.
   - Συνδυασμός συσκευής με συμβόλαιο παρόχου και πακέτο επέκτασης εγγύησης.
   - Έλεγχος διαθεσιμότητας αποθέματος σε γειτονικά καταστήματα της αλυσίδας σε < 1 δευτερόλεπτο.
3. **📚 Πωλητής Βιβλίων & Χαρτικών (Books & Stationery / Quick POS):**
   - Σκανάρισμα ISBN με άμεση ανάκτηση τίτλου και συγγραφέα.
   - Ταχύτατη έκδοση αποδείξεων POS (κάτω από 5 δευτερόλεπτα ανά πελάτη).
   - Εφαρμογή προγραμμάτων επιβράβευσης (Loyalty points & δωροκάρτες).
4. **🛠 Τεχνικό Εργαστήριο & Service (Service Lab Technician):**
   - Ψηφιακό δελτίο επισκευής, διάγνωση, εκτύπωση ετικέτας με barcode.
   - Kanban pipeline 6 σταδίων.
   - Αυτόματη δέσμευση ανταλλακτικών από την τοπική αποθήκη.

---

## 4. Αρχιτεκτονική Βάσης Δεδομένων: Σχεδιασμός DDL

Η υποδομή multi-tenant / multi-store ενσωματώνεται στον πυρήνα του `crm-core`:

```sql
-- 1. Οργανισμός / Εταιρεία (Enterprise Entity)
CREATE TABLE IF NOT EXISTS enterprises (
    enterprise_id TEXT PRIMARY KEY,               -- UUIDv7
    legal_name TEXT NOT NULL,
    tax_id TEXT NOT NULL UNIQUE,                  -- ΑΦΜ Επιχείρησης
    total_seat_quota INTEGER NOT NULL DEFAULT 10, -- Συνολικό όριο χρηστών
    used_seats INTEGER NOT NULL DEFAULT 0,
    cloud_tier TEXT NOT NULL DEFAULT 'EnterpriseCore',
    created_at TEXT NOT NULL
);

-- 2. Υποκαταστήματα / Σημεία Πώλησης (Store Branches)
CREATE TABLE IF NOT EXISTS enterprise_stores (
    store_id TEXT PRIMARY KEY,                    -- UUIDv7
    enterprise_id TEXT NOT NULL REFERENCES enterprises(enterprise_id),
    store_code TEXT NOT NULL UNIQUE,              -- π.χ. "STR-001-SYNTAGMA"
    store_name TEXT NOT NULL,
    address TEXT NOT NULL,
    phone TEXT NOT NULL,
    manager_user_id TEXT,                         -- UUIDv7 του Store Director
    allocated_seats INTEGER NOT NULL DEFAULT 5,   -- Κατανεμημένο quota θέσεων
    active_seats INTEGER NOT NULL DEFAULT 0,
    is_active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL
);

-- 3. Τμήματα Καταστήματος (Departments)
CREATE TABLE IF NOT EXISTS store_departments (
    department_id TEXT PRIMARY KEY,
    store_id TEXT NOT NULL REFERENCES enterprise_stores(store_id),
    dept_code TEXT NOT NULL,                      -- "WAREHOUSE", "MOBILE", "BOOKS", "SERVICE", "POS"
    name TEXT NOT NULL,
    created_at TEXT NOT NULL
);

-- 4. Χρήστες & Εξουσιοδοτήσεις (Delegated Users)
CREATE TABLE IF NOT EXISTS enterprise_users (
    user_id TEXT PRIMARY KEY,                     -- UUIDv7
    enterprise_id TEXT NOT NULL REFERENCES enterprises(enterprise_id),
    store_id TEXT NOT NULL REFERENCES enterprise_stores(store_id),
    department_id TEXT REFERENCES store_departments(department_id),
    full_name TEXT NOT NULL,
    email TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,                  -- Argon2id
    role_code TEXT NOT NULL,                      -- 'StoreDirector', 'Warehouse', 'MobileSales', 'PosCashier', 'ServiceTech'
    is_active INTEGER NOT NULL DEFAULT 1,
    created_by_user_id TEXT NOT NULL,             -- Audit ποιος Store Director τον δημιούργησε
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- 5. Απαραβίαστο Event-Sourced Audit Ledger (Cryptographic Backlog)
CREATE TABLE IF NOT EXISTS tamper_proof_audit_backlog (
    sequence_id INTEGER PRIMARY KEY AUTOINCREMENT,
    event_id TEXT NOT NULL UNIQUE,                -- UUIDv7
    enterprise_id TEXT NOT NULL,
    store_id TEXT NOT NULL,
    operator_id TEXT NOT NULL,
    operator_role TEXT NOT NULL,
    entity_type TEXT NOT NULL,                    -- 'INVENTORY', 'TICKET', 'USER_PROVISION', 'SALE'
    entity_id TEXT NOT NULL,
    action_type TEXT NOT NULL,                    -- 'STOCK_IN', 'SALE_COMPLETED', 'ROLE_ASSIGN'
    payload_diff_json TEXT NOT NULL,
    previous_hash TEXT NOT NULL,                  -- SHA-256 Hash προηγούμενου block (Blockchain-grade)
    current_hash TEXT NOT NULL,                   -- SHA-256(previous_hash + payload + timestamp)
    created_at TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_audit_store_seq ON tamper_proof_audit_backlog(store_id, sequence_id DESC);
```

---

## 5. Απαραβίαστη Ασφάλεια: Cryptographic Hash Chain

Στις μεγάλες επιχειρήσεις, η διαρροή δεδομένων, η εσωτερική κλοπή αποθεμάτων και η αλλοίωση τιμών είναι υπαρξιακοί κίνδυνοι. Στη SAP, ένας διαχειριστής βάσης δεδομένων μπορεί να αλλάξει εγγραφές στο SQL backend χωρίς να γίνει αντιληπτός.

Στο **Proteus BOS**, αυτό είναι **μαθηματικά αδύνατο**:

```
[Event #101]                           [Event #102]                           [Event #103]
┌───────────────────────────┐          ┌───────────────────────────┐          ┌───────────────────────────┐
│ EventID: 018f...          │          │ EventID: 018f...          │          │ EventID: 018f...          │
│ Action: STOCK_INTAKE      │          │ Action: SALE_PHONE_IMEI   │          │ Action: USER_PROVISION    │
│ PrevHash: 4a8f...         │◄─────────┤ PrevHash: e3b0...         │◄─────────┤ PrevHash: 9f1c...         │
│ CurrentHash: e3b0...      │          │ CurrentHash: 9f1c...      │          │ CurrentHash: a72b...      │
└───────────────────────────┘          └───────────────────────────┘          └───────────────────────────┘
```

1. **Αλυσίδα Hashing (Merkle Hash Chain):**
   Κάθε εγγραφή στο `tamper_proof_audit_backlog` εμπεριέχει το SHA-256 hash της ακριβώς προηγούμενης εγγραφής.
2. **Αδυναμία Παραποίησης:**
   Αν κάποιος κακόβουλος χρήστης ή εισβολέας ανοίξει το SQLite αρχείο και προσπαθήσει να τροποποιήσει μια πώληση ή ένα απόθεμα στο παρελθόν, το hash της εγγραφής αλλάζει, **σπάζοντας ακαριαία ολόκληρη την αλυσίδα**.
3. **Αυτόματος Συναγερμός (Integrity Panic):**
   Το σύστημα επαληθεύει την ακεραιότητα της αλυσίδας σε κάθε συγχρονισμό. Εάν εντοπιστεί αναντιστοιχία hashes, το κατάστημα απομονώνεται και αποστέλλεται άμεση ειδοποίηση ασφαλείας στον Enterprise Owner.

---

## 6. One-Click Cloud Orchestration: Zero-DevOps Deployment

Η εγκατάσταση ενός παραδοσιακού ERP απαιτεί εβδομάδες ρυθμίσεων σε Windows Server, Oracle DB, ODBC drivers και περίπλοκα firewalls. 

Στο **Proteus**, η διάθεση του κεντρικού cloud συγχρονισμού είναι **πραγματικό One-Click Setup**:

```bash
# Μοναδική εντολή εκκίνησης για τον κεντρικό enterprise relay server:
docker run -d --name proteus-enterprise-hub \
  -p 443:443 -p 80:80 \
  -v /var/proteus/data:/data \
  -e ENTERPRISE_KEY="ent_live_9481a8f029c" \
  -e DOMAIN="sync.mycompany.gr" \
  proteus/hub:latest
```

* **Αυτόματο SSL/TLS:** Ενσωματωμένη έκδοση πιστοποιητικών Let's Encrypt χωρίς εξωτερικό Nginx.
* **Αυτοματοποιημένα Hot-Snapshots:** Κάθε 4 ώρες παράγεται συμπιεσμένο, κρυπτογραφημένο snapshot της βάσης (`store_YYYYMMDD_HHMM.bak.zst`) με αυτόματη αποστολή σε S3-compatible cloud storage (Cloudflare R2, MinIO).
* **Μηδενική Συντήρηση (Zero-Maintenance):** Το binary εκτελείται αυτόνομα, ανακτάται αυτόματα μετά από επανεκκίνηση και δεν χρειάζεται βάσεις εξωτερικών drivers.

---

## 7. Αποκεντρωμένη Οικονομία & Δημιουργία Θέσεων Εργασίας ("Περισσότερες Δουλειές")

Το όραμα του Proteus δεν σταματά στο λογισμικό. Αφορά τη **δίκαιη κατανομή της οικονομικής αξίας**:

### 7.1 Από το Μονοπώλιο των Big-4 στην Ανοιχτή Αγορά
* Σήμερα, τα 1,3 εκατομμύρια ευρώ του κόστους ενός ERP καταλήγουν σε πολυεθνικούς κολοσσούς του εξωτερικού.
* Με το Proteus, τα χρήματα αυτά **επιστρέφουν στην πραγματική οικονομία**:
  1. **Τοπικοί IT Επαγγελματίες & Νέοι Developers:** Κάθε τοπικός τεχνικός ή απόφοιτος πληροφορικής σε οποιαδήποτε πόλη (Αθήνα, Θεσσαλονίκη, Πάτρα, Ηράκλειο) μπορεί να πιστοποιηθεί ως **Proteus Certified Specialist**.
  2. **Αμοιβές Εγκατάστασης & Customization:** Αναλαμβάνει να ρυθμίσει τα καταστήματα της περιοχής του, να σχεδιάσει προσαρμοσμένα templates στο Proteus Studio και να συνάψει συμβόλαια συντήρησης.
  3. **Ασφάλεια Πληρωμών (In-Platform Escrow):** Η επιχείρηση δεσμεύει την αμοιβή του τεχνικού στην πλατφόρμα και αποδεσμεύεται αυτόματα μόλις παραδοθεί και επικυρωθεί το πακέτο.
  4. **Κλιμάκωση Εσόδων (10-Tier Commission):** Όσο περισσότερα επιτυχημένα projects παραδίδει ο τεχνικός, τόσο μειώνεται η προμήθεια της πλατφόρμας (από 50% στο 4%), επιτρέποντάς του να χτίσει μια κερδοφόρα, ανεξάρτητη επιχείρηση.

---

## 8. Σύνοψη Αρχιτεκτονικού Οδικού Χάρτη (Roadmap)

| Φάση | Παραδοτέο | Κατάσταση |
| :--- | :--- | :---: |
| **Phase 1: Foundation** | Granular RBAC, Audit Logs, Offline-First SQLite, ESC/POS Spooler | ✅ Ολοκληρώθηκε (121/121 tests) |
| **Phase 2: Multi-Store Engine** | DDL Enterprises, Stores, Departments & Delegated Provisioning | 🔄 Σε εξέλιξη (`crm-core`) |
| **Phase 3: Cryptographic Backlog** | Merkle Hash-Chained Audit Trail & Tamper-Proof Validation | 📅 Επόμενο Sprint |
| **Phase 4: Store Director UI** | Οθόνη διαχείρισης θέσεων και προσωπικού στο `proteus-client` | 📅 Προγραμματισμένο |
| **Phase 5: 1-Click Cloud Hub** | Single-binary Docker hub με αυτόματο Let's Encrypt TLS & Snapshots | 📅 Προγραμματισμένο |

---
> *«Δεν ανταγωνιζόμαστε τη SAP στο δικό της παλιό γήπεδο της γραφειοκρατίας και του εκβιασμού. Την κερδίζουμε αλλάζοντας τους κανόνες: απόλυτη ταχύτητα, μαθηματική διαφάνεια, μηδενικό friction και ελευθερία για κάθε επιχείρηση.»*
