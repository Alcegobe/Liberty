//! libertyd — le démon d'intelligence système de Liberty OS.
//!
//! Modes :
//!   libertyd --daemon   boucle autonome : observer → réfléchir (Claude) →
//!                       décider → agir → journaliser, à chaque battement.
//!   libertyd --once     un seul battement (utile en test / cron).
//!   libertyd --demo     démonstration hors-ligne de la boucle de décision
//!                       (aucun réseau, esprit simulé).
//!
//! Sans argument : --once si un compte Anthropic est lié, sinon --demo.
//!
//!     # build OS (connexion réelle) :
//!     cargo build --release --features claude
//!     # build dev hors-ligne :
//!     cargo run -- --demo

use libertyd::autonomy::{Autonomy, Policy};
use libertyd::brain::{resolve_credentials, ClaudeBrain, SimulatedBrain};
use libertyd::config::Config;
use libertyd::decision::{decide, Action, Decision};
use libertyd::effects::Effect;
use libertyd::model;

fn main() {
    let mode = std::env::args().nth(1).unwrap_or_default();
    let cfg = Config::load();
    let brain = ClaudeBrain::new(resolve_credentials(&cfg));

    match mode.as_str() {
        "--demo" => demo(),
        "--help" | "-h" => help(),
        "--daemon" => live(&cfg, &brain, true),
        "--once" => live(&cfg, &brain, false),
        "" => {
            if brain.ready() || libertyd::claude_code::find().is_some() {
                live(&cfg, &brain, false)
            } else {
                println!("Aucun compte Anthropic lié → démonstration hors-ligne.\n");
                demo()
            }
        }
        other => {
            eprintln!("argument inconnu : {other}\n");
            help();
            std::process::exit(2);
        }
    }
}

fn help() {
    println!(
        "libertyd — démon d'intelligence système de Liberty OS\n\n\
         usage : libertyd [--daemon | --once | --demo | --help]\n\n\
         --daemon  boucle autonome continue (production)\n\
         --once    un seul battement de cœur\n\
         --demo    démonstration hors-ligne (esprit simulé, aucun réseau)\n\n\
         Config   : {} (LIBERTY_CONFIG pour changer)\n\
         Modèle   : {} (LIBERTY_MODEL pour forcer)\n\
         Compte   : ANTHROPIC_API_KEY / ANTHROPIC_AUTH_TOKEN, ou \
         credentials.api_key_file dans la config",
        Config::path().display(),
        model::default_model(),
    );
}

// ---------------------------------------------------------------------------
// Mode réel : l'esprit au travail (feature `claude`)
// ---------------------------------------------------------------------------

/// Mission envoyée à l'esprit à chaque battement de cœur.
fn heartbeat_mission() -> String {
    format!(
        "Battement de cœur autonome de Liberty. Examine ce rapport, traite \
         ce qui mérite de l'être (santé du système, disque, services en \
         échec, processus anormaux). S'il n'y a rien d'utile à faire, \
         conclus immédiatement — n'invente pas de travail.\n\n{}",
        libertyd::sensors::situation_report()
    )
}

/// Battement de cœur en mode « compte Anthropic » (esprit via Claude Code).
fn live_claude_code(cfg: &Config, cli: std::path::PathBuf, forever: bool) {
    println!("libertyd — esprit via Claude Code (compte Anthropic)");
    println!("  Profil : {}\n", cfg.policy().name);
    loop {
        match libertyd::claude_code::heartbeat(&cli, cfg, &heartbeat_mission()) {
            Ok(s) => println!("♥ bilan : {s}\n"),
            Err(e) => eprintln!("⚠️  battement interrompu : {e}\n"),
        }
        if !forever {
            break;
        }
        std::thread::sleep(std::time::Duration::from_secs(cfg.heartbeat_secs));
    }
}

#[cfg(feature = "claude")]
fn live(cfg: &Config, brain: &ClaudeBrain, forever: bool) {
    use libertyd::journal::Journal;
    use libertyd::{agent, transport};

    if !brain.ready() {
        if let Some(cli) = libertyd::claude_code::find() {
            return live_claude_code(cfg, cli, forever);
        }
        eprintln!(
            "Aucun esprit disponible : ni compte Anthropic via Claude Code \
             (installe-le : curl -fsSL https://claude.ai/install.sh | bash, \
             puis « claude /login »), ni clé API (ANTHROPIC_API_KEY ou \
             credentials.api_key_file dans {}).",
            Config::path().display()
        );
        std::process::exit(1);
    }

    let journal = Journal::open();
    let model = match transport::pick_model(brain, cfg) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("⚠️  Connexion Anthropic échouée : {e}");
            std::process::exit(1);
        }
    };

    println!("libertyd — l'esprit de Liberty est en ligne");
    println!("  Modèle    : {model}");
    println!("  Profil    : {}", cfg.policy().name);
    println!(
        "  Capacités : {}",
        cfg.capabilities
            .iter()
            .map(|e| e.describe())
            .collect::<Vec<_>>()
            .join(", ")
    );
    println!("  Journal   : {}\n", journal.path().display());

    let mut iface = DaemonInterface;
    loop {
        let mission = heartbeat_mission();
        let mut messages = Vec::new();
        match agent::run_mission(brain, cfg, &journal, &mut iface, &mut messages, &mission, &model)
        {
            Ok(summary) => println!("♥ bilan : {summary}\n"),
            Err(e) => eprintln!("⚠️  battement interrompu : {e}\n"),
        }
        if !forever {
            break;
        }
        std::thread::sleep(std::time::Duration::from_secs(cfg.heartbeat_secs));
    }
}

#[cfg(feature = "claude")]
struct DaemonInterface;

#[cfg(feature = "claude")]
impl libertyd::agent::Interface for DaemonInterface {
    fn present(&self) -> bool {
        false // personne au bout du fil : tout passe par le journal
    }
    fn notify(&mut self, line: &str) {
        println!("{line}");
    }
    fn approve(&mut self, _name: &str, _command: &str, _effects: &str) -> bool {
        false
    }
    fn answer(&mut self, _question: &str) -> Option<String> {
        None
    }
}

#[cfg(not(feature = "claude"))]
fn live(cfg: &Config, _brain: &ClaudeBrain, forever: bool) {
    // Sans transport réseau intégré, le mode « compte Anthropic » via
    // Claude Code reste possible.
    if let Some(cli) = libertyd::claude_code::find() {
        return live_claude_code(cfg, cli, forever);
    }
    eprintln!(
        "Ce binaire est compilé sans le transport réseau, et Claude Code \
         n'est pas installé.\n\
         - compte Anthropic : curl -fsSL https://claude.ai/install.sh | bash, \
         puis « claude /login »\n\
         - ou recompile avec `cargo build --release --features claude` (clé API)\n\
         - ou lance `libertyd --demo` pour la démonstration hors-ligne."
    );
    std::process::exit(1);
}

// ---------------------------------------------------------------------------
// Mode démo : la boucle de décision à l'œuvre, hors-ligne
// ---------------------------------------------------------------------------

fn demo() {
    let caps = vec![
        Effect::Process,
        Effect::Power,
        Effect::Files("~".into()),
        Effect::Email,
    ];

    println!("libertyd — démonstration hors-ligne de la boucle de décision\n");
    let sim = SimulatedBrain;
    println!("Esprit : {}", sim.name());
    println!("Modèle visé en production : {}\n", model::default_model());

    let actions = sim.assess();
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
        run_profile(&actions, p, &caps);
    }

    println!("Lecture : l'IA a *initié* chaque situation (l'inversion). Selon le");
    println!("niveau d'autonomie, elle agit en silence, propose, te consulte, ou se");
    println!("voit refuser par l'OS — la sécurité (capacités) reste invariante.");
}

fn run_profile(actions: &[Action], p: &Policy, caps: &[Effect]) {
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
