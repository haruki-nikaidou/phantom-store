use crate::entities::admin_account::{AdminRole, FindAdminById};
use framework::sqlx::DatabaseProcessor;
use kanau::processor::Processor;
use tracing::{Span, instrument};
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

impl<Oper> Processor<AuthenticatedAdminOperation<Oper>> for AuthorizationLayer
where
    Oper: AdminOperation + Send,
{
    type Output = Oper;
    type Error = framework::Error;
    #[instrument(skip_all, err)]
    async fn process(
        &self,
        input: AuthenticatedAdminOperation<Oper>,
    ) -> Result<Oper, framework::Error> {
        let Some(admin) = self
            .database_processor
            .process(FindAdminById { id: input.admin_id })
            .await?
        else {
            return Err(framework::Error::PermissionsDenied);
        };

        Span::current().record("admin_id", admin.id.to_string());
        Span::current().record("admin_role", &format!("{:?}", admin.role));

        let allowed = Oper::check_permission(admin.role);
        if !allowed {
            Err(framework::Error::PermissionsDenied)
        } else {
            Ok(input.operation)
        }
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
