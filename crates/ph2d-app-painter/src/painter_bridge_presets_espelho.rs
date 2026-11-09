//! **O nome no menu é o pincel na mão** (Bug #83, smoke do dono 10/11). O «aplicado» é por GAVETA (ferramenta ×
//! meio), mas o pincel do tool é por SLOT: o Digital, a Watercolor e o Impasto do Pincel partilham um, e com «Sync
//! with other tools» todas as ferramentas partilham um só. Um preset escolhido numa gaveta substituía o pincel da
//! outra e o menu dela continuava a nomear o de antes. Aqui lembra-se que gaveta pôs o preset em cada slot; ao
//! voltar a uma gaveta cujo pincel outra substituiu, o preset dela volta (ou, com os pincéis ligados — um só pincel
//! para todas, escolha do artista —, o menu deixa de nomear um preset que não está na mão).

use super::Presets;
use ph2d_editor_core::toast::ToastQueue;
use ph2d_tool_painter::{Gaveta, PainterTool};
use std::collections::BTreeMap;

/// Que gaveta pôs o preset que está em cada slot do pincel.
#[derive(Default)]
pub(super) struct DonoDoPincel {
    /// Slot → a chave da gaveta (os pincéis independentes).
    por_slot: BTreeMap<usize, u8>,
    /// O de todos os slots sem entrada (desligar o «Sync» copia o pincel ligado para todos).
    base: Option<u8>,
    /// O do pincel ligado (um só).
    ligado: Option<u8>,
    /// O «Sync» no quadro anterior e o slot de então.
    estava_ligado: bool,
    slot: usize,
}

impl DonoDoPincel {
    fn de(&self, ligado: bool, slot: usize) -> Option<u8> {
        if ligado {
            self.ligado
        } else {
            self.por_slot.get(&slot).copied().or(self.base)
        }
    }

    /// `g` acabou de pôr um preset no pincel na mão.
    pub(super) fn pos(&mut self, painter: &PainterTool, g: Gaveta) {
        if painter.link_shared_settings() {
            self.ligado = Some(g.chave());
        } else {
            self.por_slot.insert(painter.slot_do_pincel(), g.chave());
        }
    }

    /// O «Sync» mudou: ligado, todos passam a ter o pincel de agora; desligado, cada slot fica com uma cópia dele.
    fn seguir_o_sync(&mut self, painter: &PainterTool) {
        let ligado = painter.link_shared_settings();
        if ligado != self.estava_ligado {
            if ligado {
                self.ligado = self.de(false, self.slot);
            } else {
                self.por_slot.clear();
                self.base = self.ligado;
            }
            self.estava_ligado = ligado;
        }
    }
}

impl Presets {
    /// Antes do pedido do painel: o «Sync» que mudou.
    pub(super) fn seguir_o_sync(&mut self, painter: &PainterTool) {
        self.pincel.seguir_o_sync(painter);
    }

    /// A gaveta mudou de `antes` (antes de abrir a de agora): se as duas partilham o pincel, o preset que se deixa
    /// lembra o que a política manda — o pincel ainda é o dele.
    pub(super) fn deixar_a_gaveta(&mut self, painter: &PainterTool, antes: Option<Gaveta>) {
        let ligado = painter.link_shared_settings();
        let slot = painter.slot_do_pincel();
        if let Some(a) = antes
            && (ligado || slot == self.pincel.slot)
            && self.pincel.de(ligado, slot) == Some(a.chave())
            && let Some(rel) = self.aplicado.get(&a.chave()).cloned()
        {
            let modo = self
                .estado
                .lembrados
                .get(&rel)
                .and_then(|l| l.paint_mode.clone());
            self.estado.lembrar(&rel, painter.preset_volumes(), modo);
        }
    }

    /// Na gaveta de agora (já aberta): se outra gaveta substituiu o pincel dela, volta o preset dela — ou, ligados,
    /// o menu esquece-o.
    pub(super) fn conferir_o_pincel(&mut self, painter: &mut PainterTool, toasts: &mut ToastQueue) {
        let ligado = painter.link_shared_settings();
        let Some(g) = painter.gaveta_de_presets() else {
            return;
        };
        let Some(rel) = self.aplicado.get(&g.chave()).cloned() else {
            return;
        };
        if self.pincel.de(ligado, painter.slot_do_pincel()) == Some(g.chave()) {
            return;
        }
        if ligado {
            self.aplicado.remove(&g.chave());
        } else {
            self.aplicar(painter, &rel, toasts);
        }
    }

    /// Depois de cada quadro: o slot na mão.
    pub(super) fn ver_o_slot(&mut self, painter: &PainterTool) {
        self.pincel.slot = painter.slot_do_pincel();
    }
}
