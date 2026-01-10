use crate::core::config::Config;
use crate::core::model::component::FileItem;
use std::collections::{HashSet, VecDeque};
use std::fs;

use std::io;
use std::path::Path;

use color_eyre::{
    Result as Res,
    eyre::{Context, bail},
};

use ignore::WalkBuilder;

#[derive(Debug)]
pub struct FileOperator {
    pub show_hidden: bool,
    pub respect_gitignore: bool,
}

impl Default for FileOperator {
    fn default() -> Self {
        Self {
            show_hidden: false,
            respect_gitignore: true,
        }
    }
}

impl FileOperator {
    pub fn new(config: &Config) -> Self {
        Self {
            show_hidden: config.show_hidden,
            respect_gitignore: config.respect_gitignore,
        }
    }

    /// 根据配置获取目录下的文件列表，并封装为 FileItem
    pub fn list_items(&self, path: &Path) -> Res<VecDeque<FileItem>> {
        let mut res = VecDeque::new();
        let walker = WalkBuilder::new(path)
            .hidden(!self.show_hidden)
            .git_ignore(self.respect_gitignore)
            .max_depth(Some(1))
            .build();

        let mut id = 0;
        for result in walker {
            let entry = result?;
            // WalkBuilder 会包含根目录本身（depth 0），需要跳过
            if entry.depth() == 0 {
                continue;
            }

            let item = FileItem {
                id,
                path: entry.path().to_path_buf(),
                display_name: entry.file_name().to_string_lossy().to_string(),
                is_dir: entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false),
            };
            res.push_back(item);
            id += 1;
        }
        Ok(res)
    }

    /// 获取指定目录下的文件列表
    pub fn get_filtered_files(&self, path: &Path) -> Vec<String> {
        let mut files = Vec::new();
        let walker = WalkBuilder::new(path)
            .hidden(!self.show_hidden)
            .git_ignore(self.respect_gitignore)
            .max_depth(Some(1))
            .build();

        for result in walker {
            match result {
                Ok(entry) => {
                    if entry.depth() == 0 {
                        continue;
                    }
                    if let Some(name) = entry.file_name().to_str() {
                        files.push(name.to_string());
                    }
                }
                Err(err) => tracing::error!("Walk error: {}", err),
            }
        }
        files
    }

    pub fn organize<P: AsRef<Path>>(&self, items: &[P], target_dir_path: &Path) -> Res<()> {
        let dest_dir = target_dir_path;
        if dest_dir.exists() {
            bail!(
                "Destination directory '{}' already exists.",
                dest_dir.display()
            );
        }

        for item_path in items {
            if !item_path.as_ref().exists() {
                bail!(
                    "Source item '{}' does not exist.",
                    item_path.as_ref().display()
                );
            }
        }

        let mut seen_names = HashSet::new();
        for item_path in items {
            let name = item_path.as_ref().file_name().ok_or_else(|| {
                color_eyre::eyre::eyre!("Invalid file name: {:?}", item_path.as_ref())
            })?;
            if !seen_names.insert(name) {
                bail!("Duplicate file name detected in selection: {:?}", name);
            }
        }

        fs::create_dir_all(dest_dir)
            .with_context(|| format!("Failed to create directory '{}'", dest_dir.display()))?;

        for item_path in items {
            let path = item_path.as_ref();
            let file_name = path.file_name().unwrap();
            let dst = dest_dir.join(file_name);

            self.move_item(path, &dst).with_context(|| {
                format!("Failed to move '{}' to '{}'", path.display(), dst.display())
            })?;
        }
        Ok(())
    }

    pub fn copy<P: AsRef<Path>>(&self, items: &[P], target_dir_path: &Path) -> Res<()> {
        if target_dir_path.exists() {
            bail!(
                "Destination directory '{}' already exists.",
                target_dir_path.display()
            );
        }

        for item in items {
            if !item.as_ref().exists() {
                bail!("Source item '{}' does not exist.", item.as_ref().display());
            }
        }

        fs::create_dir_all(target_dir_path).with_context(|| {
            format!("Failed to create directory '{}'", target_dir_path.display())
        })?;

        for item in items {
            let src = item.as_ref();
            let file_name = src.file_name().ok_or_else(|| {
                color_eyre::eyre::eyre!("Could not get file name for '{}'", src.display())
            })?;
            let dst = target_dir_path.join(file_name);

            if src.is_dir() {
                self.copy_dir_all(src, &dst)?;
            } else {
                fs::copy(src, &dst).with_context(|| {
                    format!(
                        "Failed to copy file '{}' to '{}'",
                        src.display(),
                        dst.display()
                    )
                })?;
            }
        }
        Ok(())
    }

    /// 增强版的移动函数，支持跨分区移动
    fn move_item(&self, src: &Path, dst: &Path) -> Res<()> {
        if let Err(e) = fs::rename(src, dst) {
            if e.raw_os_error() == Some(18) || e.kind() == io::ErrorKind::CrossesDevices {
                if src.is_dir() {
                    self.copy_dir_all(src, dst)?;
                    fs::remove_dir_all(src)?;
                } else {
                    fs::copy(src, dst)?;
                    fs::remove_file(src)?;
                }
            } else {
                return Err(e).map_err(|err| err.into());
            }
        }
        Ok(())
    }

    fn copy_dir_all(&self, src: &Path, dst: &Path) -> Res<()> {
        let walker = WalkBuilder::new(src)
            .hidden(!self.show_hidden)
            .git_ignore(self.respect_gitignore)
            .build();

        for result in walker {
            let entry = result?;
            let path = entry.path();
            let rel_path = path.strip_prefix(src)?;
            let target_path = dst.join(rel_path);

            let file_type = entry
                .file_type()
                .ok_or_else(|| color_eyre::eyre::eyre!("Could not get file type for {:?}", path))?;

            if file_type.is_dir() {
                fs::create_dir_all(&target_path)?;
            } else if file_type.is_file() {
                fs::copy(path, &target_path)?;
            }
        }
        Ok(())
    }
}

/// 静态工具函数（不依赖过滤规则）
pub fn delete<P: AsRef<Path>>(items: &[P]) -> Res<()> {
    for item in items {
        let path = item.as_ref();
        if path.is_dir() {
            fs::remove_dir_all(path)?;
        } else {
            fs::remove_file(path)?;
        }
    }
    Ok(())
}

pub fn trash<P: AsRef<Path>>(items: &[P]) -> Res<()> {
    trash::delete_all(items).with_context(|| "Failed to move items to trash")?;
    Ok(())
}
