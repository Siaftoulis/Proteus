---
tags:
  - company/strategy
  - business/scope
aliases:
  - Master Strategy
  - Product Scope & Positioning
---
# Proteus — Master Strategy, Market Positioning & Scope Definition

> **Executive Alignment:** Approved by Founder / CEO (September 2026)  
> **Brand Identity:** **Proteus** — The Native Desktop CRM Builder  
> **Business Philosophy:** 100% Self-Funded (Bootstrapped), High Margin, Founder Independence.

---

## 1. Το Brand & Η Ταυτότητα: "Proteus BOS"

Το **Proteus** διατηρεί τη δική του αυτόνομη ταυτότητα:
- **Τι είναι:** Ένα native visual builder εργαλείο για σχεδιασμό και άμεση εκτέλεση custom CRM εφαρμογών.
- **Native Runtime:** Χτισμένο σε καθαρό Rust + egui + SQLite, χωρίς εξάρτηση από web browsers ή Electron.
- **Cross-Platform:** Ένας ενιαίος native πυρήνας που τρέχει αστραπιαία σε Windows, macOS, Linux (και μελλοντικά mobile).
- **Extensible Schema:** Δυναμικό data engine (EAV / JSON records) όπου η προσθήκη νέων πεδίων, στηλών ή φορμών δεν καταστρέφει ούτε μπλοκάρει τη βάση δεδομένων.

---

## 2. Ανάλυση Ανταγωνισμού & The "Proteus Wedge"

| Κατηγορία Ανταγωνισμού | Παραδείγματα | Αδυναμία Ανταγωνιστή | Πού Κερδίζει το Proteus |
| :--- | :--- | :--- | :--- |
| **Big Cloud SaaS** | HubSpot, Salesforce, Pipedrive | Πανάκριβο ($20–$150/χρήστη/μήνα), αργό cloud latency, μηδενική ιδιωτικότητα, καθόλου offline χρήση. | **100% Local-first**, δωρεάν για 1–3 χρήστες, απόλυτος έλεγχος δεδομένων, boot σε < 50ms. |
| **Self-Hosted Open Source** | Twenty, SuiteCRM, EspoCRM | Απαιτούν server, Docker, Linux sysadmin γνώσεις και συνεχή συντήρηση. | **Desktop App (Zero-Sysadmin)**. Ανοίγει με διπλό κλικ στο PC χωρίς καμία εγκατάσταση server. |
| **Generic No-Code Tables** | Airtable, Notion, Baserow | Είναι γενικά λογιστικά φύλλα / πίνακες, όχι dedicated visual CRM περιβάλλον. | **Visual CRM Builder** με έτοιμα modules (Kanban Deals, Contacts, Tasks, Flows). |

---

## 3. Κοινό & ICP (Τρεις Πυλώνες)

1. **Μικρομεσαίες Επιχειρήσεις (1–50 άτομα) [Κύριος Στόχος]**:
   - Επιχειρήσεις που θέλουν ένα CRM ακριβώς στα μέτρα τους χωρίς να πληρώνουν χιλιάδες ευρώ το χρόνο.
   - Δυνατότητα συνεργασίας: Ο ιδιοκτήτης και ο designer/υπεύθυνος σχεδιάζουν μαζί το CRM.
2. **Solo Επαγγελματίες / Freelancers (Δικηγόροι, Μηχανικοί, Σύμβουλοι)**:
   - Θέλουν ένα γρήγορο, απλό, καθαρό εργαλείο στο laptop τους για το πελατολόγιό τους.
   - Διεπαφή σχεδιασμένη ώστε να είναι κατανοητή από οποιονδήποτε χρήστη ("average IQ" UX).
3. **Agencies & CRM Builders**:
   - Σχεδιαστές που φτιάχνουν έτοιμα CRMs και τα παραδίδουν/πωλούν σε δικούς τους πελάτες.

---

## 4. Τιμολόγηση & Business Model (GDP-Adjusted & Free Tier)

### Α. Δωρεάν Έκδοση (Free Forever): 1 έως 3 Χρήστες
- **Κόστος:** **€0 / μήνα** για πάντα.
- **Δυνατότητες:** Πλήρης πρόσβαση στον visual builder, τοπική SQLite, Contacts, Deals, Tasks.
- **Συγχρονισμός:** Τοπικό P2P / Headless sync μεταξύ 2-3 συσκευών του ίδιου γραφείου χωρίς υποχρεωτικό server.
- **Κλείδωμα ορίου:** Αυστηρός περιορισμός στους 3 χρήστες (για να υπάρξει monetization όταν η επιχείρηση μεγαλώνει).

### Β. Ομάδες & Επιχειρήσεις (4+ Χρήστες) — GDP / PPP Adjusted
Οι τιμές κλιμακώνονται δυναμικά με βάση την αγοραστική δύναμη (GDP/PPP) κάθε χώρας:
- **Ελλάδα / Νότια Ευρώπη (Ενδεικτικά):**
  - **Self-Hosted Option:** Βασική συνδρομή **€9.99 / μήνα** + μικρή κλιμάκωση ανά χρήστη CRM.
  - **Managed Cloud / Hosted Option:** **€6.99 / μήνα** (βάση hosting) + fee ανά χρήστη (με υπογεγραμμένο συμβόλαιο ότι τα δεδομένα ανήκουν αποκλειστικά στον πελάτη και δεν αγγίζονται από κανέναν).
- **Tier 1 Χώρες (ΗΠΑ, Γερμανία, UK):**
  - Προσαρμοσμένη τιμολόγηση με βάση το υψηλότερο GDP ($29–$49/mo).
- **Tier 3 Χώρες (Ινδία, Λατινική Αμερική):**
  - Προσιτή τιμολόγηση προσαρμοσμένη στο τοπικό κόστος ζωής (π.χ. $3–$5/mo).

---

## 5. Απόφαση Εύρους (Scope Decision — v1.0 MVP)

Για να φτάσουμε σε πραγματικούς πελάτες μέσα σε **4–6 εβδομάδες** και να μην χαθούμε σε ατελείωτη ανάπτυξη:

### ✅ ΕΝΤΟΣ SCOPE (v1.0 Launch):
1. **Visual Canvas + Real Data-Bound Widgets**:
   - Το `DataTable` και τα input forms να διαβάζουν και να γράφουν ζωντανά στη SQLite.
2. **Unified Data Layer**:
   - Τα Contacts και Deals να αποθηκεύονται σε καθαρούς πίνακες SQLite ώστε να υπάρχει πλήρης συμβατότητα με τα widgets.
3. **Core Modules**:
   - Contacts (με σημειώσεις), Pipeline (7-stage Kanban), Tasks (προτεραιότητες/φίλτρα).
4. **Project Sharing / Revision Sync**:
   - Αποθήκευση έργου και εξαγωγή `.crmb` κρυπτογραφημένου αρχείου (για συνεργασία ιδιοκτήτη + designer).
5. **Shortcuts & Undo/Redo**:
   - `Ctrl+Z`, `Ctrl+C/V`, `Delete` για ευχρηστία επαγγελματικού επιπέδου.

### ❌ ΕΚΤΟΣ SCOPE (Αναβολή για v2.0):
- ❌ Ταυτόχρονο Google Docs-style real-time collaboration (multiplayer cursor).
- ❌ Native Mobile apps (iOS/Android) — παραμένουμε σε καθαρό Desktop (Win/Mac/Linux).
- ❌ Πολύπλοκες εξωτερικές ενσωματώσεις cloud / AI pipelines.

---

## 6. Startup Flow, Authentication Gate & Proteus Hub

### Α. Εκκίνηση Εφαρμογής & Splash Screen
- **Εκκίνηση:** Με διπλό κλικ / tap στο εικονίδιο, εμφανίζεται animated splash screen με το επίσημο λογότυπο του **Proteus** και ένδειξη φόρτωσης (< 1.5s boot).
- **Πολυπλατφορμική Υποστήριξη:** Native build targeting για Windows (x64, 32-bit, ARM), Linux (Debian/Ubuntu/Arch x64 & ARM), και μελλοντικό headless / VR runtime target.

### Β. Υποχρεωτικό Login Gate (Seat & Device Management)
- **Απαγόρευση Ανώνυμης Εισόδου:** Δεν επιτρέπεται η χρήση του builder χωρίς ενεργό login.
- **Σκοπός:** Αυστηρός έλεγχος του licensing: πόσοι χρήστες (seats) έχουν πρόσβαση και πόσες συσκευές έχουν συνδεθεί ανά άδεια χρήσης.
- **Μέθοδοι Σύνδεσης:**
  1. **Company Account:** Εταιρικό email & password.
  2. **Social / OAuth Providers:** Google, GitHub, GitLab, Facebook κ.λπ., ανάλογα με την επιλογή της επιχείρησης.
  3. **Offline Grace Cache:** Αποθήκευση έγκυρου κρυπτογραφημένου token τοπικά για απρόσκοπτη offline λειτουργία.

### Γ. Proteus Hub (Κεντρικό Ταμπλό Έργων)
- **User Identity Bar:** Εμφάνιση ενεργού προφίλ χρήστη (avatar, όνομα, ρόλος, ενεργή άδεια) πάνω δεξιά.
- **Project Board:** Λίστα ενεργών και πρόσφατων projects με άμεσο άνοιγμα (instant loading στο Designer με 1 κλικ ή διπλό κλικ).
- **News & Updates:** Ενσωματωμένη ροή ενημερώσεων (changelog, νέες εκδόσεις, features).
- **Marketplace & Templates:** Κατάλογος έτοιμων CRM templates και επεκτάσεων.

---

## 7. Workspace Architecture, Dockable Panels & Smart Container System

### Α. Inspector Panel (Proteus Dual-Zone Layout)
- **Άνω Ζώνη (2/3 Ύψους):** Ιδιότητες & Styling (Transform X/Y/W/H, Fill Color + Alpha, Stroke/Border, Corner Radius slider, Padding, Data Binding).
- **Κάτω Ζώνη (1/3 Ύψους):** Ιεραρχία Σκηνής / Layers & Groups (Δέντρο στοιχείων, κλείδωμα 🔒, ορατότητα 👁).

### Β. Dockable / Ευέλικτα Panels (Customizable Workspace)
- **Default Layout:** Προκαθορισμένη καθαρή διάταξη κατά την πρώτη εκκίνηση, συνοδευόμενη από interactive onboarding tutorial & documentation.
- **Ευελιξία & Μετακίνηση:** Ο χρήστης μπορεί να αποσπάσει ή να επανατοποθετήσει τα πάνελ ανάλογα με τη ροή εργασίας του:
  - **Work Tree (Αρχεία / Assets)**
  - **Database Schema Tree (Πίνακες / Πεδία SQLite)**
  - **Scene / Layer Tree**
  - **Flow Logic Canvas (Αυτοματισμοί & DAG βελάκια)**

### Γ. Smart Rectangle / Container System (Atomic Piece #1)
- **Διπλή Λειτουργία (Dual Mode):**
  1. **Static Shape:** Απλό γεωμετρικό σχήμα / background χωρίς σύλληψη συμβάντων ή πεδίων.
  2. **Smart Cell / Container:** Δυναμικό κελί εισαγωγής δεδομένων, φορμών και πινάκων.
- **Απόλυτη Ελευθερία Διαστάσεων:** Μεγάλα ή μικρά inputs, με προτεινόμενο default μέγεθος και πλήρη παραμετροποίηση.
- **Αντιγραφή / Επικόλληση Διαστάσεων:** Δυνατότητα αποθήκευσης/αντιγραφής ακριβούς μεγέθους και εφαρμογής σε γειτονικά κελιά.
- **Χάρακες & Οδηγοί Ευθυγράμμισης (Smart Rulers / Alignment Guides):** Οπτικές γραμμές snapping κατά το drag & resize για άψογη στοίχιση.

---

## 8. Anti-Piracy, Anti-Mod & Binary Security Architecture

Για τη θωράκιση της εμπορικής αξίας του Proteus απέναντι σε crackers, modders και πειρατεία:

1. **Native Compiled Machine Code (Zero Javascript/Electron Leakage)**:
   - Σε αντίθεση με web/Electron εφαρμογές όπου ο πηγαίος κώδικας είναι εκτεθειμένος σε plaintext `.asar`, το Proteus γίνεται compile σε native x86_64/ARM64 machine code.
   - Χρήση `strip = "symbols"`, `lto = "fat"`, και `opt-level = 3` στο release profile, καθιστώντας το reverse engineering / decompilation εξαιρετικά επίπονο και δυσανάγνωστο.
2. **Ed25519 Asymmetric Cryptographic Licensing**:
   - Οι άδειες υπογράφονται ασύμμετρα από τον κεντρικό License Server με Private Key.
   - Το binary του Proteus περιέχει μόνο το Public Key για επαλήθευση. Δεν είναι δυνατό να παραχθεί "Keygen" χωρίς το private key του server.
3. **Hardware-Locked Machine Fingerprint**:
   - Το token της άδειας δεσμεύεται με κρυπτογραφικό hash των αναγνωριστικών του συστήματος (CPU ID, Motherboard UUID, MAC).
   - Αντιγραφή του αρχείου άδειας σε άλλο υπολογιστή αποτυγχάνει αυτόματα.
4. **Encrypted Project Payloads (XChaCha20-Poly1305)**:
   - Όλα τα project files (`.crmb`) και τα αποθηκευμένα δεδομένα κρυπτογραφούνται με σύγχρονους αλγορίθμους RustCrypto.
5. **Anti-Tamper & Integrity Verification**:
   - Έλεγχοι ακεραιότητας κατά την εκκίνηση και προστασία μνήμης ώστε να ανιχνεύονται dynamic debuggers / memory patchers.

---

## 9. Closed Template Marketplace & Creator Monetization

1. **Κλειστό Οικοσύστημα Διανομής**:
   - Οι σχεδιαστές / agencies δεν μπορούν να εξάγουν ανεξέλεγκτα "γυμνά" (loose JSON/unencrypted) templates για να τα πουλάνε έξω από την πλατφόρμα.
   - Όλα τα templates πακετάρονται σε κρυπτογραφημένα, υπογεγραμμένα πακέτα που ξεκλειδώνουν αποκλειστικά μέσω του **Proteus Hub**.
2. **Ενσωματωμένη Προμήθεια (Commission & Revenue Share)**:
   - Κάθε πώληση template διεκπεραιώνεται μέσω της πλατφόρμας του Proteus.
   - Αυτόματη απόδοση ποσοστού προμήθειας στην πλατφόρμα και εκκαθάριση στον δημιουργό.
3. **White-Label Standalone Export (B2B Agency Tier)**:
   - Εξαγωγή branded αυτόνομου εκτελέσιμου (π.χ. `CustomerApp.exe`) για εταιρικούς πελάτες, αυστηρά κλειδωμένο με εταιρικό license key.

---

## 10. Direct Hardware Integration (Physical & Operations Layer)

Λόγω του Native Rust πυρήνα, το Proteus αποκτά άμεση πρόσβαση στο φυσικό hardware χωρίς μεσάζοντες:
- **Barcode & QR Code Scanners**: Άμεση σύλληψη κωδικών μέσω USB/HID/Virtual COM για αποθήκες, logistics, παραλαβές και παραδόσεις.
- **Θερμικοί Εκτυπωτές (POS / Label Printers)**: Άμεση αποστολή ESC/POS εντολών για εκτύπωση αποδείξεων, δελτίων αποστολής και ετικετών με 1 κλικ, χωρίς browser dialogs.
- **Hardware Peripherals**: Μελλοντική διασύνδεση με Stream Decks, ψηφιακές ζυγαριές και IoT ελεγκτές.

---

## 11. AI Architecture via MCP (Model Context Protocol)

Αντί για κοστοβόρα φιλοξενία ιδιόκτητων μοντέλων AI σε GPU servers:
- **Zero Recurring AI Hosting Costs**: Το Proteus δεν επιβαρύνεται με κόστη υποδομών AI.
- **Υποστήριξη MCP (Model Context Protocol)**: Ενσωμάτωση standard MCP client/server στο Proteus.
- **Bring-Your-Own-Key (BYOK)**: Ο τελικός χρήστης συνδέει όποιον AI provider επιθυμεί (Claude, OpenAI, Gemini, Local Ollama) πληρώνοντας άμεσα το δικό του API.
- **AI Canvas & Schema Automation**: Το AI αποκτά πρόσβαση μέσω MCP tools για να:
  - Σχεδιάζει αυτόματα φόρμες και widgets στο Canvas.
  - Δημιουργεί αυτόματα custom πίνακες και πεδία στη SQLite.
  - Συνδέει αυτόματα Data Bindings και Workflows.


