---
tags:
  - developer/standards
  - architecture/specifications
  - three-pillars
  - ui-ux
aliases:
  - Ecosystem Standards
  - Pillar Specifications
  - Proteus Operational Standards
---
# Proteus BOS: Πρότυπα Οικοσυστήματος & Προδιαγραφές των 3 Πυλώνων

> **Status:** Επίσημο Έγγραφο Προδιαγραφών (Operational & Design Standards)  
> **Ημερομηνία:** 15 Σεπτεμβρίου 2026  
> **Συγγραφέας:** Lead Architect & Core Engineering Team  
> **Σκοπός:** Ορισμός της ακριβούς συμπεριφοράς, των καρτελών, της λογικής λειτουργίας και των διεπαφών για τους τρεις πυλώνες του οικοσυστήματος: **Proteus Studio (Designer)**, **Proteus Client (Shop Counter Runtime)**, και **Proteus Web Hub (Portal & Marketplace)**.

---

## 1. Θεμελιώδεις Αρχές & Σχεδιαστικά Πρότυπα (Core Standards)

Κάθε στοιχείο του οικοσυστήματος Proteus διέπεται από 4 αδιαπραγμάτευτες αρχές:

1. **Αρχή του "Average Joe" (Μηδενικό Γνωστικό Φορτίο)**:
   - Ο χειριστής του καταστήματος στο ταμείο δεν είναι προγραμματιστής.
   - Καμία οθόνη δεν πρέπει να απαιτεί περισσότερα από 3 κλικ ή πάνω από 30 δευτερόλεπτα για να ολοκληρώσει μια συνήθη λειτουργία (π.χ. παραλαβή συσκευής, αλλαγή σταδίου, εκτύπωση).
2. **100% Offline-First & Zero-Privilege Runtime**:
   - Το runtime του καταστήματος (`proteus-client`) λειτουργεί αδιάλειπτα χωρίς σύνδεση στο διαδίκτυο, χωρίς δικαιώματα Windows Administrator (αποθήκευση στο `%APPDATA%\Proteus\data\store.db`).
   - Καμία εξάρτηση από cloud για τις καθημερινές λειτουργίες συναλλαγών.
3. **Καθαρή & Μινιμαλιστική Αισθητική (Luxury Dark Native Aesthetic)**:
   - Βαθύ obsidian φόντο (`#0E0F12`), πάνελ κάρτας (`#161820`), crisp περιγράμματα 1px (`#242836`), ζωηρές ενδείξεις κατάστασης (Emerald, Cyan, Amber, Rose) και τυπογραφία υψηλής αναγνωσιμότητας.
   - Απαγορεύεται η οπτική υπερφόρτωση (no visual clutter, no bloated boxes).
4. **Αυστηρά Bespoke & Πρωτότυπη Αρχιτεκτονική**:
   - 100% πρωτότυπος κώδικας σε native Rust, σχεδιασμένος από πρώτες αρχές (first principles), χωρίς αντιγραμμένα snippets ή τρίτα frameworks.

---

## 2. Πυλώνας 1: Proteus Studio (The Visual Designer)

### 2.1 Σκοπός & Κοινό-Στόχος
* **Χρήστες**: Certified Designers, IT Integrators, τεχνικοί σύμβουλοι και power users.
* **Αποστολή**: Οπτική σχεδίαση, παραμετροποίηση φορμών, ορισμός entities στη βάση δεδομένων, σχεδιασμός αποδείξεων και εξαγωγή σε δηλωτικό πακέτο `.pr`.
* **Τιμολόγηση**: **100% ΔΩΡΕΑΝ Desktop Tool** (Windows / macOS / Linux binary).

### 2.2 Δομή Καρτελών & Περιβάλλοντος (Workspace Tabs)

Το Studio οργανώνεται σε 3 κεντρικούς άξονες (Mode Switcher στο TopBar):

```
+-----------------------------------------------------------------------------------------+
| [🎨 1. Design Canvas]   |   [📊 2. Data Studio]   |   [📱 3. Devices & Run]   | [⚡ Flows] |
+-----------------------------------------------------------------------------------------+
```

#### Καρτέλα 1: Design Canvas (Οπτικός Καμβάς Σχεδίασης)
* **Άπειρος Καμβάς (Infinite GPU Canvas)**:
  * Pan & Smooth Zoom (0.1x έως 5.0x) με διατήρηση κεντραρίσματος στον κέρσορα.
  * Dot-matrix πλέγμα (Grid Snap 24px) με δυνατότητα εναλλαγής σε Freeform.
* **Vertical Tool Strip (Αριστερή Μπάρα Εργαλείων)**:
  * `↖` **Select Tool (V)**: Επιλογή μεμονωμένων στοιχείων ή marquee multi-selection.
  * `▢` **Smart Box (R)**: Έξυπνο κοντέινερ (Container, Input, Card, Table role).
  * `T` **Text Tool (T)**: Ετικέτες, τίτλοι, οδηγίες με άμεση επεξεργασία.
  * `🔘` **Action Button (B)**: Κουμπιά με αυτόνομη σύνδεση ενεργειών (Navigate, Submit, Print).
  * `⊞` **Data Table (G)**: Πίνακας με live δέσμευση σε entity (π.χ. `tickets`, `contacts`).
  * `📄` **Add Page (P)**: Δημιουργία νέας σελίδας / οθόνης (Multi-screen flows).
  * `💳` **KPI Metric Card (K)**: Έτοιμο component με τίτλο, μεγάλη μέτρηση και ένδειξη τάσης.
  * `📝` **Intake Form Card (F)**: Έτοιμο πλήρες φορμάκι παραλαβής με πεδία και κουμπί.
* **Διαδραστικός Επιθεωρητής (Dual-Zone Inspector - Δεξιά Μπάρα)**:
  * **Ζώνη 1 (Άνω 2/3 - Ιδιότητες)**:
    * *Transform & Size*: Συντεταγμένες X, Y, διαστάσεις W, H, Copy/Paste Size.
    * *Quick Alignment Toolbar*: 1-click στοίχιση (`⇤` Αριστερά, `⇋` Κέντρο X, `⇥` Δεξιά, `⤒` Πάνω, `⥯` Κέντρο Y, `⤓` Κάτω).
    * *Smart Box Role Selector*: Επιλογή συμπεριφοράς (Container, Shape, Text, Input, Table).
    * *Styling & Color Swatches*: 10 έτοιμα curated χρώματα για 1-click αλλαγή φόντου/κειμένου, ρύθμιση border radius και border width.
    * *On-Click Actions*: Σύνδεση με Target Screen και αυτόματο SQLite Insert.
  * **Ζώνη 2 (Κάτω 1/3 - Layers & Tree)**:
    * Δενδρική απεικόνιση ιεραρχίας σκηνής.
    * Κλείδωμα (`🔒`), απόκρυψη (`👁`), γρήγορη επιλογή και διαγραφή.
* **Canvas Floating HUD (Κάτω Μέρος)**:
  * *Bottom-Left Pill*: Ζωντανές συντεταγμένες κέρσορα (`X: 320 Y: 180`) και ένδειξη επιλεγμένου node.
  * *Bottom-Right Zoom Controller*: Κουμπιά `−`, `{zoom}%` (κλικ για 100%), `+`, και `⛶ Fit` για άμεση επαναφορά στο κέντρο.
  * *Dimension Badge HUD*: Real-time ένδειξη `W × H` απευθείας κάτω από το επιλεγμένο node κατά το σύρσιμο ή το resize.

#### Καρτέλα 2: Data Studio (Διαχειριστής Σχημάτων & Βάσης)
* **Visual Entity Designer**:
  * Ορισμός πινάκων (`service_tickets`, `inventory_parts`, `customers`).
  * Ορισμός πεδίων: `Text`, `Integer`, `Float/Currency`, `Date`, `Enum/Dropdown`, `Boolean`.
* **Additive-Only Migration Engine**:
  * Αυστηρός κανόνας: Δεν επιτρέπεται καταστροφικό `DROP TABLE` ή `DROP COLUMN` στα πακέτα. Επιτρέπονται μόνο `CREATE TABLE` και `ALTER TABLE ADD COLUMN`.
* **Formula & Metrics Studio**:
  * Live calculated fields (π.χ. `Total = Parts + Labor`, `VAT = Total * 0.24`).

#### Καρτέλα 3: Devices & Run (Προσομοιωτής & Δημοσίευση στο Hub)
* **Multi-Device Frame Simulator**:
  * Επιλογή προτύπου οθόνης: Desktop HD (1920×1080), Laptop (1440×900), Tablet (1024×768), Mobile (390×844).
  * Διαδραστική δοκιμή πλοήγησης και εισαγωγής δεδομένων σε πραγματικό χρόνο.
* **Τοπική Αποθήκευση Project (`.prproj`)**:
  * Απεριόριστη τοπική αποθήκευση και διαχείριση έργων στον δίσκο για άνοιγμα και επεξεργασία στη native Rust μηχανή.
* **Κλειστός Βρόχος Compilation (Anti-Bypass Cloud Compiler Gate)**:
  * **Απαγόρευση Τοπικού Compile**: Δεν παράγεται εκτελέσιμο `.pr` αρχείο τοπικά για απευθείας διανομή ("κάτω από το τραπέζι").
  * **Μονόδρομος Δημοσίευσης**: Το Studio υποβάλλει το project στο Proteus Hub. Το Hub εκτελεί το compilation, εφαρμόζει την κρυπτογραφική υπογραφή (Ed25519) και παραδίδει το πακέτο απευθείας στο `proteus-client` του πελάτη μόνο μετά την επιβεβαίωση της πληρωμής και τη δέσμευση στο In-Platform Escrow.

---

## 3. Πυλώνας 2: Proteus Client (The Shop Counter Runtime & Role Dashboards)

### 3.1 Σκοπός & Κοινό-Στόχος
* **Χρήστες**: Ιδιοκτήτες επισκευαστικών κέντρων, συνεργείων, τεχνικοί πάγκου, εξειδικευμένοι τεχνικοί IT support, customer service agents, sales consultants.
* **Αποστολή**: Καθημερινή εκτέλεση παραλαβών, διαχείριση ροής εργασιών (pipeline), έκδοση δελτίων σε θερμικό εκτυπωτή, τοπική αποθήκευση και εξειδικευμένη απομακρυσμένη διαχείριση πελατολογίου.
* **Τιμολόγηση**: **7.99€ / μήνα** ανά κατάστημα (με 30 ημέρες offline grace period).
* **Εκτελέσιμο**: Ενιαίο, αυτόνομο `.exe` (μόλις **5.0 MB**), χωρίς installers, χωρίς εξαρτήσεις.

### 3.2 Δομή Οθονών Ταμείου Καταστήματος (Shop Counter Views)

```
+---------------------------------------------------------------------------------------------------------+
| [PROTEUS BOS]  [🏪 Ταμείο] [🛠 IT Support] [📊 Ειδικά Dashboards]  |  [⚡ Νέα Παραλαβή] [📋 Ροή] [⚙ Ρυθμίσεις] |
+---------------------------------------------------------------------------------------------------------+
```

#### Οθόνη 1: Νέα Παραλαβή (Intake Form)
* **Στόχος**: Καταγραφή βλάβης και έκδοση αποδείξεως σε **λιγότερο από 30 δευτερόλεπτα**.
* **Πεδία Εισαγωγής (Δύο Στήλες)**:
  1. *Στήλη Πελάτη*: Ονοματεπώνυμο (*), Τηλέφωνο Επικοινωνίας (*), Εκτίμηση Κόστους (€).
  2. *Στήλη Συσκευής*: Μοντέλο Συσκευής/Οχήματος (*), Σειριακός Αριθμός / IMEI, Περιγραφή Βλάβης (*).
* **Κουμπιά Ενεργειών**:
  * `🖨 Αποθήκευση & Εκτύπωση`: Αποθηκεύει στην τοπική SQLite, αυξάνει αυτόματα το #Ticket (π.χ. #1042), δημιουργεί το binary ESC/POS stream και το στέλνει απευθείας στον Win32 Spooler.
  * `💾 Αποθήκευση Μόνο`: Καταχώρηση χωρίς εκτύπωση (για τηλεφωνικές παραγγελίες ή εκ των προτέρων ραντεβού).
  * `⟲ Καθαρισμός`: Άμεση επαναφορά πεδίων.
* **Ενημέρωση Χειριστή**: Toast notification πράσινο/κόκκινο με σαφές μήνυμα επιτυχίας ή σφάλματος.

#### Οθόνη 2: Ροή Επισκευών (Kanban Pipeline)
* **Στόχος**: Πλήρης οπτική εποπτεία όλων των ενεργών εργασιών στο εργαστήριο.
* **6 Στάδια Ροής (Lanes)**:
  1. `🔵 Παραλήφθηκε` (`Received`): Νέες εισαγωγές που περιμένουν διάγνωση.
  2. `🟡 Σε Εξέλιξη` (`InProgress`): Συσκευές στον πάγκο του τεχνικού.
  3. `🟣 Αναμονή Ανταλλακτικών` (`WaitingParts`): Παραγγελίες ανταλλακτικών.
  4. `🟢 Έτοιμο` (`Ready`): Επισκευασμένο, ειδοποίηση πελάτη.
  5. `⚪ Παραδόθηκε` (`Delivered`): Παραλαβή από πελάτη, εξόφληση, αρχειοθέτηση.
  6. `🔴 Ακυρώθηκε` (`Cancelled`): Ασύμφορη επισκευή ή απόρριψη προσφοράς.
* **Δυνατότητες Στήλης & Καρτέλας**:
  * Real-time search bar (αναζήτηση με όνομα, τηλέφωνο, μοντέλο, αριθμό δελτίου).
  * Κάρτα εισιτηρίου: Αριθμός `#Ticket`, όνομα, μοντέλο, τηλέφωνο, σύνοψη βλάβης, εκτιμώμενο κόστος σε ευρώ.
  * Κουμπί ταχείας προώθησης σταδίου (π.χ. `→ Σε Εξέλιξη`, `→ Έτοιμο`).
  * Κλικ στην κάρτα $\rightarrow$ Άνοιγμα του λεπτομερούς Modal.

#### Οθόνη 3: Καρτέλα Επισκευής (Ticket Detail Modal)
* **Στόχος**: Επιθεώρηση, διαγνωστικές σημειώσεις τεχνικού και οικονομική τακτοποίηση.
* **Περιεχόμενα**:
  * Πλήρη στοιχεία πελάτη και συσκευής.
  * Time-stamps εισαγωγής και τελευταίας τροποποίησης.
  * Μπάρα επιλογής σταδίου επισκευής με ένα κλικ.
  * Επεξεργάσιμο πλαίσιο τεχνικών σημειώσεων εργαστηρίου (Multi-line text area).
  * Πεδίο τελικού κόστους επισκευής (€).
  * Κουμπί `🖨 Επανεκτύπωση Δελτίου` (σε περίπτωση απώλειας από τον πελάτη).

#### Οθόνη 4: Ρυθμίσεις Καταστήματος & Εκτυπωτή (Settings)
* **Στοιχεία Επιχείρησης**: Επωνυμία, Διεύθυνση, Τηλέφωνο, Υποσέλιδο αποδείξεων (Footer terms).
* **Θερμικός Εκτυπωτής**:
  * Όνομα εκτυπωτή Windows Spooler (π.χ. `POS-80` ή `Generic / Text Only`).
  * Επιλογή πλάτους χαρτιού: `58mm (32 στήλες)` ή `80mm (48 στήλες)`.
  * Κουμπί `🖨 Δοκιμαστική Εκτύπωση (Test Print)`: Άμεση επαλήθευση hardware.
* **Τοπική Βάση**: Εμφάνιση διαδρομής αρχείου SQLite (`%APPDATA%\Proteus\data\store.db`) και επιβεβαίωση offline λειτουργίας.

### 3.3 Εξειδικευμένα Dashboards ανά Ρόλο (Specialist Dashboards)

Όταν ο χρήστης επιλέγει εξειδικευμένο ρόλο ή ενεργεί ως πιστοποιημένος τεχνικός, το `proteus-client` παρέχει αυτόνομα dashboards:

1. **🛠 IT Support & Remote Operations Dashboard**:
   - **Διαγνωστικά Υγείας**: Live έλεγχος SQLite, εκτυπωτή POS, απομόνωσης και ακεραιότητας.
   - **Απομακρυσμένη Συνεδρία (Remote PIN Generator)**: Δημιουργία 6-ψήφιου PIN όταν ο τεχνικός δεν μπορεί να μεταβεί φυσικά στο κατάστημα.
   - **Πελατολόγιο Τεχνικού (Client Shop Roster)**: Παρακολούθηση συνδεδεμένων καταστημάτων, uptime, τελευταίου sync και κατάστασης SLA.
   - **Ψηφιακό Συμβόλαιο SLA & Escrow**: Προβολή όρων σύμβασης και ασφαλισμένης αποδέσμευσης αμοιβής.
2. **🤝 Customer Service & Training Dashboard**:
   - **Παρακολούθηση Εκπαίδευσης**: Εκπαίδευση χειριστών ταμείου με διαδραστικούς οδηγούς.
   - **Intake Speed Benchmarking**: Μετρήσεις πραγματικού χρόνου παραλαβής (στόχος < 30 δευτερόλεπτα).
   - **Customer Feedback**: Δείκτες ικανοποίησης και αξιολογήσεις.
3. **📈 Sales & Solutions Consultant Dashboard**:
   - **Pipeline Ευκαιριών**: Υποψήφια καταστήματα προς αναβάθμιση.
   - **Quote & Hardware Calculator**: Αυτόματος υπολογισμός τιμών πακέτου (Core License + Cloud Backup + POS Printer).
   - **Tier Tracker**: Παρακολούθηση τρέχοντος Tier (π.χ. Tier 2: 35% προμήθεια), μηνιαίας συνδρομής (30€/μήνα) και στόχου για επόμενο Tier.
4. **🎨 Designer Studio Hub**:
   - Προβολή πρόσφατων `.prproj` projects και γρήγορη εκκίνηση του `proteus-studio`.
   - Κατάσταση έγκρισης και πωλήσεων στο Marketplace.
5. **💻 Developer Studio & Data Schema Hub (Προγραμματιστικό Περιβάλλον & Enterprise Migrations)**:
   - **Visual Entity & Field Schema Designer**: Επεξεργασία υφιστάμενων πινάκων (`service_tickets`, `inventory_parts`) και ορισμός custom οντοτήτων για επιχειρήσεις (`enterprise_assets`, `fleet_telematics`).
   - **Μηχανή Additive Migrations**: Αυστηρός κανόνας αποφυγής data loss (μόνο `CREATE TABLE` & `ALTER TABLE ADD COLUMN`). Αυτόματη παραγωγή και live syntax validation σε SQLite.
   - **Enterprise Developer Work Orders & Escrow**: Εταιρείες με in-house ή εξωτερικούς προγραμματιστές συνάπτουν συμβάσεις τροποποίησης δεδομένων. Η αμοιβή του developer (π.χ. 750€) δεσμεύεται σε Escrow και παραδίδεται με pull-request/package delivery στο client της επιχείρησης.
6. **📋 Activity Timeline & Audit Trail Hub (`AuditLogView`)**:
   - **Ιχνηλασιμότητα 100%**: Πλήρης καταγραφή κάθε ενέργειας (ποιος, πότε, τι άλλαξε) με μονοτονικά UUIDv7 events και ISO-8601 timestamps.
   - **Χρωματικά Badges Ρόλων**: CEO (Gold), Customer Service (Sky Blue), Technician (Emerald), Sales (Orange), Developer (Indigo).
   - **Εργαλεία Αναζήτησης & Επιθεώρησης**: Ζωντανή αναζήτηση, φιλτράρισμα ανά ρόλο, σχετικοί χρόνοι («πριν 5 λεπτά») και πτυσσόμενος JSON diff inspector.
7. **📅 Appointments & Client Scheduling Hub (`AppointmentsView`)**:
   - **Υποδοχή & Προγραμματισμός**: Κλείσιμο ραντεβού από το προσωπικό εξυπηρέτησης πελατών και πωλήσεων.
   - **Αυτόματο Audit Event**: Κάθε νέο ραντεβού ή αλλαγή κατάστασης εκπέμπει αυτόματα event στο audit log, ενημερώνοντας σε πραγματικό χρόνο όλα τα συναφή υποσυστήματα.

---

## 4. Πυλώνας 3: Proteus Web Hub & Marketplace (`proteus-hub`)

### 4.1 Σκοπός & Κοινό-Στόχος
* **Χρήστες**: Επισκέπτες, ιδιοκτήτες καταστημάτων που αγοράζουν άδεια, πιστοποιημένοι designers που δημοσιεύουν πακέτα, διαχειριστές.
* **Αποστολή**: Εμπορική διάθεση, διαχείριση συνδρομών, κατάλογος επαληθευμένων επιχειρησιακών προτύπων (`.pr`), ασφαλείς πληρωμές και in-platform escrow.
* **Τεχνολογία**: Web Portal (Vanilla CSS + σύγχρονη ελαφριά αρχιτεκτονική, Backend API σε Rust/Axum).

### 4.2 Ενότητες του Web Portal

```
+-----------------------------------------------------------------------------------------+
| [PROTEUS HUB]    [Showcase]    [Marketplace Προτύπων]    [Αγορά Άδειας]    [Είσοδος]    |
+-----------------------------------------------------------------------------------------+
```

#### Ενότητα 1: Δημόσια Αρχική Σελίδα (Landing & Showcase)
* **Hero Section**:
  * Κεντρικό μήνυμα: *"Το Μοναδικό Business OS για Επισκευές που Λειτουργεί 100% Τοπικά, Χωρίς Cloud, με Αυτόματη Θερμική Εκτύπωση"*.
  * Άμεσο CTA: `⚡ Κατεβάστε το Proteus Client (.exe)` (άμεση δοκιμή χωρίς εγγραφή).
* **Διαδραστική Επίδειξη**:
  * Video walkthrough & interactive preview των 3 βασικών οθονών (Intake $\rightarrow$ Kanban $\rightarrow$ Print).
* **Σύγκριση με Ανταγωνισμό**:
  * Πίνακας σύγκρισης (Proteus vs Generic Cloud CRMs): Μηδενικό lag, λειτουργία χωρίς internet, απευθείας υποστήριξη θερμικών POS εκτυπωτών, χαμηλό μηνιαίο κόστος (7.99€ vs 30-70€).

#### Ενότητα 2: Προσαρμοσμένη Υποβολή Έργου & Slot Board (Zero Presets / 100% Bespoke)
* **Απόλυτη Αρχή: Μηδενικά Προκατασκευασμένα Presets**:
  * Το Proteus ΔΕΝ διαθέτει ούτε πουλάει έτοιμα, generic "πακέτα-κονσέρβα" (όχι συνεργεία, όχι ιατρεία, όχι έτοιμα templates λιανικής).
  * Ο ίδιος ο πελάτης συντάσσει λεπτομερώς το επιχειρησιακό του προφίλ μέσα από τη φόρμα υποβολής:
    1. *Τι ακριβώς είναι η επιχείρησή του και ποιες είναι οι καθημερινές της ανάγκες*.
    2. *Ποιες είναι οι ροές εργασίας, τα μηχανήματα, οι ιδιαιτερότητες και οι πελάτες της*.
    3. *Τι ακριβώς απαιτεί να σχεδιαστεί (ειδικές οθόνες ταμείου, αποθήκη, τιμολόγηση, φόρμες, εκτυπώσεις)*.
  * **Αρχή Ευθύνης Επιχειρηματία**: Αν ένας πελάτης δεν μπορεί να περιγράψει αναλυτικά τις ανάγκες της επιχείρησής του, δεν μπορεί να αποκτήσει λειτουργικό λογισμικό.
* **Ανάρτηση Έργου στο 7-Slot Board & Ανάθεση Ρόλων**:
  * Το αναλυτικό brief του πελάτη μετατρέπεται σε επίσημο project στο Slot Board του Web Portal.
  * Το έργο στελεχώνεται από πιστοποιημένους freelancers μέσω In-Platform Escrow:
    1. *Business Analyst (BA)*: Επιχειρησιακοί κανόνες, λογική εγκρίσεων και προδιαγραφές.
    2. *Data Analyst (DA)*: Σχεδιασμός SQLite σχημάτων, πίνακες, σχέσεις, επικυρώσεις.
    3. *Software UI/UX Designer (PCD-App)*: Σχεδίαση διεπαφής του εσωτερικού λογισμικού καταστήματος/ταμείου στο Proteus Studio.
    4. *Web & E-Commerce Designer (PCD-Web)*: Σχεδίαση της δημόσιας ιστοσελίδας / e-shop που συνδέεται ζωντανά με τη βάση.
    5. *Systems IT Specialist (PCSS)*: LAN δικτύωση, Merkle logs, Cloudflare Tunnels, ασφάλεια και replication.
    6. *Field Support Deployer (PCDS)*: Ρύθμιση φυσικού hardware (θερμικός εκτυπωτής, συρτάρι, barcode scanner).
    7. *Customer Support (CS)*: Onboarding και εκπαίδευση προσωπικού.
  * **Ευελιξία Full-Stack**: Ένας επαγγελματίας που κατέχει πολλαπλές βεβαιώσεις μπορεί να αναλάβει 2, 3 ή και όλα τα slots του έργου μόνος του.

#### Ενότητα 2.1: Υπηρεσίες Domain, Managed Web Hosting & Επεκτασιμότητα (Νέα Ροή Εσόδων)
* **Κατοχύρωση & Ενοικίαση Domain Names**:
  * Διασύνδεση με Registrar API (π.χ. Cloudflare Registrar / Namecheap / Papaki API).
  * Ο πελάτης αναζητά και κατοχυρώνει το επίσημο domain της επιχείρησής του (π.χ. `.gr`, `.com`) απευθείας μέσα από την πύλη του Proteus.
  * Αυτόματη παραμετροποίηση DNS (A, CNAME, TXT, SSL/TLS) χωρίς καμία τεχνική δυσκολία για τον πελάτη.
  * Πρόσθετο recurring έσοδο για την πλατφόρμα από την προμήθεια ετήσιας ανανέωσης domain.
* **Μοντέλα Φιλοξενίας Ιστοσελίδας / E-shop**:
  1. *Managed Cloud Hosting (Proteus Infrastructure)*:
     * Φιλοξενία στο δίκτυο του Proteus (`proteus-web` μέσω Cloudflare Tunnel / Edge).
     * Μηνιαία συνδρομή φιλοξενίας & διαχείρισης (π.χ. 15€ – 35€ / μήνα ανάλογα με τον όγκο δεδομένων).
     * Αυτόματα SSL, απεριόριστο bandwidth, αυτόματο incremental backup της βάσης.
  2. *Self-Hosting (Ιδιόκτητος Server Πελάτη)*:
     * Ο πελάτης φιλοξενεί την ιστοσελίδα και τις βάσεις του στο δικό του μηχάνημα/hardware.
     * Πληρώνει μόνο το εφάπαξ setup fee και τη βασική άδεια συγχρονισμού.
* **Επεκτασιμότητα & Custom Αλληλεπιδράσεις Πελατών**:
  * Δυνατότητα προσθήκης έξτρα βάσεων δεδομένων και custom web endpoints (π.χ. B2B portal παραγγελιών, portal ραντεβού, customer loyalty accounts), όλα συνδεδεμένα με το τοπικό SQLite core.

#### Ενότητα 3: Πύλη Αδειών & Μαθηματική Κλιμάκωση Συνδρομών
* **Proteus Core License (Τοπική Άδεια / Self-Hosted)**:
  * **Βάση (1–4 χρήστες):** **7,99€ / μήνα** flat.
  * **Ζώνη 1 (5–20 χρήστες):** **+1,50€** ανά επιπλέον χρήστη.
  * **Ζώνη 2 (21–60 χρήστες):** **+1,00€** ανά επιπλέον χρήστη.
  * **Ζώνη 3 (61–150 χρήστες):** **+0,60€** ανά επιπλέον χρήστη.
  * **Enterprise Cap (150+ χρήστες):** **199€ / μήνα flat** (απεριόριστοι χρήστες για την τοπική άδεια).
* **Managed Cloud Sync & Cloud Backups (Προαιρετικό Add-on)**:
  $$\text{Cloud Fee} = \text{Base Backup Fee} + (\text{Active Users} \times \text{Per-Seat Sync Fee})$$
  * **Base Automated Backups:** **+9,99€ / μήνα** (καλύπτει cold storage έως 50GB, retention 30 ημερών).
  * **Managed Live Sync & Cloud DB:**
    * *Έως 20 χρήστες:* +15€ / μήνα (Κόστος Hetzner VPS: ~5€ $\rightarrow$ **Περιθώριο: 66%**)
    * *21–100 χρήστες:* +45€ / μήνα (Κόστος υποδομής: ~12€ $\rightarrow$ **Περιθώριο: 73%**)
    * *101–500 χρήστες:* +120€ / μήνα (Κόστος υποδομής: ~28€ $\rightarrow$ **Περιθώριο: 76%**)
    * *5.000 χρήστες (Dedicated Cluster):* **1,50€ / χρήστη / μήνα = 7.500€ / μήνα** (Κόστος υποδομής: ~600€ $\rightarrow$ **Καθαρό κέρδος: 6.900€ / μήνα**).
* **Κρυπτογραφική Έκδοση & Self-Service**:
  * Ασύμμετρη υπογραφή Ed25519 με μοναδικό Machine Fingerprint, 30-day offline lease token και δυνατότητα μεταφοράς άδειας με 1 κλικ.

#### Ενότητα 4: Marketplace, Πιστοποιήσεις & Creator Economy

##### Α. Οικονομικά Πιστοποιήσεων (Certifications Unit Economics)
Οι πιστοποιήσεις διαχωρίζονται αυστηρά ανάλογα με την εξειδίκευση του επαγγελματία, καθώς το UI/UX λογισμικού διαφέρει ριζικά από το UI/UX ιστοσελίδων/e-shop:
* **PCD-App Exam Voucher (Software & Desktop UI/UX):** **79€** (Σχεδιασμός εσωτερικού λογισμικού, πυκνότητα πληροφορίας, συντομεύσεις πληκτρολογίου, εργονομία ταμείου).
* **PCD-Web Exam Voucher (Website & E-Commerce UI/UX):** **79€** (Σχεδιασμός δημόσιας ιστοσελίδας, storefront, responsive mobile web, conversion rates, online checkout).
* **Dual Full-Stack Designer Bundle (PCD-App + PCD-Web):** **129€** (Καλύπτει και τους δύο ρόλους σχεδίασης).
* **Master Partner All-Roles Bundle (BA, DA, PCD-App, PCD-Web, PCSS, PCDS):** **249€** εφάπαξ.
* **Ετήσιο Verified Partner Badge (Maintenance/Listing Fee):** **39€ / έτος** (διατήρηση στο επίσημο μητρώο πιστοποιημένων συνεργατών).

##### Β. Οικονομικά Υπηρεσιών Slot Board & Escrow (Take-Rate)
Η πλατφόρμα λειτουργεί ως εκκαθαριστής πληρωμών με απόλυτη διαφάνεια (Anti-Rent-Seeking):
* **Προμήθεια Πλατφόρμας (Take-Rate):** **18%** Gross (16% καθαρά στο Proteus, 2% Stripe/Banking fees).
* **Καθαρή Αμοιβή Συνεργάτη:** **82%** καθαρά.
* **Custom Project Slot (Μέση τιμή ανά ρόλο: 350€):** Πελάτης πληρώνει 350€ $\rightarrow$ Συνεργάτης 287€ $\rightarrow$ **Proteus 63€**.
* **Domain & Hosting Recurring:** Πελάτης πληρώνει 25€/μήνα $\rightarrow$ Κόστος υποδομής ~3€ $\rightarrow$ **Καθαρό κέρδος Proteus 22€/μήνα**.

##### Γ. Συνδυαστικές Προβολές Εσόδων (Target Projections)
| Φάση Ανάπτυξης | Πιστοποιημένοι Συνεργάτες | Ενεργές Επιχειρήσεις | Μηνιαία Έσοδα Συνδρομών | Μηνιαία Έσοδα Marketplace / Services | Μηνιαία Έσοδα Certifications | Συνολικό Μηνιαίο MRR |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| **Φάση 1 (Εκκίνηση)** | 30 | 25 | 450€ | 380€ | 200€ | **1.030€** |
| **Φάση 2 (Growth)** | 100 | 120 | 2.800€ | 1.800€ | 650€ | **5.250€** |
| **Φάση 3 (Scale)** | 350 | 500 | 14.500€ | 8.200€ | 2.300€ | **25.000€** |

---

## 5. Διάγραμμα Ροής Δεδομένων & Αλληλεπίδρασης (Data & Token Flow)

```
+-------------------------------------------------------------------------------------------------+
|                                1. BESPOKE CLIENT PROJECT BRIEF                                  |
|  - Πελάτης συμπληρώνει: Επιχειρησιακό Προφίλ, Καθημερινές Ροές, Οθόνες, Εκτυπωτές, Προϋπολογισμό|
+------------------------------------------------+------------------------------------------------+
                                                 |
                                                 v
+-------------------------------------------------------------------------------------------------+
|                             2. PROTEUS WEB HUB (7-SLOT BOARD & ESCROW)                          |
|  - Ανάρτηση Έργου στο 7-Slot Board (BA, DA, PCD-App, PCD-Web, PCSS, PCDS, CS)                  |
|  - In-Platform Escrow Δέσμευση Κεφαλαίων & Anti-Bypass Algorithmic Floor Protection            |
|  - Κατοχύρωση Domain (.gr/.com) & Επιλογή Hosting (Managed Cloud vs Self-Hosted)                |
+-------------------+----------------------------+-----------------------------+------------------+
                    |                            |                             |
      (PCD-App Slot)|              (PCD-Web Slot)|                (DA/DB Slot) | (PCSS/PCDS Slot)
                    v                            v                             v                  v
+-----------------------+     +----------------------+     +----------------------+   +-----------+
| PROTEUS STUDIO (App)  |     | PROTEUS STUDIO (Web) |     | DATA STUDIO & SMLM   |   | HARDWARE  |
| - Layouts Ταμείου POS |     | - Storefront & E-Shop|     | - Additive Migrations|   | - ESC/POS |
| - Dense Desktop Forms |     | - Responsive Breakpts|     | - DDL & SQLite Tables|   | - LAN Sync|
+-----------+-----------+     +----------+-----------+     +----------+-----------+   +-----+-----+
            |                            |                            |                     |
            +----------------------------+-------------+--------------+---------------------+
                                                       |
                                                       v
+-------------------------------------------------------------------------------------------------+
|                         3. CLOUD COMPILER GATE & ENCRYPTED DELIVERY (.pr)                       |
|  - Cloud Verification & Ed25519 Cryptographic Signature (Μηδενικό τοπικό compile bypass)       |
|  - Αποδέσμευση Escrow Αμοιβών (82% Συνεργάτες / 18% Proteus)                                   |
+------------------------------------------------+------------------------------------------------+
                                                 |
                                                 v
+-------------------------------------------------------------------------------------------------+
|                      4. FRONTLINE SHOP COUNTER RUNTIME (`proteus-client.exe`)                   |
|  - Mounts Signed .pr Package & Offline Local SQLite Store (`store.db`)                          |
|  - Native Win32 Raw ESC/POS Thermal Printing & Barcode Scanning                                 |
|  - Real-time LAN Peer Sync με Handheld Mobile Companion (`proteus-mobile`)                      |
|  - Federation Bridge με Cloud Databases (`web_orders`, `web_customers`, `appointments_db`)    |
+-------------------------------------------------------------------------------------------------+
```

---

## 6. Οδικός Χάρτης Υλοποίησης (Implementation Blueprint)

1. **Άμεσο Βήμα 1 (Εκτελέστηκε)**:
   * Υλοποίηση του πυρήνα `proteus-core` (διαδρομές `%APPDATA%`, DDL `service_tickets`, raw ESC/POS printing, SMLM semantic profiler, supplier catalog reconciler).
   * Υλοποίηση του αυτόνομου `proteus-client` (Intake, Kanban, Detail Modal, Settings, Supplier Reconciliation, Contractor Ledger) σε native binary (5.0 MB).
   * Αναβάθμιση του `proteus-studio` (`proteus-design-studio`) με dimension HUD, alignment bar, luxury swatches, floating canvas HUD, Track Switcher (PCD-App vs PCD-Web).
   * Υλοποίηση του `proteus-mobile` (Handheld Companion για τεχνικούς αποθήκης & delivery, με TCP sync στο LAN).
2. **Βήμα 2: Web Portal, Domain Gateway & Escrow (Εκτελέστηκε)**:
   * Επίσημο Web Hub & Portal (`proteus-web` σε Rust/Axum).
   * Υποβολή Bespoke Project Briefs & 7-Slot Board matching.
   * Διαχωρισμός πιστοποιήσεων: PCD-App (79€), PCD-Web (79€), Dual Bundle (129€), Master Bundle (249€).
   * Domain Reseller Gateway (`.gr`, `.com`, `.eu`, `.shop`) με αυτόματη DNS ζώνη και SSL tokens.
   * Managed Cloud Hosting vs Self-Hosting & δυναμική σύνδεση πρόσθετων cloud βάσεων (`web_orders`, `web_customers`).
3. **Βήμα 3: Πιλοτική Εγκατάσταση (Οκτώβριος 2026)**:
   * Πιλοτική εγκατάσταση του `proteus-client.exe` σε 2–3 συνεργεία / καταστήματα επισκευής για συλλογή πραγματικού feedback.
   * Δοκιμή LAN synchronization μεταξύ ταμείου και mobile handheld συσκευής.

---
*Έγγραφο εγκεκριμένο από την Ομάδα Ανάπτυξης Proteus — Πλήρης ευθυγράμμιση με τις αρχές του Founder.*
