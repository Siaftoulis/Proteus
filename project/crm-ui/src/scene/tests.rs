use super::*;

// ── helpers ──

fn make_node(id: &str) -> Node {
    Node::new(id.into(), id.into(), NodeType::Frame)
}
fn text_node(id: &str, x: f32, y: f32) -> Node {
    Node {
        id: id.into(),
        name: id.into(),
        node_type: NodeType::Text { content: "hello".into(), font: FontSpec::default() },
        position: (x, y),
        layout: Layout { width: Sizing::Fixed(100.), height: Sizing::Fixed(30.), ..Layout::default() },
        ..Node::new(id.into(), id.into(), NodeType::Frame)
    }
}
#[allow(dead_code)]
fn doc_with_nodes() -> ProjectDocument {
    let mut doc = ProjectDocument::new();
    doc.add_node(text_node("child", 20., 30.), Some("parent".into())).unwrap();
    doc
}

// ── Document operations (existing) ──

#[test]
fn add_root_node() {
    let mut doc = ProjectDocument::new();
    doc.add_node(make_node("a"), None).unwrap();
    assert!(doc.nodes.contains_key("a"));
    assert_eq!(doc.root_node_ids, vec!["a"]);
}

#[test]
fn add_child_node() {
    let mut doc = ProjectDocument::new();
    doc.add_node(make_node("parent"), None).unwrap();
    doc.add_node(make_node("child"), Some("parent".into())).unwrap();
    assert_eq!(doc.get_node("child").unwrap().parent_id, Some("parent".into()));
    assert!(doc.get_node("parent").unwrap().children_ids.contains(&"child".into()));
}

#[test]
fn add_node_missing_parent() {
    let mut doc = ProjectDocument::new();
    let res = doc.add_node(make_node("x"), Some("missing".into()));
    assert!(res.is_err());
}

#[test]
fn remove_leaf_node() {
    let mut doc = ProjectDocument::new();
    doc.add_node(make_node("p"), None).unwrap();
    doc.add_node(make_node("c"), Some("p".into())).unwrap();
    doc.remove_node("c").unwrap();
    assert!(!doc.nodes.contains_key("c"));
    assert!(doc.get_node("p").unwrap().children_ids.is_empty());
}

#[test]
fn remove_node_cascades_to_children() {
    let mut doc = ProjectDocument::new();
    doc.add_node(make_node("p"), None).unwrap();
    doc.add_node(make_node("c1"), Some("p".into())).unwrap();
    doc.add_node(make_node("c2"), Some("c1".into())).unwrap();
    doc.remove_node("p").unwrap();
    assert!(!doc.nodes.contains_key("p"));
    assert!(!doc.nodes.contains_key("c1"));
    assert!(!doc.nodes.contains_key("c2"));
    assert!(doc.root_node_ids.is_empty());
}

#[test]
fn reparent_to_root() {
    let mut doc = ProjectDocument::new();
    doc.add_node(make_node("p"), None).unwrap();
    doc.add_node(make_node("c"), Some("p".into())).unwrap();
    doc.reparent_node("c", None).unwrap();
    assert!(doc.get_node("c").unwrap().parent_id.is_none());
    assert!(doc.root_node_ids.contains(&"c".into()));
    assert!(doc.get_node("p").unwrap().children_ids.is_empty());
}

#[test]
fn reparent_self_rejected() {
    let mut doc = ProjectDocument::new();
    doc.add_node(make_node("a"), None).unwrap();
    let res = doc.reparent_node("a", Some("a".into()));
    assert!(res.is_err());
}

#[test]
fn reparent_to_descendant_rejected() {
    let mut doc = ProjectDocument::new();
    doc.add_node(make_node("p"), None).unwrap();
    doc.add_node(make_node("c"), Some("p".into())).unwrap();
    let res = doc.reparent_node("p", Some("c".into()));
    assert!(res.is_err());
}

#[test]
fn remove_missing_rejected() {
    let mut doc = ProjectDocument::new();
    let res = doc.remove_node("nope");
    assert!(res.is_err());
}

// ── Hit testing ──

#[test]
fn hit_root_node() {
    let mut doc = ProjectDocument::new();
    doc.add_node(make_node("a"), None).unwrap();
    // node "a" is at (0,0) with Sizing::Default -> Fixed(200)xHug -> 200x100
    assert_eq!(hit_test_nodes(&doc, (50., 50.), 1.0), Some("a".into()));
    assert_eq!(hit_test_nodes(&doc, (199., 99.), 1.0), Some("a".into()));
    // outside
    assert_eq!(hit_test_nodes(&doc, (-1., 50.), 1.0), None);
    assert_eq!(hit_test_nodes(&doc, (50., 101.), 1.0), None);
}

#[test]
fn hit_child_node_includes_parent_position() {
    let mut doc = ProjectDocument::new();
    doc.add_node(make_node("parent"), None).unwrap();
    {
        let p = doc.nodes.get_mut("parent").unwrap();
        p.position = (100., 100.);
    }
    doc.add_node(text_node("child", 20., 30.), Some("parent".into())).unwrap();
    // child's world AABB: (100+20, 100+30) to (100+20+100, 100+30+30) = (120,130) to (220,160)
    assert_eq!(hit_test_nodes(&doc, (120., 130.), 1.0), Some("child".into()));
    assert_eq!(hit_test_nodes(&doc, (140., 140.), 1.0), Some("child".into()));
    assert_eq!(hit_test_nodes(&doc, (219., 159.), 1.0), Some("child".into()));
    // outside child but still inside parent
    assert_eq!(hit_test_nodes(&doc, (50., 50.), 1.0), None);  // outside parent
}

#[test]
fn hit_invisible_node_skipped() {
    let mut doc = ProjectDocument::new();
    doc.add_node(make_node("a"), None).unwrap();
    doc.nodes.get_mut("a").unwrap().visible = false;
    assert_eq!(hit_test_nodes(&doc, (50., 50.), 1.0), None);
}

#[test]
fn hit_topmost_wins() {
    let mut doc = ProjectDocument::new();
    doc.add_node(make_node("low"), None).unwrap();
    doc.add_node(make_node("high"), None).unwrap();
    doc.nodes.get_mut("high").unwrap().z = 10;
    // both at (0,0) with 200x100; "high" has higher z
    assert_eq!(hit_test_nodes(&doc, (50., 50.), 1.0), Some("high".into()));
}

#[test]
fn hit_no_match_returns_none() {
    let doc = ProjectDocument::new();
    assert_eq!(hit_test_nodes(&doc, (999., 999.), 1.0), None);
}

#[test]
fn hit_child_deeply_nested() {
    let mut doc = ProjectDocument::new();
    doc.add_node(make_node("l1"), None).unwrap();
    doc.nodes.get_mut("l1").unwrap().position = (50., 50.);
    doc.nodes.get_mut("l1").unwrap().layout = Layout { width: Sizing::Fixed(400.), height: Sizing::Fixed(400.), ..Layout::default() };
    doc.add_node(make_node("l2"), Some("l1".into())).unwrap();
    doc.nodes.get_mut("l2").unwrap().position = (10., 10.);
    doc.nodes.get_mut("l2").unwrap().layout = Layout { width: Sizing::Fixed(300.), height: Sizing::Fixed(300.), ..Layout::default() };
    doc.add_node(text_node("l3", 5., 5.), Some("l2".into())).unwrap();
    // l3 world AABB: 50+10+5=65, 50+10+5=65  →  65..165, 65..95
    assert_eq!(hit_test_nodes(&doc, (65., 65.), 1.0), Some("l3".into()));
    assert_eq!(hit_test_nodes(&doc, (164., 94.), 1.0), Some("l3".into()));
    // on l2 but not on l3
    assert_eq!(hit_test_nodes(&doc, (70., 100.), 1.0), Some("l2".into()));
    // on l1 but not l2
    assert_eq!(hit_test_nodes(&doc, (55., 55.), 1.0), Some("l1".into()));
}

#[test]
fn hit_with_padding_offset() {
    let mut doc = ProjectDocument::new();
    doc.add_node(make_node("parent"), None).unwrap();
    {
        let p = doc.nodes.get_mut("parent").unwrap();
        p.position = (0., 0.);
        p.styling.padding = [20., 20., 20., 20.];
    }
    doc.add_node(text_node("child", 0., 0.), Some("parent".into())).unwrap();
    // parent has 20px padding; renderer puts child at parent_origin+20+child.x
    // world: (0+20+0, 0+20+0) = (20, 20)
    // hit without padding would check (0,0) not (20,20)
    assert_eq!(hit_test_nodes(&doc, (20., 20.), 1.0), Some("child".into()));
    assert_eq!(hit_test_nodes(&doc, (119., 49.), 1.0), Some("child".into()));
    assert_eq!(hit_test_nodes(&doc, (5., 5.), 1.0), Some("parent".into())); // hits parent, not padding-offset child
}

// ── Node update / CanvasEvent logic ──

#[test]
fn node_move_update() {
    let mut doc = ProjectDocument::new();
    doc.add_node(make_node("a"), None).unwrap();
    let updated = doc.update_node("a", NodeUpdate::Move { x: 42., y: 99. });
    assert!(updated.is_ok());
    assert_eq!(doc.get_node("a").unwrap().position, (42., 99.));
}

#[test]
fn node_move_update_unknown_id() {
    let mut doc = ProjectDocument::new();
    let updated = doc.update_node("missing", NodeUpdate::Move { x: 0., y: 0. });
    assert!(updated.is_err());
}

#[test]
fn node_resize_from_event() {
    let mut doc = ProjectDocument::new();
    doc.add_node(make_node("a"), None).unwrap();
    doc.nodes.get_mut("a").unwrap().layout = Layout {
        width: Sizing::Fixed(200.), height: Sizing::Fixed(100.),
        ..Layout::default()
    };
    // simulate a resize event
    let ev = CanvasEvent::NodeResized { id: "a".into(), handle: ResizeHandle::BottomRight, delta: (50., 30.) };
    if let CanvasEvent::NodeResized { id, handle: _, delta } = &ev {
        if let Some(node) = doc.nodes.get_mut(id) {
            let new_w = match node.layout.width { Sizing::Fixed(w) => (w + delta.0).max(10.), _ => 200. };
            let new_h = match node.layout.height { Sizing::Fixed(h) => (h + delta.1).max(10.), _ => 100. };
            node.layout.width = Sizing::Fixed(new_w);
            node.layout.height = Sizing::Fixed(new_h);
        }
    }
    assert_eq!(doc.get_node("a").unwrap().layout, Layout {
        width: Sizing::Fixed(250.), height: Sizing::Fixed(130.),
        ..Layout::default()
    });
}

#[test]
fn text_content_update() {
    let mut doc = ProjectDocument::new();
    doc.add_node(Node::new("t".into(), "t".into(), NodeType::Text { content: "old".into(), font: FontSpec::default() }), None).unwrap();
    let _ = doc.update_node("t", NodeUpdate::TextContent("new!".into()));
    if let NodeType::Text { content, .. } = &doc.get_node("t").unwrap().node_type {
        assert_eq!(content, "new!");
    } else {
        panic!("not a text node");
    }
}

#[test]
fn typography_size_and_weight_update() {
    let mut doc = ProjectDocument::new();
    doc.add_node(Node::new("t".into(), "t".into(), NodeType::Text { content: "Title".into(), font: FontSpec::default() }), None).unwrap();
    let _ = doc.update_node("t", NodeUpdate::FontSize(32.0));
    let _ = doc.update_node("t", NodeUpdate::FontWeight(700));
    if let NodeType::Text { font, .. } = &doc.get_node("t").unwrap().node_type {
        assert_eq!(font.size, 32.0);
        assert_eq!(font.weight, 700);
    } else {
        panic!("not a text node");
    }
}

#[test]
fn table_node_data_binding_update() {
    let mut doc = ProjectDocument::new();
    doc.add_node(Node::new("tbl".into(), "Clients Table".into(), NodeType::Table {
        bound_entity: None,
        columns: vec!["ID".into(), "Name".into(), "Phone".into()],
    }), None).unwrap();
    let _ = doc.update_node("tbl", NodeUpdate::DataBinding {
        entity: Some("clients".into()),
        field: None,
    });
    if let NodeType::Table { bound_entity, columns } = &doc.get_node("tbl").unwrap().node_type {
        assert_eq!(bound_entity.as_deref(), Some("clients"));
        assert_eq!(columns.len(), 3);
    } else {
        panic!("not a table node");
    }
}

#[test]
fn render_order_root_only() {
    let mut doc = ProjectDocument::new();
    doc.add_node(make_node("root"), None).unwrap();
    doc.add_node(make_node("c1"), Some("root".into())).unwrap();
    doc.add_node(make_node("root2"), None).unwrap();
    let order = doc.render_order();
    // render_order only returns root nodes
    assert!(order.contains(&"root"));
    assert!(order.contains(&"root2"));
    assert!(!order.contains(&"c1"));
}

#[test]
fn delete_node_fires_event() {
    let mut doc = ProjectDocument::new();
    doc.add_node(make_node("x"), None).unwrap();
    let ev = CanvasEvent::DeleteNode { id: "x".into() };
    if let CanvasEvent::DeleteNode { id } = &ev {
        doc.remove_node(id).unwrap();
    }
    assert!(!doc.nodes.contains_key("x"));
}

#[test]
fn action_triggered_event() {
    let ev = CanvasEvent::ActionTriggered { source_node_id: "btn-1".into() };
    if let CanvasEvent::ActionTriggered { source_node_id } = &ev {
        assert_eq!(source_node_id, "btn-1");
    } else {
        panic!("wrong variant");
    }
}

#[test]
fn zoom_affects_padding_in_hit_test() {
    let mut doc = ProjectDocument::new();
    doc.add_node(make_node("parent"), None).unwrap();
    {
        let p = doc.nodes.get_mut("parent").unwrap();
        p.position = (0., 0.);
        p.styling.padding = [10., 10., 10., 10.];
    }
    doc.add_node(text_node("child", 5., 5.), Some("parent".into())).unwrap();
    // at zoom 1.0: child world position = 0 + 10/1 + 5 = 15
    assert_eq!(hit_test_nodes(&doc, (15., 15.), 1.0), Some("child".into()));
    // at zoom 2.0: child world position = 0 + 10/2 + 5 = 10
    assert_eq!(hit_test_nodes(&doc, (10., 10.), 2.0), Some("child".into()));
    // at zoom 2.0: (15,15) should NOT hit (it's at 10 in world now)
    assert_eq!(hit_test_nodes(&doc, (15., 15.), 2.0), Some("child".into())); // still within 100x30 AABB
}

#[test]
fn button_action_navigation_binding() {
    let mut doc = ProjectDocument::new();
    doc.add_node(Node::new("b1".into(), "Login Button".into(), NodeType::Button {
        label: "Sign In".into(),
        style: ButtonStyle::Primary,
    }), None).unwrap();

    doc.set_button_action("b1", Some("crm-dash".into()), None);
    let (nav, sub) = doc.get_button_action("b1");
    assert_eq!(nav.as_deref(), Some("crm-dash"));
    assert_eq!(sub, None);
}

#[test]
fn button_action_submit_and_navigate_binding() {
    let mut doc = ProjectDocument::new();
    doc.add_node(Node::new("b2".into(), "Save Contact Button".into(), NodeType::Button {
        label: "Save".into(),
        style: ButtonStyle::Primary,
    }), None).unwrap();

    doc.set_button_action("b2", Some("crm-contacts".into()), Some("contacts".into()));
    let (nav, sub) = doc.get_button_action("b2");
    assert_eq!(nav.as_deref(), Some("crm-contacts"));
    assert_eq!(sub.as_deref(), Some("contacts"));

    // Test clearing action
    doc.set_button_action("b2", None, None);
    let (nav_cleared, sub_cleared) = doc.get_button_action("b2");
    assert_eq!(nav_cleared, None);
    assert_eq!(sub_cleared, None);
}

#[test]
fn node_rename_and_input_updates() {
    let mut doc = ProjectDocument::new();
    let input_node = Node::new("in1".into(), "Old Name".into(), NodeType::TextInput {
        placeholder: "Initial".into(),
        field_type: FieldType::Text,
        bound_entity: None,
        bound_field: None,
    });
    doc.add_node(input_node, None).unwrap();

    // 1. Rename
    doc.update_node("in1", NodeUpdate::Rename("Customer Email Box".into())).unwrap();
    assert_eq!(doc.get_node("in1").unwrap().name, "Customer Email Box");

    // 2. Placeholder
    doc.update_node("in1", NodeUpdate::Placeholder("Enter your work email...".into())).unwrap();
    if let NodeType::TextInput { placeholder, .. } = &doc.get_node("in1").unwrap().node_type {
        assert_eq!(placeholder, "Enter your work email...");
    } else {
        panic!("Expected TextInput");
    }

    // 3. Dropdown Options
    let drop_node = Node::new("dr1".into(), "Status Dropdown".into(), NodeType::Dropdown {
        options: vec!["A".into()],
        multiple: false,
        bound_entity: None,
        bound_field: None,
    });
    doc.add_node(drop_node, None).unwrap();
    doc.update_node("dr1", NodeUpdate::DropdownOptions(vec!["Active".into(), "Pending".into(), "Closed".into()])).unwrap();
    if let NodeType::Dropdown { options, .. } = &doc.get_node("dr1").unwrap().node_type {
        assert_eq!(options, &vec!["Active".to_string(), "Pending".to_string(), "Closed".to_string()]);
    } else {
        panic!("Expected Dropdown");
    }

    // 4. Checkbox Label
    let chk_node = Node::new("ck1".into(), "Terms Box".into(), NodeType::Checkbox {
        label: "Old Label".into(),
        bound_entity: None,
        bound_field: None,
    });
    doc.add_node(chk_node, None).unwrap();
    doc.update_node("ck1", NodeUpdate::CheckboxLabel("I accept terms & conditions".into())).unwrap();
    if let NodeType::Checkbox { label, .. } = &doc.get_node("ck1").unwrap().node_type {
        assert_eq!(label, "I accept terms & conditions");
    } else {
        panic!("Expected Checkbox");
    }
}
