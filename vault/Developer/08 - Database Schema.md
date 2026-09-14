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

---

## Proteus BOS MVP Schema (`store.db`)

> **Path:** `%APPDATA%\Proteus\data\store.db` (Windows) | `~/.local/share/proteus/store.db` (Linux)  
> **Reference:** [[14 - Proteus BOS Blueprint|14 - Proteus BOS Blueprint]]

### `service_tickets` Table (Core Service & Repairs)

```sql
CREATE TABLE IF NOT EXISTS service_tickets (
    ticket_id TEXT PRIMARY KEY,           -- UUIDv7 (Chronologically sorted)
    ticket_number INTEGER NOT NULL,      -- Sequential shop ticket ID
    customer_name TEXT NOT NULL,
    customer_phone TEXT NOT NULL,
    device_model TEXT NOT NULL,          -- e.g. 'Samsung Galaxy S22' or 'Yamaha Crypton'
    serial_number TEXT,
    reported_fault TEXT NOT NULL,        -- Customer reported issue
    internal_notes TEXT,                 -- Technician notes & diagnostic
    estimated_cost REAL DEFAULT 0.0,
    current_status TEXT NOT NULL CHECK(
        current_status IN ('received', 'in_progress', 'waiting_parts', 'ready', 'delivered', 'cancelled')
    ),
    created_at INTEGER NOT NULL,          -- Epoch ms (Monotonic)
    updated_at INTEGER NOT NULL,          -- Epoch ms (LWW anchor)
    delivered_at INTEGER
);

CREATE INDEX IF NOT EXISTS idx_tickets_status ON service_tickets(current_status);
CREATE INDEX IF NOT EXISTS idx_tickets_phone ON service_tickets(customer_phone);
CREATE INDEX IF NOT EXISTS idx_tickets_updated ON service_tickets(updated_at);
```

### `system_events` Table (Audit Trail)

```sql
CREATE TABLE IF NOT EXISTS system_events (
    event_id TEXT PRIMARY KEY,           -- UUIDv7
    entity_id TEXT NOT NULL,
    event_type TEXT NOT NULL,            -- 'TICKET_CREATED', 'STATUS_CHANGED', etc.
    payload JSON NOT NULL,
    created_at INTEGER NOT NULL
);
```

### `sync_outbox` Table (Mobile / Offline Queue)

```sql
CREATE TABLE IF NOT EXISTS sync_outbox (
    event_id TEXT PRIMARY KEY,        -- UUIDv7
    entity_type TEXT NOT NULL,       -- e.g. 'service_ticket'
    entity_id TEXT NOT NULL,         -- UUID of record
    field_name TEXT NOT NULL,        -- e.g. 'status'
    old_value TEXT,
    new_value TEXT,
    client_timestamp INTEGER NOT NULL, -- Unix epoch ms (Monotonic)
    sync_status TEXT DEFAULT 'pending' -- pending, syncing, acked, rejected
);
```

---

Related: [[02 - Architecture|Architecture]] | [[14 - Proteus BOS Blueprint|Proteus BOS Blueprint]] | [[03 - Backend (crm-core)|Core Library]]
