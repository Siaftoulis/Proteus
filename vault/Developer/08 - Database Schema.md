---
tags:
  - developer/database
---
# Database Schema

> SQLite at `{app_data}/crm-builder.db`.

## projects Table

| Column | Type | Description |
|--------|------|-------------|
| id | TEXT (UUID v4) | Primary key |
| name | TEXT | Project name |
| widgets | JSON | Array of widget objects |
| flows | JSON | Array of flow definitions |
| created_at | ISO 8601 | Created |
| updated_at | ISO 8601 | Updated |

## Widget JSON

```json
{
  "i": "uuid",
  "type": "Table|Chart|Text|Form|Image|Container|Button|Input|Tabs",
  "x": 100, "y": 200, "w": 300, "h": 200,
  "props": {},
  "z": 0
}
```

## Flow JSON

```json
{
  "id": "uuid",
  "type": "trigger|action|condition|logicalGate",
  "position": { "x": 100, "y": 200 },
  "data": {},
  "sourceHandles": [],
  "targetHandles": []
}
```

## License Server DB

```sql
CREATE TABLE licenses (
    id TEXT PRIMARY KEY,
    license_key TEXT UNIQUE,
    customer_email TEXT,
    max_users INTEGER DEFAULT 3,
    features TEXT DEFAULT '[]',
    issued_at TEXT,
    expires_at TEXT,
    activations INTEGER DEFAULT 0
);
```

Related: [[02 - Architecture|Architecture]] | [[03 - Backend (crm-core)|Core Library]] | [[07 - Import System|Import]]
