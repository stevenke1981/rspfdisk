use thiserror::Error;

pub type LayoutResult<T> = Result<T, LayoutError>;

#[derive(Debug, Error)]
pub enum LayoutError {
    #[error("core error: {0}")]
    Core(#[from] rspfdisk_core::CoreError),
    #[error("template parse error: {0}")]
    TemplateParse(String),
    #[error("template not found: {0}")]
    TemplateNotFound(String),
    #[error("TOML error: {0}")]
    Toml(#[from] toml::de::Error),
}
