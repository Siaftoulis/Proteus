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
    out.push_str(HTML_SCRIPTS);
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
<body>
    <!-- Top Navigation Bar -->
    <header class="landing-header">
        <div class="brand-wrap">
            <a href="/" style="text-decoration: none; color: inherit; display: flex; align-items: center; gap: 0.75rem;">
                <img src="/assets/logo.png" alt="Proteus Emblem" class="brand-logo-img" />
                <div class="brand">
                    <span>PROTEUS</span>
                    <span class="brand-badge">BUSINESS ENGINE</span>
                </div>
            </a>
            <nav class="nav-links">
                <a href="#about" class="nav-link">Σχετικά</a>
                <a href="#how-it-works" class="nav-link">Πώς Λειτουργεί</a>
                <a href="#services" class="nav-link">Workshops</a>
                <a href="#talent" class="nav-link">Πρόσληψη Ειδικών</a>
                <a href="#pricing" class="nav-link">Τιμολόγηση</a>
                <a href="#contact" class="nav-link">Επικοινωνία</a>
            </nav>
        </div>
        <div class="header-actions">
            <a href="/hub" class="btn btn-secondary btn-sm">Web Hub</a>
            <a href="#contact" class="btn btn-sm">Ζητήστε Προσφορά →</a>
        </div>
    </header>

    <main class="landing-main">
        <!-- 1. HERO SECTION -->
        <section class="hero-section">
            <div class="hero-emblem-wrap">
                <img src="/assets/logo.png" alt="King Proteus Sovereign Emblem" class="hero-emblem-img" />
            </div>
            <div class="hero-pill">
                <span class="pulse-dot"></span>
                <span>NATIVE RUST &bull; LOCAL SQLITE &bull; 0% CLOUD LOCK-IN</span>
            </div>
            <h1 class="hero-title">
                Το Σύγχρονο Λειτουργικό Σύστημα για <span class="gradient-text">Επιχειρήσεις & Retail</span>
            </h1>
            <p class="hero-subtitle">
                Το Proteus αντικαθιστά τα αργά και δυσκίνητα cloud CRM με μία αστραπιαία native εφαρμογή σε Rust.
                Απόλυτη τοπική ασφάλεια SQLite, 100% offline λειτουργία, διαχείριση επισκευών και σημείο πώλησης (POS).
            </p>
            <div class="hero-cta-group">
                <a href="#contact" onclick="selectService('quote')" class="btn btn-lg">Κλείσιμο Προσφοράς & Demo →</a>
                <a href="#services" class="btn btn-secondary btn-lg">Εκπαιδευτικά Workshops</a>
            </div>
            <div class="hero-specs-strip">
                <div class="spec-item"><span>🚀 0ms</span> Local Latency</div>
                <div class="spec-item"><span>🛡 AES-256</span> Encrypted SQLite</div>
                <div class="spec-item"><span>📐 W3C</span> Design Tokens</div>
                <div class="spec-item"><span>⚡ 100%</span> Offline Resilience</div>
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
                <div style="display: flex; align-items: center; gap: 0.6rem;">
                    <img src="/assets/logo.png" alt="Proteus Sovereign Logo" style="width: 28px; height: 28px; object-fit: contain; border-radius: 4px;" />
                    <div class="brand">PROTEUS</div>
                </div>
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

const HTML_SCRIPTS: &str = r##"
    <script>
        window.addEventListener('DOMContentLoaded', () => {
            if (!localStorage.getItem('proteus_cookie_consent')) {
                const banner = document.getElementById('cookie-banner');
                if (banner) {
                    banner.style.display = 'block';
                }
            }
        });

        function acceptAllCookies() {
            localStorage.setItem('proteus_cookie_consent', JSON.stringify({ choice: 'all', timestamp: new Date().toISOString() }));
            const banner = document.getElementById('cookie-banner');
            if (banner) banner.style.display = 'none';
        }

        function rejectNonEssentialCookies() {
            localStorage.setItem('proteus_cookie_consent', JSON.stringify({ choice: 'necessary_only', timestamp: new Date().toISOString() }));
            const banner = document.getElementById('cookie-banner');
            if (banner) banner.style.display = 'none';
        }

        function selectService(type) {
            const select = document.getElementById('c-service');
            if (select) {
                select.value = type;
            }
            const contactSection = document.getElementById('contact');
            if (contactSection) {
                contactSection.scrollIntoView({ behavior: 'smooth' });
            }
        }

        async function handleContactSubmit(e) {
            e.preventDefault();
            const consentCheckbox = document.getElementById('c-consent');
            if (consentCheckbox && !consentCheckbox.checked) {
                alert('Παρακαλώ αποδεχθείτε τους Όρους Χρήσης και την Πολιτική Απορρήτου πριν την αποστολή.');
                return;
            }

            const btn = document.getElementById('c-submit');
            const fb = document.getElementById('contact-feedback');
            btn.disabled = true;
            btn.textContent = 'Αποστολή...';

            const payload = {
                service_type: document.getElementById('c-service').value,
                name: document.getElementById('c-name').value,
                email: document.getElementById('c-email').value,
                company: document.getElementById('c-company').value || null,
                phone: document.getElementById('c-phone').value || null,
                message: document.getElementById('c-msg').value
            };

            try {
                const res = await fetch('/api/v1/contact', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify(payload)
                });
                const data = await res.json();
                btn.disabled = false;
                btn.textContent = 'Αποστολή Αιτήματος ✓';
                fb.style.display = 'block';
                fb.style.background = 'rgba(16, 185, 129, 0.12)';
                fb.style.border = '1px solid #10b981';
                fb.style.color = '#34d399';
                fb.textContent = data.message || 'Το αίτημά σας καταχωρήθηκε επιτυχώς! Θα επικοινωνήσουμε μαζί σας σύντομα.';
                document.getElementById('contact-form').reset();
            } catch (err) {
                btn.disabled = false;
                btn.textContent = 'Αποστολή Αιτήματος ✓';
                fb.style.display = 'block';
                fb.style.background = 'rgba(239, 68, 68, 0.12)';
                fb.style.border = '1px solid #ef4444';
                fb.style.color = '#f87171';
                fb.textContent = 'Παρουσιάστηκε σφάλμα κατά την αποστολή. Παρακαλούμε δοκιμάστε ξανά.';
            }
        }
    </script>
</body>
</html>
"##;
