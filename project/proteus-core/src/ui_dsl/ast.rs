//! Declarative UI & WebStore AST for Proteus BOS.
//! Strict Rule 1 (100% Original Codebase), Rule 3 (<400 lines), and Rule 5 (Zero Mock Data).
//! Provides a human-readable, token-dense component tree for instant websites,
//! mobile storefronts, and internal POS/CRM displays.

use serde::{Deserialize, Serialize};

/// Layout distribution along primary axis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LayoutKind {
    Row,
    Column,
    Stack,
}

/// Action triggered by interactive buttons or tap targets.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ButtonAction {
    Navigate(String),
    AddToCart { sku: String, default_qty: u32 },
    OpenCheckout,
    SubmitOrder,
    Custom(String),
}

/// Visual styling tokens adhering to PDS Nordic Slate standards.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct UiStyle {
    pub padding_px: Option<u32>,
    pub margin_px: Option<u32>,
    pub gap_px: Option<u32>,
    pub max_width_px: Option<u32>,
    pub bg_color: Option<String>,
    pub text_color: Option<String>,
    pub border_color: Option<String>,
    pub border_width_px: Option<u32>,
    pub corner_radius_px: Option<u32>,
    pub shadow_elevation: Option<u32>,
    pub font_size_pt: Option<u32>,
    pub font_weight_bold: bool,
    pub is_centered: bool,
}

/// Core declarative UI nodes for pages, storefronts, and business interfaces.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum UiNode {
    Page {
        id: String,
        title: String,
        meta_description: Option<String>,
        children: Vec<UiNode>,
        style: UiStyle,
    },
    Container {
        layout: LayoutKind,
        children: Vec<UiNode>,
        style: UiStyle,
    },
    Grid {
        columns: u32,
        gap_px: u32,
        children: Vec<UiNode>,
        style: UiStyle,
    },
    Heading {
        level: u8,
        text: String,
        style: UiStyle,
    },
    Paragraph {
        text: String,
        style: UiStyle,
    },
    Image {
        src: String,
        alt: String,
        aspect_ratio: Option<String>,
        style: UiStyle,
    },
    Button {
        label: String,
        action: ButtonAction,
        style: UiStyle,
    },
    ProductCard {
        sku: String,
        title: String,
        price_eur: f64,
        image_url: String,
        in_stock: bool,
        style: UiStyle,
    },
    CartSummary {
        title: String,
        show_tax_breakdown: bool,
        style: UiStyle,
    },
    CheckoutForm {
        submit_label: String,
        require_phone: bool,
        require_afm: bool,
        style: UiStyle,
    },
    CustomHtml {
        raw_html: String,
    },
}

impl UiNode {
    /// Recursively counts all component nodes in this tree.
    pub fn count_nodes(&self) -> usize {
        match self {
            Self::Page { children, .. } | Self::Container { children, .. } | Self::Grid { children, .. } => {
                1 + children.iter().map(|c| c.count_nodes()).sum::<usize>()
            }
            _ => 1,
        }
    }
}
