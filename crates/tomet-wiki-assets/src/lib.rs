//! Theme and asset bundle for `tomet-wiki`.

use anyhow::{Context, Result};
use askama::Template;
use std::sync::Arc;
use tomet_shelf_document::{DocIcon, ProcessedDoc};
use tomet_shelf_icons::{IconRenderer, LUCIDE_SPRITE, SIMPLE_SPRITE, StandardIconRenderer};
use tomet_wiki_headless::{
    Backlink, EntrySummary, PageRenderer, RendererFactory, SectionSummary, StaticAsset, WikiConfig,
};

pub const WIKI_CSS: &str = include_str!("../assets/wiki.css");
pub const WIKI_JS: &str = include_str!("../assets/wiki.js");
pub const WIKI_EDITOR_JS: &str = include_str!("../assets/wiki-editor.js");
pub const WIKI_EDITOR_CSS: &str = include_str!("../assets/wiki-editor.css");

pub const ICON_ASSETS: &[StaticAsset] = &[
    StaticAsset {
        path: "icons/lucide.svg",
        mime: "image/svg+xml",
        content: LUCIDE_SPRITE,
    },
    StaticAsset {
        path: "icons/simple.svg",
        mime: "image/svg+xml",
        content: SIMPLE_SPRITE,
    },
];

pub const STATIC_ASSETS: &[StaticAsset] = &[
    StaticAsset {
        path: "wiki.css",
        mime: "text/css; charset=utf-8",
        content: WIKI_CSS,
    },
    StaticAsset {
        path: "wiki.js",
        mime: "application/javascript; charset=utf-8",
        content: WIKI_JS,
    },
    StaticAsset {
        path: "wiki-editor.js",
        mime: "application/javascript; charset=utf-8",
        content: WIKI_EDITOR_JS,
    },
    StaticAsset {
        path: "wiki-editor.css",
        mime: "text/css; charset=utf-8",
        content: WIKI_EDITOR_CSS,
    },
    StaticAsset {
        path: "icons/lucide.svg",
        mime: "image/svg+xml",
        content: LUCIDE_SPRITE,
    },
    StaticAsset {
        path: "icons/simple.svg",
        mime: "image/svg+xml",
        content: SIMPLE_SPRITE,
    },
];

#[derive(Template)]
#[template(path = "page.html")]
pub struct PageTemplate<'a> {
    pub site_title: &'a str,
    pub lang: &'a str,
    pub base_path: &'a str,
    pub clean_url_prefix: &'a str,
    pub full_url_prefix: &'a str,
    pub home_url: &'a str,
    pub strings_json: &'a str,
    pub cms_json: &'a str,
    pub doc: &'a ProcessedDoc,
    pub backlinks: &'a [Backlink],
    pub wiki_index: &'a [EntrySummary],
    pub icon_rendered: Option<String>,
}

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate<'a> {
    pub site_title: &'a str,
    pub lang: &'a str,
    pub base_path: &'a str,
    pub clean_url_prefix: &'a str,
    pub full_url_prefix: &'a str,
    pub home_url: &'a str,
    pub strings_json: &'a str,
    pub cms_json: &'a str,
    pub sections: &'a [SectionSummary],
    pub entries: &'a [EntrySummary],
}

#[derive(Template)]
#[template(path = "404.html")]
pub struct NotFoundTemplate<'a> {
    pub site_title: &'a str,
    pub lang: &'a str,
    pub base_path: &'a str,
    pub clean_url_prefix: &'a str,
    pub full_url_prefix: &'a str,
    pub home_url: &'a str,
    pub strings_json: &'a str,
    pub cms_json: &'a str,
    pub missing_path: &'a str,
}

pub struct WikiAssetsRenderer {
    site_title: String,
    lang: String,
    base_path: String,
    clean_url_prefix: String,
    full_url_prefix: String,
    home_url: String,
    strings_json: String,
    cms_json: String,
    icon_renderer: Option<Arc<dyn IconRenderer>>,
}

impl WikiAssetsRenderer {
    pub fn new(config: &WikiConfig, icon_renderer: Option<Arc<dyn IconRenderer>>) -> Self {
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
            icon_renderer,
        }
    }
}

impl PageRenderer for WikiAssetsRenderer {
    fn render_page(
        &self,
        doc: &ProcessedDoc,
        backlinks: &[Backlink],
        wiki_index: &[EntrySummary],
    ) -> Result<String> {
        let icon_rendered = doc.icon.as_ref().and_then(|icon| match icon {
            DocIcon::Text(t) => Some(format!(r#"<span class="tmt-icon-text">{t}</span>"#)),
            DocIcon::Named { name, pkg, .. } => {
                let r = self.icon_renderer.as_ref()?;
                r.render(&tomet_shelf_icons::IconRequest {
                    name,
                    pkg,
                    base_path: &self.base_path,
                    inline: false,
                    class: Some("tm-doc-icon"),
                    args: None,
                })
            }
            DocIcon::Image { src } => {
                Some(format!(r#"<img class="tm-doc-icon-img" src="{src}" alt="" />"#))
            }
        });

        let template = PageTemplate {
            site_title: &self.site_title,
            lang: &self.lang,
            base_path: &self.base_path,
            clean_url_prefix: &self.clean_url_prefix,
            full_url_prefix: &self.full_url_prefix,
            home_url: &self.home_url,
            strings_json: &self.strings_json,
            cms_json: &self.cms_json,
            doc,
            backlinks,
            wiki_index,
            icon_rendered,
        };

        template.render().context("Failed to render page template")
    }

    fn render_index(
        &self,
        sections: &[SectionSummary],
        entries: &[EntrySummary],
    ) -> Result<String> {
        let template = IndexTemplate {
            site_title: &self.site_title,
            lang: &self.lang,
            base_path: &self.base_path,
            clean_url_prefix: &self.clean_url_prefix,
            full_url_prefix: &self.full_url_prefix,
            home_url: &self.home_url,
            strings_json: &self.strings_json,
            cms_json: &self.cms_json,
            sections,
            entries,
        };

        template.render().context("Failed to render index template")
    }

    fn render_not_found(&self, missing_path: &str) -> Result<String> {
        let template = NotFoundTemplate {
            site_title: &self.site_title,
            lang: &self.lang,
            base_path: &self.base_path,
            clean_url_prefix: &self.clean_url_prefix,
            full_url_prefix: &self.full_url_prefix,
            home_url: &self.home_url,
            strings_json: &self.strings_json,
            cms_json: &self.cms_json,
            missing_path,
        };

        template
            .render()
            .context("Failed to render not_found template")
    }

    fn static_assets(&self) -> &[StaticAsset] {
        STATIC_ASSETS
    }

    fn icon_renderer(&self) -> Option<Arc<dyn IconRenderer>> {
        self.icon_renderer.clone()
    }
}

/// Official `tomet-wiki` assets and theme renderer factory.
#[derive(Debug, Clone, Copy, Default)]
pub struct WikiAssets;

pub const ASSETS: WikiAssets = WikiAssets;

impl RendererFactory for WikiAssets {
    fn create(&self, config: &WikiConfig, _is_dev: bool) -> Result<Box<dyn PageRenderer>> {
        Ok(Box::new(WikiAssetsRenderer::new(
            config,
            self.icon_renderer(),
        )))
    }

    fn icon_renderer(&self) -> Option<Arc<dyn IconRenderer>> {
        Some(Arc::new(StandardIconRenderer))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn assets_creates_renderer_and_has_icon_renderer() {
        let config = WikiConfig::default();
        let renderer = ASSETS.create(&config, false).unwrap();
        assert!(!renderer.static_assets().is_empty());
        assert!(ASSETS.icon_renderer().is_some());
    }
}
