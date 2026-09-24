//! Project Slot Board & Bidding Engine for Proteus Web Marketplace.
//! Supports:
//! - Community freelance project boards for budget-constrained SMEs.
//! - Instant Accept offers & competitive bidding on 6 specialist role slots.
//! - Algorithmic floor price validation against under-the-table evasion.

use std::sync::{Arc, Mutex};
use crm_core::pricing::{
    compute_floor_price, validate_project_budget, FloorPriceBreakdown,
    ProjectComplexityMetrics, ProjectSlotBoard, RoleEscrowSplit, SlotBid,
};
use crm_core::roles::UserRole;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcceptSlotRequest {
    pub project_id: String,
    pub role: UserRole,
    pub specialist_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitBidRequest {
    pub project_id: String,
    pub role: UserRole,
    pub specialist_id: String,
    pub specialist_name: String,
    pub proposed_amount_eur: f64,
    pub estimated_days: u32,
    pub notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FloorCalculatorRequest {
    pub metrics: ProjectComplexityMetrics,
    pub proposed_budget_eur: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FloorCalculatorResponse {
    pub breakdown: FloorPriceBreakdown,
    pub escrow_split: Option<RoleEscrowSplit>,
    pub is_valid: bool,
    pub message: String,
}

/// Global in-memory storage for active SME Slot Boards.
#[derive(Clone)]
pub struct SlotBoardManager {
    boards: Arc<Mutex<Vec<ProjectSlotBoard>>>,
}

impl Default for SlotBoardManager {
    fn default() -> Self {
        Self::new()
    }
}

impl SlotBoardManager {
    pub fn new() -> Self {
        let default_boards = vec![
            ProjectSlotBoard::new(
                "SB-GARAGE-01",
                "Speedy Garage — Fast Automotive Intake & Thermal Print",
                "Speedy Garage Ltd (Περιστέρι)",
                600.0,
                ProjectComplexityMetrics::new(2, 2, 1, 1),
                "2026-09-24T08:00:00Z",
            ).expect("Valid garage board"),
            ProjectSlotBoard::new(
                "SB-BAKERY-02",
                "Artisan Bakery — Dual Touch POS & Cash Drawer Kick",
                "Artisan Bakery & Sweets (Κηφισιά)",
                480.0,
                ProjectComplexityMetrics::new(1, 1, 1, 1),
                "2026-09-24T08:30:00Z",
            ).expect("Valid bakery board"),
            ProjectSlotBoard::new(
                "SB-DENTAL-03",
                "Dental Care Pro — Patient File & Multi-Doctor Scheduling",
                "Dental Care Clinic (Μαρούσι)",
                750.0,
                ProjectComplexityMetrics::new(3, 2, 2, 0),
                "2026-09-24T09:00:00Z",
            ).expect("Valid clinic board"),
        ];

        Self {
            boards: Arc::new(Mutex::new(default_boards)),
        }
    }

    pub fn list_boards(&self) -> Vec<ProjectSlotBoard> {
        let lock = self.boards.lock().unwrap_or_else(|e| e.into_inner());
        lock.clone()
    }

    pub fn get_board(&self, project_id: &str) -> Option<ProjectSlotBoard> {
        let lock = self.boards.lock().unwrap_or_else(|e| e.into_inner());
        lock.iter().find(|b| b.project_id == project_id).cloned()
    }

    pub fn accept_slot(&self, req: &AcceptSlotRequest) -> Result<ProjectSlotBoard, String> {
        let mut lock = self.boards.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(board) = lock.iter_mut().find(|b| b.project_id == req.project_id) {
            if board.instant_accept_slot(req.role, &req.specialist_id) {
                return Ok(board.clone());
            }
            return Err("Η θέση δεν είναι διαθέσιμη για άμεση αποδοχή (είναι ήδη κατειλημμένη ή υπό αξιολόγηση)".to_string());
        }
        Err(format!("Το Project Slot Board '{}' δεν βρέθηκε", req.project_id))
    }

    pub fn submit_bid(&self, req: &SubmitBidRequest) -> Result<ProjectSlotBoard, String> {
        let mut lock = self.boards.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(board) = lock.iter_mut().find(|b| b.project_id == req.project_id) {
            let bid = SlotBid {
                specialist_id: req.specialist_id.clone(),
                specialist_name: req.specialist_name.clone(),
                proposed_amount_eur: req.proposed_amount_eur,
                estimated_days: req.estimated_days,
                notes: req.notes.clone(),
                submitted_at: chrono::Utc::now().to_rfc3339(),
            };

            if board.submit_bid(req.role, bid) {
                return Ok(board.clone());
            }
            return Err("Αδυναμία υποβολής προσφοράς για τη συγκεκριμένη θέση".to_string());
        }
        Err(format!("Το Project Slot Board '{}' δεν βρέθηκε", req.project_id))
    }

    pub fn evaluate_floor(req: &FloorCalculatorRequest) -> FloorCalculatorResponse {
        let breakdown = compute_floor_price(&req.metrics);

        if let Some(budget) = req.proposed_budget_eur {
            match validate_project_budget(&req.metrics, budget) {
                Ok(split) => FloorCalculatorResponse {
                    breakdown,
                    escrow_split: Some(split),
                    is_valid: true,
                    message: format!("✓ Έγκυρο Budget (€{:.2}). Ικανοποιεί το αλγοριθμικό πάτωμα (€{:.2}).", budget, breakdown.total_floor_price_eur),
                },
                Err(err) => FloorCalculatorResponse {
                    breakdown,
                    escrow_split: None,
                    is_valid: false,
                    message: format!("✗ {}", err),
                },
            }
        } else {
            FloorCalculatorResponse {
                breakdown,
                escrow_split: None,
                is_valid: true,
                message: format!("Ελάχιστο αλγοριθμικό πάτωμα: €{:.2}", breakdown.total_floor_price_eur),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slot_board_manager_initialization() {
        let manager = SlotBoardManager::new();
        let boards = manager.list_boards();
        assert_eq!(boards.len(), 3);
        assert_eq!(boards[0].project_id, "SB-GARAGE-01");
        assert_eq!(boards[1].project_id, "SB-BAKERY-02");
        assert_eq!(boards[2].project_id, "SB-DENTAL-03");
    }

    #[test]
    fn test_slot_board_instant_accept() {
        let manager = SlotBoardManager::new();
        let req = AcceptSlotRequest {
            project_id: "SB-GARAGE-01".to_string(),
            role: UserRole::Technician, // Field Support Deployer
            specialist_id: "spec_technician_01".to_string(),
        };

        let res = manager.accept_slot(&req);
        assert!(res.is_ok());

        let board = res.unwrap();
        let tech_slot = board.slots.iter().find(|s| s.role == UserRole::Technician).unwrap();
        assert_eq!(tech_slot.status, crm_core::pricing::SlotStatus::Assigned);
        assert_eq!(tech_slot.assigned_specialist_id.as_deref(), Some("spec_technician_01"));

        // Second accept on same slot should fail
        let duplicate = manager.accept_slot(&req);
        assert!(duplicate.is_err());
    }

    #[test]
    fn test_slot_board_submit_bid() {
        let manager = SlotBoardManager::new();
        let req = SubmitBidRequest {
            project_id: "SB-BAKERY-02".to_string(),
            role: UserRole::Developer, // PCD Designer
            specialist_id: "spec_pcd_alex".to_string(),
            specialist_name: "Alex Papadopoulos".to_string(),
            proposed_amount_eur: 110.0,
            estimated_days: 2,
            notes: "Ready to deploy with customized dark bakery theme".to_string(),
        };

        let res = manager.submit_bid(&req);
        assert!(res.is_ok());

        let board = res.unwrap();
        let pcd_slot = board.slots.iter().find(|s| s.role == UserRole::Developer).unwrap();
        assert_eq!(pcd_slot.status, crm_core::pricing::SlotStatus::BidPending);
        assert_eq!(pcd_slot.bids.len(), 1);
        assert_eq!(pcd_slot.bids[0].specialist_name, "Alex Papadopoulos");
    }

    #[test]
    fn test_evaluate_floor_api() {
        let req_valid = FloorCalculatorRequest {
            metrics: ProjectComplexityMetrics::new(2, 2, 1, 1),
            proposed_budget_eur: Some(500.0), // floor is 395
        };
        let res_valid = SlotBoardManager::evaluate_floor(&req_valid);
        assert!(res_valid.is_valid);
        assert!(res_valid.escrow_split.is_some());

        let req_below_floor = FloorCalculatorRequest {
            metrics: ProjectComplexityMetrics::new(2, 2, 1, 1),
            proposed_budget_eur: Some(250.0), // floor is 395
        };
        let res_below = SlotBoardManager::evaluate_floor(&req_below_floor);
        assert!(!res_below.is_valid);
        assert!(res_below.escrow_split.is_none());
    }
}
