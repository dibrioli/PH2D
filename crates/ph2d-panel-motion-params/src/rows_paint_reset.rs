//! **A seta que devolve um param ao default** — a calha à direita de toda row.
//!
//! Um painel de params sem isto é um painel em que experimentar custa caro: o artista arrasta
//! um knob, não gosta, e não tem como voltar sem lembrar o número. Todo DCC resolve isso
//! (Blender por menu de contexto, Unreal/Godot por uma seta que aparece só no que foi mexido);
//! aqui é a seta, porque ela é um BOTÃO — ninguém precisa descobrir que a coisa é clicável.
//!
//! ⚠️ **A calha é reservada SEMPRE e a seta é desenhada só quando há override.** Se a largura
//! da row dependesse do estado, cada linha se mexeria no instante em que fosse tocada — e um
//! rótulo que muda de lugar por um motivo invisível não se aprende a achar. É por isso que
//! quem estreita o corpo é o `paint_rows`, uma vez, para os doze pintores de row.
//!
//! ⚠️ E "modificado" é **presença de chave** no override, nunca `valor != default`: o grafo
//! guarda overrides esparsos, então digitar de volta o número do default continua sendo uma
//! escolha do artista — e a seta some ao clicar porque a CHAVE saiu, não porque um `f32` bateu.

use crate::ParamRow;
use crate::snapshot::param_reset_id;
use ph2d_editor_core::icons::IconId;
use ph2d_editor_core::interaction::{HitIndex, WidgetStore};
use ph2d_editor_core::widget::{ButtonState, IconButtonStyle, IconGlyph, paint_icon_button};
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{ROW_H_PX, Theme};
use ph2d_vector::VectorScene;
use std::collections::BTreeSet;

/// Largura da calha que toda row cede para a seta (a seta é quadrada, altura de row).
pub(crate) const RESET_GUTTER_W: f32 = ROW_H_PX;

/// Esta row carrega algum override?
pub(crate) fn row_is_modified(row: &ParamRow, modified: &BTreeSet<String>) -> bool {
    row.params().iter().any(|p| modified.contains(*p))
}

/// Desenha e registra a seta na calha, se houver o que reverter. `y` é o topo da PRIMEIRA
/// linha da row — mesmo num editor de várias linhas, a seta mora ao lado do rótulo, que é
/// onde o nome do param está.
#[expect(
    clippy::too_many_arguments,
    reason = "espelha a porta de paint das rows deste painel"
)]
pub(crate) fn paint_reset_button(
    slot: usize,
    inner_x: f32,
    inner_w: f32,
    y: f32,
    store: &WidgetStore,
    hit_index: &mut HitIndex,
    scene: &mut VectorScene,
    theme: Theme,
) {
    let id = param_reset_id(slot);
    let rect = Rect::new(
        inner_x + inner_w - RESET_GUTTER_W,
        y,
        RESET_GUTTER_W,
        ROW_H_PX,
    );
    paint_icon_button(
        rect,
        IconGlyph::Builtin(IconId::Reset),
        IconButtonStyle::Plain,
        store.button_state(id).unwrap_or(ButtonState::Normal),
        scene,
        theme,
    );
    hit_index.register(id, rect);
}
