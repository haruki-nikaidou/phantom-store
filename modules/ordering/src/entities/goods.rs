use framework::sqlx::DatabaseProcessor;
use kanau::processor::Processor;
use tracing::instrument;

#[derive(Debug, Clone, PartialEq, Eq, sqlx::FromRow)]
pub struct Goods {
    pub id: i32,
    pub name: String,
    pub description: String,
    pub pictures: Vec<String>,
    pub price: rust_decimal::Decimal,
    pub category_id: Option<i32>,
    pub on_sale: bool,
    pub stock: i32,
}

#[derive(Debug, Clone, Copy)]
pub struct FindGoodsById {
    pub id: i32,
}

impl Processor<FindGoodsById> for DatabaseProcessor {
    type Output = Option<Goods>;
    type Error = sqlx::Error;
    #[instrument(skip_all, name = "SQL:FindGoodsById", err)]
    async fn process(&self, input: FindGoodsById) -> Result<Option<Goods>, sqlx::Error> {
        sqlx::query_as!(
            Goods,
            r#"
            SELECT id, name, description, pictures, price, category_id, on_sale, stock
            FROM "shop"."goods"
            WHERE id = $1
            "#,
            input.id
        )
        .fetch_optional(self.db())
        .await
    }
}

#[derive(Debug, Clone)]
pub struct ListGoodsUnderCategory {
    pub category_id: i32,
}

impl Processor<ListGoodsUnderCategory> for DatabaseProcessor {
    type Output = Vec<Goods>;
    type Error = sqlx::Error;
    #[instrument(skip_all, name = "SQL:ListGoodsUnderCategory", err)]
    async fn process(&self, input: ListGoodsUnderCategory) -> Result<Vec<Goods>, sqlx::Error> {
        sqlx::query_as!(
            Goods,
            r#"
            SELECT id, name, description, pictures, price, category_id, on_sale, stock
            FROM "shop"."goods"
            WHERE category_id = $1
            "#,
            input.category_id
        )
        .fetch_all(self.db())
        .await
    }
}

#[derive(Debug, Clone)]
pub struct CreateNewGoods {
    pub name: String,
    pub description: String,
    pub pictures: Vec<String>,
    pub price: rust_decimal::Decimal,
    pub category_id: Option<i32>,
    pub on_sale: bool,
    pub stock: i32,
}

impl Processor<CreateNewGoods> for DatabaseProcessor {
    type Output = Goods;
    type Error = sqlx::Error;
    #[instrument(skip_all, name = "SQL:CreateNewGoods", err)]
    async fn process(&self, input: CreateNewGoods) -> Result<Goods, sqlx::Error> {
        sqlx::query_as!(
            Goods,
            r#"
            INSERT INTO "shop"."goods" (name, description, pictures, price, category_id, on_sale, stock)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, name, description, pictures, price, category_id, on_sale, stock
            "#,
            &input.name,
            &input.description,
            &input.pictures,
            input.price,
            input.category_id,
            input.on_sale,
            input.stock
        )
        .fetch_one(self.db())
        .await
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DecreaseGoodsStock {
    pub id: i32,
    pub amount: i32,
}

impl Processor<DecreaseGoodsStock> for DatabaseProcessor {
    type Output = Goods;
    type Error = sqlx::Error;
    #[instrument(skip_all, name = "SQL:DecreaseGoodsStock", err)]
    async fn process(&self, input: DecreaseGoodsStock) -> Result<Goods, sqlx::Error> {
        sqlx::query_as!(
            Goods,
            r#"
            UPDATE "shop"."goods"
            SET stock = stock - $2
            WHERE id = $1
            RETURNING id, name, description, pictures, price, category_id, on_sale, stock
            "#,
            input.id,
            input.amount
        )
        .fetch_one(self.db())
        .await
    }
}

#[derive(Debug, Clone, Copy)]
pub struct IncreaseGoodsStock {
    pub id: i32,
    pub amount: i32,
}

impl Processor<IncreaseGoodsStock> for DatabaseProcessor {
    type Output = Goods;
    type Error = sqlx::Error;
    #[instrument(skip_all, name = "SQL:IncreaseGoodsStock", err)]
    async fn process(&self, input: IncreaseGoodsStock) -> Result<Goods, sqlx::Error> {
        sqlx::query_as!(
            Goods,
            r#"
            UPDATE "shop"."goods"
            SET stock = stock + $2
            WHERE id = $1
            RETURNING id, name, description, pictures, price, category_id, on_sale, stock
            "#,
            input.id,
            input.amount
        )
        .fetch_one(self.db())
        .await
    }
}

#[derive(Debug, Clone)]
pub struct UpdateGoods {
    pub id: i32,
    pub name: String,
    pub description: String,
    pub pictures: Vec<String>,
    pub price: rust_decimal::Decimal,
    pub category_id: Option<i32>,
    pub on_sale: bool,
}

impl Processor<UpdateGoods> for DatabaseProcessor {
    type Output = Goods;
    type Error = sqlx::Error;
    #[instrument(skip_all, name = "SQL:UpdateGoods", err)]
    async fn process(&self, input: UpdateGoods) -> Result<Goods, sqlx::Error> {
        sqlx::query_as!(
            Goods,
            r#"
            UPDATE "shop"."goods"
            SET name = $2, description = $3, pictures = $4, price = $5, category_id = $6, on_sale = $7
            WHERE id = $1
            RETURNING id, name, description, pictures, price, category_id, on_sale, stock
            "#,
            input.id,
            &input.name,
            &input.description,
            &input.pictures,
            input.price,
            input.category_id,
            input.on_sale
        )
        .fetch_one(self.db())
        .await
    }
}
