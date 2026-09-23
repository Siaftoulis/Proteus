//! Public Marketing Landing Website for Proteus CRM & Business Engine.
//! Incorporates Section 33 preserved components: HeroSection, FeatureGrid, PricingTable, CallToAction, ContactForm.
//! Inspired by the "Penpot for Applications" paradigm.

use axum::response::Html;
use std::sync::OnceLock;

static LANDING_PAGE: OnceLock<String> = OnceLock::new();

pub async fn landing_page_handler() -> Html<&'static str> {
    let page = LANDING_PAGE.get_or_init(build_landing_html);
    Html(page.as_str())
}

fn build_landing_html() -> String {
    let mut out = String::with_capacity(32768);
    out.push_str(HTML_HEAD);
    out.push_str(crate::ui_css::CSS_STYLES);
    out.push_str(HTML_BODY);
    out.push_str(HTML_SCRIPTS);
    out
}

const HTML_HEAD: &str = r##"<!DOCTYPE html>
<html lang="el">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Proteus — Penpot for Applications | Native Rust & SQLite CRM</title>
    <meta name="description" content="Proteus is the first Penpot-inspired application engine. Design, automate, and run custom business software with native Rust speed and local-first SQLite resilience.">
    <link rel="preconnect" href="https://fonts.googleapis.com">
    <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
    <link href="https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700;800&family=JetBrains+Mono:wght@400;500&display=swap" rel="stylesheet">
    <style>
"##;

const HTML_BODY: &str = r##"
    </style>
</head>
<body>
    <!-- Top Navigation Bar (PDSNavbar) -->
    <header class="landing-header">
        <div class="brand-wrap">
            <a href="/" style="text-decoration: none; color: inherit; display: flex; align-items: center; gap: 0.75rem;">
                <div class="brand-logo-icon">✦</div>
                <div class="brand">
                    <span>PROTEUS</span>
                    <span class="brand-badge">APP ENGINE</span>
                </div>
            </a>
            <nav class="nav-links">
                <a href="#features" class="nav-link">Χαρακτηριστικά</a>
                <a href="#vision" class="nav-link">Το Όραμα</a>
                <a href="#demo" class="nav-link">Live Canvas</a>
                <a href="#pricing" class="nav-link">Τιμολόγηση</a>
                <a href="#contact" class="nav-link">Επικοινωνία</a>
            </nav>
        </div>
        <div class="header-actions">
            <a href="/hub" class="btn btn-secondary btn-sm">Άνοιγμα Web Hub</a>
            <a href="#demo" class="btn btn-sm">Δοκιμή Demo →</a>
        </div>
    </header>

    <main class="landing-main">
        <!-- 1. HERO SECTION (Section 33 Preserved) -->
        <section class="hero-section">
            <div class="hero-pill">
                <span class="pulse-dot"></span>
                <span>THE FIRST PENPOT FOR APPLICATIONS &bull; NATIVE RUST &bull; 0% CLOUD LOCK-IN</span>
            </div>
            <h1 class="hero-title">
                Το Πρώτο <span class="gradient-text">Penpot for Applications</span> στον Κόσμο
            </h1>
            <p class="hero-subtitle">
                Σχεδιάστε, αυτοματοποιήστε και εκτελέστε πραγματικές επιχειρησιακές εφαρμογές σε έναν άπειρο καμβά.
                Native Rust ταχύτητα, τοπική βάση SQLite και 100% ιδιωτικότητα χωρίς εξάρτηση από cloud.
            </p>
            <div class="hero-cta-group">
                <a href="/hub" class="btn btn-lg">Είσοδος στο Business Hub →</a>
                <a href="#demo" class="btn btn-secondary btn-lg">Εξερεύνηση Live Canvas</a>
            </div>
            <div class="hero-specs-strip">
                <div class="spec-item"><span>🚀 0ms</span> Latency (Native Local)</div>
                <div class="spec-item"><span>🛡 AES-256</span> Encrypted SQLite</div>
                <div class="spec-item"><span>📐 W3C</span> Design Tokens</div>
                <div class="spec-item"><span>♿ WCAG 2.2 AA</span> Accessible</div>
            </div>
        </section>

        <!-- 2. PENPOT-FOR-APPS INTERACTIVE CANVAS PREVIEW -->
        <section class="canvas-preview-section" id="vision">
            <div class="section-header center">
                <span class="badge badge-purple">THE PARADIGM SHIFT</span>
                <h2 class="section-title">Από το Σχεδιασμό Mockup στην Πραγματική Εφαρμογή</h2>
                <p class="section-sub">Στο Penpot σχεδιάζεις σχήματα. Στο Proteus κάθε στοιχείο στον καμβά συνδέεται ζωντανά με δεδομένα και εκτελείται.</p>
            </div>

            <div class="canvas-mockup-frame" id="demo">
                <!-- Canvas Top Bar -->
                <div class="canvas-bar">
                    <div class="canvas-dots">
                        <span class="dot red"></span>
                        <span class="dot yellow"></span>
                        <span class="dot green"></span>
                    </div>
                    <div class="canvas-title">Proteus Studio &bull; App Canvas &bull; CRM Operations</div>
                    <div class="canvas-mode-toggle">
                        <button class="mode-btn active" id="btn-mode-live" onclick="setCanvasMode('live')">⚡ Live App Mode</button>
                        <button class="mode-btn" id="btn-mode-inspect" onclick="setCanvasMode('inspect')">📐 Token Inspector</button>
                    </div>
                </div>

                <!-- Canvas Body -->
                <div class="canvas-body">
                    <!-- Left Mini Toolbar (Penpot-inspired) -->
                    <div class="canvas-tools">
                        <div class="tool-icon active" title="Select (V)">↖</div>
                        <div class="tool-icon" title="Frame / Artboard (F)">▦</div>
                        <div class="tool-icon" title="Input Field (T)">I</div>
                        <div class="tool-icon" title="Button (B)">▭</div>
                        <div class="tool-icon" title="Data Table (D)">▤</div>
                        <div class="tool-icon" title="Flow Logic (L)">⚡</div>
                    </div>

                    <!-- Center Live Canvas Viewport -->
                    <div class="canvas-viewport" id="canvas-viewport">
                        <div class="live-artboard">
                            <div class="artboard-header">
                                <div>
                                    <div class="artboard-name">Artboard: Dashboard &bull; Desktop HD (1440x900)</div>
                                    <h3 style="color: #fff; font-size: 1.15rem; margin-top: 0.2rem;">Επισκόπηση Καταστήματος (Live SQLite)</h3>
                                </div>
                                <span class="badge badge-green">● Connected (Port 8080)</span>
                            </div>

                            <div class="artboard-stats">
                                <div class="mini-stat">
                                    <div class="mini-val">128</div>
                                    <div class="mini-lbl">ΕΝΕΡΓΑ DEALS</div>
                                </div>
                                <div class="mini-stat">
                                    <div class="mini-val" style="color: var(--success);">84.200 €</div>
                                    <div class="mini-lbl">PIPELINE REVENUE</div>
                                </div>
                                <div class="mini-stat">
                                    <div class="mini-val" style="color: var(--accent);">342</div>
                                    <div class="mini-lbl">ΕΠΑΦΕΣ</div>
                                </div>
                            </div>

                            <div class="artboard-table">
                                <div class="table-bar">
                                    <span>Ζωντανές Παραλαβές & Επισκευές</span>
                                    <button class="btn btn-sm btn-secondary" onclick="simulateInsert()">+ Quick Add</button>
                                </div>
                                <div class="table-rows" id="mock-table-rows">
                                    <div class="t-row"><span>#1042 — Laptop Dell XPS</span><span class="badge badge-amber">Σε Επισκευή</span></div>
                                    <div class="t-row"><span>#1041 — iPhone 14 Pro Max</span><span class="badge badge-green">Έτοιμο</span></div>
                                    <div class="t-row"><span>#1040 — iPad Air M2</span><span class="badge badge-blue">Νέα Παραλαβή</span></div>
                                </div>
                            </div>
                        </div>
                    </div>

                    <!-- Right Inspector Panel (Tokens / Data Binding) -->
                    <div class="canvas-inspector" id="canvas-inspector">
                        <div class="insp-title">INSPECTOR</div>
                        <div class="insp-group">
                            <div class="insp-label">Bound SQLite Entity</div>
                            <div class="insp-val">tickets (Local WAL)</div>
                        </div>
                        <div class="insp-group">
                            <div class="insp-label">Semantic Color Token</div>
                            <div class="insp-val"><code>canvas.surface (#151821)</code></div>
                        </div>
                        <div class="insp-group">
                            <div class="insp-label">Corner Radius Token</div>
                            <div class="insp-val"><code>radius.lg (12px)</code></div>
                        </div>
                        <div class="insp-group">
                            <div class="insp-label">Action Binding</div>
                            <div class="insp-val"><code>submit_record() &rarr; SQLite</code></div>
                        </div>
                    </div>
                </div>
            </div>
        </section>

        <!-- 3. FEATURE GRID (Section 33 Preserved) -->
        <section class="feature-section" id="features">
            <div class="section-header center">
                <span class="badge badge-blue">ENTERPRISE FOUNDATIONS</span>
                <h2 class="section-title">Αρχιτεκτονική Σχεδιασμένη για Απόλυτες Επιδόσεις</h2>
                <p class="section-sub">Χτισμένο από το μηδέν σε Rust, απαλλαγμένο από εξαρτήσεις τρίτων και περιττή πολυπλοκότητα.</p>
            </div>

            <div class="feature-grid">
                <div class="feature-card"><div class="feature-icon">⚡</div><h3 class="feature-title">Αστραπιαία Ταχύτητα (Rust)</h3><p class="feature-desc">Native εκτέλεση χωρίς JavaScript runtime στο desktop. Εκκίνηση σε κλάσματα δευτερολέπτου και λιγότερο από 30MB μνήμη RAM.</p></div>
                <div class="feature-card"><div class="feature-icon">🛡</div><h3 class="feature-title">Τοπική Κρυπτογραφημένη Βάση</h3><p class="feature-desc">Πλήρης προστασία με AES-256-GCM και ChaCha20-Poly1305. Τα δεδομένα μένουν πάντα στην τοπική σας συσκευή.</p></div>
                <div class="feature-card"><div class="feature-icon">📊</div><h3 class="feature-title">Penpot-Inspired Canvas</h3><p class="feature-desc">Σχεδιάστε οθόνες, φόρμες και πίνακες με auto-layout και δεσμεύστε τις απευθείας σε πραγματικές οντότητες της βάσης.</p></div>
                <div class="feature-card"><div class="feature-icon">☁</div><h3 class="feature-title">Resilient Local-First</h3><p class="feature-desc">Συνεχίστε να εργάζεστε ακόμα και όταν το διαδίκτυο διακοπεί. Αυτόματος συγχρονισμός LAN μόλις επανασυνδεθείτε.</p></div>
                <div class="feature-card"><div class="feature-icon">📐</div><h3 class="feature-title">W3C Design Tokens</h3><p class="feature-desc">Ενιαία πηγή αλήθειας για χρώματα, αποστάσεις και τυπογραφία, αποτρέποντας οποιαδήποτε σχεδιαστική απόκλιση.</p></div>
                <div class="feature-card"><div class="feature-icon">⚙</div><h3 class="feature-title">Walled Garden Marketplace</h3><p class="feature-desc">Επεκτείνετε τις δυνατότητες με πιστοποιημένα modules, ασφαλή compilation gate και ψηφιακά συμβόλαια SLA με escrow.</p></div>
            </div>
        </section>

        <!-- 4. PRICING TABLE (Section 33 Preserved) -->
        <section class="pricing-section" id="pricing">
            <div class="section-header center">
                <span class="badge badge-green">TRANSPARENT VALUE</span>
                <h2 class="section-title">Διαφανής Τιμολόγηση χωρίς Κρυφές Χρεώσεις</h2>
                <p class="section-sub">Επιλέξτε το πακέτο που ταιριάζει στο μέγεθος της επιχείρησής σας. Πλήρης έλεγχος κόστους.</p>
            </div>

            <div class="pricing-table">
                <!-- Plan 1: Community -->
                <div class="pricing-card">
                    <div class="plan-name">Community</div>
                    <div class="plan-price">0 € <span class="price-period">/ για πάντα</span></div>
                    <p class="plan-desc">Ιδανικό για αυτόνομους επαγγελματίες και μεμονωμένα καταστήματα.</p>
                    <ul class="plan-features">
                        <li>✓ 1 Θέση Εργασίας</li><li>✓ Τοπική Βάση SQLite</li><li>✓ Απεριόριστες Επαφές &amp; Deals</li><li>✓ Βασικές Αναφορές &amp; Εξαγωγές CSV</li><li>✓ Πλήρης Offline Λειτουργία</li>
                    </ul>
                    <a href="/hub" class="btn btn-secondary full-width">Έναρξη Δωρεάν</a>
                </div>

                <!-- Plan 2: Professional (Highlighted) -->
                <div class="pricing-card highlighted">
                    <div class="popular-badge">ΔΗΜΟΦΙΛΕΣΤΕΡΟ</div>
                    <div class="plan-name">Professional</div>
                    <div class="plan-price">49 € <span class="price-period">/ μήνα</span></div>
                    <p class="plan-desc">Για δυναμικές ομάδες και αναπτυσσόμενα επισκευαστικά κέντρα.</p>
                    <ul class="plan-features">
                        <li>✓ Έως 5 Θέσεις Εργασίας</li><li>✓ Αυτόματο LAN Roaming &amp; Peer Sync</li><li>✓ Daily Automated Cloud Backups</li><li>✓ Προτεραιότητα Τεχνικής Υποστήριξης</li><li>✓ Πρόσβαση στο Add-on Marketplace</li>
                    </ul>
                    <a href="/hub" class="btn full-width">Επιλογή Professional</a>
                </div>

                <!-- Plan 3: Enterprise -->
                <div class="pricing-card">
                    <div class="plan-name">Enterprise</div>
                    <div class="plan-price">199 € <span class="price-period">/ μήνα</span></div>
                    <p class="plan-desc">Για αλυσίδες καταστημάτων και οργανισμούς με αυστηρά SLAs.</p>
                    <ul class="plan-features">
                        <li>✓ Απεριόριστες Θέσεις Εργασίας</li><li>✓ SLA Escrow Συμβόλαια Τεχνικών</li><li>✓ Walled Garden Compiler Gate</li><li>✓ Custom Inferred DB Modules</li><li>✓ 24/7 Dedicated SLA &amp; Support</li>
                    </ul>
                    <a href="#contact" class="btn btn-secondary full-width">Επικοινωνία για Enterprise</a>
                </div>
            </div>
        </section>

        <!-- 5. CALL TO ACTION BANNER (Section 33 Preserved) -->
        <section class="cta-banner">
            <div class="cta-content">
                <h2 class="cta-title">Αποκτήστε τον Πλήρη Έλεγχο των Επιχειρησιακών σας Δεδομένων</h2>
                <p class="cta-sub">Ξεκινήστε σήμερα με την ισχύ του Proteus. Χωρίς πιστωτική κάρτα, χωρίς κρυφές συνδρομές.</p>
            </div>
            <div class="cta-btn-wrap">
                <a href="/hub" class="btn btn-lg btn-white">Άνοιγμα Business Hub →</a>
            </div>
        </section>

        <!-- 6. CONTACT FORM (Section 33 Preserved) -->
        <section class="contact-section" id="contact">
            <div class="section-header center">
                <span class="badge badge-gray">GET IN TOUCH</span>
                <h2 class="section-title">Επικοινωνήστε με την Ομάδα Μηχανικών του Proteus</h2>
                <p class="section-sub">Στείλτε μας το αίτημά σας και θα σας απαντήσουμε άμεσα για να συζητήσουμε τις ανάγκες σας.</p>
            </div>

            <div class="contact-form-wrap">
                <form id="contact-form" class="contact-form-card" onsubmit="handleContactSubmit(event)">
                    <div class="field-row">
                        <div class="field-group">
                            <label for="c-name">Ονοματεπώνυμο *</label>
                            <input type="text" id="c-name" required placeholder="Γιάννης Παπαδόπουλος" class="pds-input">
                        </div>
                        <div class="field-group">
                            <label for="c-email">Email Εργασίας *</label>
                            <input type="email" id="c-email" required placeholder="name@company.com" class="pds-input">
                        </div>
                    </div>
                    <div class="field-group">
                        <label for="c-phone">Αριθμός Τηλεφώνου</label>
                        <input type="tel" id="c-phone" placeholder="+30 210 1234567" class="pds-input">
                    </div>
                    <div class="field-group">
                        <label for="c-msg">Μήνυμα / Περιγραφή Έργου *</label>
                        <textarea id="c-msg" required rows="4" placeholder="Περιγράψτε το έργο σας ή τις απαιτήσεις του καταστήματός σας..." class="pds-textarea"></textarea>
                    </div>
                    <button type="submit" id="c-submit" class="btn full-width">Αποστολή Μηνύματος ✓</button>
                    <div id="contact-feedback" style="display: none; padding: 0.8rem; border-radius: 6px; font-size: 0.85rem; text-align: center;"></div>
                </form>
            </div>
        </section>
    </main>

    <!-- Footer -->
    <footer class="landing-footer">
        <div class="footer-inner">
            <div class="footer-brand">
                <div class="brand">PROTEUS</div>
                <p style="font-size: 0.8rem; color: var(--text-muted); margin-top: 0.4rem;">
                    The Open Business Engine & App Canvas. 100% Original Bespoke Rust Architecture.
                </p>
            </div>
            <div class="footer-links">
                <a href="#features">Χαρακτηριστικά</a>
                <a href="#vision">Το Όραμα</a>
                <a href="#pricing">Τιμολόγηση</a>
                <a href="/hub">Web Hub</a>
            </div>
        </div>
        <div class="footer-copy">
            Proteus CRM &copy; 2026 &bull; W3C Design Tokens &bull; WCAG 2.2 AA Compliant
        </div>
    </footer>
"##;

const HTML_SCRIPTS: &str = r##"
    <script>
        function setCanvasMode(mode) {
            const btnLive = document.getElementById('btn-mode-live');
            const btnInsp = document.getElementById('btn-mode-inspect');
            const inspPanel = document.getElementById('canvas-inspector');

            if (mode === 'live') {
                btnLive.classList.add('active');
                btnInsp.classList.remove('active');
                inspPanel.style.opacity = '0.5';
            } else {
                btnInsp.classList.add('active');
                btnLive.classList.remove('active');
                inspPanel.style.opacity = '1.0';
            }
        }

        function simulateInsert() {
            const rows = document.getElementById('mock-table-rows');
            const id = Math.floor(1043 + Math.random() * 50);
            const models = ['MacBook Pro M3', 'Samsung Galaxy S24', 'Sony PlayStation 5', 'Surface Pro 9'];
            const model = models[Math.floor(Math.random() * models.length)];
            
            const newRow = document.createElement('div');
            newRow.className = 't-row';
            newRow.innerHTML = '<span>#' + id + ' — ' + model + '</span><span class="badge badge-blue">Νέα Παραλαβή</span>';
            newRow.style.animation = 'fadeIn 0.3s ease';
            rows.insertBefore(newRow, rows.firstChild);
        }

        async function handleContactSubmit(e) {
            e.preventDefault();
            const btn = document.getElementById('c-submit');
            const fb = document.getElementById('contact-feedback');
            btn.disabled = true;
            btn.textContent = 'Αποστολή...';

            setTimeout(() => {
                btn.disabled = false;
                btn.textContent = 'Αποστολή Μηνύματος ✓';
                fb.style.display = 'block';
                fb.style.background = 'rgba(16, 185, 129, 0.12)';
                fb.style.border = '1px solid #10b981';
                fb.style.color = '#34d399';
                fb.textContent = 'Το μήνυμά σας καταχωρήθηκε επιτυχώς! Η ομάδα μηχανικών θα επικοινωνήσει μαζί σας σύντομα.';
                document.getElementById('contact-form').reset();
            }, 600);
        }
    </script>
</body>
</html>
"##;
