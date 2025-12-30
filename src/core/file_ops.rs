use std::collections::HashSet;
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

pub fn organize<P: AsRef<Path>>(items: &[P], target_dir_path: &Path) -> Res<()> {
    // 构建最终的目标目录路径
    // let dest_dir = target_dir_path.join(dir_name);
    let dest_dir = target_dir_path;

    // 验证目标目录是否已存在，避免覆盖
    if dest_dir.exists() {
        bail!(
            "Destination directory '{}' already exists.",
            dest_dir.display()
        );
    }

    // 验证所有待移动项是否存在
    for item_path in items {
        if !item_path.as_ref().exists() {
            bail!(
                "Source item '{}' does not exist.",
                item_path.as_ref().display()
            );
        }
    }

    // 检查内部名称冲突：防止 items 中有同名文件导致互相覆盖
    let mut seen_names = HashSet::new();
    for item_path in items {
        let name = item_path.as_ref().file_name().ok_or_else(|| {
            color_eyre::eyre::eyre!("Invalid file name: {:?}", item_path.as_ref())
        })?;
        if !seen_names.insert(name) {
            bail!("Duplicate file name detected in selection: {:?}", name);
        }
    }

    // 使用 create_dir_all 允许创建多级嵌套目录
    fs::create_dir_all(dest_dir)
        .with_context(|| format!("Failed to create directory '{}'", dest_dir.display()))?;

    // 移动所有文件
    for item_path in items {
        let path = item_path.as_ref();
        let file_name = path.file_name().unwrap(); // 前面已经校验过
        let dst = dest_dir.join(file_name);

        // 使用自定义的 move_item 处理跨分区移动
        move_item(path, &dst).with_context(|| {
            format!("Failed to move '{}' to '{}'", path.display(), dst.display())
        })?;
    }

    Ok(())
}

pub fn delete<P: AsRef<Path>>(items: &[P]) -> Res<()> {
    for item in items {
        let path = item.as_ref();
        if path.is_dir() {
            fs::remove_dir_all(path)
                .with_context(|| format!("Failed to delete directory: {:?}", path))?;
        } else {
            fs::remove_file(path).with_context(|| format!("Failed to delete file: {:?}", path))?;
        }
    }
    Ok(())
}

pub fn trash<P: AsRef<Path>>(items: &[P]) -> Res<()> {
    trash::delete_all(items).with_context(|| "Failed to move items to trash")?;
    Ok(())
}

pub fn copy<P: AsRef<Path>>(items: &[P], target_dir_path: &Path) -> Res<()> {
    // 验证目标目录是否已存在，遵循与 organize 相同的逻辑
    if target_dir_path.exists() {
        bail!(
            "Destination directory '{}' already exists.",
            target_dir_path.display()
        );
    }

    // 验证所有待复制项是否存在
    for item in items {
        if !item.as_ref().exists() {
            bail!("Source item '{}' does not exist.", item.as_ref().display());
        }
    }

    // 创建目标目录
    fs::create_dir_all(target_dir_path)
        .with_context(|| format!("Failed to create directory '{}'", target_dir_path.display()))?;

    for item in items {
        let src = item.as_ref();
        let file_name = src.file_name().ok_or_else(|| {
            color_eyre::eyre::eyre!("Could not get file name for '{}'", src.display())
        })?;
        let dst = target_dir_path.join(file_name);

        if src.is_dir() {
            copy_dir_all(src, &dst)?;
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
fn move_item(src: &Path, dst: &Path) -> Res<()> {
    if let Err(e) = fs::rename(src, dst) {
        // 如果是跨设备移动错误 (EXDEV)，回退到 复制+删除 模式
        if e.raw_os_error() == Some(18) || e.kind() == io::ErrorKind::CrossesDevices {
            if src.is_dir() {
                copy_dir_all(src, dst)?;
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

fn copy_dir_all(src: &Path, dst: &Path) -> Res<()> {
    // 检查是否是符号链接，避免死循环
    let metadata = fs::symlink_metadata(src)?;
    if metadata.is_symlink() {
        // 对于符号链接，我们通常只复制链接本身，或者跳过
        // 这里简单处理：跳过符号链接以防死循环
        return Ok(());
    }

    fs::create_dir_all(dst).with_context(|| format!("Failed to create dir: {:?}", dst))?;

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        let dest_path = dst.join(entry.file_name());

        if path.is_dir() {
            copy_dir_all(&path, &dest_path)?;
        } else {
            fs::copy(&path, &dest_path)?;
        }
    }
    Ok(())
}
