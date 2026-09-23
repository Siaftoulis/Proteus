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
"#;
