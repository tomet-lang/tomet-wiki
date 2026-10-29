use std::fs;
use std::path::PathBuf;
use tomet_wiki_headless::{MinimalWikiRendererFactory, WikiConfig, build_wiki};

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "tomet-wiki-test-{name}-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn test_build_wiki_emits_cms_and_rel_path_and_conflict() {
    let src = temp_dir("build-src");
    let out = temp_dir("build-out");

    let wiki_toml = r#"
[wiki]
title = "Test Wiki"
lang = "ja"

[cms]
owner = "tomet-lang"
repo = "tomet-wiki"
branch = "main"
auth_endpoint = "https://auth.tomet.dev/oauth"

[build]
url_prefix = "/wiki"
"#;
    fs::write(src.join("wiki.toml"), wiki_toml).unwrap();

    let notes_dir = src.join("notes");
    fs::create_dir_all(&notes_dir).unwrap();

    // Document with link to other and conflict directive
    let hello_doc = r#"#[ Hello Wiki ]

Welcome to the test wiki. See [[other]].

@conflict(
  a: [Version from branch A],
  b: [Version from branch B]
)
"#;
    fs::write(notes_dir.join("hello.tmt"), hello_doc).unwrap();

    let other_doc = r#"#[ Other Page ]

Backlink target pointing to [[hello]].
"#;
    fs::write(notes_dir.join("other.tmt"), other_doc).unwrap();

    let config = WikiConfig::load_from_dir(&src).unwrap();
    assert_eq!(config.cms.owner, "tomet-lang");
    assert_eq!(config.cms.repo, "tomet-wiki");

    let report = build_wiki(&src, &out, &config, &MinimalWikiRendererFactory, true)
        .expect("build_wiki failed");

    assert!(report.failures.is_empty(), "failures: {:?}", report.failures);
    assert_eq!(report.rendered, 2);

    let hello_html_path = out.join("wiki/notes/hello/index.html");
    assert!(hello_html_path.exists(), "hello/index.html must exist");

    let html = fs::read_to_string(&hello_html_path).unwrap();

    // 1. data-tmt-rel-path on <main>
    assert!(
        html.contains(r#"data-tmt-rel-path="notes/hello.tmt""#),
        "HTML must contain data-tmt-rel-path"
    );

    // 2. window.tmtWikiCms injection
    assert!(
        html.contains("window.tmtWikiCms = {"),
        "HTML must contain window.tmtWikiCms"
    );
    assert!(html.contains(r#""owner":"tomet-lang""#));
    assert!(html.contains(r#""repo":"tomet-wiki""#));
    assert!(html.contains(r#""branch":"main""#));
    assert!(html.contains(r#""authEndpoint":"https://auth.tomet.dev/oauth""#));

    // 3. Rendered conflict block (.tm-conflict)
    eprintln!("GENERATED HTML:\n{html}");
    assert!(
        html.contains("tm-conflict"),
        "HTML must contain .tm-conflict class for @conflict blocks: {html}"
    );
    assert!(html.contains("Version from branch A"));
    assert!(html.contains("Version from branch B"));

    // 4. Backlinks
    let other_html_path = out.join("wiki/notes/other/index.html");
    assert!(other_html_path.exists());

    let _ = fs::remove_dir_all(&src);
    let _ = fs::remove_dir_all(&out);
}
