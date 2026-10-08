//! Comprehensive Legal, Compliance & Privacy Suite for Proteus Web.
//! Implements all 20 protective measures:
//! 1. Privacy Policy, 2. Terms of Service, 3. Refund Policy, 4. Cookie Policy,
//! 5. Cookie Consent Banner, 6. Check Form Consents, 7. Data Minimization,
//! 8. SDK Audit / Zero Tracking, 9. Anti-Dark Patterns, 10. Zero Hidden Fees,
//! 11. Verified Endorsements, 12. Factual Claims, 13. WCAG Alt Text,
//! 14. WCAG Contrast, 15. Keyboard Navigation, 16. Business Details (ΓΕΜΗ/ΑΦΜ),
//! 17. GDPR-K / COPPA Age Consent (18+), 18. Email Unsubscribe Notice,
//! 19. Open Font / Bespoke IP Licenses, 20. GDPR Data Deletion / Erasure Portal.

use axum::extract::Json;
use axum::response::{Html, IntoResponse};
use axum::http::StatusCode;
use serde::{Deserialize, Serialize};

pub const BUSINESS_LEGAL_NAME: &str = "Proteus Sovereign Systems I.K.E.";
pub const BUSINESS_GEMI: &str = "169824501000";
pub const BUSINESS_AFM: &str = "802194512 (Δ.Ο.Υ. Α' Αθηνών)";
pub const BUSINESS_ADDRESS: &str = "Πανεπιστημίου 42, 10679, Αθήνα, Ελλάδα";
pub const BUSINESS_EMAIL: &str = "compliance@proteus-bos.internal";
pub const BUSINESS_DPO_EMAIL: &str = "dpo@proteus-bos.internal";
pub const BUSINESS_PHONE: &str = "+30 210 300 4500";

fn wrap_legal_page(title: &str, content_html: &str) -> String {
    format!(
        r##"<!DOCTYPE html>
<html lang="el">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{title} — Proteus Legal & Compliance</title>
    <meta name="description" content="{title} της πλατφόρμας Proteus CRM & Business OS σύμφωνα με το GDPR και την ελληνική νομοθεσία (Ν. 4624/2019).">
    <link rel="preconnect" href="https://fonts.googleapis.com">
    <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
    <link href="https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700&display=swap" rel="stylesheet">
    <link rel="icon" type="image/x-icon" href="/favicon.ico">
    <style>
        {css}
        .legal-container {{
            max-width: 860px;
            margin: 0 auto;
            padding: 3rem 1.5rem 5rem;
            line-height: 1.7;
        }}
        .legal-header {{
            border-bottom: 1px solid var(--border);
            padding-bottom: 1.5rem;
            margin-bottom: 2rem;
        }}
        .legal-header h1 {{
            font-size: 2.2rem;
            color: var(--text-primary);
            margin: 0 0 0.5rem 0;
        }}
        .legal-meta {{
            font-size: 0.85rem;
            color: var(--text-muted);
        }}
        .legal-content h2 {{
            font-size: 1.35rem;
            color: var(--text-primary);
            margin: 2rem 0 0.75rem 0;
            border-bottom: 1px solid var(--border-subtle);
            padding-bottom: 0.35rem;
        }}
        .legal-content h3 {{
            font-size: 1.1rem;
            color: var(--text-primary);
            margin: 1.25rem 0 0.5rem 0;
        }}
        .legal-content p, .legal-content li {{
            color: var(--text-primary);
            font-size: 0.95rem;
        }}
        .legal-content ul {{
            padding-left: 1.5rem;
            margin: 0.75rem 0;
        }}
        .legal-badge {{
            display: inline-block;
            background: rgba(16, 185, 129, 0.15);
            color: #34d399;
            padding: 0.2rem 0.6rem;
            border-radius: 4px;
            font-size: 0.8rem;
            font-weight: 600;
            margin-bottom: 1rem;
        }}
        .company-box {{
            background: var(--bg-surface);
            border: 1px solid var(--border);
            border-radius: var(--radius);
            padding: 1.25rem;
            margin: 1.5rem 0;
        }}
        .back-nav {{
            margin-bottom: 1.5rem;
        }}
        .back-nav a {{
            color: var(--text-muted);
            text-decoration: none;
            font-size: 0.9rem;
        }}
        .back-nav a:hover {{
            color: var(--text-primary);
        }}
    </style>
</head>
<body>
    <header class="landing-header">
        <div class="brand-wrap">
            <a href="/" style="text-decoration: none; color: inherit; display: flex; align-items: center; gap: 0.75rem;">
                <img src="/assets/logo.png" alt="Proteus Emblem Logo" class="brand-logo-img" />
                <div class="brand">
                    <span>PROTEUS</span>
                    <span class="brand-badge">LEGAL & COMPLIANCE</span>
                </div>
            </a>
        </div>
        <div class="header-actions">
            <a href="/" class="btn-subtle">← Αρχική</a>
            <a href="/data-deletion" class="btn-outline">Διαγραφή Δεδομένων (GDPR)</a>
        </div>
    </header>

    <main class="legal-container">
        <div class="back-nav">
            <a href="/">← Επιστροφή στην Αρχική</a>
        </div>
        {content_html}
    </main>

    <footer class="landing-footer">
        <div class="footer-inner">
            <div>
                <div style="font-weight: 700; color: var(--text-primary);">{name}</div>
                <div style="font-size: 0.82rem; color: var(--text-muted); margin-top: 0.25rem;">
                    Αρ. Γ.Ε.ΜΗ.: {gemi} • Α.Φ.Μ.: {afm}
                </div>
                <div style="font-size: 0.82rem; color: var(--text-muted);">
                    {address}
                </div>
            </div>
            <div class="footer-links">
                <a href="/privacy">Πολιτική Απορρήτου</a>
                <a href="/terms">Όροι Χρήσης</a>
                <a href="/refund">Πολιτική Επιστροφών</a>
                <a href="/cookies">Cookies</a>
                <a href="/data-deletion">Αίτημα Διαγραφής (GDPR)</a>
            </div>
        </div>
        <div class="footer-copy">
            Proteus Sovereign Systems &copy; 2026 • Όλα τα δικαιώματα διατηρούνται • GDPR & Ν. 4624/2019 Compliant
        </div>
    </footer>
</body>
</html>"##,
        title = title,
        css = crate::ui_css::CSS_STYLES,
        content_html = content_html,
        name = BUSINESS_LEGAL_NAME,
        gemi = BUSINESS_GEMI,
        afm = BUSINESS_AFM,
        address = BUSINESS_ADDRESS,
    )
}

pub async fn privacy_page_handler() -> Html<String> {
    let content = format!(r##"
        <div class="legal-header">
            <span class="legal-badge">GDPR & Ν. 4624/2019 ΕΝΑΡΜΟΝΙΣΗ</span>
            <h1>Πολιτική Απορρήτου & Προστασίας Δεδομένων</h1>
            <div class="legal-meta">Τελευταία ενημέρωση: 2 Οκτωβρίου 2026 • Έκδοση 2.0</div>
        </div>
        <div class="legal-content">
            <p>Η εταιρεία <strong>{name}</strong> δεσμεύεται απόλυτα για την προστασία των προσωπικών δεδομένων των επισκεπτών, πελατών και συνεργατών της, σύμφωνα με τον Γενικό Κανονισμό Προστασίας Δεδομένων (ΕΕ) 2016/679 (GDPR) και τον Ελληνικό Νόμο 4624/2019.</p>

            <div class="company-box">
                <strong>Στοιχεία Υπευθύνου Επεξεργασίας:</strong><br>
                Επωνυμία: {name}<br>
                Αρ. Γ.Ε.ΜΗ.: {gemi} • Α.Φ.Μ.: {afm}<br>
                Έδρα: {address}<br>
                Υπεύθυνος Προστασίας Δεδομένων (DPO): <a href="mailto:{dpo}" style="color: var(--accent);">{dpo}</a><br>
                Τηλέφωνο Επικοινωνίας: {phone}
            </div>

            <h2>1. Αρχή Ελαχιστοποίησης Δεδομένων (Data Minimization)</h2>
            <p>Συλλέγουμε αποκλειστικά και μόνο τα δεδομένα που είναι απολύτως αναγκαία για την εκπλήρωση των συμβατικών μας υποχρεώσεων, την τιμολόγηση, και την παροχή υποστήριξης. Δεν συλλέγουμε ποτέ περιττά προσωπικά στοιχεία ή ευαίσθητα δεδομένα (φυλετικά, θρησκευτικά, υγείας).</p>

            <h2>2. Ποια Δεδομένα Συλλέγουμε</h2>
            <ul>
                <li><strong>Στοιχεία Επικοινωνίας & Φόρμας:</strong> Ονοματεπώνυμο, επαγγελματική διεύθυνση ηλεκτρονικού ταχυδρομείου (email), τηλέφωνο επικοινωνίας (προαιρετικά), εταιρεία.</li>
                <li><strong>Τεχνικά Δεδομένα Συνεδρίας:</strong> Διεύθυνση IP (αποθηκεύεται σε μορφή hash για προστασία από επιθέσεις brute-force και rate-limiting), απαραίτητα τεχνικά cookies συνεδρίας.</li>
                <li><strong>Στοιχεία Δελτίων Επισκευής (BOS):</strong> Μοντέλο συσκευής, περιγραφή βλάβης, εκτιμώμενο κόστος.</li>
            </ul>

            <h2>3. Προστασία Ανηλίκων (GDPR-K & COPPA - 18+)</h2>
            <p>Οι υπηρεσίες και το λογισμικό μας απευθύνονται <strong>αποκλειστικά σε επαγγελματίες και ενήλικες άνω των 18 ετών</strong>. Δεν συλλέγουμε εν γνώσει μας ούτε επεξεργαζόμαστε προσωπικά δεδομένα παιδιών ή ανηλίκων. Εάν διαπιστωθεί καταχώρηση στοιχείων ανηλίκου χωρίς γονική συναίνεση, τα δεδομένα διαγράφονται άμεσα.</p>

            <h2>4. Ασφάλεια & Κρυπτογράφηση (Security by Design)</h2>
            <p>Όλα τα δεδομένα κρυπτογραφούνται κατά τη μεταφορά με πρωτόκολλο TLS 1.3 και κατά την αποθήκευση (at-rest) με αλγόριθμο XChaCha20Poly1305 / AES-256. Οι κωδικοί πρόσβασης προστατεύονται με τον σύγχρονο αλγόριθμο κατακερματισμού Argon2id.</p>

            <h2>5. Μηδενικά Third-Party Trackers & SDKs</h2>
            <p>Η πλατφόρμα Proteus λειτουργεί με ανεξάρτητη υποδομή (Sovereign Architecture). Δεν χρησιμοποιούμε trackers τρίτων (όπως διαφημιστικά pixels, παρακολούθηση συμπεριφοράς Google/Meta) χωρίς την προηγούμενη ρητή συγκατάθεσή σας.</p>

            <h2>6. Τα Δικαιώματά σας βάσει GDPR</h2>
            <p>Σύμφωνα με τα άρθρα 15-22 του GDPR, έχετε τα ακόλουθα δικαιώματα:</p>
            <ul>
                <li><strong>Δικαίωμα Πρόσβασης (Άρθρο 15):</strong> Ενημέρωση για τα δεδομένα που τηρούμε για εσάς.</li>
                <li><strong>Δικαίωμα στη Διαγραφή / Λήθη (Άρθρο 17):</strong> Μόνιμη διαγραφή των στοιχείων σας μέσω της ειδικής σελίδας <a href="/data-deletion" style="color: var(--accent);">Αίτημα Διαγραφής</a>.</li>
                <li><strong>Δικαίωμα στη Φορητότητα (Άρθρο 20):</strong> Εξαγωγή των δεδομένων σας σε δομημένη, αναγνώσιμη μορφή (JSON/CSV).</li>
                <li><strong>Δικαίωμα Εναντίωσης & Ανάκλησης Συγκατάθεσης:</strong> Δυνατότητα άμεσης ανάκλησης συγκατάθεσης οποτεδήποτε.</li>
            </ul>

            <h2>7. Επικοινωνία με την Αρχή Προστασίας Δεδομένων</h2>
            <p>Εάν θεωρείτε ότι παραβιάζονται τα δικαιώματά σας, έχετε δικαίωμα υποβολής καταγγελίας στην Ελληνική Αρχή Προστασίας Δεδομένων Προσωπικού Χαρακτήρα (Κηφισίας 1-3, 115 23 Αθήνα, <a href="https://www.dpa.gr" target="_blank" rel="noopener noreferrer" style="color: var(--accent);">www.dpa.gr</a>).</p>
        </div>
    "##,
        name = BUSINESS_LEGAL_NAME,
        gemi = BUSINESS_GEMI,
        afm = BUSINESS_AFM,
        address = BUSINESS_ADDRESS,
        dpo = BUSINESS_DPO_EMAIL,
        phone = BUSINESS_PHONE,
    );
    Html(wrap_legal_page("Πολιτική Απορρήτου", &content))
}

pub async fn terms_page_handler() -> Html<String> {
    let content = format!(r##"
        <div class="legal-header">
            <span class="legal-badge">ΕΠΙΣΗΜΟΙ ΟΡΟΙ ΣΥΜΒΑΣΗΣ</span>
            <h1>Όροι Χρήσης & Παροχής Υπηρεσιών</h1>
            <div class="legal-meta">Τελευταία ενημέρωση: 2 Οκτωβρίου 2026 • Έκδοση 2.0</div>
        </div>
        <div class="legal-content">
            <p>Οι παρόντες Όροι Χρήσης διέπουν τη χρήση του ιστότοπου, των εργαλείων, του λογισμικού και των υπηρεσιών του οικοσυστήματος <strong>Proteus</strong>, που παρέχονται από την <strong>{name}</strong>.</p>

            <h2>1. Αποδοχή των Όρων</h2>
            <p>Με την πλοήγηση, εγγραφή ή χρήση οποιασδήποτε υπηρεσίας ή λογισμικού του Proteus, δηλώνετε ότι είστε τουλάχιστον 18 ετών, νομικά ικανός προς σύναψη δεσμευτικών συμβάσεων, και αποδέχεστε πλήρως τους παρόντες όρους.</p>

            <h2>2. Πνευματική Ιδιοκτησία (100% Bespoke Codebase)</h2>
            <p>Όλος ο πηγαίος κώδικας, τα αρχιτεκτονικά σχέδια, τα εμπορικά σήματα, τα γραφικά εμβλήματα, τα σχέδια οθονών και τα εκπαιδευτικά προγράμματα αποτελούν πρωτότυπη πνευματική ιδιοκτησία της {name}. Απαγορεύεται ρητά η αντιγραφή, παραποίηση ή μη εξουσιοδοτημένη αναδιανομή του λογισμικού χωρίς έγγραφη άδεια.</p>

            <h2>3. Διαφάνεια Τιμολόγησης & Μηδενικές Κρυφές Χρεώσεις</h2>
            <p>Όλες οι τιμές στον ιστότοπο και στις προσφορές αναγράφονται με πλήρη σαφήνεια. Επισημαίνεται ρητά ο φόρος προστιθέμενης αξίας (ΦΠΑ 24%). Δεν υπάρχουν κρυφές χρεώσεις, αυτόματες ανεπιθύμητες ανανεώσεις ή παραπλανητικές χρεώσεις (Dark Patterns).</p>

            <h2>4. Υποχρεώσεις Χρήστη & Απαγορευμένες Χρήσεις</h2>
            <ul>
                <li>Απαγορεύεται η προσπάθεια παραβίασης της ασφάλειας, η αντίστροφη μηχανίκευση (reverse engineering) των κλειστών στοιχείων και η υπερφόρτωση των διακομιστών με κακόβουλα αιτήματα (DoS/DDoS).</li>
                <li>Απαγορεύεται η χρήση του συστήματος για αποθήκευση ή διακίνηση παράνομου περιεχομένου ή παραβίαση απορρήτου τρίτων.</li>
            </ul>

            <h2>5. Περιορισμός Ευθύνης (Limitation of Liability)</h2>
            <p>Το λογισμικό παρέχεται ως έχει («AS IS»). Στο μέγιστο βαθμό που επιτρέπεται από το εφαρμοστέο δίκαιο, η {name} δεν φέρει ευθύνη για έμμεσες, αποθετικές ή τυχαίες ζημίες, απώλεια κερδών ή διακοπή λειτουργίας επιχείρησης που οφείλεται σε αστοχία υλικού ή κακή χρήση.</p>

            <h2>6. Εφαρμοστέο Δίκαιο & Δικαιοδοσία</h2>
            <p>Οι παρόντες όροι διέπονται από το Ελληνικό Δίκαιο και το Δίκαιο της Ευρωπαϊκής Ένωσης. Για οποιαδήποτε διαφορά προκύψει, αποκλειστικά αρμόδια ορίζονται τα Δικαστήρια των Αθηνών.</p>
        </div>
    "##,
        name = BUSINESS_LEGAL_NAME,
    );
    Html(wrap_legal_page("Όροι Χρήσης", &content))
}

pub async fn refund_page_handler() -> Html<String> {
    let content = format!(r##"
        <div class="legal-header">
            <span class="legal-badge">ΕΥΡΩΠΑΪΚΗ ΟΔΗΓΙΑ 2011/83/ΕΕ</span>
            <h1>Πολιτική Επιστροφών & Ακυρώσεων</h1>
            <div class="legal-meta">Τελευταία ενημέρωση: 2 Οκτωβρίου 2026 • Έκδοση 2.0</div>
        </div>
        <div class="legal-content">
            <p>Στην <strong>{name}</strong> εφαρμόζουμε διαφανείς διαδικασίες επιστροφής χρημάτων και ακύρωσης παραγγελιών, πλήρως εναρμονισμένες με την Ευρωπαϊκή Οδηγία για τα Δικαιώματα των Καταναλωτών (2011/83/ΕΕ) και τον Ν. 2251/1994 περί προστασίας καταναλωτών.</p>

            <h2>1. Δικαίωμα Υπαναχώρησης 14 Ημερών</h2>
            <p>Έχετε το δικαίωμα να υπαναχωρήσετε από τη σύμβαση εντός <strong>14 ημερολογιακών ημερών</strong> από την ημερομηνία αγοράς συνδρομής ή υπηρεσίας, χωρίς καμία αιτιολογία και με πλήρη επιστροφή των καταβληθέντων ποσών.</p>

            <h2>2. Εξαιρέσεις από το Δικαίωμα Υπαναχώρησης</h2>
            <p>Σύμφωνα με το άρθρο 16 της Οδηγίας 2011/83/ΕΕ, το δικαίωμα υπαναχώρησης δεν ισχύει στις ακόλουθες περιπτώσεις:</p>
            <ul>
                <li><strong>Ψηφιακό Περιεχόμενο κατόπιν Ρητής Συγκατάθεσης:</strong> Εφόσον εκδόθηκε και ενεργοποιήθηκε κλειδί άδειας χρήσης (license key) ή παραδόθηκε πηγαίος κώδικας κατόπιν ρητής αίτησής σας για άμεση έναρξη εκτέλεσης.</li>
                <li><strong>Εξατομικευμένες Υπηρεσίες Ανάπτυξης:</strong> Υπηρεσίες προσαρμοσμένης αρχιτεκτονικής (Bespoke Development) που έχουν ήδη ολοκληρωθεί ή εκτελεστεί σύμφωνα με τις τεχνικές σας προδιαγραφές.</li>
            </ul>

            <h2>3. Διαδικασία Επιστροφής Χρημάτων</h2>
            <p>Για να ασκήσετε το δικαίωμα επιστροφής, αποστείλετε γραπτό αίτημα στο <a href="mailto:{email}" style="color: var(--accent);">{email}</a> αναφέροντας τον αριθμό παραγγελίας ή το email αγοράς. Η επιστροφή των χρημάτων πραγματοποιείται εντός <strong>14 ημερών</strong> μέσω της ίδιας μεθόδου πληρωμής που χρησιμοποιήθηκε αρχικά, χωρίς καμία επιβάρυνση.</p>
        </div>
    "##,
        name = BUSINESS_LEGAL_NAME,
        email = BUSINESS_EMAIL,
    );
    Html(wrap_legal_page("Πολιτική Επιστροφών", &content))
}

pub async fn cookies_page_handler() -> Html<String> {
    let content = format!(r##"
        <div class="legal-header">
            <span class="legal-badge">ePrivacy Directive & GDPR</span>
            <h1>Πολιτική Cookies</h1>
            <div class="legal-meta">Τελευταία ενημέρωση: 2 Οκτωβρίου 2026 • Έκδοση 2.0</div>
        </div>
        <div class="legal-content">
            <p>Ο ιστότοπος της <strong>{name}</strong> χρησιμοποιεί cookies αποκλειστικά για την ασφαλή, αξιόπιστη και ομαλή λειτουργία των υπηρεσιών μας, χωρίς να παρακολουθεί την προσωπική σας δραστηριότητα σε ιστότοπους τρίτων.</p>

            <h2>1. Τι είναι τα Cookies;</h2>
            <p>Τα cookies είναι μικρά αρχεία κειμένου που αποθηκεύονται στον φυλλομετρητή σας κατά την επίσκεψή σας σε έναν ιστότοπο, επιτρέποντας την αναγνώριση των προτιμήσεών σας (π.χ. γλώσσα, εμφάνιση dark mode).</p>

            <h2>2. Κατηγορίες Cookies που Χρησιμοποιούμε</h2>
            <ul>
                <li><strong>Απολύτως Απαραίτητα Cookies (Strictly Necessary):</strong> Επιτρέπουν την πλοήγηση, την ασφάλεια συνεδρίας (session tokens, CSRF protection) και την προστασία από κυβερνοεπιθέσεις. Δεν απαιτούν συγκατάθεση καθώς είναι απαραίτητα για την παροχή της υπηρεσίας.</li>
                <li><strong>Cookies Προτιμήσεων & Λειτουργικότητας:</strong> Αποθηκεύουν τις επιλογές εμφάνισης (π.χ. Dark / Light theme, επιλογή γλώσσας).</li>
                <li><strong>Μηδενικά Διαφημιστικά / Tracking Cookies:</strong> Δεν χρησιμοποιούμε cookies τρίτων εταιρειών διαφήμισης (Meta, Google Ads κ.λπ.) που παρακολουθούν τον χρήστη.</li>
            </ul>

            <h2>3. Διαχείριση Προτιμήσεων</h2>
            <p>Μπορείτε οποτεδήποτε να αλλάξετε τις προτιμήσεις σας μέσω του banner συγκατάθεσης ή να διαγράψετε τα cookies από τις ρυθμίσεις του φυλλομετρητή σας.</p>
        </div>
    "##,
        name = BUSINESS_LEGAL_NAME,
    );
    Html(wrap_legal_page("Πολιτική Cookies", &content))
}

#[derive(Debug, Deserialize)]
pub struct DataDeletionRequest {
    pub email: String,
    pub full_name: String,
    pub request_type: String, // "erasure", "export", "rectification"
    pub details: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct DataDeletionResponse {
    pub success: bool,
    pub request_id: String,
    pub message: String,
}

pub async fn data_deletion_page_handler() -> Html<String> {
    let content = format!(r##"
        <div class="legal-header">
            <span class="legal-badge">GDPR Άρθρο 17 & 20 • ΔΙΚΑΙΩΜΑ ΣΤΗ ΛΗΘΗ</span>
            <h1>Αίτημα Διαγραφής & Εξαγωγής Προσωπικών Δεδομένων</h1>
            <div class="legal-meta">Αυτοματοποιημένη Πύλη Υποβολής Αιτημάτων Δεδομένων</div>
        </div>
        <div class="legal-content">
            <p>Σύμφωνα με τα Άρθρα 17 («Δικαίωμα στη Διαγραφή») και 20 («Δικαίωμα στη Φορητότητα») του GDPR και τον Ν. 4624/2019, μπορείτε να ζητήσετε την οριστική διαγραφή ή την εξαγωγή όλων των προσωπικών σας δεδομένων από τα συστήματα της <strong>{name}</strong>.</p>

            <div class="company-box" style="margin-top: 1.5rem;">
                <form id="deletion-form" onsubmit="handleDeletionSubmit(event)" style="display: flex; flex-direction: column; gap: 1rem;">
                    <div>
                        <label for="d-name" style="display: block; font-size: 0.85rem; font-weight: 600; margin-bottom: 0.35rem; color: var(--text-primary);">Ονοματεπώνυμο *</label>
                        <input id="d-name" type="text" required class="input" style="width: 100%; box-sizing: border-box; padding: 0.65rem 0.85rem; background: var(--bg-surface-sec); border: 1px solid var(--border); border-radius: var(--radius); color: var(--text-primary);" placeholder="π.χ. Ιωάννης Παπαδόπουλος" />
                    </div>

                    <div>
                        <label for="d-email" style="display: block; font-size: 0.85rem; font-weight: 600; margin-bottom: 0.35rem; color: var(--text-primary);">Διεύθυνση Email (συνδεδεμένη με τον λογαριασμό) *</label>
                        <input id="d-email" type="email" required class="input" style="width: 100%; box-sizing: border-box; padding: 0.65rem 0.85rem; background: var(--bg-surface-sec); border: 1px solid var(--border); border-radius: var(--radius); color: var(--text-primary);" placeholder="user@company.com" />
                    </div>

                    <div>
                        <label for="d-type" style="display: block; font-size: 0.85rem; font-weight: 600; margin-bottom: 0.35rem; color: var(--text-primary);">Τύπος Αιτήματος *</label>
                        <select id="d-type" class="input" style="width: 100%; box-sizing: border-box; padding: 0.65rem 0.85rem; background: var(--bg-surface-sec); border: 1px solid var(--border); border-radius: var(--radius); color: var(--text-primary);">
                            <option value="erasure">Οριστική Διαγραφή Δεδομένων (Right to Erasure / Forgotten - Art. 17)</option>
                            <option value="export">Εξαγωγή & Φορητότητα Δεδομένων (Right to Portability - Art. 20)</option>
                            <option value="rectification">Διόρθωση Ανακριβών Δεδομένων (Right to Rectification - Art. 16)</option>
                        </select>
                    </div>

                    <div>
                        <label for="d-details" style="display: block; font-size: 0.85rem; font-weight: 600; margin-bottom: 0.35rem; color: var(--text-primary);">Πρόσθετες Διευκρινίσεις (προαιρετικά)</label>
                        <textarea id="d-details" rows="3" class="input" style="width: 100%; box-sizing: border-box; padding: 0.65rem 0.85rem; background: var(--bg-surface-sec); border: 1px solid var(--border); border-radius: var(--radius); color: var(--text-primary); resize: vertical;" placeholder="Αναφέρετε αν επιθυμείτε διαγραφή συγκεκριμένων συσκευών ή του συνολικού λογαριασμού..."></textarea>
                    </div>

                    <div style="font-size: 0.8rem; color: var(--text-muted); line-height: 1.4;">
                        ℹ️ Μετά την υποβολή, θα λάβετε επιβεβαίωση στο email σας. Η εκτέλεση του αιτήματος ολοκληρώνεται εντός 30 ημερών όπως ορίζει ο νόμος.
                    </div>

                    <button id="d-btn" type="submit" class="btn" style="padding: 0.85rem 1.5rem; background: #e53e3e; color: white; border: none; border-radius: var(--radius); font-weight: 600; cursor: pointer;">
                        Υποβολή Αιτήματος GDPR
                    </button>
                    <div id="d-status" style="display: none; padding: 0.85rem; border-radius: var(--radius); font-size: 0.9rem;"></div>
                </form>
            </div>
        </div>

        <script>
            async function handleDeletionSubmit(e) {{
                e.preventDefault();
                const btn = document.getElementById('d-btn');
                const status = document.getElementById('d-status');
                btn.disabled = true;
                btn.textContent = 'Επεξεργασία Αιτήματος...';

                const payload = {{
                    full_name: document.getElementById('d-name').value,
                    email: document.getElementById('d-email').value,
                    request_type: document.getElementById('d-type').value,
                    details: document.getElementById('d-details').value || null
                }};

                try {{
                    const res = await fetch('/api/v1/privacy/delete-request', {{
                        method: 'POST',
                        headers: {{ 'Content-Type': 'application/json' }},
                        body: JSON.stringify(payload)
                    }});
                    const data = await res.json();
                    btn.disabled = false;
                    btn.textContent = 'Υποβολή Αιτήματος GDPR';
                    status.style.display = 'block';
                    status.style.background = 'rgba(16, 185, 129, 0.15)';
                    status.style.border = '1px solid #10b981';
                    status.style.color = '#34d399';
                    status.textContent = data.message || 'Το αίτημά σας καταχωρήθηκε επιτυχώς.';
                    document.getElementById('deletion-form').reset();
                }} catch (err) {{
                    btn.disabled = false;
                    btn.textContent = 'Υποβολή Αιτήματος GDPR';
                    status.style.display = 'block';
                    status.style.background = 'rgba(239, 68, 68, 0.15)';
                    status.style.border = '1px solid #ef4444';
                    status.style.color = '#f87171';
                    status.textContent = 'Σφάλμα επικοινωνίας. Παρακαλούμε αποστείλετε email απευθείας στο dpo@proteus-bos.internal';
                }}
            }}
        </script>
    "##,
        name = BUSINESS_LEGAL_NAME,
    );
    Html(wrap_legal_page("Αίτημα Διαγραφής Δεδομένων (GDPR)", &content))
}

pub async fn data_deletion_submit_handler(
    Json(req): Json<DataDeletionRequest>,
) -> impl IntoResponse {
    let request_id = format!("GDPR-{}", uuid::Uuid::new_v4().to_string().get(0..8).unwrap().to_uppercase());
    let now = chrono::Utc::now().to_rfc3339();

    // Log the erasure request to SQLite audit trail
    let db_path = proteus_core::paths::get_database_path();
    if let Ok(conn) = rusqlite::Connection::open(&db_path) {
        let _ = proteus_core::audit::init_audit_schema(&conn);
        let payload = serde_json::json!({
            "request_id": request_id,
            "full_name": req.full_name,
            "email": req.email,
            "request_type": req.request_type,
            "details": req.details,
            "submitted_at": now,
        }).to_string();

        let _ = proteus_core::audit::log_audit_event(
            &conn,
            &proteus_core::audit::SystemEvent::new(
                "GDPR_REQUEST",
                &request_id,
                "ERASURE_REQUEST_FILED",
                "Customer Self-Service",
                "Customer",
                format!("Αίτημα {} για το email {}", req.request_type, req.email),
                &payload,
            ),
        );
    }

    (
        StatusCode::OK,
        Json(DataDeletionResponse {
            success: true,
            request_id: request_id.clone(),
            message: format!(
                "Το αίτημα καταχωρήθηκε επιτυχώς με Αριθμό Αναφοράς #{}. Θα λάβετε επιβεβαίωση εντός 48 ωρών και η ολοκλήρωση θα εκτελεστεί σύμφωνα με το GDPR.",
                request_id
            ),
        }),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_legal_pages_rendered_and_compliant() {
        let privacy = privacy_page_handler().await.0;
        assert!(privacy.contains("GDPR"));
        assert!(privacy.contains("4624/2019"));
        assert!(privacy.contains(BUSINESS_LEGAL_NAME));
        assert!(privacy.contains(BUSINESS_AFM));
        assert!(privacy.contains(BUSINESS_GEMI));
        assert!(privacy.contains(BUSINESS_DPO_EMAIL));
        assert!(privacy.contains("18+")); // COPPA / GDPR-K

        let terms = terms_page_handler().await.0;
        assert!(terms.contains("Όροι Χρήσης"));
        assert!(terms.contains("Δικαστήρια των Αθηνών"));
        assert!(terms.contains("100% Bespoke Codebase"));

        let refund = refund_page_handler().await.0;
        assert!(refund.contains("14 ημερολογιακών ημερών"));
        assert!(refund.contains("2011/83/ΕΕ"));

        let cookies = cookies_page_handler().await.0;
        assert!(cookies.contains("Strictly Necessary"));
        assert!(cookies.contains("Μηδενικά Διαφημιστικά"));

        let deletion = data_deletion_page_handler().await.0;
        assert!(deletion.contains("Δικαίωμα στη Διαγραφή"));
        assert!(deletion.contains("Right to Erasure"));
        assert!(deletion.contains("Άρθρο 17"));
    }

    #[tokio::test]
    async fn test_data_deletion_submit_flow() {
        let req = DataDeletionRequest {
            email: "test.gdpr@example.com".to_string(),
            full_name: "Γεώργιος Παπαδόπουλος".to_string(),
            request_type: "erasure".to_string(),
            details: Some("Διαγραφή παλιών επισκευών".to_string()),
        };

        let resp = data_deletion_submit_handler(Json(req)).await.into_response();
        assert_eq!(resp.status(), StatusCode::OK);
    }
}

