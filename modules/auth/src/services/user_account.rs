use crate::entities::db::user_account::UserAccount;
use framework::sqlx::DatabaseProcessor;
use kanau::processor::Processor;
use uuid::Uuid;

#[derive(Clone)]
pub struct UserAccountService {
    pub db: DatabaseProcessor,
}

#[derive(Debug, Clone)]
pub struct UpdateUserName {
    pub user_id: Uuid,
    pub name: Option<String>,
}

impl Processor<UpdateUserName> for UserAccountService {
    type Output = UserAccount;
    type Error = framework::Error;
    async fn process(&self, input: UpdateUserName) -> Result<UserAccount, framework::Error> {
        self.db
            .process(crate::entities::db::user_account::UpdateUserName {
                id: input.user_id,
                name: input.name,
            })
            .await
            .map_err(Into::into)
    }
}
