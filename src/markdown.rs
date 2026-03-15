use std::sync::LazyLock;

use comrak::Options;
use comrak::options::Plugins;
use comrak::plugins::syntect::{SyntectAdapter, SyntectAdapterBuilder};
use lol_html::html_content::{ContentType, Element};
use lol_html::{RewriteStrSettings, element, rewrite_str};
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;

use crate::config::{ThemeCodeSource, ThemeConfig};

static MARKDOWN_TO_HTML_CONFIG: LazyLock<MarkdownToHtmlConfig> =
    LazyLock::new(|| MarkdownToHtmlConfig::from_config(&crate::config::get().theme).unwrap());

pub fn version() -> &'static str {
    &crate::config::get().theme.render_version
}

pub fn render(markdown: &str) -> anyhow::Result<String> {
    let options = markdown_options();
    let plugins = markdown_plugins(&MARKDOWN_TO_HTML_CONFIG.syntect);

    let html = comrak::markdown_to_html_with_plugins(markdown, &options, &plugins);

    rewrite_html(&html)
}

struct MarkdownToHtmlConfig {
    syntect: SyntectAdapter,
}

impl MarkdownToHtmlConfig {
    fn from_config(config: &ThemeConfig) -> anyhow::Result<Self> {
        let mut builder = SyntectAdapterBuilder::new();

        builder = match config.code_syntax_source {
            ThemeCodeSource::Default => builder,
            ThemeCodeSource::Theme => builder.syntax_set(SyntaxSet::load_from_folder(
                &config.current().code_syntax_dir,
            )?),
            ThemeCodeSource::Custom => builder.syntax_set(SyntaxSet::load_from_folder(
                &config.custom().code_syntax_dir,
            )?),
        };
        builder = match config.code_themes_source {
            ThemeCodeSource::Default => builder,
            ThemeCodeSource::Theme => builder.theme_set(ThemeSet::load_from_folder(
                &config.current().code_themes_dir,
            )?),
            ThemeCodeSource::Custom => builder.theme_set(ThemeSet::load_from_folder(
                &config.custom().code_themes_dir,
            )?),
        };

        let syntect = builder.theme(&config.current_code_theme).build();

        Ok(Self { syntect })
    }
}

fn markdown_options() -> Options<'static> {
    let mut options = Options::default();

    options.extension.table = true;
    options.extension.strikethrough = true;
    options.extension.tasklist = true;
    options.extension.autolink = true;
    options.extension.spoiler = true;
    options.extension.underline = true;
    options.extension.footnotes = true;
    options.extension.math_code = true;
    options.extension.shortcodes = true;
    options.extension.header_ids = Some(String::new());

    options.render.r#unsafe = true;

    options
}

fn markdown_plugins(syntect: &SyntectAdapter) -> Plugins<'_> {
    let mut plugins = Plugins::default();

    plugins.render.codefence_syntax_highlighter = Some(syntect);

    plugins
}

fn rewrite_html(html: &str) -> anyhow::Result<String> {
    let handler = |el: &mut Element| {
        el.before("<div class=\"table-box\">", ContentType::Html);
        el.after("</div>", ContentType::Html);
        Ok(())
    };

    let html = rewrite_str(
        html,
        RewriteStrSettings {
            element_content_handlers: vec![element!("table", handler)],
            ..RewriteStrSettings::new()
        },
    )?;

    Ok(html)
}
