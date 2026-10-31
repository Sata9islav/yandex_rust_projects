use crate::{
    data::post_repository::PostRepository,
    domain::{
        error::DomainError,
        post::{CreatePostRequest, Post, UpdatePostRequest},
    },
};

pub struct BlogService {
    post_repository: PostRepository,
}

impl BlogService {
    pub fn new(post_repository: PostRepository) -> Self {
        Self { post_repository }
    }

    pub async fn create_post(
        &self,
        author_id: i64,
        request: CreatePostRequest,
    ) -> Result<Post, DomainError> {
        if request.title.trim().is_empty() {
            return Err(DomainError::InvalidInput(
                "Post title can not be empty".into(),
            ));
        }

        self.post_repository
            .create(&request.title, &request.content, author_id)
            .await
            .map_err(|_| DomainError::Internal)
    }

    pub async fn get_post(&self, id: i64) -> Result<Post, DomainError> {
        self.post_repository
            .find_by_id(id)
            .await
            .map_err(|_| DomainError::Internal)?
            .ok_or(DomainError::PostNotFound)
    }

    pub async fn update_post(
        &self,
        id: i64,
        author_id: i64,
        request: UpdatePostRequest,
    ) -> Result<Post, DomainError> {
        if request.title.trim().is_empty() {
            return Err(DomainError::InvalidInput(
                "POST title can not be empty".into(),
            ));
        }
        let post = self.get_post(id).await?;
        if post.author_id != author_id {
            return Err(DomainError::Forbidden);
        }

        self.post_repository
            .update(id, author_id, &request.title, &request.content)
            .await
            .map_err(|_| DomainError::Internal)?
            .ok_or(DomainError::PostNotFound)
    }

    pub async fn delete_post(&self, id: i64, author_id: i64) -> Result<(), DomainError> {
        let post = self.get_post(id).await?;

        if post.author_id != author_id {
            return Err(DomainError::Forbidden);
        }

        let deleted = self
            .post_repository
            .delete(id, author_id)
            .await
            .map_err(|_| DomainError::Internal)?;

        if !deleted {
            return Err(DomainError::PostNotFound);
        }

        Ok(())
    }

    pub async fn list_posts(
        &self,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<Post>, i64), DomainError> {
        if !(1..=100).contains(&limit) || offset < 0 {
            return Err(DomainError::InvalidInput(
                "Invalid pagination parameters".into(),
            ));
        }
        let posts = self
            .post_repository
            .list(limit, offset)
            .await
            .map_err(|_| DomainError::Internal)?;

        let total = self
            .post_repository
            .count()
            .await
            .map_err(|_| DomainError::Internal)?;

        Ok((posts, total))
    }
}

#[cfg(test)]
#[path = "tests_blog_service.rs"]
mod tests;
