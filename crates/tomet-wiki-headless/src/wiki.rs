use anyhow::{Context, Result};
use rayon::prelude::*;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;
use tracing::{info, warn};

use crate::assets::{write_if_changed, write_static_assets};
use crate::config::WikiConfig;
use crate::renderer::{Backlink, EntrySummary, RendererFactory, SectionSummary};
use crate::scan::{MediaFileInfo, ScannedVault, scan_vault};
use tomet_shelf_config::SearchEngine;
use tomet_shelf_document::ProcessedDoc;
use tomet_shelf_route::{DocRouteInput, RouteTable, extract_route_metadata};

#[derive(Debug, Clone)]
pub struct DocFailure {
    pub rel_path: String,
    pub error: String,
}

#[derive(Debug)]
pub struct BuildReport {
    pub rendered: usize,
    pub written: usize,
    pub failures: Vec<DocFailure>,
    pub backlinks: HashMap<String, Vec<Backlink>>,
    pub route_table: RouteTable,
}

/// Decides whether a document with the given `@kind` is published according to rule list.
pub fn kind_publishes(kind: Option<&str>, rules: &[String]) -> bool {
    if rules.is_empty() {
        return true;
    }
    let mut published = false;
    for rule in rules {
        let trimmed = rule.trim();
        if let Some(neg) = trimmed.strip_prefix('!') {
            if neg == "*" || kind == Some(neg) {
                published = false;
            }
        } else if trimmed == "*" || kind == Some(trimmed) {
            published = true;
        }
    }
    published
}

pub fn build_backlink_index(
    docs: &[&ProcessedDoc],
    clean_url_prefix: &str,
) -> HashMap<String, Vec<Backlink>> {
    let mut backlinks: HashMap<String, Vec<Backlink>> = HashMap::new();
    for source in docs {
        for target_slug in &source.outgoing {
            backlinks.entry(target_slug.clone()).or_default().push(Backlink {
                url: format!("{clean_url_prefix}/{}", source.slug),
                title: source.title.clone(),
                section: source.section.clone(),
            });
        }
    }
    for list in backlinks.values_mut() {
        list.sort_by(|a, b| a.title.cmp(&b.title).then_with(|| a.url.cmp(&b.url)));
    }
    backlinks
}

pub fn sync_media_file(src: &Path, dest: &Path) -> Result<()> {
    if dest.exists() {
        let _ = fs::remove_file(dest);
    }
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)?;
    }
    if fs::hard_link(src, dest).is_err() {
        fs::copy(src, dest)?;
    }
    Ok(())
}

pub fn copy_vault_media(
    out_dir: &Path,
    media_files: &[MediaFileInfo],
    config: &WikiConfig,
) -> Result<()> {
    let target = out_dir.join(config.build.asset_out_rel());
    fs::create_dir_all(&target)?;
    for media in media_files {
        let dest = target.join(&media.rel_path);
        let _ = sync_media_file(&media.abs_path, &dest);
    }
    Ok(())
}

pub fn run_pagefind(out_dir: &Path) -> Result<()> {
    use std::process::Command;

    info!("Running Pagefind search indexer on {}", out_dir.display());
    let status = Command::new("pagefind").arg("--site").arg(out_dir).status();
    match status {
        Ok(s) => {
            if s.success() {
                info!("Pagefind search index generated successfully");
            } else {
                warn!("Pagefind exited with status: {:?}", s.code());
            }
        }
        Err(e) => {
            warn!("Pagefind executable not found or failed to execute: {e}. Search indexing skipped.");
        }
    }
    Ok(())
}

pub fn build_native_search_index(
    docs: &[&ProcessedDoc],
    clean_url_prefix: &str,
) -> tomet_shelf_search::SearchIndex {
    let mut search_docs = Vec::with_capacity(docs.len());
    for (idx, doc) in docs.iter().enumerate() {
        let headings: Vec<String> = doc.toc.iter().map(|item| item.text.clone()).collect();
        let text = tomet_shelf_search::strip_html_to_plain_text(&doc.body_html);
        let image = doc.icon_image_url.clone().or_else(|| doc.banner_url.clone());

        search_docs.push(tomet_shelf_search::SearchDocument {
            id: idx as u32,
            slug: doc.slug.clone(),
            title: doc.title.clone(),
            url: format!("{clean_url_prefix}/{}", doc.slug),
            path: doc.rel_path.clone(),
            aliases: Vec::new(),
            kind: doc.kind.clone(),
            section: doc.section.clone(),
            image,
            headings,
            text,
        });
    }
    tomet_shelf_search::SearchIndex::new(search_docs)
}

/// Builds an entire Wiki site from a source directory.
pub fn build_wiki(
    src_dir: &Path,
    out_dir: &Path,
    config: &WikiConfig,
    renderer_factory: &dyn RendererFactory,
    skip_search: bool,
) -> Result<BuildReport> {
    info!(
        "Building wiki from {} to {} (production: {})",
        src_dir.display(),
        out_dir.display(),
        config.production
    );

    if config.production
        && (config.cms.owner.trim().is_empty() || config.cms.repo.trim().is_empty())
    {
        anyhow::bail!(
            "Production mode (--production) requires non-empty [cms] 'owner' and 'repo' in wiki.toml"
        );
    }

    // 1. Scan vault
    let scanned: ScannedVault = scan_vault(src_dir, config).context("Failed to scan vault")?;
    let book_config = config.to_doc_book_config();

    // 2. Initialize renderer and write static assets
    let mut renderer = renderer_factory.create(config, false)?;
    let icon_renderer = renderer
        .icon_renderer()
        .or_else(|| renderer_factory.icon_renderer());
    write_static_assets(out_dir, src_dir, config, renderer.static_assets())?;

    // 3. Copy vault media
    copy_vault_media(out_dir, &scanned.media_files, config)?;

    let wiki_rel = config.build.wiki_out_rel();
    let wiki_root = out_dir.join(wiki_rel);
    let full_url_prefix = config.build.full_url_prefix();
    let clean_url_prefix = full_url_prefix.as_str();

    // 4. Parse documents
    let parsed: Vec<Result<(tomet::ast::Document, Option<String>), DocFailure>> = scanned
        .doc_files
        .par_iter()
        .map(|doc_file| {
            let fail = |error: String| DocFailure {
                rel_path: doc_file.rel_path.clone(),
                error,
            };
            let source = match fs::read_to_string(&doc_file.abs_path) {
                Ok(s) => s,
                Err(e) => return Err(fail(format!("could not read file: {e}"))),
            };
            tomet_shelf_document::parse_tomet_document(&source, &doc_file.rel_path)
                .map_err(|e| fail(format!("could not parse document: {e}")))
        })
        .collect();

    let unpublished: HashSet<String> = parsed
        .iter()
        .zip(scanned.doc_files.iter())
        .filter(|(result, _)| match result {
            Ok((_, kind)) => !kind_publishes(kind.as_deref(), &config.build.kinds),
            Err(_) => false,
        })
        .map(|(_, doc_file)| doc_file.rel_path.clone())
        .collect();

    // 5. Build route table
    let mut route_meta = Vec::new();
    for (res, doc_file) in parsed.iter().zip(scanned.doc_files.iter()) {
        if unpublished.contains(&doc_file.rel_path) {
            continue;
        }
        if let Ok((doc, _)) = res {
            let (slug, id) = extract_route_metadata(doc);
            route_meta.push((doc_file.rel_path.as_str(), slug, id));
        }
    }
    let doc_route_inputs: Vec<DocRouteInput<'_>> = route_meta
        .iter()
        .map(|(path, slug, id)| DocRouteInput {
            rel_path: path,
            explicit_slug: slug.as_deref(),
            meta_id: id.as_deref(),
        })
        .collect();
    let route_table = RouteTable::build(&doc_route_inputs, &book_config.build)?;

    // 6. Process documents
    let custom_element = Some(tomet_shelf_render::create_doc_icon_custom_element(
        config.build.clean_base_path(),
        icon_renderer,
    ));

    let processed: Vec<Result<ProcessedDoc, DocFailure>> = parsed
        .into_par_iter()
        .zip(scanned.doc_files.par_iter())
        .filter(|(_, doc_file)| !unpublished.contains(&doc_file.rel_path))
        .map(|(result, doc_file)| {
            let (doc, _) = result?;
            tomet_shelf_document::process_parsed_document_with_custom_element(
                doc,
                &doc_file.rel_path,
                Some(&doc_file.abs_path),
                &book_config,
                &scanned.vault_index,
                &scanned.workspace_config_blocks,
                &unpublished,
                Some(&route_table),
                custom_element.clone(),
            )
            .map_err(|e| DocFailure {
                rel_path: doc_file.rel_path.clone(),
                error: format!("could not process document: {e}"),
            })
        })
        .collect();

    // Note icons
    if config.ui.links.note_icons {
        let doc_icons: HashMap<String, tomet_shelf_document::DocIcon> = processed
            .iter()
            .filter_map(|r| r.as_ref().ok())
            .filter_map(|doc| doc.icon.as_ref().map(|i| (doc.slug.clone(), i.clone())))
            .collect();
        renderer.set_doc_icons(&doc_icons);
    }

    // 7. Backlinks
    let valid_docs: Vec<&ProcessedDoc> = processed.iter().filter_map(|r| r.as_ref().ok()).collect();
    let backlinks = build_backlink_index(&valid_docs, clean_url_prefix);

    let entries_by_slug: HashMap<String, EntrySummary> = valid_docs
        .iter()
        .map(|doc| {
            (
                doc.slug.clone(),
                EntrySummary {
                    url: format!("{clean_url_prefix}/{}", doc.slug),
                    title: doc.title.clone(),
                    section: doc.section.clone(),
                    children: Vec::new(),
                },
            )
        })
        .collect();

    let wiki_index: Vec<EntrySummary> = valid_docs
        .iter()
        .filter_map(|d| entries_by_slug.get(&d.slug).cloned())
        .collect();

    // 8. Render pages
    let mut written = 0;
    let mut failures = Vec::new();
    let mut section_counts: HashMap<String, usize> = HashMap::new();

    for result in &processed {
        let doc = match result {
            Ok(d) => d,
            Err(f) => {
                failures.push(f.clone());
                continue;
            }
        };

        if let Some(ref sec) = doc.section {
            *section_counts.entry(sec.clone()).or_insert(0) += 1;
        }

        let incoming = backlinks.get(&doc.slug).map(Vec::as_slice).unwrap_or(&[]);
        let html = match renderer.render_page(doc, incoming, &wiki_index) {
            Ok(h) => h,
            Err(e) => {
                failures.push(DocFailure {
                    rel_path: doc.rel_path.clone(),
                    error: format!("could not render page: {e}"),
                });
                continue;
            }
        };

        let out_html_path = wiki_root.join(&doc.slug).join("index.html");
        if write_if_changed(&out_html_path, &html)? {
            written += 1;
        }
    }

    let rendered = valid_docs.len();

    // 9. Catalog page
    let mut sections: Vec<SectionSummary> = section_counts
        .into_iter()
        .map(|(name, count)| SectionSummary { name, count })
        .collect();
    sections.sort_by(|a, b| a.name.cmp(&b.name));

    let index_html = renderer.render_index(&sections, &wiki_index)?;
    let _ = write_if_changed(&wiki_root.join("index.html"), &index_html)?;

    // 10. Root redirect (if url_prefix != "")
    if !wiki_rel.is_empty() {
        let redirect_target = if clean_url_prefix.is_empty() {
            "/".to_string()
        } else {
            clean_url_prefix.to_string()
        };
        let redirect_html = format!(
            r#"<!doctype html><html lang="{lang}" data-pagefind-ignore><head><meta charset="utf-8"><title>Redirecting to: {prefix}</title><meta http-equiv="refresh" content="0;url={prefix}"><link rel="canonical" href="{prefix}"></head><body><a href="{prefix}">Redirecting to {prefix}</a></body></html>"#,
            lang = config.wiki.lang,
            prefix = redirect_target
        );
        let _ = write_if_changed(&out_dir.join("index.html"), &redirect_html)?;
    }

    // 11. 404 page
    let not_found_html = renderer.render_not_found("")?;
    let _ = write_if_changed(&out_dir.join("404.html"), &not_found_html)?;

    // 12. Search indexing
    if !skip_search {
        match config.build.search {
            SearchEngine::Native => {
                let search_index = build_native_search_index(&valid_docs, clean_url_prefix);
                let json = search_index.to_json()?;
                write_if_changed(&out_dir.join("search-index.json"), &json)?;
            }
            SearchEngine::Pagefind => {
                let _ = run_pagefind(out_dir);
            }
            SearchEngine::None => {}
        }
    }

    Ok(BuildReport {
        rendered,
        written,
        failures,
        backlinks,
        route_table,
    })
}
