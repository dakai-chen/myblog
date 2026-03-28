use serde::Serialize;

use crate::model::bo::article::ArticleDetailBo;
use crate::model::vo::article::ArticleAttachmentVo;
use crate::template::render::TemplateRenderData;

/// 关于页面
#[derive(Debug, Clone, Serialize)]
pub struct AboutVo {
    /// 文章ID
    pub article_id: Option<String>,
    /// Markdown 格式的正文
    pub markdown_content: String,
    /// 渲染后的 HTML 结果
    pub render_content: String,
    /// 附件列表
    pub attachments: Vec<ArticleAttachmentVo>,
}

impl Default for AboutVo {
    fn default() -> Self {
        Self {
            article_id: None,
            markdown_content: String::new(),
            render_content: String::new(),
            attachments: vec![],
        }
    }
}

impl From<ArticleDetailBo> for AboutVo {
    fn from(value: ArticleDetailBo) -> Self {
        match value {
            ArticleDetailBo::Visitor(bo) => Self {
                article_id: Some(bo.article_id),
                markdown_content: bo.markdown_content,
                render_content: bo.render_content,
                attachments: bo
                    .attachments
                    .into_iter()
                    .map(ArticleAttachmentVo::from)
                    .collect(),
            },
            ArticleDetailBo::Admin(bo) => Self {
                article_id: Some(bo.article_id),
                markdown_content: bo.markdown_content,
                render_content: bo.render_content,
                attachments: bo
                    .attachments
                    .into_iter()
                    .map(ArticleAttachmentVo::from)
                    .collect(),
            },
        }
    }
}

impl TemplateRenderData for AboutVo {
    fn template_name() -> &'static str {
        "about.html"
    }
}
