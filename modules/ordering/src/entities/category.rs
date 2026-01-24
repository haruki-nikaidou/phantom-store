use compact_str::CompactString;
use framework::sqlx::DatabaseProcessor;
use kanau::processor::Processor;
use tracing::instrument;

#[derive(Debug, Clone, PartialEq, Eq, sqlx::FromRow, serde::Deserialize, serde::Serialize)]
pub struct Category {
    pub id: i32,
    pub name: CompactString,
    pub parent_id: Option<i32>,
    pub description: String,
}

#[derive(Debug, Clone, Copy)]
pub struct CheckCategoryRelation {
    pub category_id: i32,
}

#[derive(Debug, Clone, Copy)]
pub struct CheckCategoryRelationResult {
    pub has_parent: bool,
    pub has_children: bool,
    pub has_goods: bool,
    pub has_coupons: bool,
}

impl Processor<CheckCategoryRelation> for DatabaseProcessor {
    type Output = CheckCategoryRelationResult;
    type Error = sqlx::Error;
    #[instrument(skip_all, name = "SQL:CheckCategoryRelation", err)]
    async fn process(
        &self,
        input: CheckCategoryRelation,
    ) -> Result<CheckCategoryRelationResult, sqlx::Error> {
        sqlx::query_file_as!(
            CheckCategoryRelationResult,
            "sql/check_category_relation.sql",
            input.category_id
        )
        .fetch_one(self.db())
        .await
    }
}

#[derive(Debug, Clone, Copy)]
/// List all level parents and direct children of a category.
pub struct ShowCategoryParentsAndChildren {
    pub category_id: i32,
}

#[derive(Debug, Clone)]
pub struct ShowCategoryParentsAndChildrenResult {
    pub parents: Vec<Category>,
    pub children: Vec<Category>,
}

impl Processor<ShowCategoryParentsAndChildren> for DatabaseProcessor {
    type Output = ShowCategoryParentsAndChildrenResult;
    type Error = sqlx::Error;
    #[instrument(skip_all, name = "SQL:ShowCategoryParentsAndChildren", err)]
    async fn process(
        &self,
        input: ShowCategoryParentsAndChildren,
    ) -> Result<ShowCategoryParentsAndChildrenResult, sqlx::Error> {
        let (parents, children) = tokio::try_join!(
            sqlx::query_file_as!(Category, "sql/show_category_parents.sql", input.category_id)
                .fetch_all(self.db()),
            sqlx::query_file_as!(
                Category,
                "sql/show_category_children.sql",
                input.category_id
            )
            .fetch_all(self.db())
        )?;

        Ok(ShowCategoryParentsAndChildrenResult { parents, children })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct FindCategoryById {
    pub id: i32,
}

impl Processor<FindCategoryById> for DatabaseProcessor {
    type Output = Option<Category>;
    type Error = sqlx::Error;
    #[instrument(skip_all, name = "SQL:FindCategoryById", err)]
    async fn process(&self, input: FindCategoryById) -> Result<Option<Category>, sqlx::Error> {
        sqlx::query_as!(
            Category,
            r#"
            SELECT id, name, parent_id, description
            FROM "shop"."category"
            WHERE id = $1
            "#,
            input.id
        )
        .fetch_optional(self.db())
        .await
    }
}

#[derive(Debug, Clone)]
pub struct CreateNewCategory {
    pub name: String,
    pub parent_id: Option<i32>,
    pub description: String,
}

impl Processor<CreateNewCategory> for DatabaseProcessor {
    type Output = Category;
    type Error = sqlx::Error;
    #[instrument(skip_all, name = "SQL:CreateNewCategory", err)]
    async fn process(&self, input: CreateNewCategory) -> Result<Category, sqlx::Error> {
        sqlx::query_as!(
            Category,
            r#"
            INSERT INTO "shop"."category" (name, parent_id, description)
            VALUES ($1, $2, $3)
            RETURNING id, name, parent_id, description
            "#,
            &input.name,
            input.parent_id,
            &input.description
        )
        .fetch_one(self.db())
        .await
    }
}

#[derive(Debug, Clone)]
pub struct UpdateCategory {
    pub id: i32,
    pub name: String,
    pub parent_id: Option<i32>,
    pub description: String,
}

impl Processor<UpdateCategory> for DatabaseProcessor {
    type Output = Category;
    type Error = sqlx::Error;
    #[instrument(skip_all, name = "SQL:UpdateCategory", err)]
    async fn process(&self, input: UpdateCategory) -> Result<Category, sqlx::Error> {
        sqlx::query_as!(
            Category,
            r#"
            UPDATE "shop"."category"
            SET name = $2, parent_id = $3, description = $4
            WHERE id = $1
            RETURNING id, name, parent_id, description
            "#,
            input.id,
            &input.name,
            input.parent_id,
            &input.description
        )
        .fetch_one(self.db())
        .await
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DeleteCategory {
    pub id: i32,
}

impl Processor<DeleteCategory> for DatabaseProcessor {
    type Output = bool;
    type Error = sqlx::Error;
    #[instrument(skip_all, name = "SQL:DeleteCategory", err)]
    async fn process(&self, input: DeleteCategory) -> Result<bool, sqlx::Error> {
        let delete_result = sqlx::query!(
            r#"
            DELETE FROM "shop"."category"
            WHERE id = $1
            "#,
            input.id
        )
        .execute(self.db())
        .await?;
        Ok(delete_result.rows_affected() == 1)
    }
}
