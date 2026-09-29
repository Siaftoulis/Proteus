//! Proteus Universal Reconciler & Conflict Resolution Engine.
//! Performs automated entry-by-entry schema reconciliation and federation between disparate databases.
//! Designed from first principles for seamless B2B database harmony.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::smlm::{ColumnProfile, SemanticIntent, SmlmEngine};

/// Quality level of an entry-by-entry match across disparate databases.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum MatchQuality {
    ExactPrimaryNaturalKey, // E.g. Exact AFM, EAN-13, IMO, or UUID
    CompositeNaturalKey,    // E.g. Phone + Name
    FuzzyMatch,             // E.g. Similar customer name or address
    Unmatched,
}

/// A reconciled row pairing between source database A and target database B.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchedRecordPair {
    pub source_id: String,
    pub target_id: Option<String>,
    pub match_quality: MatchQuality,
    pub match_confidence: f64,
    pub matched_on: String,
    pub field_discrepancies: Vec<FieldDiscrepancy>,
}

/// Discrepancy details when matching fields differ between databases.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FieldDiscrepancy {
    pub field_name: String,
    pub source_value: String,
    pub target_value: String,
    pub resolution_winner: String,
    pub resolution_reason: String,
}

/// Action to be taken on an individual record during federation synchronization.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FederationAction {
    InsertNew,
    UpdateExisting { target_id: String },
    SkipIdentical,
    ManualConflictReview { reason: String },
}

/// Actionable federation sync plan produced by entry-by-entry reconciliation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederationSyncPlan {
    pub matched_count: usize,
    pub inserts_count: usize,
    pub updates_count: usize,
    pub identical_count: usize,
    pub conflicts_count: usize,
    pub harmony_score: f64,
    pub pairs: Vec<MatchedRecordPair>,
}

/// Entry-by-entry database reconciliation engine.
pub struct UniversalReconciler;

impl UniversalReconciler {
    /// Identifies the primary natural key column index from semantic profiles.
    pub fn find_natural_key_index(profiles: &[ColumnProfile]) -> Option<(usize, SemanticIntent)> {
        // Priority 1: Unique tax identifiers or barcodes
        for (idx, p) in profiles.iter().enumerate() {
            if matches!(p.inferred_intent, SemanticIntent::TaxId | SemanticIntent::BarcodeOrSku | SemanticIntent::MaritimeIdentity | SemanticIntent::VehicleIdentity) {
                return Some((idx, p.inferred_intent));
            }
        }
        // Priority 2: Phone or Email
        for (idx, p) in profiles.iter().enumerate() {
            if matches!(p.inferred_intent, SemanticIntent::PhoneNumber | SemanticIntent::EmailAddress) {
                return Some((idx, p.inferred_intent));
            }
        }
        None
    }

    /// Reconciles two tables entry-by-entry using semantic profiling and natural keys.
    pub fn reconcile_tables(
        source_headers: &[String],
        source_rows: &[Vec<String>],
        target_headers: &[String],
        target_rows: &[Vec<String>],
    ) -> FederationSyncPlan {
        let source_profiles = SmlmEngine::profile_table(source_headers, source_rows);
        let target_profiles = SmlmEngine::profile_table(target_headers, target_rows);

        let source_key_info = Self::find_natural_key_index(&source_profiles);
        let target_key_info = Self::find_natural_key_index(&target_profiles);

        // Build index of target rows by natural key value
        let mut target_index: HashMap<String, (usize, &Vec<String>)> = HashMap::new();
        if let Some((target_key_col, _)) = target_key_info {
            for (idx, row) in target_rows.iter().enumerate() {
                if let Some(val) = row.get(target_key_col) {
                    let cleaned = val.trim().to_uppercase();
                    if !cleaned.is_empty() {
                        target_index.insert(cleaned, (idx, row));
                    }
                }
            }
        }

        let mut pairs = Vec::with_capacity(source_rows.len());
        let mut inserts = 0;
        let mut updates = 0;
        let mut identical = 0;
        let conflicts = 0;

        for (src_idx, src_row) in source_rows.iter().enumerate() {
            let src_id = src_row.first().cloned().unwrap_or_else(|| format!("src_{}", src_idx));

            let mut matched_target: Option<(usize, &Vec<String>)> = None;
            let mut match_quality = MatchQuality::Unmatched;
            let mut match_confidence = 0.0;
            let mut matched_on = String::from("none");

            if let Some((src_key_col, intent)) = source_key_info {
                if let Some(src_val) = src_row.get(src_key_col) {
                    let cleaned = src_val.trim().to_uppercase();
                    if let Some(&tgt) = target_index.get(&cleaned) {
                        matched_target = Some(tgt);
                        match_quality = MatchQuality::ExactPrimaryNaturalKey;
                        match_confidence = 0.99;
                        matched_on = format!("{:?} ({})", intent, cleaned);
                    }
                }
            }

            if let Some((tgt_idx, tgt_row)) = matched_target {
                let tgt_id = tgt_row.first().cloned().unwrap_or_else(|| format!("tgt_{}", tgt_idx));
                let mut discrepancies = Vec::new();

                // Compare aligned business columns (skip internal record IDs)
                for (s_col, s_prof) in source_profiles.iter().enumerate() {
                    for (t_col, t_prof) in target_profiles.iter().enumerate() {
                        if s_prof.inferred_intent == t_prof.inferred_intent && s_prof.inferred_intent != SemanticIntent::RecordId {
                            let s_val = src_row.get(s_col).map(|s| s.trim()).unwrap_or("");
                            let t_val = tgt_row.get(t_col).map(|s| s.trim()).unwrap_or("");

                            if !s_val.is_empty() && !t_val.is_empty() && s_val != t_val {
                                // Default to Source Wins (newer sync payload)
                                discrepancies.push(FieldDiscrepancy {
                                    field_name: s_prof.column_name.clone(),
                                    source_value: s_val.to_string(),
                                    target_value: t_val.to_string(),
                                    resolution_winner: s_val.to_string(),
                                    resolution_reason: "Source update preferred over target".to_string(),
                                });
                            }
                        }
                    }
                }

                if discrepancies.is_empty() {
                    identical += 1;
                } else {
                    updates += 1;
                }

                pairs.push(MatchedRecordPair {
                    source_id: src_id,
                    target_id: Some(tgt_id),
                    match_quality,
                    match_confidence,
                    matched_on,
                    field_discrepancies: discrepancies,
                });
            } else {
                // New record to insert into target
                inserts += 1;
                pairs.push(MatchedRecordPair {
                    source_id: src_id,
                    target_id: None,
                    match_quality: MatchQuality::Unmatched,
                    match_confidence: 0.0,
                    matched_on: "unmatched_new_record".to_string(),
                    field_discrepancies: Vec::new(),
                });
            }
        }

        let total = source_rows.len().max(1);
        let harmony_score = ((identical + updates) as f64 / total as f64).min(1.0);

        FederationSyncPlan {
            matched_count: identical + updates,
            inserts_count: inserts,
            updates_count: updates,
            identical_count: identical,
            conflicts_count: conflicts,
            harmony_score,
            pairs,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reconciliation_exact_afm_match_and_update() {
        // Source table from Partner CRM (e.g. SoftOne)
        let source_headers = vec!["id".to_string(), "afm".to_string(), "eponimia".to_string(), "tilefono".to_string()];
        let source_rows = vec![
            vec!["1".to_string(), "094014201".to_string(), "ALFA NAFTILIKI A.E.".to_string(), "6981234567".to_string()],
            vec!["2".to_string(), "090000045".to_string(), "BETA TOOLS E.E.".to_string(), "2109876543".to_string()],
        ];

        // Target table in Proteus SQLite (Target has older phone for ALFA NAFTILIKI)
        let target_headers = vec!["record_id".to_string(), "tax_number".to_string(), "customer_name".to_string(), "phone_mobile".to_string()];
        let target_rows = vec![
            vec!["rec_101".to_string(), "094014201".to_string(), "ALFA NAFTILIKI A.E.".to_string(), "2100000000".to_string()],
        ];

        let plan = UniversalReconciler::reconcile_tables(
            &source_headers,
            &source_rows,
            &target_headers,
            &target_rows,
        );

        assert_eq!(plan.matched_count, 1);
        assert_eq!(plan.inserts_count, 1); // "BETA TOOLS" is new
        assert_eq!(plan.updates_count, 1); // "ALFA NAFTILIKI" has discrepancy on phone

        let alfa_pair = plan.pairs.iter().find(|p| p.source_id == "1").unwrap();
        assert_eq!(alfa_pair.target_id, Some("rec_101".to_string()));
        assert_eq!(alfa_pair.match_quality, MatchQuality::ExactPrimaryNaturalKey);
        assert!(!alfa_pair.field_discrepancies.is_empty());
    }

    #[test]
    fn test_reconciliation_identical_records() {
        let source_headers = vec!["id".to_string(), "afm".to_string(), "name".to_string()];
        let source_rows = vec![
            vec!["s1".to_string(), "094014201".to_string(), "ALPHA CORP".to_string()],
        ];

        let target_headers = vec!["id".to_string(), "vat".to_string(), "company".to_string()];
        let target_rows = vec![
            vec!["t1".to_string(), "094014201".to_string(), "ALPHA CORP".to_string()],
        ];

        let plan = UniversalReconciler::reconcile_tables(
            &source_headers,
            &source_rows,
            &target_headers,
            &target_rows,
        );

        assert_eq!(plan.identical_count, 1);
        assert_eq!(plan.updates_count, 0);
        assert_eq!(plan.inserts_count, 0);
        assert_eq!(plan.harmony_score, 1.0);
    }
}
