//! 自定义提示词页 — 人格列表与激活。

use crate::paths::PersonaPaths;
use ratatui::widgets::ListState;
use std::path::Path;

/// 页面状态。
#[derive(Default)]
pub struct PromptsPage {
    pub state: ListState,
}

impl PromptsPage {
    pub fn new() -> Self {
        let mut state = ListState::default();
        state.select(Some(0));
        Self { state }
    }
}

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
}
