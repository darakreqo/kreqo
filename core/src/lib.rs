use std::path::PathBuf;
use std::sync::LazyLock;

use directories::ProjectDirs;

pub mod database;
pub mod errors;
pub mod users;

pub static PROJECT_DIRS: LazyLock<Option<ProjectDirs>> =
    LazyLock::new(|| ProjectDirs::from("org", "kreqo", "kreqo-learn"));

pub fn cookies_path() -> PathBuf {
    let project_dirs = PROJECT_DIRS.clone().unwrap();
    project_dirs.cache_dir().with_file_name("cookies.json")
}

pub trait ExternMethod
where
    Self: Sized,
{
    fn apply<F>(self, method: F) -> Self
    where
        F: Fn(Self) -> Self,
    {
        method(self)
    }
    fn apply_with<F, O>(self, method: F, options: O) -> Self
    where
        F: Fn(Self, O) -> Self,
    {
        method(self, options)
    }
}

impl<T> ExternMethod for T {}
