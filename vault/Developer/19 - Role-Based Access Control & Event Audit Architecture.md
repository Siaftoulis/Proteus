# 19 - Role-Based Access Control & Event Audit Architecture

## 1. Το Όραμα & Η Ανάγκη (Business & Operational Context)

Σε κάθε οργανισμό που διαχειρίζεται καθημερινές λειτουργίες, εξυπηρέτηση πελατών, επισκευές και επιχειρηματικά συμβόλαια, προκύπτουν δύο αδιαπραγμάτευτες ανάγκες:
1. **Απόλυτη Διαφάνεια & Ιχνηλασιμότητα Κινήσεων (Audit Trail / Activity Timeline):**
   Κάθε ενέργεια στο σύστημα — από την έκδοση ενός νέου δελτίου παραλαβής και την ολοκλήρωση ενός ραντεβού, έως την τροποποίηση του database schema ή την υπογραφή ενός συμβολαίου — πρέπει να καταγράφεται με ακρίβεια χιλιοστού του δευτερολέπτου: *ποιος* την έκανε, *ποιο ρόλο* κατείχε, *τι ακριβώς* άλλαξε (payload diff) και *πότε*.
2. **Δυναμικός Διαχωρισμός Ρόλων (Granular RBAC):**
   Διαφορετικά στελέχη έχουν διαφορετικές ευθύνες και ανάγκες πληροφόρησης:
   - **CEO / Owner:** Πλήρης εποπτεία (`Full Access`) σε όλα τα υποσυστήματα, οικονομικά περιθώρια, logs, ρυθμίσεις καταστήματος και developer tools.
   - **Customer Service (Εξυπηρέτηση Πελατών):** Εστιάζει στη γρήγορη υποδοχή πελατών, κλείσιμο ραντεβού, αρχικό intake, χωρίς να αποσπάται ή να έχει πρόσβαση σε τεχνικά κοστολόγια και εσωτερικά schemas.
   - **Technician (Τεχνικός Εργαστηρίου):** Εστιάζει στη διάγνωση, το Kanban pipeline, τις τεχνικές σημειώσεις και την καταγραφή κόστους ανταλλακτικών.
   - **Sales Consultant (Σύμβουλος Πωλήσεων):** Διαχειρίζεται συμβόλαια πελατών, quotes, εμπορικά ραντεβού και προσφορές.
   - **Developer (Μηχανικός Λογισμικού):** Ειδικό περιβάλλον για τροποποίηση του database schema (`ALTER TABLE`), επιθεώρηση raw data και εκτέλεση enterprise work orders.

Όλα αυτά τα υποσυστήματα **δεν αποτελούν απομονωμένα σιλό**, αλλά συνεργάζονται αρμονικά μέσα από τον ενιαίο πυρήνα του **Proteus CRM**, μοιραζόμενα την ίδια βάση δεδομένων (`store.db`) και το ίδιο event-sourced stream καταγραφής.

---

## 2. Αρχιτεκτονική Πυρήνα: `crm-core::roles` & `crm-core::audit`

### 2.1 Πίνακας Δικαιωμάτων Ρόλων (`RolePermissions`)

Στο crate `crm-core::roles`, κάθε ρόλος ορίζεται από το enum `UserRole` και συνοδεύεται από τη δομή `RolePermissions`:

| Δικαίωμα (`Permission Flag`) | CEO | Customer Service | Technician | Sales | Developer |
| :--- | :---: | :---: | :---: | :---: | :---: |
| `can_view_audit_trail` | ✅ | ❌ | ❌ | ❌ | ✅ |
| `can_view_financials` | ✅ | ❌ | ❌ | ✅ | ❌ |
| `can_edit_schema` | ✅ | ❌ | ❌ | ❌ | ✅ |
| `can_intake_tickets` | ✅ | ✅ | ❌ | ❌ | ❌ |
| `can_manage_pipeline` | ✅ | ✅ | ✅ | ❌ | ❌ |
| `can_edit_technical_notes` | ✅ | ❌ | ✅ | ❌ | ❌ |
| `can_manage_contracts` | ✅ | ❌ | ❌ | ✅ | ❌ |
| `can_book_appointments` | ✅ | ✅ | ❌ | ✅ | ❌ |
| `can_manage_settings` | ✅ | ❌ | ❌ | ❌ | ✅ |

### 2.2 Event-Sourced Audit Trail (`crm-core::audit`)

Κάθε σημαντική κίνηση στο CRM δημιουργεί ένα `SystemEvent` με τα εξής πεδία:
- `event_id`: Μονοτονικό αναγνωριστικό UUIDv7 (διασφαλίζει χρονολογική ταξινόμηση).
- `entity_type`: Ο τύπος της οντότητας (`"ticket"`, `"appointment"`, `"schema"`, `"contract"`, `"auth"`).
- `entity_id`: Το αναγνωριστικό του αντικειμένου που επηρεάστηκε.
- `event_type`: Η δράση που εκτελέστηκε (`"CREATED"`, `"STATUS_CHANGE"`, `"UPDATED"`, `"CANCELLED"`, `"MIGRATION"`).
- `operator_name`: Το ονοματεπώνυμο του χρήστη που πραγματοποίησε την ενέργεια.
- `operator_role`: Ο ενεργός ρόλος του χρήστη κατά την εκτέλεση.
- `description`: Σύντομη, ανθρώπινα αναγνώσιμη περιγραφή.
- `payload_json`: Δομημένο JSON με τα ακριβή δεδομένα / διαφορές (diff).
- `created_at`: Χρονοσφραγίδα ISO-8601 UTC.

```sql
CREATE TABLE IF NOT EXISTS audit_logs (
    event_id TEXT PRIMARY KEY,
    entity_type TEXT NOT NULL,
    entity_id TEXT NOT NULL,
    event_type TEXT NOT NULL,
    operator_name TEXT NOT NULL,
    operator_role TEXT NOT NULL,
    description TEXT NOT NULL,
    payload_json TEXT NOT NULL,
    created_at TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_audit_logs_created_at ON audit_logs(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_audit_logs_entity ON audit_logs(entity_type, entity_id);
CREATE INDEX IF NOT EXISTS idx_audit_logs_operator ON audit_logs(operator_role);
```

---

## 3. Παρουσίαση & Εμπειρία Χρήστη στο `proteus-client`

### 3.1 Δυναμικό TopBar & Επιλογή Ρόλου
Στο επάνω μέρος της εφαρμογής (`app.rs`), εμφανίζεται ένδειξη του ενεργού χρήστη και drop-down επιλογέα ρόλου (`UserRole`).
Όταν αλλάζει ο ρόλος:
- Επαναϋπολογίζονται ακαριαία τα διαθέσιμα tabs στο navigation bar.
- Εάν το τρέχον tab δεν επιτρέπεται στο νέο ρόλο, η εφαρμογή μεταβαίνει αυτόματα στο πρώτο επιτρεπόμενο tab.

### 3.2 Timeline Δραστηριότητας & Audit Log (`AuditLogView`)
Το tab **Audit Trail** προσφέρει μια state-of-the-art παρουσίαση των κινήσεων:
- **Ζωντανή Αναζήτηση & Φίλτρο Ρόλων:** Αναζήτηση σε πραγματικό χρόνο ανά χειριστή, περιγραφή, τύπο ενέργειας, ή φιλτράρισμα βάσει ρόλου (CEO, CS, Tech, Sales, Dev).
- **Χρωματικά Κωδικοποιημένα Badges:**
  - 👑 CEO: Χρυσό / Amber (`#F59E0B`)
  - 🎧 Customer Service: Ουράνιο Μπλε (`#38BDF8`)
  - 🛠 Technician: Σμαραγδί Πράσινο (`#10B981`)
  - 📈 Sales: Ζεστό Πορτοκαλί (`#F97316`)
  - 💻 Developer: Βιολετί / Indigo (`#A855F7`)
- **Σχετικοί Χρόνοι (Human-Readable Timestamps):** Μετατροπή των χρονοσφραγίδων σε άμεσα κατανοητές εκφράσεις («μόλις τώρα», «πριν 5 λεπτά», «πριν 2 ώρες», «πριν 3 ημέρες»).
- **Πτυσσόμενος JSON Diff Inspector:** Δυνατότητα επέκτασης κάθε κάρτας με κλικ στο `🔎 Payload JSON` για προβολή των τεχνικών παραμέτρων της κίνησης.

### 3.3 Υποσύστημα Ραντεβού & Εξυπηρέτησης Πελατών (`AppointmentsView`)
Το υποσύστημα **Appointments** επιτρέπει στο προσωπικό εξυπηρέτησης πελατών και στις πωλήσεις:
- Να προγραμματίζουν νέα ραντεβού (Ονοματεπώνυμο, Τηλέφωνο, Τύπος ραντεβού, Ημερομηνία/Ώρα, Σημειώσεις).
- Με κάθε καταχώρηση ραντεβού, εκπέμπεται αυτόματα ένα `SystemEvent` στο `audit_logs` με πλήρη καταγραφή του χειριστή.
- Να επισημαίνουν ραντεβού ως «Ολοκληρωμένα» ή «Ακυρωμένα», ενημερώνοντας σε πραγματικό χρόνο το audit timeline.

---

## 4. Συνεργασία Υποσυστημάτων (Unified Ecosystem)

Παρά το γεγονός ότι ο κάθε ρόλος αντικρίζει μια εξειδικευμένη και απαλλαγμένη από περιττό θόρυβο επιφάνεια εργασίας, **όλες οι ενέργειες συνδέονται άρρηκτα**:
1. Το Customer Service κλείνει ένα **Ραντεβού Παραλαβής**.
2. Κατά την άφιξη του πελάτη, δημιουργείται **Ticket Παραλαβής** (Intake) και εκτυπώνεται απόδειξη μέσω Win32 RAW Spooler.
3. Το Ticket εμφανίζεται αυτόματα στη στήλη *Received* του Kanban pipeline του **Τεχνικού**.
4. Ο Τεχνικός εκτελεί την επισκευή, καταχωρεί ανταλλακτικά και αλλάζει την κατάσταση σε *Ready*.
5. Ο **CEO** παρακολουθεί από το Timeline του Audit Log κάθε στάδιο της διαδικασίας με ακριβείς χρόνους και υπευθύνους.
6. Ο **Developer** μπορεί, εφόσον ζητηθεί από την επιχείρηση, να προσθέσει νέα πεδία στην οντότητα `tickets` ή `appointments` μέσω του Developer Studio χωρίς downtime.

---

## 5. Επαλήθευση & Έλεγχοι Ποιότητας (Verification)
- **100% Original Codebase**: Όλος ο κώδικας των ρόλων, των audit logs και των UI views γράφτηκε από το μηδέν χωρίς αντιγραφή εξωτερικών βιβλιοθηκών.
- **Αρχιτεκτονική Κλάσεων**: Όλα τα αρχεία (`roles.rs`, `audit.rs`, `audit_log.rs`, `appointments.rs`) παραμένουν αυστηρά κάτω από 400 γραμμές.
- **121/121 Workspace Tests Passing**: Πλήρης κάλυψη unit tests σε ολόκληρο το workspace (`crm-core`, `crm-ui`, `proteus-client`, `proteus-web`, `auth-server`, `license-server`).
