use boluo::BoxError;
use boluo::data::Form;
use boluo::extract::{FromRequest, Path};
use boluo::request::Request;
use serde::{Deserialize, Serialize};

use crate::model::bo::article::{
    CreateArticleBo, GetArticleBo, SearchArticleBo, UnlockArticleBo, UpdateArticleBo,
};
use crate::model::common::article::ArticleStatus;

/// 搜索文章
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SearchArticleDto {
    /// 全文搜索
    pub q: Option<String>,
    /// 分页页码
    pub page: Option<u64>,
    /// 分页大小
    pub size: Option<u64>,
}

impl<'a> Into<SearchArticleBo<'a>> for SearchArticleDto {
    fn into(self) -> SearchArticleBo<'a> {
        SearchArticleBo {
            full_text: self.q.map(Into::into),
            status: Some(ArticleStatus::Published),
            published_at_ge: None,
            published_at_lt: None,
            page: self.page,
            size: self.size,
            sort: None,
        }
    }
}

impl<'a> Into<SearchArticleBo<'a>> for &'a SearchArticleDto {
    fn into(self) -> SearchArticleBo<'a> {
        SearchArticleBo {
            full_text: self.q.as_ref().map(|v| v.as_str().into()),
            status: None,
            published_at_ge: None,
            published_at_lt: None,
            page: self.page,
            size: self.size,
            sort: None,
        }
    }
}

/// 获取文章详情
#[derive(Debug, Clone, Deserialize)]
pub struct GetArticleDto {
    /// 文章ID
    pub article_id: String,
}

impl<'a> Into<GetArticleBo<'a>> for GetArticleDto {
    fn into(self) -> GetArticleBo<'a> {
        GetArticleBo {
            article_id: self.article_id.into(),
            ignore_status: false,
        }
    }
}

/// 解锁文章
#[derive(Debug, Clone, Deserialize)]
pub struct UnlockArticleDto {
    /// 文章ID
    pub article_id: String,
    /// 文章访问密码
    pub password: String,
}

impl<'a> Into<UnlockArticleBo<'a>> for UnlockArticleDto {
    fn into(self) -> UnlockArticleBo<'a> {
        UnlockArticleBo {
            article_id: self.article_id.into(),
            password: self.password.into(),
        }
    }
}

impl FromRequest for UnlockArticleDto {
    type Error = BoxError;

    async fn from_request(request: &mut Request) -> Result<Self, Self::Error> {
        #[derive(Deserialize)]
        struct PathParams {
            article_id: String,
        }
        #[derive(Deserialize)]
        struct FormParams {
            password: String,
        }

        let Path(path_params) = Path::<PathParams>::from_request(request).await?;
        let Form(form_params) = Form::<FormParams>::from_request(request).await?;

        Ok(Self {
            article_id: path_params.article_id,
            password: form_params.password,
        })
    }
}

/// 创建文章
#[derive(Debug, Clone, Deserialize)]
pub struct CreateArticleSubmitDto {
    /// 标题
    pub title: String,
    /// Markdown 格式的正文
    pub markdown_content: String,
    /// 访问密码
    pub password: Option<String>,
    /// 状态
    pub status: ArticleStatus,
}

impl Into<CreateArticleBo> for CreateArticleSubmitDto {
    fn into(self) -> CreateArticleBo {
        CreateArticleBo {
            title: self.title,
            markdown_content: self.markdown_content,
            password: self.password.filter(|v| !v.is_empty()),
            status: self.status,
        }
    }
}

/// 修改文章
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateArticleDto {
    /// 文章ID
    pub article_id: String,
}

impl<'a> Into<GetArticleBo<'a>> for UpdateArticleDto {
    fn into(self) -> GetArticleBo<'a> {
        GetArticleBo {
            article_id: self.article_id.into(),
            ignore_status: false,
        }
    }
}

/// 修改文章
#[derive(Debug, Clone)]
pub struct UpdateArticleSubmitDto {
    /// 文章ID
    pub article_id: String,
    /// 标题
    pub title: String,
    /// Markdown 格式的正文
    pub markdown_content: String,
    /// 访问密码
    pub password: Option<String>,
    /// 状态
    pub status: ArticleStatus,
}

impl Into<UpdateArticleBo> for UpdateArticleSubmitDto {
    fn into(self) -> UpdateArticleBo {
        UpdateArticleBo {
            article_id: self.article_id,
            title: self.title,
            markdown_content: self.markdown_content,
            password: self.password.filter(|v| !v.is_empty()),
            status: self.status,
        }
    }
}

impl FromRequest for UpdateArticleSubmitDto {
    type Error = BoxError;

    async fn from_request(request: &mut Request) -> Result<Self, Self::Error> {
        #[derive(Deserialize)]
        struct PathParams {
            article_id: String,
        }
        #[derive(Deserialize)]
        struct FormParams {
            pub title: String,
            pub markdown_content: String,
            pub password: Option<String>,
            pub status: ArticleStatus,
        }

        let Path(path_params) = Path::<PathParams>::from_request(request).await?;
        let Form(form_params) = Form::<FormParams>::from_request(request).await?;

        Ok(Self {
            article_id: path_params.article_id,
            title: form_params.title,
            markdown_content: form_params.markdown_content,
            password: form_params.password,
            status: form_params.status,
        })
    }
}
