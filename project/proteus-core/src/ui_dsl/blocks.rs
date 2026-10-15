//! Pre-Assembled Visual Block Catalog ("Planet-Style" Drag-and-Drop Building Blocks).
//! Strict Rule 1 (100% Original Codebase), Rule 3 (<400 lines), and Rule 5 (Zero Mock Data).
//! Enables designers and merchants to assemble complete, responsive web storefronts
//! in seconds without manually crafting individual low-level AST nodes from scratch.

use crate::ui_dsl::ast::{ButtonAction, LayoutKind, UiNode, UiStyle};
use serde::{Deserialize, Serialize};

/// High-level storefront composition configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorefrontConfig {
    pub store_id: String,
    pub store_name: String,
    pub tagline: String,
    pub hero_cta: String,
    pub afm: String,
    pub address: String,
    pub phone: String,
    pub email: String,
    pub primary_color: String,
    pub features: Vec<(String, String)>,
    pub products: Vec<(String, String, f64, String, bool)>,
}

/// Catalog of pre-designed visual blocks for instant canvas composition.
pub struct VisualBlockCatalog;

impl VisualBlockCatalog {
    /// Creates a high-impact Hero banner with headline, description, and primary CTA button.
    pub fn hero_banner(
        headline: &str,
        subheadline: &str,
        cta_label: &str,
        cta_action: ButtonAction,
        accent_color: &str,
    ) -> UiNode {
        UiNode::Container {
            layout: LayoutKind::Column,
            children: vec![
                UiNode::Heading {
                    level: 1,
                    text: headline.to_string(),
                    style: UiStyle {
                        font_size_pt: Some(28),
                        font_weight_bold: true,
                        is_centered: true,
                        margin_px: Some(8),
                        ..Default::default()
                    },
                },
                UiNode::Paragraph {
                    text: subheadline.to_string(),
                    style: UiStyle {
                        font_size_pt: Some(14),
                        is_centered: true,
                        margin_px: Some(12),
                        text_color: Some("#475569".to_string()),
                        ..Default::default()
                    },
                },
                UiNode::Container {
                    layout: LayoutKind::Row,
                    children: vec![UiNode::Button {
                        label: cta_label.to_string(),
                        action: cta_action,
                        style: UiStyle {
                            bg_color: Some(accent_color.to_string()),
                            text_color: Some("#FFFFFF".to_string()),
                            padding_px: Some(14),
                            corner_radius_px: Some(8),
                            font_weight_bold: true,
                            is_centered: true,
                            ..Default::default()
                        },
                    }],
                    style: UiStyle {
                        is_centered: true,
                        margin_px: Some(16),
                        ..Default::default()
                    },
                },
            ],
            style: UiStyle {
                padding_px: Some(40),
                bg_color: Some("#F8FAFC".to_string()),
                corner_radius_px: Some(12),
                margin_px: Some(16),
                ..Default::default()
            },
        }
    }

    /// Creates a multi-column feature benefits grid (e.g. 100% Offline, myDATA Direct, 0% SaaS Fees).
    pub fn features_grid(columns: u32, items: &[(String, String)]) -> UiNode {
        let mut cards = Vec::with_capacity(items.len());
        for (title, desc) in items {
            cards.push(UiNode::Container {
                layout: LayoutKind::Column,
                children: vec![
                    UiNode::Heading {
                        level: 3,
                        text: title.clone(),
                        style: UiStyle {
                            font_size_pt: Some(16),
                            font_weight_bold: true,
                            margin_px: Some(6),
                            ..Default::default()
                        },
                    },
                    UiNode::Paragraph {
                        text: desc.clone(),
                        style: UiStyle {
                            font_size_pt: Some(12),
                            text_color: Some("#64748B".to_string()),
                            ..Default::default()
                        },
                    },
                ],
                style: UiStyle {
                    padding_px: Some(20),
                    bg_color: Some("#FFFFFF".to_string()),
                    corner_radius_px: Some(8),
                    border_width_px: Some(1),
                    border_color: Some("#E2E8F0".to_string()),
                    shadow_elevation: Some(1),
                    ..Default::default()
                },
            });
        }

        UiNode::Grid {
            columns,
            gap_px: 20,
            children: cards,
            style: UiStyle {
                padding_px: Some(24),
                ..Default::default()
            },
        }
    }

    /// Creates a responsive product showcase grid with prices and add-to-cart buttons.
    pub fn product_grid(
        section_title: &str,
        columns: u32,
        products: &[(String, String, f64, String, bool)],
    ) -> UiNode {
        let mut product_cards = Vec::with_capacity(products.len());
        for (sku, title, price, img, in_stock) in products {
            product_cards.push(UiNode::ProductCard {
                sku: sku.clone(),
                title: title.clone(),
                price_eur: *price,
                image_url: img.clone(),
                in_stock: *in_stock,
                style: UiStyle {
                    padding_px: Some(16),
                    corner_radius_px: Some(8),
                    shadow_elevation: Some(1),
                    border_width_px: Some(1),
                    border_color: Some("#E2E8F0".to_string()),
                    bg_color: Some("#FFFFFF".to_string()),
                    ..Default::default()
                },
            });
        }

        UiNode::Container {
            layout: LayoutKind::Column,
            children: vec![
                UiNode::Heading {
                    level: 2,
                    text: section_title.to_string(),
                    style: UiStyle {
                        font_size_pt: Some(22),
                        font_weight_bold: true,
                        margin_px: Some(12),
                        ..Default::default()
                    },
                },
                UiNode::Grid {
                    columns,
                    gap_px: 20,
                    children: product_cards,
                    style: UiStyle {
                        padding_px: Some(12),
                        ..Default::default()
                    },
                },
            ],
            style: UiStyle {
                padding_px: Some(16),
                ..Default::default()
            },
        }
    }

    /// Creates a statutory contact, Greek AFM fiscal identity, and store location footer card.
    pub fn contact_and_fiscal_footer(
        company_name: &str,
        afm: &str,
        address: &str,
        phone: &str,
        email: &str,
    ) -> UiNode {
        UiNode::Container {
            layout: LayoutKind::Column,
            children: vec![
                UiNode::Heading {
                    level: 3,
                    text: company_name.to_string(),
                    style: UiStyle {
                        font_size_pt: Some(16),
                        font_weight_bold: true,
                        ..Default::default()
                    },
                },
                UiNode::Paragraph {
                    text: format!("ΑΦΜ: {} | Διεύθυνση: {}", afm, address),
                    style: UiStyle {
                        font_size_pt: Some(12),
                        text_color: Some("#475569".to_string()),
                        margin_px: Some(4),
                        ..Default::default()
                    },
                },
                UiNode::Paragraph {
                    text: format!("Τηλέφωνο: {} | Email: {}", phone, email),
                    style: UiStyle {
                        font_size_pt: Some(12),
                        text_color: Some("#475569".to_string()),
                        margin_px: Some(4),
                        ..Default::default()
                    },
                },
            ],
            style: UiStyle {
                padding_px: Some(24),
                bg_color: Some("#F1F5F9".to_string()),
                corner_radius_px: Some(8),
                margin_px: Some(20),
                ..Default::default()
            },
        }
    }

    /// Assembles a complete, ready-to-compile, responsive one-page storefront in 1-click.
    pub fn assemble_one_page_store(config: &StorefrontConfig) -> UiNode {
        let hero = Self::hero_banner(
            &config.store_name,
            &config.tagline,
            &config.hero_cta,
            ButtonAction::OpenCheckout,
            &config.primary_color,
        );

        let features = Self::features_grid(3, &config.features);
        let products = Self::product_grid("Προτεινόμενα Προϊόντα", 3, &config.products);

        let cart_summary = UiNode::CartSummary {
            title: "Καλάθι Αγορών".to_string(),
            show_tax_breakdown: true,
            style: UiStyle {
                padding_px: Some(20),
                bg_color: Some("#FFFFFF".to_string()),
                corner_radius_px: Some(8),
                border_width_px: Some(1),
                border_color: Some("#CBD5E1".to_string()),
                margin_px: Some(16),
                ..Default::default()
            },
        };

        let checkout = UiNode::CheckoutForm {
            submit_label: "Ολοκλήρωση Παραγγελίας".to_string(),
            require_phone: true,
            require_afm: false,
            style: UiStyle {
                padding_px: Some(20),
                bg_color: Some("#FFFFFF".to_string()),
                corner_radius_px: Some(8),
                border_width_px: Some(1),
                border_color: Some("#CBD5E1".to_string()),
                margin_px: Some(16),
                ..Default::default()
            },
        };

        let footer = Self::contact_and_fiscal_footer(
            &config.store_name,
            &config.afm,
            &config.address,
            &config.phone,
            &config.email,
        );

        UiNode::Page {
            id: config.store_id.clone(),
            title: config.store_name.clone(),
            meta_description: Some(config.tagline.clone()),
            children: vec![hero, features, products, cart_summary, checkout, footer],
            style: UiStyle {
                max_width_px: Some(1200),
                is_centered: true,
                ..Default::default()
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui_dsl::compiler::UiDslCompiler;

    #[test]
    fn test_visual_block_catalog_and_assembly() {
        let config = StorefrontConfig {
            store_id: "hellas_organics".to_string(),
            store_name: "Ελληνικά Βιολογικά Προϊόντα".to_string(),
            tagline: "Αγνό Παρθένο Ελαιόλαδο & Μέλι από Έλληνες Παραγωγούς".to_string(),
            hero_cta: "Αγοράστε Τώρα".to_string(),
            afm: "094014201".to_string(),
            address: "Πανεπιστημίου 10, Αθήνα".to_string(),
            phone: "+30 210 3344556".to_string(),
            email: "sales@hellas-organics.gr".to_string(),
            primary_color: "#16A34A".to_string(),
            features: vec![
                ("100% Βιολογικό".to_string(), "Πιστοποίηση BIO Hellas".to_string()),
                ("Άμεση Παράδοση".to_string(), "Σε όλη την Ελλάδα εντός 24 ωρών".to_string()),
                ("Ασφαλής Πληρωμή".to_string(), "Κάρτα ή Αντικαταβολή".to_string()),
            ],
            products: vec![
                ("EVOO-01".into(), "Βιολογικό Ελαιόλαδο 5L".into(), 48.00, "olive.webp".into(), true),
                ("HONEY-02".into(), "Μέλι Θυμαρίσιο Κρήτης 1kg".into(), 16.50, "honey.webp".into(), true),
            ],
        };

        let page = VisualBlockCatalog::assemble_one_page_store(&config);
        assert!(page.count_nodes() > 10, "Assembled page should have rich component hierarchy");

        // Verify compilation to semantic HTML5
        let compiled = UiDslCompiler::compile_page(&page).expect("Compilation must succeed");
        assert!(compiled.standalone_html.contains("Ελληνικά Βιολογικά Προϊόντα"));
        assert!(compiled.standalone_html.contains("ΑΦΜ: 094014201"));
        assert!(compiled.standalone_html.contains("EVOO-01"));
        assert!(compiled.standalone_html.contains("Βιολογικό Ελαιόλαδο 5L"));
        assert!(compiled.standalone_html.contains("Ολοκλήρωση Παραγγελίας"));
    }
}
