//! Le journal de Liberty — mémoire auditable de tout ce que l'esprit fait.
//!
//! Append-only, une ligne JSON par événement (décision, exécution, question,
//! refus). C'est la contrepartie de l'autonomie : tout ce qui est fait en
//! silence est consultable, et ce qui est annulable garde sa commande d'undo.

use serde_json::json;
use std::io::Write;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

/// Répertoire d'état : `LIBERTY_STATE_DIR`, sinon `/var/lib/liberty` (si
/// accessible en écriture), sinon `~/.liberty` — et un équivalent Windows
/// pour le développement.
pub fn state_dir() -> PathBuf {
    if let Ok(d) = std::env::var("LIBERTY_STATE_DIR") {
        if !d.is_empty() {
            return PathBuf::from(d);
        }
    }
    #[cfg(unix)]
    {
        let system = PathBuf::from("/var/lib/liberty");
        if std::fs::create_dir_all(&system).is_ok()
            && std::fs::metadata(&system).map(|m| !m.permissions().readonly()).unwrap_or(false)
        {
            // Vérifie l'écriture réelle (root vs utilisateur).
            if std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(system.join(".w"))
                .is_ok()
            {
                let _ = std::fs::remove_file(system.join(".w"));
                return system;
            }
        }
        if let Ok(home) = std::env::var("HOME") {
            return PathBuf::from(home).join(".liberty");
        }
    }
    #[cfg(windows)]
    {
        if let Ok(d) = std::env::var("LOCALAPPDATA") {
            return PathBuf::from(d).join("liberty");
        }
    }
    PathBuf::from(".liberty")
}

pub struct Journal {
    path: PathBuf,
}

#[derive(Clone)]
pub struct Entry {
    pub kind: String,    // decision | exec | question | refus | intent | erreur
    pub name: String,    // libellé court de l'action / l'événement
    pub detail: String,  // décision prise, sortie de commande, question…
    pub command: String, // commande exécutée, le cas échéant
    pub undo: String,    // commande d'annulation, le cas échéant
}

impl Journal {
    pub fn open() -> Self {
        let dir = state_dir();
        let _ = std::fs::create_dir_all(&dir);
        Self { path: dir.join("journal.jsonl") }
    }

    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    pub fn record(&self, e: &Entry) {
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let line = json!({
            "ts": ts,
            "kind": e.kind,
            "name": e.name,
            "detail": e.detail,
            "command": e.command,
            "undo": e.undo,
        });
        if let Ok(mut f) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
        {
            let _ = writeln!(f, "{line}");
        }
    }

    /// La dernière entrée annulable (pour `:undo` dans lish).
    pub fn last_undoable(&self) -> Option<Entry> {
        let text = std::fs::read_to_string(&self.path).ok()?;
        for line in text.lines().rev() {
            let v: serde_json::Value = serde_json::from_str(line).ok()?;
            let undo = v["undo"].as_str().unwrap_or("");
            if !undo.is_empty() {
                return Some(Entry {
                    kind: v["kind"].as_str().unwrap_or("").into(),
                    name: v["name"].as_str().unwrap_or("").into(),
                    detail: v["detail"].as_str().unwrap_or("").into(),
                    command: v["command"].as_str().unwrap_or("").into(),
                    undo: undo.into(),
                });
            }
        }
        None
    }

    /// Les N dernières lignes brutes (pour `:journal` dans lish).
    pub fn tail(&self, n: usize) -> Vec<String> {
        std::fs::read_to_string(&self.path)
            .map(|t| {
                let lines: Vec<&str> = t.lines().collect();
                lines
                    .iter()
                    .rev()
                    .take(n)
                    .rev()
                    .map(|s| s.to_string())
                    .collect()
            })
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_journal(name: &str) -> Journal {
        let dir = std::env::temp_dir().join(format!("liberty-test-{name}"));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        Journal { path: dir.join("journal.jsonl") }
    }

    #[test]
    fn record_then_read_back_last_undoable() {
        let j = temp_journal("undo");
        j.record(&Entry {
            kind: "exec".into(),
            name: "vider le cache".into(),
            detail: "ok".into(),
            command: "rm -rf ~/.cache/x".into(),
            undo: String::new(),
        });
        j.record(&Entry {
            kind: "exec".into(),
            name: "déplacer les doublons".into(),
            detail: "ok".into(),
            command: "mv a b".into(),
            undo: "mv b a".into(),
        });
        let last = j.last_undoable().expect("une entrée annulable");
        assert_eq!(last.undo, "mv b a");
        assert_eq!(j.tail(10).len(), 2);
    }
}
