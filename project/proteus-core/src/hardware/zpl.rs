//! ZPL-II (Zebra Programming Language) Thermal Command Stream Generator.
//! Produces resolution-aware, standard ZPL-II byte streams for Zebra and ZPL-compatible industrial printers.
//! Strict Rule 1 (100% Original Codebase) and Rule 5 (Zero Mock Data).

use std::io::Write as IoWrite;

use super::label::{
    BarcodeSymbology, IndustrialLabel, LabelElement, LabelRotation, PrintDensity,
};

/// Builder for constructing ZPL-II printer command byte buffers.
#[derive(Debug, Clone)]
pub struct ZplBuilder {
    pub density: PrintDensity,
    buffer: Vec<u8>,
}

impl ZplBuilder {
    pub fn new(density: PrintDensity) -> Self {
        Self {
            density,
            buffer: Vec::new(),
        }
    }

    pub fn density(&self) -> PrintDensity {
        self.density
    }

    /// Emits start-of-format marker: `^XA`.
    pub fn start_format(mut self) -> Self {
        let _ = write!(self.buffer, "^XA\r\n");
        self
    }

    /// Emits end-of-format marker: `^XZ`.
    pub fn end_format(mut self) -> Self {
        let _ = write!(self.buffer, "^XZ\r\n");
        self
    }

    /// Sets printable width (`^PW<w>`) and label length (`^LL<h>`) in dots.
    pub fn label_dimensions_dots(mut self, width_dots: u32, height_dots: u32) -> Self {
        let _ = write!(
            self.buffer,
            "^PW{}^LL{}^LH0,0\r\n",
            width_dots.max(80),
            height_dots.max(80)
        );
        self
    }

    /// Adds standard scalable text field (`^FO`, `^A0`, `^FD...^FS`).
    pub fn text(
        mut self,
        x_dots: u32,
        y_dots: u32,
        height_dots: u32,
        rotation: LabelRotation,
        content: &str,
    ) -> Self {
        let rot_char = rotation.zpl_orientation_char();
        let font_h = height_dots.max(16);
        let font_w = (font_h as f64 * 0.9).round() as u32;

        let _ = write!(
            self.buffer,
            "^FO{},{}^A0{},{},{}^FD{}^FS\r\n",
            x_dots,
            y_dots,
            rot_char,
            font_h,
            font_w,
            content
        );
        self
    }

    /// Adds 1D barcode: Code 128 (`^BC`) or EAN-13 (`^BEN`).
    pub fn barcode(
        mut self,
        x_dots: u32,
        y_dots: u32,
        symbology: BarcodeSymbology,
        height_dots: u32,
        human_readable: bool,
        rotation: LabelRotation,
        data: &str,
    ) -> Self {
        let rot_char = rotation.zpl_orientation_char();
        let h_flag = if human_readable { 'Y' } else { 'N' };

        match symbology {
            BarcodeSymbology::Ean13 => {
                let _ = write!(
                    self.buffer,
                    "^FO{},{}^BE{},{},{},N^FD{}^FS\r\n",
                    x_dots,
                    y_dots,
                    rot_char,
                    height_dots.max(20),
                    h_flag,
                    data
                );
            }
            BarcodeSymbology::Code128 | _ => {
                let _ = write!(
                    self.buffer,
                    "^FO{},{}^BC{},{},{},N,N^FD{}^FS\r\n",
                    x_dots,
                    y_dots,
                    rot_char,
                    height_dots.max(20),
                    h_flag,
                    data
                );
            }
        }
        self
    }

    /// Adds 2D QR Code field: `^BQN,2,<magnification>,M,7`.
    pub fn qrcode(mut self, x_dots: u32, y_dots: u32, magnification: u8, data: &str) -> Self {
        let mag = magnification.clamp(2, 10);
        let _ = write!(
            self.buffer,
            "^FO{},{}^BQN,2,{},M,7^FDMA,{}^FS\r\n",
            x_dots,
            y_dots,
            mag,
            data
        );
        self
    }

    /// Adds 2D DataMatrix code field: `^BXN,<height_dots>,200`.
    pub fn datamatrix(mut self, x_dots: u32, y_dots: u32, height_dots: u32, data: &str) -> Self {
        let _ = write!(
            self.buffer,
            "^FO{},{}^BXN,{},200^FD{}^FS\r\n",
            x_dots,
            y_dots,
            height_dots.max(8),
            data
        );
        self
    }

    /// Draws a rectangular boundary box (`^GB<w>,<h>,<thickness>,B,0^FS`).
    pub fn box_outline(
        mut self,
        x_dots: u32,
        y_dots: u32,
        width_dots: u32,
        height_dots: u32,
        thickness_dots: u32,
    ) -> Self {
        let _ = write!(
            self.buffer,
            "^FO{},{}^GB{},{},{},B,0^FS\r\n",
            x_dots,
            y_dots,
            width_dots.max(1),
            height_dots.max(1),
            thickness_dots.max(1)
        );
        self
    }

    /// Sets print quantity: `^PQ<copies>`.
    pub fn print_quantity(mut self, copies: u32) -> Self {
        let _ = write!(self.buffer, "^PQ{}\r\n", copies.max(1));
        self
    }

    /// Returns the raw compiled ZPL-II command byte buffer.
    pub fn build(self) -> Vec<u8> {
        self.buffer
    }

    /// Returns string view of the ZPL-II commands for inspection and network preview.
    pub fn as_str(&self) -> Result<&str, std::str::Utf8Error> {
        std::str::from_utf8(&self.buffer)
    }

    /// Compiles an abstract `IndustrialLabel` into a complete ZPL-II format.
    pub fn compile_label(label: &IndustrialLabel, copies: u32) -> Vec<u8> {
        let w_dots = label.dimensions.width_dots(label.density);
        let h_dots = label.dimensions.height_dots(label.density);

        let mut builder = Self::new(label.density)
            .start_format()
            .label_dimensions_dots(w_dots, h_dots);

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
                    let x = label.density.mm_to_dots(*x_mm);
                    let y = label.density.mm_to_dots(*y_mm);
                    // Approximate point size to dots (1 pt ~ 0.3527 mm)
                    let pt_mm = (*font_size_pt as f64) * 0.3527;
                    let font_h_dots = label.density.mm_to_dots(pt_mm).max(18);

                    builder = builder.text(x, y, font_h_dots, *rotation, content);
                }
                LabelElement::Barcode {
                    x_mm,
                    y_mm,
                    symbology,
                    height_mm,
                    data,
                    show_human_readable,
                } => {
                    let x = label.density.mm_to_dots(*x_mm);
                    let y = label.density.mm_to_dots(*y_mm);
                    let h_dots = label.density.mm_to_dots(*height_mm);

                    if *symbology == BarcodeSymbology::QrCode {
                        let mag = match label.density {
                            PrintDensity::Dpi203 => 4,
                            PrintDensity::Dpi300 => 6,
                        };
                        builder = builder.qrcode(x, y, mag, data);
                    } else if *symbology == BarcodeSymbology::DataMatrix {
                        let cell_dots = match label.density {
                            PrintDensity::Dpi203 => 6,
                            PrintDensity::Dpi300 => 9,
                        };
                        builder = builder.datamatrix(x, y, cell_dots, data);
                    } else {
                        builder = builder.barcode(
                            x,
                            y,
                            *symbology,
                            h_dots,
                            *show_human_readable,
                            LabelRotation::Deg0,
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
                    let x = label.density.mm_to_dots(*x_mm);
                    let y = label.density.mm_to_dots(*y_mm);
                    let w = label.density.mm_to_dots(*width_mm);
                    let h = label.density.mm_to_dots(*height_mm);
                    let th = label.density.mm_to_dots(*thickness_mm).max(1);

                    builder = builder.box_outline(x, y, w, h, th);
                }
            }
        }

        builder.print_quantity(copies).end_format().build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hardware::label::LabelDimensions;

    #[test]
    fn test_zpl_builder_raw_stream() {
        let bytes = ZplBuilder::new(PrintDensity::Dpi203)
            .start_format()
            .label_dimensions_dots(400, 240)
            .text(24, 24, 28, LabelRotation::Deg0, "PROTEUS")
            .barcode(
                24,
                80,
                BarcodeSymbology::Ean13,
                80,
                true,
                LabelRotation::Deg0,
                "5201234567890",
            )
            .print_quantity(1)
            .end_format()
            .build();

        let s = std::str::from_utf8(&bytes).unwrap();
        assert!(s.starts_with("^XA"));
        assert!(s.contains("^PW400^LL240"));
        assert!(s.contains("^FO24,24^A0N,28,25^FDPROTEUS^FS"));
        assert!(s.contains("^BEN,80,Y,N^FD5201234567890^FS"));
        assert!(s.contains("^PQ1"));
        assert!(s.ends_with("^XZ\r\n"));
    }

    #[test]
    fn test_zpl_compile_industrial_label() {
        let mut label = IndustrialLabel::new(
            LabelDimensions::standard_shelf_edge(),
            PrintDensity::Dpi203,
        );
        label.add_text(3.0, 3.0, 12, "Espresso Beans 1kg", true);
        label.add_barcode(3.0, 10.0, BarcodeSymbology::Ean13, 12.0, "5201234567890", true);
        label.add_box_outline(1.0, 1.0, 48.0, 28.0, 0.5);

        let stream = ZplBuilder::compile_label(&label, 1);
        let s = std::str::from_utf8(&stream).unwrap();

        assert!(s.contains("^XA"));
        assert!(s.contains("^PW400^LL240"));
        assert!(s.contains("Espresso Beans 1kg"));
        assert!(s.contains("^GB"));
        assert!(s.contains("^XZ"));
    }
}
