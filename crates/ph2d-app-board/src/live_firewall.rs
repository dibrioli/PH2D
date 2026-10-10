//! **O botão da firewall na família** ([ADR-0181] §8): ler a firewall (sem root), pedir à do sistema
//! que libere as portas do quadro ao vivo só para a rede de casa (`pkexec`: o polkit pede a senha) e
//! desfazê-lo, lembrando em `~/.ph2d/` o que se liberou. Tudo o que espera (o `systemctl`, a senha)
//! corre numa thread: o ecrã nunca pára. O que se decide é puro e vive em `ph2d_board_net::firewall`.
//!
//! [ADR-0181]: ../../../docs/architecture/decisions/0181-o-quadro-ao-vivo-funde-por-campo-e-o-transporte-e-uma-porta.md

use std::path::{Path, PathBuf};
use std::sync::{Arc, mpsc};
use std::time::{Duration, Instant};

use ph2d_board_net::firewall::{self as fw, Allowed, Change, Firewall, Outcome, Probe};
use ph2d_board_net::{self as net, Subnet};
use ph2d_editor_core::documents::live::LiveFirewall;
use ph2d_editor_core::screens::HeroScreen;
use ph2d_i18n::{tr, tr_with};

/// Uma leitura nova da firewall, no máximo a cada… (o painel aberto pede-a ao ritmo da procura).
const PROBE_EVERY: Duration = Duration::from_secs(3);

/// Os acessos ao sistema (trocados nos testes: ninguém corre o `pkexec` num teste).
#[derive(Clone)]
pub(super) struct Env {
    pub probe: Arc<dyn Fn() -> Probe + Send + Sync>,
    pub subnets: Arc<dyn Fn() -> Vec<Subnet> + Send + Sync>,
    /// Corre o comando: o código de saída e o que escreveu no erro.
    pub run: Arc<dyn Fn(&[String]) -> (Option<i32>, String) + Send + Sync>,
    /// Onde se lembra o que se liberou (`None` sem `$HOME`).
    pub file: Option<PathBuf>,
}

impl Default for Env {
    fn default() -> Self {
        Self {
            probe: Arc::new(probe_system),
            subnets: Arc::new(net::lan_subnets),
            run: Arc::new(run_system),
            file: std::env::var_os("HOME")
                .map(|h| PathBuf::from(h).join(".ph2d").join("live_firewall.txt")),
        }
    }
}

/// O que se leu do sistema.
#[derive(Clone, Debug)]
struct Seen {
    firewall: Firewall,
    subnets: Vec<Subnet>,
    allowed: Option<Allowed>,
}

/// O fim de um comando: o desfecho, o que fica liberado depois dele, e o erro de o lembrar.
type Ran = (Change, Outcome, Option<Allowed>, Option<String>);

#[derive(Default)]
pub(super) struct Fw {
    pub env: Env,
    seen: Option<Seen>,
    /// Uma leitura a decorrer, e a porta a dizer no aviso quando ela chegar (a sala acabou de abrir).
    probing: Option<(mpsc::Receiver<Seen>, Option<u16>)>,
    last_probe: Option<Instant>,
    running: Option<mpsc::Receiver<Ran>>,
}

impl Fw {
    /// Ler a firewall de novo (no máximo a cada [`PROBE_EVERY`]; `warn` = a sala abriu agora na
    /// porta dada: avisa se ela está fechada, e lê já).
    pub fn probe(&mut self, warn: Option<u16>) {
        if let Some((_, w)) = &mut self.probing {
            *w = w.or(warn);
            return;
        }
        if warn.is_none() && self.last_probe.is_some_and(|t| t.elapsed() < PROBE_EVERY) {
            return;
        }
        self.last_probe = Some(Instant::now());
        let env = self.env.clone();
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            let _ = tx.send(Seen {
                firewall: fw::detect(&(env.probe)()),
                subnets: (env.subnets)(),
                allowed: env.file.as_deref().and_then(read_allowed),
            });
        });
        self.probing = Some((rx, warn));
    }

    /// «Permitir» / «Remover»: o comando numa thread (o `pkexec` espera pela senha).
    pub fn request(&mut self, hero: &mut HeroScreen, change: Change) {
        let Some(seen) = &self.seen else {
            return;
        };
        if self.running.is_some() {
            return;
        }
        let (firewall, subnets) = match (change, &seen.allowed) {
            (Change::Allow, _) => (seen.firewall, seen.subnets.clone()),
            (Change::Remove, Some(a)) => (a.firewall, a.subnets.clone()),
            (Change::Remove, None) => return,
        };
        let Some(cmd) = fw::command(firewall, change, &subnets) else {
            // O botão só se pinta com uma firewall suportada e uma rede: chegar aqui é um defeito.
            eprintln!("[board] firewall: nada a {change:?} em {firewall:?} {subnets:?}");
            return;
        };
        let (env, before) = (self.env.clone(), seen.allowed.clone());
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            let (code, err) = (env.run)(&cmd);
            let out = fw::outcome(code, &err);
            let (after, unsaved) = if out == Outcome::Done {
                let after = after(before, change, firewall, subnets);
                let unsaved = remember(env.file.as_deref(), after.as_ref()).err();
                (after, unsaved)
            } else {
                (before, None)
            };
            let _ = tx.send((change, out, after, unsaved));
        });
        self.running = Some(rx);
        self.show(hero);
    }

    /// Uma vez por imagem: o que chegou das threads.
    pub fn service(&mut self, hero: &mut HeroScreen) {
        if let Some((rx, warn)) = &self.probing {
            match rx.try_recv() {
                Ok(seen) => {
                    let warn = *warn;
                    self.probing = None;
                    self.seen = Some(seen);
                    if let Some(port) = warn {
                        self.warn(hero, port);
                    }
                }
                Err(mpsc::TryRecvError::Disconnected) => self.probing = None,
                Err(mpsc::TryRecvError::Empty) => {}
            }
        }
        if let Some(rx) = &self.running {
            match rx.try_recv() {
                Ok((change, out, after, unsaved)) => {
                    self.running = None;
                    if let Some(s) = &mut self.seen {
                        s.allowed = after;
                    }
                    hero.documents.tell(match (out, change) {
                        (Outcome::Done, Change::Allow) => tr("board.live.fw_allowed").to_owned(),
                        (Outcome::Done, Change::Remove) => tr("board.live.fw_removed").to_owned(),
                        (Outcome::Cancelled, _) => tr("board.live.fw_cancelled").to_owned(),
                        (Outcome::Failed(e), _) => {
                            tr_with("board.live.fw_failed", &[("error", &e)])
                        }
                    });
                    if let Some(e) = unsaved {
                        hero.documents
                            .tell(tr_with("board.live.fw_unsaved", &[("error", &e)]));
                    }
                }
                Err(mpsc::TryRecvError::Disconnected) => self.running = None,
                Err(mpsc::TryRecvError::Empty) => {}
            }
        }
        self.show(hero);
    }

    /// A sala abriu: se a firewall a fecha a outros computadores, diz-se como abrir.
    fn warn(&self, hero: &mut HeroScreen, port: u16) {
        let Some(s) = &self.seen else {
            return;
        };
        if view(s, false).can_allow {
            hero.documents.tell(tr("board.live.fw_hint").to_owned());
        } else if s.firewall == Firewall::Manual {
            hero.documents.tell(tr_with(
                "board.live.firewall",
                &[("port", &port.to_string())],
            ));
        }
    }

    fn show(&self, hero: &mut HeroScreen) {
        let v = self
            .seen
            .as_ref()
            .map_or_else(LiveFirewall::default, |s| view(s, self.running.is_some()));
        hero.documents.live_view_mut().firewall = v;
    }
}

fn view(s: &Seen, busy: bool) -> LiveFirewall {
    let covered = s
        .allowed
        .as_ref()
        .is_some_and(|a| a.covers(s.firewall, &s.subnets));
    LiveFirewall {
        can_allow: s.firewall.supported() && !s.subnets.is_empty() && !covered,
        can_remove: s.allowed.is_some(),
        busy,
    }
}

/// O que fica liberado depois de um comando que correu bem: liberar SOMA à mesma firewall (a rede
/// do trabalho e a de casa); remover esquece tudo.
fn after(
    before: Option<Allowed>,
    change: Change,
    firewall: Firewall,
    subnets: Vec<Subnet>,
) -> Option<Allowed> {
    match change {
        Change::Remove => None,
        Change::Allow => {
            let mut all = before
                .filter(|a| a.firewall == firewall)
                .map(|a| a.subnets)
                .unwrap_or_default();
            all.extend(subnets);
            all.sort();
            all.dedup();
            Some(Allowed {
                firewall,
                subnets: all,
            })
        }
    }
}

fn read_allowed(p: &Path) -> Option<Allowed> {
    Allowed::parse(&std::fs::read_to_string(p).ok()?)
}

/// Grava (ou apaga) o que ficou liberado; o erro diz-se no ecrã (sem ele o «Remover» não aparece).
fn remember(p: Option<&Path>, a: Option<&Allowed>) -> Result<(), String> {
    let Some(p) = p else {
        return Err("$HOME".to_owned());
    };
    let r = match a {
        Some(a) => p
            .parent()
            .map_or(Ok(()), std::fs::create_dir_all)
            .and_then(|()| std::fs::write(p, a.to_text())),
        None => std::fs::remove_file(p).or_else(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                Ok(())
            } else {
                Err(e)
            }
        }),
    };
    r.map_err(|e| format!("{}: {e}", p.display()))
}

/// O sistema, sem root: o ficheiro do ufw, o `systemctl` e o `pkexec`. Fora do Linux o sistema
/// pergunta sozinho na 1.ª escuta: nenhum botão.
fn probe_system() -> Probe {
    if !cfg!(target_os = "linux") {
        return Probe::default();
    }
    let active = |unit: &str| {
        std::process::Command::new("systemctl")
            .args(["is-active", "--quiet", unit])
            .status()
            .is_ok_and(|s| s.success())
    };
    Probe {
        ufw_conf: std::fs::read_to_string("/etc/ufw/ufw.conf").ok(),
        firewalld_active: Path::new("/usr/bin/firewall-cmd").exists() && active("firewalld"),
        other_active: active("nftables") || active("iptables"),
        pkexec: Path::new("/usr/bin/pkexec").exists(),
    }
}

fn run_system(cmd: &[String]) -> (Option<i32>, String) {
    let Some((prog, args)) = cmd.split_first() else {
        return (None, String::new());
    };
    match std::process::Command::new(prog).args(args).output() {
        Ok(o) => (
            o.status.code(),
            String::from_utf8_lossy(&o.stderr).into_owned(),
        ),
        Err(e) => (None, e.to_string()),
    }
}

#[cfg(test)]
#[path = "live_firewall_tests.rs"]
mod tests;
