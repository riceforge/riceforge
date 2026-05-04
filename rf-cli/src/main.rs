use clap::{Parser, Subcommand};
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use rf_core::{
    backup::BackupManager,
    deploy::DeployManager,
    git::GitManager,
    index::IndexManager,
    installed::InstalledManager,
    packages::PackageManager,
    WindowManager,
};
use std::str::FromStr;

#[derive(Parser)]
#[command(
    name = "riceforge",
    about = "Dotfile rice manager for Linux",
    version,
    propagate_version = true
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Sync local index cache from remote registry")]
    Update,

    #[command(about = "Search available rices")]
    Search {
        query: String,
        #[arg(long, short, value_name = "WM", help = "Filter by window manager")]
        wm: Option<String>,
        #[arg(long, short = 't', value_name = "THEME", help = "Filter by theme")]
        theme: Option<String>,
    },

    #[command(about = "List available or installed rices")]
    List {
        #[arg(long, help = "Show only installed rices")]
        installed: bool,
    },

    #[command(about = "Show detailed info for a rice")]
    Info { id: String },

    #[command(about = "Install a rice")]
    Install {
        id: String,
        #[arg(long, help = "Preview changes without applying them")]
        dry_run: bool,
        #[arg(long, help = "Skip package installation")]
        no_packages: bool,
    },

    #[command(about = "Remove an installed rice")]
    Remove {
        id: String,
        #[arg(long, help = "Restore backup after removing")]
        restore: bool,
        #[arg(long, help = "Also delete the cloned repository")]
        purge: bool,
    },

    #[command(about = "Manage config backups", subcommand_required = true)]
    Backup {
        #[command(subcommand)]
        cmd: BackupCmd,
    },
}

#[derive(Subcommand)]
enum BackupCmd {
    #[command(about = "List all backups")]
    List,
    #[command(about = "Restore a backup by ID")]
    Restore { id: String },
    #[command(about = "Delete old backups, keeping the N most recent")]
    Clean {
        #[arg(default_value = "5")]
        keep: usize,
    },
}

fn main() {
    let cli = Cli::parse();
    if let Err(e) = run(cli) {
        eprintln!("{} {e}", "error:".red().bold());
        std::process::exit(1);
    }
}

fn run(cli: Cli) -> rf_core::Result<()> {
    match cli.command {
        Commands::Update => cmd_update(),
        Commands::Search { query, wm, theme } => cmd_search(&query, wm.as_deref(), theme.as_deref()),
        Commands::List { installed } => cmd_list(installed),
        Commands::Info { id } => cmd_info(&id),
        Commands::Install { id, dry_run, no_packages } => cmd_install(&id, dry_run, no_packages),
        Commands::Remove { id, restore, purge } => cmd_remove(&id, restore, purge),
        Commands::Backup { cmd } => match cmd {
            BackupCmd::List => cmd_backup_list(),
            BackupCmd::Restore { id } => cmd_backup_restore(&id),
            BackupCmd::Clean { keep } => cmd_backup_clean(keep),
        },
    }
}

fn cmd_update() -> rf_core::Result<()> {
    let pb = spinner("Fetching index...");
    let index = IndexManager::update()?;
    pb.finish_and_clear();
    println!(
        "{} {} rices indexed (updated {})",
        "✓".green(),
        index.rices.len(),
        index.updated_at.format("%Y-%m-%d")
    );
    Ok(())
}

fn cmd_search(query: &str, wm: Option<&str>, theme: Option<&str>) -> rf_core::Result<()> {
    let index = IndexManager::load_cached()?;
    let wm_filter = wm.and_then(|w| WindowManager::from_str(w).ok());
    let results = IndexManager::search(&index, query, wm_filter.as_ref(), theme);

    if results.is_empty() {
        println!("{}", "No rices found.".dimmed());
        return Ok(());
    }

    for rice in results {
        let installed = InstalledManager::is_installed(&rice.id)?;
        let marker = if installed { "✓ ".green().to_string() } else { "  ".to_string() };
        println!(
            "{}{}  {}  {}  ★{}",
            marker,
            rice.id.bold(),
            rice.name.dimmed(),
            rice.wm.to_string().cyan(),
            rice.stars
        );
    }
    Ok(())
}

fn cmd_list(installed_only: bool) -> rf_core::Result<()> {
    if installed_only {
        let entries = InstalledManager::list()?;
        if entries.is_empty() {
            println!("{}", "No rices installed.".dimmed());
            return Ok(());
        }
        for e in entries {
            println!(
                "{} {} {}",
                "✓".green(),
                e.rice_id.bold(),
                e.commit_hash[..8].dimmed()
            );
        }
    } else {
        let index = IndexManager::load_cached()?;
        for rice in &index.rices {
            let installed = InstalledManager::is_installed(&rice.id)?;
            let marker = if installed { "✓".green() } else { " ".normal() };
            println!(
                "{} {}  {}  ★{}",
                marker,
                rice.id.bold(),
                rice.wm.to_string().cyan(),
                rice.stars
            );
        }
    }
    Ok(())
}

fn cmd_info(id: &str) -> rf_core::Result<()> {
    let index = IndexManager::load_cached()?;
    let rice = IndexManager::find(&index, id)
        .ok_or_else(|| rf_core::RiceForgeError::NotFound(id.to_string()))?;

    let installed = InstalledManager::is_installed(id)?;

    println!("{}", rice.name.bold());
    println!("  id          {}", rice.id.dimmed());
    println!("  author      {}", rice.author);
    println!("  wm          {}", rice.wm.to_string().cyan());
    println!("  theme       {}", rice.theme);
    println!("  stars       {}", rice.stars);
    println!("  repo        {}", rice.repo_url.underline());
    println!("  status      {}", if installed { "installed".green().to_string() } else { "not installed".dimmed().to_string() });
    println!();
    println!("  {}", rice.description);
    if !rice.dependencies.is_empty() {
        println!();
        println!("  dependencies");
        for dep in &rice.dependencies {
            let ok = PackageManager::is_installed(dep);
            let mark = if ok { "✓".green() } else { "✗".red() };
            println!("    {mark} {dep}");
        }
    }
    if !rice.fonts.is_empty() {
        println!();
        println!("  fonts  {}", rice.fonts.join(", ").dimmed());
    }
    Ok(())
}

fn cmd_install(id: &str, dry_run: bool, no_packages: bool) -> rf_core::Result<()> {
    let index = IndexManager::load_cached()?;
    let rice = IndexManager::find(&index, id)
        .ok_or_else(|| rf_core::RiceForgeError::NotFound(id.to_string()))?;

    if !dry_run && InstalledManager::is_installed(id)? {
        return Err(rf_core::RiceForgeError::AlreadyInstalled(id.to_string()));
    }

    let pb = spinner(&format!("Cloning {}...", rice.id));
    let commit = GitManager::clone_or_pull(&rice)?;
    pb.finish_and_clear();
    println!("{} cloned  {}", "✓".green(), commit[..8].dimmed());

    let plan = DeployManager::plan(&rice)?;

    println!("\n{}", "deploy plan:".bold());
    for (src, dest) in &plan.links {
        println!("  {} → {}", src.display().to_string().dimmed(), dest.display());
    }

    if !plan.to_backup.is_empty() {
        println!("\n{}", "will back up:".yellow());
        for f in &plan.to_backup {
            println!("  {}", f.display().to_string().yellow());
        }
    }

    if !rice.dependencies.is_empty() && !no_packages {
        let missing = PackageManager::missing(&rice.dependencies);
        if !missing.is_empty() {
            println!("\n{}", "packages to install:".bold());
            for pkg in &missing {
                println!("  {pkg}");
            }
        }
    }

    if dry_run {
        println!("\n{} dry run — no changes applied", "→".cyan());
        return Ok(());
    }

    let backup_id = if !plan.to_backup.is_empty() {
        let entry = BackupManager::create(Some(id), &plan.to_backup)?;
        println!("{} backup created  {}", "✓".green(), entry.id.dimmed());
        Some(entry.id)
    } else {
        None
    };

    if !rice.dependencies.is_empty() && !no_packages {
        let missing = PackageManager::missing(&rice.dependencies);
        if !missing.is_empty() {
            let pb = spinner("Installing packages...");
            PackageManager::install(&missing)?;
            pb.finish_and_clear();
            println!("{} packages installed", "✓".green());
        }
    }

    DeployManager::apply(&plan)?;
    InstalledManager::add(id, &commit, backup_id)?;

    println!("{} {} installed", "✓".green(), rice.name.bold());
    Ok(())
}

fn cmd_remove(id: &str, restore: bool, purge: bool) -> rf_core::Result<()> {
    let index = IndexManager::load_cached()?;
    let rice = IndexManager::find(&index, id)
        .ok_or_else(|| rf_core::RiceForgeError::NotFound(id.to_string()))?;

    let entry = InstalledManager::get(id)?;
    let removed = DeployManager::remove(&rice)?;

    println!("{} removed {} symlinks", "✓".green(), removed.len());

    if restore {
        if let Some(bid) = &entry.backup_id {
            BackupManager::restore(bid)?;
            println!("{} backup {} restored", "✓".green(), bid.dimmed());
        } else {
            println!("{} no backup found for this rice", "!".yellow());
        }
    }

    InstalledManager::remove(id)?;

    if purge {
        GitManager::remove(id)?;
        println!("{} repository removed", "✓".green());
    }

    println!("{} {} removed", "✓".green(), id.bold());
    Ok(())
}

fn cmd_backup_list() -> rf_core::Result<()> {
    let entries = BackupManager::list()?;
    if entries.is_empty() {
        println!("{}", "No backups found.".dimmed());
        return Ok(());
    }
    for e in entries {
        let rice = e.rice_id.as_deref().unwrap_or("manual");
        println!(
            "{}  {}  {} files",
            e.id.bold(),
            rice.dimmed(),
            e.files.len()
        );
    }
    Ok(())
}

fn cmd_backup_restore(id: &str) -> rf_core::Result<()> {
    BackupManager::restore(id)?;
    println!("{} backup {} restored", "✓".green(), id.bold());
    Ok(())
}

fn cmd_backup_clean(keep: usize) -> rf_core::Result<()> {
    let removed = BackupManager::clean(keep)?;
    if removed.is_empty() {
        println!("{}", "Nothing to clean.".dimmed());
    } else {
        for id in &removed {
            println!("{} removed {}", "✓".green(), id.dimmed());
        }
        println!("Removed {} backup(s)", removed.len());
    }
    Ok(())
}

fn spinner(msg: &str) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner} {msg}")
            .unwrap(),
    );
    pb.set_message(msg.to_string());
    pb.enable_steady_tick(std::time::Duration::from_millis(80));
    pb
}
