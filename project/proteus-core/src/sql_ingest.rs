// Proteus Core — SQL DDL Script Ingestion & Relational Schema Visualizer Engine
// 100% bespoke architecture. Zero copied boilerplate.

use rusqlite::Connection;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SqlColumnMeta {
    pub name: String,
    pub data_type: String,
    pub is_nullable: bool,
    pub is_primary_key: bool,
    #[serde(default)]
    pub is_unique: bool,
    #[serde(default)]
    pub is_indexed: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SqlRelationMeta {
    pub id: String,
    pub from_table: String,
    pub from_col: String,
    pub to_table: String,
    pub to_col: String,
    pub color_hue: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SqlTableMeta {
    pub name: String,
    pub columns: Vec<SqlColumnMeta>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SqlSchemaGraph {
    pub tables: Vec<SqlTableMeta>,
    pub relations: Vec<SqlRelationMeta>,
}

impl SqlSchemaGraph {
    /// Ingests a raw SQL DDL script, executes in an in-memory SQLite sandbox,
    /// and extracts all tables, columns, primary keys, and foreign-key relations.
    pub fn parse_sql_ddl(sql_content: &str) -> Result<Self, String> {
        let conn = Connection::open_in_memory()
            .map_err(|e| format!("Failed to create in-memory SQLite sandbox: {}", e))?;

        // Enable foreign keys in sandbox
        let _ = conn.execute_batch("PRAGMA foreign_keys = ON;");

        // Execute the full DDL script inside the sandbox
        conn.execute_batch(sql_content)
            .map_err(|e| format!("SQL DDL Execution failed: {}", e))?;

        // 1. Retrieve all user-defined table names
        let mut table_stmt = conn
            .prepare(
                "SELECT name FROM sqlite_master 
                 WHERE type = 'table' AND name NOT LIKE 'sqlite_%' 
                 ORDER BY name ASC",
            )
            .map_err(|e| e.to_string())?;

        let table_names: Vec<String> = table_stmt
            .query_map([], |row| row.get(0))
            .map_err(|e| e.to_string())?
            .filter_map(|r| r.ok())
            .collect();

        let mut tables = Vec::new();
        let mut relations = Vec::new();

        for tname in &table_names {
            // 2. Extract column metadata for this table
            let pragma_info = format!("PRAGMA table_info(\"{}\")", tname.replace('"', ""));
            let mut col_stmt = conn.prepare(&pragma_info).map_err(|e| e.to_string())?;
            let columns: Vec<SqlColumnMeta> = col_stmt
                .query_map([], |r| {
                    Ok(SqlColumnMeta {
                        name: r.get(1)?,
                        data_type: r.get(2)?,
                        is_nullable: r.get::<_, i32>(3)? == 0,
                        is_primary_key: r.get::<_, i32>(5)? > 0,
                        is_unique: false,
                        is_indexed: false,
                    })
                })
                .map_err(|e| e.to_string())?
                .filter_map(|r| r.ok())
                .collect();

            tables.push(SqlTableMeta {
                name: tname.clone(),
                columns,
            });

            // 3. Extract foreign key relationships for this table
            let pragma_fk = format!("PRAGMA foreign_key_list(\"{}\")", tname.replace('"', ""));
            let mut fk_stmt = conn.prepare(&pragma_fk).map_err(|e| e.to_string())?;
            let table_relations: Vec<SqlRelationMeta> = fk_stmt
                .query_map([], |r| {
                    let to_table: String = r.get(2)?;
                    let from_col: String = r.get(3)?;
                    let to_col: String = r.get(4)?;
                    let id = format!("{}_{}_to_{}_{}", tname, from_col, to_table, to_col);
                    
                    // Generate a distinct color hue [0..360] using deterministic hashing
                    let hash_src = format!("{}:{}", tname, to_table);
                    let hash_val: u32 = hash_src.bytes().fold(0u32, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u32));
                    let color_hue = (hash_val % 360) as f32;

                    Ok(SqlRelationMeta {
                        id,
                        from_table: tname.clone(),
                        from_col,
                        to_table,
                        to_col,
                        color_hue,
                    })
                })
                .map_err(|e| e.to_string())?
                .filter_map(|r| r.ok())
                .collect();

            relations.extend(table_relations);
        }

        Ok(Self { tables, relations })
    }

    /// Reads and parses an external .sql file from disk.
    pub fn parse_sql_file(path: &std::path::Path) -> Result<Self, String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read SQL file '{}': {}", path.display(), e))?;
        Self::parse_sql_ddl(&content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_multi_table_sql_ddl_with_foreign_keys() {
        let sql = r#"
            CREATE TABLE customers (
                id INTEGER PRIMARY KEY,
                company_name TEXT NOT NULL,
                vat_number TEXT UNIQUE,
                email TEXT
            );

            CREATE TABLE orders (
                id TEXT PRIMARY KEY,
                customer_id INTEGER NOT NULL,
                order_date TEXT NOT NULL,
                total_amount REAL DEFAULT 0.0,
                FOREIGN KEY (customer_id) REFERENCES customers(id)
            );

            CREATE TABLE order_items (
                id TEXT PRIMARY KEY,
                order_id TEXT NOT NULL,
                sku TEXT NOT NULL,
                quantity INTEGER NOT NULL,
                unit_price REAL NOT NULL,
                FOREIGN KEY (order_id) REFERENCES orders(id)
            );
        "#;

        let graph = SqlSchemaGraph::parse_sql_ddl(sql).expect("DDL must parse cleanly");
        assert_eq!(graph.tables.len(), 3);

        let cust_table = graph.tables.iter().find(|t| t.name == "customers").unwrap();
        assert_eq!(cust_table.columns.len(), 4);
        assert!(cust_table.columns[0].is_primary_key);
        assert_eq!(cust_table.columns[0].name, "id");

        // Verify foreign key relations
        assert_eq!(graph.relations.len(), 2);

        let order_to_cust = graph
            .relations
            .iter()
            .find(|r| r.from_table == "orders" && r.to_table == "customers")
            .unwrap();
        assert_eq!(order_to_cust.from_col, "customer_id");
        assert_eq!(order_to_cust.to_col, "id");
        assert!(order_to_cust.color_hue >= 0.0 && order_to_cust.color_hue < 360.0);

        let item_to_order = graph
            .relations
            .iter()
            .find(|r| r.from_table == "order_items" && r.to_table == "orders")
            .unwrap();
        assert_eq!(item_to_order.from_col, "order_id");
        assert_eq!(item_to_order.to_col, "id");
    }

    #[test]
    fn test_parse_invalid_sql_returns_clear_error() {
        let bad_sql = "CREATE TABLE broken (id INT PRIMARY KEY,";
        let res = SqlSchemaGraph::parse_sql_ddl(bad_sql);
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("SQL DDL Execution failed"));
    }

    #[test]
    fn test_parse_complex_erp_schema_with_multiple_foreign_keys() {
        let erp_sql = r#"
            CREATE TABLE customers (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                vat_number TEXT UNIQUE NOT NULL,
                company_name TEXT NOT NULL,
                credit_limit REAL NOT NULL DEFAULT 5000.0
            );

            CREATE TABLE products (
                sku TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                unit_price REAL NOT NULL
            );

            CREATE TABLE orders (
                order_number TEXT PRIMARY KEY,
                customer_id INTEGER NOT NULL,
                total_amount REAL NOT NULL,
                FOREIGN KEY (customer_id) REFERENCES customers(id)
            );

            CREATE TABLE order_items (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                order_number TEXT NOT NULL,
                product_sku TEXT NOT NULL,
                quantity INTEGER NOT NULL,
                FOREIGN KEY (order_number) REFERENCES orders(order_number),
                FOREIGN KEY (product_sku) REFERENCES products(sku)
            );

            CREATE TABLE invoices (
                invoice_number TEXT PRIMARY KEY,
                order_number TEXT NOT NULL,
                mydata_mark TEXT UNIQUE NOT NULL,
                total_payable REAL NOT NULL,
                FOREIGN KEY (order_number) REFERENCES orders(order_number)
            );
        "#;

        let graph = SqlSchemaGraph::parse_sql_ddl(erp_sql).expect("ERP DDL must parse");
        assert_eq!(graph.tables.len(), 5);
        assert_eq!(graph.relations.len(), 4);

        let hues: std::collections::HashSet<u32> = graph.relations.iter().map(|r| r.color_hue as u32).collect();
        assert!(hues.len() >= 3, "Expected distinct color hues for relationships");
    }
}

