//! Role-Based Access Control (RBAC) & Permission Engine for Proteus Ecosystem.
//! Defines system roles, dynamic permissions, and contextual view capabilities.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UserRole {
    Ceo,
    CustomerService,
    Technician,
    SalesConsultant,
    Developer,
    BusinessAnalyst,
    DataAnalyst,
}

impl UserRole {
    pub fn all() -> &'static [UserRole] {
        &[
            UserRole::Ceo,
            UserRole::CustomerService,
            UserRole::Technician,
            UserRole::SalesConsultant,
            UserRole::Developer,
            UserRole::BusinessAnalyst,
            UserRole::DataAnalyst,
        ]
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Ceo => "👑 CEO / Γενικός Διευθυντής",
            Self::CustomerService => "🤝 Εξυπηρέτηση Πελατών & Ραντεβού",
            Self::Technician => "🛠 Τεχνικός Εργαστηρίου",
            Self::SalesConsultant => "📋 Σύμβουλος Πωλήσεων & SLA",
            Self::Developer => "💻 Προγραμματιστής & Schema Architect",
            Self::BusinessAnalyst => "📊 Business Analyst (PCBA)",
            Self::DataAnalyst => "📈 Data Analyst & Architect (PCDA)",
        }
    }

    pub fn short_code(&self) -> &'static str {
        match self {
            Self::Ceo => "CEO",
            Self::CustomerService => "CS",
            Self::Technician => "TECH",
            Self::SalesConsultant => "SALES",
            Self::Developer => "DEV",
            Self::BusinessAnalyst => "BA",
            Self::DataAnalyst => "DA",
        }
    }

    pub fn permissions(&self) -> RolePermissions {
        match self {
            Self::Ceo => RolePermissions {
                can_view_audit_trail: true,
                can_view_financials: true,
                can_edit_schema: true,
                can_intake_tickets: true,
                can_manage_pipeline: true,
                can_edit_technical_notes: true,
                can_manage_contracts: true,
                can_book_appointments: true,
                can_manage_settings: true,
                can_infer_schemas: true,
                can_define_business_rules: true,
            },
            Self::CustomerService => RolePermissions {
                can_view_audit_trail: false,
                can_view_financials: false,
                can_edit_schema: false,
                can_intake_tickets: true,
                can_manage_pipeline: true,
                can_edit_technical_notes: false,
                can_manage_contracts: false,
                can_book_appointments: true,
                can_manage_settings: false,
                can_infer_schemas: false,
                can_define_business_rules: false,
            },
            Self::Technician => RolePermissions {
                can_view_audit_trail: false,
                can_view_financials: false,
                can_edit_schema: false,
                can_intake_tickets: false,
                can_manage_pipeline: true,
                can_edit_technical_notes: true,
                can_manage_contracts: false,
                can_book_appointments: false,
                can_manage_settings: false,
                can_infer_schemas: false,
                can_define_business_rules: false,
            },
            Self::SalesConsultant => RolePermissions {
                can_view_audit_trail: false,
                can_view_financials: true,
                can_edit_schema: false,
                can_intake_tickets: false,
                can_manage_pipeline: false,
                can_edit_technical_notes: false,
                can_manage_contracts: true,
                can_book_appointments: true,
                can_manage_settings: false,
                can_infer_schemas: false,
                can_define_business_rules: false,
            },
            Self::Developer => RolePermissions {
                can_view_audit_trail: true,
                can_view_financials: false,
                can_edit_schema: true,
                can_intake_tickets: false,
                can_manage_pipeline: false,
                can_edit_technical_notes: false,
                can_manage_contracts: false,
                can_book_appointments: false,
                can_manage_settings: true,
                can_infer_schemas: true,
                can_define_business_rules: true,
            },
            Self::BusinessAnalyst => RolePermissions {
                can_view_audit_trail: true,
                can_view_financials: true,
                can_edit_schema: false,
                can_intake_tickets: false,
                can_manage_pipeline: false,
                can_edit_technical_notes: false,
                can_manage_contracts: true,
                can_book_appointments: false,
                can_manage_settings: false,
                can_infer_schemas: false,
                can_define_business_rules: true,
            },
            Self::DataAnalyst => RolePermissions {
                can_view_audit_trail: true,
                can_view_financials: false,
                can_edit_schema: true,
                can_intake_tickets: false,
                can_manage_pipeline: false,
                can_edit_technical_notes: false,
                can_manage_contracts: false,
                can_book_appointments: false,
                can_manage_settings: false,
                can_infer_schemas: true,
                can_define_business_rules: true,
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RolePermissions {
    pub can_view_audit_trail: bool,
    pub can_view_financials: bool,
    pub can_edit_schema: bool,
    pub can_intake_tickets: bool,
    pub can_manage_pipeline: bool,
    pub can_edit_technical_notes: bool,
    pub can_manage_contracts: bool,
    pub can_book_appointments: bool,
    pub can_manage_settings: bool,
    pub can_infer_schemas: bool,
    pub can_define_business_rules: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ceo_has_all_permissions() {
        let p = UserRole::Ceo.permissions();
        assert!(p.can_view_audit_trail);
        assert!(p.can_view_financials);
        assert!(p.can_edit_schema);
        assert!(p.can_manage_contracts);
        assert!(p.can_manage_settings);
        assert!(p.can_infer_schemas);
        assert!(p.can_define_business_rules);
    }

    #[test]
    fn test_customer_service_restricted() {
        let p = UserRole::CustomerService.permissions();
        assert!(p.can_intake_tickets);
        assert!(p.can_book_appointments);
        assert!(!p.can_view_audit_trail);
        assert!(!p.can_view_financials);
        assert!(!p.can_edit_schema);
        assert!(!p.can_edit_technical_notes);
        assert!(!p.can_infer_schemas);
    }

    #[test]
    fn test_business_analyst_permissions() {
        let p = UserRole::BusinessAnalyst.permissions();
        assert!(!p.can_infer_schemas);
        assert!(p.can_define_business_rules);
        assert!(p.can_view_audit_trail);
        assert!(!p.can_edit_schema);
        assert!(p.can_view_financials);
        assert!(p.can_manage_contracts);
        assert!(!p.can_intake_tickets);
        assert!(!p.can_manage_settings);
    }

    #[test]
    fn test_data_analyst_permissions() {
        let p = UserRole::DataAnalyst.permissions();
        assert!(p.can_infer_schemas);
        assert!(p.can_define_business_rules);
        assert!(p.can_view_audit_trail);
        assert!(p.can_edit_schema);
        assert!(!p.can_view_financials);
        assert!(!p.can_manage_contracts);
        assert!(!p.can_intake_tickets);
        assert!(!p.can_manage_settings);
    }

    #[test]
    fn test_technician_permissions() {
        let p = UserRole::Technician.permissions();
        assert!(p.can_manage_pipeline);
        assert!(p.can_edit_technical_notes);
        assert!(!p.can_intake_tickets);
        assert!(!p.can_edit_schema);
        assert!(!p.can_manage_contracts);
    }

    #[test]
    fn test_developer_permissions() {
        let p = UserRole::Developer.permissions();
        assert!(p.can_edit_schema);
        assert!(p.can_view_audit_trail);
        assert!(!p.can_view_financials);
        assert!(!p.can_manage_contracts);
    }
}
