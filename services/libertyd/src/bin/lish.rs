//! lish — le Liberty Shell.
//!
//! Le shell de session de Liberty OS : on y parle au système en langage
//! naturel, et l'esprit (Claude, Fable 5 par défaut) observe, planifie et
//! exécute — sous les mêmes capacités et la même boucle de décision que le
//! démon. Le shell classique reste à un `!` de distance.
//!
//!   ◆ libère 2 Go sur le disque        → mission en langage naturel
//!   ◆ !df -h                           → commande shell brute, sans IA
//!   ◆ :journal                         → ce que l'esprit a fait
//!   ◆ :undo                            → annule la dernière action annulable
//!   ◆ :caps                            → capacités accordées
//!   ◆ exit                             → quitter

use libertyd::brain::{resolve_credentials, ClaudeBrain};
use libertyd::config::Config;
use libertyd::journal::Journal;
use std::io::{BufRead, Write};

fn main() {
    let cfg = Config::load();
    let brain = ClaudeBrain::new(resolve_credentials(&cfg));
    let journal = Journal::open();

    println!("lish — Liberty Shell");
    banner(&cfg, &brain);

    let stdin = std::io::stdin();
    let mut session = Session::new(cfg, brain, journal);
    loop {
        print!("\n◆ ");
        let _ = std::io::stdout().flush();
        let mut line = String::new();
        match stdin.lock().read_line(&mut line) {
            Ok(0) => break, // EOF
            Ok(_) => {}
            Err(_) => break,
        }
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        match line {
            "exit" | "quit" | ":q" => break,
            ":help" => help(),
            ":caps" => session.show_caps(),
            ":journal" => session.show_journal(15),
            ":undo" => session.undo(),
            _ => {
                if let Some(cmd) = line.strip_prefix('!') {
                    session.raw_shell(cmd.trim());
                } else {
                    session.intent(line);
                }
            }
        }
    }
    println!("\nà bientôt.");
}

fn banner(cfg: &Config, brain: &ClaudeBrain) {
    if brain.ready() {
        println!("esprit : {} · profil : {}", brain.name(), cfg.policy().name);
    } else if libertyd::claude_code::find().is_some() {
        println!(
            "esprit : Claude Code (compte Anthropic) · profil : {}",
            cfg.policy().name
        );
    } else {
        println!(
            "⚠ aucun esprit disponible — mode shell brut uniquement.\n\
             deux façons de connecter Liberty à Anthropic :\n\
             1. compte Anthropic : installe Claude Code puis « claude /login »\n\
                curl -fsSL https://claude.ai/install.sh | bash\n\
             2. clé API : credentials.api_key_file dans {}",
            Config::path().display()
        );
    }
    println!("tape :help pour l'aide, ! devant une commande shell brute.");
}

fn help() {
    println!(
        "  <intention>   demande en langage naturel — l'esprit observe, agit\n\
         \u{20}               sous capacités, et te demande validation si besoin\n\
         \u{20} !<commande>   shell brut (sh), sans IA\n\
         \u{20} :journal      les dernières actions de l'esprit\n\
         \u{20} :undo         annule la dernière action annulable\n\
         \u{20} :caps         capacités accordées (config : liberty.toml)\n\
         \u{20} :help         cette aide\n\
         \u{20} exit          quitter"
    );
}

struct Session {
    cfg: Config,
    #[cfg_attr(not(feature = "claude"), allow(dead_code))]
    brain: ClaudeBrain,
    journal: Journal,
    /// Une session Claude Code (mode compte) a déjà démarré → --continue.
    cc_started: bool,
    #[cfg(feature = "claude")]
    messages: Vec<serde_json::Value>,
    #[cfg(feature = "claude")]
    model: Option<String>,
}

impl Session {
    fn new(cfg: Config, brain: ClaudeBrain, journal: Journal) -> Self {
        Self {
            cfg,
            brain,
            journal,
            cc_started: false,
            #[cfg(feature = "claude")]
            messages: Vec::new(),
            #[cfg(feature = "claude")]
            model: None,
        }
    }

    fn show_caps(&self) {
        println!("capacités accordées ({}):", Config::path().display());
        for e in &self.cfg.capabilities {
            println!("  - {}", e.describe());
        }
    }

    fn show_journal(&self, n: usize) {
        let lines = self.journal.tail(n);
        if lines.is_empty() {
            println!("journal vide ({}).", self.journal.path().display());
            return;
        }
        for l in lines {
            println!("  {l}");
        }
    }

    fn undo(&self) {
        match self.journal.last_undoable() {
            None => println!("rien d'annulable au journal."),
            Some(e) => {
                println!("annule « {} » : {}", e.name, e.undo);
                let r = libertyd::executor::execute(
                    &format!("undo: {}", e.name),
                    &e.undo,
                    "",
                    &self.journal,
                );
                println!("{}", if r.success { "fait." } else { "ÉCHEC :" });
                if !r.output.is_empty() {
                    println!("{}", r.output);
                }
            }
        }
    }

    fn raw_shell(&self, cmd: &str) {
        if cmd.is_empty() {
            return;
        }
        #[cfg(unix)]
        let status = std::process::Command::new("sh").arg("-c").arg(cmd).status();
        #[cfg(not(unix))]
        let status = std::process::Command::new("powershell")
            .args(["-NoProfile", "-Command", cmd])
            .status();
        if let Err(e) = status {
            eprintln!("échec : {e}");
        }
    }

    fn intent(&mut self, text: &str) {
        #[cfg(feature = "claude")]
        if self.brain.ready() {
            self.intent_api(text);
            return;
        }
        // Mode compte Anthropic : l'esprit via Claude Code.
        if let Some(cli) = libertyd::claude_code::find() {
            match libertyd::claude_code::interactive(&cli, &self.cfg, text, self.cc_started) {
                Ok(()) => self.cc_started = true,
                Err(e) => eprintln!("✖ {e}"),
            }
            return;
        }
        println!(
            "aucun esprit disponible. Connecte un compte Anthropic :\n\
             curl -fsSL https://claude.ai/install.sh | bash   puis : claude /login\n\
             (ou une clé API — voir :help). En attendant : !<commande>."
        );
    }

    #[cfg(feature = "claude")]
    fn intent_api(&mut self, text: &str) {
        use libertyd::{agent, transport};

        if self.model.is_none() {
            match transport::pick_model(&self.brain, &self.cfg) {
                Ok(m) => {
                    println!("connecté — modèle : {m}");
                    self.model = Some(m);
                }
                Err(e) => {
                    eprintln!("connexion Anthropic impossible : {e}");
                    return;
                }
            }
        }
        let model = self.model.clone().unwrap();
        let mut iface = HumanInterface;
        match agent::run_mission(
            &self.brain,
            &self.cfg,
            &self.journal,
            &mut iface,
            &mut self.messages,
            text,
            &model,
        ) {
            Ok(summary) => println!("\n✔ {summary}"),
            Err(e) => eprintln!("\n✖ {e}"),
        }
        // Borne la mémoire de session (les vieux tours sortent du contexte).
        if self.messages.len() > 60 {
            let excess = self.messages.len() - 60;
            self.messages.drain(0..excess);
            // Le fil doit commencer par un message utilisateur.
            while self
                .messages
                .first()
                .map(|m| m["role"] != "user")
                .unwrap_or(false)
            {
                self.messages.remove(0);
            }
        }
    }

}

#[cfg(feature = "claude")]
struct HumanInterface;

#[cfg(feature = "claude")]
impl libertyd::agent::Interface for HumanInterface {
    fn present(&self) -> bool {
        true
    }

    fn notify(&mut self, line: &str) {
        println!("{line}");
    }

    fn approve(&mut self, name: &str, command: &str, effects: &str) -> bool {
        println!("\n  🟡 l'esprit propose : {name}");
        println!("     commande : {command}");
        println!("     effets   : {effects}");
        print!("     approuver ? [o/N] ");
        let _ = std::io::stdout().flush();
        let mut line = String::new();
        if std::io::stdin().lock().read_line(&mut line).is_err() {
            return false;
        }
        matches!(line.trim().to_lowercase().as_str(), "o" | "oui" | "y" | "yes")
    }

    fn answer(&mut self, question: &str) -> Option<String> {
        println!("\n  ❓ {question}");
        print!("     réponse (vide = passer) : ");
        let _ = std::io::stdout().flush();
        let mut line = String::new();
        if std::io::stdin().lock().read_line(&mut line).is_err() {
            return None;
        }
        let line = line.trim().to_string();
        if line.is_empty() {
            None
        } else {
            Some(line)
        }
    }
}
