//! Live Fleet & Telemetry Radar UI Template for Proteus Web.
//! Showcases real-time AIS maritime and ADS-B aviation geolocation with dead-reckoning extrapolation.

pub const TELEMETRY_HTML: &str = r#"
    <div class="tab-panel" id="panel-telemetry">
        <div class="section-header">
            <h2 class="section-title">🛰 Live Fleet & Telemetry Radar</h2>
            <p class="section-sub">Πολυμορφική παρακολούθηση στόλου σε πραγματικό χρόνο: Ναυτιλία (AIS VHF), Αεροπορία (ADS-B 1090MHz) και Offline Dead-Reckoning.</p>
        </div>

        <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 1.25rem; flex-wrap: wrap; gap: 0.75rem;">
            <div style="display: flex; gap: 0.5rem;" id="telemetry-filters">
                <button class="btn btn-sm active" onclick="filterTelemetry('all', this)">Όλα τα Σκάφη & Αεροσκάφη</button>
                <button class="btn btn-secondary btn-sm" onclick="filterTelemetry('maritime', this)">🚢 Ναυτιλία (AIS)</button>
                <button class="btn btn-secondary btn-sm" onclick="filterTelemetry('aviation', this)">✈️ Αεροπορία (ADS-B)</button>
                <button class="btn btn-secondary btn-sm" onclick="filterTelemetry('dead-reckon', this)">📡 Dead-Reckoning (Offline)</button>
            </div>
            <div style="display: flex; gap: 0.5rem; align-items: center;">
                <span class="badge badge-green" id="telemetry-pulse-badge">● Live RF Ingestion (0ms Latency)</span>
                <button class="btn btn-secondary btn-sm" onclick="runDeadReckoningStep()">⚡ Προσομοίωση 60s Dead-Reckoning</button>
            </div>
        </div>

        <div class="grid-2" id="fleet-cards-container">
            <!-- 1. Passenger Ferry -->
            <div class="pkg-card telemetry-card" data-sector="maritime" data-reckon="live">
                <div class="pkg-top">
                    <div>
                        <div class="pkg-title">Blue Star Delos &mdash; Επιβατηγό / Οχηματαγωγό (Ferry)</div>
                        <div class="pkg-meta" style="margin-top: 0.35rem;">
                            <span class="badge badge-blue">Ναυτιλία (Ακτοπλοΐα)</span>
                            <span class="badge badge-gray">MMSI: 239123400</span>
                            <span class="badge badge-green">AIS VHF Class A</span>
                        </div>
                    </div>
                    <div style="text-align: right;">
                        <div style="font-size: 1.25rem; font-weight: 700; color: var(--text-primary);"><span class="speed-val">24.2</span> kn</div>
                        <div style="font-size: 0.72rem; color: var(--text-muted);">Ταχύτητα (SOG)</div>
                    </div>
                </div>
                <div style="background: var(--bg-base); border: 1px solid var(--border); border-radius: var(--radius); padding: 0.85rem; font-size: 0.85rem; display: flex; flex-direction: column; gap: 0.4rem;">
                    <div><strong>Συντεταγμένες:</strong> <span class="coord-lat">37.4415° N</span>, <span class="coord-lon">25.3284° E</span> (Κυκλάδες)</div>
                    <div><strong>Πορεία (COG):</strong> <span class="heading-val">118°</span> (Νοτιοανατολικά) &bull; <strong>Βύθισμα:</strong> 5.8 m</div>
                    <div><strong>Προορισμός / ETA:</strong> Πειραιάς &rarr; Πάρος - Νάξος (ETA 16:30 UTC)</div>
                </div>
            </div>

            <!-- 2. Container Cargo Ship -->
            <div class="pkg-card telemetry-card" data-sector="maritime" data-reckon="live">
                <div class="pkg-top">
                    <div>
                        <div class="pkg-title">Aegean Transporter &mdash; Εμπορευματοκιβώτια (Container 4,200 TEU)</div>
                        <div class="pkg-meta" style="margin-top: 0.35rem;">
                            <span class="badge badge-blue">Ναυτιλία (Φορτηγό)</span>
                            <span class="badge badge-gray">IMO: 9481203</span>
                            <span class="badge badge-green">AIS VHF Class A</span>
                        </div>
                    </div>
                    <div style="text-align: right;">
                        <div style="font-size: 1.25rem; font-weight: 700; color: var(--text-primary);"><span class="speed-val">18.5</span> kn</div>
                        <div style="font-size: 0.72rem; color: var(--text-muted);">Ταχύτητα (SOG)</div>
                    </div>
                </div>
                <div style="background: var(--bg-base); border: 1px solid var(--border); border-radius: var(--radius); padding: 0.85rem; font-size: 0.85rem; display: flex; flex-direction: column; gap: 0.4rem;">
                    <div><strong>Συντεταγμένες:</strong> <span class="coord-lat">36.1204° N</span>, <span class="coord-lon">23.8540° E</span> (Στενό Κυθήρων)</div>
                    <div><strong>Πορεία (COG):</strong> <span class="heading-val">210°</span> (Νοτιοδυτικά) &bull; <strong>Φορτίο:</strong> 3,840 TEU</div>
                    <div><strong>Προορισμός / ETA:</strong> Πειραιάς &rarr; Gioia Tauro (ETA 22:00 UTC)</div>
                </div>
            </div>

            <!-- 3. LNG Tanker -->
            <div class="pkg-card telemetry-card" data-sector="maritime" data-reckon="live">
                <div class="pkg-top">
                    <div>
                        <div class="pkg-title">Hellas Pioneer &mdash; Υγραεριοφόρο / LNG Tanker (174,000 m³)</div>
                        <div class="pkg-meta" style="margin-top: 0.35rem;">
                            <span class="badge badge-purple">Ναυτιλία (Γκαζάδικο)</span>
                            <span class="badge badge-gray">IMO: 9763321</span>
                            <span class="badge badge-green">AIS VHF Class A</span>
                        </div>
                    </div>
                    <div style="text-align: right;">
                        <div style="font-size: 1.25rem; font-weight: 700; color: var(--text-primary);"><span class="speed-val">11.2</span> kn</div>
                        <div style="font-size: 0.72rem; color: var(--text-muted);">Ταχύτητα (SOG)</div>
                    </div>
                </div>
                <div style="background: var(--bg-base); border: 1px solid var(--border); border-radius: var(--radius); padding: 0.85rem; font-size: 0.85rem; display: flex; flex-direction: column; gap: 0.4rem;">
                    <div><strong>Συντεταγμένες:</strong> <span class="coord-lat">37.9100° N</span>, <span class="coord-lon">23.4150° E</span> (Κόλπος Μεγάρων / Ρεβυθούσα)</div>
                    <div><strong>Πορεία (COG):</strong> <span class="heading-val">045°</span> &bull; <strong>Πίεση IGS:</strong> 104 kPa &bull; <strong>Θερμοκρασία Δεξαμενών:</strong> -161.5°C</div>
                    <div><strong>Προορισμός / ETA:</strong> Τερματικός Σταθμός Ρεβυθούσας (Πρόσδεση σε εξέλιξη)</div>
                </div>
            </div>

            <!-- 4. Fishing Caique (Offline Dead Reckoning) -->
            <div class="pkg-card telemetry-card" data-sector="maritime" data-reckon="offline">
                <div class="pkg-top">
                    <div>
                        <div class="pkg-title">Άγιος Νικόλαος &mdash; Παραδοσιακό Αλιευτικό / Καΐκι</div>
                        <div class="pkg-meta" style="margin-top: 0.35rem;">
                            <span class="badge badge-amber">Αλιεία (Καΐκι)</span>
                            <span class="badge badge-gray">Νηολόγιο: ΚΑΛΥΜΝΟΣ-412</span>
                            <span class="badge badge-purple">Dead-Reckoning (Εκτός Κάλυψης)</span>
                        </div>
                    </div>
                    <div style="text-align: right;">
                        <div style="font-size: 1.25rem; font-weight: 700; color: var(--warning);"><span class="speed-val">7.8</span> kn</div>
                        <div style="font-size: 0.72rem; color: var(--text-muted);">Εκτίμηση Ταχύτητας</div>
                    </div>
                </div>
                <div style="background: var(--bg-base); border: 1px solid var(--border); border-radius: var(--radius); padding: 0.85rem; font-size: 0.85rem; display: flex; flex-direction: column; gap: 0.4rem;">
                    <div><strong>Προβλεπόμενη Θέση (Dead-Reckoning):</strong> <span class="coord-lat">36.9820° N</span>, <span class="coord-lon">27.0540° E</span> (Ανατολικά Καλύμνου)</div>
                    <div><strong>Πορεία:</strong> <span class="heading-val">310°</span> (Βορειοδυτικά) &bull; <strong>Ώρες Κινητήρα Diesel:</strong> 1,420 h</div>
                    <div><strong>Κατάσταση:</strong> Επιστροφή στο λιμάνι &bull; Τελευταίο RF σήμα πριν από 18 λεπτά</div>
                </div>
            </div>

            <!-- 5. Commercial Passenger Jet -->
            <div class="pkg-card telemetry-card" data-sector="aviation" data-reckon="live">
                <div class="pkg-top">
                    <div>
                        <div class="pkg-title">Airbus A320neo &mdash; Επιβατικό Jet (Aegean Airlines)</div>
                        <div class="pkg-meta" style="margin-top: 0.35rem;">
                            <span class="badge badge-blue">Αεροπορία (Commercial)</span>
                            <span class="badge badge-gray">Tail: SX-NEA &bull; A3-314</span>
                            <span class="badge badge-green">ADS-B 1090 MHz</span>
                        </div>
                    </div>
                    <div style="text-align: right;">
                        <div style="font-size: 1.25rem; font-weight: 700; color: var(--text-primary);"><span class="speed-val">440</span> kn</div>
                        <div style="font-size: 0.72rem; color: var(--text-muted);">Υψόμετρο: 28,000 ft</div>
                    </div>
                </div>
                <div style="background: var(--bg-base); border: 1px solid var(--border); border-radius: var(--radius); padding: 0.85rem; font-size: 0.85rem; display: flex; flex-direction: column; gap: 0.4rem;">
                    <div><strong>Συντεταγμένες:</strong> <span class="coord-lat">38.2510° N</span>, <span class="coord-lon">23.9100° E</span> (Πτήση FL280)</div>
                    <div><strong>Πορεία (Track):</strong> <span class="heading-val">195°</span> &bull; <strong>Κάθετη Ταχύτητα:</strong> 0 fpm (Level)</div>
                    <div><strong>Διαδρομή / ETA:</strong> SKG (Θεσσαλονίκη) &rarr; ATH (Ελ. Βενιζέλος) (ETA 14:45 UTC)</div>
                </div>
            </div>

            <!-- 6. Regional Turboprop Cargo Aircraft -->
            <div class="pkg-card telemetry-card" data-sector="aviation" data-reckon="live">
                <div class="pkg-top">
                    <div>
                        <div class="pkg-title">ATR 72-600 &mdash; Περιφερειακό / Cargo (Sky Express)</div>
                        <div class="pkg-meta" style="margin-top: 0.35rem;">
                            <span class="badge badge-blue">Αεροπορία (Cargo / Island Hopping)</span>
                            <span class="badge badge-gray">Tail: SX-SEV &bull; GQ-210</span>
                            <span class="badge badge-green">ADS-B 1090 MHz</span>
                        </div>
                    </div>
                    <div style="text-align: right;">
                        <div style="font-size: 1.25rem; font-weight: 700; color: var(--text-primary);"><span class="speed-val">235</span> kn</div>
                        <div style="font-size: 0.72rem; color: var(--text-muted);">Υψόμετρο: 14,000 ft</div>
                    </div>
                </div>
                <div style="background: var(--bg-base); border: 1px solid var(--border); border-radius: var(--radius); padding: 0.85rem; font-size: 0.85rem; display: flex; flex-direction: column; gap: 0.4rem;">
                    <div><strong>Συντεταγμένες:</strong> <span class="coord-lat">37.1500° N</span>, <span class="coord-lon">25.1000° E</span> (Νότιο Αιγαίο)</div>
                    <div><strong>Πορεία (Track):</strong> <span class="heading-val">082°</span> &bull; <strong>Κάθετη Ταχύτητα:</strong> -450 fpm (Κάθοδος)</div>
                    <div><strong>Διαδρομή / ETA:</strong> ATH &rarr; JTR (Σαντορίνη) (ETA 15:10 UTC)</div>
                </div>
            </div>
        </div>
    </div>
"#;

pub const TELEMETRY_JS: &str = r#"
    function filterTelemetry(category, btn) {
        document.querySelectorAll('#telemetry-filters button').forEach(b => b.classList.remove('active'));
        if (btn) btn.classList.add('active');

        const cards = document.querySelectorAll('#fleet-cards-container .telemetry-card');
        cards.forEach(card => {
            const sector = card.getAttribute('data-sector');
            const reckon = card.getAttribute('data-reckon');

            if (category === 'all') {
                card.style.display = 'flex';
            } else if (category === 'maritime' && sector === 'maritime') {
                card.style.display = 'flex';
            } else if (category === 'aviation' && sector === 'aviation') {
                card.style.display = 'flex';
            } else if (category === 'dead-reckon' && reckon === 'offline') {
                card.style.display = 'flex';
            } else {
                card.style.display = 'none';
            }
        });
    }

    function runDeadReckoningStep() {
        const cards = document.querySelectorAll('#fleet-cards-container .telemetry-card');
        cards.forEach(card => {
            const speedEl = card.querySelector('.speed-val');
            const headingEl = card.querySelector('.heading-val');
            const latEl = card.querySelector('.coord-lat');
            const lonEl = card.querySelector('.coord-lon');

            if (speedEl && headingEl && latEl && lonEl) {
                const speed = parseFloat(speedEl.textContent);
                const heading = parseFloat(headingEl.textContent);
                let lat = parseFloat(latEl.textContent);
                let lon = parseFloat(lonEl.textContent);

                // Spherical dead-reckoning extrapolation for 60 seconds (1 minute):
                // dist_nm = speed * (60 / 3600)
                const distNm = speed * (60.0 / 3600.0);
                const headingRad = (heading * Math.PI) / 180.0;
                const dLat = (distNm * Math.cos(headingRad)) / 60.0;
                const dLon = (distNm * Math.sin(headingRad)) / (60.0 * Math.cos((lat * Math.PI) / 180.0));

                lat += dLat;
                lon += dLon;

                latEl.textContent = lat.toFixed(4) + '° N';
                lonEl.textContent = lon.toFixed(4) + '° E';
            }
        });

        showToast("⚡ Εκτελέστηκε προσομοίωση Dead-Reckoning 60s! Οι συντεταγμένες ενημερώθηκαν.", "info");
    }
"#;
