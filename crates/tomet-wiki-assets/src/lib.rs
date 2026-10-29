//! Theme and asset bundle for `tomet-wiki`.

use anyhow::Result;
use std::sync::Arc;
use tomet_shelf_icons::{IconRenderer, LUCIDE_SPRITE, SIMPLE_SPRITE, StandardIconRenderer};
use tomet_wiki_headless::{
    MinimalWikiRenderer, PageRenderer, RendererFactory, StaticAsset, WikiConfig,
};

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

/// Official `tomet-wiki` assets and theme renderer factory.
#[derive(Debug, Clone, Copy, Default)]
pub struct WikiAssets;

pub const ASSETS: WikiAssets = WikiAssets;

impl RendererFactory for WikiAssets {
    fn create(&self, config: &WikiConfig, _is_dev: bool) -> Result<Box<dyn PageRenderer>> {
        // Initial implementation wrapping MinimalWikiRenderer.
        // Full Askama templates and Svelte inline editor bundle will follow in the next tasks.
        Ok(Box::new(MinimalWikiRenderer::new(config)))
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
