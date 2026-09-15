//! Specialist Role Dashboards for Proteus Ecosystem.
//! Contains:
//! - Customer Service & Staff Training Dashboard
//! - Sales & Solutions Consultant Dashboard (with Tier Commission Tracker & Price Calculator)
//! - Designer Studio Hub (Project manager & Cloud Compilation Gate status)

use egui::{Color32, CornerRadius, Frame, Margin, RichText, Stroke, Ui};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpecialistRole {
    CustomerService,
    SalesConsultant,
    DesignerHub,
}

pub struct SpecialistDashboardState {
    pub active_role: SpecialistRole,
    // Sales Consultant Calculator
    pub calc_users: u32,
    pub calc_include_cloud: bool,
    pub calc_printer_count: u32,
    // Designer Hub
    pub new_project_name: String,
    pub local_projects: Vec<String>,
}

impl Default for SpecialistDashboardState {
    fn default() -> Self {
        Self {
            active_role: SpecialistRole::CustomerService,
            calc_users: 3,
            calc_include_cloud: true,
            calc_printer_count: 1,
            new_project_name: String::new(),
            local_projects: vec![
                "AutoService_Main_v2.prproj".to_string(),
                "PhoneRepair_Kiosk.prproj".to_string(),
                "Bike_Workshop_Standard.prproj".to_string(),
            ],
        }
    }
}

pub fn draw_specialist_dashboards_view(
    ui: &mut Ui,
    state: &mut SpecialistDashboardState,
) {
    ui.vertical(|ui| {
        // Sub-navigation bar for specialist roles
        ui.horizontal(|ui| {
            let roles = [
                (SpecialistRole::CustomerService, "🤝 Εξυπηρέτηση Πελατών & Εκπαίδευση"),
                (SpecialistRole::SalesConsultant, "📈 Σύμβουλος Πωλήσεων & Tier Tracker"),
                (SpecialistRole::DesignerHub, "🎨 Designer Studio Hub (.prproj)"),
            ];

            for (role, label) in roles {
                let is_active = state.active_role == role;
                let btn = if is_active {
                    egui::Button::new(RichText::new(label).strong().size(13.0).color(Color32::WHITE))
                        .fill(crate::theme::ACCENT_PRIMARY)
                } else {
                    egui::Button::new(RichText::new(label).size(13.0).color(crate::theme::TEXT_SECONDARY))
                        .fill(crate::theme::BG_CARD)
                };

                if ui.add(btn).clicked() {
                    state.active_role = role;
                }
                ui.add_space(6.0);
            }
        });

        ui.add_space(10.0);
        ui.separator();
        ui.add_space(12.0);

        match state.active_role {
            SpecialistRole::CustomerService => draw_customer_service_dashboard(ui),
            SpecialistRole::SalesConsultant => draw_sales_consultant_dashboard(ui, state),
            SpecialistRole::DesignerHub => draw_designer_hub(ui, state),
        }
    });
}

// ---------------------------------------------------------------------------
// 1. Customer Service & Staff Training Dashboard
// ---------------------------------------------------------------------------
fn draw_customer_service_dashboard(ui: &mut Ui) {
    ui.heading(RichText::new("🤝 Dashboard Εξυπηρέτησης Πελατών & Ποιότητας").strong().size(20.0));
    ui.add_space(4.0);
    ui.label(RichText::new("Παρακολούθηση ταχύτητας ταμείου (Intake Benchmark), εκπαίδευση προσωπικού και δείκτες ικανοποίησης.")
        .size(12.0)
        .color(crate::theme::TEXT_MUTED));
    ui.add_space(14.0);

    ui.columns(3, |cols| {
        // Metric 1: Intake Speed
        cols[0].vertical(|ui| {
            draw_metric_card(
                ui,
                "ΜΕΣΟΣ ΧΡΟΝΟΣ ΠΑΡΑΛΑΒΗΣ",
                "22.4 δευτ.",
                "✓ Στόχος < 30s επετεύχθη",
                Color32::from_rgb(52, 211, 153),
            );
        });

        // Metric 2: CSAT
        cols[1].vertical(|ui| {
            draw_metric_card(
                ui,
                "ΙΚΑΝΟΠΟΙΗΣΗ ΠΕΛΑΤΩΝ (CSAT)",
                "98.2%",
                "★ 4.9/5.0 (148 αξιολογήσεις)",
                Color32::from_rgb(56, 189, 248),
            );
        });

        // Metric 3: Training Progress
        cols[2].vertical(|ui| {
            draw_metric_card(
                ui,
                "ΕΚΠΑΙΔΕΥΣΗ ΠΡΟΣΩΠΙΚΟΥ",
                "100% Πλήρης",
                "3/3 Χειριστές Πιστοποιημένοι",
                Color32::from_rgb(168, 85, 247),
            );
        });
    });

    ui.add_space(16.0);

    // Benchmarking & Best Practices Card
    Frame::new()
        .fill(crate::theme::BG_CARD)
        .stroke(Stroke::new(1.0, crate::theme::BORDER_SUBTLE))
        .corner_radius(CornerRadius::same(8))
        .inner_margin(Margin::same(16))
        .show(ui, |ui| {
            ui.label(RichText::new("ΟΔΗΓΙΕΣ ΒΕΛΤΙΣΤΟΠΟΙΗΣΗΣ ΡΟΗΣ ΤΑΜΕΙΟΥ (ZERO COGNITIVE LOAD)").strong().color(crate::theme::TEXT_MUTED));
            ui.add_space(10.0);

            let guidelines = [
                ("1. Άμεση Αναγνώριση Πελάτη:", "Χρήση του τηλεφώνου για άμεση αυτόματη συμπλήρωση ιστορικού επισκευών."),
                ("2. Επιβεβαίωση Βλάβης & Εκτύπωση:", "Καταγραφή της πρωταρχικής βλάβης σε < 20 λέξεις και άμεση έκδοση δελτίου με QR code."),
                ("3. Επικοινωνία με Τεχνικό Εργαστηρίου:", "Αυτόματη μετακίνηση του δελτίου στη λωρίδα 'Σε Εξέλιξη' μόλις το αναλάβει ο τεχνικός πάγκου."),
            ];

            for (title, desc) in guidelines {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(title).strong().color(crate::theme::TEXT_PRIMARY));
                    ui.label(RichText::new(desc).color(crate::theme::TEXT_SECONDARY));
                });
                ui.add_space(6.0);
            }
        });
}

// ---------------------------------------------------------------------------
// 2. Sales & Solutions Consultant Dashboard
// ---------------------------------------------------------------------------
fn draw_sales_consultant_dashboard(ui: &mut Ui, state: &mut SpecialistDashboardState) {
    ui.heading(RichText::new("📈 Dashboard Συμβούλου Πωλήσεων & Κλίμακα Tier").strong().size(20.0));
    ui.add_space(4.0);
    ui.label(RichText::new("Παρακολούθηση κλίμακας προμήθειας, συνδρομής αναβάθμισης (30€/μήνα) και υπολογιστής προσφορών καταστημάτων.")
        .size(12.0)
        .color(crate::theme::TEXT_MUTED));
    ui.add_space(14.0);

    ui.columns(2, |cols| {
        // Left Column: Tier Tracker
        cols[0].vertical(|ui| {
            Frame::new()
                .fill(crate::theme::BG_CARD)
                .stroke(Stroke::new(1.0, crate::theme::BORDER_SUBTLE))
                .corner_radius(CornerRadius::same(8))
                .inner_margin(Margin::same(16))
                .show(ui, |ui| {
                    ui.label(RichText::new("ΤΡΕΧΟΝ ΕΠΙΠΕΔΟ ΠΙΣΤΟΠΟΙΗΣΗΣ (TIER STATUS)").strong().color(crate::theme::TEXT_MUTED));
                    ui.add_space(10.0);

                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Tier 2").strong().size(24.0).color(crate::theme::ACCENT_CYAN));
                        ui.label(RichText::new("— Certified Specialist").size(15.0).color(crate::theme::TEXT_PRIMARY));
                    });
                    ui.add_space(6.0);

                    ui.horizontal(|ui| {
                        ui.label("Προμήθεια Πλατφόρμας:");
                        ui.label(RichText::new("35%").strong().color(Color32::from_rgb(251, 146, 60)));
                        ui.label("| Καθαρή Αμοιβή:");
                        ui.label(RichText::new("65%").strong().color(Color32::from_rgb(52, 211, 153)));
                    });

                    ui.add_space(6.0);
                    ui.horizontal(|ui| {
                        ui.label("Συνδρομή Badge:");
                        ui.label(RichText::new("30,00€ / μήνα").strong().color(crate::theme::TEXT_PRIMARY));
                        ui.label(RichText::new("(Ενεργή)").color(Color32::from_rgb(52, 211, 153)));
                    });

                    ui.add_space(12.0);
                    ui.separator();
                    ui.add_space(10.0);

                    ui.label(RichText::new("ΕΠΟΜΕΝΟΣ ΣΤΟΧΟΣ: Tier 3 (Certified Professional)").strong().color(crate::theme::TEXT_PRIMARY));
                    ui.label(RichText::new("Μείωση προμήθειας στο 30% (70% καθαρό κέρδος)").size(11.0).color(crate::theme::TEXT_MUTED));
                    ui.add_space(8.0);
                    // Progress bar
                    let progress = 0.65f32;
                    ui.add(egui::ProgressBar::new(progress).show_percentage().animate(false));
                    ui.label(RichText::new("13 από 20 επιτυχημένες εγκαταστάσεις ολοκληρώθηκαν").size(11.0).color(crate::theme::TEXT_MUTED));
                });
        });

        // Right Column: Quote Calculator
        cols[1].vertical(|ui| {
            Frame::new()
                .fill(crate::theme::BG_CARD)
                .stroke(Stroke::new(1.0, crate::theme::BORDER_SUBTLE))
                .corner_radius(CornerRadius::same(8))
                .inner_margin(Margin::same(16))
                .show(ui, |ui| {
                    ui.label(RichText::new("ΥΠΟΛΟΓΙΣΤΗΣ ΠΡΟΣΦΟΡΑΣ ΚΑΤΑΣΤΗΜΑΤΟΣ").strong().color(crate::theme::TEXT_MUTED));
                    ui.add_space(10.0);

                    ui.horizontal(|ui| {
                        ui.label("Αριθμός Χρηστών Ταμείου:");
                        ui.add(egui::Slider::new(&mut state.calc_users, 1..=25).text("χρήστες"));
                    });
                    ui.add_space(6.0);

                    ui.checkbox(&mut state.calc_include_cloud, "Managed Cloud Sync & Backups (+15€/mo)");
                    ui.add_space(6.0);

                    ui.horizontal(|ui| {
                        ui.label("Θερμικοί Εκτυπωτές POS:");
                        ui.add(egui::Slider::new(&mut state.calc_printer_count, 0..=5).text("συσκευές"));
                    });

                    ui.add_space(10.0);
                    ui.separator();
                    ui.add_space(8.0);

                    // Mathematical Calculation according to Doc 18
                    let base_license = 7.99;
                    let extra_users_fee = if state.calc_users > 4 {
                        (state.calc_users - 4) as f64 * 1.50
                    } else {
                        0.0
                    };
                    let core_total = base_license + extra_users_fee;
                    let cloud_fee = if state.calc_include_cloud { 15.0 } else { 0.0 };
                    let monthly_total = core_total + cloud_fee;
                    let hardware_one_off = state.calc_printer_count as f64 * 140.0;

                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Μηνιαία Συνδρομή Πελάτη:").strong());
                        ui.label(RichText::new(format!("{:.2}€ / μήνα", monthly_total)).strong().size(16.0).color(crate::theme::ACCENT_CYAN));
                    });

                    if hardware_one_off > 0.0 {
                        ui.horizontal(|ui| {
                            ui.label("Εξοπλισμός POS (Εφάπαξ):");
                            ui.label(RichText::new(format!("{:.2}€", hardware_one_off)).strong().color(Color32::WHITE));
                        });
                    }

                    // Consultant Net Share (65% on setup gig)
                    let setup_estimate = 350.0;
                    let consultant_setup_net = setup_estimate * 0.65;
                    ui.add_space(6.0);
                    ui.horizontal(|ui| {
                        ui.label("Καθαρή Αμοιβή Εγκατάστασης (Tier 2):");
                        ui.label(RichText::new(format!("{:.2}€", consultant_setup_net)).strong().color(Color32::from_rgb(52, 211, 153)));
                    });
                });
        });
    });
}

// ---------------------------------------------------------------------------
// 3. Designer Studio Hub (.prproj manager)
// ---------------------------------------------------------------------------
fn draw_designer_hub(ui: &mut Ui, state: &mut SpecialistDashboardState) {
    ui.heading(RichText::new("🎨 Designer Hub — Τοπικός Σχεδιασμός & Marketplace").strong().size(20.0));
    ui.add_space(4.0);
    ui.label(RichText::new("Το Proteus Studio είναι 100% ΔΩΡΕΑΝ. Διαχειριστείτε τοπικά αρχεία .prproj και υποβάλετε στο Marketplace για έλεγχο compilation.")
        .size(12.0)
        .color(crate::theme::TEXT_MUTED));
    ui.add_space(14.0);

    ui.columns(2, |cols| {
        // Left Column: Local Projects
        cols[0].vertical(|ui| {
            Frame::new()
                .fill(crate::theme::BG_CARD)
                .stroke(Stroke::new(1.0, crate::theme::BORDER_SUBTLE))
                .corner_radius(CornerRadius::same(8))
                .inner_margin(Margin::same(16))
                .show(ui, |ui| {
                    ui.label(RichText::new("ΤΟΠΙΚΑ ΑΡΧΕΙΑ ΕΡΓΩΝ (.prproj)").strong().color(crate::theme::TEXT_MUTED));
                    ui.add_space(10.0);

                    for proj in &state.local_projects {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("📦").color(crate::theme::ACCENT_CYAN));
                            ui.label(RichText::new(proj).strong().color(crate::theme::TEXT_PRIMARY));
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.small_button("Άνοιγμα στο Studio").clicked() {
                                    // Studio opener indicator
                                }
                            });
                        });
                        ui.add_space(6.0);
                    }

                    ui.add_space(10.0);
                    ui.separator();
                    ui.add_space(8.0);

                    ui.horizontal(|ui| {
                        ui.text_edit_singleline(&mut state.new_project_name);
                        if ui.button("＋ Νέο .prproj").clicked() && !state.new_project_name.trim().is_empty() {
                            let name = format!("{}.prproj", state.new_project_name.trim());
                            state.local_projects.push(name);
                            state.new_project_name.clear();
                        }
                    });
                });
        });

        // Right Column: Cloud Compiler Gate & Walled Garden
        cols[1].vertical(|ui| {
            Frame::new()
                .fill(crate::theme::BG_CARD)
                .stroke(Stroke::new(1.0, crate::theme::BORDER_SUBTLE))
                .corner_radius(CornerRadius::same(8))
                .inner_margin(Margin::same(16))
                .show(ui, |ui| {
                    ui.label(RichText::new("ΚΛΕΙΣΤΟΣ ΒΡΟΧΟΣ COMPILATION (ANTI-BYPASS GATE)").strong().color(crate::theme::TEXT_MUTED));
                    ui.add_space(10.0);

                    ui.label(RichText::new("✓ Το Studio διατίθεται 100% δωρεάν για τοπικό σχεδιασμό.")
                        .color(Color32::from_rgb(52, 211, 153)));
                    ui.add_space(4.0);
                    ui.label(RichText::new("🔒 Τοπικό export σε .pr πακέτο: ΑΠΟΚΛΕΙΣΜΕΝΟ (Αρχιτεκτονικός Μονόδρομος)")
                        .color(Color32::from_rgb(251, 146, 60)));
                    ui.add_space(4.0);
                    ui.label(RichText::new("☁ Δημοσίευση & Compilation: Αποκλειστικά μέσω του Proteus Hub.")
                        .color(crate::theme::TEXT_SECONDARY));
                    ui.add_space(4.0);
                    ui.label(RichText::new("🛡 Πληρωμή & Παράδοση: Απευθείας παράδοση στο client του πελάτη μόνο μετά την επιβεβαίωση πληρωμής σε Escrow.")
                        .color(crate::theme::TEXT_SECONDARY));

                    ui.add_space(14.0);
                    if ui.button(RichText::new("🚀 Υποβολή Project στο Marketplace Hub").strong()).clicked() {
                        // Submit trigger
                    }
                });
        });
    });
}

fn draw_metric_card(ui: &mut Ui, label: &str, value: &str, sub: &str, col: Color32) {
    Frame::new()
        .fill(crate::theme::BG_CARD)
        .stroke(Stroke::new(1.0, crate::theme::BORDER_SUBTLE))
        .corner_radius(CornerRadius::same(8))
        .inner_margin(Margin::same(14))
        .show(ui, |ui| {
            ui.label(RichText::new(label).size(10.0).strong().color(crate::theme::TEXT_MUTED));
            ui.add_space(6.0);
            ui.label(RichText::new(value).size(22.0).strong().color(col));
            ui.add_space(4.0);
            ui.label(RichText::new(sub).size(11.0).color(crate::theme::TEXT_SECONDARY));
        });
}
