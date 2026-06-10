//! Politique de modèle — toujours viser le Claude le plus capable.
//!
//! « Toujours mettre à jour quand un nouveau Claude arrive » se traduit
//! concrètement par DEUX leviers complémentaires :
//!  1. Cette liste de préférence, dont la tête est le défaut. Quand un
//!     nouveau modèle sort, on l'ajoute en haut (un seul endroit à toucher).
//!  2. À l'exécution, `select_available()` interroge l'API Modèles et choisit
//!     le plus capable auquel le compte a réellement accès.
//! L'utilisateur peut toujours forcer un modèle via `LIBERTY_MODEL`.

/// Du plus puissant au plus économique. La tête = le défaut de Liberty.
pub const PREFERRED: &[&str] = &[
    "claude-fable-5", // le plus puissant
    "claude-opus-4-8",
    "claude-opus-4-7",
    "claude-sonnet-4-6",
    "claude-haiku-4-5",
];

/// Le modèle visé par défaut : override `LIBERTY_MODEL`, sinon le plus
/// capable connu.
pub fn default_model() -> String {
    std::env::var("LIBERTY_MODEL")
        .ok()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| PREFERRED[0].to_string())
}

/// Parmi les modèles auxquels le compte a accès, choisir le plus capable
/// selon notre ordre de préférence ; à défaut, le modèle par défaut.
#[allow(dead_code)] // utilisé par le transport réseau (feature `claude`)
pub fn select_available(available: &[String]) -> String {
    for m in PREFERRED {
        if available.iter().any(|a| a == m) {
            return (*m).to_string();
        }
    }
    default_model()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_the_most_powerful_known_model() {
        std::env::remove_var("LIBERTY_MODEL");
        assert_eq!(default_model(), "claude-fable-5");
    }

    #[test]
    fn picks_most_capable_available_in_preference_order() {
        let available = vec!["claude-sonnet-4-6".to_string(), "claude-opus-4-8".to_string()];
        assert_eq!(select_available(&available), "claude-opus-4-8");
    }

    #[test]
    fn falls_back_to_default_when_none_match() {
        std::env::remove_var("LIBERTY_MODEL");
        assert_eq!(select_available(&["mystere".to_string()]), "claude-fable-5");
    }
}
