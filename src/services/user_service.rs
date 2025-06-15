use sqlx::PgPool;

use crate::{middleware::{jwt_session::Claims, model::ActionResult}, services::option_service::OptionService, CONNECTION};

pub struct UserService;

impl UserService {
    pub async fn get_user(session: Claims) -> ActionResult<serde_json::Map<std::string::String, serde_json::Value>, String> {
        let mut result: ActionResult<serde_json::Map<std::string::String, serde_json::Value>, String> = ActionResult::default();

        let connection: &PgPool = CONNECTION.get().unwrap();

        let query_result = sqlx::query(
            r#"
            SELECT 
                B.autonid AS user_id, 
                B.fullname,
                B.stage,
                B.client_id,
                B.cif_id,
                B.email, 
                A.picture,
                B.is_revised,
                B.is_rejected,
                B.is_finished,
                B.branch,
                B.referal_number,
                B.account_status,
                B.mobile_phone,
                B.spouse_relationship,
                B.spouse_name,
                B.mother_name,
                B.nationality,
                B.idcard_country,
                B.idcard_number,
                B.idcard_expire_date,
                B.sex,
                B.birth_date,
                B.birth_place,
                B.marital_status,
                B.religion,
                B.education,
                B.idcard_city,
                B.idcard_district,
                B.idcard_subdistrict,
                B.idcard_rw,
                B.idcard_rt,
                B.idcard_address,
                B.idcard_zipcode,
                B.copy_id,
                B.domicile_city,
                B.domicile_district,
                B.domicile_subdistrict,
                B.domicile_rw,
                B.domicile_rt,
                B.domicile_address,
                B.domicile_zipcode,
                B.last_update,
                B.question_rdn,
                B.bank_code,
                B.bank_name,
                B.bank_branch,
                B.bank_account_number,
                B.bank_account_holder,
                B.question_npwp,
                B.npwp_number,
                B.npwp_reason,
                B.company_name,
                B.company_address,
                B.fund_source,
                B.fund_source_text,
                B.occupation,
                B.occupation_text,
                B.nature_of_business,
                B.nature_of_business_text,
                B.position,
                B.position_text,
                B.income_per_annum,
                B.question1,
                B.question1_text,
                B.question2,
                B.question2_text,
                B.question3,
                B.question3_text,
                B.question4,
                B.question4_text,
                B.question5,
                B.question5_text,
                B.question6,
                B.question6_text,
                B.investment_objectives,
                B.risk,
                B.question_fatca,
                B.fatca1,
                B.fatca2,
                B.fatca3,
                B.spouse_occupation,
                B.spouse_occupation_text,
                B.spouse_nature_of_business,
                B.spouse_company_name,
                B.spouse_company_address,
                B.spouse_company_city,
                B.spouse_company_zipcode,
                B.spouse_fund_source,
                B.spouse_fund_source_text,
                B.idcard_file,
                B.selfie_file,
                B.signature_file,
                B.sales,
                B.spouse_relationship_text,
                B.education_text,
                B.npwp_file,
                B.birth_country
            FROM users A
            LEFT JOIN user_kyc B ON A.web_cif_id = B.autonid
            LEFT JOIN user_request C ON A.web_cif_id = C.web_cif_nid
            WHERE b.autonid = $1
            "#
        ).bind(&session.usernid)
        .fetch_one(connection)
        .await;

        match query_result {
            Ok(row) => {

                result.result = true;
                let json_obj = OptionService::row_to_json(&row);
                result.data = Some(json_obj);
            }
            Err(e) => {
                result.message = format!("Incorrect email or password");
                println!("❌ Login Error: {}", e);
            }
        }

        return result;
    }
}