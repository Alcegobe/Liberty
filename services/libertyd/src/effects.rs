//! Effets et capacités — l'unité de contrôle de sécurité de Liberty.
//!
//! Tout ce qu'une action *touche* est déclaré comme `Effect`. L'OS n'exécute
//! une action que si chaque effet requis est couvert par une capacité
//! accordée. Ce contrôle s'applique en dessous de l'IA : même une décision
//! de Claude ne peut pas le contourner.

#[derive(Clone, Debug, PartialEq)]
pub enum Effect {
    Files(String),
    Process,
    Power,
    Email,
    Network(String),
}

impl Effect {
    pub fn describe(&self) -> String {
        match self {
            Effect::Files(p) => format!("fichiers:{p}"),
            Effect::Process => "processus".to_string(),
            Effect::Power => "énergie".to_string(),
            Effect::Email => "courriel".to_string(),
            Effect::Network(h) => format!("réseau:{h}"),
        }
    }

    /// Un effet externe sort de la machine ou touche autrui.
    pub fn is_external(&self) -> bool {
        matches!(self, Effect::Email | Effect::Network(_))
    }

    /// Parser un effet déclaré par Claude (même forme que `describe()`).
    /// Tout libellé inconnu devient un effet réseau — donc bloqué par défaut,
    /// faute de capacité : on échoue côté sûr.
    #[allow(dead_code)] // utilisé par le transport réseau (feature `claude`)
    pub fn parse(s: &str) -> Effect {
        if let Some(p) = s.strip_prefix("fichiers:") {
            return Effect::Files(p.to_string());
        }
        if let Some(h) = s.strip_prefix("réseau:") {
            return Effect::Network(h.to_string());
        }
        match s {
            "processus" => Effect::Process,
            "énergie" => Effect::Power,
            "courriel" => Effect::Email,
            other => Effect::Network(other.to_string()),
        }
    }
}

/// Une capacité accordée couvre-t-elle un effet requis ?
/// (Fichiers : par préfixe de chemin. Réseau : par hôte. Le reste : exact.)
pub fn granted(caps: &[Effect], need: &Effect) -> bool {
    caps.iter().any(|g| match (g, need) {
        (Effect::Files(grant), Effect::Files(want)) => want.starts_with(grant.as_str()),
        (Effect::Network(grant), Effect::Network(want)) => want.ends_with(grant.as_str()),
        (a, b) => a == b,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_grant_covers_subpaths_only() {
        let caps = vec![Effect::Files("~/Documents".into())];
        assert!(granted(&caps, &Effect::Files("~/Documents/projets".into())));
        assert!(!granted(&caps, &Effect::Files("~/Photos".into())));
    }

    #[test]
    fn network_is_never_implied() {
        let caps = vec![Effect::Process, Effect::Files("~".into()), Effect::Email];
        assert!(!granted(&caps, &Effect::Network("api.example.com".into())));
    }

    #[test]
    fn parse_roundtrips_known_effects_and_fails_safe() {
        assert_eq!(Effect::parse("fichiers:~/Téléchargements"), Effect::Files("~/Téléchargements".into()));
        assert_eq!(Effect::parse("courriel"), Effect::Email);
        // Inconnu → réseau (bloqué par défaut).
        assert!(matches!(Effect::parse("bizarre"), Effect::Network(_)));
    }
}
