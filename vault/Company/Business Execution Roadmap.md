---
tags:
  - company/roadmap
  - business/strategy
aliases:
  - Business Execution
  - Product Roadmap
---
# Proteus — Business Execution Roadmap & Strategic Concept

> **Status:** Active Execution (Sprint 4 — Core CRM Architecture)  
> **Target Launch:** Closed Beta (5–10 pilot customers)  
> **Philosophy:** Local-First, High Performance (Rust + egui + SQLite), Zero Bloat.

---

## 1. Το Όραμα & Η Ταυτότητα του Προϊόντος (The Concept)

### "Figma for CRM — Το CRM που οι ομάδες όντως χρησιμοποιούν"
Το 63% των CRM projects αποτυγχάνει λόγω υπερβολικής πολυπλοκότητας (over-engineering), αργών cloud διεπαφών και υποχρεωτικών μηνιαίων συνδρομών που κρατούν τα δεδομένα «ομήρους».

Το **Proteus** λύνει αυτό το πρόβλημα συνδυάζοντας:
1. **Visual Drag & Drop Designer**: Οποιοσδήποτε χρήστης (ακόμα και χωρίς τεχνικές γνώσεις) μπορεί να σχεδιάσει φόρμες, πίνακες και ροές εργασίας ακριβώς στα μέτρα της επιχείρησής του.
2. **Local-First & Offline-First Αυτονομία**: Όλα τα δεδομένα ζουν τοπικά στον υπολογιστή σε SQLite (`crm.db`). Μηδενικό latency (< 50ms startup), 60 FPS rendering, πλήρης ιδιωτικότητα.
3. **Καθαρή Εμπορική Πολιτική (No Bait & Switch)**: Το τοπικό λογισμικό ανήκει στον χρήστη για πάντα (€300–400 lifetime ή $29/mo flat rate).

---

## 2. Νέα Αρθρωτή Αρχιτεκτονική Κώδικα (Modular Hierarchy)

Μετά το πρόσφατο refactoring, καταργήθηκε το μονολιθικό αρχείο και επιβλήθηκε αυστηρός κανόνας **κανένα αρχείο να μην υπερβαίνει τις 300–400 γραμμές**:

```
project/crm-ui/src/
├── main.rs                 # Ελαφρύς συντονιστής (~450 γραμμές, lifecycle & routing)
├── theme.rs                # Windows 11 / Linear dark design system
├── models.rs               # Domain types (Contact, Deal, Task, Viewport, Presets)
│
├── components/             # Επαναχρησιμοποιούμενα UI Panels
│   ├── mod.rs
│   ├── top_bar.rs          # Command bar (Save, Load, Secure, Workspace name)
│   ├── mode_bar.rs         # Tabs εναλλαγής λειτουργιών (Design, Play, Pipeline κ.α.)
│   └── device_bar.rs       # Toolbar επιλογής οθόνης (Desktop, Tablet, Phone)
│
├── views/                  # Αυτόνομα Views ανά λειτουργία
│   ├── mod.rs
│   ├── contacts.rs         # Contacts list, timeline σημειώσεων, inline edit
│   ├── pipeline.rs         # Kanban board 7 σταδίων, drag deals, auto follow-up tasks
│   ├── tasks.rs            # Tasks dashboard (Overdue, Today, Upcoming) & φίλτρα
│   ├── studio.rs           # Vector graphic tool (layers, brush, shapes, text)
│   ├── designer.rs         # Infinite canvas, 8-point handles, snapping, page tree
│   ├── play.rs             # Runtime runner (εκτέλεση κουμπιών & εγγραφή σε DB)
│   ├── flow_builder.rs     # Visual node graph (Triggers, Actions, SQLite saves)
│   ├── flow_legacy.rs      # Απλό visual flow
│   └── data_viewer.rs      # Επισκόπηση & επεξεργασία raw SQLite records
│
├── scene.rs                # Hierarchical Scene Graph & Serialization
├── renderer.rs             # Direct egui painter rendering engine
├── inspector.rs            # Properties panel για επιλεγμένα widgets
├── storage.rs              # Encrypted (.crmb) & JSON persistence
└── db.rs                   # Τοπική διεπαφή SQLite records
```

---

## 3. Φάσεις Επιχειρηματικής Ανάπτυξης (Phased Milestones)

| Φάση | Χρονικός Ορίζοντας | Τεχνικοί Στόχοι | Επιχειρηματικοί Στόχοι |
| :--- | :--- | :--- | :--- |
| **Phase 1: Foundation (Τρέχουσα)** | Q3 2026 | Data-bound widgets (`DataTable`), Undo/Redo stack, clean modular UI. | Closed Technical Beta (5–10 φιλικές επιχειρήσεις/testers). |
| **Phase 2: Revenue Engine** | Q4 2026 | Εξαγωγή προσφορών σε PDF, CSV/Excel εισαγωγή επαφών, Windows Installer (.msi). | Εμπορικό λανσάρισμα σε SOHO/SMBs ($29/mo ή €350 lifetime). |
| **Phase 3: Collaboration & Scale** | 2027 | Προαιρετικό cloud sync για ομάδες (PostgreSQL), Rhai automation scripting. | Team Plan ($49/seat/mo), Plugin marketplace. |

---

## 4. Στρατηγικές Ερωτήσεις προς τον Founder (Strategic Questions)

Για να χαράξουμε τα επόμενα βήματα με ακρίβεια:

1. **Ενοποίηση Δεδομένων (Data Unification)**:
   - Θέλουμε στο επόμενο βήμα τα Contacts και Deals να φύγουν από το JSON blob του project και να αποκτήσουν δικούς τους αυτόνομους πίνακες SQLite (`contacts`, `deals`), ώστε τα widgets του Designer να διαβάζουν απευθείας από αυτούς;
2. **Προφίλ Πρώτων Χρηστών Beta (Pilot Cohort)**:
   - Ποιο είναι το ιδανικό προφίλ των πρώτων 5–10 επιχειρήσεων για το κλειστό beta; (π.χ. freelancers, μικρά τεχνικά γραφεία, B2B agencies;)
3. **Packaging & Distribution**:
   - Προτιμάται η διανομή ως φορητό αυτόνομο `.exe` (zero-install) ή με πλήρη Windows Setup installer;
