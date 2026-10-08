//! Granular industry subcategory taxonomy covering all SME business sectors.

use super::types::{SubCategory, TopIndustryCategory};

/// Returns the available subcategories for a given top industry category.
pub fn get_available_subcategories(top: TopIndustryCategory) -> Vec<SubCategory> {
    match top {
        TopIndustryCategory::TechnologyAndRepairs => vec![
            SubCategory {
                id: "tech_smartphones_pc".into(),
                top_category: top,
                title_el: "Επισκευές Smartphones, Laptops & Υπολογιστών".into(),
                title_en: "Smartphones & PC Repair Lab".into(),
            },
            SubCategory {
                id: "tech_auto_moto".into(),
                top_category: top,
                title_el: "Συνεργείο Αυτοκινήτων, Μοτοσυκλετών & Φανοποιείο".into(),
                title_en: "Auto & Motorcycle Workshop".into(),
            },
            SubCategory {
                id: "tech_appliances".into(),
                top_category: top,
                title_el: "Επισκευές Οικιακών & Επαγγελματικών Συσκευών / Ψύξη".into(),
                title_en: "Home & Commercial Appliances Service".into(),
            },
            SubCategory {
                id: "tech_electronics_automation".into(),
                top_category: top,
                title_el: "Ηλεκτρολογικές Εγκαταστάσεις, Συναγερμοί & Αυτοματισμοί".into(),
                title_en: "Electrical, Security & Automation Systems".into(),
            },
        ],
        TopIndustryCategory::RetailAndCommerce => vec![
            SubCategory {
                id: "retail_fashion".into(),
                top_category: top,
                title_el: "Κατάστημα Ένδυσης, Υπόδησης & Αξεσουάρ (Μεγέθη/Χρώματα)".into(),
                title_en: "Fashion & Apparel Store".into(),
            },
            SubCategory {
                id: "retail_minimarket".into(),
                top_category: top,
                title_el: "Μίνι Μάρκετ, Ψιλικά, Περίπτερο & Fast POS".into(),
                title_en: "Convenience Store, Kiosk & Fast POS".into(),
            },
            SubCategory {
                id: "retail_warehouse_vmi".into(),
                top_category: top,
                title_el: "Χονδρικό Εμπόριο, Αποθήκη WMS & Logistics".into(),
                title_en: "Wholesale & Warehouse WMS Logistics".into(),
            },
            SubCategory {
                id: "retail_jewelry_gifts".into(),
                top_category: top,
                title_el: "Κοσμηματοπωλείο, Είδη Δώρων & Οπτικά".into(),
                title_en: "Jewelry, Luxury Gifts & Optics".into(),
            },
        ],
        TopIndustryCategory::HealthcareAndMedical => vec![
            SubCategory {
                id: "health_dental".into(),
                top_category: top,
                title_el: "Οδοντιατρείο & Οδοντοτεχνικό Εργαστήριο".into(),
                title_en: "Dental Practice & Lab".into(),
            },
            SubCategory {
                id: "health_clinic".into(),
                top_category: top,
                title_el: "Ιδιωτικό Ιατρείο, Διαγνωστικό & Πολυϊατρείο".into(),
                title_en: "Private Medical Clinic & Diagnostics".into(),
            },
            SubCategory {
                id: "health_vet".into(),
                top_category: top,
                title_el: "Κτηνιατρείο, Pet Care & Κτηνιατρική Κλινική".into(),
                title_en: "Veterinary Clinic & Pet Care".into(),
            },
            SubCategory {
                id: "health_physio".into(),
                top_category: top,
                title_el: "Φυσικοθεραπευτήριο & Κέντρο Αποκατάστασης".into(),
                title_en: "Physiotherapy & Rehabilitation Center".into(),
            },
        ],
        TopIndustryCategory::HospitalityAndFood => vec![
            SubCategory {
                id: "food_cafe_takeaway".into(),
                top_category: top,
                title_el: "Καφέ, Takeaway & Delivery (Touch POS)".into(),
                title_en: "Cafe, Takeaway & Quick Service".into(),
            },
            SubCategory {
                id: "food_bakery".into(),
                top_category: top,
                title_el: "Αρτοποιείο, Ζαχαροπλαστείο & Catering".into(),
                title_en: "Bakery, Pastry & Catering".into(),
            },
            SubCategory {
                id: "food_restaurant".into(),
                top_category: top,
                title_el: "Εστιατόριο & Ταβέρνα (Τραπέζια & Ασύρματη Παραγγελιοληψία)".into(),
                title_en: "Restaurant, Tavern & Wireless Table Ordering".into(),
            },
            SubCategory {
                id: "food_hotel_lodging".into(),
                top_category: top,
                title_el: "Ξενοδοχείο, Ενοικιαζόμενα Δωμάτια & Boutique Lodging".into(),
                title_en: "Hotel, Apartments & Boutique Lodging".into(),
            },
        ],
        TopIndustryCategory::ServicesAndOffices => vec![
            SubCategory {
                id: "srv_accounting".into(),
                top_category: top,
                title_el: "Λογιστικό & Φοροτεχνικό Γραφείο".into(),
                title_en: "Accounting & Tax Consultancy".into(),
            },
            SubCategory {
                id: "srv_legal".into(),
                top_category: top,
                title_el: "Δικηγορικό Γραφείο & Νομικές Υπηρεσίες".into(),
                title_en: "Law Firm & Legal Services".into(),
            },
            SubCategory {
                id: "srv_education".into(),
                top_category: top,
                title_el: "Φροντιστήριο, Κέντρο Ξένων Γλωσσών & Εκπαίδευση".into(),
                title_en: "Education, Language & Tutoring Center".into(),
            },
            SubCategory {
                id: "srv_realestate_engineering".into(),
                top_category: top,
                title_el: "Τεχνικό Γραφείο Μηχανικών & Μεσιτικό Γραφείο".into(),
                title_en: "Engineering Bureau & Real Estate Agency".into(),
            },
        ],
        TopIndustryCategory::PersonalCareAndWellness => vec![
            SubCategory {
                id: "care_hair_barber".into(),
                top_category: top,
                title_el: "Κομμωτήριο, Barber Shop & Περιποίηση Μαλλιών".into(),
                title_en: "Hair Salon & Barber Shop".into(),
            },
            SubCategory {
                id: "care_beauty_nails".into(),
                top_category: top,
                title_el: "Κέντρο Αισθητικής, Nails & Make-up Studio".into(),
                title_en: "Beauty Salon, Nails & Aesthetics Studio".into(),
            },
            SubCategory {
                id: "care_fitness_gym".into(),
                top_category: top,
                title_el: "Γυμναστήριο, Pilates / Yoga Studio & Fitness Club".into(),
                title_en: "Gym, Fitness & Pilates Studio".into(),
            },
            SubCategory {
                id: "care_spa_massage".into(),
                top_category: top,
                title_el: "Spa, Μασάζ & Κέντρο Ευεξίας".into(),
                title_en: "Spa, Massage & Wellness Center".into(),
            },
        ],
        TopIndustryCategory::CraftsAndManufacturing => vec![
            SubCategory {
                id: "craft_wood_metal".into(),
                top_category: top,
                title_el: "Εργαστήριο Ξυλουργικής, Μεταλλοκατασκευών & Επίπλων".into(),
                title_en: "Carpentry, Metal Fabrication & Furniture".into(),
            },
            SubCategory {
                id: "craft_printing_signage".into(),
                top_category: top,
                title_el: "Τυπογραφείο, Επιγραφές & Γραφικές Τέχνες".into(),
                title_en: "Printing, Signage & Graphic Arts".into(),
            },
            SubCategory {
                id: "craft_food_production".into(),
                top_category: top,
                title_el: "Εργαστήριο Τροφίμων, Ζυθοποιία & Μικρή Παραγωγή".into(),
                title_en: "Artisan Food Processing & Microbrewery".into(),
            },
            SubCategory {
                id: "craft_textile_tailoring".into(),
                top_category: top,
                title_el: "Βιοτεχνία Ενδυμάτων, Ραφείο & Υφάσματα".into(),
                title_en: "Textile Workshop & Tailoring".into(),
            },
        ],
    }
}
