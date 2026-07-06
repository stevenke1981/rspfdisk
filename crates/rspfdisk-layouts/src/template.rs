use std::collections::HashMap;
use std::fs;
use std::path::Path;

use serde::Deserialize;

use crate::error::{LayoutError, LayoutResult};

#[derive(Debug, Clone, Deserialize)]
pub struct PartitionTemplate {
    pub name: String,
    pub size: String,
    #[serde(rename = "type")]
    pub partition_type: String,
    pub filesystem: Option<String>,
    pub mount: Option<String>,
    pub note: Option<String>,
    pub flags: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LayoutTemplate {
    pub name: String,
    pub display_name: String,
    pub table: String,
    pub boot_mode: String,
    pub min_disk_size: String,
    pub partitions: Vec<PartitionTemplate>,
}

impl LayoutTemplate {
    pub fn from_toml(content: &str) -> LayoutResult<Self> {
        toml::from_str(content).map_err(LayoutError::from)
    }
}

pub fn load_template(path: impl AsRef<Path>) -> LayoutResult<LayoutTemplate> {
    let content =
        fs::read_to_string(path.as_ref()).map_err(|e| LayoutError::TemplateParse(e.to_string()))?;
    LayoutTemplate::from_toml(&content)
}

pub struct TemplateRegistry {
    templates: HashMap<String, LayoutTemplate>,
}

impl TemplateRegistry {
    pub fn new() -> Self {
        Self {
            templates: HashMap::new(),
        }
    }

    pub fn register(&mut self, template: LayoutTemplate) {
        self.templates.insert(template.name.clone(), template);
    }

    pub fn load_dir(&mut self, dir: impl AsRef<Path>) -> LayoutResult<()> {
        for entry in
            fs::read_dir(dir.as_ref()).map_err(|e| LayoutError::TemplateParse(e.to_string()))?
        {
            let entry = entry.map_err(|e| LayoutError::TemplateParse(e.to_string()))?;
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("toml") {
                let template = load_template(&path)?;
                self.register(template);
            }
        }
        Ok(())
    }

    pub fn get(&self, name: &str) -> LayoutResult<&LayoutTemplate> {
        self.templates
            .get(name)
            .ok_or_else(|| LayoutError::TemplateNotFound(name.to_string()))
    }

    pub fn names(&self) -> Vec<&str> {
        let mut names: Vec<_> = self.templates.keys().map(|s| s.as_str()).collect();
        names.sort();
        names
    }
}

impl Default for TemplateRegistry {
    fn default() -> Self {
        Self::new()
    }
}
