use clap::Parser;

#[derive(Parser, Debug)]
#[command(
    name = "tomet-wiki",
    version,
    about = "Wiki system for the Tomet ecosystem."
)]
struct Args {
    #[arg(default_value = ".")]
    path: std::path::PathBuf,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    println!("tomet-wiki initialized at: {:?}", args.path);
    Ok(())
}
