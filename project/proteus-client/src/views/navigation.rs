//! Role-Driven Workspace Navigation Engine for Proteus BOS.
//! Manages top-level role workspaces (Shop Counter, Analyst Studio, Systems IT, Hardware, Enterprise HQ)
//! and their contextual sub-navigation views, eliminating UI clutter.

use crm_core::roles::{RolePermissions, UserRole};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavTab {
    Intake,
    Pipeline,
    Appointments,
    AuditLog,
    Settings,
    Support,
    Specialist,
    Developer,
    AnalystStudio,
    EnterpriseHQ,
    StoreDirector,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoleWorkspace {
    ShopCounter,      // 🏪 Κατάστημα & Ταμείο (Front Desk, Technician, Cashier)
    AnalystStudio,    // 📊 Data & Business Analyst (PCDA)
    SystemsIT,        // 💻 IT & Systems Specialist (PCSS)
    HardwareSupport,  // 🛠 Hardware & Field Support (PCDS)
    EnterpriseHQ,     // 🏢 Enterprise HQ & Fleet (CEO/Director)
}

impl RoleWorkspace {
    pub fn display_label(&self) -> &'static str {
        match self {
            Self::ShopCounter => "🏪 Κατάστημα",
            Self::AnalystStudio => "📊 Data Analyst",
            Self::SystemsIT => "💻 IT & Systems",
            Self::HardwareSupport => "🛠 Hardware",
            Self::EnterpriseHQ => "🏢 Enterprise HQ",
        }
    }

    pub fn short_code(&self) -> &'static str {
        match self {
            Self::ShopCounter => "SHOP",
            Self::AnalystStudio => "PCDA",
            Self::SystemsIT => "PCSS",
            Self::HardwareSupport => "PCDS",
            Self::EnterpriseHQ => "HQ",
        }
    }

    pub fn sub_tabs(&self, permissions: &RolePermissions, active_role: UserRole) -> Vec<(NavTab, &'static str)> {
        let mut tabs = Vec::new();
        match self {
            Self::ShopCounter => {
                if permissions.can_intake_tickets {
                    tabs.push((NavTab::Intake, "⚡ Νέα Παραλαβή"));
                }
                if permissions.can_manage_pipeline {
                    tabs.push((NavTab::Pipeline, "📋 Ροή Επισκευών"));
                }
                if permissions.can_book_appointments {
                    tabs.push((NavTab::Appointments, "📅 Ραντεβού"));
                }
            }
            Self::AnalystStudio => {
                if permissions.can_infer_schemas || permissions.can_define_business_rules {
                    tabs.push((NavTab::AnalystStudio, "📊 Ingestion & Pipeline"));
                }
                if active_role == UserRole::Ceo
                    || active_role == UserRole::SalesConsultant
                    || active_role == UserRole::BusinessAnalyst
                {
                    tabs.push((NavTab::Specialist, "📈 Ειδικά Dashboards"));
                }
            }
            Self::SystemsIT => {
                if permissions.can_edit_schema {
                    tabs.push((NavTab::Developer, "💻 Dev & Migrations"));
                }
                if permissions.can_view_audit_trail {
                    tabs.push((NavTab::AuditLog, "📜 Merkle Audit"));
                }
                if permissions.can_manage_settings {
                    tabs.push((NavTab::Settings, "⚙ Ρυθμίσεις"));
                }
            }
            Self::HardwareSupport => {
                if active_role == UserRole::Ceo
                    || active_role == UserRole::Technician
                    || active_role == UserRole::Developer
                {
                    tabs.push((NavTab::Support, "🛠 Spooler & LAN Diagnostics"));
                }
            }
            Self::EnterpriseHQ => {
                if active_role == UserRole::Ceo {
                    tabs.push((NavTab::EnterpriseHQ, "🏢 Multi-Store Fleet"));
                    tabs.push((NavTab::StoreDirector, "🏪 Store Director KPIs"));
                }
            }
        }
        tabs
    }

    pub fn default_tab(&self, permissions: &RolePermissions, active_role: UserRole) -> NavTab {
        let tabs = self.sub_tabs(permissions, active_role);
        if let Some((tab, _)) = tabs.first() {
            *tab
        } else {
            match self {
                Self::ShopCounter => NavTab::Pipeline,
                Self::AnalystStudio => NavTab::AnalystStudio,
                Self::SystemsIT => NavTab::Developer,
                Self::HardwareSupport => NavTab::Support,
                Self::EnterpriseHQ => NavTab::EnterpriseHQ,
            }
        }
    }
}

pub fn available_workspaces(role: UserRole) -> Vec<RoleWorkspace> {
    match role {
        UserRole::CustomerService => vec![RoleWorkspace::ShopCounter],
        UserRole::Technician => vec![RoleWorkspace::ShopCounter, RoleWorkspace::HardwareSupport],
        UserRole::SalesConsultant => vec![RoleWorkspace::ShopCounter, RoleWorkspace::AnalystStudio],
        UserRole::Developer => vec![
            RoleWorkspace::SystemsIT,
            RoleWorkspace::HardwareSupport,
            RoleWorkspace::AnalystStudio,
        ],
        UserRole::BusinessAnalyst => vec![RoleWorkspace::AnalystStudio, RoleWorkspace::SystemsIT],
        UserRole::Ceo => vec![
            RoleWorkspace::ShopCounter,
            RoleWorkspace::AnalystStudio,
            RoleWorkspace::SystemsIT,
            RoleWorkspace::HardwareSupport,
            RoleWorkspace::EnterpriseHQ,
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ceo_has_all_workspaces() {
        let ws = available_workspaces(UserRole::Ceo);
        assert_eq!(ws.len(), 5);
        assert!(ws.contains(&RoleWorkspace::ShopCounter));
        assert!(ws.contains(&RoleWorkspace::AnalystStudio));
        assert!(ws.contains(&RoleWorkspace::SystemsIT));
        assert!(ws.contains(&RoleWorkspace::HardwareSupport));
        assert!(ws.contains(&RoleWorkspace::EnterpriseHQ));
    }

    #[test]
    fn test_customer_service_workspace() {
        let ws = available_workspaces(UserRole::CustomerService);
        assert_eq!(ws, vec![RoleWorkspace::ShopCounter]);

        let p = UserRole::CustomerService.permissions();
        let tabs = RoleWorkspace::ShopCounter.sub_tabs(&p, UserRole::CustomerService);
        assert_eq!(tabs.len(), 3);
        assert_eq!(tabs[0].0, NavTab::Intake);
        assert_eq!(tabs[1].0, NavTab::Pipeline);
        assert_eq!(tabs[2].0, NavTab::Appointments);
    }

    #[test]
    fn test_technician_workspace_and_tabs() {
        let ws = available_workspaces(UserRole::Technician);
        assert_eq!(
            ws,
            vec![RoleWorkspace::ShopCounter, RoleWorkspace::HardwareSupport]
        );

        let p = UserRole::Technician.permissions();
        let tabs = RoleWorkspace::ShopCounter.sub_tabs(&p, UserRole::Technician);
        // Technician cannot intake tickets, so only pipeline
        assert_eq!(tabs.len(), 1);
        assert_eq!(tabs[0].0, NavTab::Pipeline);

        let hw_tabs = RoleWorkspace::HardwareSupport.sub_tabs(&p, UserRole::Technician);
        assert_eq!(hw_tabs.len(), 1);
        assert_eq!(hw_tabs[0].0, NavTab::Support);
    }

    #[test]
    fn test_developer_workspaces() {
        let ws = available_workspaces(UserRole::Developer);
        assert_eq!(
            ws,
            vec![
                RoleWorkspace::SystemsIT,
                RoleWorkspace::HardwareSupport,
                RoleWorkspace::AnalystStudio
            ]
        );

        let p = UserRole::Developer.permissions();
        let it_tabs = RoleWorkspace::SystemsIT.sub_tabs(&p, UserRole::Developer);
        assert_eq!(it_tabs.len(), 3);
        assert_eq!(it_tabs[0].0, NavTab::Developer);
        assert_eq!(it_tabs[1].0, NavTab::AuditLog);
        assert_eq!(it_tabs[2].0, NavTab::Settings);
    }

    #[test]
    fn test_business_analyst_workspaces() {
        let ws = available_workspaces(UserRole::BusinessAnalyst);
        assert_eq!(
            ws,
            vec![RoleWorkspace::AnalystStudio, RoleWorkspace::SystemsIT]
        );

        let p = UserRole::BusinessAnalyst.permissions();
        let studio_tabs = RoleWorkspace::AnalystStudio.sub_tabs(&p, UserRole::BusinessAnalyst);
        assert_eq!(studio_tabs.len(), 2);
        assert_eq!(studio_tabs[0].0, NavTab::AnalystStudio);
        assert_eq!(studio_tabs[1].0, NavTab::Specialist);
    }

    #[test]
    fn test_default_tabs_resolution() {
        let p = UserRole::Ceo.permissions();
        assert_eq!(
            RoleWorkspace::ShopCounter.default_tab(&p, UserRole::Ceo),
            NavTab::Intake
        );
        assert_eq!(
            RoleWorkspace::AnalystStudio.default_tab(&p, UserRole::Ceo),
            NavTab::AnalystStudio
        );
        assert_eq!(
            RoleWorkspace::SystemsIT.default_tab(&p, UserRole::Ceo),
            NavTab::Developer
        );
        assert_eq!(
            RoleWorkspace::HardwareSupport.default_tab(&p, UserRole::Ceo),
            NavTab::Support
        );
        assert_eq!(
            RoleWorkspace::EnterpriseHQ.default_tab(&p, UserRole::Ceo),
            NavTab::EnterpriseHQ
        );
    }

    #[test]
    fn test_workspace_metadata() {
        assert_eq!(RoleWorkspace::ShopCounter.short_code(), "SHOP");
        assert_eq!(RoleWorkspace::AnalystStudio.short_code(), "PCDA");
        assert_eq!(RoleWorkspace::SystemsIT.short_code(), "PCSS");
        assert_eq!(RoleWorkspace::HardwareSupport.short_code(), "PCDS");
        assert_eq!(RoleWorkspace::EnterpriseHQ.short_code(), "HQ");

        assert_eq!(RoleWorkspace::ShopCounter.display_label(), "🏪 Κατάστημα");
        assert_eq!(RoleWorkspace::AnalystStudio.display_label(), "📊 Data Analyst");
    }
}
