use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tomet_shelf_config::{
    BookConfig, BookMeta, BuildConfig, CollisionStrategy, LinksConfig, MarkersConfig,
    RoutingStrategy, SearchEngine, UiConfig,
};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WikiConfig {
    #[serde(default)]
    pub wiki: WikiMeta,
    #[serde(default)]
    pub cms: CmsConfig,
    #[serde(default)]
    pub build: WikiBuildConfig,
    #[serde(default)]
    pub ui: WikiUiConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WikiMeta {
    #[serde(default = "default_title")]
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default = "default_lang")]
    pub lang: String,
    /// Destination directory relative to vault root
    #[serde(default = "default_dest")]
    pub dest: PathBuf,
}

fn default_title() -> String {
    "Tomet Wiki".to_string()
}

fn default_lang() -> String {
    "ja".to_string()
}

fn default_dest() -> PathBuf {
    PathBuf::from("dist")
}

impl Default for WikiMeta {
    fn default() -> Self {
        Self {
            title: default_title(),
            description: None,
            lang: default_lang(),
            dest: default_dest(),
        }
    }
}

impl WikiMeta {
    pub fn is_english(&self) -> bool {
        tomet_shelf_config::is_english(&self.lang)
    }
}

pub const DEFAULT_AUTH_ENDPOINT: &str = "https://auth.tomet.dev/github";

fn default_auth_endpoint() -> Option<String> {
    Some(CmsConfig::default_auth_endpoint())
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CmsConfig {
    #[serde(default)]
    pub owner: String,
    #[serde(default)]
    pub repo: String,
    #[serde(default = "default_branch")]
    pub branch: String,
    #[serde(default = "default_auth_endpoint")]
    pub auth_endpoint: Option<String>,
    #[serde(default)]
    pub site_domain: Option<String>,
    #[serde(default)]
    pub api_base_url: Option<String>,
}

fn default_branch() -> String {
    "main".to_string()
}

impl CmsConfig {
    pub fn default_auth_endpoint() -> String {
        DEFAULT_AUTH_ENDPOINT.to_string()
    }
}

impl Default for CmsConfig {
    fn default() -> Self {
        Self {
            owner: String::new(),
            repo: String::new(),
            branch: default_branch(),
            auth_endpoint: default_auth_endpoint(),
            site_domain: None,
            api_base_url: None,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct CmsClientConfig<'a> {
    owner: &'a str,
    repo: &'a str,
    branch: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    auth_endpoint: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    site_domain: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    api_base_url: Option<&'a str>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WikiBuildConfig {
    #[serde(default = "default_exclude")]
    pub exclude: Vec<String>,
    #[serde(default = "default_kinds")]
    pub kinds: Vec<String>,
    #[serde(default)]
    pub search: SearchEngine,
    #[serde(default, alias = "base")]
    pub base_path: Option<String>,
    #[serde(default = "default_url_prefix")]
    pub url_prefix: String,
    #[serde(default = "default_asset_prefix")]
    pub asset_prefix: String,
    #[serde(default)]
    pub routing: RoutingStrategy,
    #[serde(default)]
    pub on_collision: CollisionStrategy,
    #[serde(default)]
    pub config_path: Option<PathBuf>,
}

fn default_exclude() -> Vec<String> {
    vec![
        "00-09 System/01 Apps".to_string(),
        ".git".to_string(),
        ".tomet".to_string(),
        ".web".to_string(),
        "node_modules".to_string(),
        "dist".to_string(),
        "target".to_string(),
    ]
}

fn default_kinds() -> Vec<String> {
    vec![
        "*".to_string(),
        "!config".to_string(),
        "!index".to_string(),
        "!blueprint".to_string(),
        "!vocabulary".to_string(),
    ]
}

fn default_url_prefix() -> String {
    "/wiki".to_string()
}

fn default_asset_prefix() -> String {
    "/vault".to_string()
}

impl Default for WikiBuildConfig {
    fn default() -> Self {
        Self {
            exclude: default_exclude(),
            kinds: default_kinds(),
            search: SearchEngine::default(),
            base_path: None,
            url_prefix: default_url_prefix(),
            asset_prefix: default_asset_prefix(),
            routing: RoutingStrategy::default(),
            on_collision: CollisionStrategy::default(),
            config_path: None,
        }
    }
}

impl WikiBuildConfig {
    pub fn clean_base_path(&self) -> &str {
        match self.base_path.as_deref() {
            Some(base) => {
                let trimmed = base.trim_matches('/');
                if trimmed.is_empty() {
                    ""
                } else {
                    let s = base.trim_end_matches('/');
                    if s.starts_with('/') { s } else { base }
                }
            }
            None => "",
        }
    }

    pub fn clean_url_prefix(&self) -> &str {
        self.url_prefix.trim_end_matches('/')
    }

    pub fn clean_asset_prefix(&self) -> &str {
        self.asset_prefix.trim_end_matches('/')
    }

    pub fn full_url_prefix(&self) -> String {
        let base = self.clean_base_path();
        let prefix = self.clean_url_prefix();
        if base.is_empty() {
            prefix.to_string()
        } else if prefix.is_empty() {
            base.to_string()
        } else {
            format!("{base}{prefix}")
        }
    }

    pub fn full_asset_prefix(&self) -> String {
        let base = self.clean_base_path();
        let prefix = self.clean_asset_prefix();
        if base.is_empty() {
            prefix.to_string()
        } else if prefix.is_empty() {
            base.to_string()
        } else {
            format!("{base}{prefix}")
        }
    }

    pub fn wiki_out_rel(&self) -> &str {
        self.url_prefix.trim_matches('/')
    }

    pub fn asset_out_rel(&self) -> &str {
        self.asset_prefix.trim_matches('/')
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WikiUiConfig {
    #[serde(default)]
    pub strings: HashMap<String, String>,
    #[serde(default)]
    pub markers: MarkersConfig,
    #[serde(default)]
    pub links: LinksConfig,
}

impl WikiConfig {
    pub fn load_from_str(content: &str) -> Result<Self> {
        toml::from_str(content).context("Failed to parse TOML configuration")
    }

    pub fn load_from_dir(dir: &Path) -> Result<Self> {
        let config_path = dir.join("wiki.toml");
        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)
                .with_context(|| format!("Failed to read {}", config_path.display()))?;
            let mut config = Self::load_from_str(&content)
                .with_context(|| format!("Failed to parse {}", config_path.display()))?;
            config.build.config_path = Some(config_path);
            Ok(config)
        } else {
            Ok(WikiConfig::default())
        }
    }

    /// Converts this [`WikiConfig`] into a [`BookConfig`] for internal consumption by
    /// `tomet-shelf` crates (`tomet-shelf-document`, `tomet-shelf-render`, etc.).
    pub fn to_doc_book_config(&self) -> BookConfig {
        BookConfig {
            book: BookMeta {
                title: self.wiki.title.clone(),
                description: self.wiki.description.clone(),
                lang: self.wiki.lang.clone(),
                dest: self.wiki.dest.clone(),
            },
            ui: UiConfig {
                rail_title: Some(self.wiki.title.clone()),
                default_view: "wiki".to_string(),
                custom_css: Vec::new(),
                hero_chips: Vec::new(),
                infobox: Default::default(),
                strings: self.ui.strings.clone(),
                markers: self.ui.markers.clone(),
                links: self.ui.links.clone(),
            },
            build: BuildConfig {
                exclude: self.build.exclude.clone(),
                kinds: self.build.kinds.clone(),
                search: self.build.search,
                base_path: self.build.base_path.clone(),
                url_prefix: self.build.url_prefix.clone(),
                asset_prefix: self.build.asset_prefix.clone(),
                routing: self.build.routing,
                on_collision: self.build.on_collision,
                config_path: self.build.config_path.clone(),
            },
        }
    }

    /// Serializes the public CMS settings for injection into HTML pages as `window.tmtWikiCms`.
    pub fn cms_client_config_json(&self) -> String {
        let auth_endpoint = self
            .cms
            .auth_endpoint
            .as_deref()
            .or(Some(DEFAULT_AUTH_ENDPOINT));
        let client = CmsClientConfig {
            owner: &self.cms.owner,
            repo: &self.cms.repo,
            branch: &self.cms.branch,
            auth_endpoint,
            site_domain: self.cms.site_domain.as_deref(),
            api_base_url: self.cms.api_base_url.as_deref(),
        };
        serde_json::to_string(&client).unwrap_or_else(|_| "{}".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_has_expected_values() {
        let config = WikiConfig::default();
        assert_eq!(config.wiki.title, "Tomet Wiki");
        assert_eq!(config.wiki.lang, "ja");
        assert_eq!(config.wiki.dest, PathBuf::from("dist"));
        assert_eq!(config.cms.branch, "main");
        assert_eq!(config.cms.owner, "");
        assert_eq!(config.cms.repo, "");
        assert_eq!(
            config.cms.auth_endpoint.as_deref(),
            Some(DEFAULT_AUTH_ENDPOINT)
        );
        assert_eq!(config.build.url_prefix, "/wiki");
        assert_eq!(config.build.asset_prefix, "/vault");
        assert!(!config.wiki.is_english());
    }

    #[test]
    fn parses_full_wiki_toml() {
        let toml_str = r#"
[wiki]
title = "Ecosystem Knowledge Base"
description = "Central documentation and notes"
lang = "en"
dest = "public"

[cms]
owner = "tomet-lang"
repo = "wiki"
branch = "master"
auth_endpoint = "https://auth.tomet.dev/oauth/callback"
site_domain = "wiki.tomet.dev"
api_base_url = "https://api.github.com"

[build]
base_path = "/docs"
url_prefix = "/articles"
exclude = ["drafts"]

[ui.strings]
search = "Search articles"
"#;
        let config = WikiConfig::load_from_str(toml_str).expect("parse failed");
        assert_eq!(config.wiki.title, "Ecosystem Knowledge Base");
        assert_eq!(
            config.wiki.description.as_deref(),
            Some("Central documentation and notes")
        );
        assert_eq!(config.wiki.lang, "en");
        assert!(config.wiki.is_english());
        assert_eq!(config.wiki.dest, PathBuf::from("public"));

        assert_eq!(config.cms.owner, "tomet-lang");
        assert_eq!(config.cms.repo, "wiki");
        assert_eq!(config.cms.branch, "master");
        assert_eq!(
            config.cms.auth_endpoint.as_deref(),
            Some("https://auth.tomet.dev/oauth/callback")
        );
        assert_eq!(
            config.cms.site_domain.as_deref(),
            Some("wiki.tomet.dev")
        );
        assert_eq!(
            config.cms.api_base_url.as_deref(),
            Some("https://api.github.com")
        );

        assert_eq!(config.build.clean_base_path(), "/docs");
        assert_eq!(config.build.clean_url_prefix(), "/articles");
        assert_eq!(config.build.full_url_prefix(), "/docs/articles");
        assert_eq!(config.build.exclude, vec!["drafts"]);

        assert_eq!(
            config.ui.strings.get("search").map(|s| s.as_str()),
            Some("Search articles")
        );
    }

    #[test]
    fn cms_client_config_json_format() {
        let mut config = WikiConfig::default();
        config.cms.owner = "my-org".to_string();
        config.cms.repo = "my-wiki".to_string();
        config.cms.branch = "main".to_string();
        config.cms.auth_endpoint = Some("https://auth.example.com".to_string());

        let json = config.cms_client_config_json();
        assert!(json.contains(r#""owner":"my-org""#));
        assert!(json.contains(r#""repo":"my-wiki""#));
        assert!(json.contains(r#""branch":"main""#));
        assert!(json.contains(r#""authEndpoint":"https://auth.example.com""#));
        // None fields are omitted
        assert!(!json.contains("siteDomain"));
        assert!(!json.contains("apiBaseUrl"));

        // Default config automatically has default authEndpoint
        let default_json = WikiConfig::default().cms_client_config_json();
        assert!(default_json.contains(r#""authEndpoint":"https://auth.tomet.dev/github""#));
    }

    #[test]
    fn to_doc_book_config_conversion() {
        let mut config = WikiConfig::default();
        config.wiki.title = "Test Wiki".to_string();
        config.wiki.lang = "en".to_string();
        config.build.base_path = Some("/base".to_string());

        let book_cfg = config.to_doc_book_config();
        assert_eq!(book_cfg.book.title, "Test Wiki");
        assert_eq!(book_cfg.book.lang, "en");
        assert_eq!(book_cfg.build.base_path.as_deref(), Some("/base"));
    }

    #[test]
    fn load_from_dir_handles_present_and_absent() {
        let temp = std::env::temp_dir().join(format!("wiki-config-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&temp);
        std::fs::create_dir_all(&temp).unwrap();

        // Absent wiki.toml returns default
        let empty_cfg = WikiConfig::load_from_dir(&temp).unwrap();
        assert_eq!(empty_cfg.wiki.title, "Tomet Wiki");

        // Present wiki.toml
        std::fs::write(temp.join("wiki.toml"), "[wiki]\ntitle = \"Custom Wiki\"\n").unwrap();
        let loaded_cfg = WikiConfig::load_from_dir(&temp).unwrap();
        assert_eq!(loaded_cfg.wiki.title, "Custom Wiki");

        let _ = std::fs::remove_dir_all(&temp);
    }
}
