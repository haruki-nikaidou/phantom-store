use crate::entities::redis::session::SessionId;
use crate::services::session::{RefreshSession, SessionService};
use kanau::processor::Processor;
use std::sync::Arc;
use tonic::codegen::BoxFuture;
use uuid::Uuid;

#[derive(Clone)]
pub struct UserAuthLayer {
    session_service: Arc<SessionService>,
}

impl UserAuthLayer {
    pub fn new(session_service: Arc<SessionService>) -> Self {
        Self { session_service }
    }
}

impl<S> tower::Layer<S> for UserAuthLayer {
    type Service = UserAuthMiddleware<S>;
    fn layer(&self, inner: S) -> Self::Service {
        UserAuthMiddleware {
            inner,
            session_service: self.session_service.clone(),
        }
    }
}

#[derive(Clone)]
pub struct UserAuthMiddleware<S> {
    inner: S,
    session_service: Arc<SessionService>,
}

impl<S, ReqBody, ResBody> tower::Service<tonic::codegen::http::Request<ReqBody>>
    for UserAuthMiddleware<S>
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
        let session_service = self.session_service.clone();
        let inner_clone = self.inner.clone();
        let mut inner = std::mem::replace(&mut self.inner, inner_clone);
        Box::pin(async move {
            if let Ok(user_id) = user_auth(req.headers(), session_service).await {
                req.extensions_mut().insert(user_id);
            }
            inner.call(req).await
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UserId(pub Uuid);

pub const SESSION_ID_HEADER: &str = "x-user-authorization";

impl UserId {
    pub fn into_inner(self) -> Uuid {
        self.0
    }
    pub fn read_from_request<T>(req: &tonic::Request<T>) -> Result<Self, tonic::Status> {
        req.extensions()
            .get::<Self>()
            .map(|id| id.to_owned())
            .ok_or_else(|| tonic::Status::unauthenticated("Missing Identity"))
    }
    pub fn from_request<T>(req: tonic::Request<T>) -> Result<(UserId, T), tonic::Status> {
        let id = Self::read_from_request(&req)?;
        let req = req.into_inner();
        Ok((id, req))
    }
}

async fn user_auth(
    headers: &tonic::codegen::http::HeaderMap,
    session_service: Arc<SessionService>,
) -> Result<UserId, tonic::Status> {
    let header = headers
        .get(SESSION_ID_HEADER)
        .and_then(|h| h.to_str().ok())
        .ok_or(tonic::Status::unauthenticated(
            "Missing authorization header",
        ))?;
    let session_id = SessionId::try_from_ascii_string(header)
        .map_err(|_| tonic::Status::unauthenticated("Invalid session id format"))?;
    let session = session_service
        .process(RefreshSession { session_id })
        .await
        .map_err(|_| tonic::Status::unauthenticated("Invalid session"))?;
    Ok(UserId(session.user_id))
}
