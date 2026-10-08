//! Universal 3D Spatial WMS & Retail Shelf Optimizer for Proteus BOS.
//! Includes mathematical 3D bin-packing, ergonomic Golden Zone routing,
//! directed putaway, and photo audit verification.
//! Strict Rule 1 (100% Original Codebase) and Rule 5 (Zero Mock Data).

pub mod db;
pub mod packing;
pub mod types;

pub use db::*;
pub use packing::*;
pub use types::*;

/// Generates an intelligent Directed Putaway recommendation for incoming stock.
/// Prioritizes ergonomic safety (<80cm for heavy goods >=15kg), Golden Zone (80-160cm for Fast Movers),
/// and maximum 3D packing capacity with handling clearance.
pub fn suggest_directed_putaway(
    shelves: &[WarehouseShelf],
    item: &WmsItem,
    clearance_margin: f64,
) -> Result<PutawayRecommendation, WmsPackingError> {
    if shelves.is_empty() {
        return Err(WmsPackingError::ZeroDimensions);
    }

    let target_zone = if item.weight_kg >= 15.0 {
        GoldenZoneCategory::GroundHeavy
    } else {
        match item.turnover_velocity {
            TurnoverVelocity::FastMover => GoldenZoneCategory::GoldenZone,
            TurnoverVelocity::MediumMover => GoldenZoneCategory::SlowMoversUpper,
            TurnoverVelocity::SlowMover => GoldenZoneCategory::HighBufferReserve,
        }
    };

    let mut scored_options: Vec<(i64, &WarehouseShelf, Dimensions, u32)> = Vec::new();

    for shelf in shelves {
        if let Ok((orient, units)) = compute_shelf_fit(shelf, item, clearance_margin) {
            if units > 0 {
                let mut score = 0i64;
                let shelf_zone = shelf.ergonomic_zone();

                if shelf_zone == target_zone {
                    score += 2000;
                } else if is_adjacent_zone(shelf_zone, target_zone) {
                    score += 800;
                }

                // Prefer shelves that accommodate more units efficiently
                score += units as i64 * 5;

                scored_options.push((score, shelf, orient, units));
            }
        }
    }

    // Sort descending by score
    scored_options.sort_by(|a, b| b.0.cmp(&a.0));

    if let Some((_, best_shelf, best_orient, units)) = scored_options.first() {
        let shelf_zone = best_shelf.ergonomic_zone();
        let rationale = format!(
            "Προτεινόμενο Ράφι #{} (Επίπεδο {} - {}): Χωρητικότητα {} τμχ με περιθώριο χειρισμού {:.1}%. {}",
            best_shelf.shelf_id,
            best_shelf.shelf_level,
            shelf_zone.display_label(),
            units,
            clearance_margin * 100.0,
            if shelf_zone == target_zone {
                "Βέλτιστη εργονομική τοποθέτηση."
            } else {
                "Εναλλακτική τοποθέτηση βάσει διαθεσιμότητας."
            }
        );

        Ok(PutawayRecommendation {
            shelf_id: best_shelf.shelf_id.clone(),
            shelf_level: best_shelf.shelf_level,
            best_orientation: *best_orient,
            max_units_fit: *units,
            clearance_percent: clearance_margin * 100.0,
            ergonomic_zone: shelf_zone,
            rationale,
        })
    } else {
        // Find reason for failure on first shelf
        compute_shelf_fit(&shelves[0], item, clearance_margin).map(|(orient, units)| {
            PutawayRecommendation {
                shelf_id: shelves[0].shelf_id.clone(),
                shelf_level: shelves[0].shelf_level,
                best_orientation: orient,
                max_units_fit: units,
                clearance_percent: clearance_margin * 100.0,
                ergonomic_zone: shelves[0].ergonomic_zone(),
                rationale: "Default shelf fallback".to_string(),
            }
        })
    }
}

fn is_adjacent_zone(a: GoldenZoneCategory, b: GoldenZoneCategory) -> bool {
    matches!(
        (a, b),
        (GoldenZoneCategory::GroundHeavy, GoldenZoneCategory::GoldenZone)
            | (GoldenZoneCategory::GoldenZone, GoldenZoneCategory::GroundHeavy)
            | (GoldenZoneCategory::GoldenZone, GoldenZoneCategory::SlowMoversUpper)
            | (GoldenZoneCategory::SlowMoversUpper, GoldenZoneCategory::GoldenZone)
            | (GoldenZoneCategory::SlowMoversUpper, GoldenZoneCategory::HighBufferReserve)
            | (GoldenZoneCategory::HighBufferReserve, GoldenZoneCategory::SlowMoversUpper)
    )
}

/// Verifies whether a scanned SKU matches the planned inventory slot during a shelf sweep audit.
pub fn verify_shelf_placement(expected_sku: Option<&str>, scanned_sku: &str) -> (bool, &'static str) {
    match expected_sku {
        Some(expected) if expected == scanned_sku => (true, "Σωστή Θέση Προϊόντος"),
        Some(_) => (false, "ΠΡΟΕΙΔΟΠΟΙΗΣΗ: Λάθος τοποθέτηση στο ράφι!"),
        None => (true, "Νέα Καταχώριση στο Ράφι"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_directed_putaway_heavy_item_routed_to_ground() {
        let ground_shelf = WarehouseShelf {
            shelf_id: "SHELF-GROUND".to_string(),
            rack_id: "R-1".to_string(),
            shelf_level: 0,
            height_from_floor_cm: 20.0, // Ground (<80cm)
            dimensions: Dimensions::new(120.0, 60.0, 80.0),
            max_weight_kg: 500.0,
            current_weight_kg: 0.0,
        };

        let golden_shelf = WarehouseShelf {
            shelf_id: "SHELF-GOLDEN".to_string(),
            rack_id: "R-1".to_string(),
            shelf_level: 2,
            height_from_floor_cm: 120.0, // Golden zone (80-160cm)
            dimensions: Dimensions::new(120.0, 60.0, 80.0),
            max_weight_kg: 500.0,
            current_weight_kg: 0.0,
        };

        let heavy_item = WmsItem {
            sku: "ANVIL-30".to_string(),
            name: "Industrial Motor 30kg".to_string(),
            dimensions: Dimensions::new(40.0, 30.0, 30.0),
            weight_kg: 30.0, // >= 15kg -> must route to GroundHeavy!
            storage_form: StorageForm::Cuboid,
            turnover_velocity: TurnoverVelocity::FastMover,
            is_upright_only: false,
        };

        let shelves = vec![golden_shelf, ground_shelf];
        let rec = suggest_directed_putaway(&shelves, &heavy_item, DEFAULT_CLEARANCE_MARGIN).unwrap();

        assert_eq!(rec.shelf_id, "SHELF-GROUND");
        assert_eq!(rec.ergonomic_zone, GoldenZoneCategory::GroundHeavy);
    }

    #[test]
    fn test_directed_putaway_fast_mover_routed_to_golden_zone() {
        let ground_shelf = WarehouseShelf {
            shelf_id: "SHELF-GROUND".to_string(),
            rack_id: "R-1".to_string(),
            shelf_level: 0,
            height_from_floor_cm: 40.0,
            dimensions: Dimensions::new(100.0, 40.0, 40.0),
            max_weight_kg: 200.0,
            current_weight_kg: 0.0,
        };

        let golden_shelf = WarehouseShelf {
            shelf_id: "SHELF-GOLDEN".to_string(),
            rack_id: "R-1".to_string(),
            shelf_level: 2,
            height_from_floor_cm: 110.0, // Golden zone!
            dimensions: Dimensions::new(100.0, 40.0, 40.0),
            max_weight_kg: 200.0,
            current_weight_kg: 0.0,
        };

        let fast_retail_item = WmsItem {
            sku: "IPHONE-CASE".to_string(),
            name: "Silicone Case".to_string(),
            dimensions: Dimensions::new(18.0, 10.0, 3.0),
            weight_kg: 0.1,
            storage_form: StorageForm::Cuboid,
            turnover_velocity: TurnoverVelocity::FastMover, // Fast mover -> Golden zone!
            is_upright_only: false,
        };

        let shelves = vec![ground_shelf, golden_shelf];
        let rec = suggest_directed_putaway(&shelves, &fast_retail_item, DEFAULT_CLEARANCE_MARGIN).unwrap();

        assert_eq!(rec.shelf_id, "SHELF-GOLDEN");
        assert_eq!(rec.ergonomic_zone, GoldenZoneCategory::GoldenZone);
    }

    #[test]
    fn test_verify_shelf_placement() {
        let (ok, msg) = verify_shelf_placement(Some("SKU-100"), "SKU-100");
        assert!(ok);
        assert_eq!(msg, "Σωστή Θέση Προϊόντος");

        let (wrong, err_msg) = verify_shelf_placement(Some("SKU-100"), "SKU-999");
        assert!(!wrong);
        assert!(err_msg.contains("ΠΡΟΕΙΔΟΠΟΙΗΣΗ"));
    }
}
