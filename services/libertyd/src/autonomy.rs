//! Le modèle d'autonomie : niveaux réglables par domaine (le « curseur »).
//! Voir docs/AUTONOMY.md.

#[derive(Clone, Copy, PartialEq)]
pub enum Domain {
    System,
    Files,
    Communication,
}

impl Domain {
    pub fn label(&self) -> &'static str {
        match self {
            Domain::System => "Système",
            Domain::Files => "Fichiers",
            Domain::Communication => "Communication",
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum Autonomy {
    Manual,     // 🔴 l'humain fait, l'IA assiste
    Propose,    // 🟡 l'IA prépare, l'humain valide
    Autonomous, // 🟢 l'IA détecte et résout seule
}

impl Autonomy {
    pub fn icon(&self) -> &'static str {
        match self {
            Autonomy::Autonomous => "🟢",
            Autonomy::Propose => "🟡",
            Autonomy::Manual => "🔴",
        }
    }
}

/// Un profil = un niveau d'autonomie par domaine.
pub struct Policy {
    pub name: &'static str,
    pub system: Autonomy,
    pub files: Autonomy,
    pub comm: Autonomy,
}

impl Policy {
    pub fn level(&self, d: Domain) -> Autonomy {
        match d {
            Domain::System => self.system,
            Domain::Files => self.files,
            Domain::Communication => self.comm,
        }
    }
}
