use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("tomet-wiki-cli-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn test_cli_init_and_build() {
    let dir = temp_dir("init-build");

    let bin_path = env!("CARGO_BIN_EXE_tomet-wiki");

    // 1. tomet-wiki init
    let init_status = Command::new(bin_path)
        .arg("init")
        .arg(&dir)
        .arg("--title")
        .arg("Test Workspace Wiki")
        .arg("--owner")
        .arg("acme")
        .arg("--repo")
        .arg("wiki")
        .status()
        .expect("failed to run init");

    assert!(init_status.success());

    let wiki_toml = dir.join("wiki.toml");
    assert!(wiki_toml.exists());
    let toml_content = fs::read_to_string(&wiki_toml).unwrap();
    assert!(toml_content.contains("Test Workspace Wiki"));
    assert!(toml_content.contains(r#"owner = "acme""#));
    assert!(toml_content.contains(r#"repo = "wiki""#));

    // Running init again should fail
    let duplicate_init = Command::new(bin_path)
        .arg("init")
        .arg(&dir)
        .status()
        .expect("failed to run duplicate init");
    assert!(!duplicate_init.success());

    // 2. Add a document
    fs::write(
        dir.join("index.tmt"),
        "#[ Home ]\n\nWelcome to Test Workspace Wiki.\n",
    )
    .unwrap();

    // 3. tomet-wiki build
    let build_status = Command::new(bin_path)
        .arg("build")
        .arg(&dir)
        .status()
        .expect("failed to run build");

    assert!(build_status.success());
    let dist = dir.join("dist");
    assert!(dist.exists());
    assert!(dist.join("wiki/index.html").exists());

    let _ = fs::remove_dir_all(&dir);
}
