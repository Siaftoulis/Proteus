//! Visual Scratch Data Blocks & Drag-and-Drop ID Linking for Flow Studio.
//! Provides entity data sources, field extractors, foreign-key ID linkers,
//! query filter predicates, and compiles connected visual data blocks into executable SQL.

use super::ports::{NodePort, PortDataType};
use proteus_core::dsl::DslSchema;
use serde::{Deserialize, Serialize};

/// Specialized data block variants in the visual Scratch-style pipeline.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum DataBlockKind {
    /// Ingests records from a persistent database entity table.
    EntitySource { entity: String, limit: usize },

    /// Extracts a specific column or attribute value from incoming record.
    FieldExtractor {
        entity: String,
        field: String,
        data_type: PortDataType,
    },

    /// Drag-and-drop Foreign Key & Primary Key linker between entities.
    IdLinker {
        from_entity: String,
        from_field: String,
        to_entity: String,
        to_field: String,
    },

    /// Declarative predicate filter (`WHERE field op target_val`).
    QueryFilter {
        field: String,
        operator: String,
        target_val: String,
    },

    /// Database mutation block (INSERT, UPDATE, UPSERT).
    RecordMutator { entity: String, operation: String },
}

impl DataBlockKind {
    /// Generates input and output sockets for this data block.
    pub fn default_ports(&self) -> (Vec<NodePort>, Vec<NodePort>) {
        match self {
            Self::EntitySource { .. } => (
                vec![NodePort::input("trigger_in", "Fetch", PortDataType::ExecutionFlow)],
                vec![
                    NodePort::output("record_out", "Records", PortDataType::Record),
                    NodePort::output("id_out", "Row ID", PortDataType::Text),
                    NodePort::output("exec_out", "Next", PortDataType::ExecutionFlow),
                ],
            ),
            Self::FieldExtractor { field, data_type, .. } => (
                vec![NodePort::input("record_in", "Record", PortDataType::Record)],
                vec![NodePort::output(
                    "val_out",
                    format!("Val: {}", field),
                    *data_type,
                )],
            ),
            Self::IdLinker {
                from_field,
                to_field,
                ..
            } => (
                vec![NodePort::input(
                    "id_in",
                    format!("From: {}", from_field),
                    PortDataType::Text,
                )],
                vec![NodePort::output(
                    "fk_out",
                    format!("FK -> {}", to_field),
                    PortDataType::Text,
                )],
            ),
            Self::QueryFilter { field, operator, .. } => (
                vec![
                    NodePort::input("record_in", "Record", PortDataType::Record),
                    NodePort::input("val_in", "Target", PortDataType::Any),
                ],
                vec![
                    NodePort::output(
                        "record_out",
                        format!("Matched ({})", field),
                        PortDataType::Record,
                    ),
                    NodePort::output("cond_bool", operator.clone(), PortDataType::Boolean),
                ],
            ),
            Self::RecordMutator { operation, .. } => (
                vec![
                    NodePort::input("exec_in", operation.clone(), PortDataType::ExecutionFlow),
                    NodePort::input("record_in", "Payload", PortDataType::Record),
                ],
                vec![
                    NodePort::output("exec_out", "Committed", PortDataType::ExecutionFlow),
                    NodePort::output("record_out", "Saved", PortDataType::Record),
                ],
            ),
        }
    }

    /// User-friendly label displayed in the block header.
    pub fn display_label(&self) -> String {
        match self {
            Self::EntitySource { entity, limit } => format!("DB Source: {} (L:{})", entity, limit),
            Self::FieldExtractor { entity, field, .. } => format!("{}.{}", entity, field),
            Self::IdLinker { from_entity, to_entity, .. } => {
                format!("Link: {} -> {}", from_entity, to_entity)
            }
            Self::QueryFilter { field, operator, target_val } => {
                format!("Filter: {} {} '{}'", field, operator, target_val)
            }
            Self::RecordMutator { entity, operation } => format!("{}: {}", operation, entity),
        }
    }

    /// RGB styling color for the block header ribbon.
    pub fn display_color(&self) -> (u8, u8, u8) {
        match self {
            Self::EntitySource { .. } => (40, 160, 220),   // Cerulean Blue
            Self::FieldExtractor { .. } => (180, 100, 240), // Purple/Amethyst
            Self::IdLinker { .. } => (245, 130, 32),       // Amber/Orange
            Self::QueryFilter { .. } => (230, 70, 70),     // Coral/Red
            Self::RecordMutator { .. } => (46, 204, 113),  // Emerald Green
        }
    }
}

/// Standalone Scratch Data Block container.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct DataBlockNode {
    pub id: String,
    pub kind: DataBlockKind,
    pub position: (f32, f32),
    pub input_ports: Vec<NodePort>,
    pub output_ports: Vec<NodePort>,
}

impl DataBlockNode {
    pub fn new(id: impl Into<String>, kind: DataBlockKind, position: (f32, f32)) -> Self {
        let (inputs, outputs) = kind.default_ports();
        Self {
            id: id.into(),
            kind,
            position,
            input_ports: inputs,
            output_ports: outputs,
        }
    }
}

/// Visual query builder assembling connected data blocks into SQL statements.
pub struct DataPipelineQuery;

impl DataPipelineQuery {
    /// Compiles a set of connected visual data blocks into an SQL query.
    pub fn compile(
        source: &DataBlockKind,
        filters: &[DataBlockKind],
        fields: &[DataBlockKind],
        links: &[DataBlockKind],
    ) -> Result<String, String> {
        let (entity, limit) = match source {
            DataBlockKind::EntitySource { entity, limit } => (entity.as_str(), *limit),
            _ => return Err("Pipeline must originate from an EntitySource block".to_string()),
        };

        // Projected columns
        let mut select_cols = Vec::new();
        for f in fields {
            if let DataBlockKind::FieldExtractor { field, .. } = f {
                select_cols.push(field.clone());
            }
        }
        let cols_clause = if select_cols.is_empty() {
            "*".to_string()
        } else {
            select_cols.join(", ")
        };

        // JOINs from IdLinker
        let mut join_clauses = Vec::new();
        for l in links {
            if let DataBlockKind::IdLinker {
                from_entity,
                from_field,
                to_entity,
                to_field,
            } = l
            {
                join_clauses.push(format!(
                    "JOIN {} ON {}.{} = {}.{}",
                    to_entity, from_entity, from_field, to_entity, to_field
                ));
            }
        }

        // WHERE filters
        let mut where_clauses = Vec::new();
        for flt in filters {
            if let DataBlockKind::QueryFilter {
                field,
                operator,
                target_val,
            } = flt
            {
                where_clauses.push(format!("{} {} '{}'", field, operator, target_val));
            }
        }

        let mut sql = format!("SELECT {} FROM {}", cols_clause, entity);
        if !join_clauses.is_empty() {
            sql.push(' ');
            sql.push_str(&join_clauses.join(" "));
        }
        if !where_clauses.is_empty() {
            sql.push_str(" WHERE ");
            sql.push_str(&where_clauses.join(" AND "));
        }
        if limit > 0 {
            sql.push_str(&format!(" LIMIT {}", limit));
        }
        sql.push(';');

        Ok(sql)
    }
}

/// Automatically builds visual Scratch Data Blocks from a declarative DslSchema.
pub fn generate_palette_from_dsl(schema: &DslSchema) -> Vec<DataBlockNode> {
    let mut palette = Vec::new();
    let mut y_offset = 50.0;

    for entity in &schema.entities {
        // 1. Entity Source Block
        let source_node = DataBlockNode::new(
            format!("src_{}", entity.name.to_lowercase()),
            DataBlockKind::EntitySource {
                entity: entity.name.clone(),
                limit: 50,
            },
            (50.0, y_offset),
        );
        palette.push(source_node);
        y_offset += 80.0;

        // 2. Field Extractor Blocks
        for field in &entity.fields {
            let port_dt = match field.field_type {
                proteus_core::dsl::DslFieldType::Integer
                | proteus_core::dsl::DslFieldType::Decimal => PortDataType::Number,
                proteus_core::dsl::DslFieldType::Boolean => PortDataType::Boolean,
                _ => PortDataType::Text,
            };

            let field_node = DataBlockNode::new(
                format!("ext_{}_{}", entity.name.to_lowercase(), field.name),
                DataBlockKind::FieldExtractor {
                    entity: entity.name.clone(),
                    field: field.name.clone(),
                    data_type: port_dt,
                },
                (320.0, y_offset),
            );
            palette.push(field_node);
            y_offset += 65.0;
        }
    }

    // 3. ID Linker Blocks from Relations
    for rel in &schema.relations {
        let linker_node = DataBlockNode::new(
            format!(
                "link_{}_{}",
                rel.from_entity.to_lowercase(),
                rel.to_entity.to_lowercase()
            ),
            DataBlockKind::IdLinker {
                from_entity: rel.from_entity.clone(),
                from_field: rel.from_field.clone(),
                to_entity: rel.to_entity.clone(),
                to_field: rel.to_field.clone(),
            },
            (600.0, y_offset),
        );
        palette.push(linker_node);
        y_offset += 80.0;
    }

    palette
}

#[cfg(test)]
mod tests {
    use super::*;
    use proteus_core::dsl::parse_dsl;

    #[test]
    fn test_data_block_ports_and_colors() {
        let src = DataBlockKind::EntitySource {
            entity: "Customer".into(),
            limit: 25,
        };
        let (in_ports, out_ports) = src.default_ports();
        assert_eq!(in_ports.len(), 1);
        assert_eq!(out_ports.len(), 3);
        assert_eq!(out_ports[0].data_type, PortDataType::Record);
        assert_eq!(out_ports[1].data_type, PortDataType::Text);

        assert_eq!(src.display_label(), "DB Source: Customer (L:25)");
        assert_eq!(src.display_color(), (40, 160, 220));
    }

    #[test]
    fn test_id_linker_ports() {
        let linker = DataBlockKind::IdLinker {
            from_entity: "Customer".into(),
            from_field: "id".into(),
            to_entity: "Order".into(),
            to_field: "customer_id".into(),
        };
        let (in_ports, out_ports) = linker.default_ports();
        assert_eq!(in_ports[0].id, "id_in");
        assert_eq!(out_ports[0].id, "fk_out");
        assert_eq!(linker.display_label(), "Link: Customer -> Order");
    }

    #[test]
    fn test_compile_data_pipeline_to_sql() {
        let source = DataBlockKind::EntitySource {
            entity: "Customer".into(),
            limit: 10,
        };
        let fields = vec![
            DataBlockKind::FieldExtractor {
                entity: "Customer".into(),
                field: "name".into(),
                data_type: PortDataType::Text,
            },
            DataBlockKind::FieldExtractor {
                entity: "Customer".into(),
                field: "balance".into(),
                data_type: PortDataType::Number,
            },
        ];
        let filters = vec![DataBlockKind::QueryFilter {
            field: "balance".into(),
            operator: ">".into(),
            target_val: "0.0".into(),
        }];
        let links = vec![DataBlockKind::IdLinker {
            from_entity: "Customer".into(),
            from_field: "id".into(),
            to_entity: "Order".into(),
            to_field: "customer_id".into(),
        }];

        let sql = DataPipelineQuery::compile(&source, &filters, &fields, &links).unwrap();
        assert_eq!(
            sql,
            "SELECT name, balance FROM Customer JOIN Order ON Customer.id = Order.customer_id WHERE balance > '0.0' LIMIT 10;"
        );
    }

    #[test]
    fn test_generate_palette_from_dsl_schema() {
        let script = r#"
            entity Customer {
                id: uuid primary,
                name: text required
            }
            entity Order {
                id: uuid primary,
                customer_id: uuid required
            }
            relation Customer.id -> Order.customer_id [one_to_many]
        "#;
        let schema = parse_dsl(script).unwrap();
        let palette = generate_palette_from_dsl(&schema);

        // 2 Entity Sources + 4 Field Extractors + 1 ID Linker = 7 DataBlockNodes
        assert_eq!(palette.len(), 7);
        assert!(palette.iter().any(|b| matches!(b.kind, DataBlockKind::EntitySource { ref entity, .. } if entity == "Customer")));
        assert!(palette.iter().any(|b| matches!(b.kind, DataBlockKind::IdLinker { ref from_entity, ref to_entity, .. } if from_entity == "Customer" && to_entity == "Order")));
    }
}
