use crate::model::dto::api::article::{
    CreateArticleDto, DownloadArticleAttachmentDto, GetArticleDto, RemoveArticleAttachmentDto,
    RemoveArticleDto, SearchArticleDto, UnlockArticleDto, UpdateArticleDto,
    UploadArticleAttachmentDto,
};
use crate::validator::{Validation, ValidationError};

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

impl Validation<()> for CreateArticleDto {
    fn validate(&self, _context: &()) -> Result<(), ValidationError> {
        if let Some(password) = &self.password {
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
        if let Some(password) = &self.password {
            crate::validator::common::article::validate_unlock_password(password)?;
        }
        crate::validator::common::article::validate_title(&self.title)?;
        crate::validator::common::article::validate_markdown_content(&self.markdown_content)?;
        Ok(())
    }
}

impl Validation<()> for RemoveArticleDto {
    fn validate(&self, _context: &()) -> Result<(), ValidationError> {
        if self.article_id.is_empty() {
            return Err(ValidationError::validation("文章ID不能为空"));
        }
        Ok(())
    }
}

impl Validation<()> for SearchArticleDto {
    fn validate(&self, _context: &()) -> Result<(), ValidationError> {
        if let Some(full_text) = &self.full_text {
            crate::validator::common::article::validate_full_text(full_text)?;
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

impl Validation<()> for UploadArticleAttachmentDto {
    fn validate(&self, context: &()) -> Result<(), ValidationError> {
        if self.article_id.is_empty() {
            return Err(ValidationError::validation("文章ID不能为空"));
        }
        self.attachment.validate(context)?;
        Ok(())
    }
}

impl Validation<()> for RemoveArticleAttachmentDto {
    fn validate(&self, _context: &()) -> Result<(), ValidationError> {
        if self.attachment_id.is_empty() {
            return Err(ValidationError::validation("附件ID不能为空"));
        }
        Ok(())
    }
}

impl Validation<()> for DownloadArticleAttachmentDto {
    fn validate(&self, _context: &()) -> Result<(), ValidationError> {
        if self.attachment_id.is_empty() {
            return Err(ValidationError::validation("附件ID不能为空"));
        }
        Ok(())
    }
}
