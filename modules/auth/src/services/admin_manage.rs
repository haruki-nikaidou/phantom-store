use framework::sqlx::DatabaseProcessor;

#[derive(Clone)]
pub struct UserAuthManageService {
    pub db: DatabaseProcessor,
}
