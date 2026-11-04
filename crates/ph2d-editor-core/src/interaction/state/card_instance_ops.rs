//! ⭐ **T54 — o cartão solto é uma INSTÂNCIA** (`docs/CartoesUI/01_decisoes_do_dono.md`): com a
//! cópia 1 de uma secção presa a um objecto, soltar a mesma secção de novo (outro objecto) cria a
//! cópia 2 em vez de reescrever a 1. O mapa dos soltos chaveia-se pela instância; a cópia 1 tem
//! `instância == secção` (os ids, o `sections.txt` e os testes de antes continuam válidos).

use super::WidgetStore;
use ph2d_a11y::NodeId;

/// O id da instância `n` de `section` (`n = 0` é a própria secção — a cópia 1).
#[must_use]
pub fn loose_instance_of(section: NodeId, n: u64) -> NodeId {
    const SAL: u64 = 0x5B1E_C3A9_74D2_0F68;
    NodeId(section.0 ^ SAL.wrapping_mul(n))
}

/// ⭐ **T63 — a FACE da instância `k`**: a chave do tema, do contorno e das notas da CÓPIA (dono
/// 02/11). Nunca a da secção — a cópia 1 tem o id dela, e era por isso que pintar o solto pintava a
/// lista (e a cópia nova, que dela nasce).
#[must_use]
pub fn loose_face_of(k: NodeId) -> NodeId {
    const SAL: u64 = 0xC4A7_1E93_5B08_D26F;
    NodeId(k.0 ^ SAL)
}

/// `loose_sec.<instância>=<secção>` — só para a cópia que não é a 1 (um ficheiro antigo não a tem).
pub(super) const SECAO: &str = "loose_sec.";

impl WidgetStore {
    /// `id` é a face de um solto (ou do suspenso)? As notas dela não caem no fim da lista.
    #[must_use]
    pub fn is_loose_face(&self, id: NodeId) -> bool {
        let suspenso = self.card_lists.suspended.map(|(k, _)| k);
        self.card_lists
            .loose
            .keys()
            .copied()
            .chain(suspenso)
            .any(|k| loose_face_of(k) == id)
    }

    /// O solto sob `(x, y)` — a instância e o painel dela (o de cima, se se sobrepõem).
    #[must_use]
    pub fn loose_at(&self, x: f32, y: f32) -> Option<(NodeId, NodeId)> {
        self.loose_section_ids()
            .into_iter()
            .filter_map(|k| self.card_lists.loose.get(&k).map(|c| (k, *c)))
            .filter(|(k, _)| {
                self.panel_rect(super::loose_rect_of(*k))
                    .is_some_and(|r| r.contains(x, y))
            })
            .max_by_key(|(_, c)| c.z)
            .map(|(k, c)| (k, c.panel))
    }

    /// T63: tema e contorno de `de` passam a `para` no `panel` — copiados (a cópia nasce com os da
    /// lista) ou movidos (a cópia que volta devolve-os). As notas andam SEMPRE com o cartão: uma
    /// cópia delas ficava em dobro no fim do painel e ressuscitava depois de apagada (dono 03/11).
    pub(super) fn passa_face(&mut self, panel: NodeId, de: NodeId, para: NodeId, move_: bool) {
        self.set_section_theme(para, self.section_theme(de));
        self.set_section_outline_color(para, self.section_outline_color(de));
        if move_ {
            self.set_section_theme(de, None);
            self.set_section_outline_color(de, None);
        }
        if let Some(list) = self.notes_per_panel.get_mut(&panel) {
            for n in list.iter_mut().filter(|n| n.section == Some(de)) {
                n.section = Some(para);
            }
        }
    }

    /// A secção de `id`: a do solto (ou do suspenso) com esse id; senão o próprio `id`.
    #[must_use]
    pub fn loose_section(&self, id: NodeId) -> NodeId {
        self.card_lists
            .loose
            .get(&id)
            .map(|c| c.section)
            .or_else(|| {
                self.card_lists
                    .suspended
                    .filter(|(k, _)| *k == id)
                    .map(|(_, c)| c.section)
            })
            .unwrap_or(id)
    }

    /// `k` (instância ou secção) tem um solto em `panel`?
    #[must_use]
    pub fn is_loose_in(&self, k: NodeId, panel: NodeId) -> bool {
        self.card_lists
            .loose
            .iter()
            .any(|(i, c)| c.panel == panel && (*i == k || c.section == k))
    }

    /// A instância que um novo solto de `section` ocupa: a 1.ª livre (a cópia 1 se está na lista).
    pub(super) fn free_loose_instance(&self, section: NodeId) -> NodeId {
        (0..)
            .map(|n| loose_instance_of(section, n))
            .find(|i| !self.card_lists.loose.contains_key(i))
            .unwrap_or(section)
    }
}

/// Lê `loose_sec.<instância>=<secção>`; `true` se a linha era dela.
pub(super) fn le_secao(store: &mut WidgetStore, k: &str, v: &str) -> bool {
    let Some(id) = k.strip_prefix(SECAO) else {
        return false;
    };
    if let (Ok(id), Ok(sec)) = (id.parse::<u64>(), v.trim().parse::<u64>())
        && let Some(c) = store.card_lists.loose.get_mut(&NodeId(id))
    {
        c.section = NodeId(sec);
    }
    true
}
