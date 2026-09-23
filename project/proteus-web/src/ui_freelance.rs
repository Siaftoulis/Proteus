//! Account & Role Setup and Freelance Custom Requests Hub for Proteus Web.
//! Allows businesses and certified freelancers to configure profiles, select roles,
//! submit custom CRM design requests, and connect through SLA Escrow.

pub const FREELANCE_HTML: &str = r#"
    <!-- Tab: Account & Roles Setup -->
    <div class="tab-panel" id="panel-account">
        <div class="section-header">
            <h2 class="section-title">Διαχείριση Λογαριασμού & Επιχειρησιακοί Ρόλοι</h2>
            <p class="section-sub">Ρυθμίστε τα διαπιστευτήρια και επιλέξτε τους ρόλους και τα CRM projects που συγχρονίζονται στο Proteus Desktop Client.</p>
        </div>

        <div class="grid-2">
            <!-- Account Credentials Card -->
            <div class="card">
                <div class="card-header">
                    <span>Στοιχεία Σύνδεσης & Αυθεντικοποίηση</span>
                    <span class="badge badge-blue">Portal Login</span>
                </div>

                <div class="field-group">
                    <label>Διεύθυνση Email Επιχείρησης / Freelancer</label>
                    <input type="email" id="profile-email" value="owner@autoworks.gr" style="background: var(--bg-base); color: #fff; border: 1px solid var(--border); padding: 0.55rem; border-radius: 8px; font-family: inherit; font-size: 0.85rem;">
                </div>

                <div class="field-group">
                    <label>Κωδικός Πρόσβασης</label>
                    <input type="password" id="profile-password" value="••••••••••••" style="background: var(--bg-base); color: #fff; border: 1px solid var(--border); padding: 0.55rem; border-radius: 8px; font-family: inherit; font-size: 0.85rem;">
                </div>

                <div style="display: flex; gap: 0.5rem; margin-top: 0.35rem;">
                    <button class="btn btn-secondary btn-sm" onclick="alert('✓ Προσομοίωση Google OAuth: Ο λογαριασμός συνδέθηκε επιτυχώς με το Google Workspace!')">🔑 Σύνδεση με Google</button>
                    <button class="btn btn-secondary btn-sm" onclick="alert('✓ Προσομοίωση Apple ID: Επαληθεύτηκε η ταυτότητα!')"> Σύνδεση με Apple</button>
                </div>

                <div class="card-header" style="margin-top: 0.75rem; border-top: 1px solid var(--border); padding-top: 0.75rem;">
                    <span>Ενεργά CRM Projects στο Desktop Launcher</span>
                </div>
                <div style="display: flex; flex-direction: column; gap: 0.5rem;">
                    <label class="checkbox-row">
                        <input type="checkbox" id="crm-auto" checked>
                        <span>Automotive Service & Repair BOS (PKG-SERVICE-AUTO)</span>
                    </label>
                    <label class="checkbox-row">
                        <input type="checkbox" id="crm-retail" checked>
                        <span>Multi-Store Retail & Cashier BOS (PKG-RETAIL-POS)</span>
                    </label>
                    <label class="checkbox-row">
                        <input type="checkbox" id="crm-clinic">
                        <span>Medical & Dental Practice Suite (PKG-CLINIC-HEALTH)</span>
                    </label>
                    <label class="checkbox-row">
                        <input type="checkbox" id="crm-moto" checked>
                        <span>Motorcycle Workshop & Tuning BOS (PKG-MOTO-PRO)</span>
                    </label>
                    <label class="checkbox-row">
                        <input type="checkbox" id="crm-designer" checked>
                        <span>Proteus Custom Designer Canvas & Studio</span>
                    </label>
                </div>
            </div>

            <!-- Role Selector & Desktop Sync -->
            <div class="card">
                <div class="card-header">
                    <span>Επιλογή Επαγγελματικών Ρόλων (Role Segregation)</span>
                    <span class="badge badge-purple">RBAC Profile</span>
                </div>
                <p style="font-size: 0.82rem; color: var(--text-muted); line-height: 1.4;">
                    Επιλέξτε τις αρμοδιότητες που θα εμφανίζονται στο μενού του Proteus Desktop Terminal:
                </p>

                <div style="display: flex; flex-direction: column; gap: 0.65rem;">
                    <div style="background: var(--bg-base); border: 1px solid var(--border); border-radius: 8px; padding: 0.75rem;">
                        <label class="checkbox-row" style="font-weight: 600;">
                            <input type="checkbox" id="role-designer" checked>
                            <span>🎨 PCD Designer (Σχεδιασμός Φορμών & Canvas Layouts)</span>
                        </label>
                    </div>
                    <div style="background: var(--bg-base); border: 1px solid var(--border); border-radius: 8px; padding: 0.75rem;">
                        <label class="checkbox-row" style="font-weight: 600;">
                            <input type="checkbox" id="role-analyst" checked>
                            <span>📊 PCDA Business & Data Analyst (KPIs & Schema Inference)</span>
                        </label>
                    </div>
                    <div style="background: var(--bg-base); border: 1px solid var(--border); border-radius: 8px; padding: 0.75rem;">
                        <label class="checkbox-row" style="font-weight: 600;">
                            <input type="checkbox" id="role-systems" checked>
                            <span>💻 PCSS IT & Systems DB (SQLite & Δίκτυο LAN)</span>
                        </label>
                    </div>
                    <div style="background: var(--bg-base); border: 1px solid var(--border); border-radius: 8px; padding: 0.75rem;">
                        <label class="checkbox-row" style="font-weight: 600;">
                            <input type="checkbox" id="role-support" checked>
                            <span>🛠 Customer Support & Επισκευές (Tickets & Service)</span>
                        </label>
                    </div>
                    <div style="background: var(--bg-base); border: 1px solid var(--border); border-radius: 8px; padding: 0.75rem;">
                        <label class="checkbox-row" style="font-weight: 600;">
                            <input type="checkbox" id="role-owner" checked>
                            <span>🏪 Store Owner & Director (Προσωπικό & Ταμείο POS)</span>
                        </label>
                    </div>
                </div>

                <div style="margin-top: 0.5rem;">
                    <button class="btn" style="width: 100%; border-radius: 8px;" onclick="savePortalProfile()">💾 Αποθήκευση Προφίλ & Συγχρονισμός με Desktop</button>
                </div>
            </div>
        </div>
    </div>

    <!-- Tab: Freelance & Custom Requests Hub -->
    <div class="tab-panel" id="panel-freelance">
        <div class="section-header" style="display: flex; justify-content: space-between; align-items: flex-end;">
            <div>
                <h2 class="section-title">Επαγγελματικό Freelancing & Αιτήματα Έργων</h2>
                <p class="section-sub">Γέφυρα συνεργασίας: Οι επιχειρήσεις ζητούν εξειδικευμένα CRM screens και οι πιστοποιημένοι freelancers αναλαμβάνουν υλοποίηση με εγγύηση SLA Escrow.</p>
            </div>
            <button class="btn btn-sm" onclick="toggleNewRequestForm()">➕ Νέο Αίτημα Έργου</button>
        </div>

        <!-- Collapsible Request Form -->
        <div id="new-request-card" class="card" style="display: none; border-color: var(--accent);">
            <div class="card-header">
                <span>Δημοσίευση Αιτήματος Εξατομικευμένου CRM / Screen</span>
                <span class="badge badge-blue">New Project Tender</span>
            </div>
            <div class="grid-2">
                <div class="field-group">
                    <label>Τίτλος Έργου</label>
                    <input type="text" id="req-title" placeholder="π.χ. POS & Inventory για Αρτοποιείο" style="background: var(--bg-base); color: #fff; border: 1px solid var(--border); padding: 0.5rem; border-radius: 8px; font-family: inherit; font-size: 0.85rem;">
                </div>
                <div class="field-group">
                    <label>Εκτιμώμενος Προϋπολογισμός (EUR)</label>
                    <input type="number" id="req-budget" value="350" style="background: var(--bg-base); color: #fff; border: 1px solid var(--border); padding: 0.5rem; border-radius: 8px; font-family: inherit; font-size: 0.85rem;">
                </div>
                <div class="field-group">
                    <label>Απαιτούμενη Εξειδίκευση</label>
                    <select id="req-role" style="background: var(--bg-base); color: #fff; border: 1px solid var(--border); padding: 0.5rem; border-radius: 8px; font-family: inherit; font-size: 0.85rem;">
                        <option value="PCD Designer">PCD Designer (UI, Οθόνες, Φόρμες)</option>
                        <option value="PCDA Business Analyst">PCDA Business Analyst (Ανάλυση, KPIs, Εισαγωγή Δεδομένων)</option>
                        <option value="PCSS Systems DB">PCSS Systems DB (SQLite DDL & Micro-Triggers)</option>
                        <option value="PCDS Deployer">PCDS Deployer (Εγκατάσταση LAN & Θερμικοί Εκτυπωτές)</option>
                    </select>
                </div>
                <div class="field-group">
                    <label>Προθεσμία Υλοποίησης</label>
                    <input type="text" id="req-deadline" value="7 Ημέρες" style="background: var(--bg-base); color: #fff; border: 1px solid var(--border); padding: 0.5rem; border-radius: 8px; font-family: inherit; font-size: 0.85rem;">
                </div>
            </div>
            <div class="field-group">
                <label>Περιγραφή Αναγκών & Επιχειρησιακού Σκοπού</label>
                <textarea id="req-desc" rows="3" placeholder="Περιγράψτε τις οθόνες, τα πεδία ή τα συστήματα που θέλετε να συνδεθούν..." style="background: var(--bg-base); color: #fff; border: 1px solid var(--border); padding: 0.5rem; border-radius: 8px; font-family: inherit; font-size: 0.85rem;"></textarea>
            </div>
            <div style="display: flex; justify-content: flex-end; gap: 0.5rem;">
                <button class="btn btn-secondary btn-sm" onclick="toggleNewRequestForm()">Ακύρωση</button>
                <button class="btn btn-sm" onclick="submitCustomRequest()">Ανάρτηση Αιτήματος</button>
            </div>
        </div>

        <!-- Open Freelance Gigs Grid -->
        <div class="grid-2" id="freelance-gigs-container">
            <div class="pkg-card">
                <div class="pkg-top">
                    <div>
                        <div class="pkg-title">Custom POS Layout & Barcode για Αρτοποιείο</div>
                        <div class="pkg-meta" style="margin-top: 0.35rem;">
                            <span class="badge badge-purple">PCD Designer</span>
                            <span class="badge badge-blue">Λιανική / Εστίαση</span>
                            <span class="badge badge-gray">Κατάστημα: Bakery-Athens-04</span>
                        </div>
                    </div>
                    <div style="text-align: right;">
                        <div style="font-size: 1.3rem; font-weight: 700; color: #fff;">350 €</div>
                        <div style="font-size: 0.72rem; color: var(--success);">Δεσμευμένο σε Escrow</div>
                    </div>
                </div>
                <div class="pkg-desc">
                    Σχεδίαση 3 οθονών γρήγορης επιλογής αρτοσκευασμάτων με κουμπιά αφής, άμεσο άνοιγμα συρταριού και έκδοση αποδείξεων σε θερμικό 80mm.
                </div>
                <div style="display: flex; justify-content: space-between; align-items: center; border-top: 1px solid var(--border); padding-top: 0.75rem;">
                    <span style="font-size: 0.75rem; color: var(--text-muted);">Προθεσμία: 5 ημέρες</span>
                    <button class="btn btn-sm" onclick="acceptGigAndOpenSla('Custom POS Layout για Αρτοποιείο', 350, 'Bakery-Athens-04')">🤝 Ανάληψη Έργου (SLA)</button>
                </div>
            </div>

            <div class="pkg-card">
                <div class="pkg-top">
                    <div>
                        <div class="pkg-title">Data Pipeline & Excel Migration 15,000 Ανταλλακτικών</div>
                        <div class="pkg-meta" style="margin-top: 0.35rem;">
                            <span class="badge badge-purple">PCDA Business Analyst</span>
                            <span class="badge badge-blue">Automotive Parts</span>
                            <span class="badge badge-gray">Κατάστημα: Parts-Salonica-02</span>
                        </div>
                    </div>
                    <div style="text-align: right;">
                        <div style="font-size: 1.3rem; font-weight: 700; color: #fff;">420 €</div>
                        <div style="font-size: 0.72rem; color: var(--success);">Δεσμευμένο σε Escrow</div>
                    </div>
                </div>
                <div class="pkg-desc">
                    Αυτόματη κανονικοποίηση σχήματος από Excel τιμοκαταλόγων κατασκευαστών και δημιουργία έτοιμου `.pr` πακέτου αποθήκης για το Proteus.
                </div>
                <div style="display: flex; justify-content: space-between; align-items: center; border-top: 1px solid var(--border); padding-top: 0.75rem;">
                    <span style="font-size: 0.75rem; color: var(--text-muted);">Προθεσμία: 4 ημέρες</span>
                    <button class="btn btn-sm" onclick="acceptGigAndOpenSla('Excel Data Migration 15,000 Ανταλλακτικών', 420, 'Parts-Salonica-02')">🤝 Ανάληψη Έργου (SLA)</button>
                </div>
            </div>
        </div>
    </div>
"#;

pub const FREELANCE_JS: &str = r#"
    function savePortalProfile() {
        const email = document.getElementById('profile-email').value;
        const roles = [];
        if (document.getElementById('role-designer').checked) roles.push('PCD Designer');
        if (document.getElementById('role-analyst').checked) roles.push('PCDA Analyst');
        if (document.getElementById('role-systems').checked) roles.push('PCSS Systems');
        if (document.getElementById('role-support').checked) roles.push('Support Specialist');
        if (document.getElementById('role-owner').checked) roles.push('Store Owner');

        alert("✓ Το προφίλ για τον χρήστη '" + email + "' αποθηκεύτηκε επιτυχώς!\nΕνεργοί Ρόλοι: " + roles.join(', ') + "\nΜπορείτε τώρα να ανοίξετε το Proteus Desktop Launcher και να συνδεθείτε άμεσα.");
    }

    function toggleNewRequestForm() {
        const card = document.getElementById('new-request-card');
        card.style.display = (card.style.display === 'none' || card.style.display === '') ? 'flex' : 'none';
    }

    function submitCustomRequest() {
        const title = document.getElementById('req-title').value;
        const budget = document.getElementById('req-budget').value;
        const role = document.getElementById('req-role').value;
        const desc = document.getElementById('req-desc').value;

        if (!title.trim()) {
            alert('Παρακαλώ συμπληρώστε τίτλο έργου.');
            return;
        }

        const container = document.getElementById('freelance-gigs-container');
        const newCard = document.createElement('div');
        newCard.className = 'pkg-card';
        newCard.innerHTML = `
            <div class="pkg-top">
                <div>
                    <div class="pkg-title">${title}</div>
                    <div class="pkg-meta" style="margin-top: 0.35rem;">
                        <span class="badge badge-purple">${role}</span>
                        <span class="badge badge-blue">Νέο Αίτημα</span>
                        <span class="badge badge-gray">Προθεσμία: ${document.getElementById('req-deadline').value}</span>
                    </div>
                </div>
                <div style="text-align: right;">
                    <div style="font-size: 1.3rem; font-weight: 700; color: #fff;">${budget} €</div>
                    <div style="font-size: 0.72rem; color: var(--warning);">Εκκρεμεί Ανάθεση</div>
                </div>
            </div>
            <div class="pkg-desc">${desc || 'Εξατομικευμένη σχεδίαση και παραμετροποίηση για το κατάστημα.'}</div>
            <div style="display: flex; justify-content: space-between; align-items: center; border-top: 1px solid var(--border); padding-top: 0.75rem;">
                <span style="font-size: 0.75rem; color: var(--text-muted);">Κατάσταση: Δημοσιεύτηκε</span>
                <button class="btn btn-sm" onclick="acceptGigAndOpenSla('${title}', ${budget}, 'Current-Shop')">🤝 Ανάληψη Έργου (SLA)</button>
            </div>
        `;

        container.prepend(newCard);
        toggleNewRequestForm();
        alert("✓ Το αίτημα '" + title + "' δημοσιεύτηκε επιτυχώς στο Freelancing Hub!");
    }

    function acceptGigAndOpenSla(title, budget, shopId) {
        switchTab('contracts', document.querySelectorAll('.nav-tabs button')[5]);
        document.getElementById('sla-shop-id').value = shopId;
        document.getElementById('sla-retainer-amount').value = budget;
        alert("✓ Έργο '" + title + "' αναλήφθηκε! Μεταφερθήκατε στην καρτέλα Συμβολαίων SLA για ψηφιακή υπογραφή και δέσμευση Escrow.");
    }
"#;
