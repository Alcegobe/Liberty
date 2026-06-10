//! Le `Brain` — l'identité de l'esprit et ses identifiants.
//!
//! En production, l'esprit est **Claude** (Fable 5 par défaut, et chaque
//! modèle plus capable dès qu'il existe) via l'API Anthropic, authentifié par
//! le compte Anthropic (OAuth) ou une clé API. En développement hors-ligne,
//! `SimulatedBrain` fournit des situations réalistes. Voir docs/AI.md.

use crate::autonomy::Domain;
use crate::config::Config;
use crate::decision::Action;
use crate::effects::Effect;
use crate::model;

// ---------------------------------------------------------------------------
// Identifiants — « se connecter avec son compte Anthropic »
// ---------------------------------------------------------------------------

/// Ordre de résolution aligné sur l'écosystème Anthropic : clé API d'abord
/// (env), puis jeton OAuth (env), puis le fichier de clé déclaré dans la
/// config système (`/etc/liberty/anthropic.key` par convention).
pub enum Credentials {
    /// Clé API → en-tête `x-api-key`.
    ApiKey(String),
    /// Jeton OAuth (compte Anthropic) → `Authorization: Bearer` + en-tête
    /// `anthropic-beta: oauth-2025-04-20`.
    OAuth(String),
    None,
}

pub fn resolve_credentials(cfg: &Config) -> Credentials {
    if let Ok(k) = std::env::var("ANTHROPIC_API_KEY") {
        if !k.is_empty() {
            return Credentials::ApiKey(k);
        }
    }
    if let Ok(t) = std::env::var("ANTHROPIC_AUTH_TOKEN") {
        if !t.is_empty() {
            return Credentials::OAuth(t);
        }
    }
    if let Some(k) = cfg.read_api_key() {
        return Credentials::ApiKey(k);
    }
    Credentials::None
}

// ---------------------------------------------------------------------------
// ClaudeBrain — l'esprit de production
// ---------------------------------------------------------------------------

pub struct ClaudeBrain {
    creds: Credentials,
}

impl ClaudeBrain {
    pub fn new(creds: Credentials) -> Self {
        Self { creds }
    }

    pub fn ready(&self) -> bool {
        !matches!(self.creds, Credentials::None)
    }

    pub fn name(&self) -> String {
        format!("Claude ({})", model::default_model())
    }

    /// En-têtes HTTP exacts pour l'API Anthropic, selon le mode d'auth.
    pub fn headers(&self) -> Vec<(String, String)> {
        let mut h = vec![
            ("content-type".into(), "application/json".into()),
            ("anthropic-version".into(), "2023-06-01".into()),
        ];
        match &self.creds {
            Credentials::ApiKey(k) => h.push(("x-api-key".into(), k.clone())),
            Credentials::OAuth(t) => {
                h.push(("authorization".into(), format!("Bearer {t}")));
                h.push(("anthropic-beta".into(), "oauth-2025-04-20".into()));
            }
            Credentials::None => {}
        }
        h
    }
}

// ---------------------------------------------------------------------------
// SimulatedBrain — développement hors-ligne (mode --demo)
// ---------------------------------------------------------------------------

pub struct SimulatedBrain;

impl SimulatedBrain {
    pub fn name(&self) -> String {
        "Simulation (dev hors-ligne)".into()
    }

    pub fn assess(&self) -> Vec<Action> {
        vec![
            Action {
                name: "Brider un processus emballé".into(),
                trigger: "CPU à 98 °C — le process « render » part en boucle".into(),
                domain: Domain::System,
                effects: vec![Effect::Process, Effect::Power],
                reversible: true,
                affects_others: false,
                confidence: 0.96,
                question: String::new(),
            },
            Action {
                name: "Vider 8 Go de cache obsolète".into(),
                trigger: "disque presque plein, cache régénérable".into(),
                domain: Domain::System,
                effects: vec![Effect::Files("~/.cache".into())],
                reversible: true,
                affects_others: false,
                confidence: 0.92,
                question: String::new(),
            },
            Action {
                name: "Supprimer 14 doublons".into(),
                trigger: "14 fichiers identiques dans Téléchargements (corbeille versionnée)"
                    .into(),
                domain: Domain::Files,
                effects: vec![Effect::Files("~/Téléchargements".into())],
                reversible: true,
                affects_others: false,
                confidence: 0.88,
                question: String::new(),
            },
            Action {
                name: "Répondre « bien reçu, merci »".into(),
                trigger: "mail anodin de confirmation d'un fournisseur".into(),
                domain: Domain::Communication,
                effects: vec![Effect::Email],
                reversible: true, // délai d'envoi annulable
                affects_others: true,
                confidence: 0.90,
                question: String::new(),
            },
            Action {
                name: "Répondre à ton patron sur le délai".into(),
                trigger: "mail important — l'engagement de date t'appartient".into(),
                domain: Domain::Communication,
                effects: vec![Effect::Email],
                reversible: true,
                affects_others: true,
                confidence: 0.55, // doute → l'IA préfère demander
                question: "On tient le 15, ou je demande 3 jours de plus ?".into(),
            },
            Action {
                name: "Envoyer des statistiques d'usage".into(),
                trigger: "un service voudrait remonter de la télémétrie".into(),
                domain: Domain::System,
                effects: vec![Effect::Network("telemetry.example.com".into())],
                reversible: false,
                affects_others: true,
                confidence: 0.99, // très sûre... mais sans la capacité réseau
                question: String::new(),
            },
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn api_key_uses_x_api_key_header() {
        let b = ClaudeBrain::new(Credentials::ApiKey("sk-ant-test".into()));
        let h = b.headers();
        assert!(h.iter().any(|(k, v)| k == "x-api-key" && v == "sk-ant-test"));
        assert!(!h.iter().any(|(k, _)| k == "authorization"));
    }

    #[test]
    fn oauth_uses_bearer_plus_beta_header() {
        let b = ClaudeBrain::new(Credentials::OAuth("tok".into()));
        let h = b.headers();
        assert!(h.iter().any(|(k, v)| k == "authorization" && v == "Bearer tok"));
        assert!(h
            .iter()
            .any(|(k, v)| k == "anthropic-beta" && v == "oauth-2025-04-20"));
        assert!(!h.iter().any(|(k, _)| k == "x-api-key"));
    }
}
