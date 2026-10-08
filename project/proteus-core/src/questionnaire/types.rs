//! Questionnaire domain types, business scale models, and execution strategies.

use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// High-level industry sectors for top-level business classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TopIndustryCategory {
    TechnologyAndRepairs,
    RetailAndCommerce,
    HealthcareAndMedical,
    HospitalityAndFood,
    ServicesAndOffices,
    PersonalCareAndWellness,
    CraftsAndManufacturing,
}

impl TopIndustryCategory {
    pub fn display_name_el(&self) -> &'static str {
        match self {
            Self::TechnologyAndRepairs => "Τεχνολογικό / Επισκευές & Τεχνικά Επαγγέλματα",
            Self::RetailAndCommerce => "Εμπορικό / Λιανική & Χονδρική",
            Self::HealthcareAndMedical => "Ιατρικό / Υγεία & Φροντίδα",
            Self::HospitalityAndFood => "Εστίαση, Καφέ & Φιλοξενία",
            Self::ServicesAndOffices => "Υπηρεσίες, Γραφεία & Ελεύθερα Επαγγέλματα",
            Self::PersonalCareAndWellness => "Προσωπική Φροντίδα, Κομμωτήρια & Ευεξία",
            Self::CraftsAndManufacturing => "Βιοτεχνία, Εργαστήρια & Παραγωγή",
        }
    }

    pub fn display_name_en(&self) -> &'static str {
        match self {
            Self::TechnologyAndRepairs => "Technology, Repairs & Technical Trades",
            Self::RetailAndCommerce => "Retail & Wholesale Commerce",
            Self::HealthcareAndMedical => "Healthcare & Medical Practice",
            Self::HospitalityAndFood => "Hospitality, Food & Cafes",
            Self::ServicesAndOffices => "Professional Services & Offices",
            Self::PersonalCareAndWellness => "Personal Care, Salons & Wellness",
            Self::CraftsAndManufacturing => "Crafts, Workshops & Light Manufacturing",
        }
    }

    pub fn all() -> &'static [TopIndustryCategory] {
        &[
            Self::TechnologyAndRepairs,
            Self::RetailAndCommerce,
            Self::HealthcareAndMedical,
            Self::HospitalityAndFood,
            Self::ServicesAndOffices,
            Self::PersonalCareAndWellness,
            Self::CraftsAndManufacturing,
        ]
    }
}

/// Granular sub-category classification under a top industry sector.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubCategory {
    pub id: String,
    pub top_category: TopIndustryCategory,
    pub title_el: String,
    pub title_en: String,
}

/// Detailed operational and workflow checklist flags.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct WorkflowRequirements {
    pub has_intake_voucher: bool,
    pub has_serial_imei_tracking: bool,
    pub has_spare_parts_inventory: bool,
    pub has_technician_labor_rates: bool,
    pub has_customer_sms_notifications: bool,
    pub has_mydata_invoicing: bool,
    pub has_eftpos_interlock: bool,
    pub has_ergani_work_card: bool,
    pub custom_notes: String,
}

/// Operational scale of the business.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct BusinessScale {
    pub employee_count: u32,
    pub workstation_count: u32,
    pub monthly_volume_bracket: String,
    pub branch_count: u32,
}

/// Preferred execution model for Enterprise-scale deployments.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EnterpriseExecutionModel {
    /// Staff project with freelance Proteus Certified Solution Architects
    StaffWithCertifiedFreelancers,
    /// Train internal enterprise IT/Operations staff via Proteus Platform Academy
    InHouseAcademyTraining,
    /// Hybrid model: Architect blueprint supervised with internal team co-building
    HybridArchitectSupervised,
}

impl Default for EnterpriseExecutionModel {
    fn default() -> Self {
        Self::StaffWithCertifiedFreelancers
    }
}

/// Selected path for system design and implementation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DesignStrategy {
    SoloFreeStarter,
    HireDesignerDirect {
        designer_id: String,
        budget_eur: f64,
    },
    OpenTenderRfp {
        max_budget_eur: f64,
        require_nda: bool,
    },
    EnterpriseConsultation {
        contact_person: String,
        contact_email: String,
        branch_count: u32,
        legacy_erp: Option<String>,
        preferred_execution: EnterpriseExecutionModel,
    },
}

impl Default for DesignStrategy {
    fn default() -> Self {
        Self::SoloFreeStarter
    }
}

/// Complete business questionnaire profile.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BusinessQuestionnaire {
    pub id: String,
    pub business_name: String,
    pub afm: Option<String>,
    pub top_category: TopIndustryCategory,
    pub sub_category_id: String,
    pub workflow: WorkflowRequirements,
    pub scale: BusinessScale,
    pub strategy: DesignStrategy,
    pub created_at: String,
    pub updated_at: String,
}

impl BusinessQuestionnaire {
    pub fn new(
        business_name: impl Into<String>,
        top_category: TopIndustryCategory,
        sub_category_id: impl Into<String>,
    ) -> Self {
        let now = Utc::now().to_rfc3339();
        Self {
            id: Uuid::now_v7().to_string(),
            business_name: business_name.into(),
            afm: None,
            top_category,
            sub_category_id: sub_category_id.into(),
            workflow: WorkflowRequirements::default(),
            scale: BusinessScale::default(),
            strategy: DesignStrategy::default(),
            created_at: now.clone(),
            updated_at: now,
        }
    }
}
