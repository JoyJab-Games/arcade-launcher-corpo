use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// One installed game, keyed by `name` — the stable, source-agnostic
/// reference chosen at install time (see GameSource::resolve_install_target,
/// and `arcade install`'s "save as which name?" prompt). `source`/
/// `source_ref` record where it came from but nothing else keys off them.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GameConfig {
    pub name: String,
    pub source: String,
    pub source_ref: String,
    #[serde(default)]
    pub exec: Option<String>,
    #[serde(default)]
    pub proton: bool,
    #[serde(default)]
    pub prefix_path: Option<String>,
    #[serde(default)]
    pub released_for_players: bool,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct GamesFile {
    #[serde(default)]
    games: Vec<GameConfig>,
}

/// JSON-file-backed store for GameConfig entries.
pub struct GameStore {
    path: PathBuf,
}

impl GameStore {
    pub fn open(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    pub fn default_path() -> PathBuf {
        crate::data_dir().join("games.json")
    }

    fn load(&self) -> std::io::Result<GamesFile> {
        match fs::read_to_string(&self.path) {
            Ok(contents) => Ok(serde_json::from_str(&contents).unwrap_or_default()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(GamesFile::default()),
            Err(e) => Err(e),
        }
    }

    fn save(&self, file: &GamesFile) -> std::io::Result<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }
        let contents = serde_json::to_string_pretty(file).expect("GamesFile always serializes");
        fs::write(&self.path, contents)
    }

    pub fn list(&self) -> std::io::Result<Vec<GameConfig>> {
        Ok(self.load()?.games)
    }

    pub fn get(&self, name: &str) -> std::io::Result<Option<GameConfig>> {
        Ok(self.load()?.games.into_iter().find(|g| g.name == name))
    }

    /// Adds a new game. Errors if a game with the same name already exists.
    pub fn add(&self, game: GameConfig) -> Result<(), StoreError> {
        let mut file = self.load()?;
        if file.games.iter().any(|g| g.name == game.name) {
            return Err(StoreError::AlreadyExists(game.name));
        }
        file.games.push(game);
        self.save(&file)?;
        Ok(())
    }

    pub fn remove(&self, name: &str) -> Result<(), StoreError> {
        let mut file = self.load()?;
        let before = file.games.len();
        file.games.retain(|g| g.name != name);
        if file.games.len() == before {
            return Err(StoreError::NotFound(name.to_string()));
        }
        self.save(&file)?;
        Ok(())
    }

    pub fn set_released(&self, name: &str, released: bool) -> Result<(), StoreError> {
        let mut file = self.load()?;
        let game = file
            .games
            .iter_mut()
            .find(|g| g.name == name)
            .ok_or_else(|| StoreError::NotFound(name.to_string()))?;
        game.released_for_players = released;
        self.save(&file)?;
        Ok(())
    }
}

#[derive(Debug)]
pub enum StoreError {
    AlreadyExists(String),
    NotFound(String),
    Io(std::io::Error),
}

impl std::fmt::Display for StoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StoreError::AlreadyExists(name) => write!(f, "a game named '{name}' already exists"),
            StoreError::NotFound(name) => write!(f, "no game named '{name}' found"),
            StoreError::Io(e) => write!(f, "I/O error: {e}"),
        }
    }
}

impl std::error::Error for StoreError {}

impl From<std::io::Error> for StoreError {
    fn from(e: std::io::Error) -> Self {
        StoreError::Io(e)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_store() -> (GameStore, tempfile::TempDir) {
        let dir = tempfile::tempdir().unwrap();
        let store = GameStore::open(dir.path().join("games.json"));
        (store, dir)
    }

    fn sample(name: &str) -> GameConfig {
        GameConfig {
            name: name.to_string(),
            source: "steam".to_string(),
            source_ref: "70".to_string(),
            exec: None,
            proton: false,
            prefix_path: None,
            released_for_players: false,
        }
    }

    #[test]
    fn list_is_empty_before_anything_is_added() {
        let (store, _dir) = temp_store();
        assert_eq!(store.list().unwrap(), vec![]);
    }

    #[test]
    fn add_then_list_roundtrips() {
        let (store, _dir) = temp_store();
        store.add(sample("half-life")).unwrap();
        assert_eq!(store.list().unwrap(), vec![sample("half-life")]);
    }

    #[test]
    fn add_rejects_duplicate_names() {
        let (store, _dir) = temp_store();
        store.add(sample("half-life")).unwrap();
        let err = store.add(sample("half-life")).unwrap_err();
        assert!(matches!(err, StoreError::AlreadyExists(name) if name == "half-life"));
    }

    #[test]
    fn remove_deletes_and_errors_if_missing() {
        let (store, _dir) = temp_store();
        store.add(sample("half-life")).unwrap();
        store.remove("half-life").unwrap();
        assert_eq!(store.list().unwrap(), vec![]);
        assert!(matches!(
            store.remove("half-life").unwrap_err(),
            StoreError::NotFound(_)
        ));
    }

    #[test]
    fn set_released_toggles_the_flag() {
        let (store, _dir) = temp_store();
        store.add(sample("half-life")).unwrap();
        store.set_released("half-life", true).unwrap();
        assert!(store.get("half-life").unwrap().unwrap().released_for_players);
        store.set_released("half-life", false).unwrap();
        assert!(!store.get("half-life").unwrap().unwrap().released_for_players);
    }

    #[test]
    fn persists_across_separate_store_instances() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("games.json");
        GameStore::open(&path).add(sample("half-life")).unwrap();
        let reopened = GameStore::open(&path);
        assert_eq!(reopened.list().unwrap(), vec![sample("half-life")]);
    }
}
