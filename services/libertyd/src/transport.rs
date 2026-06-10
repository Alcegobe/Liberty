//! Transport réseau vers Anthropic — la vraie connexion de l'esprit.
//!
//! Compilé seulement avec `--features claude` (sinon le build par défaut
//! reste sans dépendance). Au démarrage de l'OS, Liberty :
//!   1. vérifie la connexion au compte Anthropic (liste des modèles),
//!   2. choisit le modèle le plus capable accessible,
//!   3. envoie un rapport de situation et récupère les décisions de Claude
//!      (tool use), converties en `Action` pour la boucle `decide()`.

use crate::autonomy::{infer_domain, Domain};
use crate::brain::{ClaudeBrain, API_URL};
use crate::decision::Action;
use crate::effects::Effect;

const MODELS_URL: &str = "https://api.anthropic.com/v1/models";

/// Étape de démarrage : connexion + première évaluation par Claude.
pub fn startup(brain: &ClaudeBrain) -> Result<Vec<Action>, String> {
    let models = list_models(brain)?;
    println!("✅ Connecté à Anthropic — {} modèle(s) accessibles.", models.len());
    let chosen = crate::model::select_available(&models);
    println!("   Modèle retenu : {chosen}");

    // Rapport de situation de démonstration (pré-filtré localement). En vrai,
    // ce texte serait produit par les réflexes locaux à partir des capteurs.
    let report = "Capteurs système : CPU à 96 °C, le processus 'render' tourne en \
boucle. Disque à 95 % — ~/.cache contient 8 Go régénérables. 14 fichiers \
identiques dans ~/Téléchargements (corbeille versionnée active). Boîte mail : \
1 message anodin de confirmation d'un fournisseur ; 1 message important du \
patron demandant une date d'engagement de livraison.";

    println!("   Envoi du rapport de situation à Claude…\n");
    assess(brain, report)
}

/// Vérifie la connexion et renvoie les identifiants des modèles accessibles.
pub fn list_models(brain: &ClaudeBrain) -> Result<Vec<String>, String> {
    let mut req = ureq::get(MODELS_URL);
    for (k, v) in brain.headers() {
        req = req.set(&k, &v);
    }
    let resp = req.call().map_err(stringify_err)?;
    let v: serde_json::Value = resp.into_json().map_err(|e| e.to_string())?;
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

/// Envoie un rapport de situation et convertit les `tool_use` de Claude en
/// actions pour la boucle de décision.
pub fn assess(brain: &ClaudeBrain, report: &str) -> Result<Vec<Action>, String> {
    let body = brain.request_body(report);
    let mut req = ureq::post(API_URL);
    for (k, v) in brain.headers() {
        req = req.set(&k, &v);
    }
    let resp = req.send_string(&body).map_err(stringify_err)?;
    let v: serde_json::Value = resp.into_json().map_err(|e| e.to_string())?;
    Ok(parse_actions(&v))
}

fn parse_actions(v: &serde_json::Value) -> Vec<Action> {
    let mut out = Vec::new();
    let Some(blocks) = v["content"].as_array() else {
        return out;
    };
    for b in blocks {
        if b["type"] != "tool_use" {
            continue;
        }
        let input = &b["input"];
        match b["name"].as_str().unwrap_or("") {
            "act" => {
                let effects: Vec<Effect> = input["effects"]
                    .as_array()
                    .map(|a| a.iter().filter_map(|e| e.as_str()).map(Effect::parse).collect())
                    .unwrap_or_default();
                let domain = infer_domain(&effects);
                out.push(Action {
                    name: input["name"].as_str().unwrap_or("(action)").to_string(),
                    trigger: "initié par Claude".to_string(),
                    domain,
                    effects,
                    reversible: input["reversible"].as_bool().unwrap_or(false),
                    affects_others: input["affects_others"].as_bool().unwrap_or(false),
                    confidence: input["confidence"].as_f64().unwrap_or(0.5),
                    question: String::new(),
                });
            }
            "ask_user" => out.push(Action {
                name: "Question de Claude".to_string(),
                trigger: input["context"]
                    .as_str()
                    .unwrap_or("décision qui t'appartient")
                    .to_string(),
                domain: Domain::System,
                effects: Vec::new(),
                reversible: false,
                affects_others: false,
                confidence: 0.0, // force une consultation
                question: input["question"].as_str().unwrap_or("?").to_string(),
            }),
            _ => {}
        }
    }
    out
}

fn stringify_err(e: ureq::Error) -> String {
    match e {
        ureq::Error::Status(code, resp) => {
            format!("HTTP {code} — {}", resp.into_string().unwrap_or_default())
        }
        ureq::Error::Transport(t) => format!("réseau : {t}"),
    }
}
