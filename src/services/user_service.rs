use crate::middleware::model::ActionResult;

pub struct UserService;

impl UserService {
    pub async fn get_user() -> ActionResult<String, String> {
        let mut result = ActionResult::default();

        return result;
    }
}