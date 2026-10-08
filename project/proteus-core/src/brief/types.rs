//! Core types for the Client Project Brief engine.
//! Defines the domain model for bespoke business application briefs and canvas scaffold specs.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BriefTrack {
    PcdApp,
    PcdWeb,
    DualStack,
}

impl BriefTrack {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::PcdApp => "PCD-App",
            Self::PcdWeb => "PCD-Web",
            Self::DualStack => "Dual-Stack",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "PCD-Web" | "pcd-web" => Self::PcdWeb,
            "Dual-Stack" | "dual-stack" | "Dual" => Self::DualStack,
            _ => Self::PcdApp,
        }
    }
}

/// Detailed brief submitted by a customer for their bespoke business application.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ClientProjectBrief {
    pub brief_id: String,
    pub business_name: String,
    pub business_nature: String,
    pub daily_operations_desc: String,
    pub track: BriefTrack,
    pub required_screens: Vec<String>,
    pub hardware_peripherals: Vec<String>,
    pub proposed_budget_eur: f64,
    pub domain_name_requested: Option<String>,
    pub hosting_preference: String, // "ManagedCloud" or "SelfHosted"
    pub contact_email: String,
    pub submitted_at: String,
}

/// Visual element in an auto-scaffolded canvas screen.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScaffoldElement {
    pub element_type: String, // "Header", "Button", "Input", "Table", "Card"
    pub label: String,
    pub width: f32,
    pub height: f32,
    pub x: f32,
    pub y: f32,
    pub shortcut: Option<String>,
    pub binding: Option<String>,
}

/// Specifications for a canvas artboard / frame generated from a brief.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScaffoldScreenSpec {
    pub screen_id: String,
    pub title: String,
    pub width: f32,
    pub height: f32,
    pub pos_x: f32,
    pub pos_y: f32,
    pub elements: Vec<ScaffoldElement>,
}
