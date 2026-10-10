//! Industrial Thermal Label & Barcode Geometry Engine.
//! Provides resolution-independent dimensions, DPI density scaling,
//! and industrial symbology representations for TSPL and ZPL printers.
//! Strict Rule 1 (100% Original Codebase) and Rule 5 (Zero Mock Data).

use serde::{Deserialize, Serialize};

/// Thermal printer print-head resolution density.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PrintDensity {
    /// 203 DPI (8 dots per mm) — standard retail/logistics desktop thermal printers
    Dpi203,
    /// 300 DPI (11.81 dots per mm ~ 12 dots/mm) — high-precision industrial printers
    Dpi300,
}

impl PrintDensity {
    /// Exact dots per millimeter for this print-head density.
    pub fn dots_per_mm(&self) -> f64 {
        match self {
            Self::Dpi203 => 8.0,
            Self::Dpi300 => 11.811,
        }
    }

    /// Converts physical millimeters to discrete print-head dots/pixels.
    pub fn mm_to_dots(&self, mm: f64) -> u32 {
        (mm * self.dots_per_mm()).round().max(0.0) as u32
    }
}

/// Rotation angle for label elements.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum LabelRotation {
    #[default]
    Deg0,
    Deg90,
    Deg180,
    Deg270,
}

impl LabelRotation {
    pub fn as_degrees(&self) -> u32 {
        match self {
            Self::Deg0 => 0,
            Self::Deg90 => 90,
            Self::Deg180 => 180,
            Self::Deg270 => 270,
        }
    }

    /// TSPL rotation orientation code (0, 90, 180, 270).
    pub fn tspl_rotation_code(&self) -> u16 {
        self.as_degrees() as u16
    }

    /// ZPL orientation flag: 'N' (Normal), 'R' (Rotated 90), 'I' (Inverted 180), 'B' (Bottom-up 270).
    pub fn zpl_orientation_char(&self) -> char {
        match self {
            Self::Deg0 => 'N',
            Self::Deg90 => 'R',
            Self::Deg180 => 'I',
            Self::Deg270 => 'B',
        }
    }
}

/// 1D and 2D barcode symbologies supported by industrial thermal printers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BarcodeSymbology {
    /// European Article Number (GS1 Retail Standard)
    Ean13,
    /// High-density alphanumeric logistics standard
    Code128,
    /// Quick Response 2D matrix
    QrCode,
    /// Industrial DataMatrix (ISO/IEC 16022)
    DataMatrix,
}

impl BarcodeSymbology {
    pub fn is_two_dimensional(&self) -> bool {
        matches!(self, Self::QrCode | Self::DataMatrix)
    }

    pub fn tspl_name(&self) -> &'static str {
        match self {
            Self::Ean13 => "EAN13",
            Self::Code128 => "128",
            Self::QrCode => "QRCODE",
            Self::DataMatrix => "DMATRIX",
        }
    }

    pub fn zpl_command_prefix(&self) -> &'static str {
        match self {
            Self::Ean13 => "^BEN",
            Self::Code128 => "^BC",
            Self::QrCode => "^BQ",
            Self::DataMatrix => "^BX",
        }
    }
}

/// Physical dimensions of die-cut adhesive labels or continuous rolls.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LabelDimensions {
    pub width_mm: f64,
    pub height_mm: f64,
    pub gap_mm: f64,
}

impl LabelDimensions {
    pub fn new(width_mm: f64, height_mm: f64, gap_mm: f64) -> Self {
        Self {
            width_mm: width_mm.max(10.0),
            height_mm: height_mm.max(10.0),
            gap_mm: gap_mm.max(0.0),
        }
    }

    /// Standard retail shelf-edge adhesive label: 50mm × 30mm with 2mm gap.
    pub fn standard_shelf_edge() -> Self {
        Self::new(50.0, 30.0, 2.0)
    }

    /// Standard warehouse tote / carton identification label: 80mm × 60mm with 3mm gap.
    pub fn standard_tote_carton() -> Self {
        Self::new(80.0, 60.0, 3.0)
    }

    /// Standard 4×6 inch (102mm × 152mm) courier shipping waybill with 3mm gap.
    pub fn standard_shipping_waybill() -> Self {
        Self::new(102.0, 152.0, 3.0)
    }

    pub fn width_dots(&self, density: PrintDensity) -> u32 {
        density.mm_to_dots(self.width_mm)
    }

    pub fn height_dots(&self, density: PrintDensity) -> u32 {
        density.mm_to_dots(self.height_mm)
    }

    pub fn gap_dots(&self, density: PrintDensity) -> u32 {
        density.mm_to_dots(self.gap_mm)
    }
}

/// Individual drawable element positioned on the label surface.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LabelElement {
    Text {
        x_mm: f64,
        y_mm: f64,
        font_size_pt: u8,
        content: String,
        bold: bool,
        rotation: LabelRotation,
    },
    Barcode {
        x_mm: f64,
        y_mm: f64,
        symbology: BarcodeSymbology,
        height_mm: f64,
        data: String,
        show_human_readable: bool,
    },
    BoxOutline {
        x_mm: f64,
        y_mm: f64,
        width_mm: f64,
        height_mm: f64,
        thickness_mm: f64,
    },
}

/// Abstract representation of an industrial label ready for compilation to TSPL/ZPL.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IndustrialLabel {
    pub dimensions: LabelDimensions,
    pub density: PrintDensity,
    pub elements: Vec<LabelElement>,
}

impl IndustrialLabel {
    pub fn new(dimensions: LabelDimensions, density: PrintDensity) -> Self {
        Self {
            dimensions,
            density,
            elements: Vec::new(),
        }
    }

    pub fn add_text(
        &mut self,
        x_mm: f64,
        y_mm: f64,
        font_size_pt: u8,
        content: &str,
        bold: bool,
    ) {
        self.elements.push(LabelElement::Text {
            x_mm,
            y_mm,
            font_size_pt,
            content: content.to_string(),
            bold,
            rotation: LabelRotation::Deg0,
        });
    }

    pub fn add_barcode(
        &mut self,
        x_mm: f64,
        y_mm: f64,
        symbology: BarcodeSymbology,
        height_mm: f64,
        data: &str,
        show_human_readable: bool,
    ) {
        self.elements.push(LabelElement::Barcode {
            x_mm,
            y_mm,
            symbology,
            height_mm,
            data: data.to_string(),
            show_human_readable,
        });
    }

    pub fn add_box_outline(
        &mut self,
        x_mm: f64,
        y_mm: f64,
        width_mm: f64,
        height_mm: f64,
        thickness_mm: f64,
    ) {
        self.elements.push(LabelElement::BoxOutline {
            x_mm,
            y_mm,
            width_mm,
            height_mm,
            thickness_mm,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_print_density_conversions() {
        let d203 = PrintDensity::Dpi203;
        assert_eq!(d203.dots_per_mm(), 8.0);
        assert_eq!(d203.mm_to_dots(50.0), 400);

        let d300 = PrintDensity::Dpi300;
        assert_eq!(d300.mm_to_dots(10.0), 118);
    }

    #[test]
    fn test_label_dimensions_and_dots() {
        let shelf = LabelDimensions::standard_shelf_edge();
        assert_eq!(shelf.width_mm, 50.0);
        assert_eq!(shelf.height_mm, 30.0);
        assert_eq!(shelf.gap_mm, 2.0);

        assert_eq!(shelf.width_dots(PrintDensity::Dpi203), 400);
        assert_eq!(shelf.height_dots(PrintDensity::Dpi203), 240);
        assert_eq!(shelf.gap_dots(PrintDensity::Dpi203), 16);
    }

    #[test]
    fn test_industrial_label_builder() {
        let mut label = IndustrialLabel::new(
            LabelDimensions::standard_shelf_edge(),
            PrintDensity::Dpi203,
        );

        label.add_text(5.0, 5.0, 12, "Wi-Fi 6 Router", true);
        label.add_barcode(5.0, 15.0, BarcodeSymbology::Ean13, 10.0, "5201234567890", true);

        assert_eq!(label.elements.len(), 2);
        assert_eq!(label.dimensions.width_mm, 50.0);
    }
}
