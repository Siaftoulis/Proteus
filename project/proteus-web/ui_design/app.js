// Proteus Web Client Interaction Scripts
// GDPR / ePrivacy Cookie Management, Theme Switcher, Smooth Anchors, and Contact Form API Dispatcher

window.addEventListener('DOMContentLoaded', () => {
    // 0. Auto-detect and apply System/OS Language (Zero manual toggles)
    applySystemLocale();

    // 1. Initialize Theme (Light / Dark)
    initTheme();

    // 2. Cookie Banner Visibility
    if (!localStorage.getItem('proteus_cookie_consent')) {
        const banner = document.getElementById('cookie-banner');
        if (banner) {
            banner.style.display = 'block';
        }
    }

    // 3. Initialize PowerPoint Slideshow Deck Navigation
    initSlideshowDeck();

    // 4. Initialize Ramp-Inspired Scroll-Driven Storytelling Engine
    initScrollStoryEngine();

    // 5. Initialize Industry Vertical Selector (from URL query ?v= or localStorage)
    initVerticalSelector();
});

function detectSystemLocale() {
    const raw = (navigator.languages && navigator.languages.length > 0)
        ? navigator.languages[0]
        : (navigator.language || navigator.userLanguage || 'en');
    return raw.toLowerCase().startsWith('el') ? 'el' : 'en';
}

const I18N_EN = {
    'nav_products': 'Products',
    'nav_solutions': 'Solutions',
    'nav_partners': 'Partners',
    'nav_resources': 'Resources',
    'nav_pricing': 'Pricing',
    'nav_login': 'Sign in',
    'nav_demo': 'Request Demo',
    'nav_start': 'Get Started →',
    'hero_title': 'Autonomy without cloud.<br><span class="editorial-accent">Zero subscriptions.</span>',
    'hero_subtitle': 'Device repairs, POS checkout, spatial WMS, and direct fiscal sync. Your entire business operations in an ultra-fast native engine with local encrypted SQLite and zero third-party dependencies.',
    'hero_input_ph': 'Company name or work email...',
    'hero_input_btn': 'Try Live Demo →',
    'hero_cta_free': 'Start Free & Demo →',
    'hero_cta_how': 'How It Works ↓',
    'app_window_tab': 'TechFix Solutions O.E. — Frontline POS & Service',
    'vendor_name': 'TechFix Repair Hub',
    'tx_timestamp': 'Today, 18:10 • Receipt #04821',
    'tx_item_1_name': 'OLED Screen Replacement iPhone 13 (IMEI 35892109281)',
    'tx_item_2_name': 'Technician Labor & Diagnostic Inspection',
    'tx_meta_eft': 'Payment: EFT/POS Terminal #02 (A.1155)',
    'tx_meta_esc': 'Print: ESC/POS 80mm (0.18ms)',
    'mobile_kicker': 'SERVICE ORDER #891',
    'mobile_status': 'READY',
    'mobile_title': 'On-site Field Service — Van 01',
    'mobile_sig_label': 'Customer Signature ✓',
    'quote_text': '"When internet goes down in the neighborhood, our checkout keeps printing receipts and booking repairs seamlessly."',
    'quote_author': 'George Karalis',
    'quote_role': 'Owner, TechFix Hub',
    'strip_runtime': 'SOVEREIGN RUNTIME:',
    'strip_runtime_val': '100% OFFLINE',
    'strip_tx': 'TRANSACTIONS:',
    'strip_tax': 'AADE myDATA:',
    'strip_tax_val': '0€ CLOUD TAX',
    'strip_latency': 'RESPONSE TIME:',
    'strip_printers': 'ESC/POS PRINTERS:',
    'strip_printers_val': 'ACTIVE',
    'strip_db': 'DATABASE ENGINE:',
    'spec_boot': 'Boot Time',
    'spec_db': 'Local DB',
    'spec_offline': 'Offline Resilience',
    'spec_drivers': 'Native Drivers'
};

function applySystemLocale() {
    const locale = detectSystemLocale();
    document.documentElement.setAttribute('lang', locale);
    if (locale === 'el') return; // Default HTML is already Greek

    document.querySelectorAll('[data-i18n]').forEach(el => {
        const key = el.getAttribute('data-i18n');
        if (I18N_EN[key]) {
            el.innerHTML = I18N_EN[key];
        }
    });

    document.querySelectorAll('[data-i18n-ph]').forEach(el => {
        const key = el.getAttribute('data-i18n-ph');
        if (I18N_EN[key]) {
            el.setAttribute('placeholder', I18N_EN[key]);
        }
    });
}


function initTheme() {
    const saved = localStorage.getItem('proteus_theme') || 'dark';
    document.documentElement.setAttribute('data-theme', saved);
    updateThemeToggleUI(saved);
}

function handleThemeSwitchToggle(checkbox) {
    const next = checkbox.checked ? 'dark' : 'light';
    document.documentElement.setAttribute('data-theme', next);
    localStorage.setItem('proteus_theme', next);
}

function toggleTheme() {
    const current = document.documentElement.getAttribute('data-theme') || 'dark';
    const next = current === 'dark' ? 'light' : 'dark';
    document.documentElement.setAttribute('data-theme', next);
    localStorage.setItem('proteus_theme', next);
    updateThemeToggleUI(next);
}

function updateThemeToggleUI(theme) {
    const checkbox = document.getElementById('theme-switch-checkbox');
    if (checkbox) {
        checkbox.checked = (theme === 'dark');
    }
}

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

function handleHeroSubmit(e) {
    if (e) e.preventDefault();
    const input = document.getElementById('hero-business-input');
    const val = input ? input.value.trim() : '';
    if (val) {
        const isEmail = val.includes('@');
        if (isEmail) {
            const emailField = document.getElementById('c-email');
            if (emailField) emailField.value = val;
        } else {
            const compField = document.getElementById('c-company');
            if (compField) compField.value = val;
        }
    }
    const contactSection = document.getElementById('contact');
    if (contactSection) {
        contactSection.scrollIntoView({ behavior: 'smooth' });
    }
}

// ── Industry Vertical Personas & Instant Profile Switcher Engine ──
const VERTICAL_DATA = {
    service: {
        el: {
            title: 'Αυτονομία χωρίς cloud.<br><span class="editorial-accent">Μηδέν συνδρομές.</span>',
            subtitle: 'Επισκευές συσκευών, ταμείο POS, αποθήκη WMS και άμεση έκδοση myDATA. Όλη η λειτουργία της επιχείρησής σας σε μία αστραπιαία native εφαρμογή, με τοπική κρυπτογραφημένη SQLite και πλήρη ανεξαρτησία από τρίτους.',
            tab: 'TechFix Solutions Ο.Ε. — Frontline POS & Service',
            vendor: 'TechFix Repair Hub',
            timestamp: 'Σήμερα, 18:10 • Απόδειξη #04821',
            amount: '€48,50',
            mark: '✓ myDATA MARK 400008921938',
            item1: 'Αλλαγή Οθόνης OLED iPhone 13 (IMEI 35892109281)',
            cost1: '€38,00',
            item2: 'Εργασία Τεχνικού & Διαγνωστικός Έλεγχος',
            cost2: '€10,50',
            metaEft: 'Πληρωμή: EFT/POS Terminal #02 (Α.1155)',
            metaEsc: 'Εκτύπωση: ESC/POS 80mm (0.18ms)',
            mobileKicker: 'ΕΝΤΟΛΗ SERVICE #891',
            mobileTitle: 'Επιτόπιο Service — Van 01',
            quoteText: '"Όταν πέφτει το internet στη γειτονιά, το ταμείο μας συνεχίζει να τυπώνει αποδείξεις και να καταγράφει επισκευές κανονικά."',
            quoteAuthor: 'Γιώργος Καραλής',
            quoteRole: 'Ιδιοκτήτης TechFix Hub'
        },
        en: {
            title: 'Autonomy without cloud.<br><span class="editorial-accent">Zero subscriptions.</span>',
            subtitle: 'Device repairs, POS checkout, spatial WMS, and direct fiscal sync. Your entire business operations in an ultra-fast native engine with local encrypted SQLite and zero third-party dependencies.',
            tab: 'TechFix Solutions O.E. — Frontline POS & Service',
            vendor: 'TechFix Repair Hub',
            timestamp: 'Today, 18:10 • Receipt #04821',
            amount: '€48.50',
            mark: '✓ myDATA MARK 400008921938',
            item1: 'OLED Screen Replacement iPhone 13 (IMEI 35892109281)',
            cost1: '€38.00',
            item2: 'Technician Labor & Diagnostic Inspection',
            cost2: '€10.50',
            metaEft: 'Payment: EFT/POS Terminal #02 (A.1155)',
            metaEsc: 'Print: ESC/POS 80mm (0.18ms)',
            mobileKicker: 'SERVICE ORDER #891',
            mobileTitle: 'On-site Field Service — Van 01',
            quoteText: '"When internet goes down in the neighborhood, our checkout keeps printing receipts and booking repairs seamlessly."',
            quoteAuthor: 'George Karalis',
            quoteRole: 'Owner, TechFix Hub'
        }
    },
    bakery: {
        el: {
            title: 'Ταμείο POS & Ζυγολόγιο για Φούρνους.<br><span class="editorial-accent">Ακαριαία έκδοση myDATA.</span>',
            subtitle: 'Αρτοσκευάσματα, καφέ, ζύγιση γραμμαρίου και barcode scanner. 0.1 δευτερόλεπτο ανά απόδειξη σε ώρες αιχμής, με άμεση εκτύπωση και σύνδεση με EFT-POS.',
            tab: 'Artisan Bakery & Cafe — POS & Barcode',
            vendor: 'Φούρνος "Ηπειρώτικο"',
            timestamp: 'Σήμερα, 08:24 • Απόδειξη Λιανικής #10294',
            amount: '€6,80',
            mark: '✓ myDATA MARK 400009182310',
            item1: 'Χωριάτικο Προζυμένιο Ψωμί 1kg',
            cost1: '€2,40',
            item2: 'Freddo Espresso & Τυρόπιτα Κουρού',
            cost2: '€4,40',
            metaEft: 'Πληρωμή: Ανέπαφη Κάρτα EFT/POS',
            metaEsc: 'Εκτύπωση: ESC/POS 80mm (Ακαριαία)',
            mobileKicker: 'ΠΑΡΑΓΓΕΛΙΑ TAKE-AWAY #104',
            mobileTitle: 'Έλεγχος Πάγκου & Ταμείου',
            quoteText: '"Σε ώρα αιχμής με ουρά 20 ατόμων, η έκδοση απόδειξης myDATA παίρνει 0.1 δευτερόλεπτο χωρίς καμία καθυστέρηση cloud."',
            quoteAuthor: 'Νίκος Παπαδόπουλος',
            quoteRole: 'Αρτοποιός & Ιδιοκτήτης'
        },
        en: {
            title: 'POS Checkout & Scales for Bakeries.<br><span class="editorial-accent">Instant Fiscal Sync.</span>',
            subtitle: 'Artisan bakery items, coffee, gram-level scale intake, and barcode scanning. 0.1s per ticket during morning rush hours with zero cloud lag.',
            tab: 'Artisan Bakery & Cafe — POS & Barcode',
            vendor: 'Artisan Bread Hub',
            timestamp: 'Today, 08:24 • POS Receipt #10294',
            amount: '€6.80',
            mark: '✓ myDATA MARK 400009182310',
            item1: 'Sourdough Country Bread 1kg',
            cost1: '€2.40',
            item2: 'Freddo Espresso & Butter Croissant',
            cost2: '€4.40',
            metaEft: 'Payment: Contactless Card EFT/POS',
            metaEsc: 'Print: ESC/POS 80mm (Instant)',
            mobileKicker: 'TAKE-AWAY ORDER #104',
            mobileTitle: 'Counter Checkout Terminal',
            quoteText: '"During morning rush hour with a line of 20 customers, issuing receipts takes 0.1s without waiting for external cloud servers."',
            quoteAuthor: 'Nikos Papadopoulos',
            quoteRole: 'Bakery Owner'
        }
    },
    medical: {
        el: {
            title: 'Ιατρικός Φάκελος & Ραντεβού.<br><span class="editorial-accent">Απόλυτο Ιατρικό Απόρρητο.</span>',
            subtitle: 'Κλινικό ιστορικό ασθενών, ηλεκτρονική συνταγογράφηση, έκδοση ΑΠΥ και myDATA. Πλήρης συμμόρφωση GDPR & HIPAA με κρυπτογραφημένη τοπική βάση.',
            tab: 'Ιατρικό Κέντρο & Πολυϊατρείο — Ραντεβού & ΑΠΥ',
            vendor: 'Dr. Μιχαηλίδης & Συνεργάτες',
            timestamp: 'Σήμερα, 11:30 • ΑΠΥ #00318',
            amount: '€60,00',
            mark: '✓ myDATA MARK 400007361899',
            item1: 'Κλινική Εξέταση & Ηλεκτρονική Συνταγογράφηση',
            cost1: '€50,00',
            item2: 'Υπερηχογράφημα & Ιατρική Γνωμάτευση',
            cost2: '€10,00',
            metaEft: 'Πληρωμή: IRIS B2B & Τερματικό POS',
            metaEsc: 'Σήμανση: ΑΑΔΕ myDATA QR Code',
            mobileKicker: 'ΙΑΤΡΙΚΟΣ ΦΑΚΕΛΟΣ #554',
            mobileTitle: 'Κρυπτογραφημένο Ιστορικό Ασθενούς',
            quoteText: '"Όλα τα ιατρικά ιστορικά και οι επισκέψεις μένουν αποκλειστικά στον τοπικό μας κρυπτογραφημένο δίσκο χωρίς κινδύνους διαρροής."',
            quoteAuthor: 'Δρ. Ελένη Μιχαηλίδου',
            quoteRole: 'Καρδιολόγος'
        },
        en: {
            title: 'Clinical Patient Records & Appointments.<br><span class="editorial-accent">Sovereign Medical Privacy.</span>',
            subtitle: 'Patient EHR clinical histories, appointments, and fiscal receipts. Strict GDPR & HIPAA compliance with local encrypted storage and zero cloud leaks.',
            tab: 'Medical Clinic & Care — EHR & Invoicing',
            vendor: 'Dr. Michaelides Practice',
            timestamp: 'Today, 11:30 • Invoice #00318',
            amount: '€60.00',
            mark: '✓ myDATA MARK 400007361899',
            item1: 'Clinical Consultation & E-Prescription',
            cost1: '€50.00',
            item2: 'Ultrasound Diagnostic Examination',
            cost2: '€10.00',
            metaEft: 'Payment: IRIS B2B & EFT/POS',
            metaEsc: 'Sign: Official myDATA QR Code',
            mobileKicker: 'PATIENT RECORD #554',
            mobileTitle: 'Zero-Cloud Clinical Archive',
            quoteText: '"All patient records and clinical visits remain strictly on our local encrypted drive with zero third-party exposure."',
            quoteAuthor: 'Dr. Helen Michaelides',
            quoteRole: 'Cardiologist'
        }
    },
    retail: {
        el: {
            title: 'Omnichannel POS & Barcode Scanner.<br><span class="editorial-accent">Χωρίς συνδρομές ERP.</span>',
            subtitle: 'Ηλεκτρονικές ετικέτες ραφιού (ESL), ταμείο POS, έλεγχος αποθέματος και παραστατικά myDATA. Ιδανικό για καταστήματα τεχνολογίας, ρούχων και πολυκαταστήματα.',
            tab: 'Retail Omnichannel — POS, ESL & Barcode',
            vendor: 'ElectroStore Retail S.A.',
            timestamp: 'Σήμερα, 16:45 • Απόδειξη Λιανικής #08711',
            amount: '€129,90',
            mark: '✓ myDATA MARK 400009418274',
            item1: 'Ασύρματα Ακουστικά Bluetooth ANC',
            cost1: '€89,90',
            item2: 'Powerbank 20.000mAh Fast Charge',
            cost2: '€40,00',
            metaEft: 'Πληρωμή: Apple Pay / EFT-POS #01',
            metaEsc: 'Συγχρονισμός: Ηλεκτρονικές Ετικέτες ESL',
            mobileKicker: 'ΤΑΜΕΙΟ POS #03',
            mobileTitle: 'Αυτόματη Απογραφή & Έλεγχος Κλοπής',
            quoteText: '"Αντικαταστήσαμε το παλιό αργό εμπορικό πρόγραμμα και εξοικονομούμε πάνω από 4.000€ το χρόνο σε άδειες χρήσης."',
            quoteAuthor: 'Μαρία Σταύρου',
            quoteRole: 'Store Supervisor'
        },
        en: {
            title: 'Omnichannel Retail POS & ESL Sync.<br><span class="editorial-accent">Zero Monthly ERP Fees.</span>',
            subtitle: 'Electronic Shelf Labels (ESL), high-speed barcode checkout, inventory control, and real-time myDATA invoices. Tailored for retail and tech stores.',
            tab: 'Retail Omnichannel — POS, ESL & Barcode',
            vendor: 'ElectroStore Retail S.A.',
            timestamp: 'Today, 16:45 • POS Receipt #08711',
            amount: '€129.90',
            mark: '✓ myDATA MARK 400009418274',
            item1: 'Wireless ANC Bluetooth Headphones',
            cost1: '€89.90',
            item2: 'Fast-Charge Powerbank 20,000mAh',
            cost2: '€40.00',
            metaEft: 'Payment: Apple Pay / EFT-POS #01',
            metaEsc: 'Sync: Electronic Shelf Labels (ESL)',
            mobileKicker: 'REGISTER POS #03',
            mobileTitle: 'Real-time Shrinkage & Inventory',
            quoteText: '"We replaced our legacy bloated software and save over €4,000 annually on recurring license seats."',
            quoteAuthor: 'Maria Stavrou',
            quoteRole: 'Store Supervisor'
        }
    },
    logistics: {
        el: {
            title: 'Spatial WMS & 3D Bin Packing.<br><span class="editorial-accent">Αστραπιαία διαχείριση αποθήκης.</span>',
            subtitle: 'Διαχείριση 250.000+ κωδικών SKU, ψηφιακά δελτία e-CMR, cross-docking intake και παλετοποίηση με αναζήτηση κάτω από 2 milliseconds.',
            tab: 'Spatial WMS & Distribution — e-CMR & VMI',
            vendor: 'Hellas Logistics Hub',
            timestamp: 'Σήμερα, 14:05 • Δελτίο Αποστολής #01928',
            amount: '€1.420,00',
            mark: '✓ myDATA MARK 400006281745',
            item1: 'Παλέτα Standard Euro-Pallet (48 κιβώτια)',
            cost1: '€960,00',
            item2: 'Express Μεταφορά & Cross-Docking Intake',
            cost2: '€460,00',
            metaEft: 'Όροι: Επί Πιστώσει B2B 30 Ημερών',
            metaEsc: 'Σήμανση: Ψηφιακό Δελτίο e-CMR & QR',
            mobileKicker: 'ΠΑΡΑΛΑΒΗ SMART TOTE #312',
            mobileTitle: '3D Bin Packing & Barcode Scanner',
            quoteText: '"Η τοπική SQLite διαχειρίζεται 250.000 κωδικούς αποθήκης και εκτελεί αναζήτηση αποθεμάτων σε λιγότερο από 2 milliseconds."',
            quoteAuthor: 'Κώστας Αναστασίου',
            quoteRole: 'Warehouse Manager'
        },
        en: {
            title: 'Spatial WMS & 3D Bin Packing.<br><span class="editorial-accent">Sub-2ms Inventory Engine.</span>',
            subtitle: 'Manage 250,000+ SKUs with digital e-CMR waybills, cross-docking smart tote intake, and palletization on local SQLite.',
            tab: 'Spatial WMS & Distribution — e-CMR & VMI',
            vendor: 'Hellas Logistics Hub',
            timestamp: 'Today, 14:05 • Waybill #01928',
            amount: '€1,420.00',
            mark: '✓ myDATA MARK 400006281745',
            item1: 'Standard Euro-Pallet Batch (48 units)',
            cost1: '€960.00',
            item2: 'Express Freight & Cross-Docking Intake',
            cost2: '€460.00',
            metaEft: 'Terms: B2B Credit 30 Days',
            metaEsc: 'Waybill: e-CMR Digitized & QR',
            mobileKicker: 'SMART TOTE INTAKE #312',
            mobileTitle: '3D Spatial Packing & Scan',
            quoteText: '"Local SQLite powers 250,000 warehouse SKUs and executes inventory queries in under 2 milliseconds."',
            quoteAuthor: 'Kostas Anastasiou',
            quoteRole: 'Warehouse Director'
        }
    },
    legal: {
        el: {
            title: 'Δικηγορικό Γραφείο & Χρονοχρέωση.<br><span class="editorial-accent">Μηδέν Cloud Διαρροές.</span>',
            subtitle: 'Δικογραφίες, χρονοχρέωση πελατών, έκδοση τιμολογίων παροχής και απευθείας διαβίβαση myDATA με 0€ κόστος παρόχου. Ασφάλεια τραπεζικού επιπέδου.',
            tab: 'Legal & Consulting Suite — Χρονοχρέωση & myDATA',
            vendor: 'Lex Veritas Partners',
            timestamp: 'Σήμερα, 17:15 • Τιμολόγιο Παροχής #00452',
            amount: '€250,00',
            mark: '✓ myDATA MARK 400008129304',
            item1: 'Νομική Συμβουλή & Σύνταξη Σύμβασης Εταιρείας',
            cost1: '€200,00',
            item2: 'Έλεγχος Τίτλων & Κατάθεση Δικογράφου',
            cost2: '€50,00',
            metaEft: 'Πληρωμή: Τραπεζική Μεταφορά SEPA / IRIS',
            metaEsc: 'Σήμανση: 0€ Κόστος Παρόχου myDATA',
            mobileKicker: 'ΔΙΚΟΓΡΑΦΙΑ #1128',
            mobileTitle: 'Κρυπτογραφημένο Αρχείο Υποθέσεων',
            quoteText: '"Καμία σύμβαση ή στοιχείο πελάτη δεν φεύγει σε τρίτους cloud servers. Ασφάλεια τραπεζικού επιπέδου στο δικό μας γραφείο."',
            quoteAuthor: 'Αλέξανδρος Δημητρίου',
            quoteRole: 'Δικηγόρος Παρ\' Αρείω Πάγω'
        },
        en: {
            title: 'Legal Practice & Billable Hours.<br><span class="editorial-accent">Zero Cloud Leak Risk.</span>',
            subtitle: 'Case management, client timesheets, bespoke service invoicing, and direct tax compliance. Sovereign security on your premises.',
            tab: 'Legal & Consulting Suite — Hours & myDATA',
            vendor: 'Lex Veritas Partners',
            timestamp: 'Today, 17:15 • Legal Invoice #00452',
            amount: '€250.00',
            mark: '✓ myDATA MARK 400008129304',
            item1: 'Legal Retainer & Corporate Contract Draft',
            cost1: '€200.00',
            item2: 'Title Verification & Court Filing',
            cost2: '€50.00',
            metaEft: 'Payment: SEPA Transfer / IRIS B2B',
            metaEsc: 'Sign: 0€ myDATA Cloud Provider Fee',
            mobileKicker: 'CASE DOCKET #1128',
            mobileTitle: 'Encrypted Matter Repository',
            quoteText: '"No contracts or client data are sent to third-party cloud servers. Banking-grade security inside our own chambers."',
            quoteAuthor: 'Alexandros Dimitriou',
            quoteRole: 'Senior Partner'
        }
    }
};

function switchVertical(key) {
    const verticalEntry = VERTICAL_DATA[key];
    if (!verticalEntry) return;

    const locale = (document.documentElement.getAttribute('lang') || 'el') === 'en' ? 'en' : 'el';
    const data = verticalEntry[locale] || verticalEntry['el'];

    // Update active pill UI
    document.querySelectorAll('.persona-pill').forEach(btn => {
        btn.classList.toggle('active', btn.getAttribute('data-vertical') === key);
    });

    // Update Hero Title & Subtitle with smooth subtle crossfade
    const heroTitle = document.querySelector('.hero-title');
    const heroSub = document.querySelector('.hero-subtitle');
    if (heroTitle) heroTitle.innerHTML = data.title;
    if (heroSub) heroSub.textContent = data.subtitle;

    // Update App Window elements
    const appTab = document.querySelector('[data-i18n="app_window_tab"]');
    if (appTab) appTab.textContent = data.tab;

    const vendor = document.querySelector('[data-i18n="vendor_name"]');
    if (vendor) vendor.textContent = data.vendor;

    const timestamp = document.querySelector('[data-i18n="tx_timestamp"]');
    if (timestamp) timestamp.textContent = data.timestamp;

    const txAmount = document.querySelector('.tx-amount');
    if (txAmount) txAmount.textContent = data.amount;

    const txBadge = document.querySelector('.tx-badge-verified');
    if (txBadge) txBadge.textContent = data.mark;

    const item1 = document.querySelector('[data-i18n="tx_item_1_name"]');
    if (item1) {
        item1.textContent = data.item1;
        const cost1 = item1.nextElementSibling;
        if (cost1) cost1.textContent = data.cost1;
    }

    const item2 = document.querySelector('[data-i18n="tx_item_2_name"]');
    if (item2) {
        item2.textContent = data.item2;
        const cost2 = item2.nextElementSibling;
        if (cost2) cost2.textContent = data.cost2;
    }

    const metaEft = document.querySelector('[data-i18n="tx_meta_eft"]');
    if (metaEft) metaEft.textContent = data.metaEft;

    const metaEsc = document.querySelector('[data-i18n="tx_meta_esc"]');
    if (metaEsc) metaEsc.textContent = data.metaEsc;

    // Update Floating Cards
    const mobKicker = document.querySelector('[data-i18n="mobile_kicker"]');
    if (mobKicker) mobKicker.textContent = data.mobileKicker;

    const mobTitle = document.querySelector('[data-i18n="mobile_title"]');
    if (mobTitle) mobTitle.textContent = data.mobileTitle;

    const quoteText = document.querySelector('[data-i18n="quote_text"]');
    if (quoteText) quoteText.textContent = data.quoteText;

    const quoteAuthor = document.querySelector('[data-i18n="quote_author"]');
    if (quoteAuthor) quoteAuthor.textContent = data.quoteAuthor;

    const quoteRole = document.querySelector('[data-i18n="quote_role"]');
    if (quoteRole) quoteRole.textContent = data.quoteRole;

    // Persist & URL Sync without reload
    try {
        const url = new URL(window.location.href);
        url.searchParams.set('v', key);
        window.history.replaceState({ vertical: key }, '', url.toString());
        localStorage.setItem('proteus_vertical', key);
    } catch (_) {}



function initVerticalSelector() {
    try {
        const urlParams = new URLSearchParams(window.location.search);
        const initialKey = urlParams.get('v') || localStorage.getItem('proteus_vertical') || 'service';
        if (VERTICAL_DATA[initialKey]) {
            switchVertical(initialKey);
        }
    } catch (_) {}
}

// ── PowerPoint Slideshow Deck Controller & Keyboard Navigation ──
function initSlideshowDeck() {
    const sections = Array.from(document.querySelectorAll('.landing-main > section'));
    if (!sections.length) return;

    let currentSlide = 0;

    // IntersectionObserver to track current slide position
    const observer = new IntersectionObserver((entries) => {
        entries.forEach(entry => {
            if (entry.isIntersecting) {
                const idx = sections.indexOf(entry.target);
                if (idx !== -1) {
                    currentSlide = idx;
                }
            }
        });
    }, {
        threshold: 0.5
    });

    sections.forEach(s => observer.observe(s));

    // PowerPoint-Style Keyboard Navigation (Arrows & Page Keys)
    window.addEventListener('keydown', (e) => {
        if (['INPUT', 'TEXTAREA', 'SELECT'].includes(document.activeElement?.tagName)) return;
        if (e.key === 'ArrowDown' || e.key === 'PageDown') {
            if (currentSlide < sections.length - 1) {
                e.preventDefault();
                goToSlide(currentSlide + 1);
            }
        } else if (e.key === 'ArrowUp' || e.key === 'PageUp') {
            if (currentSlide > 0) {
                e.preventDefault();
                goToSlide(currentSlide - 1);
            }
        } else if (e.key === 'Home') {
            e.preventDefault();
            goToSlide(0);
        } else if (e.key === 'End') {
            e.preventDefault();
            goToSlide(sections.length - 1);
        }
    });
}

function goToSlide(index) {
    const sections = Array.from(document.querySelectorAll('.landing-main > section'));
    if (sections[index]) {
        sections[index].scrollIntoView({ behavior: 'smooth' });
    }
}

// ── Ramp-Inspired Interactive Scroll-Driven Storytelling Engine ──
function initScrollStoryEngine() {
    const heroTrack = document.getElementById('hero');
    const heroLeft = document.querySelector('.hero-stage-left');
    const appCards = document.querySelectorAll('.ramp-app-card');
    const calloutCard = document.querySelector('.floating-callout-card');
    const mobileCard = document.querySelector('.floating-mobile-card');
    const aboutSection = document.getElementById('about');
    const featureCards = Array.from(document.querySelectorAll('.feature-card'));
    const aboutHeader = document.querySelector('#about .section-header');

    if (!heroTrack || !heroLeft || !appCards.length) return;

    let ticking = false;

    function onScroll() {
        if (!ticking) {
            window.requestAnimationFrame(() => {
                updateScrollStory();
                ticking = false;
            });
            ticking = true;
        }
    }

    function updateScrollStory() {
        const scrollY = window.scrollY || window.pageYOffset;
        const windowHeight = window.innerHeight;
        const isMobile = window.innerWidth <= 768;

        // 1. Hero Section Scrubbing Progress (0.0 to 1.0)
        const heroRect = heroTrack.getBoundingClientRect();
        const heroTop = heroRect.top + scrollY;
        const heroHeight = heroTrack.offsetHeight;
        const heroScrollTravel = Math.max(1, heroHeight - windowHeight);
        const heroProgress = Math.min(Math.max((scrollY - heroTop) / heroScrollTravel, 0), 1);

        if (!isMobile) {
            // Desktop / Laptop: Full Dimensional Parallax Scrubbing
            // A. Hero Left Column: Recedes, scales down, and fades smoothly
            const textY = heroProgress * -45;
            const textScale = 1 - heroProgress * 0.05;
            const textOpacity = Math.max(0, 1 - heroProgress * 1.45);
            heroLeft.style.transform = `translate3d(0, ${textY}px, 0) scale(${textScale})`;
            heroLeft.style.opacity = textOpacity.toFixed(3);

            // B. Hero App Window: Expands, elevates, and moves into central perspective
            const cardScale = 1 + heroProgress * 0.08;
            const cardY = heroProgress * -20;
            appCards.forEach(c => {
                c.style.transform = `translate3d(0, ${cardY}px, 0) scale(${cardScale})`;
            });

            // C. Floating Quote Card (Tony Hawk style): Floats up & left in 3D parallax
            if (calloutCard) {
                const calloutX = heroProgress * -35;
                const calloutY = heroProgress * -24;
                const calloutOpacity = Math.max(0, 1 - heroProgress * 1.6);
                const calloutScale = 1 - heroProgress * 0.08;
                calloutCard.style.transform = `translate3d(${calloutX}px, ${calloutY}px, 0) scale(${calloutScale})`;
                calloutCard.style.opacity = calloutOpacity.toFixed(3);
            }

            // D. Floating Mobile Service Ticket: Glides down & right in 3D parallax
            if (mobileCard) {
                const mobileX = heroProgress * 35;
                const mobileY = heroProgress * 24;
                const mobileOpacity = Math.max(0, 1 - heroProgress * 1.6);
                const mobileScale = 1 - heroProgress * 0.08;
                mobileCard.style.transform = `translate3d(${mobileX}px, ${mobileY}px, 0) scale(${mobileScale})`;
                mobileCard.style.opacity = mobileOpacity.toFixed(3);
            }
        } else {
            // Mobile Optimization: Subtle, non-overflowing transforms
            const textOpacity = Math.max(0, 1 - heroProgress * 1.3);
            heroLeft.style.opacity = textOpacity.toFixed(3);
            heroLeft.style.transform = `translate3d(0, ${heroProgress * -20}px, 0)`;

            const cardScale = 1 + heroProgress * 0.03;
            appCards.forEach(c => {
                c.style.transform = `scale(${cardScale})`;
            });

            if (calloutCard) calloutCard.style.opacity = Math.max(0, 1 - heroProgress * 1.6).toFixed(3);
            if (mobileCard) mobileCard.style.opacity = Math.max(0, 1 - heroProgress * 1.6).toFixed(3);
        }

        // 2. Section 2 (#about) Entrance Transition Scrubbing
        if (aboutSection) {
            const aboutRect = aboutSection.getBoundingClientRect();
            // Progress as #about approaches and fills the viewport (0.0 to 1.0)
            const aboutEnterStart = windowHeight * 0.95;
            const aboutEnterEnd = windowHeight * 0.15;
            const aboutRawProgress = (aboutEnterStart - aboutRect.top) / (aboutEnterStart - aboutEnterEnd);
            const aboutProgress = Math.min(Math.max(aboutRawProgress, 0), 1);

            if (aboutHeader) {
                const headerY = (1 - aboutProgress) * 35;
                aboutHeader.style.transform = `translate3d(0, ${headerY}px, 0)`;
                aboutHeader.style.opacity = aboutProgress.toFixed(3);
            }

            featureCards.forEach((card, idx) => {
                const cardDelay = idx * 0.08;
                const cardProgress = Math.min(Math.max((aboutProgress - cardDelay) / (1 - cardDelay), 0), 1);
                const cardY = (1 - cardProgress) * (50 + idx * 25);
                const cardScale = 0.94 + cardProgress * 0.06;
                card.style.transform = `translate3d(0, ${cardY}px, 0) scale(${cardScale})`;
                card.style.opacity = cardProgress.toFixed(3);
            });
        }
    }

    window.addEventListener('scroll', onScroll, { passive: true });
    window.addEventListener('resize', onScroll, { passive: true });
    updateScrollStory();
}


