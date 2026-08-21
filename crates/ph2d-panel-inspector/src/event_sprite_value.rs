//! **Os dois sliders-com-chip da sprite** — Opacidade e Emissive, do lado do EVENTO.
//!
//! # Por que um ficheiro irmão
//!
//! ⚠️ A linha `Emissive` (plano
//! [`docs/Sprite_projeto/18`](../../../docs/Sprite_projeto/18_precisao_de_16_bits_nas_sprites.md)
//! W8) empurrou o `apply_event_impl` para **433** LOC contra uma tolerância de 410. Levar só o braço
//! novo devolveria o número a 410 exactos — e a lei escrita naquele gate é que *as tolerâncias
//! encolhem, nunca crescem*, e ficar no mesmo sítio não é encolher. Os dois saem juntos porque são
//! **a mesma coisa**: um slider `0..1` com uma chip que mostra a unidade humana.
//!
//! Irmão de [`crate::event_precision`], que já levou os dois pares segmentados da mesma secção.
//!
//! # A assimetria entre os dois, e ela é deliberada
//!
//! ⚠️ **A Opacidade viaja como campo do `Sprite`; a Emissão, como componente próprio.** Opacidade é
//! um campo do `Sprite` e vai por `SpriteFieldEdit`; `SpriteEmissive` é um componente **opcional**
//! do ECS, e quem o insere ou remove é o shell. É por isso que o segundo braço levanta uma
//! `EditorAction` própria em vez de mais uma variante do `SpriteFieldEdit` — *o canal segue a forma
//! do dado, não a do controlo*.

use crate::state;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::screens::hero::SpriteFieldEdit;

/// Despacha os dois sliders. `true` = o evento foi consumido.
pub(crate) fn apply_sprite_slider_event(host: &mut dyn PanelHostInternal, ev: WidgetEvent) -> bool {
    // W2 Color & Tint — Opacity Slider moved (drag or linked-chip edit
    // both fire ValueChanged on the slider id). The slider stores the
    // raw 0..1 opacity.
    if let WidgetEvent::ValueChanged(id) = ev
        && id == ids::INSP_SPRITE_OPACITY
        && let Some(info) = state::current_inspector_sprite()
    {
        let opacity = host
            .store()
            .slider(ids::INSP_SPRITE_OPACITY)
            .map(|(_, v)| v)
            .unwrap_or(info.opacity);
        host.bus_mut().push(EditorAction::InspectorSpriteEdit {
            entity_bits: info.entity_bits,
            edit: SpriteFieldEdit::Opacity(opacity),
        });
        return true;
    }
    // **EMISSIVE** — a sprite como fonte de luz (plano `docs/Sprite_projeto/18` W8). O slider guarda
    // `0..1`; o que viaja no barramento é a **intensidade real**, porque quem a consome é o ECS e o
    // motor não sabe nada sobre o curso de um slider.
    //
    // ⚠️ **Zero é uma edição legítima** — é como se apaga a emissão —, por isso este braço não
    // curto-circuita em zero como o par `Format` faz. O que ele não pode é disparar sem sprite
    // selecionada.
    if let WidgetEvent::ValueChanged(id) = ev
        && id == ids::INSP_SPRITE_EMISSIVE
        && let Some(info) = state::current_inspector_sprite()
    {
        let normalized = host
            .store()
            .slider(ids::INSP_SPRITE_EMISSIVE)
            .map(|(_, v)| v)
            .unwrap_or(0.0);
        host.bus_mut()
            .push(EditorAction::InspectorSpriteEmissiveChange {
                entity_bits: info.entity_bits,
                intensity: normalized * ph2d_editor_core::EMISSIVE_MAX_UI,
            });
        return true;
    }
    false
}
