//! Multi-Vector Automated Template Linter & Certification Gate for `.pr` packages.
//! Enforces the 5-point quality standard from Master Problem Audit P19:
//! 1. Schema Consistency & Primary Key Validation
//! 2. No-Orphaned-Views Navigation Reachability
//! 3. Workflow DAG Cycle Detection (Zero Infinite Loops)
//! 4. Hardware ESC/POS & Column Width Bounds
//! 5. Additive-Only Migration Safety (Zero DROP statements)

use std::collections::{HashMap, HashSet};
use serde::{Deserialize, Serialize};

use crate::package::{PrFlowTrigger, PrPackage, PrSchemaBundle, PrViewLayout};

/// Severity tier of an identified template issue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LintSeverity {
    Error,   // Blocks export and marketplace publication
    Warning, // Advisory note, permits export with quality penalty
    Info,    // Informational recommendation
}

/// Category classification of quality checks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LintCategory {
    SchemaConsistency,
    NavigationReachability,
    DagCycle,
    HardwareEscPos,
    AdditiveSafety,
}

impl LintCategory {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::SchemaConsistency => "Schema Consistency",
            Self::NavigationReachability => "View Navigation & Reachability",
            Self::DagCycle => "DAG Workflow Topology",
            Self::HardwareEscPos => "ESC/POS Thermal Syntax",
            Self::AdditiveSafety => "Additive Migration Safety",
        }
    }
}

/// Specific flaw or defect detected during package linting.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LintIssue {
    pub category: LintCategory,
    pub severity: LintSeverity,
    pub code: String,
    pub target: String,
    pub message: String,
    pub suggested_fix: String,
}

/// Comprehensive pre-flight certification report for a `.pr` template bundle.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LintReport {
    pub is_valid: bool,
    pub quality_score: f64,
    pub errors: Vec<LintIssue>,
    pub warnings: Vec<LintIssue>,
    pub passed_checks: usize,
    pub total_checks: usize,
}

pub struct PackageLinter;

impl PackageLinter {
    /// Executes all 5 quality vectors against a package bundle.
    pub fn lint_package(pkg: &PrPackage) -> LintReport {
        let mut issues = Vec::new();
        let total_checks = 5;
        let mut passed_checks = 0;

        // Check 1: Schema Consistency
        let prev_len = issues.len();
        Self::check_schema_consistency(&pkg.schema, &mut issues);
        if issues.len() == prev_len {
            passed_checks += 1;
        }

        // Check 2: Additive Migration Safety
        let prev_len = issues.len();
        Self::check_additive_migration_safety(&pkg.schema.ddl_statements, &mut issues);
        if issues.len() == prev_len {
            passed_checks += 1;
        }

        // Check 3: View Navigation Reachability
        let prev_len = issues.len();
        Self::check_view_reachability(&pkg.views, &pkg.flows, &mut issues);
        if issues.len() == prev_len {
            passed_checks += 1;
        }

        // Check 4: Workflow DAG Cycle Detection
        let prev_len = issues.len();
        Self::check_dag_cycles(&pkg.flows, &mut issues);
        if issues.len() == prev_len {
            passed_checks += 1;
        }

        // Check 5: Hardware ESC/POS Validation
        let prev_len = issues.len();
        Self::check_hardware_escpos(&pkg.flows, &mut issues);
        if issues.len() == prev_len {
            passed_checks += 1;
        }

        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        for issue in issues {
            match issue.severity {
                LintSeverity::Error => errors.push(issue),
                LintSeverity::Warning | LintSeverity::Info => warnings.push(issue),
            }
        }

        let is_valid = errors.is_empty();

        // Calculate quality score (0.0 to 100.0)
        let error_penalty = (errors.len() as f64) * 25.0;
        let warning_penalty = (warnings.len() as f64) * 5.0;
        let raw_score = 100.0 - error_penalty - warning_penalty;
        let quality_score = raw_score.clamp(0.0, 100.0);

        LintReport {
            is_valid,
            quality_score,
            errors,
            warnings,
            passed_checks,
            total_checks,
        }
    }

    /// Vector 1: Ensures all tables define a primary key and valid columns.
    fn check_schema_consistency(schema: &PrSchemaBundle, issues: &mut Vec<LintIssue>) {
        for entity in &schema.entity_schemas {
            if entity.fields.is_empty() {
                issues.push(LintIssue {
                    category: LintCategory::SchemaConsistency,
                    severity: LintSeverity::Error,
                    code: "LINT_SCHEMA_EMPTY_ENTITY".to_string(),
                    target: entity.entity.clone(),
                    message: format!("Entity '{}' has no fields declared", entity.entity),
                    suggested_fix: "Add at least one field and a primary key to the entity".to_string(),
                });
                continue;
            }

            let has_pk = entity.fields.iter().any(|f| {
                f.name.eq_ignore_ascii_case("id")
                    || f.name.ends_with("_id")
                    || f.name.eq_ignore_ascii_case("uuid")
                    || f.name.eq_ignore_ascii_case("sku")
            });

            if !has_pk {
                issues.push(LintIssue {
                    category: LintCategory::SchemaConsistency,
                    severity: LintSeverity::Error,
                    code: "LINT_SCHEMA_NO_PK".to_string(),
                    target: entity.entity.clone(),
                    message: format!("Entity '{}' lacks an identifiable primary key column ('id', '*_id', 'uuid')", entity.entity),
                    suggested_fix: "Add an 'id TEXT PRIMARY KEY' field to ensure deterministic syncing".to_string(),
                });
            }
        }
    }

    /// Vector 2: Enforces strictly additive migrations and rejects DROP or destructive DDL.
    fn check_additive_migration_safety(ddl_statements: &[String], issues: &mut Vec<LintIssue>) {
        for (idx, sql) in ddl_statements.iter().enumerate() {
            let upper = sql.trim().to_uppercase();

            if upper.contains("DROP TABLE") || upper.contains("DROP COLUMN") || upper.contains("TRUNCATE") {
                issues.push(LintIssue {
                    category: LintCategory::AdditiveSafety,
                    severity: LintSeverity::Error,
                    code: "LINT_MIGRATION_DESTRUCTIVE_SQL".to_string(),
                    target: format!("DDL statement #{}", idx + 1),
                    message: format!("Destructive SQL statement detected: '{}'", sql),
                    suggested_fix: "Remove DROP/TRUNCATE statements. Proteus BOS strictly permits additive schema changes".to_string(),
                });
            }

            if upper.starts_with("ALTER TABLE") && upper.contains("ADD COLUMN") && !upper.contains("DEFAULT") && upper.contains("NOT NULL") {
                issues.push(LintIssue {
                    category: LintCategory::AdditiveSafety,
                    severity: LintSeverity::Error,
                    code: "LINT_MIGRATION_ADD_COLUMN_NO_DEFAULT".to_string(),
                    target: format!("DDL statement #{}", idx + 1),
                    message: "ALTER TABLE ADD COLUMN NOT NULL without DEFAULT clause will fail on non-empty tables".to_string(),
                    suggested_fix: "Provide a DEFAULT clause for NOT NULL columns or make the column nullable".to_string(),
                });
            }
        }
    }

    /// Vector 3: Checks that declared views are reachable from entry points or flows.
    fn check_view_reachability(views: &[PrViewLayout], flows: &[PrFlowTrigger], issues: &mut Vec<LintIssue>) {
        if views.is_empty() {
            return;
        }

        let mut referenced_views = HashSet::new();
        // First view is considered the entry / default root view
        if let Some(first) = views.first() {
            referenced_views.insert(first.view_id.clone());
        }

        for flow in flows {
            for v in views {
                if flow.config_json.contains(&v.view_id) || flow.action_type.contains(&v.view_id) {
                    referenced_views.insert(v.view_id.clone());
                }
            }
        }

        for v in views {
            if !referenced_views.contains(&v.view_id) {
                issues.push(LintIssue {
                    category: LintCategory::NavigationReachability,
                    severity: LintSeverity::Warning,
                    code: "LINT_ORPHANED_VIEW".to_string(),
                    target: v.view_id.clone(),
                    message: format!("View '{}' ({}) is orphaned with no navigation routes pointing to it", v.name, v.view_id),
                    suggested_fix: "Add an action button or flow transition navigating to this view".to_string(),
                });
            }
        }
    }

    /// Vector 4: Detects circular workflow cycles in reactive flow triggers.
    fn check_dag_cycles(flows: &[PrFlowTrigger], issues: &mut Vec<LintIssue>) {
        // Map source events to target triggered events
        let mut adj: HashMap<String, Vec<String>> = HashMap::new();

        for flow in flows {
            if flow.config_json.contains("trigger_event") {
                if let Ok(val) = serde_json::from_str::<serde_json::Value>(&flow.config_json) {
                    if let Some(next_ev) = val.get("next_event").and_then(|v| v.as_str()) {
                        adj.entry(flow.trigger_event.clone())
                            .or_default()
                            .push(next_ev.to_string());
                    }
                }
            }
        }

        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();

        for node in adj.keys() {
            if !visited.contains(node) {
                if Self::has_cycle(node, &adj, &mut visited, &mut rec_stack) {
                    issues.push(LintIssue {
                        category: LintCategory::DagCycle,
                        severity: LintSeverity::Error,
                        code: "LINT_DAG_CYCLE_DETECTED".to_string(),
                        target: node.clone(),
                        message: format!("Infinite recursion cycle detected originating at event '{}'", node),
                        suggested_fix: "Break the circular dependency in the flow graph to prevent UI thread lock".to_string(),
                    });
                    break;
                }
            }
        }
    }

    fn has_cycle(
        curr: &str,
        adj: &HashMap<String, Vec<String>>,
        visited: &mut HashSet<String>,
        rec_stack: &mut HashSet<String>,
    ) -> bool {
        visited.insert(curr.to_string());
        rec_stack.insert(curr.to_string());

        if let Some(neighbors) = adj.get(curr) {
            for neighbor in neighbors {
                if !visited.contains(neighbor) {
                    if Self::has_cycle(neighbor, adj, visited, rec_stack) {
                        return true;
                    }
                } else if rec_stack.contains(neighbor) {
                    return true;
                }
            }
        }

        rec_stack.remove(curr);
        false
    }

    /// Vector 5: Validates ESC/POS actions against standard 32/42/48-column thermal boundaries.
    fn check_hardware_escpos(flows: &[PrFlowTrigger], issues: &mut Vec<LintIssue>) {
        for flow in flows {
            if flow.action_type.eq_ignore_ascii_case("print") || flow.action_type.eq_ignore_ascii_case("escpos") {
                if let Ok(val) = serde_json::from_str::<serde_json::Value>(&flow.config_json) {
                    if let Some(col_width) = val.get("columns").and_then(|v| v.as_i64()) {
                        if col_width != 32 && col_width != 42 && col_width != 48 {
                            issues.push(LintIssue {
                                category: LintCategory::HardwareEscPos,
                                severity: LintSeverity::Warning,
                                code: "LINT_ESCPOS_NON_STANDARD_WIDTH".to_string(),
                                target: flow.id.clone(),
                                message: format!("Non-standard ESC/POS width of {} columns (standard: 32 for 58mm, 42/48 for 80mm)", col_width),
                                suggested_fix: "Set thermal receipt width to 32 (58mm) or 42/48 (80mm) to prevent text clipping".to_string(),
                            });
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::{EntitySchema, FieldDefinition, FieldType};

    fn make_field(name: &str) -> FieldDefinition {
        FieldDefinition {
            name: name.to_string(),
            label: name.to_string(),
            field_type: FieldType::Text,
            required: true,
            default_value: None,
        }
    }

    #[test]
    fn test_clean_package_passes_all_checks() {
        let mut pkg = PrPackage::new("PKG-TEST-01", "Clean Test Template", "PCD-AUTH-01");
        let entity = EntitySchema::new("repairs", "Device Repairs")
            .with_field(make_field("id"))
            .with_field(make_field("device"));
        pkg.schema.entity_schemas.push(entity);
        pkg.schema.ddl_statements.push("CREATE TABLE repairs (id TEXT PRIMARY KEY, device TEXT NOT NULL);".into());
        pkg.views.push(PrViewLayout {
            view_id: "main_view".into(),
            name: "Main View".into(),
            view_type: "intake_form".into(),
            layout_json: "{}".into(),
        });

        let report = PackageLinter::lint_package(&pkg);
        assert!(report.is_valid);
        assert_eq!(report.quality_score, 100.0);
    }

    #[test]
    fn test_destructive_ddl_and_missing_pk_detected() {
        let mut pkg = PrPackage::new("PKG-TEST-02", "Bad Template", "PCD-AUTH-01");
        pkg.schema.ddl_statements.push("DROP TABLE customers;".into());
        let entity = EntitySchema::new("logs", "Logs").with_field(make_field("log_text"));
        pkg.schema.entity_schemas.push(entity);

        let report = PackageLinter::lint_package(&pkg);
        assert!(!report.is_valid);
        assert!(report.errors.iter().any(|e| e.code == "LINT_MIGRATION_DESTRUCTIVE_SQL"));
        assert!(report.errors.iter().any(|e| e.code == "LINT_SCHEMA_NO_PK"));
    }

    #[test]
    fn test_dag_cycle_and_orphaned_view() {
        let mut pkg = PrPackage::new("PKG-TEST-03", "Cycle & Orphan", "PCD-AUTH-01");
        pkg.flows.push(PrFlowTrigger {
            id: "f1".into(),
            trigger_event: "EV_A".into(),
            action_type: "dispatch".into(),
            config_json: r#"{"trigger_event":true,"next_event":"EV_B"}"#.into(),
        });
        pkg.flows.push(PrFlowTrigger {
            id: "f2".into(),
            trigger_event: "EV_B".into(),
            action_type: "dispatch".into(),
            config_json: r#"{"trigger_event":true,"next_event":"EV_A"}"#.into(),
        });
        pkg.views.push(PrViewLayout { view_id: "root".into(), name: "R".into(), view_type: "d".into(), layout_json: "{}".into() });
        pkg.views.push(PrViewLayout { view_id: "lost".into(), name: "L".into(), view_type: "d".into(), layout_json: "{}".into() });

        let report = PackageLinter::lint_package(&pkg);
        assert!(!report.is_valid);
        assert!(report.errors.iter().any(|e| e.code == "LINT_DAG_CYCLE_DETECTED"));
        assert!(report.warnings.iter().any(|w| w.code == "LINT_ORPHANED_VIEW"));
    }
}

