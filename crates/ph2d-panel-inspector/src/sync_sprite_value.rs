//! **Os dois sliders-com-chip da sprite** — Opacidade e Emissive.
//!
//! # Por que um ficheiro irmão
//!
//! ⚠️ A linha `Emissive` (plano
//! [`docs/Sprite_projeto/18`](../../../docs/Sprite_projeto/18_precisao_de_16_bits_nas_sprites.md)
//! W8) empurrou o `sync_sprite_fields` para **223** LOC contra uma tolerância de 202, e a catraca
//! deste projeto **só desce** (`architecture_panel_loc_cap`). Levar só o bloco novo baixaria de
//! volta a 202 e deixaria o problema inteiro para o próximo; levar **os dois** — que são a mesma
//! coisa, um slider com chip ligada — faz a mãe cair para debaixo do tecto de 200 e a tolerância
//! **deixa de ter objeto**.
//!
//! *A cura de um tecto estourado é o corte; subir o número é adiar com juros.*
//!
//! # A lei que os dois partilham
//!
//! ⚠️ **Não sincronizar enquanto o artista está a mexer.** Enquanto o slider é arrastado ou a chip
//! está focada/em scrub, o vivo é o **gesto**, não o snapshot — escrever por cima devolveria o
//! controlo ao valor de antes a cada quadro, e o arrasto tremeria.

use ph2d_a11y::NodeId;
use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::InteractiveState;
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::screens::hero::InspectorSpriteInfo;
use ph2d_editor_core::widget::SliderState;

/// Espelha os dois sliders-com-chip a partir do snapshot.
pub(crate) fn sync_sprite_sliders(
    host: &mut dyn PanelHostInternal,
    sp: &InspectorSpriteInfo,
    focus: Option<NodeId>,
    drag: Option<NodeId>,
) {
    // Opacity Slider (0..1 storage) + linked percent chip. Skip while
    // the slider is being dragged or the chip is focused so we don't
    // fight the user's input.
    let dragging = matches!(
        host.store().slider(ids::INSP_SPRITE_OPACITY),
        Some((SliderState::Dragging, _))
    );
    if !dragging
        && focus != Some(ids::INSP_SPRITE_OPACITY_CHIP)
        && drag != Some(ids::INSP_SPRITE_OPACITY_CHIP)
    {
        // The slider track can't show "Mixed", so it parks at the
        // primary's value; the blank percent chip is the Mixed signal.
        if let Some(InteractiveState::Slider { value, .. }) =
            host.store_mut().get_mut(ids::INSP_SPRITE_OPACITY)
        {
            *value = sp.opacity;
        }
        if sp.mixed.opacity {
            host.store_mut()
                .blank_number_input(ids::INSP_SPRITE_OPACITY_CHIP);
        } else {
            // Chip lives in display space (percent) per the integer map.
            host.store_mut().set_number_value(
                ids::INSP_SPRITE_OPACITY_CHIP,
                (sp.opacity * 100.0) as f64, // LITERAL-PX-OK: opacity percent scale
            );
        }
    }
    // **EMISSIVE** — a sprite como fonte de luz (plano `docs/Sprite_projeto/18` W8). Mesma dança da
    // Opacidade, e pela mesma razão: enquanto o artista arrasta, o vivo é o gesto dele, não o
    // snapshot — sincronizar por cima devolveria o slider ao valor de antes a cada quadro.
    let em_dragging = matches!(
        host.store().slider(ids::INSP_SPRITE_EMISSIVE),
        Some((SliderState::Dragging, _))
    );
    if !em_dragging
        && focus != Some(ids::INSP_SPRITE_EMISSIVE_CHIP)
        && drag != Some(ids::INSP_SPRITE_EMISSIVE_CHIP)
    {
        // ⚠️ O slider guarda NORMALIZADO; o `emissive` do snapshot é a intensidade real.
        let normalized = (sp.emissive / ph2d_editor_core::EMISSIVE_MAX_UI).clamp(0.0, 1.0);
        if let Some(InteractiveState::Slider { value, .. }) =
            host.store_mut().get_mut(ids::INSP_SPRITE_EMISSIVE)
        {
            *value = normalized;
        }
        // ⚠️ **A MESMA honestidade da Opacidade, quinze linhas acima** — e ela faltava aqui. O
        // slider não sabe dizer «misto» (ele é uma posição), por isso estaciona no valor da
        // primária; quem dá o sinal é o **chip em branco**. Sem isto, o artista via a emissão da
        // primária a falar por toda a seleção (auditoria `docs/Sprite_projeto/20` §3.1).
        if sp.mixed.emissive {
            host.store_mut()
                .blank_number_input(ids::INSP_SPRITE_EMISSIVE_CHIP);
        } else {
            host.store_mut()
                .set_number_value(ids::INSP_SPRITE_EMISSIVE_CHIP, f64::from(sp.emissive));
        }
    }
}
