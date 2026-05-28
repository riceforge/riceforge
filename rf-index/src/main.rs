use chrono::Utc;
use clap::{Parser, Subcommand};
use rf_core::{Index, Rice, WindowManager};
use serde::Deserialize;
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

#[derive(Parser)]
#[command(
    name = "rf-index",
    about = "RiceForge index builder and validator",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Validate a rice.toml file")]
    Validate { path: PathBuf },

    #[command(about = "Build index.json from a directory of rice.toml files")]
    Build {
        directory: PathBuf,
        #[arg(long, short, default_value = "index.json")]
        output: PathBuf,
        #[arg(long)]
        base_url: Option<String>,
    },

    #[command(about = "Update star counts in an existing index.json")]
    UpdateStars {
        index_path: PathBuf,
        #[arg(long, env = "GITHUB_TOKEN", help = "GitHub token to avoid rate limits")]
        token: Option<String>,
    },
}

#[derive(Debug, Deserialize)]
struct RiceToml {
    id: String,
    name: String,
    author: String,
    description: String,
    wm: String,
    theme: String,
    #[serde(default)]
    fonts: Vec<String>,
    #[serde(default)]
    dependencies: Vec<String>,
    repo_url: String,
    #[serde(default)]
    screenshots: Vec<String>,
}

fn main() {
    let cli = Cli::parse();
    if let Err(e) = run(cli) {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}

fn run(cli: Cli) -> anyhow::Result<()> {
    match cli.command {
        Commands::Validate { path } => cmd_validate(&path),
        Commands::Build { directory, output, base_url } => {
            cmd_build(&directory, &output, base_url.as_deref())
        }
        Commands::UpdateStars { index_path, token } => {
            cmd_update_stars(&index_path, token.as_deref())
        }
    }
}

fn cmd_validate(path: &Path) -> anyhow::Result<()> {
    let content = fs::read_to_string(path)
        .map_err(|e| anyhow::anyhow!("cannot read {}: {e}", path.display()))?;

    let rice: RiceToml = toml::from_str(&content)
        .map_err(|e| anyhow::anyhow!("TOML parse error in {}: {e}", path.display()))?;

    let mut errors: Vec<String> = Vec::new();

    if rice.id.is_empty() || rice.id.contains(' ') {
        errors.push("id must be non-empty and contain no spaces".into());
    }
    if !rice
        .id
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
    {
        errors.push("id must only contain alphanumeric characters, hyphens and underscores".into());
    }
    if rice.name.is_empty() {
        errors.push("name is required".into());
    }
    if rice.author.is_empty() {
        errors.push("author is required".into());
    }
    if rice.description.len() < 20 {
        errors.push("description must be at least 20 characters".into());
    }
    if rice.description.len() > 300 {
        errors.push("description must be at most 300 characters".into());
    }
    if !rice.repo_url.starts_with("https://github.com/") {
        errors.push("repo_url must be a GitHub HTTPS URL".into());
    }
    const ALLOWED_WMS: &[&str] = &[
        "hyprland", "sway", "i3", "bspwm", "qtile", "xmonad", "openbox",
    ];
    let wm_lower = rice.wm.to_lowercase();
    if rice.wm.is_empty() {
        errors.push("wm is required".into());
    } else if !ALLOWED_WMS.contains(&wm_lower.as_str()) {
        errors.push(format!(
            "wm '{}' is not allowed; valid values: {}",
            rice.wm,
            ALLOWED_WMS.join(", ")
        ));
    }
    if rice.theme.is_empty() {
        errors.push("theme is required".into());
    }

    if errors.is_empty() {
        println!("✓ {} is valid", path.display());
    } else {
        for e in &errors {
            eprintln!("✗ {e}");
        }
        return Err(anyhow::anyhow!("{} validation errors", errors.len()));
    }

    Ok(())
}

fn cmd_build(dir: &Path, output: &Path, base_url: Option<&str>) -> anyhow::Result<()> {
    if !dir.is_dir() {
        return Err(anyhow::anyhow!("{} is not a directory", dir.display()));
    }

    let mut rices: Vec<Rice> = Vec::new();

    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        let (toml_path, rice_dir) = if path.is_dir() {
            (path.join("rice.toml"), Some(path.clone()))
        } else if path.extension().is_some_and(|e| e == "toml") {
            (path.clone(), None)
        } else {
            continue;
        };

        if !toml_path.exists() {
            continue;
        }

        let content = fs::read_to_string(&toml_path)?;
        let raw: RiceToml = match toml::from_str(&content) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("skip {}: {e}", toml_path.display());
                continue;
            }
        };

        let mut screenshots = raw.screenshots;
        if let (Some(rice_dir), Some(base)) = (&rice_dir, base_url) {
            let shots_dir = rice_dir.join("screenshots");
            if shots_dir.is_dir() {
                let mut found: Vec<String> = fs::read_dir(&shots_dir)?
                    .filter_map(|e| e.ok())
                    .filter(|e| {
                        e.path().extension().is_some_and(|ext| {
                            matches!(
                                ext.to_str().unwrap_or("").to_lowercase().as_str(),
                                "png" | "jpg" | "jpeg" | "webp" | "gif"
                            )
                        })
                    })
                    .filter_map(|e| {
                        let rel = e
                            .path()
                            .strip_prefix(dir.parent().unwrap_or(dir))
                            .ok()?
                            .to_str()?
                            .replace('\\', "/");
                        Some(format!("{base}/{rel}"))
                    })
                    .collect();
                found.sort();
                found.append(&mut screenshots);
                screenshots = found;
            }
        }

        rices.push(Rice {
            id: raw.id,
            name: raw.name,
            author: raw.author,
            description: raw.description,
            wm: raw
                .wm
                .parse::<WindowManager>()
                .unwrap_or(WindowManager::Unknown),
            theme: raw.theme,
            fonts: raw.fonts,
            dependencies: raw.dependencies,
            repo_url: raw.repo_url,
            screenshots,
            stars: 0,
            commit_hash: None,
            updated_at: None,
        });
    }

    rices.sort_by(|a, b| b.stars.cmp(&a.stars).then(a.id.cmp(&b.id)));

    let index = Index {
        version: "1".into(),
        updated_at: Utc::now(),
        rices,
    };

    let json = serde_json::to_string_pretty(&index)?;
    fs::write(output, &json)?;
    println!(
        "✓ wrote {} rices to {}",
        index.rices.len(),
        output.display()
    );
    Ok(())
}

fn cmd_update_stars(index_path: &Path, token: Option<&str>) -> anyhow::Result<()> {
    let content = fs::read_to_string(index_path)?;
    let mut index: Index = serde_json::from_str(&content)?;

    for rice in &mut index.rices {
        let stars = fetch_stars(&rice.repo_url, token);
        match stars {
            Ok(s) => {
                println!("  {} ★{}", rice.id, s);
                rice.stars = s;
            }
            Err(e) => eprintln!("  {} skip: {e}", rice.id),
        }
    }

    index.updated_at = Utc::now();
    fs::write(index_path, serde_json::to_string_pretty(&index)?)?;
    println!("✓ star counts updated");
    Ok(())
}

fn fetch_stars(repo_url: &str, token: Option<&str>) -> anyhow::Result<u32> {
    let api_url = repo_url
        .trim_end_matches('/')
        .replace("https://github.com/", "https://api.github.com/repos/");

    let mut cmd = Command::new("curl");
    cmd.args([
        "-sf",
        "--connect-timeout",
        "10",
        "-H",
        "Accept: application/vnd.github+json",
    ]);
    if let Some(t) = token {
        cmd.args(["-H", &format!("Authorization: Bearer {t}")]);
    }
    cmd.arg(&api_url);

    let output = cmd.output()?;
    if !output.status.success() {
        return Err(anyhow::anyhow!("curl failed"));
    }

    let json: serde_json::Value = serde_json::from_slice(&output.stdout)?;
    json["stargazers_count"]
        .as_u64()
        .map(|n| n as u32)
        .ok_or_else(|| anyhow::anyhow!("missing stargazers_count"))
}
