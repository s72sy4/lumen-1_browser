use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    pub url: String,
    pub title: String,
    pub visited: String,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct BrowserData {
    #[serde(default)]
    pub history: Vec<Entry>,
    #[serde(default)]
    pub bookmarks: Vec<Entry>,
}

fn timestamp() -> String {
    chrono::Local::now().format("%d.%m.%Y %H:%M").to_string()
}

impl BrowserData {
    pub fn path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("lumen-1_browser")
            .join("browser.toml")
    }

    pub fn load() -> Self {
        fs::read_to_string(Self::path())
            .ok()
            .and_then(|data| toml::from_str(&data).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, toml::to_string_pretty(self)?)?;
        Ok(())
    }

    pub fn add_history(&mut self, url: String, title: String) {
        if url.starts_with("lumen-1-browser://") || url.is_empty() {
            return;
        }
        if let Some(first) = self.history.first_mut() {
            if first.url == url {
                first.visited = timestamp();
                let _ = self.save();
                return;
            }
        }
        self.history.insert(
            0,
            Entry {
                url,
                title,
                visited: timestamp(),
            },
        );
        self.history.truncate(200);
        let _ = self.save();
    }

    pub fn add_bookmark(&mut self, url: String, title: String) {
        if url.starts_with("lumen-1-browser://") || url.is_empty() {
            return;
        }
        if self.bookmarks.iter().any(|entry| entry.url == url) {
            return;
        }
        self.bookmarks.insert(
            0,
            Entry {
                url,
                title,
                visited: timestamp(),
            },
        );
        let _ = self.save();
    }

    pub fn remove_bookmark(&mut self, url: &str) {
        self.bookmarks.retain(|entry| entry.url != url);
        let _ = self.save();
    }

    pub fn clear_history(&mut self) {
        self.history.clear();
        let _ = self.save();
    }
}
