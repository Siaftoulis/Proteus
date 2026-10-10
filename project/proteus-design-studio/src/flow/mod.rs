pub mod bezier;
pub mod data_block_ui;
pub mod data_blocks;
pub mod hardware_nodes;
pub mod nodes;
pub mod ports;

pub use bezier::*;
pub use data_block_ui::*;
pub use data_blocks::*;
pub use hardware_nodes::*;
pub use nodes::*;
pub use ports::*;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FlowGraph {
    pub nodes: std::collections::HashMap<String, FlowNode>,
    pub edges: Vec<FlowEdge>,
}

impl Default for FlowGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl FlowGraph {
    pub fn new() -> Self {
        Self {
            nodes: std::collections::HashMap::new(),
            edges: Vec::new(),
        }
    }

    pub fn remove_node(&mut self, id: &str) {
        self.nodes.remove(id);
        self.edges.retain(|e| e.from_node != id && e.to_node != id);
    }

    pub fn can_connect(
        &self,
        from_node: &str,
        from_port: &str,
        to_node: &str,
        to_port: &str,
    ) -> Result<(), &'static str> {
        if from_node == to_node {
            return Err("Cannot connect a node to itself");
        }
        let src = self.nodes.get(from_node).ok_or("Source node not found")?;
        let tgt = self.nodes.get(to_node).ok_or("Target node not found")?;

        let src_p = src.find_port(from_port).ok_or("Source port not found")?;
        let tgt_p = tgt.find_port(to_port).ok_or("Target port not found")?;

        if src_p.direction != PortDirection::Output {
            return Err("Source port must be an Output");
        }
        if tgt_p.direction != PortDirection::Input {
            return Err("Target port must be an Input");
        }
        if !src_p.data_type.is_compatible_with(&tgt_p.data_type) {
            return Err("Port data types are incompatible");
        }
        if self.edges.iter().any(|e| {
            e.from_node == from_node
                && e.from_port == from_port
                && e.to_node == to_node
                && e.to_port == to_port
        }) {
            return Err("Edge already exists");
        }
        Ok(())
    }

    pub fn add_edge(&mut self, edge: FlowEdge) -> Result<(), &'static str> {
        self.can_connect(&edge.from_node, &edge.from_port, &edge.to_node, &edge.to_port)?;
        self.edges.push(edge);
        Ok(())
    }

    pub fn incoming_edges(&self, node_id: &str) -> Vec<&FlowEdge> {
        self.edges.iter().filter(|e| e.to_node == node_id).collect()
    }

    pub fn outgoing_edges(&self, node_id: &str) -> Vec<&FlowEdge> {
        self.edges.iter().filter(|e| e.from_node == node_id).collect()
    }

    /// Imports a parsed SQL relational schema graph into the visual FlowGraph canvas,
    /// arranging tables in a clean grid and synthesizing colored relationship wires.
    pub fn import_sql_schema_graph(
        &mut self,
        schema_graph: &proteus_core::sql_ingest::SqlSchemaGraph,
        origin: (f32, f32),
    ) {
        let cols_per_row = 3;
        let x_spacing = 340.0;
        let y_spacing = 260.0;

        let mut table_node_ids = std::collections::HashMap::new();

        // 1. Instantiate TableSchema nodes
        for (idx, tbl) in schema_graph.tables.iter().enumerate() {
            let col = (idx % cols_per_row) as f32;
            let row = (idx / cols_per_row) as f32;
            let pos = (origin.0 + col * x_spacing, origin.1 + row * y_spacing);

            let node_id = format!("tbl_{}", tbl.name);
            let kind = FlowNodeKind::TableSchema {
                table_name: tbl.name.clone(),
                columns: tbl.columns.clone(),
            };
            let node = FlowNode::new(&node_id, kind, pos);
            table_node_ids.insert(tbl.name.clone(), node_id.clone());
            self.nodes.insert(node_id, node);
        }

        // 2. Synthesize multi-colored foreign key relationship wires
        for rel in &schema_graph.relations {
            if let (Some(src_id), Some(tgt_id)) = (
                table_node_ids.get(&rel.from_table),
                table_node_ids.get(&rel.to_table),
            ) {
                let edge = FlowEdge::with_ports(
                    src_id,
                    format!("out_{}", rel.from_col),
                    tgt_id,
                    format!("in_{}", rel.to_col),
                )
                .with_color(rel.color_hue);

                if !self.edges.iter().any(|e| {
                    e.from_node == edge.from_node
                        && e.from_port == edge.from_port
                        && e.to_node == edge.to_node
                        && e.to_port == edge.to_port
                }) {
                    self.edges.push(edge);
                }
            }
        }
    }

    /// Exports all visual TableSchema nodes and relationship wires into standard SQL DDL statements.
    /// Orders tables topologically so referenced tables are generated before foreign-key dependents.
    pub fn export_sql_schema_ddl(&self) -> String {
        let mut table_nodes: Vec<(&String, &String, &Vec<proteus_core::sql_ingest::SqlColumnMeta>)> = Vec::new();
        for (node_id, node) in &self.nodes {
            if let FlowNodeKind::TableSchema { ref table_name, ref columns } = node.kind {
                table_nodes.push((node_id, table_name, columns));
            }
        }

        if table_nodes.is_empty() {
            return String::new();
        }

        // Dependency sort: tables without foreign keys first, then tables referencing them
        table_nodes.sort_by(|(id_a, name_a, _), (id_b, name_b, _)| {
            let refs_b = self.edges.iter().any(|e| &e.from_node == *id_a && &e.to_node == *id_b);
            let refs_a = self.edges.iter().any(|e| &e.from_node == *id_b && &e.to_node == *id_a);
            if refs_b {
                std::cmp::Ordering::Greater
            } else if refs_a {
                std::cmp::Ordering::Less
            } else {
                name_a.cmp(name_b)
            }
        });

        let mut ddl_blocks = Vec::new();
        for (node_id, table_name, columns) in &table_nodes {
            let mut lines = Vec::new();

            // 1. Column definitions
            for col in *columns {
                let pk = if col.is_primary_key { " PRIMARY KEY" } else { "" };
                let not_null = if !col.is_nullable && !col.is_primary_key { " NOT NULL" } else { "" };
                lines.push(format!("    {} {}{}{}", col.name, col.data_type, pk, not_null));
            }

            // 2. Foreign key constraints derived from visual edges
            for edge in &self.edges {
                if edge.from_node == **node_id {
                    let from_col = edge.from_port.strip_prefix("out_").unwrap_or(&edge.from_port);
                    let to_col = edge.to_port.strip_prefix("in_").unwrap_or(&edge.to_port);
                    if let Some(target_node) = self.nodes.get(&edge.to_node) {
                        if let FlowNodeKind::TableSchema { table_name: ref target_table, .. } = target_node.kind {
                            lines.push(format!("    FOREIGN KEY ({}) REFERENCES {}({})", from_col, target_table, to_col));
                        }
                    }
                }
            }

            let block = format!("CREATE TABLE {} (\n{}\n);", table_name, lines.join(",\n"));
            ddl_blocks.push(block);
        }

        ddl_blocks.join("\n\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_ports_and_data_type_compatibility() {
        let (in_p, out_p) = default_ports_for_kind(&FlowNodeKind::Condition {
            field: "total".into(),
            operator: ">".into(),
            target_value: "100".into(),
        });
        assert_eq!(in_p.len(), 1);
        assert_eq!(in_p[0].id, "exec_in");
        assert_eq!(out_p.len(), 2);
        assert_eq!(out_p[0].id, "branch_true");
        assert_eq!(out_p[1].id, "branch_false");

        assert!(PortDataType::ExecutionFlow.is_compatible_with(&PortDataType::ExecutionFlow));
        assert!(PortDataType::Record.is_compatible_with(&PortDataType::Any));
        assert!(!PortDataType::Record.is_compatible_with(&PortDataType::Number));
    }

    #[test]
    fn test_cubic_bezier_sampling_and_distance() {
        let curve = CubicBezierCurve::from_endpoints((0.0, 0.0), (100.0, 100.0));
        let start = curve.sample(0.0);
        let mid = curve.sample(0.5);
        let end = curve.sample(1.0);

        assert!((start.0 - 0.0).abs() < 1e-4);
        assert!((start.1 - 0.0).abs() < 1e-4);
        assert!((end.0 - 100.0).abs() < 1e-4);
        assert!((end.1 - 100.0).abs() < 1e-4);
        assert!(mid.0 > 20.0 && mid.0 < 80.0);

        let dist = curve.distance_to_point((0.0, 0.0), 10);
        assert!(dist < 1e-2);
    }

    #[test]
    fn test_flow_graph_node_operations_and_wire_validation() {
        let mut graph = FlowGraph::new();
        let node1 = FlowNode::new(
            "n1",
            FlowNodeKind::TriggerClick { target_node_id: "btn-1".into() },
            (0.0, 0.0),
        );
        let node2 = FlowNode::new(
            "n2",
            FlowNodeKind::Condition {
                field: "amount".into(),
                operator: ">".into(),
                target_value: "100".into(),
            },
            (200.0, 0.0),
        );
        let node3 = FlowNode::new(
            "n3",
            FlowNodeKind::ShowToast { message: "High value deal!".into() },
            (400.0, 0.0),
        );

        graph.nodes.insert("n1".into(), node1);
        graph.nodes.insert("n2".into(), node2);
        graph.nodes.insert("n3".into(), node3);

        // Valid connections
        assert!(graph.can_connect("n1", "exec_out", "n2", "exec_in").is_ok());
        graph.add_edge(FlowEdge::with_ports("n1", "exec_out", "n2", "exec_in")).unwrap();

        assert!(graph.can_connect("n2", "branch_true", "n3", "exec_in").is_ok());
        graph.add_edge(FlowEdge::with_ports("n2", "branch_true", "n3", "exec_in")).unwrap();

        // Self-connection rejected
        assert_eq!(
            graph.can_connect("n1", "exec_out", "n1", "exec_in"),
            Err("Cannot connect a node to itself")
        );

        // Direction reversal rejected (Input to Output)
        assert_eq!(
            graph.can_connect("n2", "exec_in", "n1", "exec_out"),
            Err("Source port must be an Output")
        );

        // Serialize and deserialize roundtrip
        let json = serde_json::to_string(&graph).expect("serialize");
        let restored: FlowGraph = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(restored.nodes.len(), 3);
        assert_eq!(restored.edges.len(), 2);

        // Remove node cascades
        graph.remove_node("n2");
        assert_eq!(graph.nodes.len(), 2);
        assert_eq!(graph.edges.len(), 0);
    }

    #[test]
    fn test_import_sql_schema_graph_instantiates_nodes_and_colored_edges() {
        let sql = r#"
            CREATE TABLE suppliers (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL
            );
            CREATE TABLE products (
                id TEXT PRIMARY KEY,
                supplier_id INTEGER NOT NULL,
                title TEXT NOT NULL,
                FOREIGN KEY (supplier_id) REFERENCES suppliers(id)
            );
        "#;
        let schema_graph = proteus_core::sql_ingest::SqlSchemaGraph::parse_sql_ddl(sql)
            .expect("DDL must parse");

        let mut flow_graph = FlowGraph::new();
        flow_graph.import_sql_schema_graph(&schema_graph, (100.0, 100.0));

        assert_eq!(flow_graph.nodes.len(), 2);
        assert!(flow_graph.nodes.contains_key("tbl_suppliers"));
        assert!(flow_graph.nodes.contains_key("tbl_products"));

        // Check ports
        let supp_node = flow_graph.nodes.get("tbl_suppliers").unwrap();
        assert!(supp_node.output_ports.iter().any(|p| p.id == "out_id"));

        let prod_node = flow_graph.nodes.get("tbl_products").unwrap();
        assert!(prod_node.output_ports.iter().any(|p| p.id == "out_supplier_id"));

        // Check synthesized relationship edge with color hue
        assert_eq!(flow_graph.edges.len(), 1);
        let edge = &flow_graph.edges[0];
        assert_eq!(edge.from_node, "tbl_products");
        assert_eq!(edge.from_port, "out_supplier_id");
        assert_eq!(edge.to_node, "tbl_suppliers");
        assert_eq!(edge.to_port, "in_id");
        assert!(edge.color_hue.is_some());
    }

    #[test]
    fn test_export_sql_schema_ddl_roundtrip() {
        let original_sql = r#"
            CREATE TABLE customers (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL
            );
            CREATE TABLE orders (
                order_id TEXT PRIMARY KEY,
                customer_id INTEGER NOT NULL,
                FOREIGN KEY (customer_id) REFERENCES customers(id)
            );
        "#;
        let schema_graph = proteus_core::sql_ingest::SqlSchemaGraph::parse_sql_ddl(original_sql).unwrap();
        let mut flow_graph = FlowGraph::new();
        flow_graph.import_sql_schema_graph(&schema_graph, (0.0, 0.0));

        let exported_ddl = flow_graph.export_sql_schema_ddl();
        assert!(!exported_ddl.is_empty());
        assert!(exported_ddl.contains("CREATE TABLE customers"));
        assert!(exported_ddl.contains("CREATE TABLE orders"));
        assert!(exported_ddl.contains("FOREIGN KEY (customer_id) REFERENCES customers(id)"));

        // Roundtrip parse verification through SQLite sandbox
        let re_parsed = proteus_core::sql_ingest::SqlSchemaGraph::parse_sql_ddl(&exported_ddl).unwrap();
        assert_eq!(re_parsed.tables.len(), 2);
        assert_eq!(re_parsed.relations.len(), 1);
        assert_eq!(re_parsed.relations[0].from_table, "orders");
        assert_eq!(re_parsed.relations[0].to_table, "customers");
        assert_eq!(re_parsed.relations[0].from_col, "customer_id");
        assert_eq!(re_parsed.relations[0].to_col, "id");
    }

    #[test]
    fn test_flow_edge_compute_relation_hue() {
        let hue1 = FlowEdge::compute_relation_hue("tbl_orders", "tbl_customers");
        let hue2 = FlowEdge::compute_relation_hue("tbl_orders", "tbl_products");
        let hue3 = FlowEdge::compute_relation_hue("tbl_orders", "tbl_customers");

        assert_eq!(hue1, hue3, "Identical pair must yield deterministic hue");
        assert_ne!(hue1, hue2, "Different pair should yield distinct hue");
        assert!((0.0..360.0).contains(&hue1));
        assert!((0.0..360.0).contains(&hue2));
    }
}
