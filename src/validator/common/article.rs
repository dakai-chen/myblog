use bytesize::ByteSize;

use crate::validator::ValidationError;

pub fn validate_unlock_password(password: &str) -> Result<(), ValidationError> {
    if password.is_empty() {
        return Err(ValidationError::validation("文章解锁密码不能为空"));
    }
    if password.len() < 6 {
        return Err(ValidationError::validation("文章解锁密码长度不能少于6位"));
    }
    if password.len() > 32 {
        return Err(ValidationError::validation("文章解锁密码长度不能大于32位"));
    }
    if !password.chars().all(|c| c.is_ascii_alphanumeric()) {
        return Err(ValidationError::validation(
            "文章解锁密码仅支持字母和数字组合",
        ));
    }
    Ok(())
}

pub fn validate_title(title: &str) -> Result<(), ValidationError> {
    if title.trim().is_empty() {
        return Err(ValidationError::validation("文章标题不能为空或仅包含空格"));
    }
    let len = title.chars().count();
    if len > crate::config::get().article.title_max_length {
        return Err(ValidationError::validation(format!(
            "文章标题长度超出限制，最大允许 {} 个字符",
            crate::config::get().article.title_max_length
        )));
    }
    Ok(())
}

pub fn validate_markdown_content(markdown_content: &str) -> Result<(), ValidationError> {
    if markdown_content.len() > crate::config::get().article.content_max_size {
        return Err(ValidationError::validation(format!(
            "正文长度超出限制，最大允许 {}",
            u64::try_from(crate::config::get().article.content_max_size)
                .map(ByteSize::b)
                .map_err(ValidationError::internal)?
        )));
    }
    Ok(())
}

pub fn validate_full_text(full_text: &str) -> Result<(), ValidationError> {
    let len = full_text.chars().count();
    if len > 200 {
        return Err(ValidationError::validation(
            "搜索内容长度不能超过 200 个字符",
        ));
    }
    Ok(())
}
