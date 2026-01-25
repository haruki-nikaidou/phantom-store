#![forbid(unsafe_code, clippy::unwrap_used, clippy::panic, clippy::expect_used)]

pub mod cron;
pub mod error;
pub mod pool;
pub mod rabbitmq;
pub mod redis;
pub mod sqlx;

pub use error::Error;

pub fn now_time() -> time::PrimitiveDateTime {
    let now = time::OffsetDateTime::now_utc();
    time::PrimitiveDateTime::new(now.date(), now.time())
}
