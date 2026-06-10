//! libertyd — prototype du démon d'intelligence système de Liberty.
//!
//! L'esprit observe l'état du système et INITIE des actions (l'inversion) ;
//! la boucle `decide()` est le filet de sécurité de l'OS au-dessous
//! (capacités, autonomie, réversible/local). En production l'esprit est
//! **Claude** (compte Anthropic, Fable 5 par défaut) ; hors-ligne ou sans
//! compte, une simulation prend le relais.
//!
//!     # démo hors-ligne (sans dépendance) :
//!     cargo run  --manifest-path services/libertyd/Cargo.toml
//!     cargo test --manifest-path services/libertyd/Cargo.toml
//!
//!     # vraie connexion à ton compte Anthropic (sur ta machine) :
//!     export ANTHROPIC_API_KEY=sk-ant-...        # ta clé
//!     cargo run --features claude --manifest-path services/libertyd/Cargo.toml

mod autonomy;
mod brain;
mod decision;
mod effects;
mod model;
#[cfg(feature = "claude")]
mod transport;

use autonomy::{Autonomy, Policy};
use brain::{resolve_credentials, Brain, ClaudeBrain, SimulatedBrain};
use decision::{decide, Action, Decision};
use effects::Effect;

fn run(actions: &[Action], p: &Policy, caps: &[Effect]) {
    println!("══════════════════════════════════════════════════════════════");
    println!(" Profil d'autonomie : « {} »", p.name);
    println!(
        "   {} Système    {} Fichiers    {} Communication",
        p.system.icon(),
        p.files.icon(),
        p.comm.icon()
    );
    println!("══════════════════════════════════════════════════════════════\n");

    let mut journal: Vec<String> = Vec::new();
    let mut proposals: Vec<String> = Vec::new();
    let mut questions: Vec<(String, String)> = Vec::new();
    let mut suggestions: Vec<String> = Vec::new();
    let mut blocked: Vec<(String, String)> = Vec::new();

    for a in actions {
        let level = p.level(a.domain);
        println!("• {} détecté : {}", a.domain.label(), a.trigger);
        match decide(a, level, caps) {
            Decision::Silent => {
                println!("  🟢 Agit seul : {}", a.name);
                journal.push(format!("{} — touche {}", a.name, a.effects_summary()));
            }
            Decision::Undo(s) => {
                println!("  🟢 Fait, annulable {s} s : {}", a.name);
                journal.push(format!(
                    "{} (annulable {s} s) — touche {}",
                    a.name,
                    a.effects_summary()
                ));
            }
            Decision::Propose => {
                println!("  🟡 Propose, attend ta validation : {}", a.name);
                proposals.push(a.name.clone());
            }
            Decision::Ask => {
                println!("  ❓ Te consulte : {}", a.question);
                questions.push((a.name.clone(), a.question.clone()));
            }
            Decision::Suggest => {
                println!("  💡 Suggère (mode manuel) : {}", a.name);
                suggestions.push(a.name.clone());
            }
            Decision::Refused(e) => {
                println!(
                    "  🛡️  Bloqué par l'OS : capacité « {} » non accordée",
                    e.describe()
                );
                blocked.push((a.name.clone(), e.describe()));
            }
        }
        println!();
    }

    println!("  ── Bilan, à la place de l'humain ──");
    print_section("📓 Journal (fait en silence, consultable & annulable)", &journal);
    print_section("🟡 En attente de ta validation", &proposals);
    if !questions.is_empty() {
        println!("  ❓ Questions pour toi :");
        for (n, q) in &questions {
            println!("     - [{n}] {q}");
        }
    }
    print_section("💡 Suggestions (mode manuel)", &suggestions);
    if !blocked.is_empty() {
        println!("  🛡️  Bloqué par l'OS (garde-fou capacités) :");
        for (n, e) in &blocked {
            println!("     - {n} (effet refusé : {e})");
        }
    }
    println!();
}

fn print_section(title: &str, items: &[String]) {
    if items.is_empty() {
        return;
    }
    println!("  {title}");
    for it in items {
        println!("     - {it}");
    }
}

/// Récupère les actions à évaluer : Claude réel si possible, sinon simulation.
fn gather_actions(claude: &ClaudeBrain) -> Vec<Action> {
    #[cfg(feature = "claude")]
    {
        if claude.ready() {
            match transport::startup(claude) {
                Ok(actions) => return actions,
                Err(e) => eprintln!("⚠️  Connexion Anthropic échouée : {e}\n    → bascule sur la simulation hors-ligne.\n"),
            }
        } else {
            println!("Aucun compte Anthropic lié (ANTHROPIC_API_KEY / ANTHROPIC_AUTH_TOKEN absents).");
            println!("→ simulation hors-ligne.\n");
        }
    }
    #[cfg(not(feature = "claude"))]
    {
        if claude.ready() {
            println!("Compte Anthropic détecté, mais binaire compilé sans `--features claude`.");
            println!("→ simulation hors-ligne. Recompile avec `--features claude` pour te connecter.\n");
        } else {
            println!("Mode hors-ligne (sans dépendance) → simulation.\n");
        }
    }
    SimulatedBrain.assess()
}

fn main() {
    // Capacités accordées par l'utilisateur — le garde-fou matériel.
    // À noter : le RÉSEAU n'est volontairement PAS accordé.
    let caps = vec![
        Effect::Process,
        Effect::Power,
        Effect::Files("~".into()),
        Effect::Email,
    ];

    println!("libertyd — démon d'intelligence système de Liberty (prototype)\n");

    let claude = ClaudeBrain::new(resolve_credentials());
    println!("Modèle visé : {}", model::default_model());
    if claude.ready() {
        println!("Esprit : {} — compte Anthropic lié.\n", claude.name());
    }

    let actions = gather_actions(&claude);

    println!(
        "Capacités accordées : {}\n",
        caps.iter().map(|e| e.describe()).collect::<Vec<_>>().join(", ")
    );

    let profiles = [
        Policy {
            name: "Prudent",
            system: Autonomy::Autonomous,
            files: Autonomy::Propose,
            comm: Autonomy::Manual,
        },
        Policy {
            name: "Confiance",
            system: Autonomy::Autonomous,
            files: Autonomy::Autonomous,
            comm: Autonomy::Autonomous,
        },
    ];

    for p in &profiles {
        run(&actions, p, &caps);
    }

    println!("Lecture : l'IA a *initié* chaque situation (l'inversion). Selon le");
    println!("niveau d'autonomie, elle agit en silence, propose, te consulte, ou se");
    println!("voit refuser par l'OS — la sécurité (capacités) reste invariante.");
}
