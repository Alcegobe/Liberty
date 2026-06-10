//! Le `Brain` — l'esprit de Liberty, backend interchangeable.
//!
//! En production, l'esprit est **Claude** (`claude-opus-4-8`) via l'API
//! Anthropic, authentifié par le compte Anthropic de la session (OAuth) ou
//! une clé API. En développement hors-ligne, `SimulatedBrain` fournit des
//! situations réalistes. Voir docs/AI.md.

use crate::autonomy::Domain;
use crate::decision::Action;
use crate::effects::Effect;

/// L'interface de l'esprit : à partir de l'état du système, il *initie* des
/// actions (l'inversion) avec son jugement calibré (confiance, questions).
pub trait Brain {
    fn name(&self) -> String;
    /// Observe le système et propose des actions. C'est l'IA qui initie.
    fn assess(&self) -> Vec<Action>;
}

// ---------------------------------------------------------------------------
// Identifiants — « se connecter avec son compte Anthropic »
// ---------------------------------------------------------------------------

/// Ordre de résolution aligné sur l'écosystème Anthropic : clé API d'abord
/// (elle masque tout le reste — il faut le signaler), puis jeton OAuth.
/// À terme : profil du trousseau chiffré de Liberty.
pub enum Credentials {
    /// Clé API → en-tête `x-api-key`.
    ApiKey(String),
    /// Jeton OAuth (compte Anthropic) → `Authorization: Bearer` + en-tête
    /// `anthropic-beta: oauth-2025-04-20`.
    OAuth(String),
    None,
}

pub fn resolve_credentials() -> Credentials {
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
    Credentials::None
}

// ---------------------------------------------------------------------------
// ClaudeBrain — la vraie intégration (forme exacte des requêtes API)
// ---------------------------------------------------------------------------

pub const CLAUDE_MODEL: &str = "claude-opus-4-8";
pub const API_URL: &str = "https://api.anthropic.com/v1/messages";

/// Le prompt système de l'esprit. Stable et long → destiné au prompt caching
/// (`cache_control: ephemeral` sur ce bloc) ; le contexte volatil arrive
/// après, dans les messages.
const SYSTEM_PROMPT: &str = "Tu es libertyd, l'esprit de Liberty OS. Tu observes \
l'état du système et tu INITIES des actions au nom de l'utilisateur (il ne te \
prompte pas : tu agis, tu proposes, ou tu poses une question précise). Pour \
chaque situation, évalue ta confiance honnêtement : dans le doute, pose une \
question courte et bien cadrée plutôt que d'agir. Déclare exhaustivement les \
effets de chaque action ; l'OS les vérifiera contre les capacités accordées \
et bloquera tout dépassement.";

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

    /// En-têtes HTTP exacts pour `POST /v1/messages`, selon le mode d'auth.
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

    /// Corps de la requête : le bus d'intents de Liberty exposé comme outils
    /// (tool use), thinking adaptatif, prompt système cachable.
    /// Exercé par les tests ; sera câblé au transport HTTP en phase réseau.
    #[allow(dead_code)]
    pub fn request_body(&self, situation_report: &str) -> String {
        let escaped = situation_report.replace('\\', "\\\\").replace('"', "\\\"");
        format!(
            r#"{{
  "model": "{CLAUDE_MODEL}",
  "max_tokens": 16000,
  "thinking": {{"type": "adaptive"}},
  "system": [
    {{"type": "text", "text": "{SYSTEM_PROMPT}", "cache_control": {{"type": "ephemeral"}}}}
  ],
  "tools": [
    {{
      "name": "act",
      "description": "Exécuter une action système. À n'utiliser que si ta confiance est élevée et l'action réversible ou anodine. L'OS vérifie les effets contre les capacités accordées.",
      "input_schema": {{
        "type": "object",
        "properties": {{
          "name": {{"type": "string"}},
          "effects": {{"type": "array", "items": {{"type": "string"}}, "description": "Effets touchés, ex. fichiers:~/Téléchargements, processus, courriel"}},
          "reversible": {{"type": "boolean"}},
          "affects_others": {{"type": "boolean"}},
          "confidence": {{"type": "number"}}
        }},
        "required": ["name", "effects", "reversible", "affects_others", "confidence"]
      }}
    }},
    {{
      "name": "ask_user",
      "description": "Poser à l'utilisateur une question courte et précise quand la décision lui appartient ou que ta confiance est insuffisante.",
      "input_schema": {{
        "type": "object",
        "properties": {{
          "question": {{"type": "string"}},
          "context": {{"type": "string"}}
        }},
        "required": ["question"]
      }}
    }}
  ],
  "messages": [
    {{"role": "user", "content": "Rapport de situation (pré-filtré localement) :\n{}"}}
  ]
}}"#,
            escaped
        )
    }
}

impl Brain for ClaudeBrain {
    fn name(&self) -> String {
        format!("Claude ({CLAUDE_MODEL})")
    }

    fn assess(&self) -> Vec<Action> {
        // Le transport HTTP réel arrive avec la phase réseau ; ce prototype
        // construit des requêtes exactes (headers/body, testés) mais ne les
        // envoie pas encore. Aucune action n'est inventée à la place.
        Vec::new()
    }
}

// ---------------------------------------------------------------------------
// SimulatedBrain — développement hors-ligne
// ---------------------------------------------------------------------------

pub struct SimulatedBrain;

impl Brain for SimulatedBrain {
    fn name(&self) -> String {
        "Simulation (dev hors-ligne)".into()
    }

    fn assess(&self) -> Vec<Action> {
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

    #[test]
    fn request_body_targets_current_model_with_adaptive_thinking() {
        let b = ClaudeBrain::new(Credentials::ApiKey("k".into()));
        let body = b.request_body("disque plein");
        assert!(body.contains(r#""model": "claude-opus-4-8""#));
        assert!(body.contains(r#""thinking": {"type": "adaptive"}"#));
        assert!(body.contains(r#""cache_control": {"type": "ephemeral"}"#));
        assert!(body.contains("ask_user"));
        assert!(body.contains("disque plein"));
    }

    #[test]
    fn report_quotes_are_escaped() {
        let b = ClaudeBrain::new(Credentials::ApiKey("k".into()));
        let body = b.request_body(r#"process "render" emballé"#);
        assert!(body.contains(r#"process \"render\" emballé"#));
    }
}
