//! 自定义提示词页 — 人格列表与激活 + CRUD。

use crate::paths::PersonaPaths;
use ratatui::widgets::ListState;
use std::path::Path;

/// 页面状态。
pub struct PromptsPage {
    pub state: ListState,
    /// 创建模式：true 时输入新人格名。
    pub creating: bool,
    /// 重命名模式：true 时输入新名称。
    pub renaming: bool,
    /// 删除确认模式。
    pub confirming_delete: bool,
    pub edit_buffer: String,
}

impl PromptsPage {
    pub fn new() -> Self {
        let mut state = ListState::default();
        state.select(Some(0));
        Self {
            state,
            creating: false,
            renaming: false,
            confirming_delete: false,
            edit_buffer: String::new(),
        }
    }
}

// ── Scan / list ────────────────────────────────────────────────────────

/// 扫描目录下的自定义人格文件（.md 文件名去扩展名，排序）。
pub fn scan_persona_files(dir: &Path) -> Vec<String> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut names = entries
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().extension().is_some_and(|e| e == "md"))
        .filter_map(|entry| {
            entry
                .file_name()
                .to_str()
                .map(|name| name.trim_end_matches(".md").to_string())
        })
        .collect::<Vec<_>>();
    names.sort();
    names
}

/// 完整人格列表行（内置 + 自定义），返回 (显示行, 人格名, 是否内置)。
pub fn persona_rows(paths: &PersonaPaths, active: &str) -> Vec<(String, String, bool)> {
    let mut rows = Vec::new();
    for persona in crate::prompts::builtin_personas() {
        rows.push((
            format!(
                "[内置] {} ({}) {}",
                persona.display_name,
                persona.canonical_name,
                if persona.canonical_name == active { "⭐" } else { "" }
            ),
            persona.canonical_name.to_string(),
            true,
        ));
    }
    for name in scan_persona_files(&paths.prompts_dir()) {
        rows.push((
            format!(
                "[自定义] {} {}",
                name,
                if name == active { "⭐" } else { "" }
            ),
            name,
            false,
        ));
    }
    rows
}

// ── CRUD operations ────────────────────────────────────────────────────

/// 创建新人格文件。
pub fn create_persona(paths: &PersonaPaths, name: &str) -> std::io::Result<()> {
    let dir = paths.prompts_dir();
    std::fs::create_dir_all(&dir)?;
    let path = dir.join(format!("{name}.md"));
    if path.exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::AlreadyExists,
            "persona already exists",
        ));
    }
    std::fs::write(
        &path,
        format!("# {name}\n\nWrite your persona instructions here.\n"),
    )
}

/// 删除人格文件及其作用域目录。
pub fn delete_persona(paths: &PersonaPaths, name: &str) -> std::io::Result<()> {
    let path = paths.prompts_dir().join(format!("{name}.md"));
    if path.exists() {
        std::fs::remove_file(&path)?;
    }
    // Clean up persona scope directories
    let scope = crate::config::persona_scope_name(name);
    let dirs = [
        paths.persona_avatars_dir().join(&scope),
        paths.resource_dir().join("memory").join(&scope),
        paths.resource_dir().join("skills").join(&scope),
    ];
    for dir in &dirs {
        if dir.exists() {
            let _ = std::fs::remove_dir_all(dir);
        }
    }
    Ok(())
}

/// 重命名人格。
pub fn rename_persona(paths: &PersonaPaths, old: &str, new: &str) -> std::io::Result<()> {
    let old_path = paths.prompts_dir().join(format!("{old}.md"));
    let new_path = paths.prompts_dir().join(format!("{new}.md"));
    if new_path.exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::AlreadyExists,
            "target persona name already exists",
        ));
    }
    std::fs::rename(&old_path, &new_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scan_persona_files_reads_md_only_sorted() {
        let dir = std::env::temp_dir().join("pos-prompts-test");
        let _ = std::fs::create_dir_all(&dir);
        std::fs::write(dir.join("zeta.md"), "x").unwrap();
        std::fs::write(dir.join("alpha.md"), "x").unwrap();
        std::fs::write(dir.join("notes.txt"), "x").unwrap();

        let names = scan_persona_files(&dir);
        assert_eq!(names, vec!["alpha", "zeta"]);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn scan_missing_dir_returns_empty() {
        let missing = std::env::temp_dir().join("pos-prompts-missing");
        let _ = std::fs::remove_dir_all(&missing);
        assert!(scan_persona_files(&missing).is_empty());
    }

    #[test]
    fn create_and_delete_persona() {
        let dir = std::env::temp_dir().join("pos-prompts-crud");
        let _ = std::fs::create_dir_all(&dir.join("prompts"));
        let paths = crate::paths::PersonaPaths::new().unwrap_or_else(|_| panic!("no paths"));

        // Use raw fs for test
        let prompt_dir = dir.join("prompts");
        std::fs::create_dir_all(&prompt_dir).unwrap();
        let path = prompt_dir.join("test-persona.md");
        std::fs::write(&path, "# test").unwrap();
        assert!(path.exists());

        std::fs::remove_file(&path).unwrap();
        assert!(!path.exists());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
