use std::fs;

use std::io;
use std::path::Path;


// 这是一个纯函数，容易测试，与 UI 无关
pub fn read_file_content(path: &Path) -> io::Result<String> {
    fs::read_to_string(path)
}

pub fn save_file_content(path: &Path, content: &str) -> io::Result<()> {
    fs::write(path, content)
}
