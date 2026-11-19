//! Offline profiles: playing without a Microsoft account.
//!
//! A profile is only a name. The game gives an offline player the UUID vanilla derives from the
//! name ([`horizon_protocol::crypto::offline_uuid`], MD5 of `OfflinePlayer:<name>`), so single
//! player, LAN and offline-mode servers work; online-mode servers refuse it, as they should.
//! The launcher passes the name with `--name` and never asks the Microsoft services.
//!
//! The list lives in `offline.json` in the launcher's folder. `selected` is `None` while a
//! Microsoft account plays, so signing in and out does not lose the offline names.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Vanilla's limit (`StringUtil.isValidPlayerName`).
pub const MAX_NAME: usize = 16;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Offline {
    /// The offline profiles, oldest first.
    pub profiles: Vec<String>,
    /// The profile that plays; none means the Microsoft account does.
    pub selected: Option<String>,
}

impl Offline {
    pub fn path() -> PathBuf {
        crate::backend::data_dir().join("offline.json")
    }

    pub fn load() -> Self {
        Self::load_from(&Self::path())
    }

    pub(crate) fn load_from(path: &std::path::Path) -> Self {
        std::fs::read_to_string(path)
            .ok()
            .and_then(|t| serde_json::from_str(&t).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) -> Result<(), String> {
        self.save_to(&Self::path())
    }

    pub(crate) fn save_to(&self, path: &std::path::Path) -> Result<(), String> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
        }
        let text = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        // Renamed over the old file, so a crash mid-write leaves the old list.
        let tmp = path.with_extension(format!("json.{}.tmp", std::process::id()));
        std::fs::write(&tmp, text + "\n")
            .and_then(|()| std::fs::rename(&tmp, path))
            .map_err(|e| {
                let _ = std::fs::remove_file(&tmp);
                e.to_string()
            })
    }

    /// The name that plays, when the offline profile does.
    pub fn selected_name(&self) -> Option<&str> {
        let name = self.selected.as_deref()?;
        self.profiles
            .iter()
            .any(|p| p.eq_ignore_ascii_case(name))
            .then_some(name)
    }

    /// Adds a profile (case-insensitively unique) and selects it. A name that is not a valid
    /// player name is refused with what to fix.
    pub fn add(&mut self, name: &str) -> Result<String, String> {
        let name = name.trim();
        check(name)?;
        if self.profiles.iter().any(|p| p.eq_ignore_ascii_case(name)) {
            return Err(format!("There is already an offline profile called {name}."));
        }
        self.profiles.push(name.to_owned());
        self.selected = Some(name.to_owned());
        Ok(name.to_owned())
    }

    /// Removes a profile, keeping the selection only if it still exists.
    pub fn remove(&mut self, name: &str) {
        self.profiles.retain(|p| !p.eq_ignore_ascii_case(name));
        if self
            .selected
            .as_deref()
            .is_some_and(|s| s.eq_ignore_ascii_case(name))
        {
            self.selected = self.profiles.first().cloned();
        }
    }

    /// Plays as this profile from now on (`None`: the Microsoft account again).
    pub fn select(&mut self, name: Option<&str>) {
        self.selected = name
            .filter(|n| self.profiles.iter().any(|p| p.eq_ignore_ascii_case(n)))
            .map(str::to_owned);
    }
}

/// Whether `name` is a name the game accepts, and why not when it isn't.
pub fn check(name: &str) -> Result<(), String> {
    if name.is_empty() {
        return Err("Type a name for the offline profile.".into());
    }
    if name.chars().count() > MAX_NAME {
        return Err(format!("An offline name is at most {MAX_NAME} characters long."));
    }
    if !name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_')
    {
        return Err("An offline name uses letters, digits and underscores only.".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_are_vanillas() {
        assert!(check("Steve").is_ok());
        assert!(check("Player_01").is_ok());
        assert!(check("").is_err());
        assert!(check("a b").is_err());
        assert!(check("abcdefghijklmnopq").is_err());
        assert!(check("abcdefghijklmnop").is_ok());
        assert!(check("şahin").is_err());
    }

    #[test]
    fn a_name_is_added_once_and_selected() {
        let mut o = Offline::default();
        o.add("Steve").unwrap();
        assert_eq!(o.selected_name(), Some("Steve"));
        assert!(o.add("steve").is_err());
        o.add("Alex").unwrap();
        assert_eq!(o.selected_name(), Some("Alex"));
        // A name that is not a profile never plays.
        o.selected = Some("Nobody".into());
        assert_eq!(o.selected_name(), None);
    }

    #[test]
    fn removing_the_selected_profile_falls_back_to_the_first() {
        let mut o = Offline::default();
        o.add("Steve").unwrap();
        o.add("Alex").unwrap();
        o.remove("Alex");
        assert_eq!(o.selected_name(), Some("Steve"));
        o.remove("Steve");
        assert_eq!(o.selected_name(), None);
        assert!(o.profiles.is_empty());
    }

    #[test]
    fn the_list_survives_a_save_and_load() {
        let dir = std::env::temp_dir().join(format!("hz-offline-{}", std::process::id()));
        let path = dir.join("offline.json");
        let mut o = Offline::default();
        o.add("Steve").unwrap();
        o.save_to(&path).unwrap();
        assert_eq!(Offline::load_from(&path), o);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
