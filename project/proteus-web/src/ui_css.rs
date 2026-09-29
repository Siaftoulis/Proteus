//! Nordic Slate Design System for Proteus Web Portal & Marketplace.
//! Adheres strictly to the human-crafted, utility-first "Average Joe" principle:
//! Crisp 1px hairline borders, subtle tonal shifts, zero purple gradients, and no muddy shadows.

pub const CSS_STYLES: &str = r#"
    :root {
        /* ── Base Palette: Nordic Slate ── */
        --palette-deep-slate-navy: #27374D;
        --palette-steel-slate:     #526D82;
        --palette-frost-gray:      #9DB2BF;
        --palette-icy-mist:        #DDE6ED;

        /* ── Dark Mode Semantic Tokens (Default) ── */
        --bg-base:        #1B2430; /* grounding canvas ground */
        --bg-surface:     #27374D; /* elevated container / card fill */
        --bg-surface-sec: #1E2A3A; /* recessed elements, sidebars, code blocks */
        --bg-card:        #27374D;
        --bg-card-hover:  #2f4159;

        --border-subtle:  rgba(82, 109, 130, 0.4); /* 1px division lines */
        --border-strong:  #526D82;                 /* active strokes, focused cards */
        --border:         rgba(82, 109, 130, 0.4);
        --border-bright:  #526D82;

        --text-primary:   #DDE6ED; /* crisp off-white, zero pure-white glare */
        --text-main:      #DDE6ED;
        --text-muted:     #9DB2BF; /* secondary labels, table headers, metadata */
        --text-dim:       #9DB2BF;
        --text-faint:     #526D82;

        --cta-primary-fill:   #DDE6ED;
        --cta-primary-text:   #27374D;
        --cta-secondary-fill: #27374D;
        --cta-secondary-text: #DDE6ED;
        --cta-secondary-border: #526D82;

        --accent:         #DDE6ED;
        --accent-hover:   #FFFFFF;
        --accent-text:    #27374D;

        /* Semantic Feedback */
        --success:        #38a169;
        --warning:        #d69e2e;
        --danger:         #e53e3e;
        --info:           #526D82;

        /* Precision Metrics (Restrained Radii) */
        --radius-xs: 3px;
        --radius-sm: 4px;
        --radius:    6px;
        --radius-lg: 10px;
        --focus-ring: 0 0 0 1px #526D82;
    }

    [data-theme="light"], .theme-light {
        /* ── Light Mode Semantic Tokens ── */
        --bg-base:        #F4F7F9; /* clean, slightly cooled neutral ground */
        --bg-surface:     #FFFFFF; /* pure white elevation */
        --bg-surface-sec: #DDE6ED; /* subtle container backgrounds, tables */
        --bg-card:        #FFFFFF;
        --bg-card-hover:  #F8FAFC;

        --border-subtle:  rgba(157, 178, 191, 0.45); /* crisp 1px hairline separators */
        --border-strong:  #9DB2BF;                  /* inputs, active borders */
        --border:         rgba(157, 178, 191, 0.45);
        --border-bright:  #9DB2BF;

        --text-primary:   #27374D; /* high-contrast, sharp typographic anchor */
        --text-main:      #27374D;
        --text-muted:     #526D82; /* secondary copy, captions */
        --text-dim:       #526D82;
        --text-faint:     #9DB2BF;

        --cta-primary-fill:   #27374D;
        --cta-primary-text:   #DDE6ED;
        --cta-secondary-fill: #DDE6ED;
        --cta-secondary-text: #27374D;
        --cta-secondary-border: #526D82;

        --accent:         #27374D;
        --accent-hover:   #1E2A3A;
        --accent-text:    #DDE6ED;

        --focus-ring: 0 0 0 1px #9DB2BF;
    }

    * { box-sizing: border-box; margin: 0; padding: 0; }

    body {
        font-family: 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
        background-color: var(--bg-base);
        color: var(--text-main);
        min-height: 100vh;
        display: flex;
        flex-direction: column;
        line-height: 1.5;
        -webkit-font-smoothing: antialiased;
    }

    /* ── Header & Navigation ── */
    header {
        border-bottom: 1px solid var(--border);
        background: var(--bg-surface-sec);
        padding: 0.75rem 1.5rem;
        display: flex;
        align-items: center;
        justify-content: space-between;
        position: sticky;
        top: 0;
        z-index: 50;
    }

    .brand-wrap { display: flex; align-items: center; gap: 0.75rem; }
    .brand { font-weight: 700; font-size: 1.1rem; letter-spacing: -0.02em; color: var(--text-primary); }
    .brand-badge {
        background: var(--bg-surface);
        color: var(--text-primary);
        border: 1px solid var(--border-strong);
        padding: 0.15rem 0.5rem;
        border-radius: var(--radius-sm);
        font-size: 0.7rem;
        font-weight: 700;
        letter-spacing: 0.04em;
    }

    .brand-logo-img {
        width: 30px;
        height: 30px;
        object-fit: contain;
        border-radius: var(--radius-sm);
        display: block;
    }

    .hero-emblem-wrap {
        display: flex;
        justify-content: center;
        align-items: center;
        margin-bottom: 1.25rem;
    }

    .hero-emblem-img {
        width: 72px;
        height: 72px;
        object-fit: contain;
        border-radius: 12px;
        filter: drop-shadow(0 4px 16px rgba(38, 88, 99, 0.4));
    }

    .nav-tabs {
        display: flex;
        gap: 0.35rem;
        background: var(--bg-base);
        padding: 0.25rem;
        border-radius: var(--radius);
        border: 1px solid var(--border);
    }

    .tab-btn {
        background: transparent;
        border: 1px solid transparent;
        color: var(--text-muted);
        padding: 0.4rem 0.85rem;
        border-radius: var(--radius-sm);
        font-size: 0.82rem;
        font-weight: 600;
        cursor: pointer;
        transition: color 0.15s ease, background 0.15s ease, border-color 0.15s ease;
        font-family: inherit;
    }

    .tab-btn:hover {
        color: var(--text-main);
        background: var(--bg-surface);
        border-color: var(--border);
    }
    .tab-btn.active {
        color: var(--cta-primary-text);
        background: var(--cta-primary-fill);
        border-color: var(--cta-primary-fill);
    }
    .tab-btn:focus-visible {
        outline: none;
        box-shadow: var(--focus-ring);
    }

    .status-pill {
        display: flex;
        align-items: center;
        gap: 0.4rem;
        font-size: 0.75rem;
        color: var(--success);
        background: rgba(56, 161, 105, 0.12);
        border: 1px solid rgba(56, 161, 105, 0.3);
        padding: 0.2rem 0.6rem;
        border-radius: var(--radius-sm);
        font-weight: 600;
    }
    .status-dot { width: 6px; height: 6px; background: var(--success); border-radius: 50%; display: inline-block; }

    main {
        flex: 1;
        max-width: 1140px;
        width: 100%;
        margin: 0 auto;
        padding: 2rem 1.5rem;
        display: flex;
        flex-direction: column;
        gap: 1.5rem;
    }

    .tab-panel { display: none; flex-direction: column; gap: 1.5rem; }
    .tab-panel.active { display: flex; }

    .section-header { margin-bottom: 0.35rem; }
    .section-title { font-size: 1.25rem; font-weight: 700; color: var(--text-primary); letter-spacing: -0.01em; }
    .section-sub { font-size: 0.85rem; color: var(--text-muted); margin-top: 0.2rem; }

    .grid-2 { display: grid; grid-template-columns: repeat(auto-fit, minmax(340px, 1fr)); gap: 1.25rem; }
    .grid-3 { display: grid; grid-template-columns: repeat(auto-fit, minmax(260px, 1fr)); gap: 1.25rem; }

    /* ── Cards & Surfaces ── */
    .card {
        background: var(--bg-card);
        border: 1px solid var(--border);
        border-radius: var(--radius-lg);
        padding: 1.35rem;
        display: flex;
        flex-direction: column;
        gap: 1.1rem;
        transition: border-color 0.15s ease;
    }
    .card:hover { border-color: var(--border-bright); }

    .card-header {
        font-weight: 600;
        font-size: 0.92rem;
        color: var(--text-primary);
        display: flex;
        align-items: center;
        justify-content: space-between;
    }

    .pkg-card {
        background: var(--bg-card);
        border: 1px solid var(--border);
        border-radius: var(--radius-lg);
        padding: 1.25rem;
        display: flex;
        flex-direction: column;
        justify-content: space-between;
        gap: 0.95rem;
        transition: border-color 0.15s ease, background-color 0.15s ease;
    }
    .pkg-card:hover {
        border-color: var(--border-strong);
        background: var(--bg-card-hover);
    }

    .pkg-top { display: flex; justify-content: space-between; align-items: flex-start; }
    .pkg-title { font-size: 1rem; font-weight: 700; color: var(--text-primary); }
    .pkg-desc { font-size: 0.84rem; color: var(--text-muted); line-height: 1.45; }
    .pkg-meta { display: flex; flex-wrap: wrap; gap: 0.4rem; font-size: 0.72rem; }

    /* ── Badges ── */
    .badge {
        font-size: 0.72rem;
        padding: 0.2rem 0.5rem;
        border-radius: var(--radius-xs);
        font-weight: 600;
        display: inline-flex;
        align-items: center;
        gap: 0.3rem;
        border: 1px solid var(--border);
        background: var(--bg-surface-sec);
        color: var(--text-main);
    }
    .badge-blue, .badge-slate {
        background: rgba(82, 109, 130, 0.2);
        color: var(--text-primary);
        border: 1px solid var(--border-strong);
    }
    .badge-green {
        background: rgba(56, 161, 105, 0.15);
        color: #48bb78;
        border: 1px solid rgba(56, 161, 105, 0.3);
    }
    .badge-purple {
        background: rgba(82, 109, 130, 0.15);
        color: var(--text-muted);
        border: 1px solid var(--border);
    }
    .badge-amber {
        background: rgba(214, 158, 46, 0.15);
        color: #ecc94b;
        border: 1px solid rgba(214, 158, 46, 0.3);
    }
    .badge-gray {
        background: rgba(157, 178, 191, 0.15);
        color: var(--text-muted);
        border: 1px solid var(--border);
    }

    /* ── Buttons ── */
    .btn {
        background: var(--cta-primary-fill);
        color: var(--cta-primary-text);
        border: 1px solid var(--border-strong);
        padding: 0.48rem 0.95rem;
        border-radius: var(--radius);
        font-size: 0.84rem;
        font-weight: 600;
        cursor: pointer;
        transition: opacity 0.15s ease, background 0.15s ease, border-color 0.15s ease;
        font-family: inherit;
        display: inline-flex;
        align-items: center;
        justify-content: center;
        gap: 0.4rem;
    }
    .btn:hover {
        opacity: 0.92;
        border-color: var(--palette-steel-slate);
    }
    .btn:active {
        opacity: 0.85;
    }
    .btn:focus-visible {
        outline: none;
        box-shadow: var(--focus-ring);
    }

    .btn-secondary {
        background: var(--cta-secondary-fill);
        color: var(--cta-secondary-text);
        border: 1px solid var(--cta-secondary-border);
    }
    .btn-secondary:hover {
        border-color: var(--palette-frost-gray);
        background: var(--bg-card-hover);
    }
    .btn-sm { padding: 0.32rem 0.65rem; font-size: 0.76rem; border-radius: var(--radius-sm); }
    .btn-success {
        background: var(--success);
        color: #fff;
        border-color: var(--success);
    }
    .btn-success:hover {
        opacity: 0.92;
    }

    /* ── Form Controls ── */
    .field-group { display: flex; flex-direction: column; gap: 0.35rem; }
    label { font-size: 0.83rem; font-weight: 500; color: var(--text-muted); display: flex; justify-content: space-between; }
    input[type="range"] { accent-color: var(--palette-steel-slate); cursor: pointer; }
    .checkbox-row { display: flex; align-items: center; gap: 0.55rem; font-size: 0.85rem; color: var(--text-main); cursor: pointer; }
    input[type="checkbox"] { accent-color: var(--palette-steel-slate); cursor: pointer; width: 15px; height: 15px; }

    .pds-input, .pds-textarea, .pds-select {
        background: var(--bg-surface-sec);
        border: 1px solid var(--border);
        border-radius: var(--radius-sm);
        padding: 0.55rem 0.75rem;
        color: var(--text-main);
        font-family: inherit;
        font-size: 0.84rem;
        transition: border-color 0.15s ease, box-shadow 0.15s ease;
        outline: none;
    }
    .pds-input:hover, .pds-textarea:hover, .pds-select:hover {
        border-color: var(--border-strong);
    }
    .pds-input:focus, .pds-textarea:focus, .pds-select:focus {
        border-color: var(--border-strong);
        box-shadow: var(--focus-ring);
    }
    .pds-select option { background: var(--bg-surface); color: var(--text-main); }

    /* ── Metrics & Tables ── */
    .stats-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(140px, 1fr)); gap: 0.85rem; }
    .stat-item {
        background: var(--bg-surface-sec);
        border: 1px solid var(--border);
        border-radius: var(--radius);
        padding: 0.85rem;
        text-align: center;
    }
    .stat-num { font-size: 1.4rem; font-weight: 700; color: var(--text-primary); }
    .stat-label { font-size: 0.7rem; color: var(--text-muted); margin-top: 0.2rem; text-transform: uppercase; letter-spacing: 0.04em; }

    .ticket-list { display: flex; flex-direction: column; gap: 0.5rem; max-height: 290px; overflow-y: auto; }
    .ticket-row {
        background: var(--bg-surface-sec);
        border: 1px solid var(--border);
        border-radius: var(--radius);
        padding: 0.65rem 0.9rem;
        display: flex;
        align-items: center;
        justify-content: space-between;
        font-size: 0.83rem;
    }
    .ticket-info { display: flex; flex-direction: column; gap: 0.15rem; }
    .ticket-title { font-weight: 600; color: var(--text-primary); }
    .ticket-meta { font-size: 0.73rem; color: var(--text-muted); }

    .tier-scale-table { width: 100%; border-collapse: collapse; font-size: 0.82rem; text-align: left; }
    .tier-scale-table th {
        padding: 0.55rem 0.75rem;
        background: var(--bg-surface-sec);
        color: var(--text-muted);
        border-bottom: 1px solid var(--border);
        font-weight: 600;
        font-size: 0.72rem;
        text-transform: uppercase;
        letter-spacing: 0.03em;
    }
    .tier-scale-table td { padding: 0.6rem 0.75rem; border-bottom: 1px solid var(--border); }
    .tier-scale-table tr:hover td { background: rgba(82, 109, 130, 0.1); }
    .tier-scale-table tr.active td {
        background: rgba(82, 109, 130, 0.2);
        font-weight: 600;
        color: var(--text-primary);
    }

    footer {
        border-top: 1px solid var(--border);
        padding: 1.15rem 1.5rem;
        text-align: center;
        font-size: 0.78rem;
        color: var(--text-muted);
        background: var(--bg-surface-sec);
    }

    /* ── Landing Page Specifications (Utility-First, Anti-AI) ── */
    .landing-header {
        border-bottom: 1px solid var(--border);
        background: var(--bg-surface-sec);
        padding: 0.85rem 2rem;
        display: flex;
        align-items: center;
        justify-content: space-between;
        position: sticky;
        top: 0;
        z-index: 100;
    }
    .brand-logo-img {
        width: 32px;
        height: 32px;
        object-fit: contain;
        display: block;
        filter: drop-shadow(0 1px 3px rgba(0, 0, 0, 0.35));
    }
    .brand-logo-icon {
        width: 26px;
        height: 26px;
        background: var(--cta-primary-fill);
        color: var(--cta-primary-text);
        border-radius: var(--radius-sm);
        display: flex;
        align-items: center;
        justify-content: center;
        font-weight: 700;
        font-size: 0.95rem;
    }
    .nav-links { display: flex; gap: 1.35rem; align-items: center; }
    .nav-link {
        color: var(--text-muted);
        text-decoration: none;
        font-size: 0.84rem;
        font-weight: 500;
        transition: color 0.15s ease;
    }
    .nav-link:hover { color: var(--text-primary); }
    .header-actions { display: flex; gap: 0.6rem; align-items: center; }

    .landing-main {
        max-width: 1200px;
        width: 100%;
        margin: 0 auto;
        padding: 2.5rem 1.5rem;
        display: flex;
        flex-direction: column;
        gap: 4rem;
    }

    .hero-section {
        text-align: center;
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 1.25rem;
        padding: 2rem 0 1rem;
    }
    .hero-pill {
        display: inline-flex;
        align-items: center;
        gap: 0.45rem;
        background: var(--bg-surface-sec);
        border: 1px solid var(--border-strong);
        color: var(--text-primary);
        font-size: 0.72rem;
        font-weight: 600;
        letter-spacing: 0.04em;
        padding: 0.25rem 0.75rem;
        border-radius: var(--radius-sm);
    }
    .pulse-dot {
        width: 6px;
        height: 6px;
        background: var(--palette-steel-slate);
        border-radius: 50%;
    }
    .hero-title {
        font-size: 2.7rem;
        font-weight: 800;
        color: var(--text-primary);
        line-height: 1.18;
        letter-spacing: -0.03em;
        max-width: 860px;
    }
    .gradient-text {
        color: var(--palette-frost-gray);
    }
    .hero-subtitle {
        font-size: 1.05rem;
        color: var(--text-muted);
        max-width: 700px;
        line-height: 1.55;
    }
    .hero-cta-group { display: flex; gap: 0.85rem; flex-wrap: wrap; justify-content: center; margin-top: 0.35rem; }
    .btn-lg { padding: 0.65rem 1.4rem; font-size: 0.92rem; border-radius: var(--radius); }
    .hero-specs-strip { display: flex; gap: 1.25rem; flex-wrap: wrap; justify-content: center; font-size: 0.78rem; color: var(--text-muted); margin-top: 0.75rem; }
    .spec-item span { color: var(--text-primary); font-weight: 600; }

    .section-header.center { text-align: center; display: flex; flex-direction: column; align-items: center; gap: 0.4rem; }
    .section-title { font-size: 1.85rem; font-weight: 800; color: var(--text-primary); letter-spacing: -0.02em; }
    .section-sub { font-size: 0.95rem; color: var(--text-muted); max-width: 620px; line-height: 1.5; }

    /* ── Workflow & Cards ── */
    .workflow-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(280px, 1fr)); gap: 1.25rem; }
    .workflow-card {
        background: var(--bg-card);
        border: 1px solid var(--border);
        border-radius: var(--radius-lg);
        padding: 1.6rem;
        display: flex;
        flex-direction: column;
        gap: 0.75rem;
    }
    .step-badge {
        width: 30px;
        height: 30px;
        border-radius: var(--radius-sm);
        background: var(--bg-surface-sec);
        color: var(--text-primary);
        display: flex;
        align-items: center;
        justify-content: center;
        font-size: 0.82rem;
        font-weight: 700;
        font-family: monospace;
        border: 1px solid var(--border-strong);
    }
    .workflow-title { font-size: 1.1rem; font-weight: 700; color: var(--text-primary); }
    .workflow-desc { font-size: 0.85rem; color: var(--text-muted); line-height: 1.5; }

    /* ── Services Grid ── */
    .services-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(320px, 1fr)); gap: 1.25rem; }
    .service-card {
        background: var(--bg-card);
        border: 1px solid var(--border);
        border-radius: var(--radius-lg);
        padding: 1.75rem;
        display: flex;
        flex-direction: column;
        gap: 1.1rem;
    }
    .service-card.featured {
        border-color: var(--border-strong);
        background: var(--bg-surface);
    }
    .service-header { display: flex; justify-content: space-between; align-items: center; }
    .service-title { font-size: 1.2rem; font-weight: 700; color: var(--text-primary); }
    .service-desc { font-size: 0.85rem; color: var(--text-muted); line-height: 1.45; }
    .service-list { list-style: none; display: flex; flex-direction: column; gap: 0.55rem; font-size: 0.83rem; color: var(--text-main); flex: 1; }
    .service-list li { display: flex; gap: 0.45rem; align-items: center; }

    /* ── Feature Grid ── */
    .feature-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(320px, 1fr)); gap: 1.25rem; }
    .feature-card {
        background: var(--bg-card);
        border: 1px solid var(--border);
        border-radius: var(--radius-lg);
        padding: 1.5rem;
        display: flex;
        flex-direction: column;
        gap: 0.7rem;
        transition: border-color 0.15s ease;
    }
    .feature-card:hover { border-color: var(--border-strong); }
    .feature-icon {
        width: 36px;
        height: 36px;
        border-radius: var(--radius-sm);
        background: var(--bg-surface-sec);
        color: var(--text-primary);
        display: flex;
        align-items: center;
        justify-content: center;
        font-size: 1.15rem;
        border: 1px solid var(--border);
    }
    .feature-title { font-size: 1.05rem; font-weight: 700; color: var(--text-primary); }
    .feature-desc { font-size: 0.85rem; color: var(--text-muted); line-height: 1.5; }

    /* ── Pricing Section ── */
    .pricing-table { display: grid; grid-template-columns: repeat(auto-fit, minmax(290px, 1fr)); gap: 1.25rem; align-items: stretch; }
    .pricing-card {
        background: var(--bg-card);
        border: 1px solid var(--border);
        border-radius: var(--radius-lg);
        padding: 1.75rem;
        display: flex;
        flex-direction: column;
        gap: 1.15rem;
        position: relative;
    }
    .pricing-card.highlighted {
        border-color: var(--border-strong);
        background: var(--bg-surface);
    }
    .popular-badge {
        position: absolute;
        top: -10px;
        left: 50%;
        transform: translateX(-50%);
        background: var(--cta-primary-fill);
        color: var(--cta-primary-text);
        font-size: 0.65rem;
        font-weight: 700;
        padding: 0.18rem 0.65rem;
        border-radius: var(--radius-xs);
        letter-spacing: 0.04em;
        border: 1px solid var(--border-strong);
    }
    .plan-name { font-size: 1.15rem; font-weight: 700; color: var(--text-primary); }
    .plan-price { font-size: 2rem; font-weight: 800; color: var(--text-primary); }
    .price-period { font-size: 0.82rem; color: var(--text-muted); font-weight: 400; }
    .plan-desc { font-size: 0.83rem; color: var(--text-muted); line-height: 1.45; }
    .plan-features { list-style: none; display: flex; flex-direction: column; gap: 0.65rem; font-size: 0.83rem; color: var(--text-main); flex: 1; }
    .plan-features li { display: flex; gap: 0.45rem; align-items: center; }
    .full-width { width: 100%; }

    /* ── Call To Action Banner ── */
    .cta-banner {
        background: var(--bg-surface-sec);
        border: 1px solid var(--border-strong);
        border-radius: var(--radius-lg);
        padding: 2.5rem 2rem;
        display: flex;
        justify-content: space-between;
        align-items: center;
        gap: 1.75rem;
        flex-wrap: wrap;
    }
    .cta-title { font-size: 1.5rem; font-weight: 800; color: var(--text-primary); }
    .cta-sub { font-size: 0.92rem; color: var(--text-muted); margin-top: 0.3rem; }
    .btn-white {
        background: var(--cta-primary-fill);
        color: var(--cta-primary-text);
        border: 1px solid var(--border-strong);
    }
    .btn-white:hover {
        opacity: 0.92;
    }

    /* ── Contact Form ── */
    .contact-form-wrap { max-width: 680px; margin: 0 auto; width: 100%; }
    .contact-form-card {
        background: var(--bg-card);
        border: 1px solid var(--border);
        border-radius: var(--radius-lg);
        padding: 2rem;
        display: flex;
        flex-direction: column;
        gap: 1.15rem;
    }
    .field-row { display: grid; grid-template-columns: 1fr 1fr; gap: 0.85rem; }

    /* ── Landing Footer ── */
    .landing-footer {
        border-top: 1px solid var(--border);
        padding: 2.5rem 2rem 2rem;
        background: var(--bg-surface-sec);
        display: flex;
        flex-direction: column;
        gap: 1.75rem;
        max-width: 1200px;
        margin: 0 auto;
        width: 100%;
    }
    .footer-inner { display: flex; justify-content: space-between; align-items: flex-start; flex-wrap: wrap; gap: 1.5rem; }
    .footer-links { display: flex; gap: 1.25rem; }
    .footer-links a { color: var(--text-muted); text-decoration: none; font-size: 0.83rem; }
    .footer-links a:hover { color: var(--text-primary); }
    .footer-copy {
        text-align: center;
        font-size: 0.74rem;
        color: var(--text-muted);
        border-top: 1px solid var(--border);
        padding-top: 1.25rem;
    }

    @media (max-width: 768px) {
        .hero-title { font-size: 2rem; }
        .nav-links { display: none; }
        .field-row { grid-template-columns: 1fr; }
        .cta-banner { flex-direction: column; text-align: center; }
    }
"#;

