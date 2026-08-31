//! IDE-specific project metadata used to improve window-title matching.

use std::path::Path;

/// Read the IntelliJ-family project name for `project_dir`, if it has one.
///
/// JetBrains IDEs title each project frame with the project *name*, not the
/// folder name. When the two differ the name is stored in `.idea/.name`
/// (written on Gradle/Maven import, among others), so a folder `bmp_api` can
/// show up as `api – Foo.java [api]` in the window title. Matching on the
/// folder name alone then scores zero for every frame, and since all frames of
/// one IDE share a single process there is nothing left to disambiguate them.
///
/// Returns `None` when the file is missing, unreadable, or blank — the common
/// case, since IDEs omit it while name and folder agree.
pub fn read_ide_project_name(project_dir: &str) -> Option<String> {
    let raw = std::fs::read_to_string(Path::new(project_dir).join(".idea").join(".name")).ok()?;
    let name = raw.trim();
    if name.is_empty() {
        None
    } else {
        Some(name.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    /// Create a scratch dir under the OS temp dir. Uses a caller-supplied unique
    /// tag rather than env vars, which `dirs::home_dir()`-style lookups ignore.
    fn scratch(tag: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("agent-toast-ide-test-{}", tag));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn write_name(dir: &Path, contents: &str) {
        let idea = dir.join(".idea");
        fs::create_dir_all(&idea).unwrap();
        fs::write(idea.join(".name"), contents).unwrap();
    }

    #[test]
    fn reads_project_name() {
        let dir = scratch("reads");
        write_name(&dir, "api");
        assert_eq!(
            read_ide_project_name(dir.to_str().unwrap()).as_deref(),
            Some("api")
        );
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn trims_trailing_newline() {
        // IntelliJ writes the name with a trailing newline.
        let dir = scratch("trims");
        write_name(&dir, "kt-bom-cms-cug\n");
        assert_eq!(
            read_ide_project_name(dir.to_str().unwrap()).as_deref(),
            Some("kt-bom-cms-cug")
        );
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn none_when_file_missing() {
        let dir = scratch("missing");
        assert_eq!(read_ide_project_name(dir.to_str().unwrap()), None);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn none_when_blank() {
        let dir = scratch("blank");
        write_name(&dir, "   \n");
        assert_eq!(read_ide_project_name(dir.to_str().unwrap()), None);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn none_when_dir_missing() {
        let missing = std::env::temp_dir().join("agent-toast-ide-test-nonexistent-dir");
        let _ = fs::remove_dir_all(&missing);
        assert_eq!(read_ide_project_name(missing.to_str().unwrap()), None);
    }

    #[test]
    fn unicode_project_name() {
        let dir = scratch("unicode");
        write_name(&dir, "프로젝트-이름\n");
        assert_eq!(
            read_ide_project_name(dir.to_str().unwrap()).as_deref(),
            Some("프로젝트-이름")
        );
        let _ = fs::remove_dir_all(&dir);
    }
}
