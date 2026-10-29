//! **O «Add Constraint» do cartão CONSTRAINTS** (`docs/BlenderConstraints/01_plano.md` §4): o «Bake», o
//! botão e, aberto, a lista dos tipos com o submenu dos *Behaviours*. Irmão do `constraint.rs` (tecto de
//! LOC); o corte é por responsabilidade — aqui só o menu que ACRESCENTA, ali a pilha que existe.

use super::*;
use crate::state::MenuAdd;
use ph2d_editor_core::constraint_edits::{InspectorConstraintsInfo, TIPO_BEHAVIOUR, TIPOS};
use ph2d_editor_core::property_row::{abaixo_do_botao, caixa_do_botao};
use ph2d_i18n::tr;

/// O «Bake» (com itens) e o «Add Constraint» e, aberta, a lista dos tipos. Cheio, a frase do tecto
/// em vez do «Add».
#[allow(clippy::too_many_arguments)]
pub(super) fn acrescentar(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    (x, w, y): (f32, f32, f32),
    i: &InspectorConstraintsInfo,
    aberta: crate::state::MenuAdd,
) -> f32 {
    let mut y = y;
    if !i.rows.is_empty() {
        let id = ids::INSP_CONSTRAINT_BAKE;
        let rotulo = tr("panel.inspector.constraint.bake");
        let caixa = caixa_do_botao(text_system, x, w, y, rotulo);
        hit_index.register(id, caixa);
        let botao = Button::new(id, rotulo).kind(ButtonKind::Default);
        paint_button(
            &botao.visual(store.button_visual(id)),
            caixa,
            scene,
            text_system,
            theme,
        );
        y = abaixo_do_botao(caixa);
    }
    if i.rows.len() >= ids::INSP_CONSTRAINT_FORCA.len() {
        return super::rows::aviso(
            scene,
            text_system,
            theme,
            x,
            w,
            y,
            tr("panel.inspector.constraint.full"),
            ColorToken::Text3,
        );
    }
    let id = ids::INSP_CONSTRAINT_ADD;
    let rotulo = tr("panel.inspector.constraint.add");
    let caixa = caixa_do_botao(text_system, x, w, y, rotulo);
    hit_index.register(id, caixa);
    let botao = Button::new(id, rotulo).kind(ButtonKind::Default);
    let botao = if aberta != MenuAdd::Fechado {
        botao.state(ButtonState::Pressed)
    } else {
        botao.visual(store.button_visual(id))
    };
    paint_button(&botao, caixa, scene, text_system, theme);
    let mut cur_y = abaixo_do_botao(caixa);
    if aberta == MenuAdd::Fechado {
        return cur_y;
    }
    let mut botao = |id, rotulo: &str, x: f32, w: f32, cur_y: f32, premido: bool| {
        let caixa = caixa_do_botao(text_system, x, w, cur_y, rotulo);
        hit_index.register(id, caixa);
        let b = Button::new(id, rotulo).kind(ButtonKind::Default);
        let b = if premido {
            b.state(ButtonState::Pressed)
        } else {
            b.visual(store.button_visual(id))
        };
        paint_button(&b, caixa, scene, text_system, theme);
        abaixo_do_botao(caixa)
    };
    let (tipos, behaviours) = ids::INSP_CONSTRAINT_TIPO.split_at(usize::from(TIPO_BEHAVIOUR));
    for (&id, chave) in tipos.iter().zip(TIPOS) {
        cur_y = botao(id, tr(chave), x, w, cur_y, false);
    }
    // O submenu (decisão do dono 27/10): um botão que abre os *behaviours* por baixo, recuados.
    let sub = aberta == MenuAdd::Behaviours;
    let id = ids::INSP_CONSTRAINT_BEHAVIOURS;
    cur_y = botao(
        id,
        tr("panel.inspector.constraint.behaviours"),
        x,
        w,
        cur_y,
        sub,
    );
    if sub {
        let recuo = ph2d_tokens::list_indent_px();
        let chaves = &TIPOS[usize::from(TIPO_BEHAVIOUR)..];
        for (&id, chave) in behaviours.iter().zip(chaves) {
            cur_y = botao(id, tr(chave), x + recuo, w - recuo, cur_y, false);
        }
    }
    cur_y
}
