use framework::sqlx::DatabaseProcessor;

#[derive(Clone)]
pub struct UserAccountService {
    pub db: DatabaseProcessor,
}