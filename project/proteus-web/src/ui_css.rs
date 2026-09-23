//! Minimalist Luxury CSS Theme for Proteus Web Portal & Marketplace.
//! Adheres strictly to the "Average Joe" principle: zero visual clutter, clean cards, and responsive tabs.

pub const CSS_STYLES: &str = r#"
    :root {
        --bg-base: #0c0e14;
        --bg-card: #151821;
        --bg-card-hover: #1b202e;
        --bg-surface: #1e2436;
        --border: #262c3d;
        --border-bright: #3b4661;
        --text-main: #f1f5f9;
        --text-muted: #94a3b8;
        --accent: #3b82f6;
        --accent-hover: #2563eb;
        --success: #10b981;
        --warning: #f59e0b;
        --purple: #a855f7;
        --danger: #ef4444;
        --radius: 8px;
        --radius-lg: 12px;
    }

    * { box-sizing: border-box; margin: 0; padding: 0; }

    body {
        font-family: 'Inter', -apple-system, BlinkMacSystemFont, sans-serif;
        background-color: var(--bg-base);
        color: var(--text-main);
        min-height: 100vh;
        display: flex;
        flex-direction: column;
        line-height: 1.5;
    }

    header {
        border-bottom: 1px solid var(--border);
        background: rgba(21, 24, 33, 0.9);
        backdrop-filter: blur(10px);
        padding: 0.85rem 1.5rem;
        display: flex;
        align-items: center;
        justify-content: space-between;
        position: sticky;
        top: 0;
        z-index: 50;
    }

    .brand-wrap { display: flex; align-items: center; gap: 0.75rem; }
    .brand { font-weight: 700; font-size: 1.15rem; letter-spacing: -0.02em; color: #fff; }
    .brand-badge { background: var(--accent); color: #fff; padding: 0.15rem 0.45rem; border-radius: 4px; font-size: 0.72rem; font-weight: 700; }
    
    .nav-tabs {
        display: flex;
        gap: 0.35rem;
        background: rgba(12, 14, 20, 0.7);
        padding: 0.25rem;
        border-radius: var(--radius);
        border: 1px solid var(--border);
    }

    .tab-btn {
        background: transparent;
        border: none;
        color: var(--text-muted);
        padding: 0.4rem 0.85rem;
        border-radius: 6px;
        font-size: 0.82rem;
        font-weight: 600;
        cursor: pointer;
        transition: all 0.15s ease-in-out;
        font-family: inherit;
    }

    .tab-btn:hover { color: var(--text-main); background: rgba(255, 255, 255, 0.04); }
    .tab-btn.active { color: #fff; background: var(--accent); }

    .status-pill {
        display: flex;
        align-items: center;
        gap: 0.4rem;
        font-size: 0.75rem;
        color: var(--success);
        background: rgba(16, 185, 129, 0.1);
        border: 1px solid rgba(16, 185, 129, 0.25);
        padding: 0.25rem 0.65rem;
        border-radius: 20px;
        font-weight: 600;
    }
    .status-dot { width: 7px; height: 7px; background: var(--success); border-radius: 50%; display: inline-block; }

    main {
        flex: 1;
        max-width: 1140px;
        width: 100%;
        margin: 0 auto;
        padding: 2rem 1.5rem;
        display: flex;
        flex-direction: column;
        gap: 1.75rem;
    }

    .tab-panel { display: none; flex-direction: column; gap: 1.75rem; }
    .tab-panel.active { display: flex; }

    .section-header { margin-bottom: 0.5rem; }
    .section-title { font-size: 1.3rem; font-weight: 700; color: #fff; letter-spacing: -0.01em; }
    .section-sub { font-size: 0.875rem; color: var(--text-muted); margin-top: 0.2rem; }

    .grid-2 { display: grid; grid-template-columns: repeat(auto-fit, minmax(340px, 1fr)); gap: 1.25rem; }
    .grid-3 { display: grid; grid-template-columns: repeat(auto-fit, minmax(260px, 1fr)); gap: 1.25rem; }

    .card {
        background: var(--bg-card);
        border: 1px solid var(--border);
        border-radius: var(--radius-lg);
        padding: 1.4rem;
        display: flex;
        flex-direction: column;
        gap: 1.15rem;
        transition: border-color 0.15s ease;
    }
    .card:hover { border-color: var(--border-bright); }

    .card-header {
        font-weight: 600;
        font-size: 0.95rem;
        color: #fff;
        display: flex;
        align-items: center;
        justify-content: space-between;
    }

    .pkg-card {
        background: var(--bg-card);
        border: 1px solid var(--border);
        border-radius: var(--radius-lg);
        padding: 1.35rem;
        display: flex;
        flex-direction: column;
        justify-content: space-between;
        gap: 1rem;
        transition: transform 0.15s ease, border-color 0.15s ease, background-color 0.15s ease;
    }
    .pkg-card:hover {
        transform: translateY(-2px);
        border-color: var(--accent);
        background: var(--bg-card-hover);
    }

    .pkg-top { display: flex; justify-content: space-between; align-items: flex-start; }
    .pkg-title { font-size: 1.05rem; font-weight: 700; color: #fff; }
    .pkg-desc { font-size: 0.85rem; color: var(--text-muted); line-height: 1.45; }
    .pkg-meta { display: flex; flex-wrap: wrap; gap: 0.45rem; font-size: 0.72rem; }

    .badge {
        font-size: 0.72rem;
        padding: 0.2rem 0.55rem;
        border-radius: 4px;
        font-weight: 600;
        display: inline-flex;
        align-items: center;
        gap: 0.3rem;
    }
    .badge-blue { background: rgba(59, 130, 246, 0.15); color: #60a5fa; border: 1px solid rgba(59, 130, 246, 0.25); }
    .badge-green { background: rgba(16, 185, 129, 0.15); color: #34d399; border: 1px solid rgba(16, 185, 129, 0.25); }
    .badge-purple { background: rgba(168, 85, 247, 0.15); color: #c084fc; border: 1px solid rgba(168, 85, 247, 0.25); }
    .badge-amber { background: rgba(245, 158, 11, 0.15); color: #fbbf24; border: 1px solid rgba(245, 158, 11, 0.25); }
    .badge-gray { background: rgba(148, 163, 184, 0.15); color: #94a3b8; border: 1px solid rgba(148, 163, 184, 0.25); }

    .btn {
        background: var(--accent);
        color: #fff;
        border: none;
        padding: 0.5rem 1rem;
        border-radius: 6px;
        font-size: 0.84rem;
        font-weight: 600;
        cursor: pointer;
        transition: background 0.15s ease;
        font-family: inherit;
        display: inline-flex;
        align-items: center;
        justify-content: center;
        gap: 0.4rem;
    }
    .btn:hover { background: var(--accent-hover); }
    .btn-secondary { background: var(--bg-surface); color: var(--text-main); border: 1px solid var(--border); }
    .btn-secondary:hover { background: var(--border-bright); }
    .btn-sm { padding: 0.35rem 0.7rem; font-size: 0.78rem; }
    .btn-success { background: #059669; }
    .btn-success:hover { background: #10b981; }

    .field-group { display: flex; flex-direction: column; gap: 0.4rem; }
    label { font-size: 0.85rem; font-weight: 500; color: var(--text-muted); display: flex; justify-content: space-between; }
    input[type="range"] { accent-color: var(--accent); cursor: pointer; }
    .checkbox-row { display: flex; align-items: center; gap: 0.6rem; font-size: 0.875rem; color: var(--text-main); cursor: pointer; }
    input[type="checkbox"] { accent-color: var(--accent); cursor: pointer; width: 16px; height: 16px; }

    .stats-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(150px, 1fr)); gap: 1rem; }
    .stat-item { background: var(--bg-base); border: 1px solid var(--border); border-radius: var(--radius); padding: 1rem; text-align: center; }
    .stat-num { font-size: 1.5rem; font-weight: 700; color: #fff; }
    .stat-label { font-size: 0.72rem; color: var(--text-muted); margin-top: 0.2rem; text-transform: uppercase; letter-spacing: 0.04em; }

    .ticket-list { display: flex; flex-direction: column; gap: 0.55rem; max-height: 290px; overflow-y: auto; }
    .ticket-row { background: var(--bg-base); border: 1px solid var(--border); border-radius: var(--radius); padding: 0.75rem 1rem; display: flex; align-items: center; justify-content: space-between; font-size: 0.85rem; }
    .ticket-info { display: flex; flex-direction: column; gap: 0.15rem; }
    .ticket-title { font-weight: 600; color: #fff; }
    .ticket-meta { font-size: 0.75rem; color: var(--text-muted); }

    .tier-scale-table { width: 100%; border-collapse: collapse; font-size: 0.83rem; text-align: left; }
    .tier-scale-table th { padding: 0.6rem 0.8rem; background: var(--bg-base); color: var(--text-muted); border-bottom: 1px solid var(--border); font-weight: 600; font-size: 0.75rem; text-transform: uppercase; }
    .tier-scale-table td { padding: 0.65rem 0.8rem; border-bottom: 1px solid var(--border); }
    .tier-scale-table tr:hover td { background: rgba(255, 255, 255, 0.02); }
    .tier-scale-table tr.active td { background: rgba(59, 130, 246, 0.1); font-weight: 600; color: #fff; }

    footer { border-top: 1px solid var(--border); padding: 1.25rem 1.5rem; text-align: center; font-size: 0.8rem; color: var(--text-muted); }

    /* Landing Page Specific Styles */
    .landing-header { border-bottom: 1px solid var(--border); background: rgba(12, 14, 20, 0.85); backdrop-filter: blur(12px); padding: 0.9rem 2rem; display: flex; align-items: center; justify-content: space-between; position: sticky; top: 0; z-index: 100; }
    .brand-logo-icon { width: 28px; height: 28px; background: var(--accent); color: #fff; border-radius: 6px; display: flex; align-items: center; justify-content: center; font-weight: 700; font-size: 1rem; }
    .nav-links { display: flex; gap: 1.5rem; align-items: center; }
    .nav-link { color: var(--text-muted); text-decoration: none; font-size: 0.85rem; font-weight: 500; transition: color 0.15s ease; }
    .nav-link:hover { color: #fff; }
    .header-actions { display: flex; gap: 0.6rem; align-items: center; }

    .landing-main { max-width: 1200px; width: 100%; margin: 0 auto; padding: 3rem 1.5rem; display: flex; flex-direction: column; gap: 5rem; }
    
    .hero-section { text-align: center; display: flex; flex-direction: column; align-items: center; gap: 1.5rem; padding: 2.5rem 0 1.5rem; }
    .hero-pill { display: inline-flex; align-items: center; gap: 0.5rem; background: rgba(59, 130, 246, 0.1); border: 1px solid rgba(59, 130, 246, 0.25); color: #60a5fa; font-size: 0.72rem; font-weight: 700; letter-spacing: 0.05em; padding: 0.35rem 0.9rem; border-radius: 9999px; }
    .pulse-dot { width: 7px; height: 7px; background: #60a5fa; border-radius: 50%; box-shadow: 0 0 8px #60a5fa; }
    .hero-title { font-size: 2.9rem; font-weight: 800; color: #fff; line-height: 1.18; letter-spacing: -0.03em; max-width: 860px; }
    .gradient-text { background: linear-gradient(135deg, #60a5fa 0%, #a855f7 100%); -webkit-background-clip: text; -webkit-text-fill-color: transparent; }
    .hero-subtitle { font-size: 1.1rem; color: var(--text-muted); max-width: 720px; line-height: 1.6; }
    .hero-cta-group { display: flex; gap: 1rem; flex-wrap: wrap; justify-content: center; margin-top: 0.5rem; }
    .btn-lg { padding: 0.75rem 1.6rem; font-size: 0.95rem; border-radius: 8px; }
    .hero-specs-strip { display: flex; gap: 1.5rem; flex-wrap: wrap; justify-content: center; font-size: 0.8rem; color: var(--text-muted); margin-top: 1rem; }
    .spec-item span { color: #fff; font-weight: 600; }

    /* Canvas Mockup Frame */
    .canvas-preview-section { display: flex; flex-direction: column; gap: 2rem; }
    .section-header.center { text-align: center; display: flex; flex-direction: column; align-items: center; gap: 0.5rem; }
    .canvas-mockup-frame { background: var(--bg-card); border: 1px solid var(--border); border-radius: var(--radius-lg); overflow: hidden; box-shadow: 0 20px 40px rgba(0,0,0,0.6); }
    .canvas-bar { background: var(--bg-base); border-bottom: 1px solid var(--border); padding: 0.6rem 1rem; display: flex; align-items: center; justify-content: space-between; font-size: 0.8rem; }
    .canvas-dots { display: flex; gap: 6px; }
    .dot { width: 10px; height: 10px; border-radius: 50%; }
    .dot.red { background: #ef4444; } .dot.yellow { background: #f59e0b; } .dot.green { background: #10b981; }
    .canvas-title { color: var(--text-muted); font-size: 0.75rem; font-family: monospace; }
    .canvas-mode-toggle { display: flex; gap: 0.4rem; }
    .mode-btn { background: transparent; border: 1px solid var(--border); color: var(--text-muted); padding: 0.25rem 0.6rem; border-radius: 4px; font-size: 0.72rem; cursor: pointer; transition: all 0.15s; }
    .mode-btn.active { background: var(--accent); color: #fff; border-color: var(--accent); }
    
    .canvas-body { display: flex; height: 380px; position: relative; }
    .canvas-tools { width: 44px; background: var(--bg-base); border-right: 1px solid var(--border); display: flex; flex-direction: column; align-items: center; gap: 0.6rem; padding: 0.8rem 0; }
    .tool-icon { width: 28px; height: 28px; border-radius: 4px; display: flex; align-items: center; justify-content: center; color: var(--text-muted); font-size: 0.85rem; cursor: pointer; transition: all 0.15s; }
    .tool-icon:hover, .tool-icon.active { background: rgba(59, 130, 246, 0.2); color: var(--accent); }
    
    .canvas-viewport { flex: 1; background: #080a0f; background-image: radial-gradient(#1f2433 1px, transparent 1px); background-size: 16px 16px; padding: 1.5rem; overflow: auto; display: flex; justify-content: center; align-items: center; }
    .live-artboard { background: var(--bg-card); border: 1px solid var(--border-bright); border-radius: var(--radius-lg); width: 100%; max-width: 640px; padding: 1.25rem; display: flex; flex-direction: column; gap: 1rem; box-shadow: 0 8px 24px rgba(0,0,0,0.5); }
    .artboard-header { display: flex; justify-content: space-between; align-items: flex-start; }
    .artboard-name { font-size: 0.68rem; color: var(--accent); text-transform: uppercase; font-weight: 600; letter-spacing: 0.05em; }
    .artboard-stats { display: grid; grid-template-columns: repeat(3, 1fr); gap: 0.75rem; }
    .mini-stat { background: var(--bg-base); border: 1px solid var(--border); border-radius: var(--radius); padding: 0.6rem 0.8rem; text-align: center; }
    .mini-val { font-size: 1.1rem; font-weight: 700; color: #fff; }
    .mini-lbl { font-size: 0.62rem; color: var(--text-muted); text-transform: uppercase; margin-top: 0.15rem; }
    .artboard-table { background: var(--bg-base); border: 1px solid var(--border); border-radius: var(--radius); padding: 0.8rem; }
    .table-bar { display: flex; justify-content: space-between; align-items: center; font-size: 0.8rem; font-weight: 600; color: #fff; margin-bottom: 0.6rem; }
    .table-rows { display: flex; flex-direction: column; gap: 0.4rem; max-height: 120px; overflow-y: auto; }
    .t-row { display: flex; justify-content: space-between; align-items: center; font-size: 0.78rem; padding: 0.35rem 0.5rem; background: var(--bg-card); border-radius: 4px; }

    .canvas-inspector { width: 220px; background: var(--bg-base); border-left: 1px solid var(--border); padding: 1rem; display: flex; flex-direction: column; gap: 1rem; font-size: 0.8rem; transition: opacity 0.2s ease; }
    .insp-title { font-size: 0.68rem; font-weight: 700; color: var(--text-muted); letter-spacing: 0.06em; }
    .insp-group { display: flex; flex-direction: column; gap: 0.25rem; }
    .insp-label { font-size: 0.7rem; color: var(--text-muted); }
    .insp-val { font-size: 0.75rem; color: #fff; }
    .insp-val code { background: rgba(255,255,255,0.05); padding: 0.15rem 0.35rem; border-radius: 3px; font-family: monospace; color: var(--accent); }

    /* Feature Grid Section */
    .feature-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(320px, 1fr)); gap: 1.5rem; }
    .feature-card { background: var(--bg-card); border: 1px solid var(--border); border-radius: var(--radius-lg); padding: 1.75rem; display: flex; flex-direction: column; gap: 0.75rem; transition: border-color 0.15s, transform 0.15s; }
    .feature-card:hover { border-color: var(--accent); transform: translateY(-2px); }
    .feature-icon { width: 40px; height: 40px; border-radius: 8px; background: rgba(59, 130, 246, 0.12); color: var(--accent); display: flex; align-items: center; justify-content: center; font-size: 1.25rem; }
    .feature-title { font-size: 1.1rem; font-weight: 700; color: #fff; }
    .feature-desc { font-size: 0.875rem; color: var(--text-muted); line-height: 1.55; }

    /* Pricing Section */
    .pricing-table { display: grid; grid-template-columns: repeat(auto-fit, minmax(300px, 1fr)); gap: 1.5rem; align-items: stretch; }
    .pricing-card { background: var(--bg-card); border: 1px solid var(--border); border-radius: var(--radius-lg); padding: 2rem; display: flex; flex-direction: column; gap: 1.25rem; position: relative; }
    .pricing-card.highlighted { border-color: var(--accent); background: linear-gradient(180deg, rgba(59, 130, 246, 0.08) 0%, var(--bg-card) 40%); box-shadow: 0 8px 30px rgba(59, 130, 246, 0.15); }
    .popular-badge { position: absolute; top: -11px; left: 50%; transform: translateX(-50%); background: var(--accent); color: #fff; font-size: 0.68rem; font-weight: 700; padding: 0.2rem 0.7rem; border-radius: 9999px; letter-spacing: 0.05em; }
    .plan-name { font-size: 1.2rem; font-weight: 700; color: #fff; }
    .plan-price { font-size: 2.2rem; font-weight: 800; color: #fff; }
    .price-period { font-size: 0.85rem; color: var(--text-muted); font-weight: 400; }
    .plan-desc { font-size: 0.85rem; color: var(--text-muted); line-height: 1.45; }
    .plan-features { list-style: none; display: flex; flex-direction: column; gap: 0.7rem; font-size: 0.85rem; color: var(--text-main); flex: 1; }
    .plan-features li { display: flex; gap: 0.5rem; align-items: center; }
    .full-width { width: 100%; }

    /* Call To Action Banner */
    .cta-banner { background: linear-gradient(135deg, #1e3a8a 0%, #172554 100%); border: 1px solid #3b82f6; border-radius: var(--radius-lg); padding: 3rem 2.5rem; display: flex; justify-content: space-between; align-items: center; gap: 2rem; flex-wrap: wrap; box-shadow: 0 12px 35px rgba(30, 58, 138, 0.3); }
    .cta-title { font-size: 1.6rem; font-weight: 800; color: #fff; }
    .cta-sub { font-size: 0.95rem; color: #93c5fd; margin-top: 0.35rem; }
    .btn-white { background: #fff; color: #0f172a; }
    .btn-white:hover { background: #f1f5f9; }

    /* Contact Form Section */
    .contact-form-wrap { max-width: 640px; margin: 0 auto; width: 100%; }
    .contact-form-card { background: var(--bg-card); border: 1px solid var(--border); border-radius: var(--radius-lg); padding: 2rem; display: flex; flex-direction: column; gap: 1.2rem; }
    .field-row { display: grid; grid-template-columns: 1fr 1fr; gap: 1rem; }
    .pds-input, .pds-textarea { background: var(--bg-base); border: 1px solid var(--border); border-radius: 6px; padding: 0.65rem 0.85rem; color: #fff; font-family: inherit; font-size: 0.85rem; transition: border-color 0.15s; outline: none; }
    .pds-input:focus, .pds-textarea:focus { border-color: var(--accent); }

    /* Landing Footer */
    .landing-footer { border-top: 1px solid var(--border); padding: 3rem 2rem 2rem; background: var(--bg-base); display: flex; flex-direction: column; gap: 2rem; max-width: 1200px; margin: 0 auto; width: 100%; }
    .footer-inner { display: flex; justify-content: space-between; align-items: flex-start; flex-wrap: wrap; gap: 2rem; }
    .footer-links { display: flex; gap: 1.5rem; }
    .footer-links a { color: var(--text-muted); text-decoration: none; font-size: 0.85rem; }
    .footer-links a:hover { color: #fff; }
    .footer-copy { text-align: center; font-size: 0.75rem; color: var(--text-muted); border-top: 1px solid rgba(255,255,255,0.05); padding-top: 1.5rem; }

    @media (max-width: 768px) {
        .hero-title { font-size: 2.1rem; }
        .nav-links { display: none; }
        .field-row { grid-template-columns: 1fr; }
        .canvas-inspector { display: none; }
        .cta-banner { flex-direction: column; text-align: center; }
    }
"#;
