//! Declarative UI & WebStore DSL for Proteus BOS.
//! Strict Rule 1 (100% Original Codebase), Rule 3 (<400 lines), and Rule 5 (Zero Mock Data).
//! Provides a compact, human-readable declarative language for instant websites,
//! mobile storefronts, and internal POS/CRM layouts.

pub mod ast;
pub mod compiler;

pub use ast::{ButtonAction, LayoutKind, UiNode, UiStyle};
pub use compiler::{CompiledWebPage, UiDslCompiler};

/// High-level builder for assembling storefront and application page trees.
pub struct PageBuilder {
    id: String,
    title: String,
    children: Vec<UiNode>,
    style: UiStyle,
}

impl PageBuilder {
    pub fn new(id: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            children: Vec::new(),
            style: UiStyle::default(),
        }
    }

    pub fn with_style(mut self, style: UiStyle) -> Self {
        self.style = style;
        self
    }

    pub fn add_child(mut self, node: UiNode) -> Self {
        self.children.push(node);
        self
    }

    pub fn build(self) -> UiNode {
        UiNode::Page {
            id: self.id,
            title: self.title,
            meta_description: None,
            children: self.children,
            style: self.style,
        }
    }
}

/// Helper for constructing rapid e-commerce product grids.
pub struct StorefrontTemplate;

impl StorefrontTemplate {
    pub fn create_catalog_page(
        store_name: &str,
        products: &[(String, String, f64, String, bool)],
    ) -> UiNode {
        let mut product_nodes = Vec::with_capacity(products.len());
        for (sku, title, price, img, in_stock) in products {
            product_nodes.push(UiNode::ProductCard {
                sku: sku.clone(),
                title: title.clone(),
                price_eur: *price,
                image_url: img.clone(),
                in_stock: *in_stock,
                style: UiStyle {
                    padding_px: Some(16),
                    corner_radius_px: Some(8),
                    shadow_elevation: Some(1),
                    ..Default::default()
                },
            });
        }

        UiNode::Page {
            id: "store_front".to_string(),
            title: store_name.to_string(),
            meta_description: Some(format!("Official webstore for {}", store_name)),
            children: vec![
                UiNode::Heading {
                    level: 1,
                    text: store_name.to_string(),
                    style: UiStyle {
                        font_size_pt: Some(24),
                        font_weight_bold: true,
                        is_centered: true,
                        ..Default::default()
                    },
                },
                UiNode::Grid {
                    columns: 3,
                    gap_px: 16,
                    children: product_nodes,
                    style: UiStyle {
                        padding_px: Some(24),
                        ..Default::default()
                    },
                },
                UiNode::CartSummary {
                    title: "Shopping Cart".to_string(),
                    show_tax_breakdown: true,
                    style: UiStyle {
                        padding_px: Some(16),
                        ..Default::default()
                    },
                },
            ],
            style: UiStyle {
                max_width_px: Some(1200),
                ..Default::default()
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ui_node_tree_counting_and_serialization() {
        let page = PageBuilder::new("home", "Hardware Store")
            .add_child(UiNode::Heading {
                level: 1,
                text: "Fasteners & Screws".to_string(),
                style: UiStyle::default(),
            })
            .add_child(UiNode::Grid {
                columns: 2,
                gap_px: 12,
                children: vec![
                    UiNode::ProductCard {
                        sku: "SCRW-01".to_string(),
                        title: "M4 Hex Bolt".to_string(),
                        price_eur: 0.15,
                        image_url: "bolt.webp".to_string(),
                        in_stock: true,
                        style: UiStyle::default(),
                    },
                    UiNode::ProductCard {
                        sku: "SCRW-02".to_string(),
                        title: "M6 Flange Nut".to_string(),
                        price_eur: 0.22,
                        image_url: "nut.webp".to_string(),
                        in_stock: true,
                        style: UiStyle::default(),
                    },
                ],
                style: UiStyle::default(),
            })
            .build();

        assert_eq!(page.count_nodes(), 5);

        let json = serde_json::to_string(&page).unwrap();
        assert!(json.contains("Hardware Store"));
        assert!(json.contains("SCRW-01"));

        let deserialized: UiNode = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.count_nodes(), 5);
    }

    #[test]
    fn test_storefront_template_generation() {
        let products = vec![
            ("V-01".into(), "Stainless Steel Screw".into(), 0.12, "screw.webp".into(), true),
            ("V-02".into(), "Titanium Washer".into(), 0.45, "washer.webp".into(), true),
        ];

        let page = StorefrontTemplate::create_catalog_page("Mega Fasteners", &products);
        assert_eq!(page.count_nodes(), 6);
        if let UiNode::Page { title, children, .. } = page {
            assert_eq!(title, "Mega Fasteners");
            assert_eq!(children.len(), 3);
        } else {
            panic!("Expected Page node");
        }
    }
}
