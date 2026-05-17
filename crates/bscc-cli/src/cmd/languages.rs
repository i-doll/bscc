use anyhow::Result;
use bscc_core::Registry;

#[allow(clippy::unnecessary_wraps)] // keep signature uniform across subcommands
pub fn run(registry: &Registry) -> Result<()> {
    println!("{:<20} {:<12} Extensions", "Language", "Tier");
    println!("{}", "-".repeat(60));
    let mut langs: Vec<_> = registry.languages().collect();
    langs.sort_by(|a, b| a.name.cmp(&b.name));
    for entry in langs {
        let tier = match entry.tier {
            bscc_core::Tier::Regex => "regex",
            bscc_core::Tier::TreeSitter => "tree-sitter",
        };
        let exts = entry.extensions.join(", ");
        println!("{:<20} {:<12} {}", entry.name, tier, exts);
    }
    Ok(())
}
