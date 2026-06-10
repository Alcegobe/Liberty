//! Les sens de Liberty — observation locale du système.
//!
//! Le pré-filtrage est local et gratuit : on lit /proc, df, systemctl,
//! journalctl… et on condense le tout en un *rapport de situation* textuel,
//! compact et factuel, que l'esprit (Claude) reçoit à chaque battement.
//! Aucune donnée ne part vers le réseau ici.

use std::process::Command;

/// Une sonde : un nom + une commande en lecture seule.
struct Probe {
    label: &'static str,
    program: &'static str,
    args: &'static [&'static str],
}

#[cfg(unix)]
const PROBES: &[Probe] = &[
    Probe { label: "Uptime / charge", program: "uptime", args: &[] },
    Probe { label: "Mémoire", program: "free", args: &["-h"] },
    Probe { label: "Disques", program: "df", args: &["-h", "--output=target,size,used,pcent", "-x", "tmpfs", "-x", "devtmpfs"] },
    Probe { label: "Processus les plus gourmands", program: "sh", args: &["-c", "ps aux --sort=-%cpu | head -n 6"] },
    Probe { label: "Services en échec", program: "systemctl", args: &["--failed", "--no-legend", "--plain"] },
    Probe { label: "Erreurs récentes (journal)", program: "journalctl", args: &["-p", "3", "-n", "10", "--no-pager", "-q"] },
    Probe { label: "Température", program: "sh", args: &["-c", "cat /sys/class/thermal/thermal_zone*/temp 2>/dev/null | head -n 4"] },
];

#[cfg(not(unix))]
const PROBES: &[Probe] = &[];

/// Lance toutes les sondes et condense leurs sorties en rapport de situation.
pub fn situation_report() -> String {
    let mut sections = Vec::new();
    for p in PROBES {
        match Command::new(p.program).args(p.args).output() {
            Ok(out) => {
                let text = String::from_utf8_lossy(&out.stdout);
                let text = truncate(text.trim(), 1200);
                if !text.is_empty() {
                    sections.push(format!("## {}\n{}", p.label, text));
                } else if out.status.success() {
                    sections.push(format!("## {}\n(rien à signaler)", p.label));
                }
            }
            Err(_) => {} // sonde indisponible sur cette machine : on l'omet
        }
    }
    if sections.is_empty() {
        return "Aucune sonde système disponible sur cette plate-forme (mode \
                développement ?). Rien à signaler."
            .to_string();
    }
    format!(
        "Rapport de situation (sondes locales, lecture seule) :\n\n{}",
        sections.join("\n\n")
    )
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
    fn report_is_never_empty() {
        assert!(!situation_report().is_empty());
    }

    #[test]
    fn truncate_respects_utf8_boundaries() {
        let s = "éééééééééé"; // 2 octets par char
        let t = truncate(s, 5);
        assert!(t.starts_with("éé"));
        assert!(t.ends_with('…'));
    }
}
