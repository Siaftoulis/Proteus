//! Pre-formatted Industrial Label Templates for Retail & Logistics.
//! Provides standardized label designs for shelf edge price tags,
//! container waybills, and serial barcode stickers with TSPL and ZPL export.
//! Strict Rule 1 (100% Original Codebase) and Rule 5 (Zero Mock Data).

use serde::{Deserialize, Serialize};

use super::label::{BarcodeSymbology, IndustrialLabel, LabelDimensions, PrintDensity};
use super::tspl::TsplBuilder;
use super::zpl::ZplBuilder;

/// Pre-formatted Shelf Edge Price Label Template (Standard 50mm × 30mm).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ShelfEdgePriceTemplate {
    pub sku: String,
    pub product_name: String,
    pub retail_price: f64,
    pub barcode: String,
    pub vat_rate_pct: u8,
    pub unit_of_measure: String,
    pub shelf_location: Option<String>,
    pub dimensions: LabelDimensions,
    pub density: PrintDensity,
}

impl ShelfEdgePriceTemplate {
    pub fn new(sku: &str, product_name: &str, retail_price: f64, barcode: &str) -> Self {
        Self {
            sku: sku.trim().to_uppercase(),
            product_name: product_name.trim().to_string(),
            retail_price: retail_price.max(0.0),
            barcode: barcode.trim().to_string(),
            vat_rate_pct: 24,
            unit_of_measure: "ΤΜΧ".to_string(),
            shelf_location: None,
            dimensions: LabelDimensions::standard_shelf_edge(),
            density: PrintDensity::Dpi203,
        }
    }

    pub fn with_vat(mut self, vat_pct: u8) -> Self {
        self.vat_rate_pct = vat_pct;
        self
    }

    pub fn with_unit(mut self, uom: &str) -> Self {
        self.unit_of_measure = uom.trim().to_uppercase();
        self
    }

    pub fn with_shelf_location(mut self, shelf_id: &str) -> Self {
        self.shelf_location = Some(shelf_id.trim().to_uppercase());
        self
    }

    pub fn with_density(mut self, density: PrintDensity) -> Self {
        self.density = density;
        self
    }

    /// Formats the price in Greek/European decimal currency notation (e.g., `49,90 €`).
    pub fn formatted_price(&self) -> String {
        let whole = self.retail_price.floor() as u64;
        let cents = ((self.retail_price - (whole as f64)) * 100.0).round() as u64;
        format!("{},{:02} €", whole, cents)
    }

    /// Determines if barcode is standard EAN-13 (13 digits).
    pub fn is_ean13(&self) -> bool {
        self.barcode.len() == 13 && self.barcode.chars().all(|c| c.is_ascii_digit())
    }

    /// Builds the structured `IndustrialLabel` representation.
    pub fn to_industrial_label(&self) -> IndustrialLabel {
        let mut label = IndustrialLabel::new(self.dimensions.clone(), self.density);

        // Outer border box (1.0mm inset from edges)
        let w_box = self.dimensions.width_mm - 2.0;
        let h_box = self.dimensions.height_mm - 2.0;
        label.add_box_outline(1.0, 1.0, w_box, h_box, 0.4);

        // Product Name (truncated to 28 chars to fit 50mm width)
        let name_display: String = self.product_name.chars().take(28).collect();
        label.add_text(2.5, 2.5, 11, &name_display, true);

        // SKU and optional Shelf Location
        let mut meta = format!("SKU: {}", self.sku);
        if let Some(shelf) = &self.shelf_location {
            meta.push_str(&format!("  [Ράφι: {}]", shelf));
        }
        label.add_text(2.5, 6.5, 8, &meta, false);

        // Large Retail Price Display (18pt bold)
        let price_str = self.formatted_price();
        label.add_text(2.5, 10.0, 18, &price_str, true);

        // VAT & Unit note next to price
        let vat_note = format!("(με ΦΠΑ {}% / {})", self.vat_rate_pct, self.unit_of_measure);
        label.add_text(27.0, 12.0, 8, &vat_note, false);

        // Barcode (EAN-13 if 13 digits, otherwise Code 128)
        let symbology = if self.is_ean13() {
            BarcodeSymbology::Ean13
        } else {
            BarcodeSymbology::Code128
        };
        label.add_barcode(3.0, 16.5, symbology, 9.5, &self.barcode, true);

        label
    }

    /// Compiles directly to TSPL binary stream.
    pub fn to_tspl(&self, copies: u32) -> Vec<u8> {
        let industrial = self.to_industrial_label();
        TsplBuilder::compile_label(&industrial, copies)
    }

    /// Compiles directly to ZPL-II binary stream.
    pub fn to_zpl(&self, copies: u32) -> Vec<u8> {
        let industrial = self.to_industrial_label();
        ZplBuilder::compile_label(&industrial, copies)
    }
}

/// Pre-formatted Smart Tote & Carton Waybill Label Template (Standard 80mm × 60mm).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SmartToteWaybillTemplate {
    pub container_code: String,
    pub container_type: String,
    pub total_units: u32,
    pub total_weight_grams: f64,
    pub seal_hash: Option<String>,
    pub shelf_destination: Option<String>,
    pub sealed_at: Option<i64>,
    pub dimensions: LabelDimensions,
    pub density: PrintDensity,
}

impl SmartToteWaybillTemplate {
    pub fn new(
        container_code: &str,
        container_type: &str,
        total_units: u32,
        total_weight_grams: f64,
    ) -> Self {
        Self {
            container_code: container_code.trim().to_uppercase(),
            container_type: container_type.trim().to_uppercase(),
            total_units,
            total_weight_grams: total_weight_grams.max(0.0),
            seal_hash: None,
            shelf_destination: None,
            sealed_at: None,
            dimensions: LabelDimensions::standard_tote_carton(),
            density: PrintDensity::Dpi203,
        }
    }

    pub fn with_seal_hash(mut self, hash: &str) -> Self {
        self.seal_hash = Some(hash.trim().to_string());
        self
    }

    pub fn with_shelf_destination(mut self, shelf_id: &str) -> Self {
        self.shelf_destination = Some(shelf_id.trim().to_uppercase());
        self
    }

    pub fn with_sealed_at(mut self, timestamp: i64) -> Self {
        self.sealed_at = Some(timestamp);
        self
    }

    pub fn with_density(mut self, density: PrintDensity) -> Self {
        self.density = density;
        self
    }

    /// Converts a live `SmartContainer` directly into a printable tote waybill template.
    pub fn from_smart_container(container: &crate::wms::tote::SmartContainer) -> Self {
        let shelf_opt = match &container.status {
            crate::wms::tote::ContainerStatus::AssignedToShelf(s) => Some(s.clone()),
            _ => None,
        };

        Self {
            container_code: container.container_code.clone(),
            container_type: container.container_type.as_str().to_string(),
            total_units: container.total_units(),
            total_weight_grams: container.total_weight_grams,
            seal_hash: container.seal_hash.clone(),
            shelf_destination: shelf_opt,
            sealed_at: container.sealed_at,
            dimensions: LabelDimensions::standard_tote_carton(),
            density: PrintDensity::Dpi203,
        }
    }

    /// Builds the structured `IndustrialLabel` representation (80mm × 60mm).
    pub fn to_industrial_label(&self) -> IndustrialLabel {
        let mut label = IndustrialLabel::new(self.dimensions.clone(), self.density);

        // Outer border box (1.5mm inset: 77.0mm × 57.0mm)
        let w_box = self.dimensions.width_mm - 3.0;
        let h_box = self.dimensions.height_mm - 3.0;
        label.add_box_outline(1.5, 1.5, w_box, h_box, 0.5);

        // Header: Container Code & Type Badge (y=3.5mm)
        let header_text = format!("ΚΙΒΩΤΙΟ: {}", self.container_code);
        label.add_text(3.5, 3.5, 16, &header_text, true);
        let type_badge = format!("[{}]", self.container_type);
        label.add_text(55.0, 4.0, 10, &type_badge, false);

        // Header separator line
        label.add_box_outline(3.0, 9.5, w_box - 3.0, 0.4, 0.4);

        // Metrics row (y=11.5mm): Total Units and Weight
        let units_text = format!("ΤΕΜΑΧΙΑ: {}", self.total_units);
        let weight_kg = self.total_weight_grams / 1000.0;
        let weight_text = format!("ΒΑΡΟΣ: {:.2} kg ({:.0} g)", weight_kg, self.total_weight_grams);
        label.add_text(3.5, 11.5, 12, &units_text, true);
        label.add_text(35.0, 11.5, 12, &weight_text, true);

        // Shelf Destination (y=17.5mm)
        let shelf_text = if let Some(dest) = &self.shelf_destination {
            format!("ΘΕΣΗ ΡΑΦΙΟΥ: ΡΑΦΙ #{}", dest)
        } else {
            "ΚΑΤΑΣΤΑΣΗ: ΣΕ ΣΥΣΚΕΥΑΣΙΑ (ΑΝΟΙΧΤΟ)".to_string()
        };
        label.add_text(3.5, 17.5, 12, &shelf_text, true);

        // Middle separator line
        label.add_box_outline(3.0, 23.5, w_box - 3.0, 0.4, 0.4);

        // Merkle SHA-256 Seal Section (y=25.0mm)
        let seal_str = self.seal_hash.as_deref().unwrap_or("UNSEALED-OPEN-MANIFEST");
        let qr_payload = format!(
            "PROTEUS:CONTAINER:{}:UNITS:{}:SEAL:{}",
            self.container_code, self.total_units, seal_str
        );

        // 2D QR Code on the left (x=3.5mm, y=25.5mm)
        label.add_barcode(
            3.5,
            25.5,
            BarcodeSymbology::QrCode,
            18.0,
            &qr_payload,
            false,
        );

        // Right side of QR code (x=26.0mm)
        label.add_text(26.0, 25.5, 9, "ΚΡΥΠΤΟΓΡΑΦΙΚΗ ΣΦΡΑΓΙΔΑ (SHA-256):", true);
        let seal_display = if seal_str.len() > 18 {
            format!("{}...", &seal_str[..18])
        } else {
            seal_str.to_string()
        };
        label.add_text(26.0, 29.5, 8, &seal_display, false);

        if let Some(ts) = self.sealed_at {
            label.add_text(26.0, 33.5, 8, &format!("ΣΦΡΑΓΙΣΤΗΚΕ: UNIX-{}", ts), false);
        } else {
            label.add_text(26.0, 33.5, 8, "ΚΑΤΑΣΤΑΣΗ: ΕΚΚΡΕΜΕΙ ΣΦΡΑΓΙΣΗ", false);
        }
        label.add_text(26.0, 37.5, 8, "ΑΠΑΡΑΒΙΑΣΤΗ ΑΛΥΣΙΔΑ MERKLE", true);

        // Bottom 1D Barcode for legacy laser scanners (x=3.5mm, y=45.5mm)
        label.add_barcode(
            3.5,
            45.5,
            BarcodeSymbology::Code128,
            8.0,
            &self.container_code,
            true,
        );

        label
    }

    /// Compiles directly to TSPL binary stream.
    pub fn to_tspl(&self, copies: u32) -> Vec<u8> {
        let industrial = self.to_industrial_label();
        TsplBuilder::compile_label(&industrial, copies)
    }

    /// Compiles directly to ZPL-II binary stream.
    pub fn to_zpl(&self, copies: u32) -> Vec<u8> {
        let industrial = self.to_industrial_label();
        ZplBuilder::compile_label(&industrial, copies)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shelf_edge_price_template_formatting() {
        let tag = ShelfEdgePriceTemplate::new(
            "ROUTER-AX3000",
            "Wi-Fi 6 Gigabit Router AX3000",
            49.90,
            "5201234567890",
        )
        .with_shelf_location("S-102")
        .with_vat(24);

        assert_eq!(tag.formatted_price(), "49,90 €");
        assert!(tag.is_ean13());

        let label = tag.to_industrial_label();
        assert_eq!(label.elements.len(), 6); // box, name, sku/shelf, price, vat note, barcode
    }

    #[test]
    fn test_shelf_edge_template_tspl_and_zpl_export() {
        let tag = ShelfEdgePriceTemplate::new(
            "ESP-32-DEVKIT",
            "ESP32 Wi-Fi / BLE Dev Board",
            8.50,
            "5209876543210",
        )
        .with_shelf_location("S-101");

        let tspl_bytes = tag.to_tspl(1);
        let tspl_str = std::str::from_utf8(&tspl_bytes).unwrap();
        assert!(tspl_str.contains("SIZE 50.0 mm, 30.0 mm"));
        assert!(tspl_str.contains("8,50 €"));
        assert!(tspl_str.contains("BARCODE"));
        assert!(tspl_str.contains("PRINT 1,1"));

        let zpl_bytes = tag.to_zpl(1);
        let zpl_str = std::str::from_utf8(&zpl_bytes).unwrap();
        assert!(zpl_str.starts_with("^XA"));
        assert!(zpl_str.contains("8,50 €"));
        assert!(zpl_str.contains("^BEN"));
        assert!(zpl_str.contains("^PQ1"));
        assert!(zpl_str.ends_with("^XZ\r\n"));
    }

    #[test]
    fn test_smart_tote_waybill_template_tspl_and_zpl() {
        let tote_tpl = SmartToteWaybillTemplate::new("TOTE-101", "CartonBox", 12, 3500.0)
            .with_shelf_destination("S-103")
            .with_seal_hash("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855")
            .with_sealed_at(1700000000);

        let label = tote_tpl.to_industrial_label();
        assert_eq!(label.dimensions.width_mm, 80.0);
        assert_eq!(label.dimensions.height_mm, 60.0);

        let tspl = tote_tpl.to_tspl(1);
        let tspl_str = std::str::from_utf8(&tspl).unwrap();
        assert!(tspl_str.contains("SIZE 80.0 mm, 60.0 mm"));
        assert!(tspl_str.contains("TOTE-101"));
        assert!(tspl_str.contains("QRCODE"));
        assert!(tspl_str.contains("PRINT 1,1"));

        let zpl = tote_tpl.to_zpl(1);
        let zpl_str = std::str::from_utf8(&zpl).unwrap();
        assert!(zpl_str.starts_with("^XA"));
        assert!(zpl_str.contains("TOTE-101"));
        assert!(zpl_str.contains("^BQN"));
        assert!(zpl_str.contains("^PQ1"));
        assert!(zpl_str.ends_with("^XZ\r\n"));
    }
}
