#[cfg(unix)]
mod unix {
    use std::{
        fs,
        path::PathBuf,
        sync::{Mutex, OnceLock},
    };
    use tempfile::TempDir;

    static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

    fn env_lock() -> &'static Mutex<()> {
        ENV_LOCK.get_or_init(|| Mutex::new(()))
    }

    fn setup_xdg(tmp: &TempDir) {
        let data = tmp.path().join("data");
        let cache = tmp.path().join("cache");
        fs::create_dir_all(&data).unwrap();
        fs::create_dir_all(&cache).unwrap();
        std::env::set_var("XDG_DATA_HOME", &data);
        std::env::set_var("XDG_CACHE_HOME", &cache);
    }

    fn make_rice(id: &str) -> rf_core::Rice {
        use rf_core::{Rice, WindowManager};
        Rice {
            id: id.to_string(),
            name: "Test Rice".to_string(),
            author: "tester".to_string(),
            description: "desc".to_string(),
            wm: WindowManager::Hyprland,
            theme: "dark".to_string(),
            fonts: vec![],
            dependencies: vec![],
            repo_url: "https://example.com".to_string(),
            screenshots: vec![],
            stars: 0,
            commit_hash: Some("deadbeef".to_string()),
            updated_at: None,
        }
    }

    #[test]
    fn installed_manager_add_list_remove() {
        let _guard = env_lock().lock().unwrap();
        let tmp = TempDir::new().unwrap();
        setup_xdg(&tmp);

        use rf_core::installed::InstalledManager;

        assert!(!InstalledManager::is_installed("rice-a").unwrap());

        InstalledManager::add("rice-a", "hash1", None).unwrap();
        assert!(InstalledManager::is_installed("rice-a").unwrap());

        let list = InstalledManager::list().unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].rice_id, "rice-a");
        assert_eq!(list[0].commit_hash, "hash1");

        InstalledManager::add("rice-b", "hash2", Some("backup-1".into())).unwrap();
        let list = InstalledManager::list().unwrap();
        assert_eq!(list.len(), 2);

        let entry = InstalledManager::get("rice-b").unwrap();
        assert_eq!(entry.backup_id.as_deref(), Some("backup-1"));

        InstalledManager::remove("rice-a").unwrap();
        assert!(!InstalledManager::is_installed("rice-a").unwrap());
        assert!(InstalledManager::is_installed("rice-b").unwrap());
    }

    #[test]
    fn installed_manager_add_overwrites_existing() {
        let _guard = env_lock().lock().unwrap();
        let tmp = TempDir::new().unwrap();
        setup_xdg(&tmp);

        use rf_core::installed::InstalledManager;

        InstalledManager::add("rice-x", "hash-old", None).unwrap();
        InstalledManager::add("rice-x", "hash-new", None).unwrap();

        let list = InstalledManager::list().unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].commit_hash, "hash-new");
    }

    #[test]
    fn installed_manager_remove_nonexistent_returns_none() {
        let _guard = env_lock().lock().unwrap();
        let tmp = TempDir::new().unwrap();
        setup_xdg(&tmp);

        use rf_core::installed::InstalledManager;

        let result = InstalledManager::remove("no-such-rice").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn backup_create_and_list() {
        let _guard = env_lock().lock().unwrap();
        let tmp = TempDir::new().unwrap();
        setup_xdg(&tmp);

        use rf_core::backup::BackupManager;

        let fake_home = tmp.path().join("home");
        let config_dir = fake_home.join(".config").join("alacritty");
        fs::create_dir_all(&config_dir).unwrap();
        let file = config_dir.join("alacritty.toml");
        fs::write(&file, b"[colors]").unwrap();

        std::env::set_var("HOME", &fake_home);

        let entry = BackupManager::create(Some("rice-a"), &[file]).unwrap();
        assert_eq!(entry.rice_id.as_deref(), Some("rice-a"));
        assert_eq!(entry.files.len(), 1);

        let backups = BackupManager::list().unwrap();
        assert_eq!(backups.len(), 1);
        assert_eq!(backups[0].id, entry.id);
    }

    #[test]
    fn backup_clean_removes_old_entries() {
        let _guard = env_lock().lock().unwrap();
        let tmp = TempDir::new().unwrap();
        setup_xdg(&tmp);

        use rf_core::backup::BackupManager;

        let fake_home = tmp.path().join("home");
        std::env::set_var("HOME", &fake_home);

        BackupManager::create(None, &[]).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(1100));
        BackupManager::create(None, &[]).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(1100));
        BackupManager::create(None, &[]).unwrap();

        let removed = BackupManager::clean(1).unwrap();
        assert_eq!(removed.len(), 2);

        let remaining = BackupManager::list().unwrap();
        assert_eq!(remaining.len(), 1);
    }

    #[test]
    fn deploy_plan_excludes_root_files() {
        let _guard = env_lock().lock().unwrap();
        let tmp = TempDir::new().unwrap();
        setup_xdg(&tmp);

        use rf_core::{config::Paths, deploy::DeployManager};

        let rice = make_rice("plan-rice");
        let rice_dir = Paths::rices_dir().join(&rice.id);
        fs::create_dir_all(&rice_dir).unwrap();

        fs::write(rice_dir.join("rice.toml"), b"[rice]").unwrap();
        fs::write(rice_dir.join("README.md"), b"# readme").unwrap();
        fs::write(rice_dir.join(".gitignore"), b"target/").unwrap();

        let config_dir = rice_dir.join(".config").join("hypr");
        fs::create_dir_all(&config_dir).unwrap();
        fs::write(config_dir.join("hyprland.conf"), b"monitor=,auto").unwrap();

        let fake_home = tmp.path().join("home");
        fs::create_dir_all(&fake_home).unwrap();
        std::env::set_var("HOME", &fake_home);

        let plan = DeployManager::plan(&rice).unwrap();

        let link_names: Vec<PathBuf> = plan
            .links
            .iter()
            .map(|(src, _)| src.file_name().unwrap().into())
            .collect();

        assert!(link_names.iter().any(|n| n == "hyprland.conf"));
        assert!(!link_names.iter().any(|n| n == "rice.toml"));
        assert!(!link_names.iter().any(|n| n == "README.md"));
        assert!(!link_names.iter().any(|n| n == ".gitignore"));
    }

    #[test]
    fn deploy_apply_creates_symlinks() {
        let _guard = env_lock().lock().unwrap();
        let tmp = TempDir::new().unwrap();
        setup_xdg(&tmp);

        use rf_core::{config::Paths, deploy::DeployManager};

        let rice = make_rice("apply-rice");
        let rice_dir = Paths::rices_dir().join(&rice.id);
        let config_dir = rice_dir.join(".config").join("kitty");
        fs::create_dir_all(&config_dir).unwrap();
        fs::write(config_dir.join("kitty.conf"), b"font_size 13").unwrap();

        let fake_home = tmp.path().join("home");
        fs::create_dir_all(&fake_home).unwrap();
        std::env::set_var("HOME", &fake_home);

        let plan = DeployManager::plan(&rice).unwrap();
        assert_eq!(plan.links.len(), 1);

        DeployManager::apply(&plan).unwrap();

        let dest = &plan.links[0].1;
        assert!(dest.is_symlink());
        assert_eq!(fs::read_link(dest).unwrap(), plan.links[0].0);
    }

    #[test]
    fn deploy_remove_deletes_symlinks() {
        let _guard = env_lock().lock().unwrap();
        let tmp = TempDir::new().unwrap();
        setup_xdg(&tmp);

        use rf_core::{config::Paths, deploy::DeployManager};

        let rice = make_rice("remove-rice");
        let rice_dir = Paths::rices_dir().join(&rice.id);
        let config_dir = rice_dir.join(".config").join("wm");
        fs::create_dir_all(&config_dir).unwrap();
        fs::write(config_dir.join("config"), b"wm config").unwrap();

        let fake_home = tmp.path().join("home");
        fs::create_dir_all(&fake_home).unwrap();
        std::env::set_var("HOME", &fake_home);

        let plan = DeployManager::plan(&rice).unwrap();
        DeployManager::apply(&plan).unwrap();

        let dest = plan.links[0].1.clone();
        assert!(dest.is_symlink());

        let removed = DeployManager::remove(&rice).unwrap();
        assert_eq!(removed.len(), 1);
        assert!(!dest.exists());
    }
}
