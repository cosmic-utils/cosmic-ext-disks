// SPDX-License-Identifier: GPL-3.0-only

use std::path::Path;

use super::types::Category;

pub fn classify_path(path: &Path) -> Category {
    let Some(ext) = path.extension().and_then(|ext| ext.to_str()) else {
        return Category::Other;
    };

    let ext = ext.to_ascii_lowercase();

    match ext.as_str() {
        "txt" | "pdf" | "doc" | "docx" | "odt" | "rtf" | "md" | "markdown" | "epub"
        | "xls" | "xlsx" | "ods" | "ppt" | "pptx" | "odp" => Category::Documents,
        "jpg" | "jpeg" | "png" | "gif" | "bmp" | "webp" | "svg" | "tif" | "tiff"
        | "heic" | "avif" => Category::Images,
        "mp3" | "wav" | "flac" | "aac" | "ogg" | "m4a" | "opus" => Category::Audio,
        "mp4" | "mkv" | "webm" | "mov" | "avi" | "flv" | "m4v" => Category::Video,
        "zip" | "tar" | "gz" | "bz2" | "xz" | "7z" | "rar" | "zst" => Category::Archives,
        "rs" | "c" | "h" | "cpp" | "hpp" | "cc" | "py" | "js" | "ts" | "tsx" | "jsx"
        | "java" | "kt" | "go" | "rb" | "php" | "swift" | "cs" | "toml" | "yaml"
        | "yml" | "json" | "xml" | "sh" | "bash" | "zsh" | "fish" | "sql" => Category::Code,
        "bin" | "so" | "dll" | "exe" | "appimage" | "deb" | "rpm" | "apk" | "msi" => {
            Category::Binaries
        }
        _ => Category::Other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_by_extension_and_falls_back_to_other() {
        assert_eq!(classify_path(Path::new("/tmp/file.rs")), Category::Code);
        assert_eq!(classify_path(Path::new("/tmp/pic.JPG")), Category::Images);
        assert_eq!(classify_path(Path::new("/tmp/noext")), Category::Other);
    }
}
