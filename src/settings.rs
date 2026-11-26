use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    #[default]
    Deutsch,
    English,
}

impl Language {
    pub fn label(self) -> &'static str {
        match self {
            Language::Deutsch => "Deutsch",
            Language::English => "English",
        }
    }

    pub fn from_index(index: u32) -> Self {
        if index == 1 {
            Language::English
        } else {
            Language::Deutsch
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum SearchEngine {
    #[default]
    DuckDuckGo,
    Google,
    Startpage,
}

impl SearchEngine {
    pub fn label(self) -> &'static str {
        match self {
            SearchEngine::DuckDuckGo => "DuckDuckGo",
            SearchEngine::Google => "Google",
            SearchEngine::Startpage => "Startpage",
        }
    }

    pub fn from_index(index: u32) -> Self {
        match index {
            1 => SearchEngine::Google,
            2 => SearchEngine::Startpage,
            _ => SearchEngine::DuckDuckGo,
        }
    }

    pub fn form(self) -> (&'static str, &'static str) {
        match self {
            SearchEngine::DuckDuckGo => ("https://duckduckgo.com/", "q"),
            SearchEngine::Google => ("https://www.google.com/search", "q"),
            SearchEngine::Startpage => ("https://www.startpage.com/sp/search", "query"),
        }
    }

    pub fn query_url(self, query: &str) -> String {
        let (base, param) = self.form();
        let mut url = url::Url::parse(base).expect("gültige Suchmaschinen-URL");
        url.query_pairs_mut().append_pair(param, query);
        url.to_string()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub search_engine: SearchEngine,
    pub wallpaper: Option<String>,
    #[serde(default = "default_tab_shortcut")]
    pub tab_shortcut: String,
    #[serde(default = "default_new_tab_shortcut")]
    pub new_tab_shortcut: String,
    #[serde(default = "default_close_tab_shortcut")]
    pub close_tab_shortcut: String,
    #[serde(default = "default_url_shortcut")]
    pub url_shortcut: String,
    #[serde(default = "default_settings_shortcut")]
    pub settings_shortcut: String,
    #[serde(default = "default_bookmark_shortcut")]
    pub bookmark_shortcut: String,
    #[serde(default = "default_private_shortcut")]
    pub private_shortcut: String,
    pub profile: String,
    #[serde(default)]
    pub profiles: Vec<String>,
    #[serde(default = "default_true")]
    pub adblock: bool,
    #[serde(default = "default_opacity")]
    pub opacity: f64,
    #[serde(default)]
    pub transparent: bool,
    #[serde(default)]
    pub language: Language,
    #[serde(default = "default_hardware")]
    pub hardware_acceleration: bool,
}

fn default_true() -> bool {
    true
}

fn default_opacity() -> f64 {
    1.0
}

fn default_tab_shortcut() -> String {
    "<Control>L".to_string()
}

fn default_new_tab_shortcut() -> String {
    "<Control>T".to_string()
}

fn default_close_tab_shortcut() -> String {
    "<Control>W".to_string()
}

fn default_url_shortcut() -> String {
    "<Control>K".to_string()
}

fn default_settings_shortcut() -> String {
    "<Control>comma".to_string()
}

fn default_bookmark_shortcut() -> String {
    "<Control>D".to_string()
}

fn default_private_shortcut() -> String {
    "<Control><Shift>N".to_string()
}

fn default_hardware() -> bool {
    true
}

fn bundled_wallpaper_dirs() -> Vec<PathBuf> {
    let mut dirs = vec![
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("assets")
            .join("wallpapers"),
        PathBuf::from("/usr/share/lumen-1_browser/wallpapers"),
    ];
    if let Some(data) = dirs::data_dir() {
        dirs.push(data.join("lumen-1_browser").join("wallpapers"));
    }
    dirs
}

pub fn wallpaper_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if let Some(path) = fs::read_to_string(Settings::path())
        .ok()
        .and_then(|data| toml::from_str::<Settings>(&data).ok())
        .and_then(|settings| settings.wallpaper)
        .map(PathBuf::from)
    {
        if let Some(parent) = path.parent() {
            dirs.push(parent.to_path_buf());
        }
    }
    dirs.extend(bundled_wallpaper_dirs());
    if let Some(pictures) = dirs::picture_dir() {
        dirs.push(pictures.join("Wallpapers"));
    }
    if let Some(home) = dirs::home_dir() {
        dirs.push(home.join("Pictures").join("Wallpapers"));
        dirs.push(home.join("Bilder").join("Wallpapers"));
    }
    dirs
}

pub fn default_wallpaper() -> Option<PathBuf> {
    bundled_wallpaper_dirs()
        .into_iter()
        .flat_map(|dir| fs::read_dir(dir).ok().into_iter().flatten())
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .find(|path| {
            matches!(
                path.extension()
                    .and_then(|ext| ext.to_str())
                    .map(|ext| ext.to_ascii_lowercase())
                    .as_deref(),
                Some("png") | Some("jpg") | Some("jpeg") | Some("webp")
            )
        })
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            search_engine: SearchEngine::Startpage,
            wallpaper: default_wallpaper().map(|path| path.to_string_lossy().to_string()),
            tab_shortcut: default_tab_shortcut(),
            new_tab_shortcut: default_new_tab_shortcut(),
            close_tab_shortcut: default_close_tab_shortcut(),
            url_shortcut: default_url_shortcut(),
            settings_shortcut: default_settings_shortcut(),
            bookmark_shortcut: default_bookmark_shortcut(),
            private_shortcut: default_private_shortcut(),
            profile: "Standard".to_string(),
            profiles: vec!["Standard".to_string()],
            adblock: false,
            opacity: 1.0,
            transparent: false,
            language: Language::Deutsch,
            hardware_acceleration: true,
        }
    }
}

impl Settings {
    pub fn path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("lumen-1_browser")
            .join("config.toml")
    }

    pub fn load() -> Self {
        let mut settings: Self = fs::read_to_string(Self::path())
            .ok()
            .and_then(|data| toml::from_str(&data).ok())
            .unwrap_or_default();
        let missing = settings
            .wallpaper
            .as_ref()
            .map(|path| !PathBuf::from(path).is_file())
            .unwrap_or(true);
        if missing {
            settings.wallpaper = default_wallpaper().map(|path| path.to_string_lossy().to_string());
        }
        settings
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, toml::to_string_pretty(self)?)?;
        Ok(())
    }
}
