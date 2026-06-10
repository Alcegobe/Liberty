//! La boucle agentique — l'esprit de Liberty au travail.
//!
//! Un tour = un appel à l'API Messages avec les outils de l'OS. Claude
//! observe (lecture seule, liste blanche), agit (chaque action passe par
//! `decide()` : capacités + autonomie + réversibilité), questionne, puis
//! conclut. La boucle est la même pour le démon (`libertyd`) et le shell
//! (`lish`) ; seule l'interface humaine change.
//!
//! Compilé avec `--features claude`.

use crate::brain::ClaudeBrain;
use crate::config::Config;
use crate::decision::{decide, Action, Decision};
use crate::effects::Effect;
use crate::executor;
use crate::journal::{Entry, Journal};
use crate::transport;
use serde_json::{json, Value};

/// Garde-fou : nombre maximal de tours d'outils par mission.
pub const MAX_TURNS: usize = 16;

/// Le pont vers l'humain. Le démon journalise et passe son chemin ; lish
/// pose vraiment la question dans le terminal.
pub trait Interface {
    /// Un humain est-il au bout du fil ? (lish : oui ; démon : non)
    fn present(&self) -> bool;
    /// Affichage d'une ligne de narration (ce que fait l'esprit).
    fn notify(&mut self, line: &str);
    /// Proposer une action à valider. `true` = l'humain approuve.
    fn approve(&mut self, name: &str, command: &str, effects: &str) -> bool;
    /// Poser une question. `None` = humain indisponible (mode démon).
    fn answer(&mut self, question: &str) -> Option<String>;
}

/// Le prompt système de l'esprit. Long et stable → prompt caching.
fn system_prompt(cfg: &Config) -> String {
    let caps = cfg
        .capabilities
        .iter()
        .map(|e| e.describe())
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        "Tu es libertyd, l'esprit de Liberty OS — un système d'exploitation \
         AI-first dont tu es la couche d'intelligence native. Tu n'es pas un \
         chatbot : tu es un service système. Tu observes, tu décides, tu agis \
         au nom de l'utilisateur, et tu rends compte.\n\
         \n\
         Règles de conduite :\n\
         1. OBSERVE avant d'agir : utilise l'outil observe (lecture seule) \
         pour vérifier l'état réel avant toute modification.\n\
         2. Déclare honnêtement chaque action via l'outil act : ses effets \
         (fichiers:<chemin>, processus, énergie, courriel, réseau:<hôte>), sa \
         réversibilité, et ta confiance calibrée (0..1). L'OS vérifie tes \
         déclarations contre les capacités accordées et bloque tout \
         dépassement — déclarer trop peu ne contourne rien, c'est juste un \
         refus de plus.\n\
         3. Privilégie le réversible : fournis une commande undo chaque fois \
         que c'est possible (déplacer vers une corbeille plutôt que rm, etc.).\n\
         4. Dans le doute, une question courte et précise via ask_user vaut \
         mieux qu'un acte hasardeux.\n\
         5. Termine toujours par l'outil done avec un bilan d'une ou deux \
         phrases. Sois sobre : pas d'emphase, des faits.\n\
         \n\
         Capacités actuellement accordées par l'utilisateur : {caps}.\n\
         Profil d'autonomie : {}.\n\
         Tu tournes sur un système de la famille Linux (sauf mention \
         contraire dans le rapport). Les commandes passent par sh -c.",
        cfg.profile
    )
}

fn tools() -> Value {
    json!([
        {
            "name": "observe",
            "description": "Exécuter une commande en LECTURE SEULE pour inspecter le système (df, ps, systemctl status, journalctl, cat, ls…). Liste blanche stricte : toute commande qui modifie quoi que ce soit sera refusée. Pas de pipes ni de redirections.",
            "input_schema": {
                "type": "object",
                "properties": {
                    "command": {"type": "string", "description": "Commande simple en lecture seule, ex. « df -h » ou « systemctl status nginx »"}
                },
                "required": ["command"]
            }
        },
        {
            "name": "act",
            "description": "Exécuter une action qui MODIFIE le système. L'OS la passera par sa boucle de décision (capacités, autonomie, réversibilité, confiance) et pourra l'exécuter, la proposer à l'utilisateur, ou la refuser. Le résultat t'indique ce qui s'est passé.",
            "input_schema": {
                "type": "object",
                "properties": {
                    "name": {"type": "string", "description": "Libellé court et humain de l'action"},
                    "command": {"type": "string", "description": "La commande shell exacte à exécuter"},
                    "undo": {"type": "string", "description": "Commande qui annule l'action, si elle existe"},
                    "effects": {"type": "array", "items": {"type": "string"}, "description": "Effets touchés : fichiers:<chemin>, processus, énergie, courriel, réseau:<hôte>"},
                    "reversible": {"type": "boolean"},
                    "affects_others": {"type": "boolean", "description": "true si l'action touche autrui (envoi, publication…)"},
                    "confidence": {"type": "number", "description": "Ta confiance calibrée, entre 0 et 1"}
                },
                "required": ["name", "command", "effects", "reversible", "affects_others", "confidence"]
            }
        },
        {
            "name": "ask_user",
            "description": "Poser à l'utilisateur une question courte et précise quand la décision lui appartient ou que ta confiance est insuffisante.",
            "input_schema": {
                "type": "object",
                "properties": {
                    "question": {"type": "string"},
                    "context": {"type": "string"}
                },
                "required": ["question"]
            }
        },
        {
            "name": "done",
            "description": "Conclure la mission avec un bilan bref et factuel.",
            "input_schema": {
                "type": "object",
                "properties": {
                    "summary": {"type": "string"}
                },
                "required": ["summary"]
            }
        }
    ])
}

/// Lance une mission : `messages` est l'historique persistant (lish garde le
/// fil de la session ; le démon repart à vide à chaque battement).
/// Retourne le bilan final de l'esprit.
pub fn run_mission(
    brain: &ClaudeBrain,
    cfg: &Config,
    journal: &Journal,
    iface: &mut dyn Interface,
    messages: &mut Vec<Value>,
    mission: &str,
    model: &str,
) -> Result<String, String> {
    messages.push(json!({"role": "user", "content": mission}));

    for _ in 0..MAX_TURNS {
        let body = json!({
            "model": model,
            "max_tokens": 16000,
            "thinking": {"type": "adaptive"},
            "system": [
                {"type": "text", "text": system_prompt(cfg), "cache_control": {"type": "ephemeral"}}
            ],
            "tools": tools(),
            "messages": messages,
        });

        let resp = transport::api_post(brain, &body)?;
        let content = resp["content"].clone();
        let stop = resp["stop_reason"].as_str().unwrap_or("");

        // Toujours renvoyer le contenu assistant tel quel (blocs thinking
        // compris) pour les tours suivants.
        messages.push(json!({"role": "assistant", "content": content}));

        // Narration : les blocs de texte de l'esprit.
        for b in content.as_array().into_iter().flatten() {
            if b["type"] == "text" {
                let t = b["text"].as_str().unwrap_or("").trim();
                if !t.is_empty() {
                    iface.notify(t);
                }
            }
        }

        if stop != "tool_use" {
            let text = collect_text(&content);
            return Ok(if text.is_empty() { "(fin de tour sans bilan)".into() } else { text });
        }

        let mut results = Vec::new();
        let mut finished: Option<String> = None;
        for b in content.as_array().into_iter().flatten() {
            if b["type"] != "tool_use" {
                continue;
            }
            let id = b["id"].as_str().unwrap_or("").to_string();
            let name = b["name"].as_str().unwrap_or("");
            let input = &b["input"];
            let outcome = match name {
                "observe" => handle_observe(input, journal, iface),
                "act" => handle_act(input, cfg, journal, iface),
                "ask_user" => handle_ask(input, journal, iface),
                "done" => {
                    let s = input["summary"].as_str().unwrap_or("Terminé.").to_string();
                    finished = Some(s);
                    "Bilan enregistré.".to_string()
                }
                other => format!("Outil inconnu : {other}"),
            };
            results.push(json!({
                "type": "tool_result",
                "tool_use_id": id,
                "content": outcome,
            }));
        }
        messages.push(json!({"role": "user", "content": results}));

        if let Some(summary) = finished {
            journal.record(&Entry {
                kind: "bilan".into(),
                name: "mission terminée".into(),
                detail: summary.clone(),
                command: String::new(),
                undo: String::new(),
            });
            return Ok(summary);
        }
    }
    Err(format!("mission interrompue après {MAX_TURNS} tours (garde-fou)"))
}

fn collect_text(content: &Value) -> String {
    content
        .as_array()
        .into_iter()
        .flatten()
        .filter(|b| b["type"] == "text")
        .filter_map(|b| b["text"].as_str())
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string()
}

fn handle_observe(input: &Value, journal: &Journal, iface: &mut dyn Interface) -> String {
    let cmd = input["command"].as_str().unwrap_or("");
    iface.notify(&format!("  👁  observe : {cmd}"));
    let r = executor::observe(cmd, journal);
    if r.output.is_empty() {
        "(sortie vide)".into()
    } else {
        r.output
    }
}

fn handle_ask(input: &Value, journal: &Journal, iface: &mut dyn Interface) -> String {
    let q = input["question"].as_str().unwrap_or("?");
    match iface.answer(q) {
        Some(a) => format!("Réponse de l'utilisateur : {a}"),
        None => {
            journal.record(&Entry {
                kind: "question".into(),
                name: "en attente de l'utilisateur".into(),
                detail: q.into(),
                command: String::new(),
                undo: String::new(),
            });
            "L'utilisateur n'est pas disponible (mode autonome). Question \
             consignée au journal ; n'agis pas sur ce point et passe à la suite."
                .into()
        }
    }
}

/// Convertit l'entrée de l'outil `act` puis applique la boucle de décision.
fn handle_act(input: &Value, cfg: &Config, journal: &Journal, iface: &mut dyn Interface) -> String {
    let effects: Vec<Effect> = input["effects"]
        .as_array()
        .map(|a| a.iter().filter_map(|e| e.as_str()).map(Effect::parse).collect())
        .unwrap_or_default();
    let action = Action {
        name: input["name"].as_str().unwrap_or("(action)").to_string(),
        trigger: "initié par l'esprit".to_string(),
        domain: crate::autonomy::infer_domain(&effects),
        effects,
        reversible: input["reversible"].as_bool().unwrap_or(false),
        affects_others: input["affects_others"].as_bool().unwrap_or(false),
        confidence: input["confidence"].as_f64().unwrap_or(0.0),
        question: String::new(),
    };
    let command = input["command"].as_str().unwrap_or("").to_string();
    let undo = input["undo"].as_str().unwrap_or("").to_string();
    if command.is_empty() {
        return "Refusé : aucune commande fournie.".into();
    }

    let level = cfg.policy().level(action.domain);
    match decide(&action, level, &cfg.capabilities) {
        Decision::Refused(e) => {
            journal.record(&Entry {
                kind: "refus".into(),
                name: action.name.clone(),
                detail: format!("capacité « {} » non accordée", e.describe()),
                command,
                undo: String::new(),
            });
            iface.notify(&format!("  🛡  bloqué par l'OS : {}", action.name));
            format!(
                "REFUSÉ par l'OS : l'effet « {} » n'est pas couvert par les \
                 capacités accordées. N'insiste pas ; signale-le dans ton bilan.",
                e.describe()
            )
        }
        Decision::Suggest => {
            journal.record(&Entry {
                kind: "suggestion".into(),
                name: action.name.clone(),
                detail: "mode manuel : suggestion consignée, non exécutée".into(),
                command,
                undo,
            });
            iface.notify(&format!("  💡 suggestion (mode manuel) : {}", action.name));
            "Mode manuel : suggestion consignée au journal, non exécutée.".into()
        }
        Decision::Ask => {
            "Confiance insuffisante pour agir (< 0,6). Pose une question \
             précise via ask_user, ou renonce."
                .into()
        }
        Decision::Propose => match iface_approve(iface, &action, &command) {
            Some(true) => run_and_report(&action.name, &command, &undo, journal, iface),
            Some(false) => {
                journal.record(&Entry {
                    kind: "refus".into(),
                    name: action.name.clone(),
                    detail: "refusé par l'utilisateur".into(),
                    command,
                    undo: String::new(),
                });
                "L'utilisateur a refusé cette action.".into()
            }
            None => {
                journal.record(&Entry {
                    kind: "proposition".into(),
                    name: action.name.clone(),
                    detail: "en attente de validation (mode autonome)".into(),
                    command,
                    undo,
                });
                iface.notify(&format!("  🟡 proposé, en attente : {}", action.name));
                "Utilisateur indisponible : action consignée comme PROPOSITION \
                 au journal, non exécutée. Passe à la suite."
                    .into()
            }
        },
        Decision::Silent => run_and_report(&action.name, &command, &undo, journal, iface),
        Decision::Undo(secs) => {
            iface.notify(&format!(
                "  🟢 action externe annulable {secs} s : {}",
                action.name
            ));
            run_and_report(&action.name, &command, &undo, journal, iface)
        }
    }
}

/// Propose à l'humain ; `None` si personne n'est au bout du fil (démon).
fn iface_approve(iface: &mut dyn Interface, a: &Action, command: &str) -> Option<bool> {
    if !iface.present() {
        return None;
    }
    let effects = a.effects_summary();
    Some(iface.approve(&a.name, command, &effects))
}

fn run_and_report(
    name: &str,
    command: &str,
    undo: &str,
    journal: &Journal,
    iface: &mut dyn Interface,
) -> String {
    iface.notify(&format!("  ⚙  exécute : {command}"));
    let r = executor::execute(name, command, undo, journal);
    if r.success {
        if r.output.is_empty() {
            "OK (aucune sortie).".into()
        } else {
            format!("OK :\n{}", r.output)
        }
    } else {
        format!("ÉCHEC :\n{}", r.output)
    }
}
