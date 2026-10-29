use anyhow::Result;
use notify::Event;
use percent_encoding::percent_decode_str;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::net::IpAddr;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::time::Instant;
use tracing::{info, warn};

use crate::config::WikiConfig;
use crate::renderer::{Backlink, EntrySummary, PageRenderer, RendererFactory, SectionSummary};
use crate::scan::{ScannedVault, exclude_prefixes, is_ignored_rel, scan_vault};
use crate::wiki::{build_backlink_index, build_native_search_index, kind_publishes};
use tomet_shelf_document::ProcessedDoc;
use tomet_shelf_route::{DocRouteInput, RouteTable, extract_route_metadata};
use tomet_shelf_serve::axum::Router;
use tomet_shelf_serve::axum::http::{StatusCode, Uri, header};
use tomet_shelf_serve::axum::response::{Html, IntoResponse, Response};
use tomet_shelf_serve::{DevServerHandler, ReloadSignal, resolve_within};

pub struct DevVaultState {
    pub config: WikiConfig,
    pub src_dir: PathBuf,
    pub scanned: ScannedVault,
    pub route_table: RouteTable,
    pub backlinks: HashMap<String, Vec<Backlink>>,
    pub unpublished: HashSet<String>,
    pub wiki_index: Vec<EntrySummary>,
    pub renderer: Box<dyn PageRenderer>,
    pub sections: Vec<SectionSummary>,
    pub search_index_json: Option<String>,
}

impl DevVaultState {
    pub fn init(
        src_dir: &Path,
        config: &WikiConfig,
        renderer_factory: &dyn RendererFactory,
        is_dev: bool,
    ) -> Result<Self> {
        let scanned = scan_vault(src_dir, config)?;
        let book_config = config.to_doc_book_config();

        let mut renderer = renderer_factory.create(config, is_dev)?;
        let icon_renderer = renderer
            .icon_renderer()
            .or_else(|| renderer_factory.icon_renderer());

        let mut parsed = Vec::new();
        for doc_file in &scanned.doc_files {
            let Ok(source) = fs::read_to_string(&doc_file.abs_path) else {
                continue;
            };
            if let Ok(doc) = tomet_shelf_document::parse_tomet_document(&source, &doc_file.rel_path) {
                parsed.push((doc, doc_file.clone()));
            }
        }

        let unpublished: HashSet<String> = parsed
            .iter()
            .filter(|((_, kind), _)| !kind_publishes(kind.as_deref(), &config.build.kinds))
            .map(|(_, doc_file)| doc_file.rel_path.clone())
            .collect();

        let mut route_meta = Vec::new();
        for ((doc, _), doc_file) in &parsed {
            if unpublished.contains(&doc_file.rel_path) {
                continue;
            }
            let (slug, id) = extract_route_metadata(doc);
            route_meta.push((doc_file.rel_path.as_str(), slug, id));
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

        let custom_element = Some(tomet_shelf_render::create_doc_icon_custom_element(
            config.build.clean_base_path(),
            icon_renderer,
        ));

        let mut processed = Vec::new();
        for ((doc, _), doc_file) in parsed {
            if unpublished.contains(&doc_file.rel_path) {
                continue;
            }
            if let Ok(p) = tomet_shelf_document::process_parsed_document_with_custom_element(
                doc,
                &doc_file.rel_path,
                Some(&doc_file.abs_path),
                &book_config,
                &scanned.vault_index,
                &scanned.workspace_config_blocks,
                &unpublished,
                Some(&route_table),
                custom_element.clone(),
            ) {
                processed.push(p);
            }
        }

        if config.ui.links.note_icons {
            let doc_icons: HashMap<String, tomet_shelf_document::DocIcon> = processed
                .iter()
                .filter_map(|doc| doc.icon.as_ref().map(|i| (doc.slug.clone(), i.clone())))
                .collect();
            renderer.set_doc_icons(&doc_icons);
        }

        let valid_docs: Vec<&ProcessedDoc> = processed.iter().collect();
        let clean_prefix = config.build.full_url_prefix();
        let backlinks = build_backlink_index(&valid_docs, &clean_prefix);

        let mut section_counts: HashMap<String, usize> = HashMap::new();
        let mut wiki_index = Vec::new();
        for doc in &valid_docs {
            if let Some(ref s) = doc.section {
                *section_counts.entry(s.clone()).or_insert(0) += 1;
            }
            wiki_index.push(EntrySummary {
                url: format!("{clean_prefix}/{}", doc.slug),
                title: doc.title.clone(),
                section: doc.section.clone(),
                children: Vec::new(),
            });
        }

        let mut sections: Vec<SectionSummary> = section_counts
            .into_iter()
            .map(|(name, count)| SectionSummary { name, count })
            .collect();
        sections.sort_by(|a, b| a.name.cmp(&b.name));

        let search_index = build_native_search_index(&valid_docs, &clean_prefix);
        let search_index_json = search_index.to_json().ok();

        Ok(Self {
            config: config.clone(),
            src_dir: src_dir.to_path_buf(),
            scanned,
            route_table,
            backlinks,
            unpublished,
            wiki_index,
            renderer,
            sections,
            search_index_json,
        })
    }

    pub fn render_page_by_slug(&self, slug: &str) -> Result<Option<String>> {
        let rel_path = match self.route_table.rel_path_for_slug(slug) {
            Some(path) => path,
            None => return Ok(None),
        };

        let abs_path = self.src_dir.join(rel_path);
        let source = fs::read_to_string(&abs_path)?;
        let (doc, _) = tomet_shelf_document::parse_tomet_document(&source, rel_path)?;

        let book_config = self.config.to_doc_book_config();
        let icon_renderer = self.renderer.icon_renderer();
        let custom_element = Some(tomet_shelf_render::create_doc_icon_custom_element(
            self.config.build.clean_base_path(),
            icon_renderer,
        ));

        let processed = tomet_shelf_document::process_parsed_document_with_custom_element(
            doc,
            rel_path,
            Some(&abs_path),
            &book_config,
            &self.scanned.vault_index,
            &self.scanned.workspace_config_blocks,
            &self.unpublished,
            Some(&self.route_table),
            custom_element,
        )?;

        let incoming = self.backlinks.get(slug).map(Vec::as_slice).unwrap_or(&[]);
        let html = self.renderer.render_page(&processed, incoming, &self.wiki_index)?;
        Ok(Some(html))
    }

    pub fn render_index_page(&self) -> Result<String> {
        self.renderer.render_index(&self.sections, &self.wiki_index)
    }

    pub fn render_not_found_page(&self, missing: &str) -> Result<String> {
        self.renderer.render_not_found(missing)
    }
}

pub struct WikiDevHandler {
    src_dir: PathBuf,
    out_dir: PathBuf,
    config: RwLock<WikiConfig>,
    renderer_factory: Arc<dyn RendererFactory>,
    excludes: RwLock<Vec<String>>,
    state: Arc<RwLock<Option<DevVaultState>>>,
}

impl WikiDevHandler {
    pub fn new(
        src_dir: PathBuf,
        out_dir: PathBuf,
        config: WikiConfig,
        renderer_factory: Arc<dyn RendererFactory>,
    ) -> Self {
        let excludes = exclude_prefixes(&config);
        Self {
            src_dir,
            out_dir,
            config: RwLock::new(config),
            renderer_factory,
            excludes: RwLock::new(excludes),
            state: Arc::new(RwLock::new(None)),
        }
    }

    pub fn state(&self) -> &Arc<RwLock<Option<DevVaultState>>> {
        &self.state
    }

    pub fn full_rebuild(&self) -> bool {
        let start = Instant::now();
        let reloaded_config = match WikiConfig::load_from_dir(&self.src_dir) {
            Ok(cfg) => {
                info!("⚙️ Reloaded configuration from {}", self.src_dir.display());
                let mut cfg_lock = self.config.write().unwrap();
                *cfg_lock = cfg.clone();
                let mut exc_lock = self.excludes.write().unwrap();
                *exc_lock = exclude_prefixes(&cfg);
                cfg
            }
            Err(e) => {
                warn!("Failed to reload wiki.toml: {e}");
                self.config.read().unwrap().clone()
            }
        };

        match DevVaultState::init(
            &self.src_dir,
            &reloaded_config,
            self.renderer_factory.as_ref(),
            true,
        ) {
            Ok(vault_state) => {
                info!(
                    "⚡ In-memory vault index ready in {:?} ({} docs, {} routes)",
                    start.elapsed(),
                    vault_state.scanned.doc_files.len(),
                    vault_state.scanned.doc_files.len() - vault_state.unpublished.len()
                );
                *self.state.write().unwrap() = Some(vault_state);
                true
            }
            Err(e) => {
                warn!("Dev rebuild error: {e}");
                false
            }
        }
    }
}

impl DevServerHandler for WikiDevHandler {
    fn on_init(&self) -> Result<()> {
        let config = self.config.read().unwrap().clone();
        match DevVaultState::init(&self.src_dir, &config, self.renderer_factory.as_ref(), true) {
            Ok(vault_state) => {
                info!(
                    "⚡ Dev server ready (in-memory SSR, {} docs)",
                    vault_state.scanned.doc_files.len()
                );
                *self.state.write().unwrap() = Some(vault_state);
            }
            Err(e) => warn!("Initial vault indexing failed: {e}"),
        }
        Ok(())
    }

    fn on_events(&self, events: &[Event]) -> Vec<ReloadSignal> {
        let mut signals = Vec::new();
        let excludes = self.excludes.read().unwrap().clone();

        let mut needs_full = false;
        let mut needs_css = false;
        let mut doc_changes = Vec::new();

        for event in events {
            if matches!(event.kind, notify::EventKind::Access(_)) {
                continue;
            }

            for path in &event.paths {
                if path.starts_with(&self.out_dir) {
                    continue;
                }
                let Ok(rel) = path.strip_prefix(&self.src_dir) else {
                    continue;
                };
                let rel_str = tomet_shelf_route::normalize_path_str(&rel.to_string_lossy());
                if is_ignored_rel(&rel_str, &excludes) {
                    continue;
                }

                let file_name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
                if file_name == "wiki.toml"
                    || file_name.ends_with(".index.tmt")
                    || file_name.ends_with(".config.tmt")
                {
                    needs_full = true;
                    continue;
                }

                if file_name.ends_with(".css") {
                    needs_css = true;
                    continue;
                }

                let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
                if ext == "tmt" || ext == "tm" {
                    doc_changes.push(rel_str);
                } else if crate::scan::MEDIA_EXTENSIONS.contains(&ext) {
                    needs_full = true;
                }
            }
        }

        if needs_full {
            info!("🔄 Config or global change detected, re-indexing wiki...");
            if self.full_rebuild() {
                signals.push(ReloadSignal::Full);
            }
            return signals;
        }

        if needs_css {
            signals.push(ReloadSignal::Css);
        }

        if !doc_changes.is_empty() {
            let state_lock = self.state.read().unwrap();
            if let Some(ref st) = *state_lock {
                let clean_prefix = st.config.build.clean_url_prefix();
                for rel in &doc_changes {
                    if let Some(slug) = st.route_table.get(rel) {
                        let doc_url = format!("{clean_prefix}/{slug}");
                        signals.push(ReloadSignal::Doc { url: doc_url });
                    } else {
                        drop(state_lock);
                        if self.full_rebuild() {
                            signals.push(ReloadSignal::Full);
                        }
                        return signals;
                    }
                }
            } else {
                drop(state_lock);
                if self.full_rebuild() {
                    signals.push(ReloadSignal::Full);
                }
            }
        }

        signals
    }

    fn extend_router(&self, router: Router) -> Router {
        let state = Arc::clone(&self.state);
        let src_dir = self.src_dir.clone();
        let fallback_config = self.config.read().unwrap().clone();

        router.fallback(move |uri: Uri| {
            let state = Arc::clone(&state);
            let src_dir = src_dir.clone();
            let fallback_config = fallback_config.clone();
            async move { handle_http_request(&uri, &state, &src_dir, &fallback_config) }
        })
    }
}

pub fn handle_http_request(
    uri: &Uri,
    state: &RwLock<Option<DevVaultState>>,
    src_dir: &Path,
    config: &WikiConfig,
) -> Response {
    let raw_path = uri.path();
    let path = percent_decode_str(raw_path)
        .decode_utf8()
        .map(|c| c.into_owned())
        .unwrap_or_else(|_| raw_path.to_string());

    let (clean_base_path, clean_url_prefix, clean_asset_prefix) = {
        let state_lock = state.read().unwrap();
        if let Some(ref st) = *state_lock {
            (
                st.config.build.clean_base_path().to_string(),
                st.config.build.full_url_prefix(),
                st.config.build.full_asset_prefix(),
            )
        } else {
            (
                config.build.clean_base_path().to_string(),
                config.build.full_url_prefix(),
                config.build.full_asset_prefix(),
            )
        }
    };

    let unbased_path = if !clean_base_path.is_empty() {
        path.strip_prefix(&clean_base_path).unwrap_or(&path)
    } else {
        path.as_str()
    };

    // 1. Static assets (theme static assets)
    let rel_asset = unbased_path.trim_start_matches('/');
    {
        let state_lock = state.read().unwrap();
        if let Some(ref st) = *state_lock {
            for asset in st.renderer.static_assets() {
                if asset.path.trim_start_matches('/') == rel_asset {
                    return (
                        StatusCode::OK,
                        [(header::CONTENT_TYPE, asset.mime)],
                        asset.content,
                    )
                        .into_response();
                }
            }
        }
    }

    // 2. Search index
    let is_search_index = unbased_path == "/search-index.json"
        || (!clean_url_prefix.is_empty()
            && path == format!("{clean_url_prefix}/search-index.json"));

    if is_search_index {
        let state_lock = state.read().unwrap();
        if let Some(ref st) = *state_lock
            && let Some(ref json) = st.search_index_json
        {
            return (
                StatusCode::OK,
                [(header::CONTENT_TYPE, "application/json; charset=utf-8")],
                json.clone(),
            )
                .into_response();
        }
    }

    // 3. Root redirect
    let is_root = path == "/"
        || path.is_empty()
        || (!clean_base_path.is_empty()
            && (path == clean_base_path || path == format!("{clean_base_path}/")));

    if is_root {
        if !clean_url_prefix.is_empty() {
            let redirect_target = format!("{clean_url_prefix}/");
            return (StatusCode::FOUND, [(header::LOCATION, redirect_target)], "").into_response();
        } else {
            let state_lock = state.read().unwrap();
            if let Some(ref st) = *state_lock {
                match st.render_index_page() {
                    Ok(html) => return Html(html).into_response(),
                    Err(e) => {
                        return (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Html(format!("<h1>500 Internal Server Error</h1><pre>{e}</pre>")),
                        )
                            .into_response();
                    }
                }
            }
        }
    }

    // 4. Catalog index
    let is_catalog_index = if !clean_url_prefix.is_empty() {
        path == clean_url_prefix
            || path == format!("{clean_url_prefix}/")
            || path == format!("{clean_url_prefix}/index.html")
    } else {
        unbased_path == "/" || unbased_path == "/index.html"
    };

    if is_catalog_index {
        let state_lock = state.read().unwrap();
        if let Some(ref st) = *state_lock {
            match st.render_index_page() {
                Ok(html) => return Html(html).into_response(),
                Err(e) => {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Html(format!("<h1>500 Internal Server Error</h1><pre>{e}</pre>")),
                    )
                        .into_response();
                }
            }
        }
    }

    // 5. Wiki page SSR by slug
    let wiki_subpath = if !clean_url_prefix.is_empty() {
        path.strip_prefix(&clean_url_prefix)
            .map(|s| s.trim_matches('/'))
    } else {
        Some(unbased_path.trim_matches('/'))
    };

    if let Some(sub) = wiki_subpath
        && !sub.is_empty()
    {
        let slug = sub
            .trim_end_matches("/index.html")
            .trim_end_matches(".html")
            .trim_matches('/');
        let state_lock = state.read().unwrap();
        if let Some(ref st) = *state_lock {
            match st.render_page_by_slug(slug) {
                Ok(Some(html)) => return Html(html).into_response(),
                Ok(None) => {}
                Err(e) => {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Html(format!("<h1>500 Internal Server Error</h1><pre>{e}</pre>")),
                    )
                        .into_response();
                }
            }
        }
    }

    // 6. Vault media streaming
    let media_rel = if !clean_asset_prefix.is_empty()
        && let Some(rel) = path.strip_prefix(&clean_asset_prefix)
    {
        Some(rel.trim_matches('/'))
    } else {
        unbased_path.strip_prefix("/vault").map(|s| s.trim_matches('/'))
    };

    if let Some(rel) = media_rel
        && !rel.is_empty()
        && let Ok(target) = resolve_within(src_dir, rel)
        && target.is_file()
        && let Ok(bytes) = fs::read(&target)
    {
        let mime = guess_mime(&target);
        return (StatusCode::OK, [(header::CONTENT_TYPE, mime)], bytes).into_response();
    }

    // 7. 404 fallback
    let not_found_html = {
        let state_lock = state.read().unwrap();
        if let Some(ref st) = *state_lock {
            st.render_not_found_page(&path).unwrap_or_else(|_| {
                format!(
                    "<!doctype html><html><head><meta charset=\"utf-8\"><title>404 Not Found</title></head><body><h1>404 Not Found</h1><p>Cannot find: <code>{}</code></p></body></html>",
                    path
                )
            })
        } else {
            format!(
                "<!doctype html><html><head><meta charset=\"utf-8\"><title>404 Not Found</title></head><body><h1>404 Not Found</h1><p>Cannot find: <code>{}</code></p></body></html>",
                path
            )
        }
    };
    (StatusCode::NOT_FOUND, Html(not_found_html)).into_response()
}

fn guess_mime(path: &Path) -> &'static str {
    match path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase()
        .as_str()
    {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "webp" => "image/webp",
        "gif" => "image/gif",
        "svg" => "image/svg+xml",
        "mp4" => "video/mp4",
        "webm" => "video/webm",
        "mp3" => "audio/mpeg",
        "css" => "text/css; charset=utf-8",
        "js" => "application/javascript; charset=utf-8",
        "json" => "application/json",
        "html" => "text/html; charset=utf-8",
        _ => "application/octet-stream",
    }
}

pub async fn run_dev_server(
    src_dir: PathBuf,
    out_dir: PathBuf,
    config: WikiConfig,
    renderer_factory: Arc<dyn RendererFactory>,
    host: IpAddr,
    port: u16,
) -> Result<()> {
    let prefix = Some(config.build.clean_url_prefix().to_string());
    let handler = WikiDevHandler::new(src_dir.clone(), out_dir.clone(), config, renderer_factory);
    tomet_shelf_serve::run_dev_server(src_dir, None, host, port, prefix, handler).await
}
