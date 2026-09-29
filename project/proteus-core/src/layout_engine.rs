//! Deterministic Layout Engine & Structured Output Validator for Proteus CRM.
//! Conforms strictly to Sections 31 & 32 of the Proteus Design System specification.
//! Zero arbitrary inventions: only registered components, layouts, patterns, and tokens are allowed.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Supported layout formats conforming to Section 32
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum StructuredElement {
    Component {
        component: String,
        props: serde_json::Value,
    },
    Layout {
        layout: String,
        props: serde_json::Value,
        children: Vec<StructuredElement>,
    },
    Pattern {
        pattern: String,
        props: serde_json::Value,
    },
}

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum LayoutEngineError {
    #[error("Unregistered component: '{0}'. All components must be in the design system registry.")]
    UnregisteredComponent(String),
    #[error("Unregistered layout: '{0}'. All layouts must be in the design system registry.")]
    UnregisteredLayout(String),
    #[error("Unregistered pattern: '{0}'. All patterns must be in the design system registry.")]
    UnregisteredPattern(String),
    #[error("Missing required property '{0}' in component '{1}'.")]
    MissingRequiredProp(String, String),
    #[error("JSON serialization or parsing error: {0}")]
    JsonError(String),
}

/// Authoritative registry validator
pub struct LayoutRegistryValidator {
    pub allowed_components: HashSet<&'static str>,
    pub allowed_layouts: HashSet<&'static str>,
    pub allowed_patterns: HashSet<&'static str>,
}

impl Default for LayoutRegistryValidator {
    fn default() -> Self {
        let mut components = HashSet::new();
        // Section 33 Preserved
        components.insert("HeroSection");
        components.insert("FeatureGrid");
        components.insert("PricingTable");
        components.insert("CallToAction");
        components.insert("ContactForm");
        // Actions
        components.insert("PDSButton");
        components.insert("PDSIconButton");
        components.insert("PDSLink");
        components.insert("PDSMenu");
        // Inputs
        components.insert("PDSInput");
        components.insert("PDSSelect");
        components.insert("PDSCheckbox");
        components.insert("PDSRadio");
        components.insert("PDSSwitch");
        components.insert("PDSTextarea");
        components.insert("PDSDatePicker");
        // Feedback
        components.insert("PDSAlert");
        components.insert("PDSToast");
        components.insert("PDSTooltip");
        components.insert("PDSProgress");
        components.insert("PDSSpinner");
        components.insert("PDSSkeleton");
        // Data & Display
        components.insert("PDSBadge");
        components.insert("PDSStatCard");
        components.insert("PDSChartCard");
        components.insert("PDSDataTable");
        components.insert("PDSList");
        // Navigation
        components.insert("PDSNavbar");
        components.insert("PDSSidebar");
        components.insert("PDSBreadcrumb");
        components.insert("PDSNavTabs");
        components.insert("PDSPagination");
        // Structure
        components.insert("PDSContainer");
        components.insert("PDSSection");
        components.insert("PDSStack");
        components.insert("PDSGrid");
        components.insert("PDSCard");
        components.insert("PDSHeader");
        components.insert("PDSModal");
        components.insert("PDSDrawer");
        // States
        components.insert("PDSEmptyState");
        components.insert("PDSErrorState");
        components.insert("PDSLoadingState");
        components.insert("PDSSuccessState");

        let mut layouts = HashSet::new();
        layouts.insert("ApplicationShell");
        layouts.insert("DashboardLayout");
        layouts.insert("SettingsLayout");
        layouts.insert("AuthLayout");
        layouts.insert("MarketplaceLayout");
        layouts.insert("MarketingLayout");
        layouts.insert("ArticleLayout");

        let mut patterns = HashSet::new();
        patterns.insert("DashboardOverview");
        patterns.insert("AnalyticsDashboard");
        patterns.insert("WorkspaceLauncher");
        patterns.insert("MarketplaceCatalog");
        patterns.insert("SettingsManager");
        patterns.insert("LandingPage");
        patterns.insert("ContactPage");
        patterns.insert("PricingPage");
        patterns.insert("SearchResultsPage");

        Self {
            allowed_components: components,
            allowed_layouts: layouts,
            allowed_patterns: patterns,
        }
    }
}

impl LayoutRegistryValidator {
    /// Validates an entire element tree against the authoritative registry
    pub fn validate(&self, element: &StructuredElement) -> Result<(), LayoutEngineError> {
        match element {
            StructuredElement::Component { component, props } => {
                if !self.allowed_components.contains(component.as_str()) {
                    return Err(LayoutEngineError::UnregisteredComponent(component.clone()));
                }
                self.validate_component_props(component, props)?;
            }
            StructuredElement::Layout { layout, children, .. } => {
                if !self.allowed_layouts.contains(layout.as_str()) {
                    return Err(LayoutEngineError::UnregisteredLayout(layout.clone()));
                }
                for child in children {
                    self.validate(child)?;
                }
            }
            StructuredElement::Pattern { pattern, .. } => {
                if !self.allowed_patterns.contains(pattern.as_str()) {
                    return Err(LayoutEngineError::UnregisteredPattern(pattern.clone()));
                }
            }
        }
        Ok(())
    }

    fn validate_component_props(&self, component: &str, props: &serde_json::Value) -> Result<(), LayoutEngineError> {
        match component {
            "HeroSection" => {
                for req in &["title", "subtitle", "ctaText"] {
                    if props.get(*req).is_none() {
                        return Err(LayoutEngineError::MissingRequiredProp(req.to_string(), component.to_string()));
                    }
                }
            }
            "FeatureGrid" => {
                for req in &["sectionTitle", "items"] {
                    if props.get(*req).is_none() {
                        return Err(LayoutEngineError::MissingRequiredProp(req.to_string(), component.to_string()));
                    }
                }
            }
            "PricingTable" => {
                if props.get("plans").is_none() {
                    return Err(LayoutEngineError::MissingRequiredProp("plans".to_string(), component.to_string()));
                }
            }
            "CallToAction" => {
                for req in &["title", "buttonText"] {
                    if props.get(*req).is_none() {
                        return Err(LayoutEngineError::MissingRequiredProp(req.to_string(), component.to_string()));
                    }
                }
            }
            "ContactForm" => {
                if props.get("headline").is_none() {
                    return Err(LayoutEngineError::MissingRequiredProp("headline".to_string(), component.to_string()));
                }
            }
            "PDSButton" if props.get("label").is_none() => {
                return Err(LayoutEngineError::MissingRequiredProp("label".to_string(), component.to_string()));
            }
            _ => {}
        }
        Ok(())
    }
}

/// Deterministic Layout Compiler
#[derive(Default)]
pub struct DeterministicLayoutEngine {
    validator: LayoutRegistryValidator,
}

impl DeterministicLayoutEngine {
    pub fn new() -> Self {
        Self::default()
    }

    /// Emits structured layout JSON after strict validation
    pub fn emit_json(&self, element: &StructuredElement) -> Result<String, LayoutEngineError> {
        self.validator.validate(element)?;
        serde_json::to_string_pretty(element).map_err(|e| LayoutEngineError::JsonError(e.to_string()))
    }

    /// Parses and validates arbitrary structured JSON layout definition
    pub fn parse_and_validate(&self, json_str: &str) -> Result<StructuredElement, LayoutEngineError> {
        let element: StructuredElement = serde_json::from_str(json_str)
            .map_err(|e| LayoutEngineError::JsonError(e.to_string()))?;
        self.validator.validate(&element)?;
        Ok(element)
    }

    /// Deterministic builder for CRM Dashboard Page
    pub fn build_crm_dashboard(
        &self,
        title: &str,
        stats: &[(&str, &str, &str)],
    ) -> Result<StructuredElement, LayoutEngineError> {
        let mut children = Vec::new();

        // 1. Header
        children.push(StructuredElement::Component {
            component: "PDSHeader".into(),
            props: serde_json::json!({
                "title": title,
                "statusPill": "100% ONLINE"
            }),
        });

        // 2. Stat Cards
        for (stat_title, stat_val, trend) in stats {
            children.push(StructuredElement::Component {
                component: "PDSStatCard".into(),
                props: serde_json::json!({
                    "title": stat_title,
                    "value": stat_val,
                    "trend": trend,
                    "variant": "default"
                }),
            });
        }

        // 3. Chart Card
        children.push(StructuredElement::Component {
            component: "PDSChartCard".into(),
            props: serde_json::json!({
                "title": "Revenue & Deal Pipeline",
                "chartType": "area",
                "timeframe": "30d"
            }),
        });

        // 4. Data Table
        children.push(StructuredElement::Component {
            component: "PDSDataTable".into(),
            props: serde_json::json!({
                "columns": ["ID", "Customer", "Stage", "Value"],
                "rows": [],
                "emptyMessage": "No open deals in queue"
            }),
        });

        let layout = StructuredElement::Layout {
            layout: "DashboardLayout".into(),
            props: serde_json::json!({ "title": title }),
            children,
        };

        self.validator.validate(&layout)?;
        Ok(layout)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_component_emits_json() {
        let engine = DeterministicLayoutEngine::new();
        let btn = StructuredElement::Component {
            component: "PDSButton".into(),
            props: serde_json::json!({ "label": "Save Changes", "variant": "primary" }),
        };
        let json = engine.emit_json(&btn).unwrap();
        assert!(json.contains("Save Changes"));
    }

    #[test]
    fn test_unregistered_component_fails_validation() {
        let engine = DeterministicLayoutEngine::new();
        let invalid = StructuredElement::Component {
            component: "MyFreelyInventedWidget".into(),
            props: serde_json::json!({}),
        };
        let err = engine.emit_json(&invalid).unwrap_err();
        assert_eq!(
            err,
            LayoutEngineError::UnregisteredComponent("MyFreelyInventedWidget".into())
        );
    }

    #[test]
    fn test_unregistered_layout_fails_validation() {
        let engine = DeterministicLayoutEngine::new();
        let invalid = StructuredElement::Layout {
            layout: "ArbitraryFloatingChrome".into(),
            props: serde_json::json!({}),
            children: vec![],
        };
        let err = engine.emit_json(&invalid).unwrap_err();
        assert_eq!(
            err,
            LayoutEngineError::UnregisteredLayout("ArbitraryFloatingChrome".into())
        );
    }

    #[test]
    fn test_missing_required_prop_fails() {
        let engine = DeterministicLayoutEngine::new();
        let hero = StructuredElement::Component {
            component: "HeroSection".into(),
            props: serde_json::json!({ "title": "Incomplete Hero" }), // missing subtitle, ctaText
        };
        let err = engine.emit_json(&hero).unwrap_err();
        assert_eq!(
            err,
            LayoutEngineError::MissingRequiredProp("subtitle".into(), "HeroSection".into())
        );
    }

    #[test]
    fn test_build_crm_dashboard_deterministic_generation() {
        let engine = DeterministicLayoutEngine::new();
        let stats = [
            ("TOTAL DEALS", "128", "+12%"),
            ("REVENUE", "€84,200", "+8%"),
            ("CONTACTS", "342", "+14"),
        ];
        let dashboard = engine.build_crm_dashboard("Proteus Overview", &stats).unwrap();
        let json = engine.emit_json(&dashboard).unwrap();
        assert!(json.contains("DashboardLayout"));
        assert!(json.contains("PDSHeader"));
        assert!(json.contains("PDSStatCard"));
        assert!(json.contains("€84,200"));
    }
}
