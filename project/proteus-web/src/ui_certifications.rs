//! Certifications & Partner Program UI Template for Proteus Web.
//! Showcases professional certification tracks (PCD, PCSS, PCDS, PCDA, Master Bundle)
//! and an interactive 10-Tier sliding commission calculator.

pub const CERTIFICATIONS_HTML: &str = r#"
    <div class="tab-panel" id="panel-certifications">
        <div class="section-header">
            <h2 class="section-title">Ακαδημία Πιστοποιήσεων & Συνεργάτες</h2>
            <p class="section-sub">Επίσημες πιστοποιήσεις Proteus και κλιμακωτό πρόγραμμα προμηθειών 10 επιπέδων.</p>
        </div>

        <div class="grid-2">
            <!-- Certification Tracks Card -->
            <div class="card">
                <div class="card-header">
                    <span>Πιστοποιήσεις Επαγγελματιών</span>
                    <span class="badge badge-purple">Official Tracks</span>
                </div>
                <div style="display: flex; flex-direction: column; gap: 0.85rem;">
                    <div style="background: var(--bg-base); border: 1px solid var(--border); border-radius: var(--radius); padding: 0.85rem 1rem;">
                        <div style="display: flex; justify-content: space-between; align-items: center;">
                            <div>
                                <div style="font-weight: 600; color: #fff;">PCD — Proteus Certified Designer</div>
                                <div style="font-size: 0.75rem; color: var(--text-muted);">Σχεδιασμός οθονών, visual layouts και UX ροών.</div>
                            </div>
                            <span class="badge badge-green" style="font-size: 0.85rem;">79 €</span>
                        </div>
                    </div>

                    <div style="background: var(--bg-base); border: 1px solid var(--border); border-radius: var(--radius); padding: 0.85rem 1rem;">
                        <div style="display: flex; justify-content: space-between; align-items: center;">
                            <div>
                                <div style="font-weight: 600; color: #fff;">PCSS — Systems & DB Specialist</div>
                                <div style="font-size: 0.75rem; color: var(--text-muted);">Αρχιτεκτονική SQLite, DDL migrations και high-speed triggers.</div>
                            </div>
                            <span class="badge badge-green" style="font-size: 0.85rem;">79 €</span>
                        </div>
                    </div>

                    <div style="background: var(--bg-base); border: 1px solid var(--border); border-radius: var(--radius); padding: 0.85rem 1rem;">
                        <div style="display: flex; justify-content: space-between; align-items: center;">
                            <div>
                                <div style="font-weight: 600; color: #fff;">PCDS — Deployer / Support Specialist</div>
                                <div style="font-size: 0.75rem; color: var(--text-muted);">Εγκατάσταση δικτύων LAN, POS εκτυπωτών και SLA υποστήριξη.</div>
                            </div>
                            <span class="badge badge-green" style="font-size: 0.85rem;">79 €</span>
                        </div>
                    </div>

                    <div style="background: var(--bg-base); border: 1px solid var(--border); border-radius: var(--radius); padding: 0.85rem 1rem;">
                        <div style="display: flex; justify-content: space-between; align-items: center;">
                            <div>
                                <div style="font-weight: 600; color: #fff;">PCDA — Business & Data Analyst</div>
                                <div style="font-size: 0.75rem; color: var(--text-muted);">Data pipelines, KPI visual reporting και επιχειρησιακά dashboards.</div>
                            </div>
                            <span class="badge badge-green" style="font-size: 0.85rem;">79 €</span>
                        </div>
                    </div>

                    <div style="background: rgba(168, 85, 247, 0.08); border: 1px solid rgba(168, 85, 247, 0.3); border-radius: var(--radius); padding: 0.85rem 1rem;">
                        <div style="display: flex; justify-content: space-between; align-items: center;">
                            <div>
                                <div style="font-weight: 700; color: #c084fc;">Proteus Master Bundle (Όλα τα 4)</div>
                                <div style="font-size: 0.75rem; color: var(--text-muted);">Πλήρης πιστοποίηση PCD + PCSS + PCDS + PCDA με έκπτωση.</div>
                            </div>
                            <span class="badge badge-purple" style="font-size: 0.9rem; font-weight: 700;">149 €</span>
                        </div>
                    </div>
                </div>
            </div>

            <!-- 10-Tier Commission Calculator -->
            <div class="card">
                <div class="card-header">
                    <span>Κλιμακωτή Προμήθεια Συνεργατών (10-Tier Scale)</span>
                    <span class="badge badge-blue">Doc 18 Sec 2</span>
                </div>

                <div class="field-group">
                    <label>
                        <span>Μηνιαίος Τζίρος Έργων: <strong id="partner-gmv-label">3,000 €</strong></span>
                    </label>
                    <input type="range" id="partner-gmv-slider" min="500" max="25000" step="250" value="3000" oninput="updatePartnerPayout()">
                </div>

                <div class="field-group">
                    <label>
                        <span>Επίπεδο Συνεργάτη (Tier):</span>
                    </label>
                    <select id="partner-tier-select" onchange="updatePartnerPayout()" style="background: var(--bg-base); color: #fff; border: 1px solid var(--border); padding: 0.5rem; border-radius: 6px; font-family: inherit; font-size: 0.85rem;">
                        <option value="0">Tier 0: Μη Πιστοποιημένος (50% προμήθεια)</option>
                        <option value="1">Tier 1: Associate (40% προμήθεια - 30€/μήνα)</option>
                        <option value="2">Tier 2: Specialist (35% προμήθεια - 30€/μήνα)</option>
                        <option value="3" selected>Tier 3: Professional (30% προμήθεια - 30€/μήνα)</option>
                        <option value="4">Tier 4: Practitioner (25% προμήθεια - 30€/μήνα)</option>
                        <option value="5">Tier 5: Architect (20% προμήθεια - 30€/μήνα)</option>
                        <option value="6">Tier 6: Principal (16% προμήθεια - 45€/μήνα)</option>
                        <option value="7">Tier 7: Master Engineer (12% προμήθεια - 60€/μήνα)</option>
                        <option value="8">Tier 8: Premier Partner (10% προμήθεια - 80€/μήνα)</option>
                        <option value="9">Tier 9: Elite Agency (7% προμήθεια - 110€/μήνα)</option>
                        <option value="10">Tier 10: Enterprise Agency (4.5% Stripe cost - 150€/μήνα)</option>
                    </select>
                </div>

                <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 0.75rem;">
                    <div style="background: var(--bg-base); border: 1px solid var(--border); border-radius: var(--radius); padding: 0.85rem;">
                        <div style="font-size: 0.72rem; color: var(--text-muted); text-transform: uppercase;">Καθαρα Συνεργατη</div>
                        <div style="font-size: 1.35rem; font-weight: 700; color: var(--success);" id="partner-net-val">2,100 €</div>
                        <div style="font-size: 0.72rem; color: var(--text-muted);" id="partner-share-pct-label">70.0% μερίδιο</div>
                    </div>
                    <div style="background: var(--bg-base); border: 1px solid var(--border); border-radius: var(--radius); padding: 0.85rem;">
                        <div style="font-size: 0.72rem; color: var(--text-muted); text-transform: uppercase;">Προμηθεια Πλατφορμας</div>
                        <div style="font-size: 1.35rem; font-weight: 700; color: #fff;" id="platform-take-val">900 €</div>
                        <div style="font-size: 0.72rem; color: var(--text-muted);" id="platform-commission-label">30.0% take-rate</div>
                    </div>
                </div>

                <div style="background: rgba(16, 185, 129, 0.08); border: 1px solid rgba(16, 185, 129, 0.25); border-radius: var(--radius); padding: 0.75rem 1rem; display: flex; justify-content: space-between; align-items: center;">
                    <span style="font-size: 0.82rem; color: var(--text-main);">Ετήσιο Όφελος Έναντι Tier 0:</span>
                    <strong style="color: var(--success); font-size: 1rem;" id="annual-savings-val">+7,200 € / έτος</strong>
                </div>
            </div>
        </div>
    </div>
"#;

pub const CERTIFICATIONS_JS: &str = r#"
    const TIER_DATA = {
        0:  { name: 'Tier 0 Uncertified', take: 50.0, fee: 0 },
        1:  { name: 'Tier 1 Associate', take: 40.0, fee: 30 },
        2:  { name: 'Tier 2 Specialist', take: 35.0, fee: 30 },
        3:  { name: 'Tier 3 Professional', take: 30.0, fee: 30 },
        4:  { name: 'Tier 4 Practitioner', take: 25.0, fee: 30 },
        5:  { name: 'Tier 5 Architect', take: 20.0, fee: 30 },
        6:  { name: 'Tier 6 Principal', take: 16.0, fee: 45 },
        7:  { name: 'Tier 7 Master Engineer', take: 12.0, fee: 60 },
        8:  { name: 'Tier 8 Premier Partner', take: 10.0, fee: 80 },
        9:  { name: 'Tier 9 Elite Agency', take: 7.0, fee: 110 },
        10: { name: 'Tier 10 Enterprise Agency', take: 4.5, fee: 150 },
    };

    function updatePartnerPayout() {
        const gmv = parseFloat(document.getElementById('partner-gmv-slider').value);
        const tierIdx = parseInt(document.getElementById('partner-tier-select').value, 10);
        const tier = TIER_DATA[tierIdx] || TIER_DATA[0];

        document.getElementById('partner-gmv-label').textContent = gmv.toLocaleString('el-GR') + ' €';

        const platformTake = (gmv * (tier.take / 100));
        const partnerNet = gmv - platformTake;
        const partnerShare = 100.0 - tier.take;

        document.getElementById('partner-net-val').textContent = partnerNet.toFixed(2) + ' €';
        document.getElementById('platform-take-val').textContent = platformTake.toFixed(2) + ' €';
        document.getElementById('partner-share-pct-label').textContent = partnerShare.toFixed(1) + '% μερίδιο';
        document.getElementById('platform-commission-label').textContent = tier.take.toFixed(1) + '% take-rate';

        // Calculate annual savings compared to Tier 0
        const tier0Net = gmv * 0.50;
        const monthlyAdvantage = partnerNet - tier0Net;
        const annualSavings = (monthlyAdvantage * 12) - (tier.fee * 12);
        document.getElementById('annual-savings-val').textContent = (annualSavings >= 0 ? '+' : '') + annualSavings.toFixed(0) + ' € / έτος';
    }
"#;
