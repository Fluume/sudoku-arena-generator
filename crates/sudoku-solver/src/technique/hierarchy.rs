//! Technique hierarchy configuration: which techniques exist, their
//! difficulty category, and their relative weight. This is data only — no
//! technique *detection* logic lives here yet (that's a later milestone).
//! Keeping the config format decided and tested now means the detection
//! milestone can start from a stable `TechniqueDef.id` vocabulary.

use std::collections::HashSet;
use std::fs;
use std::path::Path;

use serde::Deserialize;
use thiserror::Error;

/// A single technique's identity, display name, difficulty category, and
/// weight. Weight orders techniques within and across categories; a
/// puzzle's difficulty rating is the weight of the hardest technique it
/// requires (see the project's ABOUT.md for the rationale).
#[derive(Debug, Deserialize, Clone, PartialEq, Eq)]
pub struct TechniqueDef {
    pub id: String,
    pub name: String,
    pub category: String,
    pub weight: u32,
}

/// A full, ordered set of techniques a technique-based solver may use.
#[derive(Debug, Deserialize, Clone, PartialEq, Eq)]
pub struct TechniqueHierarchy {
    #[serde(rename = "technique")]
    pub techniques: Vec<TechniqueDef>,
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("failed to parse technique hierarchy TOML: {0}")]
    Toml(#[from] toml::de::Error),
    #[error("failed to read technique hierarchy file: {0}")]
    Io(#[from] std::io::Error),
    #[error("duplicate technique id: {0}")]
    DuplicateId(String),
    #[error("technique hierarchy must contain at least one technique")]
    Empty,
}

const DEFAULT_HIERARCHY_TOML: &str = include_str!("techniques.default.toml");

impl TechniqueHierarchy {
    /// The bundled default hierarchy (see `techniques.default.toml`).
    pub fn default_hierarchy() -> Self {
        Self::from_toml_str(DEFAULT_HIERARCHY_TOML)
            .expect("bundled default technique hierarchy must be valid")
    }

    /// Parses and validates a hierarchy from a TOML string, so callers can
    /// fully reconfigure categories/weights/techniques without recompiling.
    pub fn from_toml_str(s: &str) -> Result<Self, ConfigError> {
        let hierarchy: TechniqueHierarchy = toml::from_str(s)?;
        hierarchy.validate()?;
        Ok(hierarchy)
    }

    /// Loads and validates a hierarchy from an external TOML file.
    pub fn from_file(path: &Path) -> Result<Self, ConfigError> {
        let contents = fs::read_to_string(path)?;
        Self::from_toml_str(&contents)
    }

    /// Looks up a technique definition by its stable id.
    pub fn get(&self, id: &str) -> Option<&TechniqueDef> {
        self.techniques.iter().find(|t| t.id == id)
    }

    /// Checks structural validity: non-empty, and every id unique.
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.techniques.is_empty() {
            return Err(ConfigError::Empty);
        }
        let mut seen = HashSet::new();
        for technique in &self.techniques {
            if !seen.insert(&technique.id) {
                return Err(ConfigError::DuplicateId(technique.id.clone()));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_default_hierarchy_is_valid_and_non_empty() {
        let hierarchy = TechniqueHierarchy::default_hierarchy();
        assert!(!hierarchy.techniques.is_empty());
        assert!(hierarchy.get("naked_single").is_some());
        assert!(hierarchy.get("xy_wing").is_some());
    }

    #[test]
    fn default_hierarchy_weights_increase_with_stated_difficulty_order() {
        let hierarchy = TechniqueHierarchy::default_hierarchy();
        let easiest = hierarchy.get("hidden_single_block").unwrap();
        let hardest = hierarchy.get("xy_wing").unwrap();
        assert!(easiest.weight < hardest.weight);
    }

    #[test]
    fn rejects_duplicate_ids() {
        let toml = r#"
            [[technique]]
            id = "naked_single"
            name = "Naked Single"
            category = "Easy"
            weight = 100

            [[technique]]
            id = "naked_single"
            name = "Naked Single (dup)"
            category = "Easy"
            weight = 110
        "#;
        assert!(matches!(
            TechniqueHierarchy::from_toml_str(toml),
            Err(ConfigError::DuplicateId(id)) if id == "naked_single"
        ));
    }

    #[test]
    fn rejects_empty_hierarchy() {
        assert!(matches!(
            TechniqueHierarchy::from_toml_str("technique = []"),
            Err(ConfigError::Empty)
        ));
    }

    #[test]
    fn custom_hierarchy_can_reorder_and_rename_categories() {
        // Demonstrates full reconfigurability: a user can invert the
        // default hierarchy's weights without touching any Rust code.
        let toml = r#"
            [[technique]]
            id = "naked_single"
            name = "Naked Single"
            category = "Vicious"
            weight = 900

            [[technique]]
            id = "xy_wing"
            name = "XY-Wing"
            category = "Very Easy"
            weight = 10
        "#;
        let hierarchy = TechniqueHierarchy::from_toml_str(toml).unwrap();
        assert!(
            hierarchy.get("naked_single").unwrap().weight
                > hierarchy.get("xy_wing").unwrap().weight
        );
    }
}
