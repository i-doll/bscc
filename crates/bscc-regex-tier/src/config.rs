use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct LanguageConfig {
    pub name: String,
    #[serde(default)]
    pub extensions: Vec<String>,
    #[serde(default)]
    pub line_comments: Vec<String>,
    #[serde(default)]
    pub block_comments: Vec<[String; 2]>,
    #[serde(default = "default_strings")]
    pub strings: Vec<[String; 2]>,
}

fn default_strings() -> Vec<[String; 2]> {
    vec![["\"".into(), "\"".into()]]
}
