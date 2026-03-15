mod format;
mod markdown;

use tera::Tera;

use crate::config::ThemeConfig;

pub fn register_helper(tera: &mut Tera, _config: &ThemeConfig) -> anyhow::Result<()> {
    tera.register_filter("markdown_to_html", markdown::render);
    tera.register_filter("human_number", format::human_number);
    tera.register_filter("human_size", format::human_size);
    Ok(())
}
