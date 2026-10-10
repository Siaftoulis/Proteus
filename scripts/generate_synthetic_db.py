#!/usr/bin/env python3
"""
Proteus Business OS — Synthetic Relational Dataset & DDL Generator
Generates realistic multi-table SQLite databases with foreign-key relations
and exports clean SQL DDL scripts for visual canvas ingestion in Proteus Design Studio.

100% original bespoke implementation using only Python standard library.
"""

import argparse
import datetime
import os
import random
import sqlite3
import sys
from pathlib import Path

# Greek & European localized business seed data
FIRST_NAMES = [
    "Panagiotis", "Dimitrios", "Konstantinos", "Nikolaos", "Georgios",
    "Ioannis", "Eleni", "Maria", "Aikaterini", "Vasiliki", "Sophia",
    "Alexandros", "Andreas", "Stefanos", "Christina", "Theodoros"
]

LAST_NAMES = [
    "Siaftoulis", "Papadopoulos", "Oikonomou", "Georgiou", "Nikolaidis",
    "Dimopoulos", "Katsaros", "Vasiliou", "Athanasiou", "Ioannidis",
    "Christodoulou", "Konstantinidis", "Michailidis", "Papantoniou"
]

COMPANY_SUFFIXES = [
    "Automations S.A.", "Logistics Ltd", "Electronics I.K.E.",
    "Hardware Solutions", "Retail Systems", "Engineering Group",
    "Industrial Tech", "Commerce Hub", "Trading Corp"
]

CITIES = ["Athens", "Thessaloniki", "Patras", "Heraklion", "Larissa", "Volos", "Ioannina", "Chania", "Rhodes"]

PRODUCT_CATALOG = [
    ("ESL-75-BW", "Electronic Shelf Label 7.5 Inch E-Ink", "IoT Display", 28.50, 24.0),
    ("ESL-42-BWR", "Electronic Shelf Label 4.2 Inch 3-Color", "IoT Display", 18.20, 24.0),
    ("ESL-29-BW", "Electronic Shelf Label 2.9 Inch Shelf Tag", "IoT Display", 11.90, 24.0),
    ("POS-80-USB", "Thermal POS Receipt Printer 80mm ESC/POS", "POS Hardware", 89.00, 24.0),
    ("POS-58-BT", "Mobile Bluetooth Receipt Printer 58mm", "POS Hardware", 52.00, 24.0),
    ("SCL-30-RS232", "Digital Retail Price Computing Scale 30kg", "Hardware Sensors", 145.00, 24.0),
    ("RLY-04-ETH", "4-Channel Industrial Ethernet Relay Controller", "Actuators", 44.00, 24.0),
    ("RLY-08-DIN", "8-Channel DIN-Rail Automation Actuator", "Actuators", 76.50, 24.0),
    ("BAR-2D-QR", "Omnidirectional 2D QR-Code Barcode Scanner", "Scanners", 68.00, 24.0),
    ("APL-STATION-PRO", "Proteus Sovereign Station Appliance Box", "Appliance Hub", 210.00, 24.0),
    ("TH-SENSOR-RS485", "Modbus Digital Temperature/Humidity Sensor", "Hardware Sensors", 32.00, 24.0),
    ("VFD-POLE-2002", "Customer VFD Pole Display 2x20 Serial", "Displays", 58.00, 24.0),
]

PAYMENT_TERMS = ["NET30", "NET60", "IMMEDIATE_CARD", "BANK_TRANSFER", "CASH_ON_DELIVERY"]
ORDER_STATUSES = ["COMPLETED", "PROCESSING", "SHIPPED", "PENDING_PAYMENT", "CONFIRMED"]


def generate_afm(existing_afms: set) -> str:
    """Generates a valid 9-digit Greek VAT/Tax identification number (AFM)."""
    while True:
        digits = [random.randint(1, 9)] + [random.randint(0, 9) for _ in range(7)]
        # Checksum algorithm for Greek AFM
        weights = [256, 128, 64, 32, 16, 8, 4, 2]
        s = sum(d * w for d, w in zip(digits, weights))
        checksum = (s % 11) % 10
        digits.append(checksum)
        afm = "".join(str(d) for d in digits)
        if afm not in existing_afms:
            existing_afms.add(afm)
            return afm


def generate_ddl_script() -> str:
    """Generates clean standard SQL DDL schema with primary and foreign keys."""
    return """-- Proteus Sovereign Business OS — Relational Schema DDL
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
"""


def build_synthetic_database(
    db_path: Path,
    sql_ddl_path: Path,
    num_customers: int = 40,
    num_orders: int = 150
) -> None:
    """Constructs the SQLite database, executes DDL, and populates with realistic synthetic records."""
    db_path.parent.mkdir(parents=True, exist_ok=True)
    sql_ddl_path.parent.mkdir(parents=True, exist_ok=True)

    if db_path.exists():
        db_path.unlink()

    # 1. Write clean DDL script
    ddl_content = generate_ddl_script()
    sql_ddl_path.write_text(ddl_content, encoding="utf-8")
    print(f"[OK] Exported SQL DDL script: {sql_ddl_path} ({len(ddl_content)} bytes)")

    # 2. Connect to SQLite and enable foreign keys
    conn = sqlite3.connect(db_path)
    conn.execute("PRAGMA foreign_keys = ON;")
    conn.executescript(ddl_content)

    cursor = conn.cursor()
    base_time = datetime.datetime.now(datetime.timezone.utc) - datetime.timedelta(days=180)

    # 3. Seed Customers
    existing_afms = set()
    customer_ids = []
    for i in range(1, num_customers + 1):
        afm = generate_afm(existing_afms)
        fn = random.choice(FIRST_NAMES)
        ln = random.choice(LAST_NAMES)
        contact_name = f"{fn} {ln}"
        suffix = random.choice(COMPANY_SUFFIXES)
        company = f"{ln} {suffix}"
        email = f"{fn.lower()}.{ln.lower()}@{company.split()[0].lower()}corp.gr"
        phone = f"+30 210 {random.randint(1000000, 9999999)}"
        city = random.choice(CITIES)
        credit_limit = round(random.uniform(2000.0, 50000.0), 2)
        created_dt = (base_time + datetime.timedelta(days=random.randint(1, 100))).isoformat()

        cursor.execute(
            """INSERT INTO customers (vat_number, company_name, contact_name, contact_email,
                                      contact_phone, city, credit_limit, status, created_at)
               VALUES (?, ?, ?, ?, ?, ?, ?, 'ACTIVE', ?)""",
            (afm, company, contact_name, email, phone, city, credit_limit, created_dt)
        )
        customer_ids.append(cursor.lastrowid)

    # 4. Seed Products
    for sku, name, category, price, vat in PRODUCT_CATALOG:
        stock = random.randint(15, 250)
        reorder = random.randint(5, 20)
        cursor.execute(
            """INSERT INTO products (sku, name, category, unit_price, vat_percent, stock_quantity, reorder_threshold)
               VALUES (?, ?, ?, ?, ?, ?, ?)""",
            (sku, name, category, price, vat, stock, reorder)
        )

    # 5. Seed Orders, Order Items, and Invoices
    order_seq = 1001
    invoice_seq = 5001
    for _ in range(num_orders):
        ord_no = f"ORD-2026-{order_seq}"
        cust_id = random.choice(customer_ids)
        ord_date = (base_time + datetime.timedelta(days=random.randint(101, 180))).isoformat()
        status = random.choice(ORDER_STATUSES)
        terms = random.choice(PAYMENT_TERMS)

        num_items = random.randint(1, 5)
        picked_products = random.sample(PRODUCT_CATALOG, num_items)

        subtotal = 0.0
        vat_total = 0.0

        item_rows = []
        for p_sku, _, _, p_price, p_vat in picked_products:
            qty = random.randint(1, 8)
            line_sub = round(p_price * qty, 2)
            line_vat = round(line_sub * (p_vat / 100.0), 2)
            line_tot = round(line_sub + line_vat, 2)
            subtotal += line_sub
            vat_total += line_vat
            item_rows.append((ord_no, p_sku, qty, p_price, p_vat, line_tot))

        total_amount = round(subtotal + vat_total, 2)

        # Insert Order
        cursor.execute(
            """INSERT INTO orders (order_number, customer_id, order_date, status, payment_terms,
                                   subtotal, vat_total, total_amount)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?)""",
            (ord_no, cust_id, ord_date, status, terms, subtotal, vat_total, total_amount)
        )

        # Insert Order Items
        for item in item_rows:
            cursor.execute(
                """INSERT INTO order_items (order_number, product_sku, quantity, unit_price, vat_percent, line_total)
                   VALUES (?, ?, ?, ?, ?, ?)""",
                item
            )

        # If completed, generate official AADE myDATA invoice
        if status in ("COMPLETED", "SHIPPED"):
            inv_no = f"INV-2026-{invoice_seq}"
            mark = f"800{random.randint(100000000, 999999999)}"
            uid = f"AADE-{random.randint(100000, 999999)}-{random.randint(1000, 9999)}"
            qr_url = f"https://mydatagreece.aade.gr/verify?mark={mark}&uid={uid}"
            cursor.execute(
                """INSERT INTO invoices (invoice_number, order_number, mydata_mark, uid, qr_code_url, issued_at, total_payable)
                   VALUES (?, ?, ?, ?, ?, ?, ?)""",
                (inv_no, ord_no, mark, uid, qr_url, ord_date, total_amount)
            )
            invoice_seq += 1

        order_seq += 1

    conn.commit()

    # Query counts for verification summary
    c_count = cursor.execute("SELECT COUNT(*) FROM customers").fetchone()[0]
    p_count = cursor.execute("SELECT COUNT(*) FROM products").fetchone()[0]
    o_count = cursor.execute("SELECT COUNT(*) FROM orders").fetchone()[0]
    i_count = cursor.execute("SELECT COUNT(*) FROM order_items").fetchone()[0]
    inv_count = cursor.execute("SELECT COUNT(*) FROM invoices").fetchone()[0]
    conn.close()

    db_size = db_path.stat().st_size
    print(f"[OK] Generated SQLite database: {db_path} ({db_size:,} bytes)")
    print(f"     • Customers:    {c_count} rows")
    print(f"     • Products:     {p_count} rows")
    print(f"     • Orders:       {o_count} rows")
    print(f"     • Order Items:  {i_count} rows")
    print(f"     • Invoices:     {inv_count} rows (AADE myDATA compliance marks)")


def main():
    parser = argparse.ArgumentParser(description="Proteus Synthetic Relational Dataset Generator")
    parser.add_argument(
        "--db",
        type=Path,
        default=Path("vault/synthetic_erp.db"),
        help="Path for generated SQLite database file (default: vault/synthetic_erp.db)"
    )
    parser.add_argument(
        "--sql-out",
        type=Path,
        default=Path("vault/synthetic_erp.sql"),
        help="Path for exported SQL DDL script file (default: vault/synthetic_erp.sql)"
    )
    parser.add_argument(
        "--customers",
        type=int,
        default=40,
        help="Number of customer entities to generate (default: 40)"
    )
    parser.add_argument(
        "--orders",
        type=int,
        default=150,
        help="Number of orders to generate (default: 150)"
    )

    args = parser.parse_args()
    build_synthetic_database(args.db, args.sql_out, args.customers, args.orders)


if __name__ == "__main__":
    main()
