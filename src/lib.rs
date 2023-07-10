use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::{env, fs, io, os};

mod utils;

const CONFIG_FILE_NAME: &str = "dotfiles.config.json";
const DOTFILES_ENV_NAME: &str = "DOTFILES";

#[derive(Debug)]
struct Link {
    src_path: PathBuf,
    backup_path: PathBuf,
}

impl Link {
    pub fn new(backup_path: &str, src_path: &str, prefix: &Path) -> Self {
        let src_path = utils::expand_home_dir(src_path);
        let src_path = PathBuf::from(src_path);
        let backup_path = prefix.join(backup_path);
        Self {
            backup_path,
            src_path,
        }
    }

    pub fn collect(&self) -> Result<(), io::Error> {
        let src_path = &self.src_path;
        let backup_path = &self.backup_path;
        if !src_path.exists() {
            println!("skip: src_path {:?} not exists", src_path);
            Ok(())
        } else if src_path.is_symlink() {
            println!("skip: src_path {:?} already a link", src_path);
            Ok(())
        } else {
            println!("collecting {:?}", src_path);
            backup_path
                .parent()
                .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Failed to get parent dir"))
                .and_then(|_| utils::copy(src_path, backup_path))
                .and_then(|_| utils::remove(src_path))
                .and_then(|_| os::unix::fs::symlink(backup_path, src_path))
        }
    }

    pub fn restore(&self, force: bool) -> io::Result<()> {
        let src_path = &self.src_path;
        let backup_path = &self.backup_path;

        if !backup_path.exists() {
            println!("skip: backup_path {:?} not exists", backup_path);
            Ok(())
        } else if !src_path.is_symlink() {
            if !force {
                println!("skip: src_path {:?} is not symlink, use --force to overwrite", src_path);
                Ok(())
            } else {
                dbg!(&force);
                utils::remove(src_path).and_then(|_| utils::copy(backup_path, src_path))
            }
        } else {
            println!("restoring {:?}", src_path);
            fs::read_link(src_path)
                .and_then(|target| {
                    if &target == backup_path {
                        Ok(())
                    } else {
                        Err(io::Error::new(
                            io::ErrorKind::Other,
                            format!(
                                "src_path {:?} is not a link to backup_path {:?}",
                                src_path, backup_path
                            ),
                        ))
                    }
                })
                .and_then(|_| utils::remove(src_path))
                .and_then(|_| utils::copy(backup_path, src_path))
        }
    }
}

#[derive(Debug)]
pub struct Dotfiles {
    links: Vec<Link>,
}

impl Dotfiles {
    pub fn new() -> Self {
        env::current_dir()
            .ok()
            .map(Dotfiles::get_config_path)
            .map(Dotfiles::create_config_file_if_not_exists)
            .map(Dotfiles::read_config_file)
            .expect("Failed to create config")
    }

    pub fn read_config() -> Dotfiles {
        Dotfiles::get_config_file()
            .map(Dotfiles::read_config_file)
            .expect("Failed to read config")
    }

    pub fn collect() -> io::Result<()> {
        Dotfiles::read_config()
            .links
            .iter()
            .try_for_each(|link| link.collect())
    }

    pub fn restore(force: bool) -> io::Result<()> {
        Dotfiles::read_config()
            .links
            .iter()
            .try_for_each(|link| link.restore(force))
    }

    fn root_dir() -> Option<PathBuf> {
        env::var(DOTFILES_ENV_NAME).map(PathBuf::from).ok()
    }

    // get config file path by parent dir
    fn get_config_path(mut parent_dir: PathBuf) -> PathBuf {
        parent_dir.push(CONFIG_FILE_NAME);
        parent_dir
    }

    // get config file path by root dir or current dir
    fn get_config_file() -> Option<PathBuf> {
        Dotfiles::root_dir()
            .or_else(|| env::current_dir().ok())
            .map(Dotfiles::get_config_path)
            .filter(|path| path.exists())
    }

    // create config file if not exists
    fn create_config_file_if_not_exists(config_path: PathBuf) -> PathBuf {
        if !config_path.exists() {
            fs::write(&config_path, "{}")
                .unwrap_or_else(|_| panic!("Failed writing {config_path:?}"));
        }
        config_path
    }

    /// read config file, return None if not exists
    fn read_config_file(config_path: PathBuf) -> Dotfiles {
        if !config_path.exists() {
            panic!("Config file {config_path:?} not exists");
        }

        let config: HashMap<String, HashMap<String, String>> = fs::read_to_string(&config_path)
            .ok()
            .and_then(|content| serde_json::from_str(&content).ok())
            .unwrap_or_else(|| panic!("Failed to parse config file {config_path:?}"));

        let parent_dir = config_path.parent().expect("Failed to get parent dir");

        let links: Vec<Link> = config
            .iter()
            .flat_map(|(scope, hash_map)| {
                let prefix = PathBuf::from(parent_dir).join(scope);
                hash_map
                    .iter()
                    .map(move |(backup_path, src_path)| Link::new(backup_path, src_path, &prefix))
            })
            .collect();

        Dotfiles::check_health(&links);

        Dotfiles { links }
    }

    fn check_health(links: &[Link]) {
        let mut links: Vec<String> = links
            .iter()
            .flat_map(|link| vec![link.src_path.clone(), link.backup_path.clone()])
            .map(|link| link.to_str().unwrap().to_string())
            .collect();

        links.sort_by_key(|a| a.len());

        for i in 0..links.len() {
            for j in i + 1..links.len() {
                if links[j].starts_with(&links[i]) {
                    panic!("Conflict files: {} and {}", &links[i], &links[j])
                }
            }
        }
    }
}

impl Default for Dotfiles {
    fn default() -> Self {
        Dotfiles::new()
    }
}
