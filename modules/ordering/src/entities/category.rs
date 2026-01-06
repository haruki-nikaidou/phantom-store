use compact_str::CompactString;

#[derive(Debug, Clone, PartialEq, Eq, sqlx::FromRow)]
pub struct Category {
    pub id: i32,
    pub name: CompactString,
    pub parent_id: Option<i32>,
    pub description: String,
}
