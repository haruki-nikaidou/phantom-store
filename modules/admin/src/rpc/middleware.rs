use crate::entities::admin_session::{AdminSession, AdminSessionId};
use framework::redis::{KeyValueRead, RedisConnection};
use tonic::codegen::BoxFuture;
use uuid::Uuid;

#[derive(Clone)]
pub struct AdminAuthLayer {
    redis: RedisConnection,
}

impl AdminAuthLayer {
    pub fn new(redis: RedisConnection) -> Self {
        Self { redis }
    }
}

impl<S> tower::Layer<S> for AdminAuthLayer {
    type Service = AdminAuthMiddleware<S>;
    fn layer(&self, inner: S) -> Self::Service {
        AdminAuthMiddleware {
            inner,
            redis: self.redis.clone(),
        }
    }
}

#[derive(Clone)]
pub struct AdminAuthMiddleware<S> {
    inner: S,
    redis: RedisConnection,
}

impl<S, ReqBody, ResBody> tower::Service<tonic::codegen::http::Request<ReqBody>>
    for AdminAuthMiddleware<S>
where
    S: tower::Service<
            tonic::codegen::http::Request<ReqBody>,
            Response = tonic::codegen::http::Response<ResBody>,
        > + Clone
        + Send
        + 'static,
    S::Future: Send + 'static,
    ReqBody: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = BoxFuture<Self::Response, Self::Error>;
    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }
    fn call(&mut self, mut req: tonic::codegen::http::Request<ReqBody>) -> Self::Future {
        let inner_clone = self.inner.clone();
        let mut inner = std::mem::replace(&mut self.inner, inner_clone);
        let redis = self.redis.clone();
        Box::pin(async move {
            match admin_auth(req.headers(), redis).await {
                Ok(admin_id) => {
                    req.extensions_mut().insert(admin_id);
                }
                Err(_) => {}
            };
            inner.call(req).await
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AdminId(pub Uuid);

impl AdminId {
    pub fn into_inner(self) -> Uuid {
        self.0
    }
    pub fn read_from_request<T>(req: &tonic::codegen::http::Request<T>) -> Option<Self> {
        req.extensions().get::<Self>().cloned()
    }
    pub fn from_request<T>(
        req: tonic::codegen::http::Request<T>,
    ) -> Result<(AdminId, T), tonic::Status> {
        let id = Self::read_from_request(&req).ok_or(tonic::Status::unauthenticated(
            "Missing admin id in request",
        ))?;
        Ok((id, req.into_body()))
    }
}

pub const ADMIN_AUTHORIZATION_HEADER: &str = "x-admin-authorization";

async fn admin_auth(
    header_map: &tonic::codegen::http::HeaderMap,
    mut redis: RedisConnection,
) -> Result<AdminId, tonic::Status> {
    let auth_header = header_map
        .get(ADMIN_AUTHORIZATION_HEADER)
        .and_then(|h| h.to_str().ok())
        .ok_or(tonic::Status::unauthenticated(
            "Missing admin authorization header",
        ))?;
    let admin_session_id = AdminSessionId::try_from_ascii_string(auth_header)
        .map_err(|_| tonic::Status::unauthenticated("Invalid admin session id format"))?;
    let session = AdminSession::read(&mut redis, admin_session_id)
        .await
        .map_err(|_| tonic::Status::internal("Internal server error"))?
        .ok_or(tonic::Status::unauthenticated("Invalid admin session"))?;
    Ok(AdminId(session.admin_id))
}
