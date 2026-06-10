//! libertyd — le cœur intelligent de Liberty OS, exposé en bibliothèque.
//!
//! Deux binaires s'appuient dessus :
//!  - `libertyd` : le démon système (boucle observer → réfléchir → décider →
//!    agir → journaliser), l'esprit qui fait tourner l'OS en solo.
//!  - `lish`     : le Liberty Shell, l'interface en langage naturel de
//!    l'utilisateur (la même intelligence, en interactif).
//!
//! L'invariant central : tout ce que l'IA veut faire passe par `decide()`
//! (autonomie + capacités) puis par l'exécuteur journalisé. L'esprit propose,
//! l'OS dispose.

pub mod autonomy;
pub mod brain;
pub mod claude_code;
pub mod config;
pub mod decision;
pub mod effects;
pub mod executor;
pub mod journal;
pub mod model;
pub mod sensors;

#[cfg(feature = "claude")]
pub mod agent;
#[cfg(feature = "claude")]
pub mod transport;
