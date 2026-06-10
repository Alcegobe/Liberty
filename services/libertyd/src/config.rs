//! Configuration système de Liberty — `/etc/liberty/liberty.toml`.
//!
//! Parseur volontairement minimal (sous-ensemble de TOML : sections, chaînes,
//! entiers, booléens, tableaux de chaînes) pour rester sans dépendance. La
//! config est la voix durable de l'utilisateur : profil d'autonomie,
//! capacités accordées, modèle, rythme du démon.

use crate::autonomy::{Autonomy, Policy};
use crate::effects::Effect;
use std::collections::HashMap;
use std::path::PathBuf;

pub struct Config {
    /// Modèle forcé ; vide = auto (le plus capable accessible au compte).
    pub model: String,
    /// Période de la boucle du démon, en secondes.
    pub heartbeat_secs: u64,
    /// Profil d'autonomie : "prudent" | "confiance" | "manuel".
    pub profile: String,
    /// Capacités accordées à l'esprit (le garde-fou matériel).
    pub capabilities: Vec<Effect>,
    /// Fichier contenant la clé API Anthropic (mode 600, hors du dépôt).
    pub api_key_file: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            model: String::new(),
            heartbeat_secs: 300,
            profile: "prudent".into(),
            capabilities: vec![
                Effect::Process,
                Effect::Power,
                Effect::Files("~".into()),
            ],
            api_key_file: String::new(),
        }
    }
}

impl Config {
    /// Chemin de la config : `LIBERTY_CONFIG`, sinon `/etc/liberty/liberty.toml`.
    pub fn path() -> PathBuf {
        std::env::var("LIBERTY_CONFIG")
            .ok()
            .filter(|s| !s.is_empty())
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("/etc/liberty/liberty.toml"))
    }

    /// Charge la config si elle existe, sinon les défauts (prudents).
    pub fn load() -> Self {
        match std::fs::read_to_string(Self::path()) {
            Ok(text) => Self::parse(&text),
            Err(_) => Self::default(),
        }
    }

    pub fn parse(text: &str) -> Self {
        let raw = parse_kv(text);
        let mut c = Self::default();
        if let Some(Value::Str(m)) = raw.get("model") {
            c.model = m.clone();
        }
        if let Some(Value::Int(n)) = raw.get("heartbeat_secs") {
            c.heartbeat_secs = (*n).max(30) as u64;
        }
        if let Some(Value::Str(p)) = raw.get("profile") {
            c.profile = p.clone();
        }
        if let Some(Value::Str(f)) = raw.get("credentials.api_key_file") {
            c.api_key_file = f.clone();
        }

        // Capacités : reconstruites entièrement depuis la config si la
        // section existe (sinon défauts). Rien n'est implicite.
        if raw.keys().any(|k| k.starts_with("capabilities.")) {
            let mut caps = Vec::new();
            if let Some(Value::List(paths)) = raw.get("capabilities.files") {
                caps.extend(paths.iter().map(|p| Effect::Files(p.clone())));
            }
            if let Some(Value::Bool(true)) = raw.get("capabilities.process") {
                caps.push(Effect::Process);
            }
            if let Some(Value::Bool(true)) = raw.get("capabilities.power") {
                caps.push(Effect::Power);
            }
            if let Some(Value::Bool(true)) = raw.get("capabilities.email") {
                caps.push(Effect::Email);
            }
            if let Some(Value::List(hosts)) = raw.get("capabilities.network") {
                caps.extend(hosts.iter().map(|h| Effect::Network(h.clone())));
            }
            c.capabilities = caps;
        }
        c
    }

    /// Le profil d'autonomie effectif (voir docs/AUTONOMY.md).
    pub fn policy(&self) -> Policy {
        match self.profile.as_str() {
            "confiance" => Policy {
                name: "Confiance",
                system: Autonomy::Autonomous,
                files: Autonomy::Autonomous,
                comm: Autonomy::Autonomous,
            },
            "manuel" => Policy {
                name: "Manuel",
                system: Autonomy::Manual,
                files: Autonomy::Manual,
                comm: Autonomy::Manual,
            },
            _ => Policy {
                name: "Prudent",
                system: Autonomy::Autonomous,
                files: Autonomy::Propose,
                comm: Autonomy::Manual,
            },
        }
    }

    /// La clé API depuis le fichier déclaré (si présent et lisible).
    pub fn read_api_key(&self) -> Option<String> {
        if self.api_key_file.is_empty() {
            return None;
        }
        std::fs::read_to_string(&self.api_key_file)
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
    }
}

// ---------------------------------------------------------------------------
// Mini-parseur clé/valeur (sous-ensemble de TOML)
// ---------------------------------------------------------------------------

enum Value {
    Str(String),
    Int(i64),
    Bool(bool),
    List(Vec<String>),
}

/// Aplati `[section]` + `clé = valeur` en `section.clé → Value`.
fn parse_kv(text: &str) -> HashMap<String, Value> {
    let mut out = HashMap::new();
    let mut section = String::new();
    for line in text.lines() {
        let line = line.split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            section = line[1..line.len() - 1].trim().to_string();
            continue;
        }
        let Some((k, v)) = line.split_once('=') else {
            continue;
        };
        let key = if section.is_empty() {
            k.trim().to_string()
        } else {
            format!("{section}.{}", k.trim())
        };
        out.insert(key, parse_value(v.trim()));
    }
    out
}

fn parse_value(v: &str) -> Value {
    if v == "true" {
        return Value::Bool(true);
    }
    if v == "false" {
        return Value::Bool(false);
    }
    if let Ok(n) = v.parse::<i64>() {
        return Value::Int(n);
    }
    if v.starts_with('[') && v.ends_with(']') {
        let items = v[1..v.len() - 1]
            .split(',')
            .map(|s| s.trim().trim_matches('"').to_string())
            .filter(|s| !s.is_empty())
            .collect();
        return Value::List(items);
    }
    Value::Str(v.trim_matches('"').to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"
# Liberty — configuration système
model = ""
heartbeat_secs = 120
profile = "confiance"

[capabilities]
files = ["~", "/tmp"]
process = true
power = true
email = false
network = ["api.anthropic.com"]

[credentials]
api_key_file = "/etc/liberty/anthropic.key"
"#;

    #[test]
    fn parses_the_reference_config() {
        let c = Config::parse(SAMPLE);
        assert_eq!(c.heartbeat_secs, 120);
        assert_eq!(c.profile, "confiance");
        assert_eq!(c.api_key_file, "/etc/liberty/anthropic.key");
        assert!(c.capabilities.contains(&Effect::Files("/tmp".into())));
        assert!(c.capabilities.contains(&Effect::Process));
        assert!(!c.capabilities.contains(&Effect::Email)); // false → absent
        assert!(c
            .capabilities
            .contains(&Effect::Network("api.anthropic.com".into())));
    }

    #[test]
    fn defaults_are_prudent_and_offline() {
        let c = Config::default();
        assert_eq!(c.profile, "prudent");
        // Pas de réseau ni de courriel par défaut : rien d'externe.
        assert!(!c.capabilities.iter().any(|e| e.is_external()));
    }

    #[test]
    fn capabilities_section_replaces_defaults_entirely() {
        let c = Config::parse("[capabilities]\nfiles = [\"~/Documents\"]\n");
        assert_eq!(c.capabilities, vec![Effect::Files("~/Documents".into())]);
    }

    #[test]
    fn heartbeat_has_a_sane_floor() {
        let c = Config::parse("heartbeat_secs = 1\n");
        assert_eq!(c.heartbeat_secs, 30);
    }
}
