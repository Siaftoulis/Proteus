-- Proteus Sovereign Business OS — Relational Schema DDL
-- Compatible with SQLite, PostgreSQL, and Proteus Visual Canvas Ingestion

CREATE TABLE customers (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    vat_number TEXT UNIQUE NOT NULL,
    company_name TEXT NOT NULL,
    contact_name TEXT NOT NULL,
    contact_email TEXT NOT NULL,
    contact_phone TEXT NOT NULL,
    city TEXT NOT NULL,
    credit_limit REAL NOT NULL DEFAULT 5000.0,
    status TEXT NOT NULL DEFAULT 'ACTIVE',
    created_at TEXT NOT NULL
);

CREATE TABLE products (
    sku TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    category TEXT NOT NULL,
    unit_price REAL NOT NULL,
    vat_percent REAL NOT NULL DEFAULT 24.0,
    stock_quantity INTEGER NOT NULL DEFAULT 0,
    reorder_threshold INTEGER NOT NULL DEFAULT 10
);

CREATE TABLE orders (
    order_number TEXT PRIMARY KEY,
    customer_id INTEGER NOT NULL,
    order_date TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'COMPLETED',
    payment_terms TEXT NOT NULL DEFAULT 'NET30',
    subtotal REAL NOT NULL,
    vat_total REAL NOT NULL,
    total_amount REAL NOT NULL,
    FOREIGN KEY (customer_id) REFERENCES customers(id)
);

CREATE TABLE order_items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    order_number TEXT NOT NULL,
    product_sku TEXT NOT NULL,
    quantity INTEGER NOT NULL DEFAULT 1,
    unit_price REAL NOT NULL,
    vat_percent REAL NOT NULL,
    line_total REAL NOT NULL,
    FOREIGN KEY (order_number) REFERENCES orders(order_number),
    FOREIGN KEY (product_sku) REFERENCES products(sku)
);

CREATE TABLE invoices (
    invoice_number TEXT PRIMARY KEY,
    order_number TEXT NOT NULL,
    mydata_mark TEXT UNIQUE NOT NULL,
    uid TEXT NOT NULL,
    qr_code_url TEXT NOT NULL,
    issued_at TEXT NOT NULL,
    total_payable REAL NOT NULL,
    FOREIGN KEY (order_number) REFERENCES orders(order_number)
);
