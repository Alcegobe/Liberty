//! La boucle de décision — le filet de sécurité de l'OS sous l'IA.
//!
//! L'évaluation (confiance, formulation des questions) vient du `Brain`
//! (Claude en production, simulation en dev). `decide()` est la couche que
//! l'IA ne contrôle PAS : capacités, niveaux d'autonomie, règle
//! réversible/local. Voir docs/AUTONOMY.md et docs/AI.md.

use crate::autonomy::{Autonomy, Domain};
use crate::effects::{granted, Effect};

/// En dessous de cette confiance, l'IA consulte l'humain au lieu d'agir.
pub const ASK_THRESHOLD: f64 = 0.6;
/// Confiance minimale pour automatiser une action externe (avec annulation).
pub const EXTERNAL_AUTO_THRESHOLD: f64 = 0.85;
/// Fenêtre d'annulation des actions externes automatisées, en secondes.
pub const UNDO_WINDOW_SECS: u32 = 30;

/// Une action que l'IA a *elle-même* décidé de proposer (l'inversion).
pub struct Action {
    pub name: String,
    pub trigger: String, // pourquoi l'IA a initié ceci
    pub domain: Domain,
    pub effects: Vec<Effect>,
    pub reversible: bool,
    pub affects_others: bool,
    pub confidence: f64,  // jugement calibré du Brain (0..1)
    pub question: String, // question précise si l'humain doit trancher
}

impl Action {
    pub fn effects_summary(&self) -> String {
        self.effects
            .iter()
            .map(|e| e.describe())
            .collect::<Vec<_>>()
            .join(", ")
    }
}

pub enum Decision {
    Refused(Effect), // bloqué par les capacités (garde-fou OS)
    Silent,          // agit seule, en silence
    Undo(u32),       // agit, mais annulable pendant N secondes
    Propose,         // prépare et attend la validation
    Ask,             // remonte une question précise
    Suggest,         // mode manuel : se contente de suggérer
}

/// Le cœur : décider quoi faire d'une action selon les capacités, le niveau
/// d'autonomie et le jugement de l'IA. L'ordre des règles est significatif :
/// les capacités priment sur tout — y compris sur le mode manuel — parce
/// qu'une suggestion d'action interdite serait déjà une fuite d'intention.
pub fn decide(a: &Action, level: Autonomy, caps: &[Effect]) -> Decision {
    for e in &a.effects {
        if !granted(caps, e) {
            return Decision::Refused(e.clone());
        }
    }

    if level == Autonomy::Manual {
        return Decision::Suggest;
    }

    // Jugement calibré : dans le doute, une question précise plutôt qu'un acte.
    if a.confidence < ASK_THRESHOLD {
        return Decision::Ask;
    }

    let external = a.affects_others || a.effects.iter().any(Effect::is_external);

    if !external {
        if a.reversible && level == Autonomy::Autonomous {
            Decision::Silent
        } else {
            Decision::Propose
        }
    } else {
        // Externe : automatisable *grâce* à la réversibilité (fenêtre
        // d'annulation), à condition d'une confiance élevée et du mode 🟢.
        if a.reversible && a.confidence >= EXTERNAL_AUTO_THRESHOLD && level == Autonomy::Autonomous
        {
            Decision::Undo(UNDO_WINDOW_SECS)
        } else {
            Decision::Propose
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn action(
        domain: Domain,
        effects: Vec<Effect>,
        reversible: bool,
        affects_others: bool,
        confidence: f64,
    ) -> Action {
        Action {
            name: "test".into(),
            trigger: "test".into(),
            domain,
            effects,
            reversible,
            affects_others,
            confidence,
            question: "?".into(),
        }
    }

    fn full_caps() -> Vec<Effect> {
        vec![
            Effect::Process,
            Effect::Power,
            Effect::Files("~".into()),
            Effect::Email,
        ]
    }

    #[test]
    fn capabilities_override_everything_even_high_confidence() {
        let a = action(
            Domain::System,
            vec![Effect::Network("telemetry.example.com".into())],
            false,
            true,
            0.99,
        );
        assert!(matches!(
            decide(&a, Autonomy::Autonomous, &full_caps()),
            Decision::Refused(_)
        ));
    }

    #[test]
    fn reversible_local_is_silent_when_autonomous() {
        let a = action(Domain::System, vec![Effect::Process], true, false, 0.9);
        assert!(matches!(
            decide(&a, Autonomy::Autonomous, &full_caps()),
            Decision::Silent
        ));
    }

    #[test]
    fn low_confidence_always_asks_regardless_of_autonomy() {
        let a = action(Domain::Communication, vec![Effect::Email], true, true, 0.5);
        assert!(matches!(
            decide(&a, Autonomy::Autonomous, &full_caps()),
            Decision::Ask
        ));
    }

    #[test]
    fn external_reversible_high_confidence_gets_undo_window() {
        let a = action(Domain::Communication, vec![Effect::Email], true, true, 0.9);
        assert!(matches!(
            decide(&a, Autonomy::Autonomous, &full_caps()),
            Decision::Undo(UNDO_WINDOW_SECS)
        ));
    }

    #[test]
    fn external_irreversible_is_proposed_never_silent() {
        let a = action(Domain::Communication, vec![Effect::Email], false, true, 0.99);
        assert!(matches!(
            decide(&a, Autonomy::Autonomous, &full_caps()),
            Decision::Propose
        ));
    }

    #[test]
    fn manual_mode_only_suggests() {
        let a = action(Domain::Files, vec![Effect::Files("~/x".into())], true, false, 0.95);
        assert!(matches!(
            decide(&a, Autonomy::Manual, &full_caps()),
            Decision::Suggest
        ));
    }

    #[test]
    fn propose_level_never_acts_silently() {
        let a = action(Domain::Files, vec![Effect::Files("~/x".into())], true, false, 0.95);
        assert!(matches!(
            decide(&a, Autonomy::Propose, &full_caps()),
            Decision::Propose
        ));
    }
}
