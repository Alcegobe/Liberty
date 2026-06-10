//! Mode « compte Anthropic » — l'esprit via Claude Code.
//!
//! Quand aucune clé API n'est configurée, Liberty cherche le CLI `claude`
//! (Claude Code) : l'utilisateur s'y connecte une fois avec son compte
//! Anthropic (abonnement Claude), et l'esprit passe par lui. Deux usages :
//!
//!  - `lish`     : session interactive Claude Code, semée de l'intention —
//!    les validations d'actions passent par l'UI de Claude Code.
//!  - `libertyd` : battement de cœur en mode `-p` (headless), outils bridés
//!    selon le profil d'autonomie (lecture seule en « prudent »).
//!
//! Trade-off assumé : dans ce mode, le garde-fou fin de Liberty (capacités
//! par effet) est remplacé par le système de permissions de Claude Code.
//! Le mode clé API garde la boucle `decide()` complète.

use crate::config::Config;
use std::path::PathBuf;
use std::process::Command;

/// Cherche le CLI `claude` : PATH, puis les emplacements d'installation
/// usuels (le shell de session lish ne source pas ~/.profile).
pub fn find() -> Option<PathBuf> {
    let mut candidates: Vec<PathBuf> = vec![PathBuf::from("claude")];
    if let Ok(home) = std::env::var("HOME") {
        candidates.insert(0, PathBuf::from(&home).join(".local/bin/claude"));
    }
    candidates.push(PathBuf::from("/usr/local/bin/claude"));
    for c in candidates {
        if Command::new(&c)
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
        {
            return Some(c);
        }
    }
    None
}

/// Le contexte Liberty injecté dans le prompt système de Claude Code.
fn system_prompt(cfg: &Config) -> String {
    let caps = cfg
        .capabilities
        .iter()
        .map(|e| e.describe())
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        "Tu opères comme l'esprit de Liberty OS (libertyd), connecté via le \
         compte Anthropic de l'utilisateur. Tu n'es pas dans un dépôt de code : \
         tu administres cette machine. Profil d'autonomie Liberty : {} ; \
         capacités accordées : {caps}. Respecte-les : n'agis que dans ces \
         limites, privilégie les actions réversibles, et sois sobre et factuel \
         dans tes comptes rendus.",
        cfg.profile
    )
}

/// Session interactive (lish) : Claude Code prend le terminal, l'utilisateur
/// valide les actions dans son UI. `cont` = reprendre le fil de la session.
pub fn interactive(cli: &PathBuf, cfg: &Config, intent: &str, cont: bool) -> Result<(), String> {
    let mut cmd = Command::new(cli);
    cmd.arg(intent)
        .arg("--append-system-prompt")
        .arg(system_prompt(cfg));
    if cont {
        cmd.arg("--continue");
    }
    let status = cmd.status().map_err(|e| format!("lancement de claude : {e}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("claude s'est terminé avec le code {status}"))
    }
}

/// Battement headless (démon) : `claude -p`, outils selon le profil.
/// En « prudent »/« manuel », lecture seule : l'esprit constate et propose ;
/// en « confiance », il peut agir (Bash autorisé).
pub fn heartbeat(cli: &PathBuf, cfg: &Config, mission: &str) -> Result<String, String> {
    let allowed = match cfg.profile.as_str() {
        "confiance" => "Bash",
        _ => {
            "Bash(df:*),Bash(du:*),Bash(free:*),Bash(ps:*),Bash(ls:*),Bash(cat:*),\
             Bash(uptime:*),Bash(uname:*),Bash(systemctl status:*),Bash(journalctl:*)"
        }
    };
    let out = Command::new(cli)
        .arg("-p")
        .arg(mission)
        .arg("--output-format")
        .arg("text")
        .arg("--append-system-prompt")
        .arg(system_prompt(cfg))
        .arg("--allowedTools")
        .arg(allowed)
        .output()
        .map_err(|e| format!("lancement de claude : {e}"))?;
    let text = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if out.status.success() {
        Ok(text)
    } else {
        let err = String::from_utf8_lossy(&out.stderr).trim().to_string();
        Err(if err.is_empty() { text } else { err })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn system_prompt_carries_profile_and_caps() {
        let cfg = Config::default();
        let p = system_prompt(&cfg);
        assert!(p.contains("prudent"));
        assert!(p.contains("processus"));
    }
}
