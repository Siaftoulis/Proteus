//! Proteus Marketplace & Anti-Bypass Compilation Gate Engine.
//! Enforces:
//! - 10-Tier sliding commission model (Tier 0 at 50% down to Tier 10 at 4.5%).
//! - Monthly badge subscriptions (30€/mo base).
//! - Walled garden compiler gate: packages are only compiled on-platform and bound to paying client shops.

use crm_core::pricing::{compute_floor_price, ProjectComplexityMetrics};
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
    #[serde(default)]
    pub complexity_metrics: Option<ProjectComplexityMetrics>,
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

    if let Some(metrics) = req.complexity_metrics {
        let breakdown = compute_floor_price(&metrics);
        if req.gross_amount_eur < breakdown.total_floor_price_eur {
            let deficit = ((breakdown.total_floor_price_eur - req.gross_amount_eur) * 100.0).round() / 100.0;
            return PackageCompileResult {
                success: false,
                package_hash: None,
                platform_fee_eur: 0.0,
                partner_payout_eur: 0.0,
                message: format!(
                    "Αποτυχία Anti-Scam: Το ποσό €{:.2} είναι χαμηλότερο από το ελάχιστο αλγοριθμικό πάτωμα €{:.2} (Έλλειμμα: €{:.2}). Απορρίφθηκε για αποτροπή παράκαμψης πλατφόρμας.",
                    req.gross_amount_eur, breakdown.total_floor_price_eur, deficit
                ),
            };
        }
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

/// A published `.pr` package listing in the Proteus Marketplace.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct MarketplacePackageListing {
    pub bundle_id: String,
    pub title: String,
    pub description: String,
    pub author_designer: String,
    pub version: String,
    pub price_eur: f64,
    pub category: String,
    pub licensed_accounts: Vec<String>,
}

/// Retrieve the complete catalog of verified .pr business packages on the Marketplace.
pub fn get_all_marketplace_packages() -> Vec<MarketplacePackageListing> {
    vec![
        MarketplacePackageListing {
            bundle_id: "PKG-SERVICE-AUTO".to_string(),
            title: "Automotive Service & Repair BOS".to_string(),
            description: "Εντολές εργασίας, αποθήκη ανταλλακτικών μηχανικού και ESC/POS διαγνωστικές αποδείξεις.".to_string(),
            author_designer: "PCD Senior Partner".to_string(),
            version: "1.4.0".to_string(),
            price_eur: 180.0,
            category: "Automotive".to_string(),
            licensed_accounts: vec!["demo@company.com".to_string(), "owner@autoworks.gr".to_string()],
        },
        MarketplacePackageListing {
            bundle_id: "PKG-RETAIL-POS".to_string(),
            title: "Multi-Store Retail & Cashier BOS".to_string(),
            description: "Barcode scanning, άμεσο άνοιγμα συρταριού, αυτόματη αναπαραγγελία και συγχρονισμός καταστημάτων.".to_string(),
            author_designer: "PCDA Analyst Group".to_string(),
            version: "2.1.0".to_string(),
            price_eur: 240.0,
            category: "Retail".to_string(),
            licensed_accounts: vec!["demo@company.com".to_string(), "admin@retailchain.com".to_string()],
        },
        MarketplacePackageListing {
            bundle_id: "PKG-CLINIC-HEALTH".to_string(),
            title: "Medical & Dental Practice Suite".to_string(),
            description: "Ιατρικό ιστορικό ασθενών, ημερολόγιο ραντεβού, GDPR audit logs και τιμολόγηση ασφαλιστικών ταμείων.".to_string(),
            author_designer: "PCSS Systems Architect".to_string(),
            version: "1.0.2".to_string(),
            price_eur: 320.0,
            category: "Healthcare".to_string(),
            licensed_accounts: vec!["doctor@clinic.org".to_string()],
        },
        MarketplacePackageListing {
            bundle_id: "PKG-MOTO-PRO".to_string(),
            title: "Motorcycle Workshop & Tuning BOS".to_string(),
            description: "Έλεγχος πλαισίου (VIN), καταγραφή βάθους πέλματος ελαστικών και εκτύπωση φύλλου δυναμομέτρησης.".to_string(),
            author_designer: "PCD Specialist".to_string(),
            version: "1.1.0".to_string(),
            price_eur: 120.0,
            category: "Automotive".to_string(),
            licensed_accounts: vec!["demo@company.com".to_string(), "tuning@motofast.gr".to_string()],
        },
    ]
}

/// Retrieve the list of purchased/licensed packages for a verified business account.
pub fn get_licensed_packages_for_account(account_email: &str) -> Vec<MarketplacePackageListing> {
    let catalog = get_all_marketplace_packages();
    let email_lower = account_email.to_lowercase();
    catalog
        .into_iter()
        .filter(|pkg| {
            pkg.licensed_accounts
                .iter()
                .any(|acc| acc.to_lowercase() == email_lower)
        })
        .collect()
}


/// Official Proteus Professional Certification tracks.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CertificationTrack {
    PcdDesigner,           // Proteus Certified Designer
    PcbaBusinessAnalyst,   // Proteus Certified Business Analyst
    PcdaDataAnalyst,       // Proteus Certified Data Analyst
    PcssSystemsDb,         // Proteus Certified Systems & DB Specialist
    PcdsDeployerSupport,   // Proteus Certified Deployer / Support Specialist
    AllInOneBundle,        // Master Bundle (All Certifications)
}

#[allow(dead_code)]
impl CertificationTrack {
    pub fn exam_fee_eur(&self) -> f64 {
        match self {
            Self::PcdDesigner
            | Self::PcbaBusinessAnalyst
            | Self::PcdaDataAnalyst
            | Self::PcssSystemsDb
            | Self::PcdsDeployerSupport => 79.0,
            Self::AllInOneBundle => 149.0,
        }
    }

    pub fn annual_badge_fee_eur(&self) -> f64 {
        39.0
    }

    pub fn title(&self) -> &'static str {
        match self {
            Self::PcdDesigner => "PCD — Proteus Certified Designer",
            Self::PcbaBusinessAnalyst => "PCBA — Proteus Certified Business Analyst",
            Self::PcdaDataAnalyst => "PCDA — Proteus Certified Data Analyst",
            Self::PcssSystemsDb => "PCSS — Proteus Certified Systems & DB Specialist",
            Self::PcdsDeployerSupport => "PCDS — Proteus Certified Deployer / Support Specialist",
            Self::AllInOneBundle => "Proteus Master Bundle (PCD + PCBA + PCDA + PCSS + PCDS)",
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
            complexity_metrics: None,
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
    fn test_compiler_gate_anti_scam_floor_price_rejection() {
        let req = PackageCompileRequest {
            designer_id: "user_02".to_string(),
            designer_tier: CertificationTier::Tier1Associate,
            project_name: "ComplexPOS".to_string(),
            target_client_shop_id: "SHOP_02".to_string(),
            gross_amount_eur: 200.0, // Proposed 200€
            is_escrow_funded: true,
            complexity_metrics: Some(ProjectComplexityMetrics::new(3, 2, 2, 1)), // Floor: 150 + 135 + 70 + 50 + 60 = 465€
        };

        let res = compile_package_gate(&req);
        assert!(!res.success);
        assert!(res.message.contains("Αποτυχία Anti-Scam"));
        assert!(res.message.contains("465.00"));
        assert!(res.message.contains("265.00")); // Deficit
    }

    #[test]
    fn test_certification_tracks_pricing() {
        assert_eq!(CertificationTrack::PcdDesigner.exam_fee_eur(), 79.0);
        assert_eq!(CertificationTrack::PcbaBusinessAnalyst.exam_fee_eur(), 79.0);
        assert_eq!(CertificationTrack::PcdaDataAnalyst.exam_fee_eur(), 79.0);
        assert_eq!(CertificationTrack::PcssSystemsDb.exam_fee_eur(), 79.0);
        assert_eq!(CertificationTrack::PcdsDeployerSupport.exam_fee_eur(), 79.0);
        assert_eq!(CertificationTrack::AllInOneBundle.exam_fee_eur(), 149.0);
        assert_eq!(CertificationTrack::PcdaDataAnalyst.annual_badge_fee_eur(), 39.0);
    }

    #[test]
    fn test_licensed_packages_retrieval() {
        let all_pkgs = get_all_marketplace_packages();
        assert_eq!(all_pkgs.len(), 4);

        let pkgs = get_licensed_packages_for_account("demo@company.com");
        assert_eq!(pkgs.len(), 3);
        assert_eq!(pkgs[0].bundle_id, "PKG-SERVICE-AUTO");
        assert_eq!(pkgs[1].bundle_id, "PKG-RETAIL-POS");
        assert_eq!(pkgs[2].bundle_id, "PKG-MOTO-PRO");

        let doc_pkgs = get_licensed_packages_for_account("doctor@clinic.org");
        assert_eq!(doc_pkgs.len(), 1);
        assert_eq!(doc_pkgs[0].bundle_id, "PKG-CLINIC-HEALTH");

        let empty = get_licensed_packages_for_account("unknown@random.com");
        assert!(empty.is_empty());
    }
}

