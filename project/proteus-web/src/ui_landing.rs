//! Public Marketing & Company Landing Website for Proteus CRM & Business Engine.
//! Presents the product, how it works, workshops, hiring specialists, and custom quotes.

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
    out.push_str("\n    <script>\n");
    out.push_str(APP_JS);
    out.push_str("\n    </script>\n</body>\n</html>\n");
    out
}

const HTML_HEAD: &str = r##"<!DOCTYPE html>
<html lang="el">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Proteus — Native Business Application Engine & CRM</title>
    <meta name="description" content="Proteus: Native Rust & SQLite CRM for repairs, retail, and store management. Local-first resilience, zero cloud lock-in, workshops, and custom enterprise engineering.">
    <link rel="preconnect" href="https://fonts.googleapis.com">
    <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
    <link href="https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700;800&family=JetBrains+Mono:wght@400;500&display=swap" rel="stylesheet">
    <link rel="icon" type="image/x-icon" href="/favicon.ico">
    <style>
"##;

const HTML_BODY: &str = r##"
    </style>
</head>
<body class="sovereign-canvas">
    <!-- Ramp-Style Top Navigation Bar (Sovereign Nordic Slate Architecture) -->
    <header class="landing-header">
        <div class="brand-wrap">
            <a href="/" class="brand-link">
                <span class="brand-logo-sovereign">
                    proteus<svg class="brand-logo-symbol" viewBox="0 0 16 16" fill="currentColor" aria-hidden="true"><rect x="0.5" y="1" width="3.5" height="14" rx="0.5"/><rect x="0.5" y="1" width="15" height="3.5" rx="0.5"/><polygon points="11,4.5 15.5,15 11,15"/></svg>
                </span>
                <span class="brand-tag-sovereign">BOS</span>
            </a>
            <nav class="nav-links">
                <!-- Dropdown: Προϊόντα -->
                <div class="nav-item-dropdown">
                    <button class="nav-link-btn" type="button">
                        <span>Προϊόντα</span>
                        <span class="nav-caret">▼</span>
                    </button>
                    <div class="nav-dropdown-menu wide-menu">
                        <a href="/hub" class="dropdown-item">
                            <span class="dropdown-item-icon">📐</span>
                            <div class="dropdown-item-content">
                                <div class="dropdown-item-title">Design Studio</div>
                                <div class="dropdown-item-desc">Native visual canvas, scene graph &amp; flow DAG builder</div>
                            </div>
                        </a>
                        <a href="/hub" class="dropdown-item">
                            <span class="dropdown-item-icon">⚡</span>
                            <div class="dropdown-item-content">
                                <div class="dropdown-item-title">Frontline POS &amp; Service</div>
                                <div class="dropdown-item-desc">Intake tickets, 6-stage Kanban &amp; ESC/POS αποδείξεις</div>
                            </div>
                        </a>
                        <a href="/hub" class="dropdown-item">
                            <span class="dropdown-item-icon">📦</span>
                            <div class="dropdown-item-content">
                                <div class="dropdown-item-title">Spatial WMS &amp; VMI</div>
                                <div class="dropdown-item-desc">3D bin packing, consignment stock &amp; smart tote intake</div>
                            </div>
                        </a>
                        <a href="/hub" class="dropdown-item">
                            <span class="dropdown-item-icon">❄️</span>
                            <div class="dropdown-item-content">
                                <div class="dropdown-item-title">Cold Chain HACCP</div>
                                <div class="dropdown-item-desc">IoT sensor telemetry &amp; Merkle cryptographic audit</div>
                            </div>
                        </a>
                        <a href="/hub" class="dropdown-item">
                            <span class="dropdown-item-icon">🚚</span>
                            <div class="dropdown-item-content">
                                <div class="dropdown-item-title">Van Sales &amp; Signature</div>
                                <div class="dropdown-item-desc">Sign-on-glass vector capture &amp; mobile Bluetooth print</div>
                            </div>
                        </a>
                        <a href="/hub" class="dropdown-item">
                            <span class="dropdown-item-icon">🏛️</span>
                            <div class="dropdown-item-content">
                                <div class="dropdown-item-title">Direct AADE myDATA</div>
                                <div class="dropdown-item-desc">0€ REST sync, QR codes &amp; Ergani II ψηφιακή κάρτα</div>
                            </div>
                        </a>
                    </div>
                </div>

                <!-- Dropdown: Λύσεις -->
                <div class="nav-item-dropdown">
                    <button class="nav-link-btn" type="button">
                        <span>Λύσεις</span>
                        <span class="nav-caret">▼</span>
                    </button>
                    <div class="nav-dropdown-menu">
                        <a href="#about" class="dropdown-item">
                            <span class="dropdown-item-icon">🔧</span>
                            <div class="dropdown-item-content">
                                <div class="dropdown-item-title">Επισκευές &amp; Service</div>
                                <div class="dropdown-item-desc">IMEI serial tracking, εγγυήσεις &amp; RMA hub</div>
                            </div>
                        </a>
                        <a href="#about" class="dropdown-item">
                            <span class="dropdown-item-icon">🛒</span>
                            <div class="dropdown-item-content">
                                <div class="dropdown-item-title">Λιανική &amp; Ταμείο</div>
                                <div class="dropdown-item-desc">ESC/POS 80mm αποδείξεις, barcode scanner &amp; ESL</div>
                            </div>
                        </a>
                        <a href="#about" class="dropdown-item">
                            <span class="dropdown-item-icon">🏗️</span>
                            <div class="dropdown-item-content">
                                <div class="dropdown-item-title">Εργοτάξια &amp; Μάστορες</div>
                                <div class="dropdown-item-desc">Καρτέλα μάστορα, υλικά ανά έργο &amp; sub-ledger</div>
                            </div>
                        </a>
                        <a href="#about" class="dropdown-item">
                            <span class="dropdown-item-icon">🌐</span>
                            <div class="dropdown-item-content">
                                <div class="dropdown-item-title">Αλυσίδες LAN Mesh</div>
                                <div class="dropdown-item-desc">P2P replication μεταξύ καταστημάτων χωρίς cloud</div>
                            </div>
                        </a>
                    </div>
                </div>

                <!-- Dropdown: Συνεργάτες -->
                <div class="nav-item-dropdown">
                    <button class="nav-link-btn" type="button">
                        <span>Συνεργάτες</span>
                        <span class="nav-caret">▼</span>
                    </button>
                    <div class="nav-dropdown-menu">
                        <a href="#services" class="dropdown-item">
                            <span class="dropdown-item-icon">💼</span>
                            <div class="dropdown-item-content">
                                <div class="dropdown-item-title">Λογιστές &amp; Ορκωτοί</div>
                                <div class="dropdown-item-desc">Απευθείας εξαγωγή myDATA &amp; ελεγκτικά ίχνη Merkle</div>
                            </div>
                        </a>
                        <a href="#services" class="dropdown-item">
                            <span class="dropdown-item-icon">🖨️</span>
                            <div class="dropdown-item-content">
                                <div class="dropdown-item-title">Hardware Vendors</div>
                                <div class="dropdown-item-desc">Πιστοποιημένοι θερμικοί εκτυπωτές &amp; ESL ετικέτες</div>
                            </div>
                        </a>
                        <a href="#talent" class="dropdown-item">
                            <span class="dropdown-item-icon">🧑‍💻</span>
                            <div class="dropdown-item-content">
                                <div class="dropdown-item-title">Ειδικοί &amp; Developers</div>
                                <div class="dropdown-item-desc">Προσαρμοσμένες επεκτάσεις &amp; bespoke ενσωματώσεις</div>
                            </div>
                        </a>
                    </div>
                </div>

                <!-- Dropdown: Πόροι -->
                <div class="nav-item-dropdown">
                    <button class="nav-link-btn" type="button">
                        <span>Πόροι</span>
                        <span class="nav-caret">▼</span>
                    </button>
                    <div class="nav-dropdown-menu">
                        <a href="#how-it-works" class="dropdown-item">
                            <span class="dropdown-item-icon">📖</span>
                            <div class="dropdown-item-content">
                                <div class="dropdown-item-title">Πώς Λειτουργεί</div>
                                <div class="dropdown-item-desc">Αρχιτεκτονική τοπικής SQLite &amp; Rust engine</div>
                            </div>
                        </a>
                        <a href="/legal/terms" class="dropdown-item">
                            <span class="dropdown-item-icon">🛡️</span>
                            <div class="dropdown-item-content">
                                <div class="dropdown-item-title">Ασφάλεια &amp; Compliance</div>
                                <div class="dropdown-item-desc">GDPR, Ν.4624/2019, ISO 27001 &amp; μηδενικά cloud data</div>
                            </div>
                        </a>
                    </div>
                </div>

                <a href="#pricing" class="nav-link">Τιμολόγηση</a>
            </nav>
        </div>

        <div class="header-actions">
            <!-- Manual Theme Toggle Switch (Light / Dark) -->
            <button id="theme-toggle-btn" class="theme-toggle-btn" type="button" onclick="toggleTheme()" title="Εναλλαγή σε Light Mode">
                <span class="theme-icon">🌙</span>
                <span>Dark</span>
            </button>
            <a href="/hub" class="nav-link" style="font-weight: 600; padding: 0.4rem 0.6rem;">Σύνδεση</a>
            <a href="#contact" class="btn btn-secondary btn-sm">Ζητήστε Demo</a>
            <a href="/hub" class="btn btn-sm">Έναρξη →</a>
        </div>
    </header>

    <main class="landing-main">
        <!-- 1. HERO SECTION (Ramp-Inspired Asymmetrical Editorial Stage & Showcase) -->
        <section class="hero-section ramp-hero-layout">
            <div class="sovereign-status-ticker">
                <span class="ticker-dot"></span>
                <span>SOVEREIGN RUNTIME</span>
                <span>&bull;</span>
                <span class="ticker-val">100% OFFLINE</span>
                <span>&bull;</span>
                <span class="ticker-val">0€ SAAS TAX</span>
                <span>&bull;</span>
                <span>RUST NATIVE ENGINE</span>
            </div>

            <div class="hero-stage-grid">
                <!-- Left: Headline, Human Phrasing & Ramp-Style Action Bar -->
                <div class="hero-stage-left">
                    <div class="hero-pill">
                        <span class="pulse-dot"></span>
                        <span>SOVEREIGN BOS &bull; LOCAL-FIRST &bull; ZERO SUBSCRIPTIONS</span>
                    </div>
                    <h1 class="hero-title">
                        Αυτονομία χωρίς cloud.<br>
                        <span class="editorial-accent">Μηδέν συνδρομές.</span>
                    </h1>
                    <p class="hero-subtitle">
                        Επισκευές συσκευών, ταμείο POS, αποθήκη WMS και άμεση έκδοση myDATA.
                        Όλη η λειτουργία της επιχείρησής σας σε μία αστραπιαία native εφαρμογή,
                        με τοπική κρυπτογραφημένη SQLite και πλήρη ανεξαρτησία από τρίτους.
                    </p>

                    <!-- Ramp-Style Interactive Input & CTA Bar -->
                    <form class="ramp-hero-form" onsubmit="handleHeroSubmit(event)">
                        <div class="ramp-input-group">
                            <input type="text" id="hero-business-input" placeholder="Όνομα επιχείρησης ή email..." class="ramp-email-input" required>
                            <button type="submit" class="ramp-cta-btn">
                                <span>Δοκιμάστε το Demo</span>
                                <span class="btn-arrow">→</span>
                            </button>
                        </div>
                        <div class="ramp-form-subtext">
                            <span>⚡ Άμεση εκτέλεση σε Windows, macOS &amp; Linux</span>
                            <span class="subtext-bullet">&bull;</span>
                            <span>Κανένα δεδομένο δεν φεύγει από το κατάστημα</span>
                        </div>
                    </form>

                    <div class="hero-cta-group">
                        <a href="/hub" class="btn btn-lg">Έναρξη Δωρεάν &amp; Demo →</a>
                        <a href="#how-it-works" class="btn btn-secondary btn-lg">Πώς Λειτουργεί ↓</a>
                    </div>
                </div>

                <!-- Right: Ramp-Style Layered BOS Application Window & Live Receipts -->
                <div class="hero-interactive-stage">
                    <div class="ramp-app-card">
                        <!-- macOS App Window Chrome -->
                        <div class="app-window-header">
                            <div class="window-controls">
                                <span class="win-btn win-close"></span>
                                <span class="win-btn win-min"></span>
                                <span class="win-btn win-max"></span>
                            </div>
                            <div class="window-tab-title">
                                <span>TechFix Solutions Ο.Ε. — Frontline POS &amp; Service</span>
                            </div>
                            <div class="window-status-pill">● Localhost:3000</div>
                        </div>

                        <!-- Real Retail & Service Receipt Ledger -->
                        <div class="app-window-body">
                            <div class="tx-headline-row">
                                <div class="tx-vendor-badge">
                                    <div class="vendor-icon">TF</div>
                                    <div>
                                        <div class="vendor-name">TechFix Repair Hub</div>
                                        <div class="tx-timestamp">Σήμερα, 18:10 &bull; Απόδειξη #04821</div>
                                    </div>
                                </div>
                                <div class="tx-amount-col">
                                    <div class="tx-amount">€48,50</div>
                                    <div class="tx-badge-verified">✓ myDATA MARK 400008921938</div>
                                </div>
                            </div>

                            <div class="tx-receipt-details">
                                <div class="tx-line-item">
                                    <span class="item-name">Αλλαγή Οθόνης OLED iPhone 13 (IMEI 35892109281)</span>
                                    <span class="item-cost">€38,00</span>
                                </div>
                                <div class="tx-line-item">
                                    <span class="item-name">Εργασία Τεχνικού &amp; Διαγνωστικός Έλεγχος</span>
                                    <span class="item-cost">€10,50</span>
                                </div>
                                <div class="tx-footer-meta">
                                    <span>Πληρωμή: EFT/POS Terminal #02 (Α.1155)</span>
                                    <span>Εκτύπωση: ESC/POS 80mm (0.18ms)</span>
                                </div>
                            </div>
                        </div>
                    </div>

                    <!-- Overlapping Floating Mobile Van Ticket -->
                    <div class="ramp-floating-card floating-mobile-card">
                        <div class="mobile-card-header">
                            <span class="card-kicker">ΕΝΤΟΛΗ SERVICE #891</span>
                            <span class="status-ready">ΕΤΟΙΜΟ</span>
                        </div>
                        <div class="mobile-card-body">
                            <div class="mobile-item-title">Επιτόπιο Service — Van 01</div>
                            <div class="mobile-signature-line">
                                <svg viewBox="0 0 100 24" class="sig-svg">
                                    <path d="M5 18 Q 25 2, 45 14 T 85 8 T 95 18" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
                                </svg>
                                <span class="sig-label">Υπογραφή Πελάτη ✓</span>
                            </div>
                        </div>
                    </div>

                    <!-- Overlapping Floating Customer Quote (Tony Hawk Callout Style) -->
                    <div class="ramp-floating-card floating-callout-card">
                        <div class="callout-quote">
                            "Όταν πέφτει το internet στη γειτονιά, το ταμείο μας συνεχίζει να τυπώνει αποδείξεις και να καταγράφει επισκευές κανονικά."
                        </div>
                        <div class="callout-author">
                            <span class="author-avatar">ΓΚ</span>
                            <div>
                                <div class="author-name">Γιώργος Καραλής</div>
                                <div class="author-role">Ιδιοκτήτης TechFix Hub</div>
                            </div>
                        </div>
                    </div>
                </div>
            </div>

            <!-- Ramp-Style Bottom Telemetry Ticker Marquee -->
            <div class="ramp-telemetry-strip">
                <div class="telemetry-strip-inner">
                    <span class="strip-item"><span class="strip-label">⚙ ΚΥΡΙΑΡΧΟ RUNTIME:</span> <span class="strip-val">100% OFFLINE</span></span>
                    <span class="strip-divider">|</span>
                    <span class="strip-item"><span class="strip-label">ΣΥΝΑΛΛΑΓΕΣ:</span> <span class="strip-val">1,284,910</span></span>
                    <span class="strip-divider">|</span>
                    <span class="strip-item"><span class="strip-label">ΑΑΔΕ myDATA:</span> <span class="strip-val">0€ CLOUD TAX</span></span>
                    <span class="strip-divider">|</span>
                    <span class="strip-item"><span class="strip-label">ΧΡΟΝΟΣ ΑΠΟΚΡΙΣΗΣ:</span> <span class="strip-val">&lt; 0.2ms</span></span>
                    <span class="strip-divider">|</span>
                    <span class="strip-item"><span class="strip-label">ΕΚΤΥΠΩΤΕΣ ESC/POS:</span> <span class="strip-val">ΕΝΕΡΓΟΙ</span></span>
                    <span class="strip-divider">|</span>
                    <span class="strip-item"><span class="strip-label">ΒΑΣΗ ΔΕΔΟΜΕΝΩΝ:</span> <span class="strip-val">SQLITE AES-256</span></span>
                </div>
            </div>

            <div class="hero-specs-strip">
                <div class="spec-item"><span class="spec-metric">⚡ &lt;1s</span> Boot Time</div>
                <div class="spec-item"><span class="spec-metric">🛡️ AES-256</span> Local DB</div>
                <div class="spec-item"><span class="spec-metric">📶 100%</span> Offline Resilience</div>
                <div class="spec-item"><span class="spec-metric">🖨️ ESC/POS</span> Native Drivers</div>
            </div>
        </section>

        <!-- 2. ABOUT & CAPABILITIES -->
        <section class="feature-section" id="about">
            <div class="section-header center">
                <span class="badge badge-purple">THE ALL-IN-ONE BUSINESS PLATFORM</span>
                <h2 class="section-title">Σχεδιασμένο για την Πραγματική Επιχείρηση</h2>
                <p class="section-sub">Όλα τα κρίσιμα εργαλεία λειτουργίας ενσωματωμένα σε μία ενιαία αρχιτεκτονική χωρίς συνδρομές cloud.</p>
            </div>

            <div class="feature-grid">
                <div class="feature-card">
                    <div class="feature-icon">🔧</div>
                    <h3 class="feature-title">Επισκευές & Τεχνικό Τμήμα</h3>
                    <p class="feature-desc">Παραλαβές συσκευών, παρακολούθηση σταδίων επισκευής, κοστολόγηση ανταλλακτικών και αυτόματη ενημέρωση πελατών.</p>
                </div>
                <div class="feature-card">
                    <div class="feature-icon">💳</div>
                    <h3 class="feature-title">Σημείο Πώλησης & Ταμείο (POS)</h3>
                    <p class="feature-desc">Έκδοση παραστατικών, εκτυπώσεις αποδείξεων ESC/POS και διασύνδεση με συρτάρι ταμείου με άμεση απόκριση.</p>
                </div>
                <div class="feature-card">
                    <div class="feature-icon">🎨</div>
                    <h3 class="feature-title">Penpot-Inspired Native Designer</h3>
                    <p class="feature-desc">Το περιβάλλον σχεδιασμού της desktop εφαρμογής αντλεί έμπνευση από το Penpot, προσφέροντας καθαρή και ταχύτατη διάταξη φορμών.</p>
                </div>
                <div class="feature-card">
                    <div class="feature-icon">🛡</div>
                    <h3 class="feature-title">Τοπική SQLite & Zero Cloud Lock-in</h3>
                    <p class="feature-desc">Πλήρης ανεξαρτησία από το internet. Όλα τα δεδομένα αποθηκεύονται τοπικά με ισχυρή κρυπτογράφηση και συγχρονίζονται μέσω LAN.</p>
                </div>
            </div>
        </section>

        <!-- 3. HOW IT WORKS / WORKFLOW -->
        <section class="workflow-section" id="how-it-works">
            <div class="section-header center">
                <span class="badge badge-blue">SIMPLE & DETERMINISTIC</span>
                <h2 class="section-title">Πώς Λειτουργεί στην Πράξη</h2>
                <p class="section-sub">Τρία απλά βήματα από την πρώτη εγκατάσταση μέχρι την καθημερινή παραγωγική χρήση.</p>
            </div>

            <div class="workflow-grid">
                <div class="workflow-card">
                    <div class="step-badge">01</div>
                    <h3 class="workflow-title">Εγκατάσταση Desktop Εφαρμογής</h3>
                    <p class="workflow-desc">Κατεβάζετε το αυτόνομο native εκτελέσιμο για Windows ή Linux. Άμεση εκκίνηση σε λιγότερο από ένα δευτερόλεπτο χωρίς βαριά JavaScript runtimes.</p>
                </div>
                <div class="workflow-card">
                    <div class="step-badge">02</div>
                    <h3 class="workflow-title">Εξατομικευμένο Brief & Ροές Καταστήματος</h3>
                    <p class="workflow-desc">Περιγράφετε αναλυτικά τη λειτουργία της επιχείρησής σας, και πιστοποιημένοι designers (PCD-App / Web) διαμορφώνουν τις εξατομικευμένες οθόνες και DDL.</p>
                </div>
                <div class="workflow-card">
                    <div class="step-badge">03</div>
                    <h3 class="workflow-title">Απρόσκοπτη Καθημερινή Λειτουργία</h3>
                    <p class="workflow-desc">Εκτελείτε πωλήσεις, παραλαβές και αναφορές με μηδενικό latency, ακόμα και σε πλήρη διακοπή διαδικτύου, με ασφαλή τοπικά αντίγραφα.</p>
                </div>
            </div>
        </section>

        <!-- 4. SERVICES: WORKSHOPS & SPECIALISTS -->
        <section class="services-section" id="services">
            <div class="section-header center">
                <span class="badge badge-green">WORKSHOPS & SPECIALISTS</span>
                <h2 class="section-title">Επαγγελματικές Υπηρεσίες & Συνεργασία</h2>
                <p class="section-sub">Εκπαιδεύστε την ομάδα σας, προσλάβετε πιστοποιημένους μηχανικούς ή ζητήστε custom υλοποίηση για τον οργανισμό σας.</p>
            </div>

            <div class="services-grid">
                <!-- Service 1: Workshops -->
                <div class="service-card">
                    <div class="service-header">
                        <span class="badge badge-blue">ONBOARDING & TRAINING</span>
                    </div>
                    <h3 class="service-title">Εκπαιδευτικά Workshops</h3>
                    <p class="service-desc">Εντατικά πρακτικά σεμινάρια για ιδιοκτήτες, τεχνικούς και διαχειριστές καταστημάτων.</p>
                    <ul class="service-list">
                        <li>✓ Εκπαίδευση προσωπικού στις καθημερινές ροές</li>
                        <li>✓ Διαμόρφωση φορμών & custom πεδίων</li>
                        <li>✓ Βέλτιστες πρακτικές ασφάλειας και τοπικών backups</li>
                    </ul>
                    <button class="btn btn-secondary full-width" onclick="selectService('workshop')">Κράτηση Workshop →</button>
                </div>

                <!-- Service 2: Hire Specialists -->
                <div class="service-card featured" id="talent">
                    <div class="service-header">
                        <span class="badge badge-purple">DEDICATED TALENT</span>
                    </div>
                    <h3 class="service-title">Πρόσληψη Εξειδικευμένων Μηχανικών</h3>
                    <p class="service-desc">Συνεργαστείτε απευθείας με πιστοποιημένους μηχανικούς λογισμικού του οικοσυστήματος Proteus.</p>
                    <ul class="service-list">
                        <li>✓ Ανάπτυξη εξατομικευμένων modules & API</li>
                        <li>✓ Ασφαλής μετάπτωση δεδομένων από παλιά ERP</li>
                        <li>✓ On-demand τεχνική επίβλεψη και συμβουλευτική</li>
                    </ul>
                    <button class="btn full-width" onclick="selectService('talent')">Πρόσληψη Specialist →</button>
                </div>

                <!-- Service 3: Enterprise Quotes -->
                <div class="service-card">
                    <div class="service-header">
                        <span class="badge badge-amber">ENTERPRISE SCALE</span>
                    </div>
                    <h3 class="service-title">Custom Enterprise & SLAs</h3>
                    <p class="service-desc">Ολοκληρωμένες λύσεις εγκατάστασης για αλυσίδες καταστημάτων και franchises.</p>
                    <ul class="service-list">
                        <li>✓ Multi-store αρχιτεκτονική & LAN mesh</li>
                        <li>✓ 24/7 dedicated υποστήριξη & συμβόλαια SLA</li>
                        <li>✓ Επαλήθευση κώδικα με compiler verification</li>
                    </ul>
                    <button class="btn btn-secondary full-width" onclick="selectService('quote')">Ζητήστε Custom Προσφορά →</button>
                </div>
            </div>
        </section>

        <!-- 5. PRICING TABLE -->
        <section class="pricing-section" id="pricing">
            <div class="section-header center">
                <span class="badge badge-gray">TRANSPARENT VALUE</span>
                <h2 class="section-title">Διαφανή Πακέτα Συνεργασίας</h2>
                <p class="section-sub">Επιλέξτε το πακέτο που ανταποκρίνεται στις ανάγκες της επιχείρησής σας.</p>
            </div>

            <div class="pricing-table">
                <div class="pricing-card">
                    <div class="plan-name">Community</div>
                    <div class="plan-price">0 € <span class="price-period">/ για πάντα</span></div>
                    <p class="plan-desc">Ιδανικό για αυτόνομους επαγγελματίες και μεμονωμένα καταστήματα.</p>
                    <ul class="plan-features">
                        <li>✓ 1 Θέση Εργασίας</li><li>✓ Τοπική Βάση SQLite</li><li>✓ Απεριόριστες Επαφές &amp; Επισκευές</li><li>✓ Πλήρης Offline Λειτουργία</li>
                    </ul>
                    <a href="/hub" class="btn btn-secondary full-width">Έναρξη Δωρεάν</a>
                </div>

                <div class="pricing-card highlighted">
                    <div class="popular-badge">ΔΗΜΟΦΙΛΕΣΤΕΡΟ</div>
                    <div class="plan-name">Professional</div>
                    <div class="plan-price">49 € <span class="price-period">/ μήνα</span></div>
                    <p class="plan-desc">Για δυναμικές ομάδες και αναπτυσσόμενα επισκευαστικά κέντρα.</p>
                    <ul class="plan-features">
                        <li>✓ Έως 5 Θέσεις Εργασίας</li><li>✓ Αυτόματο LAN Roaming &amp; Peer Sync</li><li>✓ Daily Automated Backups</li><li>✓ Προτεραιότητα Υποστήριξης</li>
                    </ul>
                    <button class="btn full-width" onclick="selectService('quote')">Επιλογή Professional</button>
                </div>

                <div class="pricing-card">
                    <div class="plan-name">Enterprise Custom</div>
                    <div class="plan-price">Custom <span class="price-period">/ προσφορά</span></div>
                    <p class="plan-desc">Για αλυσίδες καταστημάτων, δίκτυα franchise και οργανισμούς.</p>
                    <ul class="plan-features">
                        <li>✓ Απεριόριστες Θέσεις &amp; Καταστήματα</li><li>✓ Συμπερίληψη Εκπαιδευτικού Workshop</li><li>✓ Δυνατότητα Dedicated Specialist</li><li>✓ 24/7 Dedicated SLA &amp; Support</li>
                    </ul>
                    <button class="btn btn-secondary full-width" onclick="selectService('quote')">Ζητήστε Προσφορά</button>
                </div>
            </div>
        </section>

        <!-- 6. CALL TO ACTION BANNER -->
        <section class="cta-banner">
            <div class="cta-content">
                <h2 class="cta-title">Αναβαθμίστε την Επιχείρησή σας με την Ισχύ του Proteus</h2>
                <p class="cta-sub">Μιλήστε άμεσα με την ομάδα μηχανικών μας για κλείσιμο προσφοράς ή οργάνωση workshop.</p>
            </div>
            <div class="cta-btn-wrap">
                <button onclick="selectService('quote')" class="btn btn-lg btn-white">Ζητήστε Προσφορά &rarr;</button>
            </div>
        </section>

        <!-- 7. CONTACT & QUOTE FORM -->
        <section class="contact-section" id="contact">
            <div class="section-header center">
                <span class="badge badge-blue">GET IN TOUCH</span>
                <h2 class="section-title">Κλείσιμο Προσφοράς & Επικοινωνία</h2>
                <p class="section-sub">Συμπληρώστε τη φόρμα και η τεχνική μας ομάδα θα επικοινωνήσει μαζί σας εντός 24 ωρών.</p>
            </div>

            <div class="contact-form-wrap">
                <form id="contact-form" class="contact-form-card" onsubmit="handleContactSubmit(event)">
                    <div class="field-group">
                        <label for="c-service">Σκοπός Επικοινωνίας *</label>
                        <select id="c-service" class="pds-select" required>
                            <option value="quote">Κλείσιμο Προσφοράς (Custom Quote)</option>
                            <option value="workshop">Κράτηση Εκπαιδευτικού Workshop</option>
                            <option value="talent">Πρόσληψη Εξειδικευμένου Μηχανικού (Specialist)</option>
                            <option value="demo">Ενημέρωση &amp; Live Demo</option>
                        </select>
                    </div>

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

                    <div class="field-row">
                        <div class="field-group">
                            <label for="c-company">Επωνυμία Εταιρείας / Καταστήματος</label>
                            <input type="text" id="c-company" placeholder="π.χ. TechRepair Hellas" class="pds-input">
                        </div>
                        <div class="field-group">
                            <label for="c-phone">Αριθμός Τηλεφώνου</label>
                            <input type="tel" id="c-phone" placeholder="+30 210 1234567" class="pds-input">
                        </div>
                    </div>

                    <div class="field-group">
                        <label for="c-msg">Μήνυμα / Περιγραφή Απαιτήσεων *</label>
                        <textarea id="c-msg" required rows="4" placeholder="Περιγράψτε το κατάστημά σας, τον αριθμό των θέσεων ή τις ανάγκες του workshop..." class="pds-textarea"></textarea>
                    </div>

                    <div class="field-group" style="margin-top: 0.5rem;">
                        <label style="display: flex; align-items: flex-start; gap: 0.6rem; font-size: 0.82rem; color: var(--text-muted); cursor: pointer;">
                            <input type="checkbox" id="c-consent" required style="accent-color: var(--accent); margin-top: 0.2rem; width: 16px; height: 16px;">
                            <span>Έχω διαβάσει και συναινώ με την <a href="/privacy" target="_blank" style="color: var(--text-primary); text-decoration: underline;">Πολιτική Απορρήτου</a> και τους <a href="/terms" target="_blank" style="color: var(--text-primary); text-decoration: underline;">Όρους Χρήσης</a> (GDPR & Ν. 4624/2019).</span>
                        </label>
                    </div>
                    <div style="font-size: 0.75rem; color: var(--text-faint); margin-top: -0.25rem;">
                        * Τα στοιχεία σας χρησιμοποιούνται αποκλειστικά για την απάντηση στο αίτημά σας. Κάθε μελλοντική επικοινωνία περιλαμβάνει άμεση δυνατότητα διαγραφής (1-Click Unsubscribe).
                    </div>

                    <button type="submit" id="c-submit" class="btn full-width" style="margin-top: 0.75rem;">Αποστολή Αιτήματος ✓</button>
                    <div id="contact-feedback" style="display: none; padding: 0.8rem; border-radius: 6px; font-size: 0.85rem; text-align: center;"></div>
                </form>
            </div>
        </section>
    </main>

    <!-- Footer -->
    <footer class="landing-footer">
        <div class="footer-inner">
            <div class="footer-brand">
                <a href="/" class="brand-link">
                    <span class="brand-logo-sovereign">
                        proteus<svg class="brand-logo-symbol" viewBox="0 0 16 16" fill="currentColor" aria-hidden="true"><rect x="0.5" y="1" width="3.5" height="14" rx="0.5"/><rect x="0.5" y="1" width="15" height="3.5" rx="0.5"/><polygon points="11,4.5 15.5,15 11,15"/></svg>
                    </span>
                    <span class="brand-tag-sovereign">BOS</span>
                </a>
                <p style="font-size: 0.8rem; color: var(--text-muted); margin-top: 0.4rem; line-height: 1.5;">
                    <strong>Proteus Sovereign Systems I.K.E.</strong><br>
                    Αρ. Γ.Ε.ΜΗ.: 169824501000 • Α.Φ.Μ.: 802194512 (Δ.Ο.Υ. Α' Αθηνών)<br>
                    Πανεπιστημίου 42, 10679, Αθήνα, Ελλάδα • Τηλ: +30 210 300 4500<br>
                    Email: compliance@proteus-bos.internal • DPO: dpo@proteus-bos.internal
                </p>
            </div>
            <div class="footer-links">
                <a href="/privacy">Πολιτική Απορρήτου</a>
                <a href="/terms">Όροι Χρήσης</a>
                <a href="/refund">Πολιτική Επιστροφών</a>
                <a href="/cookies">Cookies</a>
                <a href="/data-deletion">Αίτημα Διαγραφής (GDPR)</a>
                <a href="/hub">Web Hub</a>
            </div>
        </div>
        <div class="footer-copy" style="line-height: 1.6;">
            Proteus Sovereign Systems &copy; 2026 &bull; 18+ B2B Architecture (COPPA & GDPR-K Compliant) &bull; WCAG 2.2 AA &bull; W3C Design Tokens &bull; Fonts: SIL Open Font License 1.1 &bull; 100% Bespoke Codebase
        </div>
    </footer>

    <!-- Interactive Cookie Consent Banner -->
    <div id="cookie-banner" style="display: none; position: fixed; bottom: 1.25rem; left: 50%; transform: translateX(-50%); width: calc(100% - 2.5rem); max-width: 860px; background: var(--bg-surface); border: 1px solid var(--border-strong); border-radius: 10px; padding: 1.25rem; z-index: 9999; box-shadow: 0 12px 36px rgba(0,0,0,0.45);">
        <div style="display: flex; justify-content: space-between; align-items: flex-start; flex-wrap: wrap; gap: 1rem;">
            <div style="flex: 1; min-width: 280px;">
                <div style="font-weight: 700; color: var(--text-primary); font-size: 0.95rem; margin-bottom: 0.35rem;">🍪 Σεβασμός στην Ιδιωτικότητα & Cookies (ePrivacy & GDPR)</div>
                <p style="font-size: 0.83rem; color: var(--text-muted); line-height: 1.5; margin: 0;">
                    Χρησιμοποιούμε μόνο απολύτως αναγκαία τεχνικά cookies για την ασφάλεια συνεδρίας και προτιμήσεις εμφάνισης. Δεν χρησιμοποιούμε διαφημιστικά cookies παρακολούθησης. Διαβάστε την <a href="/cookies" target="_blank" style="color: var(--text-primary); text-decoration: underline;">Πολιτική Cookies</a>.
                </p>
            </div>
            <div style="display: flex; gap: 0.5rem; align-items: center; flex-wrap: wrap;">
                <button onclick="acceptAllCookies()" class="btn" style="padding: 0.5rem 1rem; font-size: 0.82rem; background: var(--cta-primary-fill); color: var(--cta-primary-text); border: none; border-radius: 4px; font-weight: 600; cursor: pointer;">Αποδοχή Όλων</button>
                <button onclick="rejectNonEssentialCookies()" class="btn" style="padding: 0.5rem 1rem; font-size: 0.82rem; background: var(--bg-surface-sec); color: var(--text-primary); border: 1px solid var(--border); border-radius: 4px; font-weight: 600; cursor: pointer;">Μόνο Απαραίτητα</button>
                <a href="/cookies" style="padding: 0.5rem 0.75rem; font-size: 0.82rem; color: var(--text-muted); text-decoration: none;">Ρυθμίσεις</a>
            </div>
        </div>
    </div>
"##;

const APP_JS: &str = include_str!("../ui_design/app.js");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hero_cognitive_anchor_and_structure() {
        let html = build_landing_html();
        assert!(html.contains("<html lang=\"el\">"));
        assert!(html.contains("<title>Proteus — Native Business Application Engine & CRM</title>"));
        assert!(html.contains("hero-section"));
        assert!(html.contains("hero-title"));
        assert!(html.contains("Αυτονομία χωρίς cloud"));
        assert!(html.contains("Έναρξη Δωρεάν &amp; Demo →"));
        assert!(html.contains("Πώς Λειτουργεί ↓"));
        assert!(html.contains("hero-specs-strip"));
        assert!(html.contains("spec-metric"));
        assert!(html.contains("100%"));
    }

    #[test]
    fn test_landing_ramp_style_navbar_and_theme_toggle() {
        let html = build_landing_html();
        assert!(html.contains("landing-header"));
        assert!(html.contains("brand-logo-sovereign"));
        assert!(html.contains("nav-item-dropdown"));
        assert!(html.contains("nav-dropdown-menu"));
        assert!(html.contains("theme-toggle-btn"));
        assert!(html.contains("sovereign-status-ticker"));
        assert!(html.contains("toggleTheme()"));
    }
}
