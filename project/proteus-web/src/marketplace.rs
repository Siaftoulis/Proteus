//! Proteus Marketplace & Anti-Bypass Compilation Gate Engine.
//! Enforces:
//! - 10-Tier sliding commission model (Tier 0 at 50% down to Tier 10 at 4.5%).
//! - Monthly badge subscriptions (30€/mo base).
//! - Walled garden compiler gate: packages are only compiled on-platform and bound to paying client shops.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CertificationTier {
    Tier0Uncertified,
    Tier1Associate,
    Tier2Specialist,
    Tier3Professional,
    Tier4Practitioner,
    Tier5Architect,
    Tier6Principal,
    Tier7MasterEngineer,
    Tier8PremierPartner,
    Tier9EliteAgency,
    Tier10EnterpriseAgency,
}

impl CertificationTier {
    /// Platform take-rate (commission) as a percentage (0.0 to 100.0).
    pub fn platform_commission_pct(&self) -> f64 {
        match self {
            Self::Tier0Uncertified => 50.0,
            Self::Tier1Associate => 40.0,
            Self::Tier2Specialist => 35.0,
            Self::Tier3Professional => 30.0,
            Self::Tier4Practitioner => 25.0,
            Self::Tier5Architect => 20.0,
            Self::Tier6Principal => 16.0,
            Self::Tier7MasterEngineer => 12.0,
            Self::Tier8PremierPartner => 10.0,
            Self::Tier9EliteAgency => 7.0,
            Self::Tier10EnterpriseAgency => 4.5, // 4% - 5% stripe/MoR cost only
        }
    }

    /// Partner net share percentage (0.0 to 100.0).
    pub fn partner_share_pct(&self) -> f64 {
        100.0 - self.platform_commission_pct()
    }

    /// Monthly badge subscription fee in Euros.
    pub fn monthly_badge_fee(&self) -> f64 {
        match self {
            Self::Tier0Uncertified => 0.0,
            Self::Tier1Associate
            | Self::Tier2Specialist
            | Self::Tier3Professional
            | Self::Tier4Practitioner
            | Self::Tier5Architect => 30.0,
            Self::Tier6Principal => 45.0,
            Self::Tier7MasterEngineer => 60.0,
            Self::Tier8PremierPartner => 80.0,
            Self::Tier9EliteAgency => 110.0,
            Self::Tier10EnterpriseAgency => 150.0,
        }
    }

    /// Calculate payout split for a transaction.
    pub fn calculate_payout(&self, gross_amount: f64) -> (f64, f64) {
        let platform_take = (gross_amount * (self.platform_commission_pct() / 100.0) * 100.0).round() / 100.0;
        let partner_net = ((gross_amount - platform_take) * 100.0).round() / 100.0;
        (platform_take, partner_net)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageCompileRequest {
    pub designer_id: String,
    pub designer_tier: CertificationTier,
    pub project_name: String,
    pub target_client_shop_id: String,
    pub gross_amount_eur: f64,
    pub is_escrow_funded: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageCompileResult {
    pub success: bool,
    pub package_hash: Option<String>,
    pub platform_fee_eur: f64,
    pub partner_payout_eur: f64,
    pub message: String,
}

/// Gated compilation: packages can ONLY be compiled if bound to a valid client shop
/// and protected by in-platform escrow. Local compiler export is blocked.
pub fn compile_package_gate(req: &PackageCompileRequest) -> PackageCompileResult {
    if !req.is_escrow_funded {
        return PackageCompileResult {
            success: false,
            package_hash: None,
            platform_fee_eur: 0.0,
            partner_payout_eur: 0.0,
            message: "Αποτυχία: Η πληρωμή δεν έχει δεσμευτεί στο In-Platform Escrow. Το compilation απορρίφθηκε.".to_string(),
        };
    }

    if req.target_client_shop_id.trim().is_empty() {
        return PackageCompileResult {
            success: false,
            package_hash: None,
            platform_fee_eur: 0.0,
            partner_payout_eur: 0.0,
            message: "Αποτυχία: Δεν δηλώθηκε εξουσιοδοτημένο κατάστημα-παραλήπτης.".to_string(),
        };
    }

    let (platform_fee, partner_payout) = req.designer_tier.calculate_payout(req.gross_amount_eur);
    let hash = format!("PR-PKG-{:x}", req.gross_amount_eur.to_bits() ^ 0x5A5A5A5A);

    PackageCompileResult {
        success: true,
        package_hash: Some(hash),
        platform_fee_eur: platform_fee,
        partner_payout_eur: partner_payout,
        message: format!(
            "Επιτυχές compilation: Πακέτο κρυπτογραφημένο για {}. Προμήθεια: {:.2}€ ({:.1}%), Καθαρά Συνεργάτη: {:.2}€.",
            req.target_client_shop_id,
            platform_fee,
            req.designer_tier.platform_commission_pct(),
            partner_payout
        ),
    }
}

/// Official Proteus Professional Certification tracks.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CertificationTrack {
    PcdDesigner,           // Proteus Certified Designer
    PcssSystemsDb,         // Proteus Certified Systems & DB Specialist
    PcdsDeployerSupport,   // Proteus Certified Deployer / Support Specialist
    PcdaBusinessAnalyst,   // Proteus Certified Data/Business Analyst
    AllInOneBundle,        // Master Bundle (All 4 Certifications)
}

#[allow(dead_code)]
impl CertificationTrack {
    pub fn exam_fee_eur(&self) -> f64 {
        match self {
            Self::PcdDesigner | Self::PcssSystemsDb | Self::PcdsDeployerSupport | Self::PcdaBusinessAnalyst => 79.0,
            Self::AllInOneBundle => 149.0,
        }
    }

    pub fn annual_badge_fee_eur(&self) -> f64 {
        39.0
    }

    pub fn title(&self) -> &'static str {
        match self {
            Self::PcdDesigner => "PCD — Proteus Certified Designer",
            Self::PcssSystemsDb => "PCSS — Proteus Certified Systems & DB Specialist",
            Self::PcdsDeployerSupport => "PCDS — Proteus Certified Deployer / Support Specialist",
            Self::PcdaBusinessAnalyst => "PCDA — Proteus Certified Data/Business Analyst",
            Self::AllInOneBundle => "Proteus Master Bundle (PCD + PCSS + PCDS + PCDA)",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tier_commission_scale() {
        assert_eq!(CertificationTier::Tier0Uncertified.platform_commission_pct(), 50.0);
        assert_eq!(CertificationTier::Tier1Associate.platform_commission_pct(), 40.0);
        assert_eq!(CertificationTier::Tier2Specialist.platform_commission_pct(), 35.0);
        assert_eq!(CertificationTier::Tier3Professional.platform_commission_pct(), 30.0);
        assert_eq!(CertificationTier::Tier10EnterpriseAgency.platform_commission_pct(), 4.5);
    }

    #[test]
    fn test_payout_calculation() {
        let gig_amount = 350.0;
        // Tier 0: 50%
        let (take_0, net_0) = CertificationTier::Tier0Uncertified.calculate_payout(gig_amount);
        assert_eq!(take_0, 175.0);
        assert_eq!(net_0, 175.0);

        // Tier 2: 35% -> 122.50 take, 227.50 net
        let (take_2, net_2) = CertificationTier::Tier2Specialist.calculate_payout(gig_amount);
        assert_eq!(take_2, 122.50);
        assert_eq!(net_2, 227.50);

        // Tier 10: 4.5% -> 15.75 take, 334.25 net
        let (take_10, net_10) = CertificationTier::Tier10EnterpriseAgency.calculate_payout(gig_amount);
        assert_eq!(take_10, 15.75);
        assert_eq!(net_10, 334.25);
    }

    #[test]
    fn test_compiler_gate_escrow_check() {
        let req_unfunded = PackageCompileRequest {
            designer_id: "user_01".to_string(),
            designer_tier: CertificationTier::Tier2Specialist,
            project_name: "RepairPOS".to_string(),
            target_client_shop_id: "SHOP_01".to_string(),
            gross_amount_eur: 350.0,
            is_escrow_funded: false,
        };
        let res = compile_package_gate(&req_unfunded);
        assert!(!res.success);

        let req_funded = PackageCompileRequest {
            is_escrow_funded: true,
            ..req_unfunded
        };
        let res_ok = compile_package_gate(&req_funded);
        assert!(res_ok.success);
        assert_eq!(res_ok.platform_fee_eur, 122.50);
        assert_eq!(res_ok.partner_payout_eur, 227.50);
    }

    #[test]
    fn test_certification_tracks_pricing() {
        assert_eq!(CertificationTrack::PcdDesigner.exam_fee_eur(), 79.0);
        assert_eq!(CertificationTrack::PcssSystemsDb.exam_fee_eur(), 79.0);
        assert_eq!(CertificationTrack::PcdsDeployerSupport.exam_fee_eur(), 79.0);
        assert_eq!(CertificationTrack::PcdaBusinessAnalyst.exam_fee_eur(), 79.0);
        assert_eq!(CertificationTrack::AllInOneBundle.exam_fee_eur(), 149.0);
        assert_eq!(CertificationTrack::PcdaBusinessAnalyst.annual_badge_fee_eur(), 39.0);
    }
}
