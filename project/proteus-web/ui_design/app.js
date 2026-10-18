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

function toggleTheme() {
    const current = document.documentElement.getAttribute('data-theme') || 'dark';
    const next = current === 'dark' ? 'light' : 'dark';
    document.documentElement.setAttribute('data-theme', next);
    localStorage.setItem('proteus_theme', next);
    updateThemeToggleUI(next);
}

function updateThemeToggleUI(theme) {
    const btn = document.getElementById('theme-toggle-btn');
    if (!btn) return;
    if (theme === 'light') {
        btn.innerHTML = '<span class="theme-icon">☀️</span> <span>Light</span>';
        btn.setAttribute('title', 'Εναλλαγή σε Dark Mode');
    } else {
        btn.innerHTML = '<span class="theme-icon">🌙</span> <span>Dark</span>';
        btn.setAttribute('title', 'Εναλλαγή σε Light Mode');
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
