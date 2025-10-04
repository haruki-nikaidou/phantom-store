#[derive(Debug, Clone)]
pub struct DatabaseProcessor {
    executor: sqlx::PgPool,
}

impl DatabaseProcessor {
    pub fn new(executor: sqlx::PgPool) -> Self {
        Self { executor }
    }
}

impl DatabaseProcessor {
    pub fn db(&self) -> &sqlx::PgPool {
        &self.executor
    }

    pub fn executor(&self) -> &sqlx::PgPool {
        &self.executor
    }
}

impl DatabaseProcessor {
    pub fn new_static(pool: sqlx::PgPool) -> DatabaseProcessor {
        DatabaseProcessor::new(pool)
    }
}

impl DatabaseProcessor {
    pub fn from_pool(pool: sqlx::PgPool) -> Self {
        Self::new(pool)
    }
}
