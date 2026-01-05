use crate::entities::admin_account::AdminRole;
use framework::sqlx::DatabaseProcessor;
use kanau::processor::Processor;
use tracing::instrument;
use uuid::Uuid;

pub struct AuthenticatedAdminOperation<T: AdminOperation> {
    pub admin_id: Uuid,
    pub operation: T,
}

pub trait AdminOperation {
    const ALLOWED_ROLES: &'static [AdminRole];
    fn check_permission(role: AdminRole) -> bool {
        Self::ALLOWED_ROLES.contains(&role)
    }
}

#[derive(Debug, Clone)]
pub struct AuthorizationLayer {
    database_processor: DatabaseProcessor,
}

impl AuthorizationLayer {
    pub fn new(database_processor: DatabaseProcessor) -> Self {
        Self { database_processor }
    }
}

impl<Oper, Output, Proc>
    kanau::layer::Layer<AuthenticatedAdminOperation<Oper>, Result<Output, framework::Error>, Proc>
    for AuthorizationLayer
where
    Oper: AdminOperation + Send,
    Output: Send,
    Proc: Processor<AuthenticatedAdminOperation<Oper>, Result<Output, framework::Error>>
        + Send
        + Sync,
{
    #[instrument(
        skip_all,
        err,
        fields(
            admin_id = %input.admin_id,
        )
    )]
    async fn wrap(
        &self,
        processor: &Proc,
        input: AuthenticatedAdminOperation<Oper>,
    ) -> Result<Output, framework::Error> {
        todo!()
    }
}

#[macro_export]
/// A macro to implement RBAC for admin operations.
///
/// It will:
///
/// 1. Implement the `AdminOperation` trait for the operation type, specifying the allowed roles.
/// 2. Implement the `Processor<AuthenticatedAdminOperation<Oper>, Result<Output, framework::Error>>` trait for the processor type,
///
/// # Example
/// ```ignore
/// rbac! {MyProcessor : MyOperation => MyOutput | [AdminRole::SuperAdmin, AdminRole::UserManager]}
/// ```
macro_rules! rbac {
    ($processor:ty : $oper:ty => $output:ty | $roles:expr ) => {
        impl $crate::utils::rbac::AdminOperation for $oper {
            const ALLOWED_ROLES: &'static [$crate::entities::admin_account::AdminRole] = &$roles;
        }
        impl kanau::processor::Processor<
            $crate::utils::rbac::AuthenticatedAdminOperation<$oper>,
            Result<$output, framework::Error>,
        > for $processor {
            async fn process(
                &self,
                input: $crate::utils::rbac::AuthenticatedAdminOperation<$oper>,
            ) -> Result<$output, framework::Error> {
                <Self as kanau::processor::Processor<$oper, Result<$output, framework::Error>>>::process(
                    self,
                    input.operation,
                )
                .await
            }
        }
    };
}
