use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tomet_shelf_serve::DevServerHandler;
use tomet_shelf_serve::axum::body::to_bytes;
use tomet_shelf_serve::axum::http::{StatusCode, Uri, header};
use tomet_wiki_headless::{MinimalWikiRendererFactory, WikiConfig, WikiDevHandler};

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "tomet-wiki-dev-test-{name}-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

#[tokio::test]
async fn test_dev_server_ssr_renders_page_and_handles_wiki_toml_change() {
    let src = temp_dir("dev-ssr-src");
    let out = temp_dir("dev-ssr-out");

    let wiki_toml = r#"
[wiki]
title = "Live Wiki"
lang = "ja"

[cms]
owner = "tomet-lang"
repo = "wiki"
branch = "main"

[build]
url_prefix = "/wiki"
"#;
    fs::write(src.join("wiki.toml"), wiki_toml).unwrap();

    let notes = src.join("guides");
    fs::create_dir_all(&notes).unwrap();
    fs::write(
        notes.join("start.tmt"),
        "#[ Quick Start ]\n\nGetting started with Tomet Wiki.\n",
    )
    .unwrap();

    let config = WikiConfig::load_from_dir(&src).unwrap();
    let handler = WikiDevHandler::new(
        src.clone(),
        out.clone(),
        config.clone(),
        Arc::new(MinimalWikiRendererFactory),
    );

    handler.on_init().expect("on_init failed");

    // 1. SSR request to /wiki/guides/start
    let uri = Uri::from_static("/wiki/guides/start");
    let resp = tomet_wiki_headless::serve::handle_http_request(&uri, handler.state(), &src, &config);
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(
        resp.headers().get(header::CONTENT_TYPE).unwrap(),
        "text/html; charset=utf-8"
    );

    let body_bytes = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let body_str = String::from_utf8_lossy(&body_bytes);
    assert!(body_str.contains("Quick Start"));
    assert!(body_str.contains(r#"data-tmt-rel-path="guides/start.tmt""#));
    assert!(body_str.contains(r#""owner":"tomet-lang""#));

    // 2. Modify wiki.toml and notify events
    let updated_toml = r#"
[wiki]
title = "Renamed Wiki"
lang = "ja"

[cms]
owner = "new-owner"
repo = "wiki"
branch = "dev"

[build]
url_prefix = "/wiki"
"#;
    fs::write(src.join("wiki.toml"), updated_toml).unwrap();

    let event = notify::Event {
        kind: notify::EventKind::Modify(notify::event::ModifyKind::Any),
        paths: vec![src.join("wiki.toml")],
        attrs: Default::default(),
    };

    let signals = handler.on_events(&[event]);
    assert_eq!(signals, vec![tomet_shelf_serve::ReloadSignal::Full]);

    // Check that config is reloaded
    let resp2 = tomet_wiki_headless::serve::handle_http_request(&uri, handler.state(), &src, &config);
    let body_bytes2 = to_bytes(resp2.into_body(), usize::MAX).await.unwrap();
    let body_str2 = String::from_utf8_lossy(&body_bytes2);
    assert!(body_str2.contains("Renamed Wiki"));
    assert!(body_str2.contains(r#""owner":"new-owner""#));
    assert!(body_str2.contains(r#""branch":"dev""#));

    let _ = fs::remove_dir_all(&src);
    let _ = fs::remove_dir_all(&out);
}
