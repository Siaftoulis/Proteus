//! Proteus Platform Legal Shield & In-Platform Digital Contracts.
//! Ensures mutually binding SLA agreements, user/technician safety, and Escrow fund holding.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContractStatus {
    PendingSignatures,
    EscrowFunded,
    Active,
    Completed,
    Disputed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlaContract {
    pub contract_id: String,
    pub shop_id: String,
    pub technician_id: String,
    pub service_scope: String,
    pub response_time_hours: u32,
    pub monthly_retainer_eur: f64,
    pub escrow_balance_eur: f64,
    pub signed_by_shop: bool,
    pub signed_by_technician: bool,
    pub status: ContractStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateContractRequest {
    pub contract_id: String,
    pub shop_id: String,
    pub technician_id: String,
    pub service_scope: String,
    pub response_time_hours: u32,
    pub monthly_retainer_eur: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignContractRequest {
    pub contract: SlaContract,
    pub signer_role: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FundContractRequest {
    pub contract: SlaContract,
    pub amount: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayoutContractRequest {
    pub contract: SlaContract,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayoutContractResponse {
    pub contract: SlaContract,
    pub payout_eur: f64,
}

impl SlaContract {
    pub fn new(
        contract_id: String,
        shop_id: String,
        technician_id: String,
        service_scope: String,
        response_time_hours: u32,
        monthly_retainer_eur: f64,
    ) -> Self {
        Self {
            contract_id,
            shop_id,
            technician_id,
            service_scope,
            response_time_hours,
            monthly_retainer_eur,
            escrow_balance_eur: 0.0,
            signed_by_shop: false,
            signed_by_technician: false,
            status: ContractStatus::PendingSignatures,
        }
    }

    pub fn sign_by_shop(&mut self) {
        self.signed_by_shop = true;
        self.check_activation();
    }

    pub fn sign_by_technician(&mut self) {
        self.signed_by_technician = true;
        self.check_activation();
    }

    pub fn fund_escrow(&mut self, amount: f64) {
        self.escrow_balance_eur += amount;
        self.check_activation();
    }

    fn check_activation(&mut self) {
        if self.signed_by_shop && self.signed_by_technician && self.escrow_balance_eur >= self.monthly_retainer_eur {
            self.status = ContractStatus::Active;
        } else if self.signed_by_shop && self.signed_by_technician {
            self.status = ContractStatus::EscrowFunded;
        }
    }

    pub fn release_escrow_payout(&mut self) -> Result<f64, &'static str> {
        if self.status != ContractStatus::Active && self.status != ContractStatus::Completed {
            return Err("Δεν είναι δυνατή η αποδέσμευση: Το συμβόλαιο δεν είναι ενεργό ή ολοκληρωμένο.");
        }
        let payout = self.escrow_balance_eur;
        self.escrow_balance_eur = 0.0;
        self.status = ContractStatus::Completed;
        Ok(payout)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contract_signing_and_escrow_flow() {
        let mut contract = SlaContract::new(
            "CTR-101".to_string(),
            "SHOP-ALPHA".to_string(),
            "TECH-ATHENS".to_string(),
            "IT Support & Hardware Diagnostics".to_string(),
            2,
            80.0,
        );

        assert_eq!(contract.status, ContractStatus::PendingSignatures);

        contract.sign_by_shop();
        assert_eq!(contract.status, ContractStatus::PendingSignatures);

        contract.sign_by_technician();
        assert_eq!(contract.status, ContractStatus::EscrowFunded);

        // Fund escrow
        contract.fund_escrow(80.0);
        assert_eq!(contract.status, ContractStatus::Active);

        // Release payout
        let payout = contract.release_escrow_payout().expect("payout success");
        assert_eq!(payout, 80.0);
        assert_eq!(contract.status, ContractStatus::Completed);
        assert_eq!(contract.escrow_balance_eur, 0.0);
    }
}
