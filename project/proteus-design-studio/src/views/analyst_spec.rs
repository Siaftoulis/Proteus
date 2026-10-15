//! Sovereign Business Analysis & Client Discovery Engine (`views::analyst_spec`).
//! Compiles client interview notes and requirements into bespoke Markdown specifications.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BusinessSector {
    Retail,
    FieldService,
    Hospitality,
    Wholesale,
    Healthcare,
    ECommerce,
    Manufacturing,
}

impl BusinessSector {
    pub fn label_el(&self) -> &'static str {
        match self {
            Self::Retail => "🏪 Λιανική & Εμπορικό Κατάστημα",
            Self::FieldService => "🔧 Τεχνικό Συνεργείο & Επιτόπιο Service",
            Self::Hospitality => "☕ Εστίαση, Café & Φιλοξενία",
            Self::Wholesale => "📦 Χονδρικό Εμπόριο & Διανομές",
            Self::Healthcare => "🩺 Ιατρείο & Υπηρεσίες Υγείας",
            Self::ECommerce => "🌐 Ηλεκτρονικό Κατάστημα (E-shop)",
            Self::Manufacturing => "🏭 Βιοτεχνία & Παραγωγή",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UiStylePreference {
    MinimalistClean,
    HighContrastDark,
    TouchTabletOptimized,
    DenseDesktop,
}

impl UiStylePreference {
    pub fn label_el(&self) -> &'static str {
        match self {
            Self::MinimalistClean => "✨ Minimalist & Καθαρό (Average Joe)",
            Self::HighContrastDark => "🌙 High-Contrast Dark Mode",
            Self::TouchTabletOptimized => "📱 Touch Tablet / Μεγάλα Πλήκτρα",
            Self::DenseDesktop => "🖥 High-Density ERP Desktop",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnalystSubTab {
    ClientDiscovery,
    MarkdownSpec,
    SchemaInferer,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DiscoveryInterview {
    pub client_business_name: String,
    pub client_tax_id: String,
    pub client_contact_person: String,
    pub client_contact_email: String,
    pub client_industry: BusinessSector,
    pub ui_style: UiStylePreference,
    pub module_crm: bool,
    pub module_pos: bool,
    pub module_wms: bool,
    pub module_field_service: bool,
    pub module_cardex: bool,
    pub module_webstore: bool,
    pub role_cashier: bool,
    pub role_technician: bool,
    pub role_manager: bool,
    pub role_field_agent: bool,
    pub interview_notes: String,
    pub estimated_users: u32,
    pub estimated_budget_eur: f64,
}

impl Default for DiscoveryInterview {
    fn default() -> Self {
        Self {
            client_business_name: "TechFix Solutions Ο.Ε.".to_string(),
            client_tax_id: "099887766".to_string(),
            client_contact_person: "Γιάννης Οικονόμου".to_string(),
            client_contact_email: "info@techfix.gr".to_string(),
            client_industry: BusinessSector::FieldService,
            ui_style: UiStylePreference::MinimalistClean,
            module_crm: true,
            module_pos: true,
            module_wms: true,
            module_field_service: true,
            module_cardex: true,
            module_webstore: false,
            role_cashier: true,
            role_technician: true,
            role_manager: true,
            role_field_agent: false,
            interview_notes: "Ο πελάτης θέλει ένα απλό σύστημα διαχείρισης επισκευών και λιανικής. \
                              Οι τεχνικοί πρέπει να βλέπουν τα εισερχόμενα δελτία επισκευής στο κινητό, \
                              να καταγράφουν τα ανταλλακτικά από το van και ο πελάτης να υπογράφει στην οθόνη. \
                              Στο κατάστημα θέλουν γρήγορο ταμείο POS με έκδοση αποδείξεων myDATA και διασύνδεση EFT/POS."
                .to_string(),
            estimated_users: 5,
            estimated_budget_eur: 3500.0,
        }
    }
}

impl DiscoveryInterview {
    pub fn to_client_project_brief(&self) -> proteus_core::brief::ClientProjectBrief {
        let mut screens = Vec::new();
        if self.module_crm {
            screens.push("CRM & Deals".to_string());
        }
        if self.module_pos {
            screens.push("Point of Sale".to_string());
        }
        if self.module_wms {
            screens.push("WMS Inventory".to_string());
        }
        if self.module_field_service {
            screens.push("Field Service Bench".to_string());
        }
        if self.module_cardex {
            screens.push("Client Cardex".to_string());
        }
        if screens.is_empty() {
            screens.push("Commercial Dashboard".to_string());
        }

        let mut peripherals = Vec::new();
        if self.module_pos {
            peripherals.push("Thermal Receipt Printer".to_string());
            peripherals.push("EFT/POS Terminal".to_string());
            peripherals.push("Barcode Scanner".to_string());
        }
        if self.module_field_service {
            peripherals.push("Touch Tablet Signature".to_string());
        }

        let track = if self.module_webstore && self.module_pos {
            proteus_core::brief::BriefTrack::DualStack
        } else if self.module_webstore {
            proteus_core::brief::BriefTrack::PcdWeb
        } else {
            proteus_core::brief::BriefTrack::PcdApp
        };

        proteus_core::brief::ClientProjectBrief {
            brief_id: format!("BRIEF-{}", uuid::Uuid::new_v4().to_string().chars().take(8).collect::<String>()),
            business_name: if self.client_business_name.trim().is_empty() {
                "Bespoke Client App".to_string()
            } else {
                self.client_business_name.clone()
            },
            business_nature: self.client_industry.label_el().to_string(),
            daily_operations_desc: self.interview_notes.clone(),
            track,
            required_screens: screens,
            hardware_peripherals: peripherals,
            proposed_budget_eur: self.estimated_budget_eur,
            domain_name_requested: None,
            hosting_preference: "SelfHosted".to_string(),
            contact_email: self.client_contact_email.clone(),
            submitted_at: chrono::Utc::now().to_rfc3339(),
        }
    }
}

/// Compiles structured client interview responses into a comprehensive Markdown specification document.
pub fn compile_markdown_specification(interview: &DiscoveryInterview) -> String {
    let mut doc = String::with_capacity(4096);

    doc.push_str(&format!("# Τεχνική & Επιχειρησιακή Προδιαγραφή Έργου: {}\n\n", interview.client_business_name));
    doc.push_str("> **Έγγραφο Ανάλυσης Απαιτήσεων & Αρχιτεκτονικού Σχεδιασμού (Proteus Design Studio)**\n\n");
    doc.push_str(&format!("- **Επωνυμία Επιχείρησης**: {}\n", interview.client_business_name));
    doc.push_str(&format!("- **Α.Φ.Μ.**: {}\n", if interview.client_tax_id.is_empty() { "-" } else { &interview.client_tax_id }));
    doc.push_str(&format!("- **Υπεύθυνος Επικοινωνίας**: {}\n", interview.client_contact_person));
    doc.push_str(&format!("- **Email**: {}\n", interview.client_contact_email));
    doc.push_str(&format!("- **Κλάδος Δραστηριότητας**: {}\n", interview.client_industry.label_el()));
    doc.push_str(&format!("- **Επιθυμητό Στυλ Διεπαφής**: {}\n", interview.ui_style.label_el()));
    doc.push_str(&format!("- **Εκτιμώμενοι Χρήστες**: {}\n", interview.estimated_users));
    doc.push_str(&format!("- **Εκτιμώμενος Προϋπολογισμός**: €{:.2}\n\n", interview.estimated_budget_eur));

    doc.push_str("## 1. Σύνοψη & Σκοπός Έργου\n\n");
    doc.push_str(&interview.interview_notes);
    doc.push_str("\n\n---\n\n");

    doc.push_str("## 2. Απαιτούμενα Αυτόνομα Υποσυστήματα (Proteus Sovereign Modules)\n\n");
    if interview.module_crm {
        doc.push_str("- **[x] CRM & Διαχείριση Πελατών**: Διαχείριση επαφών, ιστορικό αλληλεπιδράσεων και αγωγός ευκαιριών (Kanban Deals).\n");
    }
    if interview.module_pos {
        doc.push_str("- **[x] POS Ταμείο & myDATA**: Ταχύτατη έκδοση αποδείξεων λιανικής, άμεση διασύνδεση με ΑΑΔΕ (Α.1138/2020) και τερματικό EFT/POS (Α.1155/2023).\n");
    }
    if interview.module_wms {
        doc.push_str("- **[x] WMS & Διαχείριση Αποθήκης**: Παρακολούθηση αποθεμάτων, ανάγνωση γραμμωτού κώδικα (Barcode/QR), παρτίδες και δελτία αποστολής.\n");
    }
    if interview.module_field_service {
        doc.push_str("- **[x] Επιτόπιο Service & Mobile Τεχνικός**: Εντολές τεχνικού, διαγνωστικές λίστες, καταγραφή ανταλλακτικών van stock και ψηφιακή υπογραφή πελάτη.\n");
    }
    if interview.module_cardex {
        doc.push_str("- **[x] Οικονομική Καρτέλα Πελάτη (Cardex)**: Χρεώσεις, πιστώσεις, τρέχοντα υπόλοιπα και 5-tier ηλικίωση απαιτήσεων (0-30, 31-60, 61-90, 91-120, 120+ ημέρες).\n");
    }
    if interview.module_webstore {
        doc.push_str("- **[x] WebStore & B2B Portal**: Ηλεκτρονικός κατάλογος προϊόντων, B2B κρατήσεις αποθέματος και αυτόματη ανάθεση παραγγελιών.\n");
    }
    doc.push_str("\n---\n\n");

    doc.push_str("## 3. Ρόλοι Χρηστών & Συσκευές Πρόσβασης\n\n");
    if interview.role_cashier {
        doc.push_str("- **Ταμίας / Εξυπηρέτηση**: Touch Screen POS με ελάχιστα κλικ και υποστήριξη οθόνης πελάτη.\n");
    }
    if interview.role_technician {
        doc.push_str("- **Τεχνικός Πεδίου**: Εφαρμογή κινητού/tablet offline-first με τοπική βάση SQLite και συγχρονισμό mesh.\n");
    }
    if interview.role_manager {
        doc.push_str("- **Διαχειριστής / Ιδιοκτήτης**: Πλήρης εποπτεία πωλήσεων, οικονομικών αναφορών και διαχείρισης προσωπικού (Εργάνη ΙΙ).\n");
    }
    if interview.role_field_agent {
        doc.push_str("- **Πωλητής / Εξωτερικός Συνεργάτης**: Φορητή παραγγελιοληψία και έλεγχος πιστωτικού ορίου πελάτη σε πραγματικό χρόνο.\n");
    }
    doc.push_str("\n---\n\n");

    doc.push_str("## 4. Στρατηγική Mockup & Οπτικών Blocks Canvas\n\n");
    doc.push_str("Για την άμεση κάλυψη των αναγκών χωρίς περίπλοκες καθυστερήσεις, δημιουργούνται αυτόματα τα παρακάτω canvas blocks:\n\n");
    if interview.module_crm {
        doc.push_str("1. **Block CRM Agogos**: Στήλες σταδίων (Νέα Επαφή -> Προσφορά -> Σε Εξέλιξη -> Ολοκληρώθηκε) με κάρτες επαφής.\n");
    }
    if interview.module_pos {
        doc.push_str("2. **Block Quick POS**: Μπάρα γρήγορων προϊόντων, οθόνη καλαθιού και κουμπιά άμεσης πληρωμής (Μετρητά / POS).\n");
    }
    if interview.module_field_service {
        doc.push_str("3. **Block Field Service**: Διαγνωστικό checklist, καταγραφή ωρών εργασίας, αναλώσιμα και touch πλαίσιο υπογραφής.\n");
    }
    if interview.module_wms {
        doc.push_str("4. **Block WMS Ledger**: Πίνακας καταγραφής αποθέματος, πεδίο άμεσης σάρωσης barcode και ένδειξη ελάχιστου ορίου.\n");
    }
    doc.push_str("\n---\n\n");

    doc.push_str("## 5. Προτεινόμενες Σχεσιακές Οντότητες Βάσης Δεδομένων\n\n");
    doc.push_str("Η αρχιτεκτονική βασίζεται σε κυρίαρχη τοπική βάση SQLite (`store.db`) με μηδενικό κόστος συνδρομών:\n\n");
    doc.push_str("- `entities` (id, name, tax_id, phone, email, credit_limit, created_at)\n");
    if interview.module_pos {
        doc.push_str("- `sales_receipts` (id, receipt_number, total_amount, vat_amount, payment_method, mydata_mark, created_at)\n");
    }
    if interview.module_field_service {
        doc.push_str("- `service_tickets` (id, customer_id, equipment_desc, status, labor_hours, signature_blob, created_at)\n");
    }
    if interview.module_wms {
        doc.push_str("- `inventory_items` (id, sku, barcode, description, stock_qty, unit_cost, retail_price)\n");
    }
    doc.push_str("\n---\n\n");

    doc.push_str("## 6. Επόμενα Βήματα Υλοποίησης για Σχεδιαστή & Προγραμματιστές\n\n");
    doc.push_str("1. **UI Designer**: Πάτημα του κουμπιού `⚡ Δημιουργία Canvas Mockup Blocks` για άμεση δημιουργία της διάταξης στο Studio Canvas.\n");
    doc.push_str("2. **Data Structurer**: Αυτόματη μεταφορά των σχεσιακών οντοτήτων στο module Schemas για δημιουργία DDL.\n");
    doc.push_str("3. **Business Engine Expert**: Σύνδεση των οπτικών triggers με τους αντίστοιχους κανόνες (Tripartite Pipeline).\n");

    doc
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compile_markdown_specification() {
        let interview = DiscoveryInterview::default();
        let md = compile_markdown_specification(&interview);

        assert!(md.contains("TechFix Solutions Ο.Ε."));
        assert!(md.contains("Τεχνική & Επιχειρησιακή Προδιαγραφή Έργου"));
        assert!(md.contains("CRM & Διαχείριση Πελατών"));
        assert!(md.contains("POS Ταμείο & myDATA"));
        assert!(md.contains("Επιτόπιο Service & Mobile Τεχνικός"));
        assert!(md.contains("Block Field Service"));
        assert!(md.contains("service_tickets"));
    }
}
