use tomet_shelf_document::ProcessedDoc;
use tomet_wiki_assets::ASSETS;
use tomet_wiki_headless::{Backlink, EntrySummary, RendererFactory, SectionSummary, WikiConfig};

fn create_sample_doc(body_html: &str) -> ProcessedDoc {
    ProcessedDoc {
        slug: "guides/getting-started".to_string(),
        rel_path: "guides/getting-started.tmt".to_string(),
        source_path: "/vault/guides/getting-started.tmt".to_string(),
        title: "Getting Started".to_string(),
        section: Some("guides".to_string()),
        kind: None,
        primary_color: None,
        icon: None,
        icon_image_url: None,
        banner_url: None,
        banner_original_url: None,
        banner_y: None,
        images: vec![],
        original_images: vec![],
        hero_chips: vec![],
        infobox_rows: vec![],
        has_data: false,
        toc: vec![tomet_shelf_document::TocItem {
            level: 2,
            id: "step-1".to_string(),
            text: "Step 1: Installation".to_string(),
        }],
        section_tabs: vec![],
        body_html: body_html.to_string(),
        outgoing: vec!["overview".to_string()],
    }
}

#[test]
fn test_wiki_page_renders_without_sidebar_and_has_editor_mount_points() {
    let mut config = WikiConfig::default();
    config.wiki.title = "Engineering Wiki".to_string();
    config.cms.owner = "acme".to_string();
    config.cms.repo = "wiki".to_string();

    let renderer = ASSETS.create(&config, false).expect("renderer creation failed");

    let doc = create_sample_doc(
        r#"<p>Follow these steps.</p>
<div class="tm-element tm-conflict"><div class="tm-conflict-a">Local edits</div><div class="tm-conflict-b">Remote edits</div></div>"#,
    );

    let backlinks = vec![Backlink {
        url: "/wiki/overview".to_string(),
        title: "Project Overview".to_string(),
        section: Some("docs".to_string()),
    }];

    let wiki_index = vec![EntrySummary {
        url: "/wiki/guides/getting-started".to_string(),
        title: "Getting Started".to_string(),
        section: Some("guides".to_string()),
        children: vec![],
    }];

    let html = renderer.render_page(&doc, &backlinks, &wiki_index).expect("render_page failed");

    // 1. Check title & brand
    assert!(html.contains("Getting Started | Engineering Wiki"));
    assert!(html.contains(r#"class="tmt-wiki-brand">Engineering Wiki</a>"#));

    // 2. No sidebar in wiki theme!
    assert!(!html.contains("sidebar"));
    assert!(!html.contains("tmt-shelf-sidebar"));
    assert!(!html.contains("pagination-prev"));
    assert!(!html.contains("pagination-next"));

    // 3. Editor mount points & buttons
    assert!(html.contains(r#"id="tmt-wiki-editor-root""#), "Must have editor mount root");
    assert!(html.contains(r#"id="tmt-wiki-edit-btn""#), "Must have edit button");
    assert!(html.contains(r#"data-rel-path="guides/getting-started.tmt""#), "Edit button must have rel_path");
    assert!(html.contains(r#"id="tmt-wiki-new-btn""#), "Must have new page button");
    assert!(html.contains(r#"id="tmt-wiki-search-trigger""#), "Must have search trigger");

    // 4. Main container with data-tmt-rel-path
    assert!(html.contains(r#"data-tmt-rel-path="guides/getting-started.tmt""#));

    // 5. Backlinks section
    assert!(html.contains("Linked References"));
    assert!(html.contains("/wiki/overview"));
    assert!(html.contains("Project Overview"));
    assert!(html.contains("docs"));

    // 6. Conflict blocks present in body
    assert!(html.contains("tm-conflict"));
    assert!(html.contains("Local edits"));
    assert!(html.contains("Remote edits"));

    // 7. Client config injected
    assert!(html.contains("window.tmtWikiCms = {\"owner\":\"acme\",\"repo\":\"wiki\",\"branch\":\"main\"}"));

    // 8. Editor assets linked and present
    assert!(html.contains("wiki-editor.css"));
    let static_assets = renderer.static_assets();
    assert!(static_assets.iter().any(|a| a.path == "wiki-editor.js"));
    assert!(static_assets.iter().any(|a| a.path == "wiki-editor.css"));
    assert!(static_assets.iter().any(|a| a.path == "wiki.css"));
    assert!(static_assets.iter().any(|a| a.path == "wiki.js"));
}

#[test]
fn test_wiki_index_and_404_render() {
    let config = WikiConfig::default();
    let renderer = ASSETS.create(&config, false).unwrap();

    // Index page
    let sections = vec![SectionSummary {
        name: "Guides".to_string(),
        count: 5,
    }];
    let entries = vec![EntrySummary {
        url: "/wiki/start".to_string(),
        title: "Start".to_string(),
        section: Some("Guides".to_string()),
        children: vec![],
    }];

    let index_html = renderer.render_index(&sections, &entries).unwrap();
    assert!(index_html.contains("All Pages"));
    assert!(index_html.contains("Guides (5)"));
    assert!(index_html.contains("Start"));
    assert!(!index_html.contains("sidebar"));

    // 404 page
    let not_found_html = renderer.render_not_found("unknown/page").unwrap();
    assert!(not_found_html.contains("404"));
    assert!(not_found_html.contains("unknown/page"));
    assert!(not_found_html.contains("Return to Home"));
}
