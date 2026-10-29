//! Headless engine for `tomet-wiki`.

pub mod assets;
pub mod config;
pub mod renderer;
pub mod scan;
pub mod serve;
pub mod wiki;

// Re-exports
pub use assets::{StaticAsset, content_version, write_if_changed, write_static_assets};
pub use config::{CmsConfig, WikiBuildConfig, WikiConfig, WikiMeta, WikiUiConfig};
pub use renderer::{
    Backlink, EntrySummary, MinimalWikiRenderer, MinimalWikiRendererFactory, PageRenderer,
    RendererFactory, SectionSummary,
};
pub use scan::{DocFileInfo, MediaFileInfo, ScannedVault, scan_vault};
pub use serve::{DevVaultState, WikiDevHandler, run_dev_server};
pub use wiki::{BuildReport, DocFailure, build_wiki};
