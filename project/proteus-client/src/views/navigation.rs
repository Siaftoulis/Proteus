//! Role-Driven Workspace Navigation Engine for Proteus BOS.
//! Manages top-level role workspaces (Shop Counter, Analyst Studio, Systems IT, Hardware, Enterprise HQ)
//! and their contextual sub-navigation views, eliminating UI clutter.

use proteus_core::roles::{RolePermissions, UserRole};

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
    FleetRadar,
    ContractorLedger,
    SupplierReconcile,
    GenealogyRma,
    ShippingNote,
    ColdChain,
    RetailPos,
    SpatialWms,
    WorkCard,
    EslGateway,
    AccountingCpa,
    CardexLedger,
    NotificationsGateway,
    BranchMesh,
    CloudHosting,
    PilotTelemetry,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoleWorkspace {
    ShopCounter,      // 🏪 Κατάστημα & Ταμείο (Front Desk, Technician, Cashier)
    Logistics,        // 🚚 Logistics & Αποθήκη (Shipping Notes, Cold Chain, RMA, Suppliers)
    AnalystStudio,    // 📊 Data & Business Analyst (PCDA)
    SystemsIT,        // 💻 IT & Systems Specialist (PCSS)
    HardwareSupport,  // 🛠 Hardware & Field Support (PCDS)
    EnterpriseHQ,     // 🏢 Enterprise HQ & Fleet (CEO/Director)
}

impl RoleWorkspace {
    pub fn display_label(&self) -> &'static str {
        match self {
            Self::ShopCounter => "Κατάστημα",
            Self::Logistics => "Logistics & Αποθήκη",
            Self::AnalystStudio => "Data Analyst",
            Self::SystemsIT => "IT & Systems",
            Self::HardwareSupport => "Hardware",
            Self::EnterpriseHQ => "Enterprise HQ",
        }
    }

    pub fn short_code(&self) -> &'static str {
        match self {
            Self::ShopCounter => "SHOP",
            Self::Logistics => "LOG",
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
                    tabs.push((NavTab::Intake, "Νέα Παραλαβή"));
                }
                tabs.push((NavTab::RetailPos, "Ταμείο & myDATA"));
                if permissions.can_manage_pipeline {
                    tabs.push((NavTab::Pipeline, "Ροή Επισκευών"));
                }
                if permissions.can_book_appointments {
                    tabs.push((NavTab::Appointments, "Ραντεβού"));
                }
                tabs.push((NavTab::ContractorLedger, "Ταμείο Υλικών & Μάστορες"));
                tabs.push((NavTab::WorkCard, "⏱ Κάρτα Εργασίας"));
                tabs.push((NavTab::CardexLedger, "👥 Καρτέλες CRM"));
                tabs.push((NavTab::NotificationsGateway, "📢 Ειδοποιήσεις & Live Track"));
            }
            Self::Logistics => {
                tabs.push((NavTab::ShippingNote, "Δελτία Αποστολής"));
                tabs.push((NavTab::SpatialWms, "Χωρικό WMS & Ράφια"));
                tabs.push((NavTab::EslGateway, "🏷 Ετικέτες ESL"));
                tabs.push((NavTab::ColdChain, "Τηλεμετρία & HACCP"));
                tabs.push((NavTab::GenealogyRma, "Ιστορικό S/N & RMA"));
                if active_role == UserRole::Ceo || active_role == UserRole::SalesConsultant {
                    tabs.push((NavTab::SupplierReconcile, "Τιμοκατάλογοι Προμηθευτών"));
                }
            }
            Self::AnalystStudio => {
                if permissions.can_infer_schemas {
                    tabs.push((NavTab::AnalystStudio, "Ingestion & Pipeline"));
                }
                if permissions.can_define_business_rules
                    || active_role == UserRole::Ceo
                    || active_role == UserRole::SalesConsultant
                    || active_role == UserRole::BusinessAnalyst
                    || active_role == UserRole::DataAnalyst
                {
                    tabs.push((NavTab::Specialist, "Ειδικά Dashboards"));
                }
            }
            Self::SystemsIT => {
                if permissions.can_edit_schema {
                    tabs.push((NavTab::Developer, "Dev & Migrations"));
                }
                if permissions.can_view_audit_trail {
                    tabs.push((NavTab::AuditLog, "Merkle Audit"));
                }
                if permissions.can_manage_settings {
                    tabs.push((NavTab::Settings, "Ρυθμίσεις"));
                }
                tabs.push((NavTab::BranchMesh, "🕸 Δίκτυο Mesh"));
                tabs.push((NavTab::CloudHosting, "☁️ Cloud Υποδομές"));
            }
            Self::HardwareSupport => {
                if active_role == UserRole::Ceo
                    || active_role == UserRole::Technician
                    || active_role == UserRole::Developer
                {
                    tabs.push((NavTab::Support, "Spooler & LAN Diagnostics"));
                    tabs.push((NavTab::GenealogyRma, "Ιστορικό S/N & RMA"));
                    tabs.push((NavTab::PilotTelemetry, "🩺 Pilot Τηλεμετρία"));
                }
            }
            Self::EnterpriseHQ => {
                if active_role == UserRole::Ceo || active_role == UserRole::BusinessAnalyst {
                    tabs.push((NavTab::EnterpriseHQ, "Multi-Store Fleet"));
                    tabs.push((NavTab::FleetRadar, "🛰 Live Telemetry Radar"));
                    tabs.push((NavTab::StoreDirector, "Store Director KPIs"));
                    tabs.push((NavTab::WorkCard, "⏱ Κάρτα Εργασίας & Ωράριο"));
                    tabs.push((NavTab::AccountingCpa, "📊 1-Click Λογιστήριο"));
                    tabs.push((NavTab::CardexLedger, "👥 Καρτέλες CRM"));
                    tabs.push((NavTab::NotificationsGateway, "📢 Ειδοποιήσεις & Live Track"));
                    tabs.push((NavTab::BranchMesh, "🕸 Δίκτυο Mesh"));
                    tabs.push((NavTab::CloudHosting, "☁️ Cloud Υποδομές"));
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
                Self::Logistics => NavTab::ShippingNote,
                Self::AnalystStudio => {
                    if permissions.can_infer_schemas {
                        NavTab::AnalystStudio
                    } else {
                        NavTab::Specialist
                    }
                }
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
        UserRole::Technician => vec![
            RoleWorkspace::ShopCounter,
            RoleWorkspace::HardwareSupport,
            RoleWorkspace::Logistics,
        ],
        UserRole::SalesConsultant => vec![
            RoleWorkspace::ShopCounter,
            RoleWorkspace::Logistics,
            RoleWorkspace::AnalystStudio,
        ],
        UserRole::Developer => vec![
            RoleWorkspace::SystemsIT,
            RoleWorkspace::HardwareSupport,
            RoleWorkspace::AnalystStudio,
            RoleWorkspace::Logistics,
        ],
        UserRole::BusinessAnalyst => vec![
            RoleWorkspace::AnalystStudio,
            RoleWorkspace::EnterpriseHQ,
            RoleWorkspace::Logistics,
        ],
        UserRole::DataAnalyst => vec![RoleWorkspace::AnalystStudio, RoleWorkspace::SystemsIT],
        UserRole::Ceo => vec![
            RoleWorkspace::ShopCounter,
            RoleWorkspace::Logistics,
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
        assert_eq!(ws.len(), 6);
        assert!(ws.contains(&RoleWorkspace::ShopCounter));
        assert!(ws.contains(&RoleWorkspace::Logistics));
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
        assert_eq!(tabs.len(), 8);
        assert_eq!(tabs[0].0, NavTab::Intake);
        assert_eq!(tabs[1].0, NavTab::RetailPos);
        assert_eq!(tabs[2].0, NavTab::Pipeline);
        assert_eq!(tabs[3].0, NavTab::Appointments);
        assert_eq!(tabs[4].0, NavTab::ContractorLedger);
        assert_eq!(tabs[5].0, NavTab::WorkCard);
        assert_eq!(tabs[6].0, NavTab::CardexLedger);
        assert_eq!(tabs[7].0, NavTab::NotificationsGateway);
    }

    #[test]
    fn test_technician_workspace_and_tabs() {
        let ws = available_workspaces(UserRole::Technician);
        assert_eq!(
            ws,
            vec![
                RoleWorkspace::ShopCounter,
                RoleWorkspace::HardwareSupport,
                RoleWorkspace::Logistics
            ]
        );

        let p = UserRole::Technician.permissions();
        let tabs = RoleWorkspace::ShopCounter.sub_tabs(&p, UserRole::Technician);
        assert_eq!(tabs.len(), 6);
        assert_eq!(tabs[0].0, NavTab::RetailPos);
        assert_eq!(tabs[1].0, NavTab::Pipeline);
        assert_eq!(tabs[2].0, NavTab::ContractorLedger);
        assert_eq!(tabs[3].0, NavTab::WorkCard);
        assert_eq!(tabs[4].0, NavTab::CardexLedger);
        assert_eq!(tabs[5].0, NavTab::NotificationsGateway);

        let hw_tabs = RoleWorkspace::HardwareSupport.sub_tabs(&p, UserRole::Technician);
        assert_eq!(hw_tabs.len(), 3);
        assert_eq!(hw_tabs[0].0, NavTab::Support);
        assert_eq!(hw_tabs[1].0, NavTab::GenealogyRma);
        assert_eq!(hw_tabs[2].0, NavTab::PilotTelemetry);

        let log_tabs = RoleWorkspace::Logistics.sub_tabs(&p, UserRole::Technician);
        assert_eq!(log_tabs.len(), 5);
        assert_eq!(log_tabs[0].0, NavTab::ShippingNote);
        assert_eq!(log_tabs[1].0, NavTab::SpatialWms);
        assert_eq!(log_tabs[2].0, NavTab::EslGateway);
        assert_eq!(log_tabs[3].0, NavTab::ColdChain);
        assert_eq!(log_tabs[4].0, NavTab::GenealogyRma);
    }

    #[test]
    fn test_developer_workspaces() {
        let ws = available_workspaces(UserRole::Developer);
        assert_eq!(
            ws,
            vec![
                RoleWorkspace::SystemsIT,
                RoleWorkspace::HardwareSupport,
                RoleWorkspace::AnalystStudio,
                RoleWorkspace::Logistics,
            ]
        );

        let p = UserRole::Developer.permissions();
        let it_tabs = RoleWorkspace::SystemsIT.sub_tabs(&p, UserRole::Developer);
        assert_eq!(it_tabs.len(), 5);
        assert_eq!(it_tabs[0].0, NavTab::Developer);
        assert_eq!(it_tabs[1].0, NavTab::AuditLog);
        assert_eq!(it_tabs[2].0, NavTab::Settings);
        assert_eq!(it_tabs[3].0, NavTab::BranchMesh);
        assert_eq!(it_tabs[4].0, NavTab::CloudHosting);
    }

    #[test]
    fn test_business_analyst_workspaces() {
        let ws = available_workspaces(UserRole::BusinessAnalyst);
        assert_eq!(
            ws,
            vec![
                RoleWorkspace::AnalystStudio,
                RoleWorkspace::EnterpriseHQ,
                RoleWorkspace::Logistics,
            ]
        );

        let p = UserRole::BusinessAnalyst.permissions();
        let studio_tabs = RoleWorkspace::AnalystStudio.sub_tabs(&p, UserRole::BusinessAnalyst);
        assert_eq!(studio_tabs.len(), 1);
        assert_eq!(studio_tabs[0].0, NavTab::Specialist);

        let hq_tabs = RoleWorkspace::EnterpriseHQ.sub_tabs(&p, UserRole::BusinessAnalyst);
        assert_eq!(hq_tabs.len(), 9);
        assert_eq!(hq_tabs[4].0, NavTab::AccountingCpa);
        assert_eq!(hq_tabs[5].0, NavTab::CardexLedger);
        assert_eq!(hq_tabs[6].0, NavTab::NotificationsGateway);
        assert_eq!(hq_tabs[7].0, NavTab::BranchMesh);
        assert_eq!(hq_tabs[8].0, NavTab::CloudHosting);
    }

    #[test]
    fn test_data_analyst_workspaces() {
        let ws = available_workspaces(UserRole::DataAnalyst);
        assert_eq!(
            ws,
            vec![RoleWorkspace::AnalystStudio, RoleWorkspace::SystemsIT]
        );

        let p = UserRole::DataAnalyst.permissions();
        let studio_tabs = RoleWorkspace::AnalystStudio.sub_tabs(&p, UserRole::DataAnalyst);
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
        assert_eq!(
            RoleWorkspace::Logistics.default_tab(&p, UserRole::Ceo),
            NavTab::ShippingNote
        );
    }

    #[test]
    fn test_workspace_metadata() {
        assert_eq!(RoleWorkspace::ShopCounter.short_code(), "SHOP");
        assert_eq!(RoleWorkspace::Logistics.short_code(), "LOG");
        assert_eq!(RoleWorkspace::AnalystStudio.short_code(), "PCDA");
        assert_eq!(RoleWorkspace::SystemsIT.short_code(), "PCSS");
        assert_eq!(RoleWorkspace::HardwareSupport.short_code(), "PCDS");
        assert_eq!(RoleWorkspace::EnterpriseHQ.short_code(), "HQ");

        assert_eq!(RoleWorkspace::ShopCounter.display_label(), "Κατάστημα");
        assert_eq!(RoleWorkspace::Logistics.display_label(), "Logistics & Αποθήκη");
        assert_eq!(RoleWorkspace::AnalystStudio.display_label(), "Data Analyst");
    }
}
