pub mod helper;
pub mod render;

use serde::Serialize;
use tera::{Context, Tera};

use crate::config::ThemeConfig;
use crate::template::render::{PageContext, TemplateRenderData};

pub fn build_template(config: &ThemeConfig) -> anyhow::Result<TemplateEngine> {
    let default = Tera::parse(
        &crate::util::path::root(&config.current().templates_dir)
            .join("**/*")
            .into_string(),
    )?;

    let mut custom = Tera::parse(
        &crate::util::path::root(&config.custom_template_dir)
            .join("**/*")
            .into_string(),
    )?;

    custom.extend(&default)?;
    custom.build_inheritance_chains()?;

    helper::register_helper(&mut custom, config)?;

    Ok(TemplateEngine { tera: custom })
}

pub struct TemplateEngine {
    tera: Tera,
}

impl TemplateEngine {
    pub fn render(&self, template_name: &str) -> anyhow::Result<String> {
        self.tera
            .render(template_name, &Context::default())
            .map_err(Into::into)
    }

    pub fn render_with<T>(&self, template_name: &str, context: &T) -> anyhow::Result<String>
    where
        T: Serialize,
    {
        self.tera
            .render(template_name, &Context::from_serialize(context)?)
            .map_err(Into::into)
    }

    pub fn typed_render<T>(&self, data: &PageContext<T>) -> anyhow::Result<String>
    where
        T: TemplateRenderData,
    {
        self.render_with(T::template_name(), data)
    }
}
