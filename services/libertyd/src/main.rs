//! libertyd — prototype de la boucle de décision de Liberty.
//!
//! Démontre, de façon exécutable, le cœur de la vision de l'OS :
//!  - **l'inversion** : libertyd *initie* (il détecte des situations et agit),
//!    plutôt que d'attendre des ordres ;
//!  - le **modèle d'autonomie** (silencieux / propose / manuel), réglable par
//!    domaine ;
//!  - la règle **« réversible + local => silencieux »**, et le principe
//!    **« rendre réversible pour pouvoir automatiser »** (fenêtre d'annulation
//!    pour les actions externes) ;
//!  - les **capacités comme garde-fou matériel** : même une IA autonome et
//!    très sûre d'elle est *bloquée* si l'effet n'est pas accordé ;
//!  - le **jugement calibré** : une confiance faible remonte une question
//!    précise à l'humain plutôt que d'agir ;
//!  - la **transparence** : tout ce qui est fait en silence est journalisé.
//!
//! Tout est simulé et sans dépendance externe :
//!     cargo run --manifest-path services/libertyd/Cargo.toml

/// Ce qu'une action *touche*. C'est l'unité de contrôle des capacités.
#[derive(Clone, Debug, PartialEq)]
enum Effect {
    Files(String),
    Process,
    Power,
    Email,
    Network(String),
}

impl Effect {
    fn describe(&self) -> String {
        match self {
            Effect::Files(p) => format!("fichiers:{p}"),
            Effect::Process => "processus".to_string(),
            Effect::Power => "énergie".to_string(),
            Effect::Email => "courriel".to_string(),
            Effect::Network(h) => format!("réseau:{h}"),
        }
    }
}

/// Une capacité accordée couvre-t-elle un effet requis ?
/// (Fichiers : par préfixe de chemin. Réseau : par hôte. Le reste : exact.)
fn granted(caps: &[Effect], need: &Effect) -> bool {
    caps.iter().any(|g| match (g, need) {
        (Effect::Files(grant), Effect::Files(want)) => want.starts_with(grant.as_str()),
        (Effect::Network(grant), Effect::Network(want)) => want.ends_with(grant.as_str()),
        (a, b) => a == b,
    })
}

#[derive(Clone, Copy, PartialEq)]
enum Domain {
    System,
    Files,
    Communication,
}

impl Domain {
    fn label(&self) -> &'static str {
        match self {
            Domain::System => "Système",
            Domain::Files => "Fichiers",
            Domain::Communication => "Communication",
        }
    }
}

/// Le niveau d'autonomie, réglable par domaine (le « curseur »).
#[derive(Clone, Copy, PartialEq)]
enum Autonomy {
    Manual,     // 🔴 l'humain fait, l'IA assiste
    Propose,    // 🟡 l'IA prépare, l'humain valide
    Autonomous, // 🟢 l'IA détecte et résout seule
}

impl Autonomy {
    fn icon(&self) -> &'static str {
        match self {
            Autonomy::Autonomous => "🟢",
            Autonomy::Propose => "🟡",
            Autonomy::Manual => "🔴",
        }
    }
}

/// Un profil = un niveau d'autonomie par domaine. Le « curseur » de l'humain.
struct Policy {
    name: &'static str,
    system: Autonomy,
    files: Autonomy,
    comm: Autonomy,
}

impl Policy {
    fn level(&self, d: Domain) -> Autonomy {
        match d {
            Domain::System => self.system,
            Domain::Files => self.files,
            Domain::Communication => self.comm,
        }
    }
}

/// Une action que l'IA a *elle-même* décidé de proposer (l'inversion).
struct Action {
    name: &'static str,
    trigger: &'static str, // pourquoi l'IA a initié ceci
    domain: Domain,
    effects: Vec<Effect>,
    reversible: bool,
    affects_others: bool,
    confidence: f64,        // jugement calibré de l'IA (0..1)
    question: &'static str, // question précise si l'humain doit trancher
}

enum Decision {
    Refused(Effect), // bloqué par les capacités (garde-fou OS)
    Silent,          // agit seule, en silence
    Undo(u32),       // agit, mais annulable pendant N secondes
    Propose,         // prépare et attend la validation
    Ask,             // remonte une question précise
    Suggest,         // mode manuel : se contente de suggérer
}

/// Le cœur : à partir d'une action, du niveau d'autonomie et des capacités,
/// décider quoi faire. C'est ici que vit toute la philosophie de Liberty.
fn decide(a: &Action, level: Autonomy, caps: &[Effect]) -> Decision {
    // 1) Garde-fou matériel : les capacités priment sur tout le reste.
    //    Une IA, même autonome et sûre d'elle, ne peut PAS déborder.
    for e in &a.effects {
        if !granted(caps, e) {
            return Decision::Refused(e.clone());
        }
    }

    // 2) Mode manuel : l'IA n'agit pas, elle suggère.
    if level == Autonomy::Manual {
        return Decision::Suggest;
    }

    // 3) Jugement calibré : dans le doute, on remonte une question précise.
    if a.confidence < 0.6 {
        return Decision::Ask;
    }

    let external = a.affects_others
        || a.effects
            .iter()
            .any(|e| matches!(e, Effect::Email | Effect::Network(_)));

    if !external {
        // Local : si réversible et qu'on est autonome → silencieux.
        if a.reversible && level == Autonomy::Autonomous {
            Decision::Silent
        } else {
            Decision::Propose
        }
    } else {
        // Externe / touche autrui : on l'automatise *grâce* à la réversibilité
        // (fenêtre d'annulation), à condition d'être confiant et autonome.
        if a.reversible && a.confidence >= 0.85 && level == Autonomy::Autonomous {
            Decision::Undo(30)
        } else {
            Decision::Propose
        }
    }
}

fn scenarios() -> Vec<Action> {
    vec![
        Action {
            name: "Brider un processus emballé",
            trigger: "CPU à 98 °C — le process « render » part en boucle",
            domain: Domain::System,
            effects: vec![Effect::Process, Effect::Power],
            reversible: true,
            affects_others: false,
            confidence: 0.96,
            question: "",
        },
        Action {
            name: "Vider 8 Go de cache obsolète",
            trigger: "disque presque plein, cache régénérable",
            domain: Domain::System,
            effects: vec![Effect::Files("~/.cache".into())],
            reversible: true,
            affects_others: false,
            confidence: 0.92,
            question: "",
        },
        Action {
            name: "Supprimer 14 doublons",
            trigger: "14 fichiers identiques dans Téléchargements (corbeille versionnée)",
            domain: Domain::Files,
            effects: vec![Effect::Files("~/Téléchargements".into())],
            reversible: true,
            affects_others: false,
            confidence: 0.88,
            question: "",
        },
        Action {
            name: "Répondre « bien reçu, merci »",
            trigger: "mail anodin de confirmation d'un fournisseur",
            domain: Domain::Communication,
            effects: vec![Effect::Email],
            reversible: true, // délai d'envoi annulable
            affects_others: true,
            confidence: 0.90,
            question: "",
        },
        Action {
            name: "Répondre à ton patron sur le délai",
            trigger: "mail important — l'engagement de date t'appartient",
            domain: Domain::Communication,
            effects: vec![Effect::Email],
            reversible: true,
            affects_others: true,
            confidence: 0.55, // doute → l'IA préfère demander
            question: "On tient le 15, ou je demande 3 jours de plus ?",
        },
        Action {
            name: "Envoyer des statistiques d'usage",
            trigger: "un service voudrait remonter de la télémétrie",
            domain: Domain::System,
            effects: vec![Effect::Network("telemetry.example.com".into())],
            reversible: false,
            affects_others: true,
            confidence: 0.99, // très sûre... mais sans la capacité réseau
            question: "",
        },
    ]
}

fn effs(a: &Action) -> String {
    a.effects
        .iter()
        .map(|e| e.describe())
        .collect::<Vec<_>>()
        .join(", ")
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

fn run(p: &Policy, caps: &[Effect]) {
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
    let mut questions: Vec<(&str, &str)> = Vec::new();
    let mut suggestions: Vec<String> = Vec::new();
    let mut blocked: Vec<(&str, String)> = Vec::new();

    for a in &scenarios() {
        let level = p.level(a.domain);
        println!("• {} détecté : {}", a.domain.label(), a.trigger);
        match decide(a, level, caps) {
            Decision::Silent => {
                println!("  🟢 Agit seul : {}", a.name);
                journal.push(format!("{} — touche {}", a.name, effs(a)));
            }
            Decision::Undo(s) => {
                println!("  🟢 Fait, annulable {s} s : {}", a.name);
                journal.push(format!("{} (annulable {s} s) — touche {}", a.name, effs(a)));
            }
            Decision::Propose => {
                println!("  🟡 Propose, attend ta validation : {}", a.name);
                proposals.push(a.name.to_string());
            }
            Decision::Ask => {
                println!("  ❓ Te consulte : {}", a.question);
                questions.push((a.name, a.question));
            }
            Decision::Suggest => {
                println!("  💡 Suggère (mode manuel) : {}", a.name);
                suggestions.push(a.name.to_string());
            }
            Decision::Refused(e) => {
                println!(
                    "  🛡️  Bloqué par l'OS : capacité « {} » non accordée",
                    e.describe()
                );
                blocked.push((a.name, e.describe()));
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

fn main() {
    // Capacités accordées par l'utilisateur — le garde-fou matériel.
    // À noter : le RÉSEAU n'est volontairement PAS accordé.
    let caps = vec![
        Effect::Process,
        Effect::Power,
        Effect::Files("~".into()),
        Effect::Email,
    ];

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

    println!("libertyd — démon d'intelligence système de Liberty (prototype)\n");
    println!(
        "Capacités accordées : {}\n",
        caps.iter().map(|e| e.describe()).collect::<Vec<_>>().join(", ")
    );

    for p in &profiles {
        run(p, &caps);
    }

    println!("Lecture : l'IA a *initié* chaque situation (l'inversion). Selon le");
    println!("niveau d'autonomie, elle agit en silence, propose, te consulte, ou se");
    println!("voit refuser par l'OS. Le même jugement faible (mail au patron) remonte");
    println!("toujours une question, quel que soit le curseur ; et la télémétrie reste");
    println!("bloquée même à 99 % de confiance, faute de capacité réseau.");
}
