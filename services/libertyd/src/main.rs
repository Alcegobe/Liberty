//! libertyd — prototype du démon d'intelligence système de Liberty.
//!
//! L'esprit (`Brain`) observe et initie des actions ; la boucle de décision
//! (`decide()`) est le filet de sécurité de l'OS au-dessous : capacités,
//! niveaux d'autonomie, règle réversible/local. En production l'esprit est
//! Claude (compte Anthropic) ; hors-ligne, une simulation prend le relais.
//!
//!     cargo run --manifest-path services/libertyd/Cargo.toml
//!     cargo test --manifest-path services/libertyd/Cargo.toml

mod autonomy;
mod brain;
mod decision;
mod effects;

use autonomy::{Autonomy, Policy};
use brain::{resolve_credentials, Brain, ClaudeBrain, SimulatedBrain};
use decision::{decide, Decision};
use effects::Effect;

fn run(brain: &dyn Brain, p: &Policy, caps: &[Effect]) {
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

    for a in brain.assess() {
        let level = p.level(a.domain);
        println!("• {} détecté : {}", a.domain.label(), a.trigger);
        match decide(&a, level, caps) {
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

    // Sélection de l'esprit : Claude si un compte/clé Anthropic est lié,
    // sinon simulation hors-ligne.
    let claude = ClaudeBrain::new(resolve_credentials());
    let brain: Box<dyn Brain> = if claude.ready() {
        println!("Esprit : {} — compte Anthropic lié.", claude.name());
        println!(
            "Cible : {} ({} en-têtes d'auth prêts).",
            brain::API_URL,
            claude.headers().len()
        );
        println!("(Transport HTTP non câblé dans ce prototype : requêtes construites");
        println!(" et testées, envoi à venir. Bascule sur la simulation.)\n");
        Box::new(SimulatedBrain)
    } else {
        println!("Esprit : aucun compte Anthropic lié (ANTHROPIC_API_KEY /");
        println!("ANTHROPIC_AUTH_TOKEN absents) → simulation hors-ligne.\n");
        Box::new(SimulatedBrain)
    };

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
        run(brain.as_ref(), p, &caps);
    }

    println!("Lecture : l'IA a *initié* chaque situation (l'inversion). Selon le");
    println!("niveau d'autonomie, elle agit en silence, propose, te consulte, ou se");
    println!("voit refuser par l'OS. Le même jugement faible (mail au patron) remonte");
    println!("toujours une question, quel que soit le curseur ; et la télémétrie reste");
    println!("bloquée même à 99 % de confiance, faute de capacité réseau.");
}
