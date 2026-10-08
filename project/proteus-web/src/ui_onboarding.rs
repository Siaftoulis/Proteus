//! Interactive Business Discovery Questionnaire & Enterprise Solution Architect Desk
//! Handles SME automated starter scaffolding, verified RFP tenders, and Enterprise Custom Blueprint routing.

pub const ONBOARDING_HTML: &str = r#"
    <!-- Tab: Onboarding & Account Registration Gate -->
    <div class="tab-panel" id="panel-onboarding">
        <div class="section-header">
            <h2 class="section-title">Καλωσήρθατε στο Proteus Sovereign OS</h2>
            <p class="section-sub">Επιλέξτε την ιδιότητά σας για αυτόματη δημιουργία συστήματος ή εξατομικευμένη αρχιτεκτονική μελέτη.</p>
        </div>

        <!-- Role Selection Cards -->
        <div class="grid-3" id="onboarding-role-selection" style="margin-bottom: 1.5rem;">
            <div class="card" style="cursor: pointer; border-color: var(--border-strong);" onclick="selectOnboardingRole('owner')">
                <div class="card-header">
                    <span style="font-weight: 700;">🏪 Μικρομεσαία Επιχείρηση (SME)</span>
                    <span class="badge badge-blue">Operator Path</span>
                </div>
                <p style="font-size: 0.82rem; color: var(--text-muted); line-height: 1.4; margin-top: 0.4rem;">
                    Έτοιμο σύστημα για καταστήματα & εργαστήρια (Ταμείο POS, myDATA, Service, ΕΡΓΑΝΗ ΙΙ). Άμεσο starter setup ή διαγωνισμός RFP.
                </p>
                <div style="margin-top: 0.75rem;">
                    <button class="btn btn-sm" style="width: 100%;">Οδηγός SME (1-15 άτομα) →</button>
                </div>
            </div>

            <div class="card" style="cursor: pointer; border-color: var(--accent);" onclick="selectOnboardingRole('enterprise')">
                <div class="card-header">
                    <span style="font-weight: 700;">🏢 Μεγάλη Επιχείρηση / Enterprise</span>
                    <span class="badge badge-purple">Solutions Desk</span>
                </div>
                <p style="font-size: 0.82rem; color: var(--text-muted); line-height: 1.4; margin-top: 0.4rem;">
                    Πολυκαταστήματα (>15 θέσεις) & μετάβαση από SoftOne/SAP. Επικοινωνία με πιστοποιημένο Solution Architect για custom πλάνο & εκπαίδευση.
                </p>
                <div style="margin-top: 0.75rem;">
                    <button class="btn btn-secondary btn-sm" style="width: 100%;">Contact Enterprise Sales →</button>
                </div>
            </div>

            <div class="card" style="cursor: pointer;" onclick="selectOnboardingRole('designer')">
                <div class="card-header">
                    <span style="font-weight: 700;">🎨 Σχεδιαστής / IT Partner</span>
                    <span class="badge badge-gray">Builder Path</span>
                </div>
                <p style="font-size: 0.82rem; color: var(--text-muted); line-height: 1.4; margin-top: 0.4rem;">
                    Σχεδίαση οθονών CRM, ανάληψη έργων επιχειρήσεων με εγγύηση SLA Escrow και πιστοποίηση στο Proteus Design Studio.
                </p>
                <div style="margin-top: 0.75rem;">
                    <button class="btn btn-secondary btn-sm" style="width: 100%;">Designer Portal →</button>
                </div>
            </div>
        </div>

        <!-- Enterprise Solutions Architect Desk Form -->
        <div id="onboarding-enterprise" class="card" style="display: none; border-color: var(--accent); margin-bottom: 1.5rem;">
            <div class="card-header">
                <span style="font-size: 1.05rem; font-weight: 700;">🏛️ Enterprise Sales & Certified Solutions Architect Desk</span>
                <span class="badge badge-purple">Bespoke SLA Architecture</span>
            </div>
            <p style="font-size: 0.85rem; color: var(--text-muted); margin-top: 0.5rem; line-height: 1.5;">
                Για μεγάλες επιχειρήσεις και ομίλους, ένας πιστοποιημένος Freelance Solution Architect αναλαμβάνει τη σύνταξη custom blueprint, 
                τη στελέχωση του έργου με πιστοποιημένους επαγγελματίες ή την εκπαίδευση του εσωτερικού προσωπικού σας μέσω του Proteus Academy.
            </p>
            <div class="grid-2" style="margin-top: 1rem;">
                <div class="field-group">
                    <label>Επωνυμία Επιχείρησης / Ομίλου</label>
                    <input type="text" id="ent-biz-name" placeholder="π.χ. Delta Retail Group A.E." style="background: var(--bg-base); color: #fff; border: 1px solid var(--border); padding: 0.5rem; border-radius: 6px; font-size: 0.85rem;">
                </div>
                <div class="field-group">
                    <label>Υπεύθυνος Επικοινωνίας & Θέση</label>
                    <input type="text" id="ent-contact-person" placeholder="π.χ. Νίκος Αντωνίου (IT Director)" style="background: var(--bg-base); color: #fff; border: 1px solid var(--border); padding: 0.5rem; border-radius: 6px; font-size: 0.85rem;">
                </div>
                <div class="field-group">
                    <label>Εταιρικό Email & Τηλέφωνο</label>
                    <input type="text" id="ent-contact-info" placeholder="director@deltagroup.gr | 210-..." style="background: var(--bg-base); color: #fff; border: 1px solid var(--border); padding: 0.5rem; border-radius: 6px; font-size: 0.85rem;">
                </div>
                <div class="field-group">
                    <label>Αριθμός Υποκαταστημάτων / Θέσεων Εργασίας</label>
                    <input type="number" id="ent-branches" value="16" min="15" style="background: var(--bg-base); color: #fff; border: 1px solid var(--border); padding: 0.5rem; border-radius: 6px; font-size: 0.85rem;">
                </div>
                <div class="field-group">
                    <label>Υφιστάμενο ERP προς Μετάβαση (Legacy System)</label>
                    <input type="text" id="ent-legacy-erp" placeholder="π.χ. SoftOne, SAP, Entersoft, SingularLogic" style="background: var(--bg-base); color: #fff; border: 1px solid var(--border); padding: 0.5rem; border-radius: 6px; font-size: 0.85rem;">
                </div>
                <div class="field-group">
                    <label>Προτιμώμενο Μοντέλο Εκτέλεσης & Στελέχωσης</label>
                    <select id="ent-execution-model" style="background: var(--bg-base); color: #fff; border: 1px solid var(--border); padding: 0.5rem; border-radius: 6px; font-size: 0.85rem;">
                        <option value="freelance_staffing">Στελέχωση με Πιστοποιημένους Solution Architects & Designers (SLA Escrow)</option>
                        <option value="in_house_academy">Πρόσληψη & Εκπαίδευση In-house Προσωπικού μέσω του Proteus Academy</option>
                        <option value="hybrid">Υβριδικό Μοντέλο (Επίβλεψη από Freelance Architect + Co-building με In-house ομάδα)</option>
                    </select>
                </div>
            </div>
            <div style="display: flex; justify-content: flex-end; gap: 0.5rem; margin-top: 1rem;">
                <button class="btn btn-secondary btn-sm" onclick="selectOnboardingRole('owner')">← Μετάβαση σε Οδηγό SME</button>
                <button class="btn btn-sm" onclick="submitEnterpriseInquiry()">Αποστολή Αιτήματος στον Solution Architect →</button>
            </div>
        </div>

        <!-- SME Questionnaire Container -->
        <div id="onboarding-questionnaire" class="card" style="display: none; border-color: var(--border-strong);">
            <div class="card-header">
                <span id="wizard-step-title">Οδηγός Επιχείρησης — Βήμα 1 από 4: Κλάδος & Εξειδίκευση</span>
                <span class="badge badge-blue" id="wizard-progress-badge">Βήμα 1/4</span>
            </div>

            <!-- Step 1: Sector & Dynamic Subcategories -->
            <div id="wizard-step-1">
                <div class="grid-2">
                    <div class="field-group">
                        <label>Επωνυμία Επιχείρησης / Καταστήματος</label>
                        <input type="text" id="biz-name" placeholder="π.χ. TechFix Lab, Artisan Bakery" style="background: var(--bg-base); color: #fff; border: 1px solid var(--border); padding: 0.5rem; border-radius: 6px; font-size: 0.85rem;">
                    </div>
                    <div class="field-group">
                        <label>Κύριος Τομέας Δραστηριότητας</label>
                        <select id="biz-sector" onchange="updateSubcategoryDropdown()" style="background: var(--bg-base); color: #fff; border: 1px solid var(--border); padding: 0.5rem; border-radius: 6px; font-size: 0.85rem;">
                            <option value="TechnologyAndRepairs">Τεχνολογικό / Επισκευές & Τεχνικά Επαγγέλματα</option>
                            <option value="RetailAndCommerce">Εμπορικό / Λιανική & Χονδρική</option>
                            <option value="HealthcareAndMedical">Ιατρικό / Υγεία & Φροντίδα</option>
                            <option value="HospitalityAndFood">Εστίαση, Καφέ & Φιλοξενία</option>
                            <option value="ServicesAndOffices">Υπηρεσίες, Γραφεία & Ελεύθερα Επαγγέλματα</option>
                            <option value="PersonalCareAndWellness">Προσωπική Φροντίδα, Κομμωτήρια & Ευεξία</option>
                            <option value="CraftsAndManufacturing">Βιοτεχνία, Εργαστήρια & Παραγωγή</option>
                        </select>
                    </div>
                    <div class="field-group">
                        <label>Εξειδικευμένος Υποκλάδος (Subcategory)</label>
                        <select id="biz-subcategory" style="background: var(--bg-base); color: #fff; border: 1px solid var(--border); padding: 0.5rem; border-radius: 6px; font-size: 0.85rem;"></select>
                    </div>
                    <div class="field-group">
                        <label>Κλίμακα & Θέσεις Εργασίας</label>
                        <select id="biz-scale" onchange="checkScaleForEnterprise()" style="background: var(--bg-base); color: #fff; border: 1px solid var(--border); padding: 0.5rem; border-radius: 6px; font-size: 0.85rem;">
                            <option value="solo">Ατομική Επιχείρηση (1 άτομο)</option>
                            <option value="small">Μικρή Ομάδα (2 - 5 εργαζόμενοι)</option>
                            <option value="medium">Μεσαίο Κατάστημα (6 - 15 εργαζόμενοι)</option>
                            <option value="enterprise">Μεγάλη Επιχείρηση (> 15 εργαζόμενοι / Πολυκατάστημα)</option>
                        </select>
                    </div>
                </div>
                <div style="display: flex; justify-content: flex-end; margin-top: 1rem;">
                    <button class="btn btn-sm" onclick="advanceWizard(2)">Επόμενο: Φορολογικά & myDATA →</button>
                </div>
            </div>

            <!-- Step 2: Tax & myDATA & EFTPOS -->
            <div id="wizard-step-2" style="display: none;">
                <p style="font-size: 0.85rem; color: var(--text-muted); margin-bottom: 0.75rem;">
                    Επιλέξτε τις φορολογικές & ταμειακές λειτουργίες που απαιτεί η επιχείρησή σας:
                </p>
                <div style="display: flex; flex-direction: column; gap: 0.5rem;">
                    <label class="checkbox-row">
                        <input type="checkbox" id="biz-mydata" checked>
                        <span>Απευθείας διασύνδεση με ΑΑΔΕ myDATA REST API (0€ κόστος παρόχου)</span>
                    </label>
                    <label class="checkbox-row">
                        <input type="checkbox" id="biz-eftpos" checked>
                        <span>Διασύνδεση Ταμειακής με τερματικό EFT-POS (Απόφαση Α.1155/2023)</span>
                    </label>
                    <label class="checkbox-row">
                        <input type="checkbox" id="biz-b2b" checked>
                        <span>Έκδοση Τιμολογίων B2B και Δελτίων Αποστολής e-CMR</span>
                    </label>
                </div>
                <div style="display: flex; justify-content: space-between; margin-top: 1rem;">
                    <button class="btn btn-secondary btn-sm" onclick="advanceWizard(1)">← Πίσω</button>
                    <button class="btn btn-sm" onclick="advanceWizard(3)">Επόμενο: Ροές & Προσωπικό →</button>
                </div>
            </div>

            <!-- Step 3: Operational Workflows & Ergani II -->
            <div id="wizard-step-3" style="display: none;">
                <p style="font-size: 0.85rem; color: var(--text-muted); margin-bottom: 0.75rem;">
                    Εξειδικευμένες λειτουργίες ροής εργασίας & ρυθμίσεις προσωπικού:
                </p>
                <div style="display: flex; flex-direction: column; gap: 0.5rem;">
                    <label class="checkbox-row">
                        <input type="checkbox" id="biz-voucher" checked>
                        <span>Δελτίο Παραλαβής & Αυτόματη Θερμική Εκτύπωση ESC/POS (80mm/58mm)</span>
                    </label>
                    <label class="checkbox-row">
                        <input type="checkbox" id="biz-inventory" checked>
                        <span>Ιχνηλασιμότητα Barcode/Serial/IMEI & Αποθήκη Ανταλλακτικών</span>
                    </label>
                    <label class="checkbox-row">
                        <input type="checkbox" id="biz-ergani" checked>
                        <span>Ψηφιακή Κάρτα Εργασίας ΕΡΓΑΝΗ ΙΙ (Kiosk αφής & κρυπτογραφικό Merkle buffer)</span>
                    </label>
                    <label class="checkbox-row">
                        <input type="checkbox" id="biz-sms" checked>
                        <span>Αυτόματη ειδοποίηση πελατών SMS / Viber για την εξέλιξη της εργασίας</span>
                    </label>
                </div>
                <div style="display: flex; justify-content: space-between; margin-top: 1rem;">
                    <button class="btn btn-secondary btn-sm" onclick="advanceWizard(2)">← Πίσω</button>
                    <button class="btn btn-sm" onclick="advanceWizard(4)">Επόμενο: Τρόπος Υλοποίησης →</button>
                </div>
            </div>

            <!-- Step 4: Execution Strategy -->
            <div id="wizard-step-4" style="display: none;">
                <div class="grid-2">
                    <div class="card" style="border-color: var(--border-strong); background: var(--bg-surface-sec);">
                        <div class="card-header">
                            <span style="font-weight: 600;">⚡ Αυτόματο Starter Setup (Δωρεάν)</span>
                            <span class="badge badge-gray">0.00 €</span>
                        </div>
                        <p style="font-size: 0.8rem; color: var(--text-muted); line-height: 1.4; margin-top: 0.4rem;">
                            Παράγει άμεσα έτοιμο σφραγισμένο πακέτο (.pr) με DDL & Views για τον κλάδο σας και σας δίνει το link λήψης του Proteus Client.
                        </p>
                        <div style="margin-top: 0.75rem;">
                            <button class="btn btn-sm" style="width: 100%;" onclick="completeOnboarding('solo')">Λήψη Έτοιμου Συστήματος →</button>
                        </div>
                    </div>

                    <div class="card" style="border-color: var(--accent); background: var(--bg-surface-sec);">
                        <div class="card-header">
                            <span style="font-weight: 600;">🤝 Ανάθεση σε Πιστοποιημένο Designer</span>
                            <span class="badge badge-blue">SLA Escrow</span>
                        </div>
                        <p style="font-size: 0.8rem; color: var(--text-muted); line-height: 1.4; margin-top: 0.4rem;">
                            Ανάρτηση αιτήματος RFP στο Freelancing Board με αυτόματο νομικό NDA. Οι πιστοποιημένοι designers σχεδιάζουν κατά παραγγελία.
                        </p>
                        <div class="field-group" style="margin-top: 0.5rem;">
                            <label style="font-size: 0.75rem;">Εκτιμώμενο Budget Σχεδίασης (EUR)</label>
                            <input type="number" id="biz-tender-budget" value="250" style="background: var(--bg-base); color: #fff; border: 1px solid var(--border); padding: 0.35rem; border-radius: 6px; font-size: 0.8rem;">
                        </div>
                        <div style="margin-top: 0.5rem;">
                            <button class="btn btn-sm" style="width: 100%;" onclick="completeOnboarding('hire')">Ανάρτηση Αιτήματος με NDA →</button>
                        </div>
                    </div>
                </div>
                <div style="display: flex; justify-content: flex-start; margin-top: 1rem;">
                    <button class="btn btn-secondary btn-sm" onclick="advanceWizard(3)">← Πίσω</button>
                </div>
            </div>
        </div>

        <!-- Software Download & License Card -->
        <div id="download-delivery-card" class="card" style="display: none; border-color: var(--success); margin-top: 1.5rem;">
            <div class="card-header">
                <span>✓ Το Σύστημά σας Δημιουργήθηκε Επιτυχώς!</span>
                <span class="badge badge-blue">Ready for Deployment</span>
            </div>
            <p style="font-size: 0.85rem; color: var(--text-muted); line-height: 1.5;">
                Το προ-ρυθμισμένο πακέτο καταστήματος <strong><span id="delivery-pkg-name">Starter CRM</span></strong> είναι έτοιμο. 
                Κατεβάστε την αυτόνομη εφαρμογή runtime (Windows / macOS / Linux) και εκτελέστε την τοπικά χωρίς εγκατάσταση.
            </p>
            <div style="display: flex; gap: 0.75rem; flex-wrap: wrap; margin-top: 1rem;">
                <button class="btn btn-sm" onclick="simulateDownload('Windows')">💻 Λήψη για Windows (Proteus.exe)</button>
                <button class="btn btn-secondary btn-sm" onclick="simulateDownload('macOS')">🍎 Λήψη για macOS (Mach-O)</button>
                <button class="btn btn-secondary btn-sm" onclick="simulateDownload('Linux')">🐧 Λήψη για Linux (AppImage)</button>
                <button class="btn btn-secondary btn-sm" onclick="simulateDownload('Android')">📱 Λήψη Android Companion (.apk)</button>
            </div>
        </div>
    </div>
"#;

pub const ONBOARDING_JS: &str = r#"
    const PROTEUS_TAXONOMY = {
        'TechnologyAndRepairs': [
            { id: 'tech_smartphones_pc', title: 'Επισκευές Smartphones, Laptops & Υπολογιστών' },
            { id: 'tech_auto_moto', title: 'Συνεργείο Αυτοκινήτων, Μοτοσυκλετών & Φανοποιείο' },
            { id: 'tech_appliances', title: 'Επισκευές Οικιακών & Επαγγελματικών Συσκευών' },
            { id: 'tech_electronics_automation', title: 'Ηλεκτρολογικά, Συναγερμοί & Αυτοματισμοί' }
        ],
        'RetailAndCommerce': [
            { id: 'retail_fashion', title: 'Ένδυση, Υπόδηση & Αξεσουάρ (Μεγέθη/Χρώματα)' },
            { id: 'retail_minimarket', title: 'Μίνι Μάρκετ, Ψιλικά, Περίπτερο & Fast POS' },
            { id: 'retail_warehouse_vmi', title: 'Χονδρικό Εμπόριο, Αποθήκη WMS & Logistics' },
            { id: 'retail_jewelry_gifts', title: 'Κοσμηματοπωλείο, Είδη Δώρων & Οπτικά' }
        ],
        'HealthcareAndMedical': [
            { id: 'health_dental', title: 'Οδοντιατρείο & Οδοντοτεχνικό Εργαστήριο' },
            { id: 'health_clinic', title: 'Ιδιωτικό Ιατρείο, Διαγνωστικό & Πολυϊατρείο' },
            { id: 'health_vet', title: 'Κτηνιατρείο, Pet Care & Κλινική Ζώων' },
            { id: 'health_physio', title: 'Φυσικοθεραπευτήριο & Κέντρο Αποκατάστασης' }
        ],
        'HospitalityAndFood': [
            { id: 'food_cafe_takeaway', title: 'Καφέ, Takeaway & Delivery (Touch POS)' },
            { id: 'food_bakery', title: 'Αρτοποιείο, Ζαχαροπλαστείο & Catering' },
            { id: 'food_restaurant', title: 'Εστιατόριο & Ταβέρνα (Τραπέζια & Παραγγελιοληψία)' },
            { id: 'food_hotel_lodging', title: 'Ξενοδοχείο, Ενοικιαζόμενα Δωμάτια & Lodging' }
        ],
        'ServicesAndOffices': [
            { id: 'srv_accounting', title: 'Λογιστικό & Φοροτεχνικό Γραφείο' },
            { id: 'srv_legal', title: 'Δικηγορικό Γραφείο & Νομικές Υπηρεσίες' },
            { id: 'srv_education', title: 'Φροντιστήριο, Κέντρο Ξένων Γλωσσών & Εκπαίδευση' },
            { id: 'srv_realestate_engineering', title: 'Τεχνικό Γραφείο Μηχανικών & Μεσιτικό' }
        ],
        'PersonalCareAndWellness': [
            { id: 'care_hair_barber', title: 'Κομμωτήριο, Barber Shop & Περιποίηση Μαλλιών' },
            { id: 'care_beauty_nails', title: 'Κέντρο Αισθητικής, Nails & Make-up Studio' },
            { id: 'care_fitness_gym', title: 'Γυμναστήριο, Pilates/Yoga Studio & Fitness' },
            { id: 'care_spa_massage', title: 'Spa, Μασάζ & Κέντρο Ευεξίας' }
        ],
        'CraftsAndManufacturing': [
            { id: 'craft_wood_metal', title: 'Εργαστήριο Ξυλουργικής & Μεταλλοκατασκευών' },
            { id: 'craft_printing_signage', title: 'Τυπογραφείο, Επιγραφές & Γραφικές Τέχνες' },
            { id: 'craft_food_production', title: 'Εργαστήριο Τροφίμων & Μικρή Παραγωγή' },
            { id: 'craft_textile_tailoring', title: 'Βιοτεχνία Ενδυμάτων, Ραφείο & Υφάσματα' }
        ]
    };

    function updateSubcategoryDropdown() {
        const sectorEl = document.getElementById('biz-sector');
        const subcatEl = document.getElementById('biz-subcategory');
        if (!sectorEl || !subcatEl) return;
        const list = PROTEUS_TAXONOMY[sectorEl.value] || [];
        subcatEl.innerHTML = '';
        list.forEach(item => {
            const opt = document.createElement('option');
            opt.value = item.id;
            opt.innerText = item.title;
            subcatEl.appendChild(opt);
        });
    }

    function checkScaleForEnterprise() {
        const scale = document.getElementById('biz-scale').value;
        if (scale === 'enterprise') {
            selectOnboardingRole('enterprise');
        }
    }

    function selectOnboardingRole(role) {
        const qEl = document.getElementById('onboarding-questionnaire');
        const entEl = document.getElementById('onboarding-enterprise');
        if (role === 'owner') {
            if (entEl) entEl.style.display = 'none';
            if (qEl) {
                qEl.style.display = 'block';
                updateSubcategoryDropdown();
                qEl.scrollIntoView({ behavior: 'smooth' });
            }
        } else if (role === 'enterprise') {
            if (qEl) qEl.style.display = 'none';
            if (entEl) {
                entEl.style.display = 'block';
                entEl.scrollIntoView({ behavior: 'smooth' });
            }
        } else if (role === 'designer') {
            switchTab('freelance');
        }
    }

    function advanceWizard(step) {
        for (let i = 1; i <= 4; i++) {
            const el = document.getElementById('wizard-step-' + i);
            if (el) el.style.display = (i === step) ? 'block' : 'none';
        }
        const badge = document.getElementById('wizard-progress-badge');
        const title = document.getElementById('wizard-step-title');
        if (badge) badge.innerText = 'Βήμα ' + step + '/4';
        if (title) {
            const titles = [
                'Οδηγός Επιχείρησης — Βήμα 1 από 4: Κλάδος & Εξειδίκευση',
                'Οδηγός Επιχείρησης — Βήμα 2 από 4: Φορολογικά & myDATA',
                'Οδηγός Επιχείρησης — Βήμα 3 από 4: Ροές & Προσωπικό',
                'Οδηγός Επιχείρησης — Βήμα 4 από 4: Τρόπος Υλοποίησης'
            ];
            title.innerText = titles[step - 1];
        }
    }

    function completeOnboarding(mode) {
        const name = document.getElementById('biz-name').value || 'Το Κατάστημά μου';
        const subcatEl = document.getElementById('biz-subcategory');
        const subcatText = subcatEl && subcatEl.selectedOptions[0] ? subcatEl.selectedOptions[0].innerText : 'BOS';
        const budget = document.getElementById('biz-tender-budget') ? document.getElementById('biz-tender-budget').value : '250';

        if (mode === 'solo') {
            const card = document.getElementById('download-delivery-card');
            const pkgName = document.getElementById('delivery-pkg-name');
            if (pkgName) pkgName.innerText = name + ' (' + subcatText + ')';
            if (card) {
                card.style.display = 'block';
                card.scrollIntoView({ behavior: 'smooth' });
            }
        } else if (mode === 'hire') {
            alert('✓ Το αίτημα αναρτήθηκε επιτυχώς στο Freelancing Hub με αυτόματο NDA και δέσμευση προϋπολογισμού ' + budget + '€ σε SLA Escrow!');
            switchTab('freelance');
        }
    }

    function submitEnterpriseInquiry() {
        const name = document.getElementById('ent-biz-name').value || 'Enterprise';
        const contact = document.getElementById('ent-contact-person').value || 'Εκπρόσωπος';
        const model = document.getElementById('ent-execution-model').value;
        const branches = document.getElementById('ent-branches').value || '16';
        alert('✓ Το αίτημα Enterprise Architecture για ' + name + ' (' + branches + ' θέσεις) καταχωρήθηκε! Ο πιστοποιημένος Solution Architect θα επικοινωνήσει με τον/την ' + contact + ' εντός 24 ωρών.');
    }

    function simulateDownload(platform) {
        alert('✓ Λήψη επίσημου υπογεγραμμένου binary Proteus για ' + platform + ' (SHA-256 Verified, Zero-Privilege %APPDATA% Execution).');
    }

    document.addEventListener('DOMContentLoaded', () => {
        updateSubcategoryDropdown();
    });
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_onboarding_html_structure_and_categories() {
        assert!(ONBOARDING_HTML.contains("TechnologyAndRepairs"));
        assert!(ONBOARDING_HTML.contains("RetailAndCommerce"));
        assert!(ONBOARDING_HTML.contains("HealthcareAndMedical"));
        assert!(ONBOARDING_HTML.contains("HospitalityAndFood"));
        assert!(ONBOARDING_HTML.contains("ServicesAndOffices"));
        assert!(ONBOARDING_HTML.contains("PersonalCareAndWellness"));
        assert!(ONBOARDING_HTML.contains("CraftsAndManufacturing"));
        assert!(ONBOARDING_HTML.contains("onboarding-enterprise"));
        assert!(ONBOARDING_HTML.contains("ent-execution-model"));
        assert!(ONBOARDING_HTML.contains("download-delivery-card"));
    }

    #[test]
    fn test_onboarding_js_taxonomy_and_enterprise_functions() {
        assert!(ONBOARDING_JS.contains("PROTEUS_TAXONOMY"));
        assert!(ONBOARDING_JS.contains("tech_smartphones_pc"));
        assert!(ONBOARDING_JS.contains("care_hair_barber"));
        assert!(ONBOARDING_JS.contains("craft_wood_metal"));
        assert!(ONBOARDING_JS.contains("submitEnterpriseInquiry"));
        assert!(ONBOARDING_JS.contains("updateSubcategoryDropdown"));
    }
}
