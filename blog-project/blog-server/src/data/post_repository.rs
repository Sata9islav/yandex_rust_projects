use sqlx::{PgPool, Row};

use crate::domain::post::Post;

#[derive(Clone)]
pub struct PostRepository {
    pool: PgPool,
}

impl PostRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(
        &self,
        title: &str,
        content: &str,
        author_id: i64,
    ) -> Result<Post, sqlx::Error> {
        let row = sqlx::query(
            r#"
            INSERT INTO posts (title, content, author_id)
            VALUES ($1, $2, $3)
            RETURNING id, title, content, author_id, created_at, updated_at
            "#,
        )
        .bind(title)
        .bind(content)
        .bind(author_id)
        .fetch_one(&self.pool)
        .await?;

        Self::map_post(row)
    }

    pub async fn find_by_id(&self, id: i64) -> Result<Option<Post>, sqlx::Error> {
        let row = sqlx::query(
            r#"
            SELECT id, title, content, author_id, created_at, updated_at
            FROM posts
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        row.map(Self::map_post).transpose()
    }

    pub async fn update(
        &self,
        id: i64,
        author_id: i64,
        title: &str,
        content: &str,
    ) -> Result<Option<Post>, sqlx::Error> {
        let row = sqlx::query(
            r#"
            UPDATE posts
            SET title = $3,
                content = $4,
                updated_at = NOW()
            WHERE id = $1 and author_id = $2
            RETURNING id, title, content, author_id, created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(author_id)
        .bind(title)
        .bind(content)
        .fetch_optional(&self.pool)
        .await?;

        row.map(Self::map_post).transpose()
    }

    pub async fn delete(&self, id: i64, author_id: i64) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            r#"
            DELETE FROM posts
            WHERE id = $1 and author_id = $2
            "#,
        )
        .bind(id)
        .bind(author_id)
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn list(&self, limit: i64, offset: i64) -> Result<Vec<Post>, sqlx::Error> {
        let rows = sqlx::query(
            r#"
            SELECT id, title, content, author_id, created_at, updated_at
            FROM posts
            ORDER BY created_at DESC, id DESC
            LIMIT $1 OFFSET $2
            "#,
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(Self::map_post).collect()
    }

    pub async fn count(&self) -> Result<i64, sqlx::Error> {
        sqlx::query_scalar("SELECT COUNT(*) FROM posts")
            .fetch_one(&self.pool)
            .await
    }

    fn map_post(row: sqlx::postgres::PgRow) -> Result<Post, sqlx::Error> {
        Ok(Post {
            id: row.try_get("id")?,
            title: row.try_get("title")?,
            content: row.try_get("content")?,
            author_id: row.try_get("author_id")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }
}
