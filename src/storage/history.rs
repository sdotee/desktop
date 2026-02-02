use crate::config::Config;
use crate::error::Result;
use crate::storage::models::{FileEntry, LinkEntry, TextEntry};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct History {
    pub links: Vec<LinkEntry>,
    pub texts: Vec<TextEntry>,
    pub files: Vec<FileEntry>,
}

#[derive(Debug)]
pub struct HistoryStorage {
    path: PathBuf,
    history: History,
}

impl HistoryStorage {
    pub fn load() -> Result<Self> {
        let path = Self::history_file_path()?;
        let history = if path.exists() {
            let content = fs::read_to_string(&path)?;
            serde_json::from_str(&content)?
        } else {
            History::default()
        };

        Ok(Self { path, history })
    }

    fn history_file_path() -> Result<PathBuf> {
        let data_dir = Config::data_dir()?;
        Ok(data_dir.join("history.json"))
    }

    pub fn save(&self) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(&self.history)?;
        fs::write(&self.path, content)?;
        Ok(())
    }

    // Links
    pub fn add_link(&mut self, entry: LinkEntry) -> Result<()> {
        self.history.links.insert(0, entry);
        self.save()
    }

    pub fn remove_link(&mut self, domain: &str, slug: &str) -> Result<()> {
        self.history
            .links
            .retain(|l| !(l.domain == domain && l.slug == slug));
        self.save()
    }

    pub fn links(&self) -> &[LinkEntry] {
        &self.history.links
    }

    pub fn clear_links(&mut self) {
        self.history.links.clear();
    }

    // Texts
    pub fn add_text(&mut self, entry: TextEntry) -> Result<()> {
        self.history.texts.insert(0, entry);
        self.save()
    }

    pub fn remove_text(&mut self, domain: &str, slug: &str) -> Result<()> {
        self.history
            .texts
            .retain(|t| !(t.domain == domain && t.slug == slug));
        self.save()
    }

    pub fn texts(&self) -> &[TextEntry] {
        &self.history.texts
    }

    pub fn clear_texts(&mut self) {
        self.history.texts.clear();
    }

    // Files
    pub fn add_file(&mut self, entry: FileEntry) -> Result<()> {
        self.history.files.insert(0, entry);
        self.save()
    }

    pub fn remove_file(&mut self, domain: &str, slug: &str) -> Result<()> {
        self.history
            .files
            .retain(|f| !(f.domain == domain && f.slug == slug));
        self.save()
    }

    pub fn files(&self) -> &[FileEntry] {
        &self.history.files
    }

    pub fn clear_files(&mut self) {
        self.history.files.clear();
    }
}
