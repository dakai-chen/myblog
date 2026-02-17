use crate::model::dto::web::article::{
    CreateArticleSubmitDto, GetArticleDto, SearchArticleDto, UnlockArticleDto, UpdateArticleDto,
    UpdateArticleSubmitDto,
};
use crate::validator::{Validation, ValidationError};

impl Validation<()> for SearchArticleDto {
    fn validate(&self, _context: &()) -> Result<(), ValidationError> {
        if let Some(q) = &self.q {
            crate::validator::common::article::validate_full_text(q)?;
        }
        Ok(())
    }
}

impl Validation<()> for GetArticleDto {
    fn validate(&self, _context: &()) -> Result<(), ValidationError> {
        if self.article_id.is_empty() {
            return Err(ValidationError::validation("文章ID不能为空"));
        }
        Ok(())
    }
}

impl Validation<()> for UnlockArticleDto {
    fn validate(&self, _context: &()) -> Result<(), ValidationError> {
        if self.article_id.is_empty() {
            return Err(ValidationError::validation("文章ID不能为空"));
        }
        if self.password.len() > 32 {
            return Err(ValidationError::validation("文章访问密码太长"));
        }
        Ok(())
    }
}

impl Validation<()> for CreateArticleSubmitDto {
    fn validate(&self, _context: &()) -> Result<(), ValidationError> {
        if let Some(password) = self.password.as_ref().filter(|v| !v.is_empty()) {
            crate::validator::common::article::validate_unlock_password(password)?;
        }
        crate::validator::common::article::validate_title(&self.title)?;
        crate::validator::common::article::validate_markdown_content(&self.markdown_content)?;
        Ok(())
    }
}

impl Validation<()> for UpdateArticleDto {
    fn validate(&self, _context: &()) -> Result<(), ValidationError> {
        if self.article_id.is_empty() {
            return Err(ValidationError::validation("文章ID不能为空"));
        }
        Ok(())
    }
}

impl Validation<()> for UpdateArticleSubmitDto {
    fn validate(&self, _context: &()) -> Result<(), ValidationError> {
        if self.article_id.is_empty() {
            return Err(ValidationError::validation("文章ID不能为空"));
        }
        if let Some(password) = self.password.as_ref().filter(|v| !v.is_empty()) {
            crate::validator::common::article::validate_unlock_password(password)?;
        }
        crate::validator::common::article::validate_title(&self.title)?;
        crate::validator::common::article::validate_markdown_content(&self.markdown_content)?;
        Ok(())
    }
}
