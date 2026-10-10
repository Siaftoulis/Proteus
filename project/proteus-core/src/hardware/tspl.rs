//! TSPL (Taiwan Semiconductor / TSC / Xprinter) Thermal Command Stream Generator.
//! Produces resolution-aware TSPL-EZ compatible byte streams for industrial label printers.
//! Strict Rule 1 (100% Original Codebase) and Rule 5 (Zero Mock Data).

use std::io::Write as IoWrite;

use super::label::{
    BarcodeSymbology, IndustrialLabel, LabelElement, PrintDensity,
};

/// Builder for constructing TSPL-II printer command byte buffers.
#[derive(Debug, Clone)]
pub struct TsplBuilder {
    density: PrintDensity,
    buffer: Vec<u8>,
}

impl TsplBuilder {
    pub fn new(density: PrintDensity) -> Self {
        Self {
            density,
            buffer: Vec::new(),
        }
    }

    /// Sets label width and height in millimeters: `SIZE <w> mm, <h> mm`.
    pub fn size_mm(mut self, width_mm: f64, height_mm: f64) -> Self {
        let _ = writeln!(self.buffer, "SIZE {:.1} mm, {:.1} mm", width_mm, height_mm);
        self
    }

    /// Sets gap distance between die-cut labels: `GAP <gap> mm, <offset> mm`.
    pub fn gap_mm(mut self, gap_mm: f64, offset_mm: f64) -> Self {
        let _ = writeln!(self.buffer, "GAP {:.1} mm, {:.1} mm", gap_mm, offset_mm);
        self
    }

    /// Sets print orientation (0: normal top-to-bottom, 1: inverted 180°): `DIRECTION <n>`.
    pub fn direction(mut self, dir: u8) -> Self {
        let _ = writeln!(self.buffer, "DIRECTION {}", dir.min(1));
        self
    }

    /// Clears the printer image memory buffer: `CLS`.
    pub fn cls(mut self) -> Self {
        let _ = writeln!(self.buffer, "CLS");
        self
    }

    /// Adds standard bitmap/scalable text line to label buffer.
    pub fn text(
        mut self,
        x_dots: u32,
        y_dots: u32,
        font: &str,
        rotation: u16,
        x_mul: u8,
        y_mul: u8,
        content: &str,
    ) -> Self {
        // Escape quotes if any in content
        let safe_content = content.replace('"', "\\\"");
        let _ = writeln!(
            self.buffer,
            "TEXT {},{},\"{}\",{},{},{},\"{}\"",
            x_dots,
            y_dots,
            font,
            rotation,
            x_mul.max(1),
            y_mul.max(1),
            safe_content
        );
        self
    }

    /// Adds 1D barcode to the label image buffer.
    pub fn barcode(
        mut self,
        x_dots: u32,
        y_dots: u32,
        symbology: BarcodeSymbology,
        height_dots: u32,
        human_readable: bool,
        rotation: u16,
        data: &str,
    ) -> Self {
        let readable_flag = if human_readable { 2 } else { 0 }; // 2 = centered below
        let (narrow, wide) = match self.density {
            PrintDensity::Dpi203 => (2, 4),
            PrintDensity::Dpi300 => (3, 6),
        };

        let _ = writeln!(
            self.buffer,
            "BARCODE {},{},\"{}\",{},{},{},{},{},\"{}\"",
            x_dots,
            y_dots,
            symbology.tspl_name(),
            height_dots,
            readable_flag,
            rotation,
            narrow,
            wide,
            data
        );
        self
    }

    /// Adds 2D QR Code: `QRCODE x,y,ecc,cell_width,mode,rotation,model,mask,"content"`.
    pub fn qrcode(
        mut self,
        x_dots: u32,
        y_dots: u32,
        ecc: char,
        cell_width: u8,
        data: &str,
    ) -> Self {
        let ecc_upper = ecc.to_ascii_uppercase();
        let valid_ecc = if matches!(ecc_upper, 'L' | 'M' | 'Q' | 'H') {
            ecc_upper
        } else {
            'M'
        };

        let _ = writeln!(
            self.buffer,
            "QRCODE {},{},{},{},A,0,M2,S7,\"{}\"",
            x_dots,
            y_dots,
            valid_ecc,
            cell_width.clamp(1, 10),
            data
        );
        self
    }

    /// Adds 2D DataMatrix code: `DMATRIX x,y,w,h,"content"`.
    pub fn datamatrix(mut self, x_dots: u32, y_dots: u32, cell_size: u8, data: &str) -> Self {
        let _ = writeln!(
            self.buffer,
            "DMATRIX {},{},{},{},\"{}\"",
            x_dots,
            y_dots,
            cell_size.max(4),
            cell_size.max(4),
            data
        );
        self
    }

    /// Draws a rectangular boundary box: `BOX x_start,y_start,x_end,y_end,thickness`.
    pub fn box_outline(
        mut self,
        x_start: u32,
        y_start: u32,
        x_end: u32,
        y_end: u32,
        thickness_dots: u32,
    ) -> Self {
        let _ = writeln!(
            self.buffer,
            "BOX {},{},{},{},{}",
            x_start,
            y_start,
            x_end,
            y_end,
            thickness_dots.max(1)
        );
        self
    }

    /// Appends print trigger command: `PRINT <sets>,<copies>`.
    pub fn print(mut self, copies: u32) -> Self {
        let _ = writeln!(self.buffer, "PRINT 1,{}", copies.max(1));
        self
    }

    /// Returns the raw compiled TSPL command byte array.
    pub fn build(self) -> Vec<u8> {
        self.buffer
    }

    /// Returns string view of the TSPL commands for debugging and preview.
    pub fn as_str(&self) -> Result<&str, std::str::Utf8Error> {
        std::str::from_utf8(&self.buffer)
    }

    /// Compiles an abstract `IndustrialLabel` into a complete TSPL print job.
    pub fn compile_label(label: &IndustrialLabel, copies: u32) -> Vec<u8> {
        let mut builder = Self::new(label.density)
            .size_mm(label.dimensions.width_mm, label.dimensions.height_mm)
            .gap_mm(label.dimensions.gap_mm, 0.0)
            .direction(0)
            .cls();

        for element in &label.elements {
            match element {
                LabelElement::Text {
                    x_mm,
                    y_mm,
                    font_size_pt,
                    content,
                    bold: _,
                    rotation,
                } => {
                    let x_dots = label.density.mm_to_dots(*x_mm);
                    let y_dots = label.density.mm_to_dots(*y_mm);
                    let (font, mul) = if *font_size_pt <= 10 {
                        ("2", 1)
                    } else if *font_size_pt <= 14 {
                        ("3", 1)
                    } else if *font_size_pt <= 20 {
                        ("4", 1)
                    } else {
                        ("4", 2)
                    };

                    builder = builder.text(
                        x_dots,
                        y_dots,
                        font,
                        rotation.tspl_rotation_code(),
                        mul,
                        mul,
                        content,
                    );
                }
                LabelElement::Barcode {
                    x_mm,
                    y_mm,
                    symbology,
                    height_mm,
                    data,
                    show_human_readable,
                } => {
                    let x_dots = label.density.mm_to_dots(*x_mm);
                    let y_dots = label.density.mm_to_dots(*y_mm);
                    let h_dots = label.density.mm_to_dots(*height_mm);

                    if *symbology == BarcodeSymbology::QrCode {
                        let cell_w = match label.density {
                            PrintDensity::Dpi203 => 4,
                            PrintDensity::Dpi300 => 6,
                        };
                        builder = builder.qrcode(x_dots, y_dots, 'M', cell_w, data);
                    } else if *symbology == BarcodeSymbology::DataMatrix {
                        let cell_w = match label.density {
                            PrintDensity::Dpi203 => 6,
                            PrintDensity::Dpi300 => 9,
                        };
                        builder = builder.datamatrix(x_dots, y_dots, cell_w, data);
                    } else {
                        builder = builder.barcode(
                            x_dots,
                            y_dots,
                            *symbology,
                            h_dots,
                            *show_human_readable,
                            0,
                            data,
                        );
                    }
                }
                LabelElement::BoxOutline {
                    x_mm,
                    y_mm,
                    width_mm,
                    height_mm,
                    thickness_mm,
                } => {
                    let x1 = label.density.mm_to_dots(*x_mm);
                    let y1 = label.density.mm_to_dots(*y_mm);
                    let x2 = label.density.mm_to_dots(*x_mm + *width_mm);
                    let y2 = label.density.mm_to_dots(*y_mm + *height_mm);
                    let thick = label.density.mm_to_dots(*thickness_mm).max(1);

                    builder = builder.box_outline(x1, y1, x2, y2, thick);
                }
            }
        }

        builder.print(copies).build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hardware::label::LabelDimensions;

    #[test]
    fn test_tspl_builder_raw_stream() {
        let bytes = TsplBuilder::new(PrintDensity::Dpi203)
            .size_mm(50.0, 30.0)
            .gap_mm(2.0, 0.0)
            .cls()
            .text(24, 24, "3", 0, 1, 1, "PROTEUS")
            .barcode(24, 80, BarcodeSymbology::Ean13, 80, true, 0, "5201234567890")
            .print(1)
            .build();

        let s = std::str::from_utf8(&bytes).unwrap();
        assert!(s.contains("SIZE 50.0 mm, 30.0 mm"));
        assert!(s.contains("GAP 2.0 mm, 0.0 mm"));
        assert!(s.contains("CLS"));
        assert!(s.contains("TEXT 24,24,\"3\",0,1,1,\"PROTEUS\""));
        assert!(s.contains("BARCODE 24,80,\"EAN13\",80,2,0,2,4,\"5201234567890\""));
        assert!(s.contains("PRINT 1,1"));
    }

    #[test]
    fn test_tspl_compile_industrial_label() {
        let mut label = IndustrialLabel::new(
            LabelDimensions::standard_shelf_edge(),
            PrintDensity::Dpi203,
        );
        label.add_text(3.0, 3.0, 12, "Espresso Beans 1kg", true);
        label.add_barcode(3.0, 10.0, BarcodeSymbology::Ean13, 12.0, "5201234567890", true);
        label.add_box_outline(1.0, 1.0, 48.0, 28.0, 0.5);

        let stream = TsplBuilder::compile_label(&label, 2);
        let s = std::str::from_utf8(&stream).unwrap();

        assert!(s.contains("SIZE 50.0 mm, 30.0 mm"));
        assert!(s.contains("Espresso Beans 1kg"));
        assert!(s.contains("BARCODE"));
        assert!(s.contains("BOX"));
        assert!(s.contains("PRINT 1,2"));
    }
}
