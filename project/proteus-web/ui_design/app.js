// Proteus Web Client Interaction Scripts
// GDPR / ePrivacy Cookie Management, Theme Switcher, Smooth Anchors, and Contact Form API Dispatcher

window.addEventListener('DOMContentLoaded', () => {
    // 0. Initialize Theme (Light / Dark)
    initTheme();

    // 1. Cookie Banner Visibility
    if (!localStorage.getItem('proteus_cookie_consent')) {
        const banner = document.getElementById('cookie-banner');
        if (banner) {
            banner.style.display = 'block';
        }
    }
});

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
