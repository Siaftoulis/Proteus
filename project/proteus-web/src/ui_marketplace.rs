//! Marketplace Tab UI Template & Catalog Logic for Proteus Web.
//! Renders verified .pr business package cards, category filtering, and package details.

pub const MARKETPLACE_HTML: &str = r#"
    <div class="tab-panel" id="panel-marketplace">
        <div class="section-header" style="display: flex; justify-content: space-between; align-items: flex-end;">
            <div>
                <h2 class="section-title">Proteus Package Marketplace</h2>
                <p class="section-sub">Επαληθευμένα επιχειρησιακά πρότυπα σχεδιασμένα από πιστοποιημένους συνεργάτες (PCD & PCDA).</p>
            </div>
            <div style="display: flex; gap: 0.5rem;" id="category-filters">
                <button class="btn btn-secondary btn-sm active" onclick="filterMarketplace('all', this)">Όλα</button>
                <button class="btn btn-secondary btn-sm" onclick="filterMarketplace('Automotive', this)">Automotive</button>
                <button class="btn btn-secondary btn-sm" onclick="filterMarketplace('Retail', this)">Retail</button>
                <button class="btn btn-secondary btn-sm" onclick="filterMarketplace('Healthcare', this)">Healthcare</button>
            </div>
        </div>

        <div class="grid-2" id="marketplace-cards-container">
            <!-- Dynamically populated or initial fallback -->
            <div class="pkg-card" data-category="Automotive">
                <div class="pkg-top">
                    <div>
                        <div class="pkg-title">Automotive Service & Repair BOS</div>
                        <div class="pkg-meta" style="margin-top: 0.35rem;">
                            <span class="badge badge-blue">Automotive</span>
                            <span class="badge badge-gray">v1.4.0</span>
                            <span class="badge badge-purple">PCD Senior Partner</span>
                        </div>
                    </div>
                    <div style="text-align: right;">
                        <div style="font-size: 1.25rem; font-weight: 700; color: #fff;">180 €</div>
                        <div style="font-size: 0.72rem; color: var(--text-muted);">One-time + License</div>
                    </div>
                </div>
                <div class="pkg-desc">
                    Πλήρες πακέτο συνεργείου: εντολές εργασίας, αποθήκη ανταλλακτικών μηχανικού, διαγνωστικός έλεγχος και εκτυπώσεις ESC/POS.
                </div>
                <div style="display: flex; justify-content: space-between; align-items: center; border-top: 1px solid var(--border); padding-top: 0.75rem;">
                    <span style="font-size: 0.75rem; color: var(--text-muted);">Bundle: <code>PKG-SERVICE-AUTO</code></span>
                    <button class="btn btn-sm" onclick="installPackage('PKG-SERVICE-AUTO', 'Automotive Service & Repair BOS')">📥 Εγκατάσταση</button>
                </div>
            </div>

            <div class="pkg-card" data-category="Retail">
                <div class="pkg-top">
                    <div>
                        <div class="pkg-title">Multi-Store Retail & Cashier BOS</div>
                        <div class="pkg-meta" style="margin-top: 0.35rem;">
                            <span class="badge badge-blue">Retail</span>
                            <span class="badge badge-gray">v2.1.0</span>
                            <span class="badge badge-purple">PCDA Analyst Group</span>
                        </div>
                    </div>
                    <div style="text-align: right;">
                        <div style="font-size: 1.25rem; font-weight: 700; color: #fff;">240 €</div>
                        <div style="font-size: 0.72rem; color: var(--text-muted);">One-time + License</div>
                    </div>
                </div>
                <div class="pkg-desc">
                    Λιανικό εμπόριο και ταμεία: Barcode scanner, συρτάρι ταμείου, αυτόματη αναπαραγγελία αποθήκης και συγχρονισμός καταστημάτων.
                </div>
                <div style="display: flex; justify-content: space-between; align-items: center; border-top: 1px solid var(--border); padding-top: 0.75rem;">
                    <span style="font-size: 0.75rem; color: var(--text-muted);">Bundle: <code>PKG-RETAIL-POS</code></span>
                    <button class="btn btn-sm" onclick="installPackage('PKG-RETAIL-POS', 'Multi-Store Retail & Cashier BOS')">📥 Εγκατάσταση</button>
                </div>
            </div>

            <div class="pkg-card" data-category="Healthcare">
                <div class="pkg-top">
                    <div>
                        <div class="pkg-title">Medical & Dental Practice Suite</div>
                        <div class="pkg-meta" style="margin-top: 0.35rem;">
                            <span class="badge badge-blue">Healthcare</span>
                            <span class="badge badge-gray">v1.0.2</span>
                            <span class="badge badge-purple">PCSS Systems Architect</span>
                        </div>
                    </div>
                    <div style="text-align: right;">
                        <div style="font-size: 1.25rem; font-weight: 700; color: #fff;">320 €</div>
                        <div style="font-size: 0.72rem; color: var(--text-muted);">One-time + License</div>
                    </div>
                </div>
                <div class="pkg-desc">
                    Ιατρεία και οδοντιατρεία: Ιστορικό ασθενών, ημερολόγιο ραντεβού, GDPR audit logs και τιμολόγηση ασφαλιστικών ταμείων.
                </div>
                <div style="display: flex; justify-content: space-between; align-items: center; border-top: 1px solid var(--border); padding-top: 0.75rem;">
                    <span style="font-size: 0.75rem; color: var(--text-muted);">Bundle: <code>PKG-CLINIC-HEALTH</code></span>
                    <button class="btn btn-sm" onclick="installPackage('PKG-CLINIC-HEALTH', 'Medical & Dental Practice Suite')">📥 Εγκατάσταση</button>
                </div>
            </div>

            <div class="pkg-card" data-category="Automotive">
                <div class="pkg-top">
                    <div>
                        <div class="pkg-title">Motorcycle Workshop & Tuning BOS</div>
                        <div class="pkg-meta" style="margin-top: 0.35rem;">
                            <span class="badge badge-blue">Automotive</span>
                            <span class="badge badge-gray">v1.1.0</span>
                            <span class="badge badge-purple">PCD Specialist</span>
                        </div>
                    </div>
                    <div style="text-align: right;">
                        <div style="font-size: 1.25rem; font-weight: 700; color: #fff;">120 €</div>
                        <div style="font-size: 0.72rem; color: var(--text-muted);">One-time + License</div>
                    </div>
                </div>
                <div class="pkg-desc">
                    Συνεργεία δικύκλων: Έλεγχος πλαισίου (VIN), καταγραφή βάθους πέλματος ελαστικών και εκτύπωση φύλλου δυναμομέτρησης.
                </div>
                <div style="display: flex; justify-content: space-between; align-items: center; border-top: 1px solid var(--border); padding-top: 0.75rem;">
                    <span style="font-size: 0.75rem; color: var(--text-muted);">Bundle: <code>PKG-MOTO-PRO</code></span>
                    <button class="btn btn-sm" onclick="installPackage('PKG-MOTO-PRO', 'Motorcycle Workshop & Tuning BOS')">📥 Εγκατάσταση</button>
                </div>
            </div>
        </div>
    </div>
"#;

pub const MARKETPLACE_JS: &str = r#"
    function filterMarketplace(category, btn) {
        document.querySelectorAll('#category-filters button').forEach(b => b.classList.remove('active'));
        if (btn) btn.classList.add('active');
        
        const cards = document.querySelectorAll('#marketplace-cards-container .pkg-card');
        cards.forEach(card => {
            if (category === 'all' || card.getAttribute('data-category') === category) {
                card.style.display = 'flex';
            } else {
                card.style.display = 'none';
            }
        });
    }

    function installPackage(bundleId, title) {
        alert("📥 Το πακέτο '" + title + "' (" + bundleId + ") είναι έτοιμο για άμεση προσάρτηση στο Proteus Client!");
    }
"#;
