use anyhow::Result;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::config::WikiConfig;
use tomet_shelf_route::normalize_path_str;

#[derive(Debug, Clone)]
pub struct ScannedVault {
    pub doc_files: Vec<DocFileInfo>,
    pub media_files: Vec<MediaFileInfo>,
    pub vault_index: tomet::links::VaultLinkIndex,
    pub workspace_config_src: Option<String>,
    pub workspace_config_blocks: Vec<tomet::ast::Block>,
    pub control_files: Vec<DocFileInfo>,
}

#[derive(Debug, Clone)]
pub struct DocFileInfo {
    pub abs_path: PathBuf,
    pub rel_path: String,
}

#[derive(Debug, Clone)]
pub struct MediaFileInfo {
    pub abs_path: PathBuf,
    pub rel_path: String,
}

pub const MEDIA_EXTENSIONS: &[&str] = &[
    "png", "jpg", "jpeg", "gif", "svg", "webp", "pdf", "mp4", "mp3", "webm", "avif", "ico",
];

pub fn is_excluded_path(rel: &str, prefix: &str) -> bool {
    let prefix = prefix.trim_matches('/');
    if prefix.is_empty() {
        return false;
    }

    std::iter::once(0)
        .chain(rel.match_indices('/').map(|(i, _)| i + 1))
        .any(|start| {
            rel[start..]
                .strip_prefix(prefix)
                .is_some_and(|tail| tail.is_empty() || tail.starts_with('/'))
        })
}

pub fn exclude_prefixes(config: &WikiConfig) -> Vec<String> {
    let mut prefixes: Vec<String> = config
        .build
        .exclude
        .iter()
        .map(|s| normalize_path_str(s))
        .collect();

    let dest_str = normalize_path_str(&config.wiki.dest.to_string_lossy());
    if !dest_str.is_empty() && !prefixes.contains(&dest_str) {
        prefixes.push(dest_str);
    }

    prefixes
}

pub fn is_ignored_rel(rel_str: &str, exclude_prefixes: &[String]) -> bool {
    if rel_str.starts_with('.') || rel_str.contains("/.") {
        return true;
    }
    exclude_prefixes
        .iter()
        .any(|prefix| is_excluded_path(rel_str, prefix))
}

pub fn scan_vault(src_dir: &Path, config: &WikiConfig) -> Result<ScannedVault> {
    let mut doc_files = Vec::new();
    let mut media_files = Vec::new();
    let mut control_files = Vec::new();
    let mut all_rel_paths = Vec::new();

    let excludes = exclude_prefixes(config);

    for entry in WalkDir::new(src_dir)
        .into_iter()
        .filter_entry(|e| {
            if e.path() == src_dir {
                return true;
            }
            let Ok(rel) = e.path().strip_prefix(src_dir) else {
                return false;
            };
            let rel_str = normalize_path_str(&rel.to_string_lossy());
            !is_ignored_rel(&rel_str, &excludes)
        })
    {
        let entry = entry?;
        if !entry.file_type().is_file() {
            continue;
        }

        let abs_path = entry.path().to_path_buf();
        let Ok(rel) = abs_path.strip_prefix(src_dir) else {
            continue;
        };
        let rel_str = normalize_path_str(&rel.to_string_lossy());

        let ext = abs_path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_lowercase();

        if ext == "tmt" || ext == "tm" {
            all_rel_paths.push(rel_str.clone());
            let file_name = abs_path.file_name().and_then(|s| s.to_str()).unwrap_or("");
            if file_name.ends_with(".index.tmt") || file_name.ends_with(".config.tmt") {
                control_files.push(DocFileInfo {
                    abs_path,
                    rel_path: rel_str,
                });
            } else {
                doc_files.push(DocFileInfo {
                    abs_path,
                    rel_path: rel_str,
                });
            }
        } else if MEDIA_EXTENSIONS.contains(&ext.as_str()) {
            media_files.push(MediaFileInfo {
                abs_path,
                rel_path: rel_str,
            });
        }
    }

    doc_files.sort_by(|a, b| a.rel_path.cmp(&b.rel_path));
    media_files.sort_by(|a, b| a.rel_path.cmp(&b.rel_path));
    control_files.sort_by(|a, b| a.rel_path.cmp(&b.rel_path));

    let vault_index = tomet::links::VaultLinkIndex::from_paths(&all_rel_paths);

    let workspace_cfg_src = [".writ.tmt", "default.config.tmt"]
        .iter()
        .map(|name| src_dir.join(name))
        .find(|path| path.is_file())
        .and_then(|path| std::fs::read_to_string(path).ok());

    let workspace_config_blocks = workspace_cfg_src
        .as_deref()
        .map(tomet_shelf_document::extract_workspace_config_blocks)
        .unwrap_or_default();

    Ok(ScannedVault {
        doc_files,
        media_files,
        vault_index,
        workspace_config_src: workspace_cfg_src,
        workspace_config_blocks,
        control_files,
    })
}
