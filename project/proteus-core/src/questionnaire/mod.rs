//! Hierarchical Business Questionnaire & Requirements Engine.
//! Captures business profile, workflow requirements, statutory tax/labor options,
//! and design strategy to scaffold starter `.pr` packages or publish RFP tenders.

pub mod categories;
pub mod scaffold;
pub mod storage;
pub mod types;

pub use categories::*;
pub use scaffold::generate_starter_package_from_answers;
pub use storage::*;
pub use types::*;

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn in_memory_db() -> Connection {
        Connection::open_in_memory().unwrap()
    }

    #[test]
    fn test_all_top_categories_and_subcategories() {
        let all_cats = TopIndustryCategory::all();
        assert_eq!(all_cats.len(), 7);

        for cat in all_cats {
            let subcats = get_available_subcategories(*cat);
            assert!(
                !subcats.is_empty(),
                "Category {:?} should have subcategories",
                cat
            );
            assert!(!cat.display_name_el().is_empty());
            assert!(!cat.display_name_en().is_empty());
        }

        let care_subs = get_available_subcategories(TopIndustryCategory::PersonalCareAndWellness);
        assert_eq!(care_subs[0].id, "care_hair_barber");

        let craft_subs = get_available_subcategories(TopIndustryCategory::CraftsAndManufacturing);
        assert_eq!(craft_subs[0].id, "craft_wood_metal");
    }

    #[test]
    fn test_questionnaire_sqlite_lifecycle() {
        let conn = in_memory_db();
        init_questionnaire_schema(&conn).unwrap();

        let mut q = BusinessQuestionnaire::new(
            "TechFix Patras",
            TopIndustryCategory::TechnologyAndRepairs,
            "tech_smartphones_pc",
        );
        q.afm = Some("123456789".into());
        q.workflow.has_intake_voucher = true;
        q.workflow.has_mydata_invoicing = true;
        q.workflow.has_ergani_work_card = true;
        q.scale.employee_count = 2;
        q.strategy = DesignStrategy::OpenTenderRfp {
            max_budget_eur: 300.0,
            require_nda: true,
        };

        save_questionnaire(&conn, &q).unwrap();

        let loaded = get_questionnaire(&conn, &q.id)
            .unwrap()
            .expect("Should find questionnaire");
        assert_eq!(loaded.business_name, "TechFix Patras");
        assert_eq!(loaded.afm.as_deref(), Some("123456789"));
        assert!(loaded.workflow.has_intake_voucher);
        assert!(loaded.workflow.has_ergani_work_card);
        assert_eq!(loaded.scale.employee_count, 2);

        match loaded.strategy {
            DesignStrategy::OpenTenderRfp {
                max_budget_eur,
                require_nda,
            } => {
                assert_eq!(max_budget_eur, 300.0);
                assert!(require_nda);
            }
            _ => panic!("Expected OpenTenderRfp"),
        }

        let all = list_questionnaires(&conn).unwrap();
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].id, q.id);
    }

    #[test]
    fn test_enterprise_strategy_persistence() {
        let conn = in_memory_db();
        init_questionnaire_schema(&conn).unwrap();

        let mut q = BusinessQuestionnaire::new(
            "Acme Retail Group",
            TopIndustryCategory::RetailAndCommerce,
            "retail_warehouse_vmi",
        );
        q.scale.employee_count = 45;
        q.scale.branch_count = 8;
        q.strategy = DesignStrategy::EnterpriseConsultation {
            contact_person: "George Papadopoulos".into(),
            contact_email: "george@acmegroup.gr".into(),
            branch_count: 8,
            legacy_erp: Some("SoftOne Cloud ERP".into()),
            preferred_execution: EnterpriseExecutionModel::InHouseAcademyTraining,
        };

        save_questionnaire(&conn, &q).unwrap();

        let loaded = get_questionnaire(&conn, &q.id).unwrap().unwrap();
        match loaded.strategy {
            DesignStrategy::EnterpriseConsultation {
                contact_person,
                preferred_execution,
                legacy_erp,
                ..
            } => {
                assert_eq!(contact_person, "George Papadopoulos");
                assert_eq!(
                    preferred_execution,
                    EnterpriseExecutionModel::InHouseAcademyTraining
                );
                assert_eq!(legacy_erp.as_deref(), Some("SoftOne Cloud ERP"));
            }
            _ => panic!("Expected EnterpriseConsultation"),
        }
    }

    #[test]
    fn test_scaffold_all_categories() {
        for cat in TopIndustryCategory::all() {
            let subcats = get_available_subcategories(*cat);
            let q = BusinessQuestionnaire::new("Test Shop", *cat, &subcats[0].id);
            let pkg = generate_starter_package_from_answers(&q);
            assert!(!pkg.schema.ddl_statements.is_empty());
            assert!(!pkg.views.is_empty());
        }
    }
}
