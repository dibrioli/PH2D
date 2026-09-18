//! **O CHIP COM NOME do modo `Fixed`** — a largura e a altura, lado a lado.
//!
//! ⚠️ Irmão de [`super::paint`] por RESPONSABILIDADE e não por contagem: lá fica *que secções este
//! painel tem*, aqui *como se desenha um chip com nome*. O corte foi forçado pelo tecto de LOC da
//! workspace (600 para `crates/ph2d-panel-*`) e é melhor do que o ficheiro era — o pintor do chip
//! não tem nada a ver com a ordem das secções.

use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::{HitIndex, InteractiveState, WidgetStore};
use ph2d_editor_core::paint::{paint_text, resolve};
use ph2d_editor_core::widget::TextInputState;
use ph2d_editor_core::widget::paint_number_chip;
use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Spacing, Theme, TypeToken};
use ph2d_vector::VectorScene;

/// Paint a label + NumberInput chip pair on one row (no slider). The
/// chip uses the stored number_value (already mirrored by the host on
/// **Um rótulo por cima do seu campo numérico**, e a altura que ele de facto usou.
///
/// ⚠️ **Isto reusava o painter de SLIDER com um track de largura zero**, e o comentário de então
/// admitia o truque: *"o slider colapsa atrás da coluna do rótulo"*. Ele colapsava **enquanto a
/// row coubesse numa linha**. Num painel estreito — que é o caso destas duas metades — o painter
/// adaptativo EMPILHA, e aí o truque desmonta-se de duas maneiras ao mesmo tempo (Enio,
/// 2026-08-19: *"o painel de equalize sizes: fixed está todo embolado"*):
///
/// 1. o track deixa de estar escondido atrás do rótulo e desenha-se **à largura toda** — o
///    retângulo preto à esquerda do `256`;
/// 2. a row passa a ocupar **duas** linhas, e quem a chamava avançava `y` por **uma** — a linha
///    seguinte (`Upscale if smaller`) caía por cima dos campos.
///
/// A cura não é medir melhor a altura: é **não pedir um slider quando não há slider**. O
/// `paint_number_chip` existe exatamente para isto — o doc dele diz *"callable directly when a
/// chip needs to live somewhere a slider row layout doesn't fit"*.
///
/// *Um componente reusado com um dos seus eixos posto a zero não é reuso; é um caso especial à
/// espera do primeiro layout que não o respeite.*
#[allow(clippy::too_many_arguments)]
/// ⚠️⚠️ **O rótulo entra TIPADO (`TextKey`) e não como `&str`, e isso é a cerca.**
///
/// Até 2026-09-18 ele era um `&str` e os dois chamadores passavam `"W"` e `"H"` **crus** — com o
/// censo desta crate VERDE, porque o `is_language` exige duas letras SEGUIDAS (senão acusaria todo
/// identificador) e uma letra sozinha não tem forma que a distinga de uma. ⇒ *quando a régua não
/// consegue ver a diferença, quem a vê é o TIPO*: com este parâmetro, escrever `"W"` aqui deixa de
/// compilar. É a lei que a memória desta casa já regista — *chave e texto do mesmo tipo é um
/// defeito à espera*.
pub(crate) fn paint_labeled_chip(
    rect: Rect,
    label: ph2d_i18n::TextKey,
    chip_id: NodeId,
    value: f64,
    store: &WidgetStore,
    hit_index: &mut HitIndex,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) -> f32 {
    let font = TypeToken::Xs.px();
    let label_h = font + Spacing::Xs.px();
    paint_text(
        text_system,
        scene,
        label.tr(),
        rect.x,
        rect.y,
        font,
        rect.w,
        resolve(ColorToken::Text2, theme),
    );
    let chip_rect = Rect::new(rect.x, rect.y + label_h, rect.w, rect.h);
    // O estado vem do store para que a escrita, o cursor e a seleção sejam vivos — a mesma
    // leitura que o `paint_slider_with_chip_layout` faz do seu chip.
    let (state, buffer, caret, anchor) = match store.get(chip_id) {
        Some(InteractiveState::NumberInput {
            state,
            buffer,
            caret,
            selection_anchor,
            ..
        }) => (*state, Some(buffer.as_str()), *caret, *selection_anchor),
        _ => (TextInputState::Normal, None, 0, None),
    };
    let display = value.round().to_string();
    paint_number_chip(
        chip_rect,
        state,
        value,
        Some(&display),
        buffer,
        caret,
        anchor,
        scene,
        text_system,
        theme,
    );
    hit_index.register(chip_id, chip_rect);
    label_h + rect.h
}
