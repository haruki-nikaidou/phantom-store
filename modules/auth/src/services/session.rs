use crate::config::AuthConfig;
use crate::entities::redis::session::{Session, SessionId};
use crate::entities::redis::user_session_list::{UserSessionIndex, UserSessions};
use admin::utils::config_provider::find_config_from_redis;
use framework::now_time;
use framework::rabbitmq::AmqpPool;
use framework::redis::{KeyValue, KeyValueRead, KeyValueWrite, RedisConnection};
use kanau::processor::{Processor, parallel_map};
use tonic::codegen::tokio_stream::StreamExt;
use uuid::Uuid;

#[derive(Clone)]
pub struct SessionService {
    pub redis: RedisConnection,
    pub config_store: RedisConnection,
    pub mq: AmqpPool,
}

#[derive(Debug, Clone)]
pub struct CreateSession {
    pub user_id: Uuid,
}

impl Processor<CreateSession> for SessionService {
    type Output = SessionId;
    type Error = framework::Error;
    async fn process(&self, input: CreateSession) -> Result<SessionId, framework::Error> {
        let session_id = SessionId::generate();
        let now = now_time();
        let now_timestamp = now.assume_utc().unix_timestamp() as u64;
        let mut redis = self.redis.clone();
        let config = find_config_from_redis::<AuthConfig>(&mut redis)
            .await?
            .session;
        let session_ttl = config.session_ttl.try_into().map_err(|e| {
            framework::Error::BusinessPanic(anyhow::anyhow!("Invalid session ttl: {e}"))
        })?;
        let session = Session {
            id: session_id,
            user_id: input.user_id,
            terminated: false,
            last_refreshed: now_timestamp,
        };
        session.write_with_ttl(&mut redis, session_ttl).await?;
        Ok(session_id)
    }
}

#[derive(Debug, Clone)]
pub struct RefreshSession {
    pub session_id: SessionId,
}

impl Processor<RefreshSession> for SessionService {
    type Output = Session;
    type Error = framework::Error;
    async fn process(&self, input: RefreshSession) -> Result<Session, framework::Error> {
        let mut redis = self.redis.clone();
        let config = find_config_from_redis::<AuthConfig>(&mut redis)
            .await?
            .session;
        let session_ttl = config.session_ttl.try_into().map_err(|e| {
            framework::Error::BusinessPanic(anyhow::anyhow!("Invalid session ttl: {e}"))
        })?;
        let now = now_time().assume_utc().unix_timestamp() as u64;
        let mut session = Session::read(&mut redis, input.session_id)
            .await?
            .ok_or(framework::Error::NotFound)?;
        session.last_refreshed = now;
        session.write_with_ttl(&mut redis, session_ttl).await?;
        Ok(session)
    }
}

#[derive(Debug, Clone)]
pub struct TerminateSession {
    pub session_id: SessionId,
}

impl Processor<TerminateSession> for SessionService {
    type Output = ();
    type Error = framework::Error;
    async fn process(&self, input: TerminateSession) -> Result<(), framework::Error> {
        Session::delete(&mut self.redis.clone(), input.session_id).await?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct FetchSessionInfo {
    pub session_id: SessionId,
}

impl Processor<FetchSessionInfo> for SessionService {
    type Output = Session;
    type Error = framework::Error;
    async fn process(&self, input: FetchSessionInfo) -> Result<Session, framework::Error> {
        let session = Session::read(&mut self.redis.clone(), input.session_id)
            .await?
            .ok_or(framework::Error::NotFound)?;
        Ok(session)
    }
}

#[derive(Debug, Clone)]
pub struct ListUserSessions {
    pub user_id: Uuid,
}

impl Processor<ListUserSessions> for SessionService {
    type Output = Vec<Session>;
    type Error = framework::Error;
    async fn process(&self, input: ListUserSessions) -> Result<Vec<Session>, framework::Error> {
        let mut redis = self.redis.clone();
        let Some(session_ids) =
            UserSessions::read(&mut redis, UserSessionIndex(input.user_id)).await?
        else {
            return Ok(vec![]);
        };
        let mut sessions_stream = parallel_map(
            session_ids
                .session_ids
                .into_iter()
                .map(|id| FetchSessionInfo {
                    session_id: SessionId(id),
                }),
            self,
        );
        let mut sessions = Vec::new();
        while let Some(Ok(session)) = sessions_stream.next().await {
            sessions.push(session);
        }
        sessions.sort_by_key(|s| s.last_refreshed);
        Ok(sessions)
    }
}
