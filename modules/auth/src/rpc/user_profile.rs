use crate::rpc::middleware::UserId;
use crate::services::user_account::{
    UpdateUserName, UserAccountService as InnerUserAccountService,
};
use kanau::processor::Processor;
use phantom_shop_proto::v1::auth::user::{UpdateUserNameRequest, UpdateUserNameResponse};
use tonic::{Request, Response, Status};

pub struct UserProfileServiceImpl {
    pub user_account_service: InnerUserAccountService,
}

impl UserProfileServiceImpl {
    pub fn new(user_account_service: InnerUserAccountService) -> Self {
        Self {
            user_account_service,
        }
    }
}

#[tonic::async_trait]
impl phantom_shop_proto::v1::auth::user::user_profile_service_server::UserProfileService
    for UserProfileServiceImpl
{
    async fn update_user_name(
        &self,
        request: Request<UpdateUserNameRequest>,
    ) -> Result<Response<UpdateUserNameResponse>, Status> {
        let (user_id, req) = UserId::from_request(request)?;

        let name = if req.name.is_empty() {
            None
        } else {
            Some(req.name)
        };

        let updated_account = self
            .user_account_service
            .process(UpdateUserName {
                user_id: user_id.into_inner(),
                name,
            })
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(UpdateUserNameResponse {
            new_name: updated_account.name.unwrap_or_default(),
        }))
    }
}
