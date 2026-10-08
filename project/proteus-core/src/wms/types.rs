//! Spatial WMS & Retail Shelf Optimizer Data Types for Proteus BOS.
//! Defines 3D dimensions, storage forms, ergonomic golden zones, and warehouse models.
//! Strict Rule 1 (100% Original Codebase) and Rule 5 (Zero Mock Data).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Dimensions {
    pub width_cm: f64,
    pub height_cm: f64,
    pub depth_cm: f64,
}

impl Dimensions {
    pub fn new(width_cm: f64, height_cm: f64, depth_cm: f64) -> Self {
        Self {
            width_cm: width_cm.max(0.0),
            height_cm: height_cm.max(0.0),
            depth_cm: depth_cm.max(0.0),
        }
    }

    pub fn volume_cm3(&self) -> f64 {
        self.width_cm * self.height_cm * self.depth_cm
    }

    pub fn is_valid(&self) -> bool {
        self.width_cm > 0.0 && self.height_cm > 0.0 && self.depth_cm > 0.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StorageForm {
    /// Rigid boxes, cartons, electronics packaging (all 6 3D orientations allowed unless upright-only)
    Cuboid,
    /// Folded textiles, garments in bags, flat packs (vertical stacking preferred)
    Stackable,
    /// Garments on hangers, cables on hooks (height axis is fixed upright)
    Hanging,
    /// Small components, loose screws, electronic parts in modular bins or totes
    BinsTotes,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TurnoverVelocity {
    /// High-velocity products (80% of picks; belongs in the Golden Zone 0.8m-1.6m)
    FastMover,
    /// Moderate picking frequency (eye-to-shoulder or knee-to-waist)
    MediumMover,
    /// Low picking frequency, seasonal or bulk reserve (high buffer >2.2m)
    SlowMover,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GoldenZoneCategory {
    /// Below 80cm: Bulky, heavy goods (>15kg) to prevent worker strain
    GroundHeavy,
    /// 80cm - 160cm: Prime eye-and-reach level; maximum speed and ergonomics
    GoldenZone,
    /// 160cm - 220cm: Upper shelf reach; lightweight medium movers
    SlowMoversUpper,
    /// Above 220cm: Buffer reserve requiring ladder or reach equipment
    HighBufferReserve,
}

impl GoldenZoneCategory {
    pub fn classify(height_from_floor_cm: f64) -> Self {
        if height_from_floor_cm < 80.0 {
            Self::GroundHeavy
        } else if height_from_floor_cm <= 160.0 {
            Self::GoldenZone
        } else if height_from_floor_cm <= 220.0 {
            Self::SlowMoversUpper
        } else {
            Self::HighBufferReserve
        }
    }

    pub fn display_label(&self) -> &'static str {
        match self {
            Self::GroundHeavy => "Βαρέα / Εδάφους (<80cm)",
            Self::GoldenZone => "Χρυσή Ζώνη (80-160cm)",
            Self::SlowMoversUpper => "Άνω Ράφι (160-220cm)",
            Self::HighBufferReserve => "Απόθεμα Ασφαλείας (>220cm)",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WarehouseZone {
    pub zone_id: String,
    pub name: String,
    pub code: String,
    pub temperature_class: String, // Ambient, Chilled, Frozen
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WarehouseRack {
    pub rack_id: String,
    pub zone_id: String,
    pub rack_code: String,
    pub aisle_number: u32,
    pub x_pos: f64,
    pub y_pos: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WarehouseShelf {
    pub shelf_id: String,
    pub rack_id: String,
    pub shelf_level: u32,
    pub height_from_floor_cm: f64,
    pub dimensions: Dimensions,
    pub max_weight_kg: f64,
    pub current_weight_kg: f64,
}

impl WarehouseShelf {
    pub fn ergonomic_zone(&self) -> GoldenZoneCategory {
        GoldenZoneCategory::classify(self.height_from_floor_cm)
    }

    pub fn remaining_weight_kg(&self) -> f64 {
        (self.max_weight_kg - self.current_weight_kg).max(0.0)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WmsItem {
    pub sku: String,
    pub name: String,
    pub dimensions: Dimensions,
    pub weight_kg: f64,
    pub storage_form: StorageForm,
    pub turnover_velocity: TurnoverVelocity,
    pub is_upright_only: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PutawayRecommendation {
    pub shelf_id: String,
    pub shelf_level: u32,
    pub best_orientation: Dimensions,
    pub max_units_fit: u32,
    pub clearance_percent: f64,
    pub ergonomic_zone: GoldenZoneCategory,
    pub rationale: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ShelfAuditProof {
    pub audit_id: String,
    pub shelf_id: String,
    pub scanned_sku: String,
    pub is_misplaced: bool,
    pub expected_sku: Option<String>,
    pub photo_sha256: Option<String>,
    pub audit_timestamp: i64,
}
