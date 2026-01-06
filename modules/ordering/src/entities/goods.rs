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
