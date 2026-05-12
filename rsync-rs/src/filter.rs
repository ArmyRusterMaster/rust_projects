use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterAction {
    Include,
    Exclude,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FilterRule {
    action: FilterAction,
    pattern: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FilterSet {
    rules: Vec<FilterRule>,
}

impl FilterSet {
    pub fn include(&mut self, pattern: impl Into<String>) {
        self.rules.push(FilterRule {
            action: FilterAction::Include,
            pattern: pattern.into(),
        });
    }

    pub fn exclude(&mut self, pattern: impl Into<String>) {
        self.rules.push(FilterRule {
            action: FilterAction::Exclude,
            pattern: pattern.into(),
        });
    }

    pub fn is_included(&self, relative: &Path) -> bool {
        let mut included = true;
        for rule in &self.rules {
            if pattern_matches(&rule.pattern, relative) {
                included = rule.action == FilterAction::Include;
            }
        }
        included
    }
}

fn pattern_matches(pattern: &str, relative: &Path) -> bool {
    let pattern = normalize(pattern);
    if pattern.is_empty() {
        return false;
    }

    let path = normalize(&relative.to_string_lossy());
    if path == pattern || path.starts_with(&format!("{pattern}/")) {
        return true;
    }

    let file_name = relative
        .file_name()
        .map(|name| normalize(&name.to_string_lossy()))
        .unwrap_or_default();

    if has_wildcards(&pattern) {
        wildcard_match(pattern.as_bytes(), path.as_bytes())
            || wildcard_match(pattern.as_bytes(), file_name.as_bytes())
    } else {
        path.split('/').any(|component| component == pattern)
    }
}

fn normalize(value: &str) -> String {
    value.trim_matches('/').replace('\\', "/")
}

fn has_wildcards(pattern: &str) -> bool {
    pattern.contains('*') || pattern.contains('?')
}

fn wildcard_match(pattern: &[u8], text: &[u8]) -> bool {
    let (mut p, mut t) = (0, 0);
    let mut star = None;
    let mut star_text = 0;

    while t < text.len() {
        if p < pattern.len() && (pattern[p] == b'?' || pattern[p] == text[t]) {
            p += 1;
            t += 1;
        } else if p < pattern.len() && pattern[p] == b'*' {
            star = Some(p);
            p += 1;
            star_text = t;
        } else if let Some(star_pos) = star {
            p = star_pos + 1;
            star_text += 1;
            t = star_text;
        } else {
            return false;
        }
    }

    while p < pattern.len() && pattern[p] == b'*' {
        p += 1;
    }

    p == pattern.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exclude_plain_component_matches_nested_paths() {
        let mut filters = FilterSet::default();
        filters.exclude("target");

        assert!(!filters.is_included(Path::new("target/debug/app")));
        assert!(!filters.is_included(Path::new("src/target/file")));
        assert!(filters.is_included(Path::new("src/main.rs")));
    }

    #[test]
    fn later_include_overrides_exclude() {
        let mut filters = FilterSet::default();
        filters.exclude("*.log");
        filters.include("keep.log");

        assert!(!filters.is_included(Path::new("debug.log")));
        assert!(filters.is_included(Path::new("keep.log")));
    }

    #[test]
    fn wildcard_matches_file_name_and_path() {
        let mut filters = FilterSet::default();
        filters.exclude("*.tmp");
        filters.exclude("cache/*");

        assert!(!filters.is_included(Path::new("nested/file.tmp")));
        assert!(!filters.is_included(Path::new("cache/item.bin")));
        assert!(filters.is_included(Path::new("cache")));
    }
}
