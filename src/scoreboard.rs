use std::cmp::Reverse;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;

const LEADERBOARD_FILENAME: &str = "leaderboard.txt";
const MAX_ENTRIES: usize = 10;

#[derive(Clone)]
pub struct ScoreEntry {
    pub name: String,
    pub score: u32,
}

impl ScoreEntry {
    fn serialize(&self) -> String {
        format!("{}|{}", self.score, self.name)
    }

    fn parse(line: &str) -> Option<Self> {
        let mut parts = line.splitn(2, '|');
        let score_part = parts.next()?;
        let name_part = parts.next()?;
        let score = score_part.parse().ok()?;
        Some(Self {
            name: name_part.to_string(),
            score,
        })
    }
}

#[derive(Default)]
pub struct Leaderboard {
    entries: Vec<ScoreEntry>,
}

impl Leaderboard {
    pub fn load() -> Self {
        let path = Self::path();
        if let Ok(file) = File::open(&path) {
            let reader = BufReader::new(file);
            let mut leaderboard = Leaderboard::default();
            for line in reader.lines().flatten() {
                if let Some(entry) = ScoreEntry::parse(&line) {
                    leaderboard.entries.push(entry);
                }
            }
            leaderboard.normalize();
            leaderboard
        } else {
            Leaderboard::default()
        }
    }

    pub fn save(&self) {
        if let Ok(mut file) = File::create(Self::path()) {
            for entry in self.entries.iter() {
                if writeln!(file, "{}", entry.serialize()).is_err() {
                    break;
                }
            }
        }
    }

    pub fn submit(&mut self, name: &str, score: u32) {
        self.entries.push(ScoreEntry {
            name: name.to_string(),
            score,
        });
        self.normalize();
    }

    fn normalize(&mut self) {
        self.entries
            .sort_unstable_by_key(|entry| Reverse(entry.score));
        self.entries.truncate(MAX_ENTRIES);
    }

    pub fn entries(&self) -> &[ScoreEntry] {
        &self.entries
    }

    fn path() -> PathBuf {
        if let Ok(current) = std::env::current_dir() {
            current.join(LEADERBOARD_FILENAME)
        } else {
            PathBuf::from(LEADERBOARD_FILENAME)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::sync::{Mutex, OnceLock};
    use tempfile::{TempDir, tempdir};

    #[test]
    fn score_entry_round_trip() {
        let entry = ScoreEntry {
            name: "tester".to_string(),
            score: 1234,
        };
        let serialized = entry.serialize();
        let parsed = ScoreEntry::parse(&serialized).expect("should parse serialized");
        assert_eq!(parsed.name, "tester");
        assert_eq!(parsed.score, 1234);
        assert!(ScoreEntry::parse("garbage").is_none());
    }

    #[test]
    fn leaderboard_submit_normalizes() {
        let mut leaderboard = Leaderboard::default();
        for score in 0u32..(MAX_ENTRIES as u32 + 5) {
            leaderboard.submit("player", score);
        }
        assert_eq!(leaderboard.entries().len(), MAX_ENTRIES);
        assert_eq!(leaderboard.entries()[0].score, MAX_ENTRIES as u32 + 4);
        assert_eq!(leaderboard.entries().last().unwrap().score, 5);
    }

    fn run_in_temp_dir<T>(dir: &TempDir, test: impl FnOnce() -> T) -> T {
        static CWD_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        let lock = CWD_LOCK.get_or_init(|| Mutex::new(()));
        let _guard = lock.lock().expect("lock current dir guard");
        let cwd = env::current_dir().expect("current dir available");
        env::set_current_dir(dir.path()).expect("set cwd to temp dir");
        let result = test();
        env::set_current_dir(cwd).expect("restore cwd");
        result
    }

    #[test]
    fn load_missing_file_is_empty() {
        let dir = tempdir().unwrap();
        let board = run_in_temp_dir(&dir, || Leaderboard::load());
        assert!(board.entries().is_empty());
    }

    #[test]
    fn save_and_load_round_trip() {
        let dir = tempdir().unwrap();
        run_in_temp_dir(&dir, || {
            let mut board = Leaderboard::default();
            board.submit("alpha", 50);
            board.submit("bravo", 150);
            board.save();
            let reloaded = Leaderboard::load();
            assert_eq!(reloaded.entries().len(), 2);
            assert_eq!(reloaded.entries()[0].name, "bravo");
            assert_eq!(reloaded.entries()[0].score, 150);
            assert_eq!(reloaded.entries()[1].name, "alpha");
        });
    }
}
