//! ⭐ **De QUEM é um cartão** (`docs/CartoesUI/02_especificacao.md` §8, T7): o painel publica o
//! sujeito da seleção (bits + nome); um cartão SOLTO de uma secção «com sujeito» mostra-o no
//! subtítulo e o ALFINETE prende-o a esse sujeito. O despacho anota sobre que cartão preso caiu a
//! última pressão — é por aí que o app sabe que o pedido vem do sujeito preso e não da seleção.

use super::super::{InteractiveState, WidgetStore};
use super::loose_rect_of;
use crate::interaction::WidgetEvent;
use ph2d_a11y::NodeId;

/// O id do alfinete no cabeçalho do cartão solto.
#[must_use]
pub fn loose_pin_of(section: NodeId) -> NodeId {
    const SAL: u64 = 0x5BA7_E2C9_04D1_86F3;
    NodeId(section.0 ^ SAL)
}

impl WidgetStore {
    /// O painel publica o sujeito da seleção de agora (`None` = nada a que os cartões se apliquem).
    pub fn set_card_subject(&mut self, panel: NodeId, subject: Option<(u64, String)>) {
        match subject {
            Some((bits, nome)) => {
                self.card_lists.names.insert(bits, nome);
                self.card_lists.subjects.insert(panel, bits);
            }
            None => {
                self.card_lists.subjects.remove(&panel);
            }
        }
    }

    /// Os bits do sujeito de agora de `panel`.
    #[must_use]
    pub fn card_subject(&self, panel: NodeId) -> Option<u64> {
        self.card_lists.subjects.get(&panel).copied()
    }

    /// O nome de um sujeito (da seleção ou preso), como o painel o publicou.
    pub fn set_card_subject_name(&mut self, bits: u64, nome: String) {
        self.card_lists.names.insert(bits, nome);
    }

    #[must_use]
    pub fn card_subject_name(&self, bits: u64) -> Option<&str> {
        self.card_lists.names.get(&bits).map(String::as_str)
    }

    /// A secção mostra um sujeito (o osso) — o cartão solto dela ganha subtítulo e alfinete.
    pub fn mark_card_pinnable(&mut self, section: NodeId) {
        self.card_lists.pinnable.insert(section);
    }

    #[must_use]
    pub fn is_card_pinnable(&self, section: NodeId) -> bool {
        self.card_lists
            .pinnable
            .contains(&self.loose_section(section))
    }

    /// O alfinete, em ciclo (T44): solto ⇒ preso ao sujeito de agora do painel (sem sujeito, nada) ⇒
    /// preso COM corda ⇒ solto.
    pub fn toggle_loose_pin(&mut self, section: NodeId) {
        let Some(c) = self.card_lists.loose.get(&section).copied() else {
            return;
        };
        let (pin, rope) = match (c.pin, c.rope) {
            (Some(p), false) => (Some(p), true),
            (Some(_), true) => (None, false),
            (None, _) => (self.card_subject(c.panel), false),
        };
        if let Some(c) = self.card_lists.loose.get_mut(&section) {
            c.pin = pin;
            c.rope = rope;
        }
        if !rope {
            self.card_lists.ropes.remove(&section);
        }
        if pin.is_none() {
            self.card_lists.pin_keys.remove(&section);
        }
        self.mark_section_prefs_dirty();
    }

    /// Os soltos presos ou com alfinete durável por resolver: `(secção, bits, chave durável)`. Quem
    /// tem o mundo (a shell, por `ph2d_card_rope::chave`) liga uns aos outros a cada quadro.
    #[must_use]
    pub fn loose_pin_keys(&self) -> Vec<LoosePinKey> {
        self.card_lists
            .loose
            .iter()
            .map(|(k, c)| (*k, c.pin, self.card_lists.pin_keys.get(k).cloned()))
            .filter(|(_, p, d)| p.is_some() || d.is_some())
            .collect()
    }

    /// A chave durável `(StableId, nome)` do sujeito preso de `section`.
    #[must_use]
    pub fn loose_pin_key(&self, section: NodeId) -> Option<(u64, &str)> {
        self.card_lists
            .pin_keys
            .get(&section)
            .map(|(id, n)| (*id, n.as_str()))
    }

    pub fn set_loose_pin_key(&mut self, section: NodeId, key: (u64, String)) {
        if self.card_lists.loose.contains_key(&section)
            && self.card_lists.pin_keys.get(&section) != Some(&key)
        {
            self.card_lists.pin_keys.insert(section, key);
            self.mark_section_prefs_dirty();
        }
    }

    /// Os bits de agora do sujeito durável de `section` (`None` = ainda não está no mundo: reabrir
    /// outro projecto, o respawn do undo). A chave fica; o alfinete volta quando o sujeito voltar.
    pub fn rebind_loose_pin(&mut self, section: NodeId, bits: Option<u64>) {
        if let Some(c) = self.card_lists.loose.get_mut(&section) {
            c.pin = bits;
        }
    }

    /// T44: os soltos de `panel` presos COM corda — `(secção, sujeito)`; o app anda a corda de cada um.
    #[must_use]
    pub fn loose_ropes_of(&self, panel: NodeId) -> Vec<(NodeId, u64)> {
        self.card_lists
            .loose
            .iter()
            .chain(self.card_lists.suspended.as_ref().map(|(k, c)| (k, c)))
            .filter(|(_, c)| c.panel == panel && c.rope)
            .filter_map(|(k, c)| Some((*k, c.pin?)))
            .collect()
    }

    /// T44: os pontos da corda de `section` em ECRÃ, do cartão ao objecto (vazio = sem corda).
    pub fn set_loose_rope(&mut self, section: NodeId, pontos: Vec<[f32; 2]>) {
        if pontos.is_empty() {
            self.card_lists.ropes.remove(&section);
        } else {
            self.card_lists.ropes.insert(section, pontos);
        }
    }

    #[must_use]
    pub fn loose_rope(&self, section: NodeId) -> &[[f32; 2]] {
        self.card_lists
            .ropes
            .get(&section)
            .map_or(&[], Vec::as_slice)
    }

    /// T41: a corda de `section` que se soltou, a recolher-se (vazio = acabou).
    pub fn set_reeling_rope(&mut self, section: NodeId, panel: NodeId, pontos: Vec<[f32; 2]>) {
        if pontos.is_empty() {
            self.card_lists.reels.remove(&section);
        } else {
            self.card_lists.reels.insert(section, (panel, pontos));
        }
    }

    /// As cordas de `panel` a recolher-se.
    #[must_use]
    pub fn reeling_ropes_of(&self, panel: NodeId) -> Vec<(NodeId, &[[f32; 2]])> {
        self.card_lists
            .reels
            .iter()
            .filter(|(_, (p, _))| *p == panel)
            .map(|(k, (_, v))| (*k, v.as_slice()))
            .collect()
    }

    /// Larga o alfinete (o sujeito preso deixou de existir).
    pub fn unpin_loose_card(&mut self, section: NodeId) {
        if let Some(c) = self.card_lists.loose.get_mut(&section) {
            c.pin = None;
            c.rope = false;
        }
        self.card_lists.ropes.remove(&section);
        self.card_lists.pin_keys.remove(&section);
        self.mark_section_prefs_dirty();
    }

    /// Os sujeitos presos pelos cartões soltos de `panel` (a shell publica o estado de cada um).
    #[must_use]
    pub fn loose_pins_of(&self, panel: NodeId) -> Vec<u64> {
        let mut v: Vec<u64> = self
            .card_lists
            .loose
            .values()
            .filter(|c| c.panel == panel)
            .filter_map(|c| c.pin)
            .collect();
        v.sort_unstable();
        v.dedup();
        v
    }

    /// O título de um cartão solto, guardado a cada quadro em que se pinta (o «Not available»
    /// precisa dele quando o painel já não o dá).
    pub fn set_loose_card_title(&mut self, section: NodeId, title: &str) {
        if self.card_lists.titles.get(&section).map(String::as_str) != Some(title) {
            self.card_lists.titles.insert(section, title.to_owned());
            self.mark_section_prefs_dirty();
        }
    }

    #[must_use]
    pub fn loose_card_title(&self, section: NodeId) -> Option<&str> {
        self.card_lists.titles.get(&section).map(String::as_str)
    }

    /// ⭐ A pressão em `(x, y)`: caiu num cartão solto PRESO? Então os pedidos deste gesto são do
    /// sujeito dele; noutro sítio qualquer, da seleção.
    pub fn note_card_press(&mut self, x: f32, y: f32) {
        // A lista aberta é do chip: a opção que sai para fora do solto continua a ser do sujeito
        // da pressão que a abriu (bloco A, sem isto a escolha ia para a seleção).
        if self.dropdown_popover().is_some_and(|(id, r)| {
            r.contains(x, y)
                && matches!(
                    self.get(id),
                    Some(InteractiveState::Dropdown { open: true, .. })
                )
        }) {
            return;
        }
        if self.card_dropdown_at(x, y) || self.context_menu().is_some() {
            return;
        }
        // C2: a lista de estilos e o seletor de cor são de quem os abriu, nunca do que está por baixo.
        if let Some(sujeito) = crate::interaction::sujeito_da_capa_dos_estilos(x, y) {
            self.card_lists.press_subject = sujeito;
            return;
        }
        let no_seletor = self.picker_target().is_some()
            && self
                .panel_rect(crate::ids::INSP_BLENDER_PICKER)
                .is_some_and(|r| r.contains(x, y));
        let mut topo: Option<(u32, NodeId, Option<u64>)> = None;
        for (k, c) in &self.card_lists.loose {
            if let Some(r) = self.panel_rects.get(&loose_rect_of(*k))
                && r.contains(x, y)
                && topo.is_none_or(|(z, _, _)| c.z > z)
            {
                topo = Some((c.z, *k, c.pin));
            }
        }
        topo = topo.filter(|_| !no_seletor);
        let antes = self.card_lists.press_subject;
        let agora = if no_seletor {
            self.card_lists.picker_subject
        } else {
            topo.and_then(|(_, _, p)| p)
        };
        // C1: o campo em foco no preso faz o commit ao sair do foco, e é aqui que ele sai: é dele.
        self.card_lists.blur_subject = antes
            .filter(|_| antes != agora)
            .and_then(|s| self.focus_id.map(|f| (f, s)));
        self.card_lists.press_subject = agora;
        // T40: o gesto parte dos números do preso (o scrub e a edição leem o store já a seguir).
        if let Some((_, k, Some(_))) = topo {
            for (id, v) in self.pinned_values(k).to_vec() {
                self.set_number_value(id, v);
            }
            self.poe_estados_presos(k);
        }
    }

    /// T40: os números do sujeito preso de `section`, publicados pelo painel ao construir o cartão.
    pub fn set_pinned_values(&mut self, section: NodeId, values: Vec<(NodeId, f64)>) {
        self.card_lists.pinned_values.insert(section, values);
    }

    #[must_use]
    pub fn pinned_values(&self, section: NodeId) -> &[(NodeId, f64)] {
        self.card_lists
            .pinned_values
            .get(&section)
            .map_or(&[], Vec::as_slice)
    }

    /// C2: de quem é o seletor de cor aberto (`None` = da seleção). Quem o abriu num cartão preso
    /// continua nele quando a seleção muda, e um clique fora não o fecha.
    #[must_use]
    pub fn picker_subject(&self) -> Option<u64> {
        self.card_lists.picker_subject
    }

    /// O sujeito preso sobre o qual caiu a última pressão (`None` = os pedidos são da seleção).
    #[must_use]
    pub fn card_press_subject(&self) -> Option<u64> {
        self.card_lists.press_subject
    }

    /// De quem é este evento: o commit do campo que sai do foco é do preso onde foi escrito, mesmo
    /// que a pressão que o tirou de lá tenha caído noutro sítio.
    #[must_use]
    pub fn card_event_subject(&self, ev: WidgetEvent) -> Option<u64> {
        match (self.card_lists.blur_subject, ev) {
            (
                Some((f, s)),
                WidgetEvent::ValueChanged(id)
                | WidgetEvent::TextChanged(id)
                | WidgetEvent::Blur(id),
            ) if f == id => Some(s),
            _ => self.card_lists.press_subject,
        }
    }

    /// O preso das edições deste quadro (`None` = são da seleção): o do commit que saiu do foco, ou
    /// o da pressão. A guarda do espalhamento e a escolha das formas perguntam isto.
    #[must_use]
    pub fn card_edit_subject(&self) -> Option<u64> {
        self.card_lists
            .blur_subject
            .map(|(_, s)| s)
            .or(self.card_lists.press_subject)
    }

    /// Por onde uma edição do Inspector se espalha: os `n` da multi-seleção; num gesto de cartão
    /// preso, por ninguém (a edição leva os `entity_bits` dele). Vazio = só o da edição.
    pub fn card_fan_out(&self, n: usize, selecao: impl Iterator<Item = u64>) -> Vec<u64> {
        if n > 1 && self.card_edit_subject().is_none() {
            selecao.collect()
        } else {
            Vec::new()
        }
    }

    /// O quadro acabou de aplicar as edições: o commit que saiu do foco já foi servido.
    pub fn end_card_edit_frame(&mut self) {
        self.card_lists.blur_subject = None;
    }
}

/// Um solto por resolver: `(secção, bits do alfinete, chave durável)`.
pub type LoosePinKey = (NodeId, Option<u64>, Option<(u64, String)>);
