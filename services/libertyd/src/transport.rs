//! Transport réseau vers Anthropic — la connexion physique de l'esprit.
//!
//! Compilé seulement avec `--features claude` (le build par défaut reste
//! hors-ligne). Deux primitives : `api_post` (Messages) et `list_models`
//! (vérification de connexion + découverte des modèles accessibles).

use crate::brain::ClaudeBrain;
use crate::config::Config;
use crate::model;
use serde_json::Value;
use std::time::Duration;

pub const API_URL: &str = "https://api.anthropic.com/v1/messages";
const MODELS_URL: &str = "https://api.anthropic.com/v1/models";

/// POST /v1/messages, avec une reprise simple sur les erreurs transitoires
/// (429/5xx) — l'esprit d'un OS ne s'arrête pas au premier hoquet.
pub fn api_post(brain: &ClaudeBrain, body: &Value) -> Result<Value, String> {
    let payload = body.to_string();
    let mut last_err = String::new();
    for attempt in 0..3 {
        if attempt > 0 {
            std::thread::sleep(Duration::from_secs(2 << attempt));
        }
        let mut req = ureq::post(API_URL).timeout(Duration::from_secs(600));
        for (k, v) in brain.headers() {
            req = req.set(&k, &v);
        }
        match req.send_string(&payload) {
            Ok(resp) => return resp.into_json().map_err(|e| e.to_string()),
            Err(ureq::Error::Status(code, resp)) if code == 429 || code >= 500 => {
                last_err = format!("HTTP {code} — {}", resp.into_string().unwrap_or_default());
            }
            Err(e) => return Err(stringify_err(e)),
        }
    }
    Err(format!("après 3 tentatives : {last_err}"))
}

/// Vérifie la connexion et renvoie les identifiants des modèles accessibles.
pub fn list_models(brain: &ClaudeBrain) -> Result<Vec<String>, String> {
    let mut req = ureq::get(MODELS_URL).timeout(Duration::from_secs(30));
    for (k, v) in brain.headers() {
        req = req.set(&k, &v);
    }
    let resp = req.call().map_err(stringify_err)?;
    let v: Value = resp.into_json().map_err(|e| e.to_string())?;
    let ids = v["data"]
        .as_array()
        .map(|a| {
            a.iter()
                .filter_map(|m| m["id"].as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();
    Ok(ids)
}

/// Le modèle effectif : forcé par la config / `LIBERTY_MODEL`, sinon le plus
/// capable réellement accessible au compte. C'est ainsi que Liberty profite
/// automatiquement de chaque nouveau Claude.
pub fn pick_model(brain: &ClaudeBrain, cfg: &Config) -> Result<String, String> {
    if !cfg.model.is_empty() {
        return Ok(cfg.model.clone());
    }
    if let Ok(forced) = std::env::var("LIBERTY_MODEL") {
        if !forced.is_empty() {
            return Ok(forced);
        }
    }
    let models = list_models(brain)?;
    Ok(model::select_available(&models))
}

fn stringify_err(e: ureq::Error) -> String {
    match e {
        ureq::Error::Status(code, resp) => {
            format!("HTTP {code} — {}", resp.into_string().unwrap_or_default())
        }
        ureq::Error::Transport(t) => format!("réseau : {t}"),
    }
}
