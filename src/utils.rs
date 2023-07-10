use std::path::PathBuf;
use std::{env, fs, io};

pub fn expand_home_dir(path_like: &str) -> String {
    if path_like.starts_with("~/") {
        let home_dir = env::var("HOME").expect("Can not find $HOME env");
        path_like.replacen('~', home_dir.as_str(), 1)
    } else {
        path_like.to_string()
    }
}

pub fn copy(from: &PathBuf, to: &PathBuf) -> io::Result<()> {
    to.parent()
        .map(fs::create_dir_all)
        .expect("Failed to get parent dir")
        .expect("Failed to create parent dir");

    if from.is_file() {
        fs::copy(from, to).unwrap_or_else(|_| panic!("Failed to copy {:?} to {:?}", from, to));
    } else if from.is_dir() {
        from.read_dir()?
            .filter_map(|item| item.ok())
            .for_each(|item| {
                let dest = to.join(item.file_name());
                copy(&item.path(), &dest).unwrap();
            });
    }
    Ok(())
}

pub fn remove(target: &PathBuf) -> io::Result<()> {
    if target.is_file() {
        fs::remove_file(target)
    } else if target.is_dir() {
        fs::remove_dir_all(target)
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::expand_home_dir;

    #[cfg(unix)]
    #[test]
    fn test_expand_home_dir() -> Result<(), Box<dyn std::error::Error>> {
        let origin_home_env = env::var("HOME")?;
        let test_home_env = "dotfiles_test_user";
        let test_path = "~/.zshrc";

        env::set_var("HOME", test_home_env);
        let expanded_path = expand_home_dir(test_path);
        env::set_var("HOME", origin_home_env);

        assert_eq!(
            expanded_path,
            format!("{}/.zshrc", test_home_env),
            "expand_home_dir failed"
        );
        assert_eq!("./zshrc", expand_home_dir("./zshrc"));
        Ok(())
    }
}
