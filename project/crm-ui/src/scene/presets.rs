use super::types::*;
use super::document::ProjectDocument;

// ── Dummy document for testing ──

pub fn create_dummy_document() -> ProjectDocument {
    let mut doc = ProjectDocument::new();

    let page = Node {
        id: "page-1".into(),
        name: "Login Form".into(),
        node_type: NodeType::Frame,
        parent_id: None,
        children_ids: vec![],
        styling: Styling {
            background: Some(Rgba { r: 38, g: 38, b: 38, a: 255 }),
            corner_radius: [8., 8., 8., 8.],
            border: Some(Border { width: 1., color: Rgba { r: 60, g: 60, b: 60, a: 255 } }),
            padding: [16., 16., 16., 16.],
            shadow: None,
            opacity: 1.0,
        },
        style: NodeStyle::default(),
        layout: Layout { width: Sizing::Fixed(320.), height: Sizing::Fixed(260.), ..Default::default() },
        position: (100., 100.),
        visible: true,
        locked: false,
        z: 0,
    };

    let title = Node {
        id: "title-1".into(),
        name: "Title".into(),
        node_type: NodeType::Text {
            content: "Enter your details".into(),
            font: FontSpec { family: "Inter".into(), size: 18., weight: 700, color: Rgba { r: 215, g: 215, b: 215, a: 255 } },
        },
        parent_id: None,
        children_ids: vec![],
        styling: Styling::default(),
        style: NodeStyle::default(),
        layout: Layout::default(),
        position: (0., 0.),
        visible: true,
        locked: false,
        z: 1,
    };

    let email_input = Node {
        id: "input-1".into(),
        name: "Email Input".into(),
        node_type: NodeType::TextInput {
            placeholder: "Email address".into(),
            field_type: FieldType::Email,
            bound_entity: None,
            bound_field: None,
        },
        parent_id: None,
        children_ids: vec![],
        styling: Styling::default(),
        style: NodeStyle::default(),
        layout: Layout { width: Sizing::Fixed(288.), height: Sizing::Fixed(36.), ..Default::default() },
        position: (0., 32.),
        visible: true,
        locked: false,
        z: 2,
    };

    let submit_btn = Node {
        id: "btn-1".into(),
        name: "Submit".into(),
        node_type: NodeType::Button { label: "Sign In".into(), style: ButtonStyle::Primary },
        parent_id: None,
        children_ids: vec![],
        styling: Styling::default(),
        style: NodeStyle::default(),
        layout: Layout { width: Sizing::Fixed(288.), height: Sizing::Fixed(40.), ..Default::default() },
        position: (0., 80.),
        visible: true,
        locked: false,
        z: 3,
    };

    // Build tree via arena API
    let _ = doc.add_node(page, None);
    let _ = doc.add_node(title, Some("page-1".into()));
    let _ = doc.add_node(email_input, Some("page-1".into()));
    let _ = doc.add_node(submit_btn, Some("page-1".into()));

    // Page 2 — Dashboard
    let page2 = Node {
        id: "page-2".into(),
        name: "Dashboard".into(),
        node_type: NodeType::Frame,
        parent_id: None,
        children_ids: vec![],
        styling: Styling {
            background: Some(Rgba { r: 30, g: 30, b: 30, a: 255 }),
            corner_radius: [8., 8., 8., 8.],
            border: Some(Border { width: 1., color: Rgba { r: 60, g: 60, b: 60, a: 255 } }),
            padding: [16., 16., 16., 16.],
            shadow: None,
            opacity: 1.0,
        },
        style: NodeStyle::default(),
        layout: Layout { width: Sizing::Fixed(400.), height: Sizing::Fixed(200.), ..Default::default() },
        position: (500., 100.),
        visible: true,
        locked: false,
        z: 0,
    };
    let welcome = Node {
        id: "welcome-1".into(),
        name: "Welcome Text".into(),
        node_type: NodeType::Text {
            content: "Welcome to the Dashboard!".into(),
            font: FontSpec { family: "Inter".into(), size: 24., weight: 700, color: Rgba { r: 215, g: 215, b: 215, a: 255 } },
        },
        parent_id: None,
        children_ids: vec![],
        styling: Styling::default(),
        style: NodeStyle::default(),
        layout: Layout::default(),
        position: (0., 0.),
        visible: true,
        locked: false,
        z: 1,
    };
    let _ = doc.add_node(page2, None);
    let _ = doc.add_node(welcome, Some("page-2".into()));

    // Dummy flow graph: Trigger → NavigateTo page-2
    doc.flow_graph.nodes.insert("f1".into(), crate::flow::FlowNode {
        id: "f1".into(),
        kind: crate::flow::FlowNodeKind::TriggerClick { target_node_id: "btn-1".into() },
        position: (200., 100.),
    });
    doc.flow_graph.nodes.insert("f2".into(), crate::flow::FlowNode {
        id: "f2".into(),
        kind: crate::flow::FlowNodeKind::NavigateTo { page_id: "page-2".into() },
        position: (500., 100.),
    });
    doc.flow_graph.edges.push(crate::flow::FlowEdge {
        from_node: "f1".into(),
        to_node: "f2".into(),
    });

    doc
}

/// Creates a full CRM preset template with multiple pages designed for Desktop canvas.
pub fn create_crm_preset() -> ProjectDocument {
    let mut doc = ProjectDocument::new();

    // Helper to build a styled node
    #[allow(clippy::too_many_arguments)]
    fn mk_page(id: &str, name: &str, x: f32, y: f32, w: f32, h: f32, bg: Rgba, padding: f32) -> Node {
        Node {
            id: id.into(), name: name.into(),
            node_type: NodeType::Frame,
            parent_id: None, children_ids: vec![],
            styling: Styling {
                background: Some(bg),
                corner_radius: [8., 8., 8., 8.],
                border: Some(Border { width: 1., color: Rgba { r: 55, g: 55, b: 55, a: 255 } }),
                padding: [padding, padding, padding, padding],
                shadow: None, opacity: 1.0,
            },
            style: NodeStyle::default(),
            layout: Layout { width: Sizing::Fixed(w), height: Sizing::Fixed(h), ..Default::default() },
            position: (x, y), visible: true, locked: false, z: 0,
        }
    }
    #[allow(clippy::too_many_arguments)]
    fn mk_text(id: &str, content: &str, x: f32, y: f32, size: f32, bold: bool, r: u8, g: u8, b: u8) -> Node {
        Node {
            id: id.into(), name: content.into(),
            node_type: NodeType::Text { content: content.into(), font: FontSpec { family: "Inter".into(), size, weight: if bold { 700 } else { 400 }, color: Rgba { r, g, b, a: 255 } } },
            parent_id: None, children_ids: vec![],
            styling: Styling::default(), style: NodeStyle::default(),
            layout: Layout { width: Sizing::Hug, height: Sizing::Hug, ..Default::default() },
            position: (x, y), visible: true, locked: false, z: 1,
        }
    }
    fn mk_input(id: &str, placeholder: &str, x: f32, y: f32, w: f32, h: f32, ft: FieldType) -> Node {
        Node {
            id: id.into(), name: placeholder.into(),
            node_type: NodeType::TextInput { placeholder: placeholder.into(), field_type: ft, bound_entity: None, bound_field: None },
            parent_id: None, children_ids: vec![],
            styling: Styling::default(), style: NodeStyle::default(),
            layout: Layout { width: Sizing::Fixed(w), height: Sizing::Fixed(h), ..Default::default() },
            position: (x, y), visible: true, locked: false, z: 1,
        }
    }
    #[allow(clippy::too_many_arguments)]
    fn mk_bound_input(id: &str, placeholder: &str, x: f32, y: f32, w: f32, h: f32, ft: FieldType, entity: &str, field: &str) -> Node {
        Node {
            id: id.into(), name: placeholder.into(),
            node_type: NodeType::TextInput {
                placeholder: placeholder.into(),
                field_type: ft,
                bound_entity: Some(entity.into()),
                bound_field: Some(field.into()),
            },
            parent_id: None, children_ids: vec![],
            styling: Styling::default(), style: NodeStyle::default(),
            layout: Layout { width: Sizing::Fixed(w), height: Sizing::Fixed(h), ..Default::default() },
            position: (x, y), visible: true, locked: false, z: 1,
        }
    }
    fn mk_btn(id: &str, label: &str, x: f32, y: f32, w: f32, h: f32, style: ButtonStyle) -> Node {
        Node {
            id: id.into(), name: label.into(),
            node_type: NodeType::Button { label: label.into(), style },
            parent_id: None, children_ids: vec![],
            styling: Styling::default(), style: NodeStyle::default(),
            layout: Layout { width: Sizing::Fixed(w), height: Sizing::Fixed(h), ..Default::default() },
            position: (x, y), visible: true, locked: false, z: 1,
        }
    }
    fn mk_card(id: &str, x: f32, y: f32, w: f32, h: f32, bg: Rgba) -> Node {
        Node {
            id: id.into(), name: "Card".into(),
            node_type: NodeType::Frame,
            parent_id: None, children_ids: vec![],
            styling: Styling {
                background: Some(bg), corner_radius: [10., 10., 10., 10.],
                border: None, padding: [12., 12., 12., 12.],
                shadow: None, opacity: 1.0,
            },
            style: NodeStyle::default(),
            layout: Layout { width: Sizing::Fixed(w), height: Sizing::Fixed(h), ..Default::default() },
            position: (x, y), visible: true, locked: false, z: 1,
        }
    }
    fn mk_dropdown(id: &str, label: &str, opts: &[&str], x: f32, y: f32, w: f32, h: f32) -> Node {
        Node {
            id: id.into(), name: label.into(),
            node_type: NodeType::Dropdown { options: opts.iter().map(|s| s.to_string()).collect(), multiple: false, bound_entity: None, bound_field: None },
            parent_id: None, children_ids: vec![],
            styling: Styling::default(), style: NodeStyle::default(),
            layout: Layout { width: Sizing::Fixed(w), height: Sizing::Fixed(h), ..Default::default() },
            position: (x, y), visible: true, locked: false, z: 1,
        }
    }
    fn mk_checkbox(id: &str, label: &str, x: f32, y: f32) -> Node {
        Node {
            id: id.into(), name: label.into(),
            node_type: NodeType::Checkbox { label: label.into(), bound_entity: None, bound_field: None },
            parent_id: None, children_ids: vec![],
            styling: Styling::default(), style: NodeStyle::default(),
            layout: Layout { width: Sizing::Hug, height: Sizing::Hug, ..Default::default() },
            position: (x, y), visible: true, locked: false, z: 1,
        }
    }

    let gray = Rgba { r: 28, g: 28, b: 28, a: 255 };
    let dark = Rgba { r: 22, g: 22, b: 22, a: 255 };
    let card = Rgba { r: 35, g: 35, b: 35, a: 255 };
    let accent = Rgba { r: 52, g: 152, b: 219, a: 255 };
    let green = Rgba { r: 46, g: 204, b: 113, a: 255 };
    let orange = Rgba { r: 230, g: 126, b: 34, a: 255 };

    // ── PAGE 1: LOGIN ──
    let p1 = mk_page("crm-login", "Login", 40., 100., 400., 500., dark, 24.);
    let _ = doc.add_node(p1, None);
    // Logo area
    let _ = doc.add_node(mk_text("login-logo", "✦ Proteus CRM", 24., 20., 28., true, 52, 152, 219), Some("crm-login".into()));
    let _ = doc.add_node(mk_text("login-sub", "Sign in to your workspace", 24., 58., 13., false, 130, 130, 130), Some("crm-login".into()));
    // Form fields
    let _ = doc.add_node(mk_input("login-email", "Email address", 24., 100., 352., 42., FieldType::Email), Some("crm-login".into()));
    let _ = doc.add_node(mk_input("login-pass", "Password", 24., 154., 352., 42., FieldType::Password), Some("crm-login".into()));
    let _ = doc.add_node(mk_checkbox("login-remember", "Remember me", 24., 210.), Some("crm-login".into()));
    let _ = doc.add_node(mk_text("login-forgot", "Forgot password?", 220., 212., 11., false, 52, 152, 219), Some("crm-login".into()));
    let _ = doc.add_node(mk_btn("login-btn", "Sign In →", 24., 246., 352., 44., ButtonStyle::Primary), Some("crm-login".into()));
    // Divider
    let _ = doc.add_node(mk_text("login-or", "───  or continue with  ───", 100., 310., 10., false, 100, 100, 100), Some("crm-login".into()));
    // Social buttons
    let _ = doc.add_node(mk_btn("login-google", "⊙  Google", 24., 340., 165., 36., ButtonStyle::Secondary), Some("crm-login".into()));
    let _ = doc.add_node(mk_btn("login-github", "○  GitHub", 211., 340., 165., 36., ButtonStyle::Secondary), Some("crm-login".into()));
    let _ = doc.add_node(mk_text("login-signup", "Don't have an account?  Sign up", 80., 400., 12., false, 130, 130, 130), Some("crm-login".into()));

    // ── PAGE 2: DASHBOARD ──
    let p2 = mk_page("crm-dash", "Dashboard", 500., 100., 800., 600., gray, 20.);
    let _ = doc.add_node(p2, None);
    // Header
    let _ = doc.add_node(mk_text("dash-head", "📊  Dashboard", 20., 16., 22., true, 215, 215, 215), Some("crm-dash".into()));
    let _ = doc.add_node(mk_text("dash-date", "Today • July 20, 2026", 20., 46., 11., false, 130, 130, 130), Some("crm-dash".into()));
    let _ = doc.add_node(mk_btn("dash-settings", "⚙ Settings", 670., 14., 110., 32., ButtonStyle::Ghost), Some("crm-dash".into()));
    // Stats row
    let _ = doc.add_node(mk_card("stat-1", 20., 72., 180., 90., card), Some("crm-dash".into()));
    let _ = doc.add_node(mk_text("stat-1v", "128", 32., 82., 28., true, 52, 152, 219), Some("crm-dash".into()));
    let _ = doc.add_node(mk_text("stat-1l", "Active Deals", 32., 116., 10., false, 130, 130, 130), Some("crm-dash".into()));
    let _ = doc.add_node(mk_card("stat-2", 215., 72., 180., 90., card), Some("crm-dash".into()));
    let _ = doc.add_node(mk_text("stat-2v", "$84,200", 227., 82., 28., true, 46, 204, 113), Some("crm-dash".into()));
    let _ = doc.add_node(mk_text("stat-2l", "Revenue Pipeline", 227., 116., 10., false, 130, 130, 130), Some("crm-dash".into()));
    let _ = doc.add_node(mk_card("stat-3", 410., 72., 180., 90., card), Some("crm-dash".into()));
    let _ = doc.add_node(mk_text("stat-3v", "342", 422., 82., 28., true, 230, 126, 34), Some("crm-dash".into()));
    let _ = doc.add_node(mk_text("stat-3l", "Total Contacts", 422., 116., 10., false, 130, 130, 130), Some("crm-dash".into()));
    let _ = doc.add_node(mk_card("stat-4", 605., 72., 180., 90., card), Some("crm-dash".into()));
    let _ = doc.add_node(mk_text("stat-4v", "89%", 617., 82., 28., true, 231, 76, 60), Some("crm-dash".into()));
    let _ = doc.add_node(mk_text("stat-4l", "Win Rate", 617., 116., 10., false, 130, 130, 130), Some("crm-dash".into()));
    // Activity
    let _ = doc.add_node(mk_card("act-box", 20., 180., 380., 180., card), Some("crm-dash".into()));
    let _ = doc.add_node(mk_text("act-h", "📋 Recent Activity", 32., 192., 13., true, 215, 215, 215), Some("crm-dash".into()));
    let _ = doc.add_node(mk_text("act-1", "● John Smith — Deal moved to Proposal", 32., 216., 10., false, 200, 200, 200), Some("crm-dash".into()));
    let _ = doc.add_node(mk_text("act-2", "● Jane Doe — New contact added", 32., 234., 10., false, 200, 200, 200), Some("crm-dash".into()));
    let _ = doc.add_node(mk_text("act-3", "● Widgets Co — Email sent", 32., 252., 10., false, 200, 200, 200), Some("crm-dash".into()));
    let _ = doc.add_node(mk_text("act-4", "● Meeting scheduled with Bob Wilson", 32., 270., 10., false, 200, 200, 200), Some("crm-dash".into()));
    let _ = doc.add_node(mk_text("act-5", "● Contract uploaded — Enterprise Plan", 32., 288., 10., false, 200, 200, 200), Some("crm-dash".into()));
    // Pipeline summary
    let _ = doc.add_node(mk_card("pipe-box", 415., 180., 370., 180., card), Some("crm-dash".into()));
    let _ = doc.add_node(mk_text("pipe-h", "▤  Pipeline Overview", 427., 192., 13., true, 215, 215, 215), Some("crm-dash".into()));
    let _ = doc.add_node(mk_text("pipe-1", "New:  24 deals  —  $18,200", 427., 216., 10., false, 200, 200, 200), Some("crm-dash".into()));
    let _ = doc.add_node(mk_text("pipe-2", "Qualified:  18 deals  —  $24,500", 427., 234., 10., false, 200, 200, 200), Some("crm-dash".into()));
    let _ = doc.add_node(mk_text("pipe-3", "Proposal:  12 deals  —  $31,000", 427., 252., 10., false, 200, 200, 200), Some("crm-dash".into()));
    let _ = doc.add_node(mk_text("pipe-4", "Negotiation:  5 deals  —  $10,500", 427., 270., 10., false, 200, 200, 200), Some("crm-dash".into()));
    // Quick actions
    let _ = doc.add_node(mk_btn("qa-1", "+  New Contact", 20., 380., 180., 38., ButtonStyle::Primary), Some("crm-dash".into()));
    let _ = doc.add_node(mk_btn("qa-2", "+  New Deal", 215., 380., 180., 38., ButtonStyle::Secondary), Some("crm-dash".into()));
    let _ = doc.add_node(mk_btn("qa-3", "📧  Send Campaign", 410., 380., 180., 38., ButtonStyle::Secondary), Some("crm-dash".into()));

    // ── PAGE 3: CONTACTS LIST ──
    let p3 = mk_page("crm-contacts", "Contacts", 1360., 100., 700., 720., gray, 16.);
    let _ = doc.add_node(p3, None);
    let _ = doc.add_node(mk_text("con-head", "◎  Contacts", 16., 16., 20., true, 215, 215, 215), Some("crm-contacts".into()));
    let _ = doc.add_node(mk_text("con-count", "342 total • 12 added this week", 16., 42., 10., false, 130, 130, 130), Some("crm-contacts".into()));
    let _ = doc.add_node(mk_btn("con-back", "← Dashboard", 560., 14., 120., 32., ButtonStyle::Ghost), Some("crm-contacts".into()));
    // Search & filter
    let _ = doc.add_node(mk_input("con-search", "🔍  Search contacts...", 16., 66., 480., 36., FieldType::Text), Some("crm-contacts".into()));
    let _ = doc.add_node(mk_dropdown("con-filter", "Filter: All", &["All", "VIP", "Lead", "Partner", "Customer"], 508., 66., 180., 36.), Some("crm-contacts".into()));
    // Table header
    let _ = doc.add_node(mk_card("con-th", 16., 116., 668., 28., dark), Some("crm-contacts".into()));
    let _ = doc.add_node(mk_text("con-th-n", "Name", 28., 121., 10., true, 130, 130, 130), Some("crm-contacts".into()));
    let _ = doc.add_node(mk_text("con-th-e", "Email", 158., 121., 10., true, 130, 130, 130), Some("crm-contacts".into()));
    let _ = doc.add_node(mk_text("con-th-p", "Phone", 318., 121., 10., true, 130, 130, 130), Some("crm-contacts".into()));
    let _ = doc.add_node(mk_text("con-th-c", "Company", 458., 121., 10., true, 130, 130, 130), Some("crm-contacts".into()));
    // Table rows
    let rows = [
        ("John Smith", "john@acme.com", "555-0100", "Acme Inc", "VIP"),
        ("Jane Doe", "jane@widgets.co", "555-0200", "Widgets Co", "Lead"),
        ("Bob Wilson", "bob@example.com", "555-0300", "Example LLC", "Partner"),
        ("Alice Brown", "alice@test.io", "555-0400", "Test IO", "Customer"),
        ("Charlie Davis", "charlie@demo.org", "555-0500", "Demo Org", "Lead"),
        ("Eve Martin", "eve@sample.com", "555-0600", "Sample Inc", "VIP"),
    ];
    let mut ry = 152.;
    for (i, (n, e, p, c, tag)) in rows.iter().enumerate() {
        let row_bg = if i % 2 == 0 { card } else { Rgba { r: 40, g: 40, b: 40, a: 255 } };
        let _ = doc.add_node(mk_card(&format!("con-r{}", i), 16., ry, 668., 32., row_bg), Some("crm-contacts".into()));
        let _ = doc.add_node(mk_text(&format!("con-{}-n", i), n, 22., ry + 8., 11., false, 215, 215, 215), Some("crm-contacts".into()));
        let _ = doc.add_node(mk_text(&format!("con-{}-e", i), e, 150., ry + 8., 10., false, 180, 180, 180), Some("crm-contacts".into()));
        let _ = doc.add_node(mk_text(&format!("con-{}-p", i), p, 310., ry + 8., 10., false, 180, 180, 180), Some("crm-contacts".into()));
        let _ = doc.add_node(mk_text(&format!("con-{}-c", i), c, 450., ry + 8., 10., false, 180, 180, 180), Some("crm-contacts".into()));
        let _ = doc.add_node(mk_text(&format!("con-{}-t", i), tag, 620., ry + 8., 9., true, 52, 152, 219), Some("crm-contacts".into()));
        ry += 38.;
    }

    // Quick Add Contact Form (Live Form-to-SQLite Insert)
    let _ = doc.add_node(mk_card("con-add-card", 16., 390., 668., 92., card), Some("crm-contacts".into()));
    let _ = doc.add_node(mk_text("con-add-h", "+ Quick Add Contact (Live SQLite Form)", 28., 400., 11., true, 52, 152, 219), Some("crm-contacts".into()));
    let _ = doc.add_node(mk_bound_input("new-con-name", "Full Name", 28., 428., 160., 36., FieldType::Text, "contacts", "name"), Some("crm-contacts".into()));
    let _ = doc.add_node(mk_bound_input("new-con-email", "Email Address", 198., 428., 180., 36., FieldType::Email, "contacts", "email"), Some("crm-contacts".into()));
    let _ = doc.add_node(mk_bound_input("new-con-comp", "Company", 388., 428., 140., 36., FieldType::Text, "contacts", "company"), Some("crm-contacts".into()));
    let _ = doc.add_node(mk_btn("con-save-btn", "Add Contact ✓", 538., 428., 136., 36., ButtonStyle::Primary), Some("crm-contacts".into()));

    // Data-bound Live SQLite Table Widget
    let _ = doc.add_node(Node {
        id: "con-table".into(),
        name: "Contacts SQLite Table".into(),
        node_type: NodeType::Table { bound_entity: Some("contacts".into()), columns: vec!["ID".into(), "Name".into(), "Email".into(), "Company".into()] },
        parent_id: Some("crm-contacts".into()),
        children_ids: vec![],
        styling: Styling {
            background: Some(card), corner_radius: [8., 8., 8., 8.],
            border: None, padding: [10., 10., 10., 10.], shadow: None, opacity: 1.0,
        },
        style: NodeStyle { border_radius: 8.0, ..NodeStyle::default() },
        layout: Layout { width: Sizing::Fixed(668.), height: Sizing::Fixed(200.), ..Layout::default() },
        position: (16., 495.), visible: true, locked: false, z: 1,
    }, Some("crm-contacts".into()));

    // ── PAGE 4: CONTACT DETAIL ──
    let p4 = mk_page("crm-contact-detail", "Contact Detail", 1360., 840., 700., 500., gray, 16.);
    let _ = doc.add_node(p4, None);
    let _ = doc.add_node(mk_text("cd-head", "👤  John Smith", 16., 16., 20., true, 215, 215, 215), Some("crm-contact-detail".into()));
    let _ = doc.add_node(mk_text("cd-job", "CEO at Acme Inc •  john@acme.com", 16., 42., 11., false, 130, 130, 130), Some("crm-contact-detail".into()));
    let _ = doc.add_node(mk_btn("cd-back", "← Contacts", 560., 14., 120., 32., ButtonStyle::Ghost), Some("crm-contact-detail".into()));
    // Info card
    let _ = doc.add_node(mk_card("cd-info", 16., 68., 320., 160., card), Some("crm-contact-detail".into()));
    let _ = doc.add_node(mk_text("cd-info-h", "Contact Information", 28., 80., 12., true, 215, 215, 215), Some("crm-contact-detail".into()));
    let _ = doc.add_node(mk_text("cd-p", "📞  (555) 555-0100", 28., 102., 11., false, 200, 200, 200), Some("crm-contact-detail".into()));
    let _ = doc.add_node(mk_text("cd-e", "📧  john@acme.com", 28., 120., 11., false, 200, 200, 200), Some("crm-contact-detail".into()));
    let _ = doc.add_node(mk_text("cd-w", "🌐  acme.com", 28., 138., 11., false, 200, 200, 200), Some("crm-contact-detail".into()));
    let _ = doc.add_node(mk_text("cd-loc", "📍  San Francisco, CA", 28., 156., 11., false, 200, 200, 200), Some("crm-contact-detail".into()));
    let _ = doc.add_node(mk_text("cd-tags", "🏷  VIP · Enterprise · Tech", 28., 174., 11., false, 52, 152, 219), Some("crm-contact-detail".into()));
    // Deals card
    let _ = doc.add_node(mk_card("cd-deals", 350., 68., 334., 160., card), Some("crm-contact-detail".into()));
    let _ = doc.add_node(mk_text("cd-deals-h", "💰  Active Deals", 362., 80., 12., true, 215, 215, 215), Some("crm-contact-detail".into()));
    let _ = doc.add_node(mk_text("cd-d1", "Enterprise License — $12,000", 362., 106., 11., false, 46, 204, 113), Some("crm-contact-detail".into()));
    let _ = doc.add_node(mk_text("cd-d1s", "Stage: Proposal  •  Close: Aug 2026", 362., 124., 10., false, 130, 130, 130), Some("crm-contact-detail".into()));
    let _ = doc.add_node(mk_text("cd-d2", "Support Contract — $4,800", 362., 146., 11., false, 46, 204, 113), Some("crm-contact-detail".into()));
    let _ = doc.add_node(mk_text("cd-d2s", "Stage: Negotiation  •  Close: Sep 2026", 362., 164., 10., false, 130, 130, 130), Some("crm-contact-detail".into()));
    // Notes
    let _ = doc.add_node(mk_card("cd-notes", 16., 244., 668., 120., card), Some("crm-contact-detail".into()));
    let _ = doc.add_node(mk_text("cd-notes-h", "📝  Notes & Activity", 28., 256., 12., true, 215, 215, 215), Some("crm-contact-detail".into()));
    let _ = doc.add_node(mk_text("cd-n1", "📌  Initial meeting — interested in Enterprise plan. Follow up with pricing.", 28., 282., 11., false, 200, 200, 200), Some("crm-contact-detail".into()));
    let _ = doc.add_node(mk_text("cd-n1d", "Jul 18, 2026  •  by you", 28., 300., 9., false, 130, 130, 130), Some("crm-contact-detail".into()));
    let _ = doc.add_node(mk_text("cd-n2", "📌  Sent proposal with 3-year discount. Awaiting feedback.", 28., 322., 11., false, 200, 200, 200), Some("crm-contact-detail".into()));
    let _ = doc.add_node(mk_text("cd-n2d", "Jul 15, 2026  •  by you", 28., 340., 9., false, 130, 130, 130), Some("crm-contact-detail".into()));

    // ── PAGE 5: PIPELINE / KANBAN ──
    let p5 = mk_page("crm-pipeline", "Pipeline", 2120., 100., 760., 600., gray, 12.);
    let _ = doc.add_node(p5, None);
    let _ = doc.add_node(mk_text("pipe-head", "▤  Sales Pipeline", 12., 12., 20., true, 215, 215, 215), Some("crm-pipeline".into()));
    let _ = doc.add_node(mk_btn("pipe-back", "← Dashboard", 620., 12., 120., 30., ButtonStyle::Ghost), Some("crm-pipeline".into()));
    // Columns
    let cols = [("New", orange), ("Qualified", accent), ("Proposal", green), ("Negotiation", Rgba{r:155,g:89,b:182,a:255}), ("Closed Won", Rgba{r:39,g:174,b:96,a:255})];
    let col_w = 144.;
    let col_gap = 4.;
    for (ci, (cname, ccolor)) in cols.iter().enumerate() {
        let cx = 12. + ci as f32 * (col_w + col_gap);
        let _ = doc.add_node(mk_card(&format!("pipe-col{}", ci), cx, 50., col_w, 420., dark), Some("crm-pipeline".into()));
        // Column header
        let _ = doc.add_node(mk_text(&format!("pipe-col{}h", ci), cname, cx + 8., 58., 12., true, ccolor.r, ccolor.g, ccolor.b), Some("crm-pipeline".into()));
        // Cards
        let deals: &[(&str, &str)] = match ci {
            0 => &[("Acme Inc — $12K", "Contact: John Smith"), ("Widgets Co — $5K", "Contact: Jane Doe")],
            1 => &[("Example LLC — $3K", "Contact: Bob Wilson"), ("Tech Corp — $8K", "Contact: Alice Brown")],
            2 => &[("Global Inc — $25K", "3-year enterprise"), ("StartupXYZ — $6K", "SaaS pilot")],
            3 => &[("MegaCorp — $50K", "Final review"), ("DataFlow — $15K", "Legal review")],
            _ => &[("Acme Inc — $12K", "Signed ✅")],
        };
        for (di, (deal_title, deal_sub)) in deals.iter().enumerate() {
            let dy = 80. + di as f32 * 72.;
            let _ = doc.add_node(mk_card(&format!("pipe-{}-d{}", ci, di), cx + 6., dy, col_w - 12., 62., card), Some("crm-pipeline".into()));
            let _ = doc.add_node(mk_text(&format!("pipe-{}-t{}", ci, di), deal_title, cx + 14., dy + 8., 10., true, 215, 215, 215), Some("crm-pipeline".into()));
            let _ = doc.add_node(mk_text(&format!("pipe-{}-s{}", ci, di), deal_sub, cx + 14., dy + 26., 9., false, 130, 130, 130), Some("crm-pipeline".into()));
            let _ = doc.add_node(mk_text(&format!("pipe-{}-v{}", ci, di), "★", cx + col_w - 28., dy + 8., 10., false, 243, 156, 18), Some("crm-pipeline".into()));
        }
    }
    // Add lead button
    let _ = doc.add_node(mk_btn("pipe-add", "+  Add Deal", 12., 478., 144., 36., ButtonStyle::Primary), Some("crm-pipeline".into()));

    // ── PAGE 6: SETTINGS ──
    let p6 = mk_page("crm-settings", "Settings", 2940., 100., 500., 500., gray, 20.);
    let _ = doc.add_node(p6, None);
    let _ = doc.add_node(mk_text("set-head", "⚙  Settings", 20., 16., 20., true, 215, 215, 215), Some("crm-settings".into()));
    let _ = doc.add_node(mk_btn("set-back", "← Dashboard", 360., 14., 120., 32., ButtonStyle::Ghost), Some("crm-settings".into()));
    // Profile section
    let _ = doc.add_node(mk_card("set-profile", 20., 50., 460., 150., card), Some("crm-settings".into()));
    let _ = doc.add_node(mk_text("set-ph", "👤  Profile", 32., 62., 14., true, 215, 215, 215), Some("crm-settings".into()));
    let _ = doc.add_node(mk_text("set-pl", "Full Name", 32., 86., 10., false, 130, 130, 130), Some("crm-settings".into()));
    let _ = doc.add_node(mk_input("set-name", "Alex Johnson", 32., 100., 200., 32., FieldType::Text), Some("crm-settings".into()));
    let _ = doc.add_node(mk_text("set-pe", "Email", 248., 86., 10., false, 130, 130, 130), Some("crm-settings".into()));
    let _ = doc.add_node(mk_input("set-email", "alex@proteus.app", 248., 100., 200., 32., FieldType::Email), Some("crm-settings".into()));
    // Preferences
    let _ = doc.add_node(mk_card("set-prefs", 20., 216., 460., 120., card), Some("crm-settings".into()));
    let _ = doc.add_node(mk_text("set-prefh", "🎨  Preferences", 32., 228., 14., true, 215, 215, 215), Some("crm-settings".into()));
    let _ = doc.add_node(mk_dropdown("set-theme", "Theme: Dark", &["Dark", "Light", "System"], 32., 250., 180., 32.), Some("crm-settings".into()));
    let _ = doc.add_node(mk_dropdown("set-lang", "Language: English", &["English", "Greek", "Spanish", "French"], 224., 250., 180., 32.), Some("crm-settings".into()));
    let _ = doc.add_node(mk_checkbox("set-notif", "Enable email notifications", 32., 292.), Some("crm-settings".into()));
    // Save
    let _ = doc.add_node(mk_btn("set-save", "💾  Save Changes", 20., 360., 200., 40., ButtonStyle::Primary), Some("crm-settings".into()));

    // ── PAGE 7: NAVIGATION BAR (always visible, no scroll) ──
    let p7 = mk_page("crm-nav", "Navigation", 40., 630., 400., 50., Rgba{r:30,g:30,b:30,a:255}, 8.);
    let _ = doc.add_node(p7, None);
    let nav_items = [("📊", "Dashboard"), ("◎", "Contacts"), ("▤", "Pipeline"), ("⚙", "Settings"), ("👤", "Profile")];
    let nav_w = 76.;
    for (ni, (icon, label)) in nav_items.iter().enumerate() {
        let nx = 8. + ni as f32 * nav_w;
        let _ = doc.add_node(mk_btn(&format!("nav-{}", ni), &format!("{} {}", icon, label), nx, 8., 72., 34., ButtonStyle::Ghost), Some("crm-nav".into()));
    }

    // ── Flow graph screen navigation and live insert actions ──
    doc.set_button_action("login-btn", Some("crm-dash".into()), None);
    doc.set_button_action("qa-1", Some("crm-contacts".into()), None);
    doc.set_button_action("qa-2", Some("crm-pipeline".into()), None);
    doc.set_button_action("dash-settings", Some("crm-settings".into()), None);
    doc.set_button_action("con-back", Some("crm-dash".into()), None);
    doc.set_button_action("cd-back", Some("crm-contacts".into()), None);
    doc.set_button_action("pipe-back", Some("crm-dash".into()), None);
    doc.set_button_action("set-back", Some("crm-dash".into()), None);
    doc.set_button_action("con-save-btn", None, Some("contacts".into()));
    doc.set_button_action("set-save", None, Some("settings".into()));
    doc.set_button_action("nav-0", Some("crm-dash".into()), None);
    doc.set_button_action("nav-1", Some("crm-contacts".into()), None);
    doc.set_button_action("nav-2", Some("crm-pipeline".into()), None);
    doc.set_button_action("nav-3", Some("crm-settings".into()), None);
    doc.set_button_action("nav-4", Some("crm-contact-detail".into()), None);

    doc
}
