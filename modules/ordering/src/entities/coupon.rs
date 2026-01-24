use framework::sqlx::DatabaseProcessor;
use kanau::processor::Processor;
use rust_decimal::Decimal;
use tracing::instrument;

#[derive(Debug, Clone, PartialEq, Eq, sqlx::FromRow)]
pub struct Coupon {
    pub id: i32,
    pub code: String,
    pub set_active: bool,
    pub discount: sqlx::types::Json<Discount>,
    pub available_since: Option<time::PrimitiveDateTime>,
    pub available_until: Option<time::PrimitiveDateTime>,
    pub limit_to_category: Option<i32>,
    pub limit_per_user: Option<i32>,
    pub limit_total: Option<i32>,
    pub used_count: i32,
    pub created_at: time::PrimitiveDateTime,
    pub updated_at: time::PrimitiveDateTime,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(tag = "type")]
pub enum Discount {
    Rate(RateDiscount),
    Amount(AmountDiscount),
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct RateDiscount {
    pub rate: Decimal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct AmountDiscount {
    pub min_amount: Decimal,
    pub discount: Decimal,
}

#[derive(Debug, Clone)]
pub struct FindCouponByCode {
    pub code: String,
}

impl Processor<FindCouponByCode> for DatabaseProcessor {
    type Output = Option<Coupon>;
    type Error = sqlx::Error;
    #[instrument(skip_all, name = "SQL:FindCouponByCode", err)]
    async fn process(&self, input: FindCouponByCode) -> Result<Option<Coupon>, sqlx::Error> {
        sqlx::query_as!(
            Coupon,
            r#"
            SELECT
            id, code, set_active,
            discount as "discount: sqlx::types::Json<Discount>",
            available_since, available_until, limit_to_category, limit_per_user, limit_total,
            used_count, created_at, updated_at
            FROM "shop"."coupon"
            WHERE code = $1
            "#,
            &input.code
        )
        .fetch_optional(self.db())
        .await
    }
}

#[derive(Debug, Clone)]
pub struct FindCouponById {
    pub id: i32,
}

impl Processor<FindCouponById> for DatabaseProcessor {
    type Output = Option<Coupon>;
    type Error = sqlx::Error;
    #[instrument(skip_all, name = "SQL:FindCouponById", err)]
    async fn process(&self, input: FindCouponById) -> Result<Option<Coupon>, sqlx::Error> {
        sqlx::query_as!(
            Coupon,
            r#"
            SELECT
            id, code, set_active,
            discount as "discount: sqlx::types::Json<Discount>",
            available_since, available_until, limit_to_category, limit_per_user, limit_total,
            used_count, created_at, updated_at
            FROM "shop"."coupon"
            WHERE id = $1
            "#,
            input.id
        )
        .fetch_optional(self.db())
        .await
    }
}

#[derive(Debug, Clone)]
pub struct CreateNewCoupon {
    pub code: String,
    pub set_active: bool,
    pub discount: Discount,
    pub available_since: Option<time::PrimitiveDateTime>,
    pub available_until: Option<time::PrimitiveDateTime>,
    pub limit_to_category: Option<i32>,
    pub limit_per_user: Option<i32>,
    pub limit_total: Option<i32>,
}

impl Processor<CreateNewCoupon> for DatabaseProcessor {
    type Output = Coupon;
    type Error = sqlx::Error;
    #[instrument(skip_all, name = "SQL:CreateNewCoupon", err)]
    async fn process(&self, input: CreateNewCoupon) -> Result<Coupon, sqlx::Error> {
        sqlx::query_as!(
            Coupon,
            r#"
            INSERT INTO "shop"."coupon" (code, set_active, discount, available_since, available_until, limit_to_category, limit_per_user, limit_total)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING
            id, code, set_active,
            discount as "discount: sqlx::types::Json<Discount>",
            available_since, available_until, limit_to_category, limit_per_user, limit_total,
            used_count, created_at, updated_at
            "#,
            &input.code,
            input.set_active,
            sqlx::types::Json(input.discount) as sqlx::types::Json<Discount>,
            input.available_since,
            input.available_until,
            input.limit_to_category,
            input.limit_per_user,
            input.limit_total
        )
        .fetch_one(self.db())
        .await
    }
}

#[derive(Debug, Clone)]
pub struct UpdateCoupon {
    pub id: i32,
    pub discount: Discount,
    pub available_since: Option<time::PrimitiveDateTime>,
    pub available_until: Option<time::PrimitiveDateTime>,
    pub limit_to_category: Option<i32>,
    pub limit_per_user: Option<i32>,
    pub limit_total: Option<i32>,
}

impl Processor<UpdateCoupon> for DatabaseProcessor {
    type Output = Coupon;
    type Error = sqlx::Error;
    #[instrument(skip_all, name = "SQL:UpdateCoupon", err)]
    async fn process(&self, input: UpdateCoupon) -> Result<Coupon, sqlx::Error> {
        sqlx::query_as!(
            Coupon,
            r#"
            UPDATE "shop"."coupon"
            SET discount = $2, available_since = $3, available_until = $4, limit_to_category = $5, limit_per_user = $6, limit_total = $7
            WHERE id = $1
            RETURNING
            id, code, set_active,
            discount as "discount: sqlx::types::Json<Discount>",
            available_since, available_until, limit_to_category, limit_per_user, limit_total,
            used_count, created_at, updated_at
            "#,
            input.id,
            sqlx::types::Json(input.discount) as sqlx::types::Json<Discount>,
            input.available_since,
            input.available_until,
            input.limit_to_category,
            input.limit_per_user,
            input.limit_total
        )
        .fetch_one(self.db())
        .await
    }
}

#[derive(Debug, Clone)]
pub struct DisableOrEnableCoupon {
    pub id: i32,
    pub set_active: bool,
}

impl Processor<DisableOrEnableCoupon> for DatabaseProcessor {
    type Output = Coupon;
    type Error = sqlx::Error;
    #[instrument(skip_all, name = "SQL:DisableOrEnableCoupon", err)]
    async fn process(&self, input: DisableOrEnableCoupon) -> Result<Coupon, sqlx::Error> {
        sqlx::query_as!(
            Coupon,
            r#"
            UPDATE "shop"."coupon"
            SET set_active = $2
            WHERE id = $1
            RETURNING
            id, code, set_active,
            discount as "discount: sqlx::types::Json<Discount>",
            available_since, available_until, limit_to_category, limit_per_user, limit_total,
            used_count, created_at, updated_at
            "#,
            input.id,
            input.set_active
        )
        .fetch_one(self.db())
        .await
    }
}
