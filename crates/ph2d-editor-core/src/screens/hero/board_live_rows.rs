//! ⭐ O QUE o painel «Live» mostra, por papel e por caminho de ligação (módulo filho de [`super`]):
//! a rede local, a internet e as pessoas, cada uma no seu cartão com título; as ações da sessão no
//! fim, fora deles (pedido do dono 18/10, `docs/MiroClone/12_cartao_live_por_ligacao.md`). O pintor
//! ([`super::paint`]) só dispõe e desenha estas filas.

use super::super::super::super::HeroScreen;
use super::{LiveButton, show_code};
use crate::documents::live::{LiveInternet, LiveRole};
use ph2d_a11y::NodeId;
use ph2d_i18n::{tr, tr_with};
use ph2d_tokens::{ColorToken, TypeToken};

/// Um cartão do painel: por onde se liga, ou quem está.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LiveSection {
    /// A rede local (o mesmo Wi-Fi): o código, os endereços, a firewall.
    Network,
    /// A internet: o link de convite.
    Internet,
    /// As pessoas na sala.
    People,
}

impl LiveSection {
    pub(super) fn title_key(self) -> &'static str {
        match self {
            Self::Network => "board.live.sec_network",
            Self::Internet => "board.live.sec_internet",
            Self::People => "board.live.people",
        }
    }
}

/// Os campos do `Off`, pela ordem: o rótulo e o campo (a coluna dos rótulos mede-se por eles).
pub(super) const FIELDS: [(&str, NodeId); 3] = [
    ("board.live.code", crate::ids::BOARD_LIVE_CODE_INPUT),
    ("board.live.address", crate::ids::BOARD_LIVE_ADDR_INPUT),
    ("board.live.link", crate::ids::BOARD_LIVE_LINK_INPUT),
];

/// Uma fila do painel.
pub(super) enum Row {
    /// O título de um cartão: as filas seguintes, até ao [`Row::End`], vão dentro dele.
    Section(LiveSection),
    /// Fecha o cartão aberto.
    End,
    /// Texto que quebra em linhas.
    Text(String, TypeToken, ColorToken),
    /// O código grande da sala.
    Code(String),
    Button(LiveButton, String, bool),
    /// Um campo com o rótulo à esquerda.
    Field(&'static str, NodeId),
    /// Um quadro achado na rede (clicável).
    Found(usize, String),
    /// Uma pessoa na sala (a bolinha na cor dela e o nome).
    Person(String, u8),
}

impl Row {
    pub(super) fn is_list(&self) -> bool {
        matches!(self, Row::Found(..) | Row::Person(..))
    }
}

fn text(k: &str, t: TypeToken, c: ColorToken) -> Row {
    Row::Text(tr(k).to_owned(), t, c)
}

fn button(b: LiveButton, k: &str, main: bool) -> Row {
    Row::Button(b, tr(k).to_owned(), main)
}

/// Um cartão com o título e as filas dentro.
fn section(out: &mut Vec<Row>, s: LiveSection, body: Vec<Row>) {
    out.push(Row::Section(s));
    out.extend(body);
    out.push(Row::End);
}

/// O que o painel mostra, por papel.
pub(super) fn rows(hero: &HeroScreen) -> Vec<Row> {
    let lv = &hero.documents.live_view;
    let people = || {
        let mut v = Vec::new();
        let from = lv.panel.people_from % lv.peers.len().max(1);
        for p in lv.peers[from..].iter().chain(&lv.peers[..from]) {
            let name = if p.me {
                tr_with("board.live.you", &[("name", &p.name)])
            } else {
                p.name.clone()
            };
            v.push(Row::Person(name, p.color));
        }
        v
    };
    // A firewall (ADR-0181 §8): antes de abrir e enquanto se é anfitrião — quem entra precisa dela
    // aberta deste lado, e a descoberta também.
    let firewall = || {
        let f = lv.firewall;
        let mut v = Vec::new();
        if f.busy {
            v.push(text("board.live.fw_busy", TypeToken::Xs, ColorToken::Text2));
            return v;
        }
        if f.can_allow {
            v.push(text(
                "board.live.fw_blocked",
                TypeToken::Xs,
                ColorToken::Text2,
            ));
            v.push(button(
                LiveButton::FirewallAllow,
                "board.live.fw_allow",
                false,
            ));
        }
        if f.can_remove {
            v.push(button(
                LiveButton::FirewallRemove,
                "board.live.fw_remove",
                false,
            ));
        }
        v
    };
    // ADR-0187 — a entrada pela internet do anfitrião.
    let internet = || -> Vec<Row> {
        match &lv.internet {
            LiveInternet::Off => vec![button(
                LiveButton::InviteInternet,
                "board.live.invite_net",
                false,
            )],
            LiveInternet::Preparing => vec![text(
                "board.live.invite_net_busy",
                TypeToken::Xs,
                ColorToken::Text2,
            )],
            LiveInternet::Ready { .. } => {
                let mut v = vec![text(
                    "board.live.invite_net_ready",
                    TypeToken::Xs,
                    ColorToken::Text2,
                )];
                // A rede desta casa muda o endereço por destino: quem está fora pode não entrar.
                if lv.link.strict_nat {
                    v.push(text(
                        "board.live.nat_strict_host",
                        TypeToken::Xs,
                        ColorToken::Text1,
                    ));
                }
                v.push(button(
                    LiveButton::InviteInternet,
                    "board.live.invite_net_again",
                    false,
                ));
                v
            }
            LiveInternet::Failed(why) => vec![
                Row::Text(why.clone(), TypeToken::Xs, ColorToken::Text2),
                button(LiveButton::InviteInternet, "board.live.invite_net", false),
            ],
        }
    };
    // Por onde entrou (ou entra) este convidado.
    let way = if lv.link.by_link {
        LiveSection::Internet
    } else {
        LiveSection::Network
    };
    let mut out = Vec::new();
    match &lv.role {
        LiveRole::Off => {
            let mut net = vec![button(LiveButton::Open, "board.live.open", true)];
            net.extend(firewall());
            net.push(text(
                "board.live.join_title",
                TypeToken::Sm,
                ColorToken::Text2,
            ));
            for (i, f) in lv.found.iter().enumerate() {
                let line = tr_with(
                    "board.live.found_row",
                    &[("board", &f.board), ("host", &f.host)],
                );
                net.push(Row::Found(i, line));
            }
            if lv.found.is_empty() {
                net.push(text(
                    "board.live.none_found",
                    TypeToken::Xs,
                    ColorToken::Text3,
                ));
            }
            net.extend(FIELDS[..2].iter().map(|(k, id)| Row::Field(k, *id)));
            net.push(button(LiveButton::Join, "board.live.join", true));
            section(&mut out, LiveSection::Network, net);
            let mut web = vec![
                button(LiveButton::InviteInternet, "board.live.invite_net", false),
                text("board.live.link_hint", TypeToken::Xs, ColorToken::Text3),
            ];
            let (k, id) = FIELDS[2];
            web.push(Row::Field(k, id));
            web.push(button(LiveButton::JoinLink, "board.live.join_link", false));
            // ADR-0187 — porque a última entrada por link falhou, e o que fazer (debaixo do botão dela).
            if let Some(why) = &lv.link.failed {
                web.push(Row::Text(why.clone(), TypeToken::Xs, ColorToken::Text1));
            }
            section(&mut out, LiveSection::Internet, web);
        }
        // ADR-0187 — por link, a espera é a da internet (até `s` segundos), não a da rede local.
        LiveRole::Joining => {
            let mut body = Vec::new();
            match lv.link.searching {
                Some(s) => {
                    body.push(text(
                        "board.live.link_searching",
                        TypeToken::Sm,
                        ColorToken::Text1,
                    ));
                    let hint = tr_with("board.live.link_searching_hint", &[("s", &s.to_string())]);
                    body.push(Row::Text(hint, TypeToken::Xs, ColorToken::Text2));
                    if lv.link.strict_nat {
                        body.push(text(
                            "board.live.nat_strict_guest",
                            TypeToken::Xs,
                            ColorToken::Text1,
                        ));
                    }
                }
                None => body.push(text(
                    "board.live.connecting",
                    TypeToken::Sm,
                    ColorToken::Text1,
                )),
            }
            section(&mut out, way, body);
            out.push(button(LiveButton::Cancel, "board.live.cancel", false));
        }
        LiveRole::Hosting { code, addrs } => {
            let mut net = vec![
                Row::Code(show_code(code)),
                text("board.live.share", TypeToken::Xs, ColorToken::Text2),
            ];
            for a in addrs {
                net.push(Row::Text(a.clone(), TypeToken::Xs, ColorToken::Text3));
            }
            net.extend(firewall());
            section(&mut out, LiveSection::Network, net);
            section(&mut out, LiveSection::Internet, internet());
            section(&mut out, LiveSection::People, people());
            out.push(button(LiveButton::Summon, "board.live.summon", false));
            out.push(button(LiveButton::End, "board.live.end", false));
        }
        LiveRole::Guest { host } => {
            let line = tr_with("board.live.in_board", &[("host", host)]);
            section(
                &mut out,
                way,
                vec![Row::Text(line, TypeToken::Sm, ColorToken::Text1)],
            );
            section(&mut out, LiveSection::People, people());
            out.push(button(LiveButton::Summon, "board.live.summon", false));
            out.push(button(LiveButton::Leave, "board.live.leave", false));
        }
    }
    out
}

/// Os textos das filas do painel como o `rows` os monta (os testes: a frase certa no papel certo).
#[cfg(test)]
pub(crate) fn row_texts(hero: &HeroScreen) -> Vec<String> {
    rows(hero)
        .into_iter()
        .filter_map(|r| match r {
            Row::Text(t, ..) => Some(t),
            _ => None,
        })
        .collect()
}

/// Os cartões do painel, pela ordem (os testes: que caminhos ele mostra).
#[cfg(test)]
pub(crate) fn sections(hero: &HeroScreen) -> Vec<LiveSection> {
    rows(hero)
        .into_iter()
        .filter_map(|r| match r {
            Row::Section(s) => Some(s),
            _ => None,
        })
        .collect()
}
