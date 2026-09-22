//! Minimalist Web UI for Proteus Web Portal.
//! Serves a self-contained, responsive, clean web page adhering to the "Average Joe" principle.

use axum::response::Html;

pub async fn index_page_handler() -> Html<&'static str> {
    Html(INDEX_HTML)
}

const INDEX_HTML: &str = r#"<!DOCTYPE html>
<html lang="el">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Proteus — Web Hub</title>
    <link rel="preconnect" href="https://fonts.googleapis.com">
    <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
    <link href="https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700&display=swap" rel="stylesheet">
    <style>
        :root {
            --bg-base: #0c0e14;
            --bg-card: #151821;
            --bg-hover: #1c2130;
            --border: #262c3d;
            --text-main: #e2e8f0;
            --text-muted: #94a3b8;
            --accent: #3b82f6;
            --accent-hover: #2563eb;
            --success: #10b981;
            --radius: 8px;
        }

        * {
            box-sizing: border-box;
            margin: 0;
            padding: 0;
        }

        body {
            font-family: 'Inter', -apple-system, BlinkMacSystemFont, sans-serif;
            background-color: var(--bg-base);
            color: var(--text-main);
            min-height: 100vh;
            display: flex;
            flex-direction: column;
            line-height: 1.5;
        }

        header {
            border-bottom: 1px solid var(--border);
            background: rgba(21, 24, 33, 0.85);
            backdrop-filter: blur(8px);
            padding: 1rem 1.5rem;
            display: flex;
            align-items: center;
            justify-content: space-between;
        }

        .brand {
            display: flex;
            align-items: center;
            gap: 0.6rem;
            font-weight: 700;
            font-size: 1.15rem;
            letter-spacing: -0.02em;
        }

        .brand-badge {
            background: var(--accent);
            color: #fff;
            padding: 0.15rem 0.45rem;
            border-radius: 4px;
            font-size: 0.75rem;
            font-weight: 600;
        }

        .status-pill {
            display: flex;
            align-items: center;
            gap: 0.4rem;
            font-size: 0.8rem;
            color: var(--success);
            background: rgba(16, 185, 129, 0.1);
            border: 1px solid rgba(16, 185, 129, 0.25);
            padding: 0.3rem 0.75rem;
            border-radius: 20px;
        }

        .status-dot {
            width: 7px;
            height: 7px;
            background: var(--success);
            border-radius: 50%;
            display: inline-block;
        }

        main {
            flex: 1;
            max-width: 1040px;
            width: 100%;
            margin: 0 auto;
            padding: 2rem 1.5rem;
            display: flex;
            flex-direction: column;
            gap: 2rem;
        }

        .section-header {
            margin-bottom: 1rem;
        }

        .section-title {
            font-size: 1.25rem;
            font-weight: 600;
            color: #fff;
        }

        .section-sub {
            font-size: 0.875rem;
            color: var(--text-muted);
            margin-top: 0.2rem;
        }

        .grid-2 {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(320px, 1fr));
            gap: 1.25rem;
        }

        .card {
            background: var(--bg-card);
            border: 1px solid var(--border);
            border-radius: var(--radius);
            padding: 1.5rem;
            display: flex;
            flex-direction: column;
            gap: 1.25rem;
        }

        .card-header {
            font-weight: 600;
            font-size: 1rem;
            color: #fff;
            display: flex;
            align-items: center;
            justify-content: space-between;
        }

        .field-group {
            display: flex;
            flex-direction: column;
            gap: 0.4rem;
        }

        label {
            font-size: 0.85rem;
            font-weight: 500;
            color: var(--text-muted);
            display: flex;
            justify-content: space-between;
        }

        input[type="range"] {
            accent-color: var(--accent);
            cursor: pointer;
        }

        .checkbox-row {
            display: flex;
            align-items: center;
            gap: 0.6rem;
            font-size: 0.875rem;
            color: var(--text-main);
            cursor: pointer;
        }

        input[type="checkbox"] {
            accent-color: var(--accent);
            cursor: pointer;
            width: 16px;
            height: 16px;
        }

        .price-box {
            background: var(--bg-base);
            border: 1px solid var(--border);
            border-radius: var(--radius);
            padding: 1rem 1.25rem;
            display: flex;
            justify-content: space-between;
            align-items: center;
        }

        .price-val {
            font-size: 1.5rem;
            font-weight: 700;
            color: #fff;
        }

        .price-sub {
            font-size: 0.75rem;
            color: var(--text-muted);
        }

        .stats-grid {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(140px, 1fr));
            gap: 1rem;
        }

        .stat-item {
            background: var(--bg-base);
            border: 1px solid var(--border);
            border-radius: var(--radius);
            padding: 1rem;
            text-align: center;
        }

        .stat-num {
            font-size: 1.5rem;
            font-weight: 700;
            color: #fff;
        }

        .stat-label {
            font-size: 0.75rem;
            color: var(--text-muted);
            margin-top: 0.2rem;
            text-transform: uppercase;
            letter-spacing: 0.04em;
        }

        .ticket-list {
            display: flex;
            flex-direction: column;
            gap: 0.6rem;
            max-height: 280px;
            overflow-y: auto;
        }

        .ticket-row {
            background: var(--bg-base);
            border: 1px solid var(--border);
            border-radius: var(--radius);
            padding: 0.75rem 1rem;
            display: flex;
            align-items: center;
            justify-content: space-between;
            font-size: 0.875rem;
        }

        .ticket-info {
            display: flex;
            flex-direction: column;
            gap: 0.15rem;
        }

        .ticket-title {
            font-weight: 600;
            color: #fff;
        }

        .ticket-meta {
            font-size: 0.75rem;
            color: var(--text-muted);
        }

        .badge {
            font-size: 0.75rem;
            padding: 0.2rem 0.5rem;
            border-radius: 4px;
            font-weight: 600;
        }

        .badge-received { background: rgba(59, 130, 246, 0.15); color: #60a5fa; }
        .badge-in_progress { background: rgba(245, 158, 11, 0.15); color: #fbbf24; }
        .badge-ready { background: rgba(16, 185, 129, 0.15); color: #34d399; }
        .badge-delivered { background: rgba(148, 163, 184, 0.15); color: #94a3b8; }

        footer {
            border-top: 1px solid var(--border);
            padding: 1.25rem 1.5rem;
            text-align: center;
            font-size: 0.8rem;
            color: var(--text-muted);
        }
    </style>
</head>
<body>
    <header>
        <div class="brand">
            <span>PROTEUS</span>
            <span class="brand-badge">WEB HUB</span>
        </div>
        <div class="status-pill">
            <span class="status-dot"></span>
            <span>Online (Port 8080)</span>
        </div>
    </header>

    <main>
        <div>
            <div class="section-header">
                <h1 class="section-title">Επισκόπηση Συστήματος Proteus</h1>
                <p class="section-sub">Κεντρικός πίνακας ελέγχου και παρακολούθηση λειτουργίας καταστήματος.</p>
            </div>

            <div class="stats-grid">
                <div class="stat-item">
                    <div class="stat-num" id="stat-total">-</div>
                    <div class="stat-label">Συνολικες Παραλαβες</div>
                </div>
                <div class="stat-item">
                    <div class="stat-num" id="stat-active">-</div>
                    <div class="stat-label">Σε Επισκευη</div>
                </div>
                <div class="stat-item">
                    <div class="stat-num" id="stat-ready">-</div>
                    <div class="stat-label">Ετοιμες προς Παραδοση</div>
                </div>
                <div class="stat-item">
                    <div class="stat-num" id="stat-sync">100%</div>
                    <div class="stat-label">Τοπικη Βαση (SQLite)</div>
                </div>
            </div>
        </div>

        <div class="grid-2">
            <!-- Tickets List -->
            <div class="card">
                <div class="card-header">
                    <span>Πρόσφατες Παραλαβές & Επισκευές</span>
                    <span style="font-size: 0.75rem; color: var(--text-muted);" id="tickets-count">0 εγγραφές</span>
                </div>
                <div class="ticket-list" id="tickets-container">
                    <div style="text-align: center; color: var(--text-muted); padding: 1rem;">Φόρτωση εγγραφών...</div>
                </div>
            </div>

            <!-- Pricing Calculator -->
            <div class="card">
                <div class="card-header">
                    <span>Υπολογισμός Συνδρομής (Pricing Engine)</span>
                    <span class="badge" style="background: rgba(59,130,246,0.15); color: #60a5fa;">Doc 18 Sec 6</span>
                </div>

                <div class="field-group">
                    <label>
                        <span>Θέσεις Εργασίας (Seats): <strong id="seats-val">4</strong></span>
                        <span id="seats-tier-text" style="color: var(--accent);">Βασικό (1-4)</span>
                    </label>
                    <input type="range" id="seats-slider" min="1" max="180" value="4" oninput="updatePricing()">
                </div>

                <div class="field-group" style="gap: 0.75rem;">
                    <label class="checkbox-row">
                        <input type="checkbox" id="cloud-sync" onchange="updatePricing()">
                        <span>Cloud Sync & Terminal Roaming</span>
                    </label>
                    <label class="checkbox-row">
                        <input type="checkbox" id="cloud-backup" onchange="updatePricing()">
                        <span>Automated Cloud Backups (Daily)</span>
                    </label>
                </div>

                <div class="price-box">
                    <div>
                        <div class="price-sub">ΜΗΝΙΑΙΟ ΚΟΣΤΟΣ</div>
                        <div class="price-val" id="monthly-val">7.99 €</div>
                    </div>
                    <div style="text-align: right;">
                        <div class="price-sub">ΕΤΗΣΙΟ (2 ΜΗΝΕΣ ΔΩΡΟ)</div>
                        <div style="font-size: 1.1rem; font-weight: 600; color: var(--success);" id="annual-val">79.90 €</div>
                    </div>
                </div>
            </div>
        </div>
    </main>

    <footer>
        Proteus BOS &copy; 2026 &mdash; 100% Original Minimalist Architecture
    </footer>

    <script>
        async function fetchTickets() {
            try {
                const res = await fetch('/api/v1/tickets');
                if (!res.ok) throw new Error('API error');
                const tickets = await res.json();
                
                const container = document.getElementById('tickets-container');
                document.getElementById('tickets-count').textContent = tickets.length + ' εγγραφές';
                document.getElementById('stat-total').textContent = tickets.length;
                
                let active = 0;
                let ready = 0;

                if (tickets.length === 0) {
                    container.innerHTML = '<div style="text-align: center; color: var(--text-muted); padding: 1.5rem;">Δεν υπάρχουν καταχωρημένες παραλαβές ακόμα.</div>';
                    document.getElementById('stat-active').textContent = '0';
                    document.getElementById('stat-ready').textContent = '0';
                    return;
                }

                container.innerHTML = tickets.map(t => {
                    if (t.current_status === 'in_progress' || t.current_status === 'received' || t.current_status === 'waiting_parts') active++;
                    if (t.current_status === 'ready') ready++;

                    const badgeClass = 'badge-' + t.current_status;
                    const statusText = {
                        'received': 'Παραλήφθηκε',
                        'in_progress': 'Σε Επισκευή',
                        'waiting_parts': 'Αναμονή',
                        'ready': 'Έτοιμο',
                        'delivered': 'Παραδόθηκε',
                        'cancelled': 'Ακυρώθηκε'
                    }[t.current_status] || t.current_status;

                    return `
                        <div class="ticket-row">
                            <div class="ticket-info">
                                <div class="ticket-title">#${t.ticket_number} &mdash; ${t.device_model}</div>
                                <div class="ticket-meta">${t.customer_name} &bull; ${t.reported_fault}</div>
                            </div>
                            <span class="badge ${badgeClass}">${statusText}</span>
                        </div>
                    `;
                }).join('');

                document.getElementById('stat-active').textContent = active;
                document.getElementById('stat-ready').textContent = ready;
            } catch (err) {
                document.getElementById('tickets-container').innerHTML = '<div style="text-align: center; color: var(--text-muted); padding: 1rem;">Αναμονή δεδομένων τοπικής βάσης...</div>';
                document.getElementById('stat-total').textContent = '0';
                document.getElementById('stat-active').textContent = '0';
                document.getElementById('stat-ready').textContent = '0';
            }
        }

        async function updatePricing() {
            const seats = parseInt(document.getElementById('seats-slider').value, 10);
            const includeSync = document.getElementById('cloud-sync').checked;
            const includeBackups = document.getElementById('cloud-backup').checked;

            document.getElementById('seats-val').textContent = seats;

            let tierText = "Βασικό (1-4)";
            if (seats >= 150) tierText = "Enterprise Cap";
            else if (seats > 60) tierText = "Ζώνη 3 (61-150)";
            else if (seats > 20) tierText = "Ζώνη 2 (21-60)";
            else if (seats > 4) tierText = "Ζώνη 1 (5-20)";
            document.getElementById('seats-tier-text').textContent = tierText;

            try {
                const res = await fetch('/api/v1/pricing/quote', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({
                        seats: seats,
                        include_cloud_sync: includeSync,
                        include_cloud_backups: includeBackups
                    })
                });
                if (res.ok) {
                    const data = await res.json();
                    document.getElementById('monthly-val').textContent = data.total_monthly.toFixed(2) + ' €';
                    document.getElementById('annual-val').textContent = data.annual_total_with_discount.toFixed(2) + ' €';
                }
            } catch (e) {
                console.error(e);
            }
        }

        fetchTickets();
        updatePricing();
        setInterval(fetchTickets, 10000);
    </script>
</body>
</html>
"#;
