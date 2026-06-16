//! Discovering images from a file or directory path.

use std::path::{Path, PathBuf};

use walkdir::WalkDir;

/// Extensions we are able to decode.
const SUPPORTED: &[&str] = &["png", "jpg", "jpeg", "bmp", "webp", "gif"];

/// Whether `path` looks like an image we can open, based on its extension.
pub fn is_image(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(str::to_ascii_lowercase)
        .is_some_and(|ext| SUPPORTED.contains(&ext.as_str()))
}

/// Build the playlist for a given path.
///
/// * A file → that file plus its sibling images, so navigation works.
/// * A directory → every image it contains (recursively).
///
/// Results are sorted by path for a stable, predictable order.
pub fn load_from_path(path: &str) -> Vec<PathBuf> {
    let p = Path::new(path);

    let root = if p.is_file() {
        p.parent().unwrap_or(p).to_path_buf()
    } else {
        p.to_path_buf()
    };

    let mut images: Vec<PathBuf> = WalkDir::new(&root)
        .into_iter()
        .filter_map(Result::ok)
        .map(|e| e.into_path())
        .filter(|p| is_image(p))
        .collect();

    images.sort();

    // If a single file was requested but somehow filtered out, keep it.
    if p.is_file() && !images.iter().any(|i| i == p) {
        images.insert(0, p.to_path_buf());
    }

    images
}

/// Index of `target` within `images`, defaulting to `0`.
pub fn index_of(images: &[PathBuf], target: &Path) -> usize {
    images.iter().position(|p| p == target).unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognises_supported_extensions() {
        assert!(is_image(Path::new("photo.png")));
        assert!(is_image(Path::new("PHOTO.JPG")));
        assert!(is_image(Path::new("a/b/c.webp")));
    }

    #[test]
    fn rejects_unsupported_and_extensionless() {
        assert!(!is_image(Path::new("notes.txt")));
        assert!(!is_image(Path::new("archive.tar.gz")));
        assert!(!is_image(Path::new("README")));
    }

    #[test]
    fn index_of_defaults_to_zero_when_missing() {
        let images = vec![PathBuf::from("a.png"), PathBuf::from("b.png")];
        assert_eq!(index_of(&images, Path::new("b.png")), 1);
        assert_eq!(index_of(&images, Path::new("z.png")), 0);
    }
}
