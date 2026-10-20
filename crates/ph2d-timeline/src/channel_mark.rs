//! ⭐⭐ **O ESTADO DE UM CANAL NO INSTANTE** — o que o losango ao lado de um campo do painel diz
//! (plano 34 §3; o *decorator* do Blender, o losango do Rive).
//!
//! Uma pergunta só para todo painel: o campo de opacidade do vetor, o X do Inspector e o alvo do
//! motor de uma junta perguntam aqui, e o diff do auto-key compara pela MESMA curva
//! ([`crate::autokey::curve_seen`]) — duas portas discordariam de «saiu da curva».

use crate::{PropKind, SelectedKey, TimelineDoc};

/// O que o losango mostra.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChannelMark {
    /// Nenhuma curva conduz este canal — sem losango.
    Livre,
    /// Uma curva conduz; não há chave neste quadro (losango vazio).
    Animado,
    /// Há uma chave neste quadro (losango cheio) — a chave, para o clique a apagar.
    Chave(SelectedKey),
    /// O mundo saiu da curva e ninguém o gravou (auto-key desarmado): perde-se quando o tempo
    /// mudar — o laranja do Blender.
    Mudado,
}

/// O estado de `(entity, prop)` no instante de fonte `t_src` (o relógio em que o auto-key compara e
/// a tecla K grava). `solo` = a vista Keys (a curva do clip activo); senão a pilha. `world` = o
/// valor que o mundo tem agora (`None` = o canal não tem escalar de mundo).
///
/// «Neste quadro» = a menos de meio quadro de exibição (`fps_display`): uma chave posta com o snap
/// desligado continua a ser «a chave deste quadro» para quem olha a régua.
#[must_use]
pub fn channel_mark(
    doc: &TimelineDoc,
    entity: u64,
    prop: PropKind,
    t_src: f64,
    solo: bool,
    world: Option<f32>,
) -> ChannelMark {
    let Some(b) = doc.binding_for(entity, prop) else {
        return ChannelMark::Livre;
    };
    let Some(track) = doc.active_clip().track(b.target).filter(|t| !t.is_empty()) else {
        return ChannelMark::Livre;
    };
    let curva = crate::autokey::curve_seen(doc, entity, prop, t_src, solo);
    // ⚠️ `!=` exacto, a regra do diff do auto-key: o apply escreve no mundo o próprio `f32` da curva.
    if let (Some(v), Some(c)) = (world, curva)
        && v != c
    {
        return ChannelMark::Mudado;
    }
    let meio = if doc.fps_display.is_finite() && doc.fps_display > 0.0 {
        0.5 / doc.fps_display
    } else {
        0.0
    };
    track
        .keys()
        .iter()
        .zip(track.ids())
        .find(|(k, _)| (k.t.to_seconds() - t_src).abs() <= meio)
        .map_or(ChannelMark::Animado, |(_, &key)| {
            ChannelMark::Chave(SelectedKey {
                target: b.target,
                key,
            })
        })
}

impl TimelineDoc {
    /// ⭐ **O relógio do AUTOR, corte incluído** — o instante em que o auto-key compara, a tecla K
    /// grava e o losango responde. Antes de tudo o apply corta todo relógio na duração autorada da
    /// vista: a vista Keys (e o Arrange de pilha vazia) pelo clip activo, um contentor aberto pelo
    /// dele, a cena pela dela. Depois do corte a pose congela em `curva(corte)`; quem lesse
    /// `curva(t_raw)` veria uma diferença fantasma. `t_src` de cada objecto compõe-se POR CIMA disto
    /// (`remapped_time`).
    #[must_use]
    pub fn author_cut(&self, solo: bool, root: Option<usize>, t_raw: f64) -> f64 {
        if solo || (root.is_none() && self.stack().is_empty()) {
            self.clip_cut(self.active_index(), t_raw)
        } else if let Some(c) = root {
            self.container_cut(c, t_raw)
        } else {
            self.cut_scene(t_raw)
        }
    }
}

#[cfg(test)]
#[path = "channel_mark_tests.rs"]
mod tests;
