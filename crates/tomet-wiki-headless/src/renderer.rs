use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

use crate::assets::StaticAsset;
use crate::config::WikiConfig;
use tomet_shelf_document::{DocIcon, ProcessedDoc};
use tomet_shelf_icons::IconRenderer;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SectionSummary {
    pub name: String,
    pub count: usize,
}

/// A page that points at the one being rendered.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Backlink {
    pub url: String,
    pub title: String,
    pub section: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntrySummary {
    pub url: String,
    pub title: String,
    pub section: Option<String>,
    #[serde(default)]
    pub children: Vec<EntrySummary>,
}

/// Template-engine-agnostic page renderer for `tomet-wiki`.
pub trait PageRenderer: Send + Sync {
    /// Render a single document page into a complete HTML string.
    fn render_page(
        &self,
        doc: &ProcessedDoc,
        backlinks: &[Backlink],
        wiki_index: &[EntrySummary],
    ) -> Result<String>;

    /// Render the catalog/index page into a complete HTML string.
    fn render_index(
        &self,
        sections: &[SectionSummary],
        entries: &[EntrySummary],
    ) -> Result<String>;

    /// Render a 404 Not Found page into a complete HTML string.
    fn render_not_found(&self, missing_path: &str) -> Result<String> {
        Ok(format!(
            "<!doctype html><html><head><meta charset=\"utf-8\"><title>404 Not Found</title></head><body><h1>404 Not Found</h1><p>Cannot find: <code>{missing_path}</code></p></body></html>"
        ))
    }

    /// Static assets (CSS, JS, etc.) supplied by this theme.
    fn static_assets(&self) -> &[StaticAsset] {
        &[]
    }

    /// Optional notification of document note icons across the vault.
    fn set_doc_icons(&mut self, _icons: &HashMap<String, DocIcon>) {}

    /// Optional icon renderer supplied by this theme.
    fn icon_renderer(&self) -> Option<Arc<dyn IconRenderer>> {
        None
    }
}

/// Factory that creates a [`PageRenderer`] bound to a [`WikiConfig`].
pub trait RendererFactory: Send + Sync {
    fn create(&self, config: &WikiConfig, is_dev: bool) -> Result<Box<dyn PageRenderer>>;

    fn icon_renderer(&self) -> Option<Arc<dyn IconRenderer>> {
        None
    }
}

impl<F> RendererFactory for F
where
    F: Fn(&WikiConfig, bool) -> Result<Box<dyn PageRenderer>> + Send + Sync,
{
    fn create(&self, config: &WikiConfig, is_dev: bool) -> Result<Box<dyn PageRenderer>> {
        self(config, is_dev)
    }
}

const MINIMAL_STATIC_ASSETS: &[StaticAsset] = &[
    StaticAsset {
        path: "wiki.css",
        mime: "text/css; charset=utf-8",
        content: "/* Minimal Wiki CSS */\n.tm-conflict { border: 2px dashed #f59e0b; padding: 1rem; margin: 1rem 0; background: #fffbeb; }",
    },
    StaticAsset {
        path: "wiki.js",
        mime: "application/javascript; charset=utf-8",
        content: "/* Minimal Wiki JS */",
    },
];

/// Minimal template-free renderer for headless builds and tests.
#[derive(Debug, Clone)]
pub struct MinimalWikiRenderer {
    site_title: String,
    lang: String,
    base_path: String,
    clean_url_prefix: String,
    full_url_prefix: String,
    home_url: String,
    strings_json: String,
    cms_json: String,
}

impl MinimalWikiRenderer {
    pub fn new(config: &WikiConfig) -> Self {
        let base_path = config.build.clean_base_path().to_string();
        let clean_url_prefix = config.build.clean_url_prefix().to_string();
        let full_url_prefix = config.build.full_url_prefix();
        let home_url = if full_url_prefix.is_empty() {
            if base_path.is_empty() {
                "/".to_string()
            } else {
                format!("{base_path}/")
            }
        } else {
            full_url_prefix.clone()
        };
        let strings_json =
            serde_json::to_string(&config.ui.strings).unwrap_or_else(|_| "{}".to_string());
        let cms_json = config.cms_client_config_json();

        Self {
            site_title: config.wiki.title.clone(),
            lang: config.wiki.lang.clone(),
            base_path,
            clean_url_prefix,
            full_url_prefix,
            home_url,
            strings_json,
            cms_json,
        }
    }

    fn wrap_base(&self, page_title: Option<&str>, content_html: &str) -> String {
        let title_prefix = match page_title {
            Some(t) if !t.is_empty() => format!("{t} | "),
            _ => String::new(),
        };
        let base = &self.base_path;
        let site_title = &self.site_title;
        let lang = &self.lang;
        let home_url = &self.home_url;
        let base_json = serde_json::to_string(&self.base_path).unwrap_or_default();
        let clean_prefix_json = serde_json::to_string(&self.clean_url_prefix).unwrap_or_default();
        let full_prefix_json = serde_json::to_string(&self.full_url_prefix).unwrap_or_default();
        let home_url_json = serde_json::to_string(&self.home_url).unwrap_or_default();
        let strings_json = &self.strings_json;
        let cms_json = &self.cms_json;

        format!(
            r#"<!DOCTYPE html>
<html lang="{lang}" data-view="wiki">
<head>
  <meta charset="utf-8" />
  <title>{title_prefix}{site_title}</title>
  <link rel="stylesheet" href="{base}/wiki.css" />
</head>
<body>
  <a href="{home_url}">{site_title}</a>
  {content_html}
  <script>
    window.tmtBasePath = {base_json};
    window.tmtUrlPrefix = {clean_prefix_json};
    window.tmtFullUrlPrefix = {full_prefix_json};
    window.tmtHomeUrl = {home_url_json};
    window.tmtStrings = {strings_json};
    window.tmtWikiCms = {cms_json};
  </script>
  <script src="{base}/wiki.js"></script>
</body>
</html>"#
        )
    }
}

impl PageRenderer for MinimalWikiRenderer {
    fn render_page(
        &self,
        doc: &ProcessedDoc,
        _backlinks: &[Backlink],
        _wiki_index: &[EntrySummary],
    ) -> Result<String> {
        let slug_span = if !doc.slug.is_empty() {
            format!(
                r#"<span class="sr-only" data-pagefind-meta="path" data-pagefind-weight="10.0">{}</span>"#,
                doc.slug
            )
        } else {
            String::new()
        };
        let rel_span = if !doc.rel_path.is_empty() && doc.rel_path != doc.slug {
            format!(
                r#"<span class="sr-only" data-pagefind-weight="10.0">{}</span>"#,
                doc.rel_path
            )
        } else {
            String::new()
        };
        let icon_span = match &doc.icon {
            Some(DocIcon::Text(t)) => format!(r#"<span class="page-icon">{t}</span>"#),
            Some(DocIcon::Named { name, .. }) => {
                format!(r#"<span class="page-icon">{name}</span>"#)
            }
            Some(DocIcon::Image { src }) => {
                format!(r#"<img class="page-icon" src="{src}" />"#)
            }
            None => String::new(),
        };

        // Note: data-tmt-rel-path on <main> is essential for the inline Svelte editor
        let content = format!(
            r#"<main data-pagefind-body data-tmt-rel-path="{}">
  {slug_span}
  {rel_span}
  {icon_span}
  <h1>{}</h1>
  <article>{}</article>
</main>"#,
            doc.rel_path, doc.title, doc.body_html
        );
        Ok(self.wrap_base(Some(&doc.title), &content))
    }

    fn render_index(
        &self,
        _sections: &[SectionSummary],
        entries: &[EntrySummary],
    ) -> Result<String> {
        let mut items = String::new();
        for entry in entries {
            items.push_str(&format!(
                r#"<li><a href="{}">{}</a></li>"#,
                entry.url, entry.title
            ));
        }
        let content = format!("<main><ul>{items}</ul></main>");
        Ok(self.wrap_base(None, &content))
    }

    fn static_assets(&self) -> &[StaticAsset] {
        MINIMAL_STATIC_ASSETS
    }
}

/// Unit factory that creates a [`MinimalWikiRenderer`].
#[derive(Debug, Clone, Copy, Default)]
pub struct MinimalWikiRendererFactory;

impl RendererFactory for MinimalWikiRendererFactory {
    fn create(&self, config: &WikiConfig, _is_dev: bool) -> Result<Box<dyn PageRenderer>> {
        Ok(Box::new(MinimalWikiRenderer::new(config)))
    }
}
