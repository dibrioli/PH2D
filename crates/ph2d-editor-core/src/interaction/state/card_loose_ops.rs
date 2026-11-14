//! ⭐ **C5 — o cartão SOLTO no canvas** (`docs/CartoesUI/02_especificacao.md` §8): um cartão
//! arrastado para fora da lista fica a pairar sobre o canvas, em coordenadas de ECRÃ; na lista fica
//! a linha-fantasma. O estado mora aqui; a pintura (`panel::cards`) conduz o gesto e publica o rect
//! do cartão para o clique ser dele (`panel_at` pergunta-lhe antes do canvas).

use super::WidgetStore;
use crate::zones::Rect;
use ph2d_a11y::NodeId;

/// Um cartão fora da lista.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct LooseCard {
    /// T54: a secção de que esta instância é cópia (a chave do mapa é a instância).
    pub section: NodeId,
    /// O painel de onde saiu (é ele que o pinta e para onde «Return to panel» o devolve).
    pub panel: NodeId,
    /// O canto superior esquerdo no ecrã (px lógicos).
    pub at: (f32, f32),
    /// A largura (a da lista de onde saiu).
    pub w: f32,
    /// Profundidade: o último tocado fica por cima.
    pub z: u32,
    /// A mão sobre o cabeçalho: desvio desde o canto (só durante o arrasto).
    pub grab: Option<(f32, f32)>,
    /// O piscar de «trazer à vista» (clique na linha-fantasma).
    pub flash: Option<std::time::Instant>,
    /// T7: o sujeito a que o alfinete o prende (`None` = segue a seleção). Bits da sessão: a chave
    /// que sobrevive a reabrir é a durável ([`WidgetStore::loose_pin_key`]).
    pub pin: Option<u64>,
    /// T44: o alfinete com CORDA — preso e com a corda visível até ao objecto.
    pub rope: bool,
    /// T39: a borda agarrada — `(x, w)` de quando a mão a apanhou (só durante o arrasto).
    pub resize: Option<(f32, f32)>,
    /// T51: a altura pintada (px; `0` = ainda não pintado ⇒ o cabeçalho). Estado do CARTÃO: o rect
    /// publicado apaga-se com a aba atrás e a altura não pode ir com ele.
    pub h: f32,
}

impl LooseCard {
    /// A altura do cartão (o cabeçalho enquanto não foi pintado).
    #[must_use]
    pub fn altura(&self) -> f32 {
        if self.h > 0.0 {
            self.h
        } else {
            ph2d_card_stack::metrics::CARD_HEAD_H
        }
    }
}

/// T39: a borda do solto que muda a largura.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum LooseEdge {
    Left,
    Right,
}

/// O id da faixa de uma borda do cartão solto.
#[must_use]
pub fn loose_edge_of(section: NodeId, edge: LooseEdge) -> NodeId {
    const SAL: [u64; 2] = [0x3D6A_F1C4_8E27_B905, 0xA4C8_0B6E_57D3_19F2];
    NodeId(section.0 ^ SAL[usize::from(edge == LooseEdge::Right)])
}

/// O id do rect publicado de um cartão solto (o alvo do clique, distinto do painel).
#[must_use]
pub fn loose_rect_of(section: NodeId) -> NodeId {
    const SAL: u64 = 0x7C21_9A4E_D0B3_5F16;
    NodeId(section.0 ^ SAL)
}

/// O id da linha-fantasma de `section` na lista (clique = trazer à vista).
#[must_use]
pub fn loose_ghost_of(section: NodeId) -> NodeId {
    const SAL: u64 = 0x2E8D_45B7_19C6_A03F;
    NodeId(section.0 ^ SAL)
}

/// O id do ícone «Return to panel» no cabeçalho do cartão solto.
#[must_use]
pub fn loose_return_of(section: NodeId) -> NodeId {
    const SAL: u64 = 0x91F3_6C08_B7E2_4D5A;
    NodeId(section.0 ^ SAL)
}

#[path = "card_subject_ops.rs"]
mod subject;
pub use subject::loose_pin_of;
#[path = "card_instance_ops.rs"]
mod instance;
pub use instance::{loose_face_of, loose_instance_of};

const CHAVE: &str = "loose.";

impl WidgetStore {
    #[must_use]
    pub fn loose_card(&self, section: NodeId) -> Option<LooseCard> {
        self.card_lists.loose.get(&section).copied()
    }

    /// O painel cuja aba está atrás e que pinta só os soltos neste passo (a lista não se vê).
    #[must_use]
    pub fn loose_only_pass(&self) -> Option<NodeId> {
        self.card_lists.loose_only
    }

    /// Abre o passo só-dos-soltos de `panel` (a aba dele está atrás e o solto continua no canvas).
    pub fn begin_loose_only_pass(&mut self, panel: NodeId) {
        self.card_lists.loose_only = Some(panel);
    }

    /// Fecha o passo: o painel escondido larga o rect que a pintura publicou (não reserva coluna nem
    /// leva cliques), e os rects dos soltos ficam.
    pub fn end_loose_only_pass(&mut self, panel: NodeId) {
        self.card_lists.loose_only = None;
        self.panel_rects.remove(&panel);
        self.floating_panels.remove(&panel);
    }

    /// Todas as secções soltas (de qualquer painel).
    #[must_use]
    pub fn loose_section_ids(&self) -> Vec<NodeId> {
        self.card_lists.loose.keys().copied().collect()
    }

    /// Algum cartão solto cobre `r`? A costura da coluna e a marca dela ficam debaixo dele.
    #[must_use]
    pub fn loose_card_covers(&self, r: Rect) -> bool {
        self.card_lists
            .loose
            .keys()
            .filter_map(|k| self.panel_rect(loose_rect_of(*k)))
            .any(|c| r.x < c.x + c.w && c.x < r.x + r.w && r.y < c.y + c.h && c.y < r.y + r.h)
    }

    /// Os cartões soltos de `panel`, do mais fundo ao de cima.
    #[must_use]
    pub fn loose_cards_of(&self, panel: NodeId) -> Vec<(NodeId, LooseCard)> {
        let mut v: Vec<(NodeId, LooseCard)> = self
            .card_lists
            .loose
            .iter()
            .filter(|(_, c)| c.panel == panel)
            .map(|(k, c)| (*k, *c))
            .collect();
        v.sort_by_key(|(_, c)| c.z);
        v
    }

    /// Solta `section` de `panel` em `at` (ecrã), largura `w`; fica por cima.
    /// T53: o suspenso de `section` volta com o alfinete, a corda e a chave.
    /// T54: com uma cópia já solta (presa), a nova é OUTRA instância ([`loose_instance_of`]).
    pub fn detach_card(&mut self, section: NodeId, panel: NodeId, at: (f32, f32), w: f32) {
        let z = self.next_loose_z();
        let antes = self
            .card_lists
            .suspended
            .take_if(|(_, c)| c.section == section);
        let k = antes.map_or_else(|| self.free_loose_instance(section), |(k, _)| k);
        let antes = antes.map(|(_, c)| c);
        self.card_lists.loose.insert(
            k,
            LooseCard {
                section,
                panel,
                at,
                w,
                z,
                grab: None,
                flash: None,
                pin: antes.and_then(|c| c.pin),
                rope: antes.is_some_and(|c| c.rope),
                resize: None,
                h: antes.map_or(0.0, |c| c.h),
            },
        );
        if antes.is_none() {
            self.passa_face(panel, section, loose_face_of(k), false);
        }
        self.mark_section_prefs_dirty();
    }

    /// T53: a queda de `section` que NÃO o soltou de novo (encaixou ou cancelou): o suspenso acaba —
    /// o alfinete e a chave esquecem-se e a corda recolhe-se (`ph2d-card-rope`).
    pub fn settle_suspended_card(&mut self, section: NodeId) {
        if let Some((k, c)) = self
            .card_lists
            .suspended
            .take_if(|(_, c)| c.section == section)
        {
            self.passa_face(c.panel, loose_face_of(k), c.section, true);
            self.card_lists.ropes.remove(&k);
            self.card_lists.pin_keys.remove(&k);
            self.mark_section_prefs_dirty();
        }
    }

    /// T53: o solto suspenso (a mão trouxe-o à coluna e ainda não largou).
    #[must_use]
    pub fn suspended_card(&self) -> Option<(NodeId, LooseCard)> {
        self.card_lists.suspended
    }

    /// O rect do solto `k` NESTE quadro: a mão que o arrasta já andou e a pintura só o move depois
    /// (T44c); sem rect publicado (a aba atrás, T51) a geometria do cartão; o suspenso (T53) sob a mão.
    #[must_use]
    pub fn loose_card_rect_now(&self, k: NodeId) -> Option<Rect> {
        let susp = self.card_lists.suspended.filter(|(s, _)| *s == k);
        let sec = susp.map_or(k, |(_, c)| c.section);
        let d = self.card_lists.drag.filter(|d| {
            d.section == sec && d.active && d.edge.is_none() && (d.loose || susp.is_some())
        });
        let (c, publicado) = match susp {
            Some((_, c)) => (c, d.map(|_| None)?),
            None => (
                *self.card_lists.loose.get(&k)?,
                self.panel_rect(loose_rect_of(k)),
            ),
        };
        let mut r = publicado.unwrap_or(Rect::new(c.at.0, c.at.1, c.w, c.altura()));
        if let Some(d) = d {
            let g = c.grab.unwrap_or((d.down.0 - c.at.0, d.down.1 - c.at.1));
            (r.x, r.y) = (d.at.0 - g.0, d.at.1 - g.1);
        }
        Some(r)
    }

    /// Devolve `section` à lista (o lugar dela na ordem nunca saiu). T41: de onde estava no ecrã a
    /// mola parte ([`Self::take_card_arrival`]) — o cartão volta a voar, não se teletransporta.
    pub fn return_loose_card(&mut self, section: NodeId) -> bool {
        let era = self.card_lists.loose.remove(&section);
        let ok = era.is_some();
        // T44d: a mão que o trazia passa a arrastá-lo NA LISTA (o voo segue-a até largar). T53: com
        // alfinete fica SUSPENSO — a corda só rompe se encaixar ao largar.
        if let Some(d) = self
            .card_lists
            .drag
            .as_mut()
            .filter(|d| d.section == section)
        {
            d.loose = false;
            // T54: a mão leva a SECÇÃO de volta à lista (o voo é dela, não da instância).
            d.section = era.map_or(section, |c| c.section);
            if let Some(c) = era.filter(|c| c.pin.is_some()) {
                self.card_lists.suspended = Some((section, c));
            }
        }
        if self.card_lists.suspended.is_none_or(|(s, _)| s != section) {
            self.card_lists.ropes.remove(&section);
            self.card_lists.pin_keys.remove(&section);
            if let Some(c) = era {
                self.passa_face(c.panel, loose_face_of(section), c.section, true);
            }
        }
        if let Some(c) = era {
            if let Some(r) = self.panel_rect(loose_rect_of(section)) {
                self.card_lists.arrivals.insert(c.section, (r.x, r.y));
            }
            self.clear_panel_rect(loose_rect_of(section));
            self.mark_section_prefs_dirty();
        }
        ok
    }

    /// T41: de onde regressa `section` (ecrã), uma vez — a pintura da lista consome-o.
    pub fn take_card_arrival(&mut self, section: NodeId) -> Option<(f32, f32)> {
        self.card_lists.arrivals.remove(&section)
    }

    /// Devolve à lista todos os soltos de `panel` (Reset Positions / Reset All).
    pub fn return_loose_cards_of(&mut self, panel: NodeId) {
        for (k, _) in self.loose_cards_of(panel) {
            self.return_loose_card(k);
        }
    }

    /// T39: a borda de um solto que `id` nomeia.
    #[must_use]
    pub fn loose_edge(&self, id: NodeId) -> Option<(NodeId, LooseEdge)> {
        self.card_lists.loose.keys().find_map(|k| {
            [LooseEdge::Left, LooseEdge::Right]
                .into_iter()
                .find(|e| loose_edge_of(*k, *e) == id)
                .map(|e| (*k, e))
        })
    }

    /// T39: a mão apanhou a borda `edge` de `section` (o arrasto começa a mudar a largura).
    pub fn begin_loose_resize(&mut self, section: NodeId, edge: LooseEdge, x: f32, y: f32) {
        let Some(c) = self.card_lists.loose.get_mut(&section) else {
            return;
        };
        c.resize = Some((c.at.0, c.w));
        self.begin_card_drag(section, x, y);
        if let Some(d) = self.card_lists.drag.as_mut() {
            d.loose = true;
            d.edge = Some(edge);
        }
    }

    /// B23.2: algum piscar do chrome a correr pelo relógio de parede (o anel do comando, `Ctrl+K`, ou o
    /// lampejo do cartão solto)? Com o laço a pedido, é um produtor de quadro (`Produtor::UiAnima`).
    #[must_use]
    pub fn a_piscar(&self) -> bool {
        let solto = |c: &LooseCard| {
            c.flash.is_some_and(|t| {
                t.elapsed().as_secs_f64() < ph2d_card_stack::metrics::LOOSE_FLASH_S
            })
        };
        self.flashing(true).is_some() || self.card_lists.loose.values().any(solto)
    }

    /// Por cima de todos (o último tocado); `flash` = pisca para o artista o achar.
    pub fn raise_loose_card(&mut self, section: NodeId, flash: bool) {
        let z = self.next_loose_z();
        if let Some(c) = self.card_lists.loose.get_mut(&section) {
            c.z = z;
            if flash {
                c.flash = Some(std::time::Instant::now());
            }
        }
    }

    /// A pintura move/agarra o cartão (sem marcar a gravação — só a queda grava).
    pub fn loose_card_mut(&mut self, section: NodeId) -> Option<&mut LooseCard> {
        self.card_lists.loose.get_mut(&section)
    }

    /// A queda de um cartão solto: larga a mão e grava a posição.
    pub fn settle_loose_card(&mut self, section: NodeId) {
        if let Some(c) = self.card_lists.loose.get_mut(&section) {
            c.grab = None;
            c.resize = None;
            self.mark_section_prefs_dirty();
        }
    }

    /// ⭐ Publica o rect do cartão solto: alvo do clique por cima do canvas, mas não reserva coluna
    /// nem marca o lado como lista flutuante (`docked_panel_rects`/`floating_panel_rects` saltam-no).
    pub fn set_loose_card_rect(&mut self, section: NodeId, rect: Rect) {
        let id = loose_rect_of(section);
        self.panel_rects.insert(id, rect);
        self.floating_panels.remove(&id);
        self.card_lists.loose_rects.insert(id);
    }

    /// O rect publicado é de um cartão solto?
    #[must_use]
    pub fn is_loose_card_rect(&self, id: NodeId) -> bool {
        self.card_lists.loose_rects.contains(&id)
    }

    /// Larga os rects dos cartões soltos de `panel` (o painel não pinta neste quadro).
    pub fn clear_loose_card_rects(&mut self, panel: NodeId) {
        let ids: Vec<NodeId> = self
            .card_lists
            .loose
            .iter()
            .filter(|(_, c)| c.panel == panel)
            .map(|(k, _)| loose_rect_of(*k))
            .collect();
        for id in ids {
            self.panel_rects.remove(&id);
        }
    }

    fn next_loose_z(&mut self) -> u32 {
        self.card_lists.loose_z += 1;
        self.card_lists.loose_z
    }
}

/// Grava `loose.<secção>=<painel>,<x>,<y>,<w>` (o ficheiro das secções, ao lado da ordem).
pub(super) fn escreve(store: &WidgetStore, s: &mut String) {
    use std::fmt::Write as _;
    for (k, c) in &store.card_lists.loose {
        let titulo = store.loose_card_title(*k).unwrap_or_default();
        let _ = writeln!(
            s,
            "{CHAVE}{}={},{},{},{},{titulo}",
            k.0, c.panel.0, c.at.0, c.at.1, c.w
        );
        if c.section != *k {
            let _ = writeln!(s, "{}{}={}", instance::SECAO, k.0, c.section.0);
        }
        if let Some((id, nome)) = store.card_lists.pin_keys.get(k) {
            let _ = writeln!(s, "{PINO}{}={id},{},{nome}", k.0, u8::from(c.rope));
        }
    }
}

/// O alfinete durável: `loose_pin.<secção>=<StableId>,<corda 0|1>,<nome>` (um ficheiro de antes não
/// tem a linha e o solto segue a seleção, como antes).
const PINO: &str = "loose_pin.";

fn le_pino(store: &mut WidgetStore, k: &str, v: &str) -> bool {
    let Some(id) = k.strip_prefix(PINO) else {
        return instance::le_secao(store, k, v);
    };
    let mut p = v.splitn(3, ',');
    if let (Ok(id), Some(Ok(sid)), Some(corda), Some(nome)) = (
        id.parse::<u64>(),
        p.next().map(|t| t.trim().parse::<u64>()),
        p.next(),
        p.next(),
    ) && sid != 0
    {
        let id = NodeId(id);
        store.card_lists.pin_keys.insert(id, (sid, nome.to_owned()));
        if let Some(c) = store.card_lists.loose.get_mut(&id) {
            c.rope = corda.trim() == "1";
        }
    }
    true
}

/// Lê uma linha `k=v`; `true` se era de um cartão solto (mesmo ilegível).
pub(super) fn le(store: &mut WidgetStore, k: &str, v: &str) -> bool {
    let Some(id) = k.strip_prefix(CHAVE) else {
        return le_pino(store, k, v);
    };
    // O título vem no fim e pode ter vírgulas; um ficheiro de antes não o tem.
    let mut partes: Vec<&str> = v.splitn(5, ',').collect();
    let titulo = (partes.len() == 5).then(|| partes.remove(4).trim().to_owned());
    let partes: Vec<&str> = partes.into_iter().map(str::trim).collect();
    if let (Ok(id), [p, x, y, w]) = (id.parse::<u64>(), partes.as_slice())
        && let (Ok(p), Ok(x), Ok(y), Ok(w)) = (
            p.parse::<u64>(),
            x.parse::<f32>(),
            y.parse::<f32>(),
            w.parse::<f32>(),
        )
        && x.is_finite()
        && y.is_finite()
        && w.is_finite()
        && w > 0.0
    {
        let z = store.next_loose_z();
        store.card_lists.loose.insert(
            NodeId(id),
            LooseCard {
                section: NodeId(id),
                panel: NodeId(p),
                at: (x, y),
                w,
                z,
                grab: None,
                flash: None,
                pin: None,
                rope: false,
                resize: None,
                h: 0.0,
            },
        );
        if let Some(t) = titulo.filter(|t| !t.is_empty()) {
            store.card_lists.titles.insert(NodeId(id), t);
        }
    }
    true
}

#[cfg(test)]
#[path = "card_loose_ops_tests.rs"]
mod tests;
