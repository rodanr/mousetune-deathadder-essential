// SPDX-License-Identifier: GPL-3.0-or-later
//! Remembers the last lighting effect per mouse, because the mouse cannot
//! report its current effect.

use std::collections::BTreeMap;
use std::path::PathBuf;

use directories::ProjectDirs;
use mousetune_core::{Effect, Led};
use serde::{Deserialize, Serialize};

/// serial -> LED name -> effect name.
type Store = BTreeMap<String, BTreeMap<String, String>>;

/// Persistent preferences. Failures to read or write are logged and ignored:
/// losing a remembered effect must never break the app.
#[derive(Debug)]
pub struct Prefs {
    path: Option<PathBuf>,
    store: Store,
}

#[derive(Serialize, Deserialize, Default)]
struct File {
    effects: Store,
}

impl Prefs {
    /// Loads preferences from the per-user config directory.
    #[must_use]
    pub fn load_default() -> Self {
        let path = ProjectDirs::from("io.github", "rodanr", "MouseTune")
            .map(|dirs| dirs.config_dir().join("prefs.json"));
        Self::load(path)
    }

    /// Loads preferences from `path` (`None` keeps them in memory only).
    #[must_use]
    pub fn load(path: Option<PathBuf>) -> Self {
        let store = path
            .as_ref()
            .and_then(|path| std::fs::read(path).ok())
            .and_then(|bytes| serde_json::from_slice::<File>(&bytes).ok())
            .map(|file| file.effects)
            .unwrap_or_default();
        Self { path, store }
    }

    /// The remembered effect of an LED, if any.
    #[must_use]
    pub fn effect(&self, serial: &str, led: Led) -> Option<Effect> {
        self.store
            .get(serial)?
            .get(led_key(led))
            .and_then(|name| parse_effect(name))
    }

    /// Remembers an effect and writes the file.
    pub fn set_effect(&mut self, serial: &str, led: Led, effect: Effect) {
        self.store
            .entry(serial.to_owned())
            .or_default()
            .insert(led_key(led).to_owned(), effect_name(effect).to_owned());
        self.save();
    }

    fn save(&self) {
        let Some(path) = &self.path else { return };
        let file = File {
            effects: self.store.clone(),
        };
        let result = path
            .parent()
            .map_or(Ok(()), std::fs::create_dir_all)
            .and_then(|()| serde_json::to_vec_pretty(&file).map_err(Into::into))
            .and_then(|bytes| std::fs::write(path, bytes));
        if let Err(error) = result {
            tracing::warn!(%error, "could not save preferences");
        }
    }
}

fn led_key(led: Led) -> &'static str {
    match led {
        Led::Logo => "logo",
        Led::Scroll => "scroll",
    }
}

fn effect_name(effect: Effect) -> &'static str {
    match effect {
        Effect::Off => "off",
        Effect::Static => "static",
        Effect::Breathe => "breathe",
    }
}

fn parse_effect(name: &str) -> Option<Effect> {
    match name {
        "off" => Some(Effect::Off),
        "static" => Some(Effect::Static),
        "breathe" => Some(Effect::Breathe),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn effects_persist_per_serial_and_led() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("nested").join("prefs.json");

        let mut prefs = Prefs::load(Some(path.clone()));
        assert_eq!(prefs.effect("A", Led::Logo), None);
        prefs.set_effect("A", Led::Logo, Effect::Breathe);
        prefs.set_effect("A", Led::Scroll, Effect::Off);
        prefs.set_effect("B", Led::Logo, Effect::Static);

        let reloaded = Prefs::load(Some(path));
        assert_eq!(reloaded.effect("A", Led::Logo), Some(Effect::Breathe));
        assert_eq!(reloaded.effect("A", Led::Scroll), Some(Effect::Off));
        assert_eq!(reloaded.effect("B", Led::Logo), Some(Effect::Static));
        assert_eq!(reloaded.effect("B", Led::Scroll), None);
    }

    #[test]
    fn corrupt_file_is_ignored() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("prefs.json");
        std::fs::write(&path, b"{not json").unwrap();
        assert_eq!(Prefs::load(Some(path)).effect("A", Led::Logo), None);
    }

    #[test]
    fn in_memory_prefs_work_without_a_path() {
        let mut prefs = Prefs::load(None);
        prefs.set_effect("A", Led::Logo, Effect::Static);
        assert_eq!(prefs.effect("A", Led::Logo), Some(Effect::Static));
    }
}
