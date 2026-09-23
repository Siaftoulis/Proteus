//! Digital SLA Contracts & Escrow Simulation UI Template for Proteus Web.
//! Showcases the digital agreement framework, escrow locking, and dual-party release.

pub const CONTRACTS_HTML: &str = r#"
    <div class="tab-panel" id="panel-contracts">
        <div class="section-header">
            <h2 class="section-title">Ψηφιακά Συμβόλαια SLA & Escrow</h2>
            <p class="section-sub">Δεσμευτικές συμφωνίες επιπέδου εξυπηρέτησης (SLA) μεταξύ καταστημάτων και πιστοποιημένων τεχνικών.</p>
        </div>

        <div class="grid-2">
            <!-- SLA Contract Lifecycle Flow -->
            <div class="card">
                <div class="card-header">
                    <span>Προσομοίωση Κύκλου SLA Συμβολαίου</span>
                    <span class="badge badge-blue">Escrow Engine</span>
                </div>

                <div style="display: flex; flex-direction: column; gap: 0.75rem;">
                    <div class="field-group">
                        <label>Κωδικός Καταστήματος (Shop ID)</label>
                        <input type="text" id="sla-shop-id" value="SHOP-THESSALONIKI-01" style="background: var(--bg-base); color: #fff; border: 1px solid var(--border); padding: 0.5rem; border-radius: 6px; font-family: inherit; font-size: 0.85rem;">
                    </div>

                    <div class="field-group">
                        <label>Πιστοποιημένος Τεχνικός (PCDS Partner)</label>
                        <input type="text" id="sla-tech-id" value="TECH-NIKOS-PCDS" style="background: var(--bg-base); color: #fff; border: 1px solid var(--border); padding: 0.5rem; border-radius: 6px; font-family: inherit; font-size: 0.85rem;">
                    </div>

                    <div class="field-group">
                        <label>Μηνιαία Αμοιβή Υποστήριξης (Retainer EUR)</label>
                        <input type="number" id="sla-retainer-amount" value="350" style="background: var(--bg-base); color: #fff; border: 1px solid var(--border); padding: 0.5rem; border-radius: 6px; font-family: inherit; font-size: 0.85rem;">
                    </div>

                    <div style="display: flex; gap: 0.5rem; margin-top: 0.5rem;">
                        <button class="btn" onclick="stepCreateSla()">1. Δημιουργία</button>
                        <button class="btn btn-secondary" id="btn-sign-shop" onclick="stepSignShop()" disabled>2. Υπογραφή Καταστήματος</button>
                        <button class="btn btn-secondary" id="btn-sign-tech" onclick="stepSignTech()" disabled>3. Υπογραφή Τεχνικού</button>
                    </div>

                    <div style="display: flex; gap: 0.5rem; margin-top: 0.25rem;">
                        <button class="btn btn-success" id="btn-fund-escrow" onclick="stepFundEscrow()" disabled>4. Δέσμευση Escrow</button>
                        <button class="btn btn-secondary" id="btn-payout" onclick="stepReleasePayout()" disabled>5. Αποδέσμευση Αμοιβής</button>
                    </div>
                </div>
            </div>

            <!-- Live Status & Event Audit -->
            <div class="card">
                <div class="card-header">
                    <span>Κατάσταση Συμβολαίου & Audit Log</span>
                    <span class="badge badge-gray" id="sla-status-badge">Αναμονή Έναρξης</span>
                </div>

                <div style="background: var(--bg-base); border: 1px solid var(--border); border-radius: var(--radius); padding: 1rem; display: flex; flex-direction: column; gap: 0.6rem; font-size: 0.85rem;">
                    <div><strong>Συμβόλαιο:</strong> <span id="sla-display-id" style="color: var(--text-muted);">-</span></div>
                    <div><strong>Κατάστημα:</strong> <span id="sla-display-shop">-</span></div>
                    <div><strong>Τεχνικός:</strong> <span id="sla-display-tech">-</span></div>
                    <div><strong>Δεσμευμένα στο Escrow:</strong> <span id="sla-display-escrow" style="color: var(--warning); font-weight: 700;">0.00 €</span></div>
                    <div><strong>Αποδεσμευμένη Αμοιβή:</strong> <span id="sla-display-paid" style="color: var(--success); font-weight: 700;">0.00 €</span></div>
                </div>

                <div style="font-size: 0.75rem; color: var(--text-muted);" id="sla-audit-log">
                    Κανένα ενεργό συμβόλαιο. Πατήστε '1. Δημιουργία' για έναρξη ροής.
                </div>
            </div>
        </div>
    </div>
"#;

pub const CONTRACTS_JS: &str = r#"
    let currentContract = null;

    async function stepCreateSla() {
        const contractId = "SLA-" + Math.floor(1000 + Math.random() * 9000);
        const shopId = document.getElementById('sla-shop-id').value;
        const techId = document.getElementById('sla-tech-id').value;
        const retainer = parseFloat(document.getElementById('sla-retainer-amount').value);

        try {
            const res = await fetch('/api/v1/contracts/create', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({
                    contract_id: contractId,
                    shop_id: shopId,
                    technician_id: techId,
                    service_scope: "24/7 POS Hardware & SQLite Data Integrity",
                    response_time_hours: 4,
                    monthly_retainer_eur: retainer
                })
            });
            if (res.ok) {
                currentContract = await res.json();
                updateSlaUI("Δημιουργήθηκε επιτυχώς draft συμβόλαιο.");
                document.getElementById('btn-sign-shop').disabled = false;
            }
        } catch (e) {
            console.error(e);
        }
    }

    async function stepSignShop() {
        if (!currentContract) return;
        try {
            const res = await fetch('/api/v1/contracts/sign', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ contract: currentContract, signer_role: "shop" })
            });
            if (res.ok) {
                currentContract = await res.json();
                updateSlaUI("Υπογράφηκε από το κατάστημα.");
                document.getElementById('btn-sign-tech').disabled = false;
            }
        } catch (e) {
            console.error(e);
        }
    }

    async function stepSignTech() {
        if (!currentContract) return;
        try {
            const res = await fetch('/api/v1/contracts/sign', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ contract: currentContract, signer_role: "technician" })
            });
            if (res.ok) {
                currentContract = await res.json();
                updateSlaUI("Υπογράφηκε από τον πιστοποιημένο τεχνικό. Ενεργό!");
                document.getElementById('btn-fund-escrow').disabled = false;
            }
        } catch (e) {
            console.error(e);
        }
    }

    async function stepFundEscrow() {
        if (!currentContract) return;
        const retainer = currentContract.monthly_retainer_eur;
        try {
            const res = await fetch('/api/v1/contracts/fund', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ contract: currentContract, amount: retainer })
            });
            if (res.ok) {
                currentContract = await res.json();
                updateSlaUI("Δεσμεύτηκαν " + retainer.toFixed(2) + "€ στον λογαριασμό Escrow.");
                document.getElementById('btn-payout').disabled = false;
            }
        } catch (e) {
            console.error(e);
        }
    }

    async function stepReleasePayout() {
        if (!currentContract) return;
        try {
            const res = await fetch('/api/v1/contracts/payout', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ contract: currentContract })
            });
            if (res.ok) {
                const data = await res.json();
                currentContract = data.contract;
                updateSlaUI("Επιτυχής εκκαθάριση! Αποδεσμεύτηκαν " + data.payout_eur.toFixed(2) + "€ στον τεχνικό.");
            }
        } catch (e) {
            console.error(e);
        }
    }

    function updateSlaUI(eventMsg) {
        if (!currentContract) return;
        document.getElementById('sla-display-id').textContent = currentContract.contract_id;
        document.getElementById('sla-display-shop').textContent = currentContract.shop_id + (currentContract.shop_signature ? " (Υπογεγραμμένο)" : "");
        document.getElementById('sla-display-tech').textContent = currentContract.technician_id + (currentContract.technician_signature ? " (Υπογεγραμμένο)" : "");
        document.getElementById('sla-display-escrow').textContent = currentContract.escrow_locked_eur.toFixed(2) + ' €';
        document.getElementById('sla-status-badge').textContent = currentContract.status;
        document.getElementById('sla-status-badge').className = "badge badge-green";
        document.getElementById('sla-audit-log').textContent = "✓ " + eventMsg;
    }
"#;
