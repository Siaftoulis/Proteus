//! Proteus Core — Single-Member IKE (Μονοπρόσωπη Ι.Κ.Ε.) Legal Charter & Filing Engine.
//! Designed under Greek Law 4072/2012 and General Commercial Registry (Γ.Ε.ΜΗ. / e-ΥΜΣ) standards.
//! Provides statutory articles compilation, KAD validation, and corporate registry persistence.

use crate::tax_registry::validate_greek_afm_checksum;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

fn current_epoch_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct KadEntry {
    pub kad_code: String,
    pub description: String,
    pub is_primary: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CompanyAdministrator {
    pub full_name: String,
    pub father_name: String,
    pub afm: String,
    pub amka: String,
    pub id_card_number: String,
    pub residential_address: String,
    pub email: String,
    pub phone: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CorporateCharter {
    pub company_id: String,
    pub company_name: String,
    pub distinctive_title: String,
    pub seat_municipality: String,
    pub seat_address: String,
    pub capital_eur: f64,
    pub share_nominal_value_eur: f64,
    pub total_shares: u32,
    pub duration_years: u32,
    pub fiscal_year_end_month: u8,
    pub fiscal_year_end_day: u8,
    pub kad_codes: Vec<KadEntry>,
    pub administrator: CompanyAdministrator,
    pub created_at_epoch: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CorporateFilingRecord {
    pub filing_id: String,
    pub company_id: String,
    pub filing_kind: String,
    pub reference_number: String,
    pub timestamp_epoch: u64,
    pub notes: Option<String>,
}

impl CorporateCharter {
    pub fn new_single_member_ike(
        company_id: &str,
        company_name: &str,
        distinctive_title: &str,
        seat_municipality: &str,
        seat_address: &str,
        administrator: CompanyAdministrator,
        kad_codes: Vec<KadEntry>,
    ) -> Result<Self, String> {
        let charter = Self {
            company_id: company_id.to_string(),
            company_name: company_name.trim().to_string(),
            distinctive_title: distinctive_title.trim().to_string(),
            seat_municipality: seat_municipality.trim().to_string(),
            seat_address: seat_address.trim().to_string(),
            capital_eur: 1.0,
            share_nominal_value_eur: 1.0,
            total_shares: 1,
            duration_years: 50,
            fiscal_year_end_month: 12,
            fiscal_year_end_day: 31,
            kad_codes,
            administrator,
            created_at_epoch: current_epoch_secs(),
        };

        charter.validate()?;
        Ok(charter)
    }

    pub fn validate(&self) -> Result<(), String> {
        let upper_name = self.company_name.to_uppercase();
        if !upper_name.contains("ΜΟΝΟΠΡΟΣΩΠΗ ΙΔΙΩΤΙΚΗ ΚΕΦΑΛΑΙΟΥΧΙΚΗ ΕΤΑΙΡΕΙΑ")
            && !upper_name.contains("ΜΟΝΟΠΡΟΣΩΠΗ Ι.Κ.Ε.")
            && !upper_name.contains("Μ. Ι.Κ.Ε.")
        {
            return Err("Η επωνυμία πρέπει να περιέχει τον εταιρικό τύπο 'ΜΟΝΟΠΡΟΣΩΠΗ Ι.Κ.Ε.'".into());
        }

        if !validate_greek_afm_checksum(&self.administrator.afm) {
            return Err(format!("Μη έγκυρο ΑΦΜ διαχειριστή: {}", self.administrator.afm));
        }

        if self.capital_eur < 1.0 {
            return Err("Το ελάχιστο εταιρικό κεφάλαιο είναι 1.00€ βάσει του Ν. 4072/2012".into());
        }

        let computed_capital = (self.total_shares as f64) * self.share_nominal_value_eur;
        if (computed_capital - self.capital_eur).abs() > 0.001 {
            return Err("Το γινόμενο μεριδίων επί την ονομαστική αξία δεν ισούται με το κεφάλαιο".into());
        }

        if self.kad_codes.is_empty() {
            return Err("Απαιτείται τουλάχιστον ένας ΚΑΔ δραστηριότητας".into());
        }

        let primary_count = self.kad_codes.iter().filter(|k| k.is_primary).count();
        if primary_count != 1 {
            return Err("Απαιτείται ακριβώς ένας κύριος ΚΑΔ δραστηριότητας".into());
        }

        Ok(())
    }

    pub fn generate_articles_text(&self) -> String {
        let mut kad_text = String::new();
        for k in &self.kad_codes {
            let role = if k.is_primary { " (Κύριος)" } else { "" };
            kad_text.push_str(&format!("  - {}: {}{}\n", k.kad_code, k.description, role));
        }

        format!(
            r#"ΚΑΤΑΣΤΑΤΙΚΟ ΣΥΣΤΑΣΗΣ ΜΟΝΟΠΡΟΣΩΠΗΣ Ι.Κ.Ε.
(Σύμφωνα με τον Ν. 4072/2012 και το Πρότυπο Καταστατικό e-ΥΜΣ)

Στην πόλη {} σήμερα, ο κάτωθι υπογεγραμμένος:
{} του {}, με ΑΦΜ {} και ΑΜΚΑ {}, κάτοικος {}, κάτοχος του ΑΔΤ {}, email: {}, τηλ: {},
συνιστά Μονοπρόσωπη Ιδιωτική Κεφαλαιουχική Εταιρεία με τους ακόλουθους όρους:

ΑΡΘΡΟ 1: ΕΠΩΝΥΜΙΑ & ΕΤΑΙΡΙΚΟΣ ΤΥΠΟΣ
Η επωνυμία της εταιρείας είναι: "{}"
Ο διακριτικός τίτλος είναι: "{}"

ΑΡΘΡΟ 2: ΕΔΡΑ
Έδρα της εταιρείας ορίζεται ο {}: {}

ΑΡΘΡΟ 3: ΣΚΟΠΟΣ ΤΗΣ ΕΤΑΙΡΕΙΑΣ
Σκοπός της εταιρείας είναι η άσκηση των ακόλουθων δραστηριοτήτων:
{}
ΑΡΘΡΟ 4: ΔΙΑΡΚΕΙΑ
Η διάρκεια της εταιρείας ορίζεται σε {} έτη από την καταχώρισή της στο Γ.Ε.ΜΗ.

ΑΡΘΡΟ 5: ΕΤΑΙΡΙΚΟ ΚΕΦΑΛΑΙΟ & ΜΕΡΙΔΙΑ
Το εταιρικό κεφάλαιο ορίζεται σε {:.2} ευρώ, διαιρούμενο σε {} εταιρικά μερίδια ονομαστικής αξίας {:.2} ευρώ έκαστο.

ΑΡΘΡΟ 6: ΕΙΣΦΟΡΕΣ ΕΤΑΙΡΟΥ
Το σύνολο των εταιρικών μεριδίων καλύπτεται εξ ολοκλήρου από κεφαλαιακές εισφορές σε μετρητά από τον μοναδικό εταίρο.

ΑΡΘΡΟ 7: ΔΙΑΧΕΙΡΙΣΗ & ΕΚΠΡΟΣΩΠΗΣΗ
Διαχειριστής της εταιρείας για αόριστη διάρκεια ορίζεται ο μοναδικός εταίρος {} του {}.
Ο Διαχειριστής εκπροσωπεί και δεσμεύει την εταιρεία έναντι παντός τρίτου, του Δημοσίου, Τραπεζών και Δικαστηρίων.

ΑΡΘΡΟ 8: ΕΤΑΙΡΙΚΗ ΧΡΗΣΗ
Η εταιρική χρήση λήγει την {}/{} εκάστου έτους.

ΑΡΘΡΟ 9: ΤΕΛΙΚΕΣ ΔΙΑΤΑΞΕΙΣ & ΚΑΤΑΧΩΡΙΣΗ ΣΤΟ Γ.Ε.ΜΗ.
Για όσα θέματα δεν ρυθμίζονται από το παρόν εφαρμόζονται οι διατάξεις του Ν. 4072/2012."#,
            self.seat_municipality,
            self.administrator.full_name,
            self.administrator.father_name,
            self.administrator.afm,
            self.administrator.amka,
            self.administrator.residential_address,
            self.administrator.id_card_number,
            self.administrator.email,
            self.administrator.phone,
            self.company_name,
            self.distinctive_title,
            self.seat_municipality,
            self.seat_address,
            kad_text,
            self.duration_years,
            self.capital_eur,
            self.total_shares,
            self.share_nominal_value_eur,
            self.administrator.full_name,
            self.administrator.father_name,
            self.fiscal_year_end_day,
            self.fiscal_year_end_month,
        )
    }
}

pub fn init_corporate_schema(conn: &Connection) -> Result<(), rusqlite::Error> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS corporate_charters (
            company_id TEXT PRIMARY KEY,
            company_name TEXT NOT NULL,
            distinctive_title TEXT NOT NULL,
            seat_municipality TEXT NOT NULL,
            seat_address TEXT NOT NULL,
            capital_eur REAL NOT NULL,
            admin_afm TEXT NOT NULL,
            admin_name TEXT NOT NULL,
            payload_json TEXT NOT NULL,
            created_at_epoch INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS corporate_filings (
            filing_id TEXT PRIMARY KEY,
            company_id TEXT NOT NULL,
            filing_kind TEXT NOT NULL,
            reference_number TEXT NOT NULL,
            timestamp_epoch INTEGER NOT NULL,
            notes TEXT
        );

        CREATE INDEX IF NOT EXISTS idx_corporate_filings_company ON corporate_filings(company_id);
        "#,
    )
}

pub fn save_corporate_charter(
    conn: &Connection,
    charter: &CorporateCharter,
) -> Result<(), rusqlite::Error> {
    let payload = serde_json::to_string(charter).unwrap_or_else(|_| "{}".into());
    conn.execute(
        r#"
        INSERT INTO corporate_charters (
            company_id, company_name, distinctive_title, seat_municipality, seat_address,
            capital_eur, admin_afm, admin_name, payload_json, created_at_epoch
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
        ON CONFLICT(company_id) DO UPDATE SET
            company_name = excluded.company_name,
            distinctive_title = excluded.distinctive_title,
            seat_municipality = excluded.seat_municipality,
            seat_address = excluded.seat_address,
            capital_eur = excluded.capital_eur,
            admin_afm = excluded.admin_afm,
            admin_name = excluded.admin_name,
            payload_json = excluded.payload_json
        "#,
        params![
            charter.company_id,
            charter.company_name,
            charter.distinctive_title,
            charter.seat_municipality,
            charter.seat_address,
            charter.capital_eur,
            charter.administrator.afm,
            charter.administrator.full_name,
            payload,
            charter.created_at_epoch as i64,
        ],
    )?;
    Ok(())
}

pub fn load_corporate_charter(
    conn: &Connection,
    company_id: &str,
) -> Result<Option<CorporateCharter>, rusqlite::Error> {
    let mut stmt = conn.prepare("SELECT payload_json FROM corporate_charters WHERE company_id = ?1")?;
    let mut rows = stmt.query(params![company_id])?;
    if let Some(row) = rows.next()? {
        let json: String = row.get(0)?;
        if let Ok(charter) = serde_json::from_str::<CorporateCharter>(&json) {
            return Ok(Some(charter));
        }
    }
    Ok(None)
}

pub fn record_corporate_filing(
    conn: &Connection,
    filing_id: &str,
    company_id: &str,
    filing_kind: &str,
    reference_number: &str,
    notes: Option<&str>,
) -> Result<(), rusqlite::Error> {
    let now = current_epoch_secs();
    conn.execute(
        r#"
        INSERT INTO corporate_filings (filing_id, company_id, filing_kind, reference_number, timestamp_epoch, notes)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6)
        "#,
        params![filing_id, company_id, filing_kind, reference_number, now as i64, notes],
    )?;
    Ok(())
}

pub fn list_corporate_filings(
    conn: &Connection,
    company_id: &str,
) -> Result<Vec<CorporateFilingRecord>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT filing_id, company_id, filing_kind, reference_number, timestamp_epoch, notes
         FROM corporate_filings WHERE company_id = ?1 ORDER BY timestamp_epoch DESC",
    )?;
    let rows = stmt.query_map(params![company_id], |r| {
        let timestamp_raw: i64 = r.get(4)?;
        Ok(CorporateFilingRecord {
            filing_id: r.get(0)?,
            company_id: r.get(1)?,
            filing_kind: r.get(2)?,
            reference_number: r.get(3)?,
            timestamp_epoch: timestamp_raw as u64,
            notes: r.get(5)?,
        })
    })?;

    let mut res = Vec::new();
    for row in rows {
        res.push(row?);
    }
    Ok(res)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_test_admin() -> CompanyAdministrator {
        CompanyAdministrator {
            full_name: "Νικόλαος Παπαδόπουλος".into(),
            father_name: "Ιωάννης".into(),
            afm: "094259216".into(), // Valid Greek AFM modulo 11
            amka: "15088501234".into(),
            id_card_number: "ΑΝ123456".into(),
            residential_address: "Σόλωνος 12, 10673 Αθήνα".into(),
            email: "admin@proteus-bos.gr".into(),
            phone: "+302101234567".into(),
        }
    }

    fn sample_kads() -> Vec<KadEntry> {
        vec![
            KadEntry {
                kad_code: "62.01.11".into(),
                description: "Παραγωγή πρωτοτύπων λογισμικού εφαρμογών".into(),
                is_primary: true,
            },
            KadEntry {
                kad_code: "62.02.10".into(),
                description: "Υπηρεσίες παροχής συμβουλών για θέματα συστημάτων πληροφορικής".into(),
                is_primary: false,
            },
        ]
    }

    #[test]
    fn test_valid_ike_charter_creation_and_articles() {
        let admin = valid_test_admin();
        let kads = sample_kads();
        let charter = CorporateCharter::new_single_member_ike(
            "COMP-01",
            "PROTEUS SOVEREIGN SOFTWARE ΜΟΝΟΠΡΟΣΩΠΗ ΙΔΙΩΤΙΚΗ ΚΕΦΑΛΑΙΟΥΧΙΚΗ ΕΤΑΙΡΕΙΑ",
            "PROTEUS BOS ΜΟΝΟΠΡΟΣΩΠΗ Ι.Κ.Ε.",
            "Δήμος Αθηναίων",
            "Σταδίου 24, 10564 Αθήνα",
            admin,
            kads,
        )
        .expect("Charter creation should succeed");

        assert_eq!(charter.capital_eur, 1.0);
        assert_eq!(charter.total_shares, 1);

        let articles = charter.generate_articles_text();
        assert!(articles.contains("ΚΑΤΑΣΤΑΤΙΚΟ ΣΥΣΤΑΣΗΣ ΜΟΝΟΠΡΟΣΩΠΗΣ Ι.Κ.Ε."));
        assert!(articles.contains("Ν. 4072/2012"));
        assert!(articles.contains("62.01.11"));
        assert!(articles.contains("Νικόλαος Παπαδόπουλος"));
        assert!(articles.contains("094259216"));
    }

    #[test]
    fn test_invalid_afm_rejection() {
        let mut admin = valid_test_admin();
        admin.afm = "123456789".into(); // Invalid checksum
        let kads = sample_kads();

        let err = CorporateCharter::new_single_member_ike(
            "COMP-02",
            "PROTEUS ΜΟΝΟΠΡΟΣΩΠΗ Ι.Κ.Ε.",
            "PROTEUS Μ. Ι.Κ.Ε.",
            "Αθήνα",
            "Ερμού 1",
            admin,
            kads,
        );
        assert!(err.is_err());
        assert!(err.unwrap_err().contains("Μη έγκυρο ΑΦΜ"));
    }

    #[test]
    fn test_missing_primary_kad_rejection() {
        let admin = valid_test_admin();
        let kads = vec![KadEntry {
            kad_code: "62.01".into(),
            description: "Λογισμικό".into(),
            is_primary: false, // No primary!
        }];

        let err = CorporateCharter::new_single_member_ike(
            "COMP-03",
            "PROTEUS ΜΟΝΟΠΡΟΣΩΠΗ Ι.Κ.Ε.",
            "PROTEUS Μ. Ι.Κ.Ε.",
            "Αθήνα",
            "Ερμού 1",
            admin,
            kads,
        );
        assert!(err.is_err());
        assert!(err.unwrap_err().contains("κύριος ΚΑΔ"));
    }

    #[test]
    fn test_sqlite_charter_persistence_and_filings() {
        let conn = Connection::open_in_memory().unwrap();
        init_corporate_schema(&conn).unwrap();

        let admin = valid_test_admin();
        let kads = sample_kads();
        let charter = CorporateCharter::new_single_member_ike(
            "COMP-GR-001",
            "PROTEUS BOS ΜΟΝΟΠΡΟΣΩΠΗ Ι.Κ.Ε.",
            "PROTEUS BOS",
            "Δήμος Αθηναίων",
            "Πανεπιστημίου 10",
            admin,
            kads,
        )
        .unwrap();

        save_corporate_charter(&conn, &charter).unwrap();
        let loaded = load_corporate_charter(&conn, "COMP-GR-001")
            .unwrap()
            .expect("Charter must exist");
        assert_eq!(loaded.company_name, charter.company_name);
        assert_eq!(loaded.administrator.afm, "094259216");

        // Record filings
        record_corporate_filing(
            &conn,
            "FIL-01",
            "COMP-GR-001",
            "GEMI_REGISTRATION",
            "169999000000",
            Some("Εγγραφή στο Γενικό Εμπορικό Μητρώο e-ΥΜΣ"),
        )
        .unwrap();

        let filings = list_corporate_filings(&conn, "COMP-GR-001").unwrap();
        assert_eq!(filings.len(), 1);
        assert_eq!(filings[0].reference_number, "169999000000");
    }
}
