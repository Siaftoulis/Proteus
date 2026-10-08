//! SQLite persistence for business questionnaires.

use super::types::{BusinessQuestionnaire, TopIndustryCategory};
use rusqlite::{params, Connection};

/// Initializes the `business_questionnaires` table in SQLite.
pub fn init_questionnaire_schema(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS business_questionnaires (
            id TEXT PRIMARY KEY,
            business_name TEXT NOT NULL,
            afm TEXT,
            top_category TEXT NOT NULL,
            sub_category_id TEXT NOT NULL,
            workflow_json TEXT NOT NULL,
            scale_json TEXT NOT NULL,
            strategy_json TEXT NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_questionnaire_created ON business_questionnaires(created_at);",
    )
}

/// Persists or updates a business questionnaire profile in SQLite.
pub fn save_questionnaire(conn: &Connection, q: &BusinessQuestionnaire) -> rusqlite::Result<()> {
    let top_cat_str = serde_json::to_string(&q.top_category).unwrap_or_default();
    let workflow_json = serde_json::to_string(&q.workflow).unwrap_or_default();
    let scale_json = serde_json::to_string(&q.scale).unwrap_or_default();
    let strategy_json = serde_json::to_string(&q.strategy).unwrap_or_default();

    conn.execute(
        "INSERT INTO business_questionnaires (
            id, business_name, afm, top_category, sub_category_id,
            workflow_json, scale_json, strategy_json, created_at, updated_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
        ON CONFLICT(id) DO UPDATE SET
            business_name = excluded.business_name,
            afm = excluded.afm,
            top_category = excluded.top_category,
            sub_category_id = excluded.sub_category_id,
            workflow_json = excluded.workflow_json,
            scale_json = excluded.scale_json,
            strategy_json = excluded.strategy_json,
            updated_at = excluded.updated_at;",
        params![
            q.id,
            q.business_name,
            q.afm,
            top_cat_str,
            q.sub_category_id,
            workflow_json,
            scale_json,
            strategy_json,
            q.created_at,
            q.updated_at,
        ],
    )?;
    Ok(())
}

/// Loads a business questionnaire by its unique identifier.
pub fn get_questionnaire(conn: &Connection, id: &str) -> rusqlite::Result<Option<BusinessQuestionnaire>> {
    let mut stmt = conn.prepare(
        "SELECT id, business_name, afm, top_category, sub_category_id,
                workflow_json, scale_json, strategy_json, created_at, updated_at
         FROM business_questionnaires WHERE id = ?1",
    )?;

    let mut rows = stmt.query(params![id])?;
    if let Some(row) = rows.next()? {
        let top_cat_str: String = row.get(3)?;
        let top_category = serde_json::from_str(&top_cat_str).unwrap_or(TopIndustryCategory::TechnologyAndRepairs);
        let workflow_json: String = row.get(5)?;
        let workflow = serde_json::from_str(&workflow_json).unwrap_or_default();
        let scale_json: String = row.get(6)?;
        let scale = serde_json::from_str(&scale_json).unwrap_or_default();
        let strategy_json: String = row.get(7)?;
        let strategy = serde_json::from_str(&strategy_json).unwrap_or_default();

        Ok(Some(BusinessQuestionnaire {
            id: row.get(0)?,
            business_name: row.get(1)?,
            afm: row.get(2)?,
            top_category,
            sub_category_id: row.get(4)?,
            workflow,
            scale,
            strategy,
            created_at: row.get(8)?,
            updated_at: row.get(9)?,
        }))
    } else {
        Ok(None)
    }
}

/// Lists all questionnaires ordered by creation date descending.
pub fn list_questionnaires(conn: &Connection) -> rusqlite::Result<Vec<BusinessQuestionnaire>> {
    let mut stmt = conn.prepare(
        "SELECT id, business_name, afm, top_category, sub_category_id,
                workflow_json, scale_json, strategy_json, created_at, updated_at
         FROM business_questionnaires ORDER BY created_at DESC",
    )?;

    let rows = stmt.query_map([], |row| {
        let top_cat_str: String = row.get(3)?;
        let top_category = serde_json::from_str(&top_cat_str).unwrap_or(TopIndustryCategory::TechnologyAndRepairs);
        let workflow_json: String = row.get(5)?;
        let workflow = serde_json::from_str(&workflow_json).unwrap_or_default();
        let scale_json: String = row.get(6)?;
        let scale = serde_json::from_str(&scale_json).unwrap_or_default();
        let strategy_json: String = row.get(7)?;
        let strategy = serde_json::from_str(&strategy_json).unwrap_or_default();

        Ok(BusinessQuestionnaire {
            id: row.get(0)?,
            business_name: row.get(1)?,
            afm: row.get(2)?,
            top_category,
            sub_category_id: row.get(4)?,
            workflow,
            scale,
            strategy,
            created_at: row.get(8)?,
            updated_at: row.get(9)?,
        })
    })?;

    let mut result = Vec::new();
    for r in rows {
        result.push(r?);
    }
    Ok(result)
}
