//! Algorithmic Complexity Floor Price Engine & Project Slot Board for Proteus.
//! Enforces:
//! - Mathematical floor price to eliminate under-the-table evasion:
//!   Floor Price = Base Overhead (€150) + (€45 × screens) + (€35 × entities) + (€25 × triggers) + (€60 × hardware)
//! - 6-Role automated escrow allocations (CS 10%, BA 15%, DA 18%, PCD 22%, PCSS 15%, PCDS 10%) + Platform 10%.
//! - Project Slot Board architecture for community freelancer matching and instant accept offers.

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::package::PrPackage;
use crate::roles::UserRole;

#[derive(Debug, Error, PartialEq, Clone)]
pub enum FloorPricingError {
    #[error("Proposed budget €{proposed:.2} is below the algorithmic floor price €{minimum_floor:.2} (deficit: €{deficit:.2})")]
    BudgetBelowFloor {
        proposed: f64,
        minimum_floor: f64,
        deficit: f64,
    },
    #[error("Invalid metric: {0}")]
    InvalidMetric(String),
}

/// Project complexity metrics parsed from manifests and hardware declarations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ProjectComplexityMetrics {
    pub screens_count: usize,
    pub entities_count: usize,
    pub triggers_count: usize,
    pub hardware_peripherals_count: usize,
}

impl ProjectComplexityMetrics {
    pub fn new(screens: usize, entities: usize, triggers: usize, hardware: usize) -> Self {
        Self {
            screens_count: screens,
            entities_count: entities,
            triggers_count: triggers,
            hardware_peripherals_count: hardware,
        }
    }

    /// Automatically extracts complexity metrics from a declarative PrPackage bundle.
    pub fn from_package(pkg: &PrPackage, hardware_count: usize) -> Self {
        Self {
            screens_count: pkg.views.len(),
            entities_count: pkg.schema.entity_schemas.len(),
            triggers_count: pkg.flows.len(),
            hardware_peripherals_count: hardware_count,
        }
    }
}

/// Baseline configuration for algorithmic floor pricing calculations.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ComplexityFloorConfig {
    pub base_overhead_eur: f64,
    pub per_screen_eur: f64,
    pub per_entity_eur: f64,
    pub per_trigger_eur: f64,
    pub per_hardware_eur: f64,
}

impl Default for ComplexityFloorConfig {
    fn default() -> Self {
        Self {
            base_overhead_eur: 150.0,
            per_screen_eur: 45.0,
            per_entity_eur: 35.0,
            per_trigger_eur: 25.0,
            per_hardware_eur: 60.0,
        }
    }
}

/// Itemized breakdown of the minimum floor price.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct FloorPriceBreakdown {
    pub base_overhead_eur: f64,
    pub screens_cost_eur: f64,
    pub entities_cost_eur: f64,
    pub triggers_cost_eur: f64,
    pub hardware_cost_eur: f64,
    pub total_floor_price_eur: f64,
}

/// Automated escrow splits across the 6 specialist roles plus platform fee.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct RoleEscrowSplit {
    pub gross_budget_eur: f64,
    pub cs_share_eur: f64,     // 10%
    pub ba_share_eur: f64,     // 15%
    pub da_share_eur: f64,     // 18%
    pub pcd_share_eur: f64,    // 22%
    pub pcss_share_eur: f64,   // 15%
    pub pcds_share_eur: f64,   // 10%
    pub platform_fee_eur: f64, // 10%
}

impl RoleEscrowSplit {
    pub const CS_PCT: f64 = 0.10;
    pub const BA_PCT: f64 = 0.15;
    pub const DA_PCT: f64 = 0.18;
    pub const PCD_PCT: f64 = 0.22;
    pub const PCSS_PCT: f64 = 0.15;
    pub const PCDS_PCT: f64 = 0.10;
    pub const PLATFORM_PCT: f64 = 0.10;

    pub fn calculate(gross_budget: f64) -> Self {
        let round_cents = |val: f64| (val * 100.0).round() / 100.0;
        Self {
            gross_budget_eur: gross_budget,
            cs_share_eur: round_cents(gross_budget * Self::CS_PCT),
            ba_share_eur: round_cents(gross_budget * Self::BA_PCT),
            da_share_eur: round_cents(gross_budget * Self::DA_PCT),
            pcd_share_eur: round_cents(gross_budget * Self::PCD_PCT),
            pcss_share_eur: round_cents(gross_budget * Self::PCSS_PCT),
            pcds_share_eur: round_cents(gross_budget * Self::PCDS_PCT),
            platform_fee_eur: round_cents(gross_budget * Self::PLATFORM_PCT),
        }
    }

    /// Sum of all role specialist shares (excluding platform take).
    pub fn total_specialist_payout(&self) -> f64 {
        let sum = self.cs_share_eur
            + self.ba_share_eur
            + self.da_share_eur
            + self.pcd_share_eur
            + self.pcss_share_eur
            + self.pcds_share_eur;
        (sum * 100.0).round() / 100.0
    }
}

/// Computes the itemized floor price for given complexity metrics.
pub fn compute_floor_price(metrics: &ProjectComplexityMetrics) -> FloorPriceBreakdown {
    compute_floor_price_with_config(metrics, &ComplexityFloorConfig::default())
}

/// Computes floor price using customized configuration parameters.
pub fn compute_floor_price_with_config(
    metrics: &ProjectComplexityMetrics,
    config: &ComplexityFloorConfig,
) -> FloorPriceBreakdown {
    let screens_cost = metrics.screens_count as f64 * config.per_screen_eur;
    let entities_cost = metrics.entities_count as f64 * config.per_entity_eur;
    let triggers_cost = metrics.triggers_count as f64 * config.per_trigger_eur;
    let hardware_cost = metrics.hardware_peripherals_count as f64 * config.per_hardware_eur;
    let total = config.base_overhead_eur + screens_cost + entities_cost + triggers_cost + hardware_cost;

    FloorPriceBreakdown {
        base_overhead_eur: config.base_overhead_eur,
        screens_cost_eur: screens_cost,
        entities_cost_eur: entities_cost,
        triggers_cost_eur: triggers_cost,
        hardware_cost_eur: hardware_cost,
        total_floor_price_eur: (total * 100.0).round() / 100.0,
    }
}

/// Validates that a proposed budget satisfies the minimum algorithmic floor.
pub fn validate_project_budget(
    metrics: &ProjectComplexityMetrics,
    proposed_budget_eur: f64,
) -> Result<RoleEscrowSplit, FloorPricingError> {
    let breakdown = compute_floor_price(metrics);
    if proposed_budget_eur < breakdown.total_floor_price_eur {
        let deficit = ((breakdown.total_floor_price_eur - proposed_budget_eur) * 100.0).round() / 100.0;
        return Err(FloorPricingError::BudgetBelowFloor {
            proposed: proposed_budget_eur,
            minimum_floor: breakdown.total_floor_price_eur,
            deficit,
        });
    }

    Ok(RoleEscrowSplit::calculate(proposed_budget_eur))
}

/// Status of an individual role slot in a Project Slot Board.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SlotStatus {
    Open,
    BidPending,
    Assigned,
    InProgress,
    CompletedVerified,
}

/// A bid submitted by a specialist for a role slot.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SlotBid {
    pub specialist_id: String,
    pub specialist_name: String,
    pub proposed_amount_eur: f64,
    pub estimated_days: u32,
    pub notes: String,
    pub submitted_at: String,
}

/// An individual role slot within a Project Slot Board.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProjectRoleSlot {
    pub role: UserRole,
    pub title: String,
    pub escrow_amount_eur: f64,
    pub assigned_specialist_id: Option<String>,
    pub status: SlotStatus,
    pub instant_accept_available: bool,
    pub bids: Vec<SlotBid>,
}

/// Complete 6-slot board for SME projects.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProjectSlotBoard {
    pub project_id: String,
    pub title: String,
    pub client_business_name: String,
    pub total_budget_eur: f64,
    pub metrics: ProjectComplexityMetrics,
    pub floor_breakdown: FloorPriceBreakdown,
    pub escrow_split: RoleEscrowSplit,
    pub slots: Vec<ProjectRoleSlot>,
    pub created_at: String,
}

impl ProjectSlotBoard {
    pub fn new(
        project_id: impl Into<String>,
        title: impl Into<String>,
        client_name: impl Into<String>,
        total_budget: f64,
        metrics: ProjectComplexityMetrics,
        created_at: impl Into<String>,
    ) -> Result<Self, FloorPricingError> {
        let escrow_split = validate_project_budget(&metrics, total_budget)?;
        let floor_breakdown = compute_floor_price(&metrics);

        let slots = vec![
            ProjectRoleSlot {
                role: UserRole::CustomerService,
                title: "Customer Support & Intake (CS)".to_string(),
                escrow_amount_eur: escrow_split.cs_share_eur,
                assigned_specialist_id: None,
                status: SlotStatus::Open,
                instant_accept_available: true,
                bids: Vec::new(),
            },
            ProjectRoleSlot {
                role: UserRole::BusinessAnalyst,
                title: "Business Analyst (PCBA)".to_string(),
                escrow_amount_eur: escrow_split.ba_share_eur,
                assigned_specialist_id: None,
                status: SlotStatus::Open,
                instant_accept_available: true,
                bids: Vec::new(),
            },
            ProjectRoleSlot {
                role: UserRole::DataAnalyst,
                title: "Data Analyst & DB Architect (PCDA)".to_string(),
                escrow_amount_eur: escrow_split.da_share_eur,
                assigned_specialist_id: None,
                status: SlotStatus::Open,
                instant_accept_available: true,
                bids: Vec::new(),
            },
            ProjectRoleSlot {
                role: UserRole::Developer,
                title: "UI/UX Designer (PCD)".to_string(),
                escrow_amount_eur: escrow_split.pcd_share_eur,
                assigned_specialist_id: None,
                status: SlotStatus::Open,
                instant_accept_available: true,
                bids: Vec::new(),
            },
            ProjectRoleSlot {
                role: UserRole::SalesConsultant,
                title: "IT & Systems Specialist (PCSS)".to_string(),
                escrow_amount_eur: escrow_split.pcss_share_eur,
                assigned_specialist_id: None,
                status: SlotStatus::Open,
                instant_accept_available: true,
                bids: Vec::new(),
            },
            ProjectRoleSlot {
                role: UserRole::Technician,
                title: "Field Support & Deployer (PCDS)".to_string(),
                escrow_amount_eur: escrow_split.pcds_share_eur,
                assigned_specialist_id: None,
                status: SlotStatus::Open,
                instant_accept_available: true,
                bids: Vec::new(),
            },
        ];

        Ok(Self {
            project_id: project_id.into(),
            title: title.into(),
            client_business_name: client_name.into(),
            total_budget_eur: total_budget,
            metrics,
            floor_breakdown,
            escrow_split,
            slots,
            created_at: created_at.into(),
        })
    }

    /// Instant accept an open slot.
    pub fn instant_accept_slot(&mut self, role: UserRole, specialist_id: impl Into<String>) -> bool {
        if let Some(slot) = self.slots.iter_mut().find(|s| s.role == role) {
            if slot.status == SlotStatus::Open && slot.instant_accept_available {
                slot.assigned_specialist_id = Some(specialist_id.into());
                slot.status = SlotStatus::Assigned;
                return true;
            }
        }
        false
    }

    /// Submit a bid for a role slot.
    pub fn submit_bid(&mut self, role: UserRole, bid: SlotBid) -> bool {
        if let Some(slot) = self.slots.iter_mut().find(|s| s.role == role) {
            if slot.status == SlotStatus::Open || slot.status == SlotStatus::BidPending {
                slot.status = SlotStatus::BidPending;
                slot.bids.push(bid);
                return true;
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_floor_price_formula() {
        // Base: 150 + (3 * 45) + (2 * 35) + (4 * 25) + (1 * 60)
        // = 150 + 135 + 70 + 100 + 60 = 515.00
        let metrics = ProjectComplexityMetrics::new(3, 2, 4, 1);
        let breakdown = compute_floor_price(&metrics);

        assert_eq!(breakdown.base_overhead_eur, 150.0);
        assert_eq!(breakdown.screens_cost_eur, 135.0);
        assert_eq!(breakdown.entities_cost_eur, 70.0);
        assert_eq!(breakdown.triggers_cost_eur, 100.0);
        assert_eq!(breakdown.hardware_cost_eur, 60.0);
        assert_eq!(breakdown.total_floor_price_eur, 515.0);
    }

    #[test]
    fn test_budget_validation_rejection_below_floor() {
        let metrics = ProjectComplexityMetrics::new(3, 2, 4, 1); // floor is 515.00
        let res = validate_project_budget(&metrics, 300.0);

        match res {
            Err(FloorPricingError::BudgetBelowFloor { proposed, minimum_floor, deficit }) => {
                assert_eq!(proposed, 300.0);
                assert_eq!(minimum_floor, 515.0);
                assert_eq!(deficit, 215.0);
            }
            _ => panic!("Expected BudgetBelowFloor error"),
        }
    }

    #[test]
    fn test_budget_validation_success_and_splits() {
        let metrics = ProjectComplexityMetrics::new(2, 1, 1, 0);
        // Base 150 + (2*45=90) + (1*35=35) + (1*25=25) + 0 = 300.00
        let res = validate_project_budget(&metrics, 600.0);
        assert!(res.is_ok());

        let splits = res.unwrap();
        assert_eq!(splits.gross_budget_eur, 600.0);
        assert_eq!(splits.cs_share_eur, 60.0);     // 10%
        assert_eq!(splits.ba_share_eur, 90.0);     // 15%
        assert_eq!(splits.da_share_eur, 108.0);    // 18%
        assert_eq!(splits.pcd_share_eur, 132.0);   // 22%
        assert_eq!(splits.pcss_share_eur, 90.0);   // 15%
        assert_eq!(splits.pcds_share_eur, 60.0);   // 10%
        assert_eq!(splits.platform_fee_eur, 60.0); // 10%
        assert_eq!(splits.total_specialist_payout(), 540.0);
    }

    #[test]
    fn test_project_slot_board_lifecycle() {
        let metrics = ProjectComplexityMetrics::new(2, 1, 1, 0); // floor is 300
        let mut board = ProjectSlotBoard::new(
            "proj_sme_01",
            "AutoService CRM",
            "Speedy Garage Ltd",
            600.0,
            metrics,
            "2026-09-24T10:00:00Z",
        ).expect("Valid project slot board");

        assert_eq!(board.slots.len(), 6);
        assert_eq!(board.slots[0].role, UserRole::CustomerService);
        assert_eq!(board.slots[0].escrow_amount_eur, 60.0);
        assert_eq!(board.slots[0].status, SlotStatus::Open);

        // Instant accept DA slot
        let accepted = board.instant_accept_slot(UserRole::DataAnalyst, "specialist_eleni");
        assert!(accepted);
        let da_slot = board.slots.iter().find(|s| s.role == UserRole::DataAnalyst).unwrap();
        assert_eq!(da_slot.status, SlotStatus::Assigned);
        assert_eq!(da_slot.assigned_specialist_id.as_deref(), Some("specialist_eleni"));

        // Submit bid for PCD slot
        let bid = SlotBid {
            specialist_id: "freelancer_alex".to_string(),
            specialist_name: "Alex Designer".to_string(),
            proposed_amount_eur: 132.0,
            estimated_days: 3,
            notes: "Ready to start immediately with Penpot tokens".to_string(),
            submitted_at: "2026-09-24T10:15:00Z".to_string(),
        };
        let bid_ok = board.submit_bid(UserRole::Developer, bid);
        assert!(bid_ok);

        let pcd_slot = board.slots.iter().find(|s| s.role == UserRole::Developer).unwrap();
        assert_eq!(pcd_slot.status, SlotStatus::BidPending);
        assert_eq!(pcd_slot.bids.len(), 1);
    }
}
