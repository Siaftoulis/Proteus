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
}

impl UserRole {
    pub fn all() -> &'static [UserRole] {
        &[
            UserRole::Ceo,
            UserRole::CustomerService,
            UserRole::Technician,
            UserRole::SalesConsultant,
            UserRole::Developer,
        ]
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Ceo => "👑 CEO / Γενικός Διευθυντής",
            Self::CustomerService => "🤝 Εξυπηρέτηση Πελατών & Ραντεβού",
            Self::Technician => "🛠 Τεχνικός Εργαστηρίου",
            Self::SalesConsultant => "📋 Σύμβουλος Πωλήσεων & SLA",
            Self::Developer => "💻 Προγραμματιστής & Schema Architect",
        }
    }

    pub fn short_code(&self) -> &'static str {
        match self {
            Self::Ceo => "CEO",
            Self::CustomerService => "CS",
            Self::Technician => "TECH",
            Self::SalesConsultant => "SALES",
            Self::Developer => "DEV",
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
