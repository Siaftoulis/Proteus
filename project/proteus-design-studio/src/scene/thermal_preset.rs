//! Thermal Receipt Preset Artboard Generator for ESC/POS Hardware Canvas.
//! Generates pixel-accurate, monospace receipt layouts for 58mm (32-col) and 80mm (48-col) thermal rolls.

use super::types::*;
use super::document::ProjectDocument;

/// Creates an authentic thermal receipt artboard with dot-leader rows, header, and hardware actions.
pub fn create_thermal_receipt_preset(is_58mm: bool, shop_name: &str) -> ProjectDocument {
    let mut doc = ProjectDocument::new();

    let (w, h, cols) = if is_58mm {
        (384.0, 620.0, 32)
    } else {
        (576.0, 780.0, 48)
    };

    let title_name = if shop_name.trim().is_empty() {
        "PROTEUS SERVICE & RETAIL LAB"
    } else {
        shop_name
    };

    let dark_bg = Rgba { r: 20, g: 24, b: 33, a: 255 };
    let border_color = Rgba { r: 50, g: 60, b: 80, a: 255 };
    let text_bright = Rgba { r: 240, g: 243, b: 246, a: 255 };
    let text_dim = Rgba { r: 140, g: 150, b: 165, a: 255 };
    let divider_char = if is_58mm {
        "--------------------------------"
    } else {
        "------------------------------------------------"
    };
    let double_divider_char = if is_58mm {
        "================================"
    } else {
        "================================================"
    };

    // Helper: build text node
    fn mk_text(id: &str, content: &str, x: f32, y: f32, size: f32, bold: bool, color: Rgba) -> Node {
        Node {
            id: id.into(),
            name: content.into(),
            node_type: NodeType::Text {
                content: content.into(),
                font: FontSpec {
                    family: "Inter".into(),
                    size,
                    weight: if bold { 700 } else { 400 },
                    color,
                },
            },
            parent_id: None,
            children_ids: vec![],
            styling: Styling::default(),
            style: NodeStyle::default(),
            layout: Layout { width: Sizing::Hug, height: Sizing::Hug, ..Default::default() },
            position: (x, y),
            visible: true,
            locked: false,
            z: 1,
        }
    }

    // Helper: build action button
    fn mk_btn(id: &str, label: &str, x: f32, y: f32, w: f32, h: f32, style: ButtonStyle) -> Node {
        Node {
            id: id.into(),
            name: label.into(),
            node_type: NodeType::Button { label: label.into(), style },
            parent_id: None,
            children_ids: vec![],
            styling: Styling::default(),
            style: NodeStyle::default(),
            layout: Layout { width: Sizing::Fixed(w), height: Sizing::Fixed(h), ..Default::default() },
            position: (x, y),
            visible: true,
            locked: false,
            z: 2,
        }
    }

    // 1. Root Receipt Frame (The Paper Roll)
    let receipt_page = Node {
        id: "thermal-receipt".into(),
        name: format!("Thermal Receipt ({}mm)", if is_58mm { "58" } else { "80" }),
        node_type: NodeType::Frame,
        parent_id: None,
        children_ids: vec![],
        styling: Styling {
            background: Some(dark_bg),
            corner_radius: [4., 4., 4., 4.],
            border: Some(Border { width: 1.5, color: border_color }),
            padding: [16., 16., 16., 16.],
            shadow: None,
            opacity: 1.0,
        },
        style: NodeStyle::default(),
        layout: Layout { width: Sizing::Fixed(w), height: Sizing::Fixed(h), ..Default::default() },
        position: (80., 40.),
        visible: true,
        locked: false,
        z: 0,
    };
    let _ = doc.add_node(receipt_page, None);

    // 2. Receipt Header
    let _ = doc.add_node(mk_text("rcpt-title", title_name, 20., 20., 15., true, text_bright), Some("thermal-receipt".into()));
    let _ = doc.add_node(mk_text("rcpt-spec", &format!("Hardware ESC/POS Monospace Engine • {} Columns", cols), 20., 42., 9.5, false, text_dim), Some("thermal-receipt".into()));
    let _ = doc.add_node(mk_text("rcpt-info", "Vouliagmenis Ave 124 • +30 210 8945600 • VAT EL998877665", 20., 58., 9.0, false, text_dim), Some("thermal-receipt".into()));

    // 3. Metadata Section
    let _ = doc.add_node(mk_text("rcpt-div1", divider_char, 20., 76., 9.5, false, text_dim), Some("thermal-receipt".into()));
    let _ = doc.add_node(mk_text("rcpt-meta1", "TICKET #0042 • TERMINAL #01 • CASHIER: ALEX", 20., 92., 10.0, true, text_bright), Some("thermal-receipt".into()));
    let _ = doc.add_node(mk_text("rcpt-meta2", "DATE: 2026-09-28 17:45 • CLIENT: GEORGE K.", 20., 108., 9.0, false, text_dim), Some("thermal-receipt".into()));
    let _ = doc.add_node(mk_text("rcpt-div2", divider_char, 20., 124., 9.5, false, text_dim), Some("thermal-receipt".into()));

    // 4. Line Items Table Header
    let _ = doc.add_node(mk_text("rcpt-th", "QTY  DESCRIPTION                                  PRICE", 20., 140., 9.5, true, text_bright), Some("thermal-receipt".into()));

    // Line items data
    let items = [
        (" 1x ", "Full Synthetic 5W-30 Oil Service", "€65.00"),
        (" 2x ", "Ceramic Brake Pad Set (Front)", "€110.00"),
        (" 1x ", "Electronic OBD-II Diagnostic", "€35.00"),
        (" 1x ", "Eco Environmental Disposal Fee", "€4.50"),
    ];

    let mut current_y = 160.0;
    for (i, (qty, desc, price)) in items.iter().enumerate() {
        let line_content = format!("{}{} ... {}", qty, desc, price);
        let _ = doc.add_node(mk_text(&format!("rcpt-it-{}", i), &line_content, 20., current_y, 9.5, false, text_bright), Some("thermal-receipt".into()));
        current_y += 22.0;
    }

    // 5. Totals & Tax
    let _ = doc.add_node(mk_text("rcpt-div3", double_divider_char, 20., current_y, 9.5, false, text_dim), Some("thermal-receipt".into()));
    current_y += 18.0;

    let _ = doc.add_node(mk_text("rcpt-subtotal", "SUBTOTAL (NET):                           €172.98", 20., current_y, 10.0, false, text_dim), Some("thermal-receipt".into()));
    current_y += 18.0;
    let _ = doc.add_node(mk_text("rcpt-vat", "VAT 24%:                                   €41.52", 20., current_y, 10.0, false, text_dim), Some("thermal-receipt".into()));
    current_y += 22.0;
    let _ = doc.add_node(mk_text("rcpt-total", "TOTAL AMOUNT:                             €214.50", 20., current_y, 13.0, true, text_bright), Some("thermal-receipt".into()));
    current_y += 26.0;

    // 6. Payment & Verification
    let _ = doc.add_node(mk_text("rcpt-pay", "PAID VIA: CONTACTLESS VISA **** 4812 [APPROVED]", 20., current_y, 9.5, true, text_bright), Some("thermal-receipt".into()));
    current_y += 18.0;
    let _ = doc.add_node(mk_text("rcpt-auth", "AUTH CODE: #092841 • RRN: 882910384729", 20., current_y, 9.0, false, text_dim), Some("thermal-receipt".into()));
    current_y += 20.0;

    // 7. Footer & Barcode/QR
    let _ = doc.add_node(mk_text("rcpt-div4", divider_char, 20., current_y, 9.5, false, text_dim), Some("thermal-receipt".into()));
    current_y += 18.0;
    let _ = doc.add_node(mk_text("rcpt-fmsg", "Thank you for trusting Proteus Business OS!", 20., current_y, 9.5, false, text_dim), Some("thermal-receipt".into()));
    current_y += 18.0;
    let _ = doc.add_node(mk_text("rcpt-qr", "[ QR VERIFICATION: https://proteus.app/t/0042 ]", 20., current_y, 9.0, true, text_bright), Some("thermal-receipt".into()));
    current_y += 30.0;

    // 8. Hardware Quick Action Controls
    let btn_w = (w - 52.0) * 0.5;
    let _ = doc.add_node(mk_btn("btn-print-receipt", "🖨 Spool Print", 20., current_y, btn_w, 36., ButtonStyle::Primary), Some("thermal-receipt".into()));
    let _ = doc.add_node(mk_btn("btn-cash-drawer", "💵 Kick Drawer", 26.0 + btn_w, current_y, btn_w, 36., ButtonStyle::Secondary), Some("thermal-receipt".into()));

    doc
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_thermal_receipt_preset_dimensions() {
        let doc80 = create_thermal_receipt_preset(false, "Acme Store");
        let root80 = doc80.get_node("thermal-receipt").expect("root node exists");
        if let Sizing::Fixed(w) = root80.layout.width {
            assert_eq!(w, 576.0);
        } else {
            panic!("Expected fixed width for 80mm");
        }

        let doc58 = create_thermal_receipt_preset(true, "Small Kiosk");
        let root58 = doc58.get_node("thermal-receipt").expect("root node exists");
        if let Sizing::Fixed(w) = root58.layout.width {
            assert_eq!(w, 384.0);
        } else {
            panic!("Expected fixed width for 58mm");
        }

        assert!(doc80.get_node("btn-print-receipt").is_some());
        assert!(doc80.get_node("btn-cash-drawer").is_some());
    }
}
