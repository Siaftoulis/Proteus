# 22 - Business Data Analyst Pipeline & Universal Connector

---

## 1. Το Πλαίσιο & Ο Ρόλος: PCDA (Proteus Certified Data/Business Analyst)

### 1.1 Γιατί Χρειάζεται ο Business/Data Analyst;
Στα παραδοσιακά έργα ERP/CRM, το μεγαλύτερο ποσοστό αποτυχίας προκύπτει από το γεγονός ότι οι **Designers** ή οι **Προγραμματιστές** αναγκάζονται να ερμηνεύσουν επιχειρηματικές διαδικασίες και ακατάστατα δεδομένα (messy data).
- Ένας Designer γνωρίζει από UI/UX, διατάξεις και χρώματα. **Δεν φέρει νομική ή επιχειρησιακή ευθύνη** για το πώς διαρθρώνονται τα λογιστικά βιβλία, τα αποθέματα ή τα API contracts.
- Αν ο Designer προσπαθήσει να κάνει data engineering, το visual design tool φουσκώνει, γίνεται δυσνόητο και η πιθανότητα λαθών εκτοξεύεται.

### 1.2 Ο Ρόλος του PCDA ως Πρώτο Σημείο Επαφής
Ο **PCDA (Proteus Certified Data/Business Analyst)** είναι ο **πρώτος επαγγελματίας** που επικοινωνεί με την επιχείρηση:

```
┌─────────────────────────────────────────────────────────────┐
│                          ΕΠΙΧΕΙΡΗΣΗ                         │
│       (Workflows, Business Needs, Real Data Samples)        │
└──────────────────────────────┬──────────────────────────────┘
                               │ (1. Ανάλυση & Καταγραφή)
                               ▼
┌─────────────────────────────────────────────────────────────┐
│             BUSINESS / DATA ANALYST (PCDA)                  │
│  - Business Analysis: Ροές εργασίας & προδιαγραφές          │
│  - Data Analysis: Universal Connector (JSON, CSV, APIs)     │
│  - Schema Inference: Κανονικοποίηση & Primary Key Detection  │
│  - Business Rules: "ΑΝ X ΤΟΤΕ Y" επικυρώσεις & transformations│
└──────────────────────────────┬──────────────────────────────┘
                               │ (2. Δομημένο Data Contract)
                               ▼
┌─────────────────────────────────────────────────────────────┐
│                  PROTEUS DESIGNER (PCD)                     │
│  - Εκτελεί UI, διατάξεις, οθόνες & κουμπιά                 │
│  - Δεν ερμηνεύει, δεν μαντεύει, δεν φέρει ευθύνη δεδομένων │
└──────────────────────────────┬──────────────────────────────┘
                               │ (3. Παραγωγή Runtime .pr)
                               ▼
┌─────────────────────────────────────────────────────────────┐
│                 PROTEUS CLIENT RUNTIME                      │
│  - Εγκατάσταση στο ταμείο/εργαστήριο (PCDS Deployer)        │
│  - Database integrity & Migrations (PCSS Specialist)        │
└─────────────────────────────────────────────────────────────┘
```

---

## 2. Τεχνική Αρχιτεκτονική: Core Modules

### 2.1 Universal API Connector & Schema Inference (`crm-core::inference`)
- **JSON Flattening & Column Normalization:**
  - Αυτόματη μετατροπή βαθιά εμφωλευμένων JSON objects σε καθαρές σχεσιακές στήλες (π.χ. `customer.address.city` $\rightarrow$ `customer_address_city`).
  - Υποστήριξη πρωτογενών τύπων: `Integer`, `Real`, `Text`, `Boolean`, `Jsonb` (για σύνθετες λίστες).
- **Ανίχνευση Πρωτευόντων Κλειδιών (Primary Key Detection):**
  - Αυτόματη ανίχνευση ευρετηρίων: `id`, `uuid`, `_id`, `sku`, `barcode`, `code`.
- **Ανοχή σε Ακατάστατα Δεδομένα (Messy / Semi-Structured Ingestion):**
  - Ανάλυση πολλαπλών εγγραφών δείγματος (sample batch profiling) για εύρεση του πληρέστερου συνόλου στηλών.

### 2.2 Αποκωδικοποίηση Προτύπου GS1 (`crm-core::gs1`)
Υποστήριξη γραμμωτών κωδίκων **GS1-128** με ανάλυση Application Identifiers (AIs):
- `(01)` GTIN (Global Trade Item Number - 14 ψηφία)
- `(10)` Αριθμός Παρτίδας (Batch / Lot Number)
- `(17)` Ημερομηνία Λήξης (YYMMDD)
- `(21)` Σειριακός Αριθμός (Serial Number)
- `(00)` SSCC (Serial Shipping Container Code)

### 2.3 Μηχανή Επιχειρησιακών Κανόνων (`crm-core::rules`)
- Δηλωτική γλώσσα κανόνων (Event-Driven Rules): `"IF <condition> THEN <action>"`.
- Επικύρωση τιμών, αυτόματες εκπτώσεις, δεσμεύσεις αποθεμάτων, και triggers ειδοποιήσεων.
- **Αμιγώς Sandboxed Rust Evaluator:** Μηδενικός κίνδυνος εκτέλεσης κακόβουλου κώδικα, καμία πρόσβαση σε αρχεία συστήματος ή δίκτυο, προστασία από ατέρμονους βρόχους (infinite loops).

### 2.4 Visual Mapping Canvas (`crm-ui` / `proteus-client`)
- Σύγχρονος καμβάς σε **egui 0.31** για αντιστοίχιση εξωτερικών πεδίων API σε σχεσιακούς πίνακες.
- **Αλγόριθμος Fuzzy Matching:** Αυτόματη προ-αντιστοίχιση παρόμοιων ονομάτων πεδίων (π.χ. `cust_name` $\leftrightarrow$ `customer_name`).
- Live Data Preview: Προεπισκόπηση των πρώτων 10 εγγραφών σε πραγματικό χρόνο πριν την οριστικοποίηση του σχήματος.

---

## 3. Οικονομικό Μοντέλο Πιστοποίησης PCDA

| Στοιχείο | Τιμή | Περιθώριο Πλατφόρμας |
| :--- | :---: | :---: |
| **Exam Voucher (PCDA)** | **79 €** | ~95% (~75€ καθαρό κέρδος) |
| **Ετήσιο Verified Partner Badge** | **39 € / έτος** | 100% recurring |
| **All-in-One Master Bundle (PCDA + PCD + PCSS + PCDS)** | **149 €** | ~96% |

---

## 4. Χρονοδιάγραμμα Υλοποίησης 4 Φάσεων

1. **Φάση 1 (Core Data & Schema Inference Engine + GS1 Parser):**
   - Υλοποίηση `crm-core::inference` (flattening, type inference, PK detection).
   - Υλοποίηση `crm-core::gs1` (GS1-128 Application Identifiers).
   - Επέκταση `crm-core::roles` με τον νέο ρόλο `UserRole::BusinessAnalyst`.
2. **Φάση 2 (Business Rules Engine & Event Bus):**
   - Υλοποίηση `crm-core::rules` (declarative rule evaluator).
   - Ενδοσυστημικός Event Bus για ασύγχρονη επικοινωνία.
3. **Φάση 3 (Visual Mapping Canvas σε egui 0.31):**
   - Δημιουργία διαδραστικού UI στο `crm-ui` για drag & drop αντιστοίχιση API fields $\rightarrow$ schema fields.
4. **Φάση 4 (Analyst Studio & Web Marketplace Integration):**
   - Νέο tab `AnalystStudio` στο `proteus-client` για τον πιστοποιημένο PCDA.
   - Ενημέρωση των endpoints πιστοποιήσεων και bundles στο `proteus-web`.
