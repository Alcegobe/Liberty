//! L'exécuteur — la main de Liberty, sous le contrôle de l'OS.
//!
//! C'est ici (et seulement ici) que les décisions deviennent des actes.
//! Deux portes distinctes :
//!  - `observe()` : commandes en LECTURE SEULE (liste blanche stricte de
//!    programmes), que l'esprit peut lancer librement pour comprendre.
//!  - `execute()` : actions qui modifient le système — uniquement après un
//!    verdict favorable de `decide()`, et toujours journalisées.

use crate::journal::{Entry, Journal};
use std::process::Command;

pub struct ExecResult {
    pub success: bool,
    pub output: String,
}

/// Programmes autorisés pour l'observation (lecture seule, sans effet).
/// Première ligne de défense : si le premier mot n'est pas là-dedans, refus.
const READ_ONLY: &[&str] = &[
    "cat", "ls", "df", "du", "free", "ps", "uptime", "who", "id", "stat",
    "head", "tail", "wc", "grep", "find", "file", "uname", "hostname", "date",
    "ip", "ss", "systemctl", "journalctl", "lsblk", "lscpu", "nproc", "env",
    "which", "echo", "sensors",
];

/// Sous-commandes systemctl qui écrivent (interdites en observation).
const SYSTEMCTL_WRITES: &[&str] = &[
    "start", "stop", "restart", "reload", "enable", "disable", "mask",
    "unmask", "edit", "set-property", "daemon-reload", "kill", "isolate",
    "poweroff", "reboot", "halt",
];

/// Une commande est-elle admissible comme observation (lecture seule) ?
pub fn is_read_only(command: &str) -> bool {
    // Pas de chaînage ni de redirection : une commande simple, c'est tout.
    if command.contains(['|', ';', '&', '>', '<', '`'])
        || command.contains("$(")
    {
        return false;
    }
    let mut words = command.split_whitespace();
    let Some(first) = words.next() else {
        return false;
    };
    let prog = first.rsplit('/').next().unwrap_or(first);
    if !READ_ONLY.contains(&prog) {
        return false;
    }
    if prog == "systemctl" {
        if let Some(sub) = words.next() {
            if SYSTEMCTL_WRITES.contains(&sub) {
                return false;
            }
        }
    }
    if prog == "find" && (command.contains("-delete") || command.contains("-exec")) {
        return false;
    }
    true
}

fn shell(command: &str) -> std::io::Result<std::process::Output> {
    #[cfg(unix)]
    {
        Command::new("sh").arg("-c").arg(command).output()
    }
    #[cfg(not(unix))]
    {
        Command::new("powershell")
            .args(["-NoProfile", "-Command", command])
            .output()
    }
}

fn run(command: &str) -> ExecResult {
    match shell(command) {
        Ok(out) => {
            let mut text = String::from_utf8_lossy(&out.stdout).trim().to_string();
            let err = String::from_utf8_lossy(&out.stderr);
            let err = err.trim();
            if !err.is_empty() {
                if !text.is_empty() {
                    text.push('\n');
                }
                text.push_str("[stderr] ");
                text.push_str(err);
            }
            ExecResult { success: out.status.success(), output: truncate(&text, 4000) }
        }
        Err(e) => ExecResult { success: false, output: format!("échec du lancement : {e}") },
    }
}

/// Porte « observation » : refuse tout ce qui n'est pas en liste blanche.
pub fn observe(command: &str, journal: &Journal) -> ExecResult {
    if !is_read_only(command) {
        let msg = format!(
            "Observation refusée : « {command} » n'est pas une commande en \
             lecture seule admise. Utilise l'outil act pour modifier le système."
        );
        journal.record(&Entry {
            kind: "refus".into(),
            name: "observation hors liste blanche".into(),
            detail: msg.clone(),
            command: command.into(),
            undo: String::new(),
        });
        return ExecResult { success: false, output: msg };
    }
    run(command)
}

/// Porte « action » : exécute une commande validée par la boucle de décision,
/// et la journalise avec son éventuelle commande d'annulation.
pub fn execute(name: &str, command: &str, undo: &str, journal: &Journal) -> ExecResult {
    let r = run(command);
    journal.record(&Entry {
        kind: "exec".into(),
        name: name.into(),
        detail: if r.success {
            format!("ok — {}", truncate(&r.output, 300))
        } else {
            format!("ÉCHEC — {}", truncate(&r.output, 300))
        },
        command: command.into(),
        undo: undo.into(),
    });
    r
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        return s.to_string();
    }
    let mut end = max;
    while !s.is_char_boundary(end) {
        end -= 1;
    }
    format!("{}…", &s[..end])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_only_allows_plain_inspection() {
        assert!(is_read_only("df -h"));
        assert!(is_read_only("systemctl status libertyd"));
        assert!(is_read_only("journalctl -p 3 -n 10"));
        assert!(is_read_only("cat /proc/meminfo"));
    }

    #[test]
    fn read_only_rejects_writes_and_chaining() {
        assert!(!is_read_only("rm -rf /"));
        assert!(!is_read_only("systemctl restart nginx"));
        assert!(!is_read_only("cat /etc/passwd > /tmp/x"));
        assert!(!is_read_only("ls; rm x"));
        assert!(!is_read_only("echo $(rm x)"));
        assert!(!is_read_only("find / -name x -delete"));
        assert!(!is_read_only(""));
    }

    #[test]
    fn absolute_paths_resolve_to_program_name() {
        assert!(is_read_only("/usr/bin/df -h"));
        assert!(!is_read_only("/usr/bin/rm x"));
    }
}
