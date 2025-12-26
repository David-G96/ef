use std::fs;

use std::io;
use std::path::Path;

use color_eyre::{
    Result as Res,
    eyre::{Context, bail},
};

// 这是一个纯函数，容易测试，与 UI 无关
pub fn read_file_content(path: &Path) -> io::Result<String> {
    fs::read_to_string(path)
}

pub fn save_file_content(path: &Path, content: &str) -> io::Result<()> {
    fs::write(path, content)
}

pub enum FileOpError {
    FileAlreadyExists(),
    DirAlreadyExists(),
}

pub fn organize<P: AsRef<Path>>(items: &[P], target_path: &Path, dir_name: &str) -> Res<()> {
    // 1. 验证目标路径是否存在且为目录
    if !target_path.is_dir() {
        bail!(
            "Target path '{}' is not a valid directory.",
            target_path.display()
        );
    }

    // 2. 构建最终的目标目录路径
    let dest_dir = target_path.join(dir_name);

    // 3. 验证目标目录是否已存在，避免覆盖
    if dest_dir.exists() {
        bail!(
            "Destination directory '{}' already exists.",
            dest_dir.display()
        );
    }

    // 4. 验证所有待移动项是否存在
    for item_path in items {
        if !item_path.as_ref().exists() {
            bail!(
                "Source item '{}' does not exist.",
                item_path.as_ref().display()
            );
        }
    }

    // 5. 创建目录
    fs::create_dir(&dest_dir)
        .with_context(|| format!("Failed to create directory '{}'", dest_dir.display()))?;

    // 6. 移动所有文件
    for item_path in items {
        let path = item_path.as_ref();
        let file_name = path.file_name().ok_or_else(|| {
            color_eyre::eyre::eyre!("Could not get file name for '{}'", path.display())
        })?;
        fs::rename(path, dest_dir.join(file_name))
            .with_context(|| format!("Failed to move item '{}'", path.display()))?;
    }

    Ok(())
}

pub fn do_nothing<P: AsRef<Path>>(items: &[P]) {
    //do nothing
}
pub fn delete() {}
