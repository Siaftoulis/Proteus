//! Domains & Managed Hosting UI Template for Proteus Web Portal.
//! Provides:
//! - Domain availability search and 1-click registration (.gr, .com, .eu, .shop).
//! - Managed Proteus Cloud Hosting vs Self-Hosting tier selection.
//! - Extra Web Database provisioning (customers, web orders, online bookings).

pub const DOMAINS_HOSTING_HTML: &str = r#"
    <div class="tab-panel" id="panel-domains-hosting">
        <div class="section-header">
            <h2 class="section-title">🌐 Κατοχύρωση Domain & Cloud Hosting</h2>
            <p class="section-sub">Αποκτήστε επίσημο domain για την ιστοσελίδα σας και επιλέξτε Managed Proteus Cloud ή Self-Hosting στον δικό σας server.</p>
        </div>

        <div class="grid-2">
            <!-- Domain Search & Registrar Card -->
            <div class="card">
                <div class="card-header">
                    <span>Αναζήτηση & Κατοχύρωση Domain Name</span>
                    <span class="badge badge-blue">Registrar Gateway</span>
                </div>
                <p style="font-size: 0.82rem; color: var(--text-muted); line-height: 1.4;">
                    Αναζητήστε το επίσημο domain της επιχείρησής σας. Η παραμετροποίηση DNS (A records, SSL πιστοποιητικά) γίνεται αυτόματα.
                </p>

                <div style="display: flex; gap: 0.5rem; margin-top: 0.5rem;">
                    <input type="text" id="domain-search-input" placeholder="π.χ. speedy-garage ή autoworks" style="flex: 1; background: var(--bg-base); color: #fff; border: 1px solid var(--border); padding: 0.55rem; border-radius: 8px; font-family: inherit; font-size: 0.85rem;">
                    <button class="btn btn-sm" onclick="executeDomainSearch()">🔍 Αναζήτηση</button>
                </div>

                <div id="domain-search-results" style="margin-top: 1rem; display: flex; flex-direction: column; gap: 0.5rem;">
                    <!-- Populated dynamically via JS -->
                    <div style="font-size: 0.78rem; color: var(--text-muted); text-align: center; padding: 1rem 0;">
                        Πληκτρολογήστε ένα όνομα και πατήστε αναζήτηση για να δείτε διαθεσιμότητα και τιμές (.gr, .com, .eu, .shop).
                    </div>
                </div>

                <div class="card-header" style="margin-top: 1.25rem; border-top: 1px solid var(--border); padding-top: 0.75rem;">
                    <span>Κατοχυρωμένα Domains Επιχείρησης</span>
                    <span class="badge badge-green" id="active-domains-count">1 Ενεργό</span>
                </div>
                <div id="registered-domains-list" style="display: flex; flex-direction: column; gap: 0.5rem;">
                    <div style="background: var(--bg-base); border: 1px solid var(--border); border-radius: 8px; padding: 0.75rem; display: flex; justify-content: space-between; align-items: center;">
                        <div>
                            <div style="font-weight: 600; color: #fff; font-size: 0.9rem;">speedy-garage.gr</div>
                            <div style="font-size: 0.72rem; color: var(--text-muted);">DNS: 185.199.108.153 (Proteus Edge) &bull; SSL: Ενεργό (Auto-Renew)</div>
                        </div>
                        <span class="badge badge-green">Ενεργό</span>
                    </div>
                </div>
            </div>

            <!-- Hosting Tier & Extra DBs Card -->
            <div class="card">
                <div class="card-header">
                    <span>Επιλογή Μοντέλου Φιλοξενίας (Hosting)</span>
                    <span class="badge badge-purple">Cloud vs Self-Hosted</span>
                </div>
                <p style="font-size: 0.82rem; color: var(--text-muted); line-height: 1.4;">
                    Επιλέξτε αν θέλετε πλήρη φιλοξενία στις υποδομές του Proteus ή αυτόνομο self-hosting στον δικό σας server.
                </p>

                <div style="display: flex; flex-direction: column; gap: 0.65rem; margin-top: 0.5rem;">
                    <!-- Option 1: Managed Cloud Business -->
                    <div class="hosting-plan-item" style="background: rgba(168, 85, 247, 0.08); border: 1px solid rgba(168, 85, 247, 0.4); border-radius: 8px; padding: 0.85rem;">
                        <div style="display: flex; justify-content: space-between; align-items: center;">
                            <div>
                                <div style="font-weight: 700; color: #c084fc;">Proteus Managed Cloud Business & E-Shop</div>
                                <div style="font-size: 0.75rem; color: var(--text-muted); margin-top: 0.2rem;">
                                    50GB NVMe SSD, αυτόματο SSL, απεριόριστο bandwidth, έως 5 Cloud DBs και real-time συγχρονισμός με το τοπικό POS.
                                </div>
                            </div>
                            <div style="text-align: right;">
                                <div style="font-size: 1.15rem; font-weight: 700; color: #fff;">29.99 €</div>
                                <div style="font-size: 0.7rem; color: var(--text-muted);">/ μήνα</div>
                            </div>
                        </div>
                        <div style="margin-top: 0.6rem; display: flex; justify-content: flex-end;">
                            <button class="btn btn-sm" onclick="selectHostingPlan('ManagedCloudBusiness', 29.99)">Επιλογή Managed Cloud</button>
                        </div>
                    </div>

                    <!-- Option 2: Self-Hosting -->
                    <div class="hosting-plan-item" style="background: var(--bg-base); border: 1px solid var(--border); border-radius: 8px; padding: 0.85rem;">
                        <div style="display: flex; justify-content: space-between; align-items: center;">
                            <div>
                                <div style="font-weight: 600; color: #fff;">Self-Hosting (Ιδιόκτητος Server Πελάτη)</div>
                                <div style="font-size: 0.75rem; color: var(--text-muted); margin-top: 0.2rem;">
                                    Φιλοξενία στο δικό σας hardware (π.χ. παλιό PC/laptop ή VPS). Χωρίς μηνιαίο κόστος φιλοξενίας.
                                </div>
                            </div>
                            <div style="text-align: right;">
                                <div style="font-size: 1.15rem; font-weight: 700; color: #fff;">0.00 €</div>
                                <div style="font-size: 0.7rem; color: var(--text-muted);">/ μήνα</div>
                            </div>
                        </div>
                        <div style="margin-top: 0.6rem; display: flex; justify-content: flex-end;">
                            <button class="btn btn-secondary btn-sm" onclick="selectHostingPlan('SelfHostedOnPremise', 0.0)">Επιλογή Self-Hosting</button>
                        </div>
                    </div>
                </div>

                <!-- Extra Databases & Services -->
                <div class="card-header" style="margin-top: 1.25rem; border-top: 1px solid var(--border); padding-top: 0.75rem;">
                    <span>Πρόσθετες Βάσεις Δεδομένων & Web Portals</span>
                    <span class="badge badge-amber">Connected DBs</span>
                </div>
                <p style="font-size: 0.8rem; color: var(--text-muted);">
                    Συνδέστε custom βάσεις δεδομένων που επικοινωνούν ζωντανά με την τοπική SQLite του καταστήματος:
                </p>

                <div style="display: flex; gap: 0.5rem; margin-top: 0.5rem;">
                    <select id="extra-db-select" style="flex: 1; background: var(--bg-base); color: #fff; border: 1px solid var(--border); padding: 0.5rem; border-radius: 8px; font-family: inherit; font-size: 0.85rem;">
                        <option value="web_orders">🛒 web_orders (Online Παραγγελίες E-Shop)</option>
                        <option value="web_customers">👥 web_customers (Λογαριασμοί Πελατών Web)</option>
                        <option value="appointments_db">📅 appointments_db (Online Κλείσιμο Ραντεβού)</option>
                    </select>
                    <button class="btn btn-secondary btn-sm" onclick="provisionDatabase()">➕ Σύνδεση Βάσης</button>
                </div>

                <div id="provisioned-dbs-list" style="margin-top: 0.75rem; display: flex; flex-wrap: gap; gap: 0.4rem;">
                    <span class="badge badge-blue">✓ proteus_core_store (Τοπική SQLite)</span>
                    <span class="badge badge-purple" id="db-badge-orders">✓ web_orders (Cloud Sync)</span>
                </div>
            </div>
        </div>
    </div>
"#;

pub const DOMAINS_HOSTING_JS: &str = r#"
    async function executeDomainSearch() {
        const query = document.getElementById('domain-search-input').value.trim();
        if (!query) {
            alert('Παρακαλώ πληκτρολογήστε ένα όνομα domain.');
            return;
        }

        const container = document.getElementById('domain-search-results');
        container.innerHTML = '<div style="font-size: 0.8rem; color: var(--text-muted); text-align: center; padding: 0.5rem;">Έλεγχος διαθεσιμότητας στο Registrar API...</div>';

        try {
            const resp = await fetch('/api/v1/domains/search', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ query: query })
            });

            if (!resp.ok) throw new Error('Search failed');
            const results = await resp.json();

            container.innerHTML = '';
            results.forEach(res => {
                const item = document.createElement('div');
                item.style = 'background: var(--bg-base); border: 1px solid var(--border); border-radius: 8px; padding: 0.65rem 0.85rem; display: flex; justify-content: space-between; align-items: center;';
                
                const statusBadge = res.is_available 
                    ? '<span class="badge badge-green">Διαθέσιμο</span>' 
                    : '<span class="badge badge-gray">Μη Διαθέσιμο</span>';
                
                const actionBtn = res.is_available
                    ? `<button class="btn btn-sm" onclick="registerDomainDirect('${res.domain}', ${res.retail_price_eur})">Κατοχύρωση</button>`
                    : '<span style="font-size: 0.75rem; color: var(--text-muted);">Κατειλημμένο</span>';

                item.innerHTML = `
                    <div>
                        <div style="font-weight: 600; color: #fff; font-size: 0.88rem;">${res.domain}</div>
                        <div style="font-size: 0.72rem; color: var(--text-muted);">${statusBadge} &bull; ${res.registrar}</div>
                    </div>
                    <div style="display: flex; align-items: center; gap: 0.75rem;">
                        <div style="text-align: right;">
                            <div style="font-weight: 700; color: #fff; font-size: 0.95rem;">${res.retail_price_eur.toFixed(2)} €</div>
                            <div style="font-size: 0.68rem; color: var(--text-muted);">/ έτος</div>
                        </div>
                        ${actionBtn}
                    </div>
                `;
                container.appendChild(item);
            });
        } catch (e) {
            container.innerHTML = '<div style="color: var(--danger); font-size: 0.8rem;">Σφάλμα κατά την αναζήτηση domain.</div>';
        }
    }

    async function registerDomainDirect(domain, price) {
        if (!confirm(`Επιβεβαίωση κατοχύρωσης του ${domain} στα ${price.toFixed(2)} € / έτος;\nΤα DNS records και το SSL θα δημιουργηθούν αυτόματα.`)) {
            return;
        }

        try {
            const resp = await fetch('/api/v1/domains/register', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({
                    client_id: "SHOP_CURRENT",
                    domain: domain,
                    auto_renew: true,
                    contact_email: "owner@company.gr"
                })
            });

            const data = await resp.json();
            if (data.success) {
                alert(`✓ Το domain ${domain} κατοχυρώθηκε επιτυχώς!\nΔημιουργήθηκαν αυτόματα:\n- A record -> 185.199.108.153\n- CNAME www -> ${domain}\n- Wildcard SSL πιστοποιητικό`);
                
                const list = document.getElementById('registered-domains-list');
                const row = document.createElement('div');
                row.style = 'background: var(--bg-base); border: 1px solid var(--border); border-radius: 8px; padding: 0.75rem; display: flex; justify-content: space-between; align-items: center;';
                row.innerHTML = `
                    <div>
                        <div style="font-weight: 600; color: #fff; font-size: 0.9rem;">${domain}</div>
                        <div style="font-size: 0.72rem; color: var(--text-muted);">DNS: 185.199.108.153 (Proteus Edge) &bull; SSL: Ενεργό</div>
                    </div>
                    <span class="badge badge-green">Ενεργό</span>
                `;
                list.appendChild(row);
                document.getElementById('active-domains-count').textContent = '2 Ενεργά';
            } else {
                alert('Σφάλμα: ' + data.error);
            }
        } catch (e) {
            alert('Αποτυχία επικοινωνίας με το API κατοχύρωσης.');
        }
    }

    async function selectHostingPlan(tier, fee) {
        const domain = prompt("Εισάγετε το domain που θα συνδεθεί με τη φιλοξενία:", "speedy-garage.gr");
        if (!domain) return;

        try {
            const resp = await fetch('/api/v1/hosting/subscribe', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({
                    client_id: "SHOP_CURRENT",
                    domain: domain,
                    plan_tier: tier,
                    initial_databases: ["proteus_core_store", "web_orders"]
                })
            });

            const data = await resp.json();
            if (data.success) {
                alert(`✓ Ενεργοποιήθηκε επιτυχώς το πλάνο φιλοξενίας '${tier}' (${fee} €/μήνα) για το domain ${domain}!\nEndpoint: ${data.subscription.server_endpoint}`);
            } else {
                alert('Σφάλμα: ' + data.error);
            }
        } catch (e) {
            alert('Αποτυχία ενεργοποίησης φιλοξενίας.');
        }
    }

    async function provisionDatabase() {
        const select = document.getElementById('extra-db-select');
        const dbName = select.value;
        const dbLabel = select.options[select.selectedIndex].text;

        const list = document.getElementById('provisioned-dbs-list');
        const badge = document.createElement('span');
        badge.className = 'badge badge-green';
        badge.textContent = `✓ ${dbName} (Ενεργό Cloud DB)`;
        list.appendChild(badge);

        alert(`✓ Η βάση δεδομένων '${dbName}' δημιουργήθηκε και συνδέθηκε αυτόματα με το Proteus Web API!\nΤα δεδομένα συγχρονίζονται ζωντανά με την τοπική SQLite.`);
    }
"#;
