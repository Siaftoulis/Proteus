//! Minimalist Web UI for Proteus Web Portal & Marketplace.
//! Assembles responsive views adhering to the "Average Joe" principle.

use axum::response::Html;
use std::sync::OnceLock;

static INDEX_PAGE: OnceLock<String> = OnceLock::new();

pub async fn index_page_handler() -> Html<&'static str> {
    let page = INDEX_PAGE.get_or_init(build_index_html);
    Html(page.as_str())
}

fn build_index_html() -> String {
    let mut out = String::with_capacity(49152);
    out.push_str(HTML_HEAD_START);
    out.push_str(crate::ui_css::CSS_STYLES);
    out.push_str(HTML_NAV_AND_OVERVIEW);
    out.push_str(crate::ui_marketplace::MARKETPLACE_HTML);
    out.push_str(crate::ui_freelance::FREELANCE_HTML);
    out.push_str(crate::ui_certifications::CERTIFICATIONS_HTML);
    out.push_str(crate::ui_contracts::CONTRACTS_HTML);
    out.push_str(HTML_PRICING_AND_SCRIPTS_START);
    out.push_str(crate::ui_marketplace::MARKETPLACE_JS);
    out.push_str(crate::ui_freelance::FREELANCE_JS);
    out.push_str(crate::ui_certifications::CERTIFICATIONS_JS);
    out.push_str(crate::ui_contracts::CONTRACTS_JS);
    out.push_str(HTML_SCRIPTS_END);
    out
}

const HTML_HEAD_START: &str = r#"<!DOCTYPE html>
<html lang="el">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Proteus — Web Hub & Marketplace</title>
    <link rel="preconnect" href="https://fonts.googleapis.com">
    <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
    <link href="https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700&display=swap" rel="stylesheet">
    <style>
"#;

const HTML_NAV_AND_OVERVIEW: &str = r#"
    </style>
</head>
<body>
    <header>
        <div class="brand-wrap">
            <div class="brand">
                <span>PROTEUS</span>
                <span class="brand-badge">HUB & MARKETPLACE</span>
            </div>
            <nav class="nav-tabs">
                <button class="tab-btn active" onclick="switchTab('overview', this)">📊 Επισκόπηση</button>
                <button class="tab-btn" onclick="switchTab('marketplace', this)">🛒 Marketplace</button>
                <button class="tab-btn" onclick="switchTab('account', this)">👥 Λογαριασμός & Ρόλοι</button>
                <button class="tab-btn" onclick="switchTab('freelance', this)">💼 Freelancing Hub</button>
                <button class="tab-btn" onclick="switchTab('certifications', this)">🎓 Πιστοποιήσεις</button>
                <button class="tab-btn" onclick="switchTab('contracts', this)">📜 Συμβόλαια SLA</button>
                <button class="tab-btn" onclick="switchTab('pricing', this)">⚡ Κοστολόγηση</button>
            </nav>
        </div>
        <div class="status-pill">
            <span class="status-dot"></span>
            <span>Online (Port 8080)</span>
        </div>
    </header>

    <main>
        <!-- Tab 1: Overview -->
        <div class="tab-panel active" id="panel-overview">
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

            <div class="card">
                <div class="card-header">
                    <span>Πρόσφατες Παραλαβές & Επισκευές</span>
                    <span style="font-size: 0.75rem; color: var(--text-muted);" id="tickets-count">0 εγγραφές</span>
                </div>
                <div class="ticket-list" id="tickets-container">
                    <div style="text-align: center; color: var(--text-muted); padding: 1.5rem;">Φόρτωση εγγραφών...</div>
                </div>
            </div>
        </div>
"#;

const HTML_PRICING_AND_SCRIPTS_START: &str = r#"
        <!-- Tab 5: Pricing -->
        <div class="tab-panel" id="panel-pricing">
            <div class="section-header">
                <h2 class="section-title">Υπολογισμός Συνδρομής (Pricing Engine)</h2>
                <p class="section-sub">Διαφανής κλιμάκωση θέσεων εργασίας και πρόσθετων υπηρεσιών (Doc 18 Sec 6).</p>
            </div>

            <div class="grid-2">
                <div class="card">
                    <div class="card-header">
                        <span>Παράμετροι Συνδρομής</span>
                        <span class="badge badge-blue">Interactive Slider</span>
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
                </div>

                <div class="card" style="justify-content: center;">
                    <div style="background: var(--bg-base); border: 1px solid var(--border); border-radius: var(--radius); padding: 1.5rem; display: flex; justify-content: space-between; align-items: center;">
                        <div>
                            <div style="font-size: 0.75rem; color: var(--text-muted); text-transform: uppercase;">Μηνιαιο Κοστος</div>
                            <div style="font-size: 1.8rem; font-weight: 700; color: #fff;" id="monthly-val">7.99 €</div>
                        </div>
                        <div style="text-align: right;">
                            <div style="font-size: 0.75rem; color: var(--text-muted); text-transform: uppercase;">Ετησιο (2 Μηνες Δωρο)</div>
                            <div style="font-size: 1.3rem; font-weight: 700; color: var(--success);" id="annual-val">79.90 €</div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    </main>

    <footer>
        Proteus Web Hub & Marketplace &copy; 2026 &mdash; 100% Original Minimalist Architecture
    </footer>

    <script>
        function switchTab(tabKey, btn) {
            document.querySelectorAll('.tab-btn').forEach(b => b.classList.remove('active'));
            if (btn) btn.classList.add('active');

            document.querySelectorAll('.tab-panel').forEach(p => p.classList.remove('active'));
            const target = document.getElementById('panel-' + tabKey);
            if (target) target.classList.add('active');
        }

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

                    const badgeMap = {
                        'received': 'badge-blue',
                        'in_progress': 'badge-amber',
                        'waiting_parts': 'badge-purple',
                        'ready': 'badge-green',
                        'delivered': 'badge-gray',
                        'cancelled': 'badge-gray'
                    };
                    const statusText = {
                        'received': 'Παραλήφθηκε',
                        'in_progress': 'Σε Επισκευή',
                        'waiting_parts': 'Αναμονή Ανταλλακτικών',
                        'ready': 'Έτοιμο προς Παράδοση',
                        'delivered': 'Παραδόθηκε',
                        'cancelled': 'Ακυρώθηκε'
                    }[t.current_status] || t.current_status;

                    return `
                        <div class="ticket-row">
                            <div class="ticket-info">
                                <div class="ticket-title">#` + t.ticket_number + ` &mdash; ` + t.device_model + `</div>
                                <div class="ticket-meta">` + t.customer_name + ` &bull; ` + t.reported_fault + `</div>
                            </div>
                            <span class="badge ` + (badgeMap[t.current_status] || 'badge-gray') + `">` + statusText + `</span>
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
"#;

const HTML_SCRIPTS_END: &str = r#"
        fetchTickets();
        updatePricing();
        updatePartnerPayout();
        setInterval(fetchTickets, 10000);
    </script>
</body>
</html>
"#;
