//! Bespoke Projects & Client Brief Submission Hub for Proteus Web Portal.
//! Enforces Zero Presets: Customers submit custom business briefs, and certified
//! specialists take on slots with In-Platform Escrow protection.

pub const MARKETPLACE_HTML: &str = r#"
    <div class="tab-panel" id="panel-marketplace">
        <div class="section-header" style="display: flex; justify-content: space-between; align-items: flex-end;">
            <div>
                <h2 class="section-title">📋 Εξατομικευμένα Έργα & Bespoke Briefs</h2>
                <p class="section-sub">Μηδενικά έτοιμα templates: Κάθε επιχείρηση περιγράφει αναλυτικά τη λειτουργία της και πιστοποιημένοι συνεργάτες υλοποιούν το λογισμικό.</p>
            </div>
            <button class="btn btn-sm" onclick="toggleBriefForm()">➕ Υποβολή Νέου Brief Επιχείρησης</button>
        </div>

        <!-- Bespoke Project Brief Submission Form -->
        <div id="brief-submission-card" class="card" style="display: none; border-color: var(--accent); margin-bottom: 1.5rem;">
            <div class="card-header">
                <span>Υποβολή Επιχειρησιακού Brief Πελάτη (Zero Presets / 100% Bespoke)</span>
                <span class="badge badge-purple">In-Platform Escrow</span>
            </div>
            <p style="font-size: 0.82rem; color: var(--text-muted); line-height: 1.4;">
                Περιγράψτε αναλυτικά τις ανάγκες σας. Το έργο θα αναρτηθεί στο 7-Slot Board και θα στελεχωθεί από πιστοποιημένους freelancers (PCD-App, PCD-Web, PCDA, PCSS, PCDS).
            </p>

            <div class="grid-2" style="margin-top: 0.75rem;">
                <div class="field-group">
                    <label>Επωνυμία Επιχείρησης / Καταστήματος (*)</label>
                    <input type="text" id="brief-business-name" placeholder="π.χ. Speedy Auto Repair Ε.Π.Ε." style="background: var(--bg-base); color: #fff; border: 1px solid var(--border); padding: 0.5rem; border-radius: 8px; font-family: inherit; font-size: 0.85rem;">
                </div>
                <div class="field-group">
                    <label>Φύση Δραστηριότητας & Αντικείμενο (*)</label>
                    <input type="text" id="brief-business-nature" placeholder="π.χ. Επισκευές Αυτοκινήτων & Εμπόριο Ανταλλακτικών" style="background: var(--bg-base); color: #fff; border: 1px solid var(--border); padding: 0.5rem; border-radius: 8px; font-family: inherit; font-size: 0.85rem;">
                </div>
                <div class="field-group">
                    <label>Προτεινόμενος Προϋπολογισμός Έργου (EUR) (*)</label>
                    <input type="number" id="brief-budget" value="650" style="background: var(--bg-base); color: #fff; border: 1px solid var(--border); padding: 0.5rem; border-radius: 8px; font-family: inherit; font-size: 0.85rem;">
                </div>
                <div class="field-group">
                    <label>Επιθυμητό Domain Name (Προαιρετικό)</label>
                    <input type="text" id="brief-domain" placeholder="π.χ. speedy-garage.gr" style="background: var(--bg-base); color: #fff; border: 1px solid var(--border); padding: 0.5rem; border-radius: 8px; font-family: inherit; font-size: 0.85rem;">
                </div>
            </div>

            <div class="field-group">
                <label>Αναλυτική Περιγραφή Καθημερινών Λειτουργιών & Ροών (*)</label>
                <textarea id="brief-operations" rows="3" placeholder="Περιγράψτε τι κάνετε καθημερινά: π.χ. παραλαβή οχήματος, καταγραφή χιλιομέτρων, ανάθεση σε μηχανικό, τιμολόγηση, έκδοση απόδειξης σε θερμικό εκτυπωτή..." style="background: var(--bg-base); color: #fff; border: 1px solid var(--border); padding: 0.5rem; border-radius: 8px; font-family: inherit; font-size: 0.85rem;"></textarea>
            </div>

            <div class="grid-2">
                <div class="field-group">
                    <label>Απαιτούμενες Οθόνες (χωρισμένες με κόμμα)</label>
                    <input type="text" id="brief-screens" value="Νέα Παραλαβή, Ροή Επισκευών Kanban, Αποθήκη Ανταλλακτικών, Ταμείο POS" style="background: var(--bg-base); color: #fff; border: 1px solid var(--border); padding: 0.5rem; border-radius: 8px; font-family: inherit; font-size: 0.85rem;">
                </div>
                <div class="field-group">
                    <label>Hardware Περιφερειακά (χωρισμένα με κόμμα)</label>
                    <input type="text" id="brief-hardware" value="Θερμικός Εκτυπωτής 80mm, Barcode Scanner, Συρτάρι Ταμείου" style="background: var(--bg-base); color: #fff; border: 1px solid var(--border); padding: 0.5rem; border-radius: 8px; font-family: inherit; font-size: 0.85rem;">
                </div>
            </div>

            <div style="display: flex; justify-content: flex-end; gap: 0.5rem; margin-top: 0.75rem;">
                <button class="btn btn-secondary btn-sm" onclick="toggleBriefForm()">Ακύρωση</button>
                <button class="btn btn-sm" onclick="submitBespokeBrief()">🚀 Υποβολή Έργου & Ανάρτηση στο 7-Slot Board</button>
            </div>
        </div>

        <!-- Active Bespoke Client Projects -->
        <div class="grid-2" id="marketplace-cards-container">
            <div class="pkg-card" data-category="Automotive">
                <div class="pkg-top">
                    <div>
                        <div class="pkg-title">Speedy Garage — Fast Automotive Intake & Thermal Print</div>
                        <div class="pkg-meta" style="margin-top: 0.35rem;">
                            <span class="badge badge-blue">Bespoke Automotive</span>
                            <span class="badge badge-purple">PCD-App Software UI</span>
                            <span class="badge badge-gray">ID: SB-GARAGE-01</span>
                        </div>
                    </div>
                    <div style="text-align: right;">
                        <div style="font-size: 1.25rem; font-weight: 700; color: #fff;">600 €</div>
                        <div style="font-size: 0.72rem; color: var(--success);">Escrow Funded</div>
                    </div>
                </div>
                <div class="pkg-desc">
                    Εξατομικευμένη ροή συνεργείου: ψηφιακές εντολές εργασίας, αποθήκη μηχανικού και ESC/POS δελτία παραλαβής.
                </div>
                <div style="display: flex; justify-content: space-between; align-items: center; border-top: 1px solid var(--border); padding-top: 0.75rem;">
                    <span style="font-size: 0.75rem; color: var(--text-muted);">Slots: 6 Διαθέσιμα &bull; Floor: 300.00 €</span>
                    <button class="btn btn-sm" onclick="viewSlotBoardDetails('SB-GARAGE-01')">📋 Προβολή 7-Slot Board</button>
                </div>
            </div>

            <div class="pkg-card" data-category="Retail">
                <div class="pkg-top">
                    <div>
                        <div class="pkg-title">Artisan Bakery — Dual Touch POS & Cash Drawer Kick</div>
                        <div class="pkg-meta" style="margin-top: 0.35rem;">
                            <span class="badge badge-blue">Bespoke Retail</span>
                            <span class="badge badge-purple">Dual Full-Stack (PCD-App + Web)</span>
                            <span class="badge badge-gray">ID: SB-BAKERY-02</span>
                        </div>
                    </div>
                    <div style="text-align: right;">
                        <div style="font-size: 1.25rem; font-weight: 700; color: #fff;">480 €</div>
                        <div style="font-size: 0.72rem; color: var(--success);">Escrow Funded</div>
                    </div>
                </div>
                <div class="pkg-desc">
                    Ταμείο λιανικής αρτοποιείου: Barcode scanner, συρτάρι ταμείου, touch κατηγορίες και αυτόματος συγχρονισμός αποθήκης.
                </div>
                <div style="display: flex; justify-content: space-between; align-items: center; border-top: 1px solid var(--border); padding-top: 0.75rem;">
                    <span style="font-size: 0.75rem; color: var(--text-muted);">Slots: 6 Διαθέσιμα &bull; Floor: 285.00 €</span>
                    <button class="btn btn-sm" onclick="viewSlotBoardDetails('SB-BAKERY-02')">📋 Προβολή 7-Slot Board</button>
                </div>
            </div>

            <div class="pkg-card" data-category="Healthcare">
                <div class="pkg-top">
                    <div>
                        <div class="pkg-title">Dental Care Pro — Patient File & Multi-Doctor Scheduling</div>
                        <div class="pkg-meta" style="margin-top: 0.35rem;">
                            <span class="badge badge-blue">Bespoke Healthcare</span>
                            <span class="badge badge-purple">PCSS Systems & DB</span>
                            <span class="badge badge-gray">ID: SB-DENTAL-03</span>
                        </div>
                    </div>
                    <div style="text-align: right;">
                        <div style="font-size: 1.25rem; font-weight: 700; color: #fff;">750 €</div>
                        <div style="font-size: 0.72rem; color: var(--success);">Escrow Funded</div>
                    </div>
                </div>
                <div class="pkg-desc">
                    Ιατρεία και οδοντιατρεία: Ιστορικό ασθενών, ημερολόγιο ραντεβού, GDPR audit logs και τιμολόγηση ασφαλιστικών ταμείων.
                </div>
                <div style="display: flex; justify-content: space-between; align-items: center; border-top: 1px solid var(--border); padding-top: 0.75rem;">
                    <span style="font-size: 0.75rem; color: var(--text-muted);">Slots: 6 Διαθέσιμα &bull; Floor: 405.00 €</span>
                    <button class="btn btn-sm" onclick="viewSlotBoardDetails('SB-DENTAL-03')">📋 Προβολή 7-Slot Board</button>
                </div>
            </div>

            <div class="pkg-card" data-category="Enterprise">
                <div class="pkg-top">
                    <div>
                        <div class="pkg-title">Custom Enterprise Storefront & Warehouse BOS</div>
                        <div class="pkg-meta" style="margin-top: 0.35rem;">
                            <span class="badge badge-blue">Bespoke Enterprise</span>
                            <span class="badge badge-purple">Full Engineering Guild</span>
                            <span class="badge badge-gray">ID: PRJ-BESPOKE-STORE</span>
                        </div>
                    </div>
                    <div style="text-align: right;">
                        <div style="font-size: 1.25rem; font-weight: 700; color: #fff;">650 €</div>
                        <div style="font-size: 0.72rem; color: var(--success);">Escrow Funded</div>
                    </div>
                </div>
                <div class="pkg-desc">
                    Πλήρης εξατομικευμένη υλοποίηση: διασύνδεση τοπικού POS με responsive e-shop, domain και cloud sync.
                </div>
                <div style="display: flex; justify-content: space-between; align-items: center; border-top: 1px solid var(--border); padding-top: 0.75rem;">
                    <span style="font-size: 0.75rem; color: var(--text-muted);">Slots: Assigned &bull; Status: Active</span>
                    <button class="btn btn-sm" onclick="alert('Το έργο έχει ολοκληρωθεί και παραδοθεί στον πελάτη.')">✓ Παραδομένο</button>
                </div>
            </div>
        </div>
    </div>
"#;

pub const MARKETPLACE_JS: &str = r#"
    function toggleBriefForm() {
        const card = document.getElementById('brief-submission-card');
        card.style.display = (card.style.display === 'none' || card.style.display === '') ? 'block' : 'none';
    }

    async function submitBespokeBrief() {
        const name = document.getElementById('brief-business-name').value.trim();
        const nature = document.getElementById('brief-business-nature').value.trim();
        const budget = parseFloat(document.getElementById('brief-budget').value) || 0;
        const operations = document.getElementById('brief-operations').value.trim();
        const screens = document.getElementById('brief-screens').value.split(',').map(s => s.trim()).filter(Boolean);
        const hardware = document.getElementById('brief-hardware').value.split(',').map(s => s.trim()).filter(Boolean);
        const domain = document.getElementById('brief-domain').value.trim();

        if (!name || !nature || !operations) {
            alert('Παρακαλώ συμπληρώστε την επωνυμία, τη φύση δραστηριότητας και τις καθημερινές λειτουργίες.');
            return;
        }

        const briefId = "SB-" + name.replace(/[^a-zA-Z0-9]/g, '').substring(0, 8).toUpperCase() + "-" + Math.floor(10 + Math.random() * 90);

        try {
            const resp = await fetch('/api/v1/marketplace/submit-brief', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({
                    brief_id: briefId,
                    business_name: name,
                    business_nature: nature,
                    daily_operations_desc: operations,
                    required_screens: screens,
                    hardware_peripherals: hardware,
                    proposed_budget_eur: budget,
                    domain_name_requested: domain || null,
                    hosting_preference: "ManagedCloud",
                    contact_email: "owner@" + name.toLowerCase().replace(/[^a-z0-9]/g, '') + ".gr",
                    submitted_at: new Date().toISOString()
                })
            });

            const data = await resp.json();
            if (data.success) {
                alert("✓ " + data.message + "\nID Έργου: " + briefId + "\nΤα κεφάλαια δεσμεύτηκαν στο In-Platform Escrow.");
                toggleBriefForm();
                
                // Add card to container
                const container = document.getElementById('marketplace-cards-container');
                const card = document.createElement('div');
                card.className = 'pkg-card';
                card.innerHTML = `
                    <div class="pkg-top">
                        <div>
                            <div class="pkg-title">${name} — ${nature}</div>
                            <div class="pkg-meta" style="margin-top: 0.35rem;">
                                <span class="badge badge-purple">Νέο Bespoke Έργο</span>
                                <span class="badge badge-gray">ID: ${briefId}</span>
                            </div>
                        </div>
                        <div style="text-align: right;">
                            <div style="font-size: 1.25rem; font-weight: 700; color: #fff;">${budget.toFixed(2)} €</div>
                            <div style="font-size: 0.72rem; color: var(--success);">Escrow Funded</div>
                        </div>
                    </div>
                    <div class="pkg-desc">${operations}</div>
                    <div style="display: flex; justify-content: space-between; align-items: center; border-top: 1px solid var(--border); padding-top: 0.75rem;">
                        <span style="font-size: 0.75rem; color: var(--text-muted);">Οθόνες: ${screens.join(', ')}</span>
                        <button class="btn btn-sm" onclick="switchTab('freelance')">💼 Δείτε τις Θέσεις στο Slot Board</button>
                    </div>
                `;
                container.prepend(card);
            } else {
                alert('Σφάλμα: ' + data.error);
            }
        } catch (e) {
            alert('Αποτυχία υποβολής brief.');
        }
    }

    function viewSlotBoardDetails(projectId) {
        alert("📋 Άνοιγμα 7-Slot Board για το έργο '" + projectId + "'.\nΜετάβαση στο Freelancing Hub για ανάληψη θέσεων από πιστοποιημένους επαγγελματίες.");
        const freelanceTab = document.querySelectorAll('.nav-tabs button')[4];
        if (freelanceTab) freelanceTab.click();
    }
"#;
