//! **A firewall do sistema** ([ADR-0181] §8): o PH2D nunca a abre SOZINHO. No Linux o sistema não
//! pergunta — só bloqueia —, por isso o painel oferece «Permitir na rede de casa»: as portas do
//! quadro ao vivo (TCP [`PORT`]..[`PORT`]+[`PORT_TRIES`]) e a descoberta (mDNS, UDP [`MDNS_PORT`])
//! liberadas SÓ para a sub-rede local, com a senha do sistema (`pkexec`), e desfeitas pelo botão
//! irmão. Aqui só se DECIDE (qual firewall, que comandos, o que se lembrou); correr é da família.
//!
//! [ADR-0181]: ../../../docs/architecture/decisions/0181-o-quadro-ao-vivo-funde-por-campo-e-o-transporte-e-uma-porta.md

use std::fmt;
use std::net::Ipv4Addr;
use std::str::FromStr;

use crate::{PORT, PORT_TRIES};

/// A porta do mDNS: sem ela liberada um 2.º computador não ACHA o quadro (entra só pelo endereço).
pub const MDNS_PORT: u16 = 5353;

/// Uma sub-rede IPv4 normalizada (o endereço de rede e o prefixo). Só se escreve com algarismos,
/// pontos e uma barra: é seguro pô-la num comando.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Subnet {
    net: Ipv4Addr,
    prefix: u8,
}

impl Subnet {
    /// A sub-rede de `ip` com `prefix` bits (`None` se `prefix > 32`).
    #[must_use]
    pub fn new(ip: Ipv4Addr, prefix: u8) -> Option<Self> {
        if prefix > 32 {
            return None;
        }
        let mask = u32::MAX.checked_shl(32 - u32::from(prefix)).unwrap_or(0);
        Some(Self {
            net: Ipv4Addr::from(u32::from(ip) & mask),
            prefix,
        })
    }

    #[must_use]
    pub fn contains(&self, ip: Ipv4Addr) -> bool {
        Self::new(ip, self.prefix).is_some_and(|s| s.net == self.net)
    }
}

impl fmt::Display for Subnet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.net, self.prefix)
    }
}

impl FromStr for Subnet {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, ()> {
        let (ip, p) = s.trim().split_once('/').ok_or(())?;
        Self::new(ip.parse().map_err(|_| ())?, p.parse().map_err(|_| ())?).ok_or(())
    }
}

/// Que firewall manda nesta máquina.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Firewall {
    /// Nenhuma ligada (ou Windows/macOS, onde o sistema pergunta sozinho na 1.ª escuta).
    Off,
    Ufw,
    Firewalld,
    /// Ligada, mas não se muda daqui (nftables/iptables à mão, ou sem `pkexec`): só a frase com a porta.
    Manual,
}

impl Firewall {
    /// O PH2D sabe liberá-la (o botão aparece).
    #[must_use]
    pub fn supported(self) -> bool {
        matches!(self, Self::Ufw | Self::Firewalld)
    }

    fn name(self) -> &'static str {
        match self {
            Self::Ufw => "ufw",
            Self::Firewalld => "firewalld",
            Self::Off | Self::Manual => "",
        }
    }
}

/// O que se leu do sistema (sem root) para decidir.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Probe {
    /// O conteúdo de `/etc/ufw/ufw.conf`, se existe.
    pub ufw_conf: Option<String>,
    /// `systemctl is-active firewalld`.
    pub firewalld_active: bool,
    /// `nftables.service` ou `iptables.service` activos.
    pub other_active: bool,
    /// Há `pkexec` (o polkit pede a senha).
    pub pkexec: bool,
}

/// A firewall que manda: ufw ligado, senão firewalld activo, senão outra regra de sistema activa.
#[must_use]
pub fn detect(p: &Probe) -> Firewall {
    let ufw = p
        .ufw_conf
        .as_deref()
        .is_some_and(|s| s.lines().any(|l| l.trim() == "ENABLED=yes"));
    let fw = if ufw {
        Firewall::Ufw
    } else if p.firewalld_active {
        Firewall::Firewalld
    } else if p.other_active {
        Firewall::Manual
    } else {
        Firewall::Off
    };
    if fw.supported() && !p.pkexec {
        Firewall::Manual
    } else {
        fw
    }
}

/// Liberar ou desfazer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Change {
    Allow,
    Remove,
}

/// O que o PH2D liberou (lembrado em `~/.ph2d/`: a lista de regras da firewall só a lê o root).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Allowed {
    pub firewall: Firewall,
    pub subnets: Vec<Subnet>,
}

impl Allowed {
    /// 1.ª linha a firewall, uma sub-rede por linha.
    #[must_use]
    pub fn to_text(&self) -> String {
        let mut s = format!("{}\n", self.firewall.name());
        for n in &self.subnets {
            s.push_str(&format!("{n}\n"));
        }
        s
    }

    /// `None` se o ficheiro não é deste formato (ou não liberou nada).
    #[must_use]
    pub fn parse(s: &str) -> Option<Self> {
        let mut lines = s.lines().map(str::trim).filter(|l| !l.is_empty());
        let firewall = match lines.next()? {
            "ufw" => Firewall::Ufw,
            "firewalld" => Firewall::Firewalld,
            _ => return None,
        };
        let subnets: Vec<Subnet> = lines.filter_map(|l| l.parse().ok()).collect();
        (!subnets.is_empty()).then_some(Self { firewall, subnets })
    }

    /// As sub-redes de agora já estão TODAS liberadas nesta firewall?
    #[must_use]
    pub fn covers(&self, firewall: Firewall, now: &[Subnet]) -> bool {
        self.firewall == firewall && now.iter().all(|n| self.subnets.contains(n))
    }
}

fn ports() -> (u16, u16) {
    (PORT, PORT + PORT_TRIES - 1)
}

/// O comando (programa e argumentos) que libera ou desfaz `subnets` em `firewall`: UMA senha
/// (`pkexec /bin/sh -c …`), as regras por ordem, a 1.ª que falha pára as outras. `None` se a
/// firewall não é das que se mudam daqui ou não há sub-redes.
#[must_use]
pub fn command(firewall: Firewall, change: Change, subnets: &[Subnet]) -> Option<Vec<String>> {
    if !firewall.supported() || subnets.is_empty() {
        return None;
    }
    let (lo, hi) = ports();
    let mut steps = Vec::new();
    for n in subnets {
        let rules = [
            (format!("{lo}"), format!("{hi}"), "tcp"),
            (MDNS_PORT.to_string(), MDNS_PORT.to_string(), "udp"),
        ];
        for (a, b, proto) in rules {
            steps.push(match firewall {
                Firewall::Ufw => {
                    let port = if a == b { a } else { format!("{a}:{b}") };
                    let verb = match change {
                        Change::Allow => "allow",
                        Change::Remove => "delete allow",
                    };
                    format!("ufw {verb} from {n} to any port {port} proto {proto}")
                }
                _ => {
                    let port = if a == b { a } else { format!("{a}-{b}") };
                    let verb = match change {
                        Change::Allow => "add",
                        Change::Remove => "remove",
                    };
                    format!(
                        "firewall-cmd --permanent --{verb}-rich-rule='rule family=\"ipv4\" source address=\"{n}\" port port=\"{port}\" protocol=\"{proto}\" accept'"
                    )
                }
            });
        }
    }
    if firewall == Firewall::Firewalld {
        steps.push("firewall-cmd --reload".to_owned());
    }
    Some(
        ["pkexec", "/bin/sh", "-c", &steps.join(" && ")]
            .map(str::to_owned)
            .to_vec(),
    )
}

/// Como acabou o comando.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    Done,
    /// A senha não foi dada (o `pkexec` sai com 126 se se fechou a janela, 127 se a senha falhou).
    Cancelled,
    Failed(String),
}

/// O código de saída e o que o comando escreveu no erro → o desfecho.
#[must_use]
pub fn outcome(code: Option<i32>, stderr: &str) -> Outcome {
    match code {
        Some(0) => Outcome::Done,
        Some(126 | 127) => Outcome::Cancelled,
        _ => {
            let line = stderr.lines().map(str::trim).rfind(|l| !l.is_empty());
            Outcome::Failed(line.unwrap_or("?").to_owned())
        }
    }
}
