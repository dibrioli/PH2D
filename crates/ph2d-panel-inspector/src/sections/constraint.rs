//! **O cartão CONSTRAINTS** (`docs/BlenderConstraints/01_plano.md` §4): a pilha do objecto, de cima
//! para baixo — por item o tipo, «Enabled», subir · descer · apagar, o alvo (quando o tipo o pede),
//! a força, os ajustes do tipo (números, escolhas, «Reset Offset») e a frase do estado; no fim o «Add
//! Constraint» com a lista dos tipos.
//!
//! Os valores pintados vêm do INSTANTÂNEO; a barra da força só mostra o store enquanto o dedo a
//! segura.

use super::*;
use crate::plano::{Plano, emoldurada};
use crate::state::MenuAdd;
use ph2d_editor_core::constraint_edits::{
    InspectorConstraintRow, InspectorConstraintsInfo, Peca, TIPO_BEHAVIOUR, TIPOS,
};
use ph2d_editor_core::property_row::{
    Seccao, abaixo_do_botao, caixa_do_botao, paint_check_row, paint_choice_row, paint_label_row,
};
use ph2d_editor_core::widget::SliderOrientation;
use ph2d_i18n::{TextKey, tr};

/// Os botões agrupados pela porta do grupo, quebrados pelas palavras. Devolve o `y` seguinte.
#[allow(clippy::too_many_arguments)]
fn fileira(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    botoes: &[(NodeId, &str)],
) -> f32 {
    let rotulos: Vec<&str> = botoes.iter().map(|(_, l)| *l).collect();
    // Quebra pelas palavras: no dock estreito «Move Up | Move Down | Delete» não cabe numa fileira.
    let seg = ph2d_editor_core::widget::wrapped_cells_in_font(
        Rect::new(x, y, w, 0.0),
        &rotulos,
        ALTURA_DE_CAMPO,
        usize::MAX,
        ph2d_editor_core::widget::button_label_font(),
        text_system,
    );
    let mut fundo = y;
    for (&(id, rotulo), (rect, group)) in botoes.iter().zip(seg.into_iter().flatten()) {
        hit_index.register(id, rect);
        paint_button(
            &Button::new(id, rotulo)
                .kind(ButtonKind::Default)
                .visual(store.button_visual(id))
                .in_group(group),
            rect,
            scene,
            text_system,
            theme,
        );
        fundo = fundo.max(rect.y + rect.h);
    }
    fundo + ph2d_tokens::control_gap_px()
}

/// A linha do ALVO (ou do objecto do peso): o nome (ou «None») e, por baixo, «Pick» — premido
/// enquanto escolhe.
#[allow(clippy::too_many_arguments)]
fn alvo(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    (x, w, y): (f32, f32, f32),
    (rotulo, nome, nada, escolhendo): (&str, Option<&str>, &str, bool),
    id: NodeId,
    seccao: Seccao,
) -> f32 {
    let linha = paint_label_row(
        scene,
        text_system,
        theme,
        x,
        w,
        y,
        ROW_H_PX,
        tr(rotulo),
        seccao,
    );
    let font = TypeToken::Sm.px();
    let (nome, cor) = match nome {
        Some(n) => (n, ColorToken::Text1),
        None => (tr(nada), ColorToken::Text3),
    };
    paint_text(
        text_system,
        scene,
        nome,
        linha.control.x,
        linha.control.y + (linha.control.h - font) * 0.5,
        font,
        linha.control.w,
        resolve(cor, theme),
    );
    let cur_y = y + ph2d_tokens::row_pitch_px();
    let rotulo = tr(if escolhendo {
        "panel.inspector.constraint.picking"
    } else {
        "panel.inspector.constraint.pick"
    });
    let caixa = caixa_do_botao(text_system, x, w, cur_y, rotulo);
    hit_index.register(id, caixa);
    let botao = Button::new(id, rotulo).kind(ButtonKind::Default);
    let botao = if escolhendo {
        botao.state(ButtonState::Pressed)
    } else {
        botao.visual(store.button_visual(id))
    };
    paint_button(&botao, caixa, scene, text_system, theme);
    abaixo_do_botao(caixa)
}

/// A barra da FORÇA (`0..1`) com o losango do Auto Key na coluna do decorator.
#[allow(clippy::too_many_arguments)]
fn forca(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    (x, w, y): (f32, f32, f32),
    row: &InspectorConstraintRow,
    k: usize,
    seccao: Seccao,
) -> f32 {
    let id = ids::INSP_CONSTRAINT_FORCA[k];
    let chave = TextKey::new("panel.inspector.constraint.influence");
    let linha = paint_label_row(
        scene,
        text_system,
        theme,
        x,
        w,
        y,
        ROW_H_PX,
        chave.tr(),
        seccao,
    );
    let visual = store.slider_visual(id);
    let valor = match store.slider(id) {
        Some((SliderState::Dragging, v)) => v,
        _ => row.forca,
    };
    hit_index.register(id, linha.control);
    ph2d_editor_core::widget::paint_slider_track(
        linha.control,
        valor,
        SliderOrientation::Horizontal,
        visual,
        scene,
        theme,
    );
    let font = TypeToken::Sm.px();
    paint_text(
        text_system,
        scene,
        &format!("{valor:.2}"),
        linha.control.x + Spacing::Sm.px(),
        linha.control.y + (linha.control.h - font) * 0.5,
        font,
        linha.control.w,
        resolve(ColorToken::Text1, theme),
    );
    if let Some(alvo) = ph2d_editor_core::widget::key_mark::paint_decorator_campo(
        scene,
        theme,
        linha.dot,
        &[ids::INSP_CONSTRAINT_FORCA[k]],
        chave.into(),
    ) {
        hit_index.register(alvo, linha.dot);
    }
    y + ph2d_tokens::row_pitch_px()
}

/// Os ajustes do tipo (os números, as escolhas e o «Reset Offset»), na ordem que a família deu.
#[allow(clippy::too_many_arguments)]
fn ajustes(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    (x, w, y): (f32, f32, f32),
    row: &InspectorConstraintRow,
    k: usize,
    seccao: Seccao,
) -> f32 {
    let mut cur_y = y;
    // As peças pela ordem que a família deu (o Follow Value intercala os ▾ com as faixas).
    for peca in row.pecas() {
        let (j, n, e) = match peca {
            Peca::Numero(j) => (usize::from(j), row.numeros.get(usize::from(j)), None),
            Peca::Escolha(j) => (usize::from(j), None, row.escolhas.get(usize::from(j))),
            Peca::Propriedade(j) => {
                if let Some(p) = row.propriedades.get(usize::from(j)) {
                    cur_y = super::constraint_valor::propriedade(
                        scene,
                        text_system,
                        theme,
                        (hit_index, store),
                        (x, w, cur_y),
                        (k, usize::from(j)),
                        p,
                        seccao,
                    );
                }
                continue;
            }
        };
        if let (Some(n), Some(ids_n)) = (n, ids::INSP_CONSTRAINT_NUMERO.get(j)) {
            let (unit, step) = crate::sync_constraint::unidade_e_passo(n.unidade);
            cur_y = ph2d_editor_core::property_row::paint_field_row(
                scene,
                text_system,
                theme,
                hit_index,
                store,
                x,
                w,
                cur_y,
                TextKey::new(n.rotulo),
                ids_n[k],
                step,
                unit,
                seccao,
            )
            .0;
        }
        let (Some(e), Some(ids_e)) = (e, ids::INSP_CONSTRAINT_OPCAO.get(j)) else {
            continue;
        };
        let segmentos: Vec<(&str, bool, NodeId)> = e
            .opcoes
            .iter()
            .zip(ids_e.iter())
            .enumerate()
            .map(|(o, (chave, tabela))| (tr(chave), o == usize::from(e.valor), tabela[k]))
            .collect();
        cur_y = paint_choice_row(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            cur_y,
            TextKey::new(e.rotulo),
            &segmentos,
            seccao,
        );
    }
    if row.zera_desvio {
        let id = ids::INSP_CONSTRAINT_ZERAR[k];
        let rotulo = tr("panel.inspector.constraint.reset_offset");
        let caixa = caixa_do_botao(text_system, x, w, cur_y, rotulo);
        hit_index.register(id, caixa);
        paint_button(
            &Button::new(id, rotulo)
                .kind(ButtonKind::Default)
                .visual(store.button_visual(id)),
            caixa,
            scene,
            text_system,
            theme,
        );
        cur_y = abaixo_do_botao(caixa);
    }
    cur_y
}

/// Um item da pilha, na posição `k`. Devolve o `y` seguinte.
#[allow(clippy::too_many_arguments)]
fn item(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    (x, w, y): (f32, f32, f32),
    k: usize,
    row: &InspectorConstraintRow,
    seccao: Seccao,
) -> f32 {
    let font = TypeToken::Md.px();
    let tipo = TIPOS.get(usize::from(row.tipo)).map_or("", |c| tr(c));
    paint_text(
        text_system,
        scene,
        tipo,
        x,
        y + (ROW_H_PX - font) * 0.5,
        font,
        w,
        resolve(ColorToken::Text1, theme),
    );
    let mut cur_y = y + ph2d_tokens::row_pitch_px();
    cur_y = paint_check_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        (
            ids::INSP_CONSTRAINT_LIGADA[k],
            TextKey::new("panel.inspector.constraint.enabled"),
            row.ligada,
        ),
        seccao,
    );
    cur_y = fileira(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        &[
            (
                ids::INSP_CONSTRAINT_SUBIR[k],
                tr("panel.inspector.constraint.up"),
            ),
            (
                ids::INSP_CONSTRAINT_DESCER[k],
                tr("panel.inspector.constraint.down"),
            ),
            (
                ids::INSP_CONSTRAINT_APAGAR[k],
                tr("panel.inspector.constraint.delete"),
            ),
        ],
    );
    if row.precisa_alvo {
        cur_y = alvo(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            (x, w, cur_y),
            (
                "panel.inspector.constraint.target",
                row.alvo.as_deref(),
                "panel.inspector.constraint.no_target",
                row.escolhendo,
            ),
            ids::INSP_CONSTRAINT_ALVO[k],
            seccao,
        );
    }
    if let Some(p) = &row.peso {
        cur_y = alvo(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            (x, w, cur_y),
            (
                "panel.inspector.constraint.weight_by",
                p.nome.as_deref(),
                "panel.inspector.constraint.no_weight",
                p.escolhendo,
            ),
            ids::INSP_CONSTRAINT_PESO[k],
            seccao,
        );
        if p.nome.is_some() {
            let id = ids::INSP_CONSTRAINT_SEM_PESO[k];
            let rotulo = tr("panel.inspector.constraint.weight_clear");
            let caixa = caixa_do_botao(text_system, x, w, cur_y, rotulo);
            hit_index.register(id, caixa);
            let b = Button::new(id, rotulo)
                .kind(ButtonKind::Default)
                .visual(store.button_visual(id));
            paint_button(&b, caixa, scene, text_system, theme);
            cur_y = abaixo_do_botao(caixa);
        }
    }
    cur_y = forca(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        (x, w, cur_y),
        row,
        k,
        seccao,
    );
    cur_y = ajustes(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        (x, w, cur_y),
        row,
        k,
        seccao,
    );
    if let Some(frase) = row.estado.frase() {
        let cor = if row.estado.erro() {
            ColorToken::Warn
        } else {
            ColorToken::Text3
        };
        cur_y = super::rows::aviso(scene, text_system, theme, x, w, cur_y, tr(frase), cor);
    }
    cur_y + ph2d_tokens::control_gap_px()
}

/// O «Bake» (com itens) e o «Add Constraint» e, aberta, a lista dos tipos. Cheio, a frase do tecto
/// em vez do «Add».
#[allow(clippy::too_many_arguments)]
fn acrescentar(
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

/// Pinta o cartão. Devolve o `y` seguinte.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_constraints_section(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    i: &InspectorConstraintsInfo,
    aberta: crate::state::MenuAdd,
) -> f32 {
    let (fold, mut cur_y) = match super::nav::cabecalho(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        y,
        core_ids::INSP_LIVE_CONSTRAINTS_SECTION,
        tr("component.constraints.name"),
    ) {
        Ok(v) => v,
        Err(y) => return y,
    };
    let seccao = Seccao::medida(
        text_system,
        1,
        &[
            tr("panel.inspector.constraint.enabled"),
            tr("panel.inspector.constraint.target"),
            tr("panel.inspector.constraint.influence"),
            tr("panel.inspector.constraint.distance"),
            tr("panel.inspector.constraint.offset"),
            tr("panel.inspector.constraint.space"),
        ],
    );
    if i.rows.is_empty() && i.colas.is_empty() {
        cur_y = super::rows::aviso(
            scene,
            text_system,
            theme,
            x,
            w,
            cur_y,
            tr("panel.inspector.constraint.empty"),
            ColorToken::Text3,
        );
    }
    // `zip` com a tabela de ids: uma pilha maior que o cartão perde os excedentes em vez de os
    // pintar uns sobre os outros (o «+» já recusa no tecto).
    for (k, (row, _)) in i
        .rows
        .iter()
        .zip(ids::INSP_CONSTRAINT_FORCA.iter())
        .enumerate()
    {
        cur_y = item(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            (x, w, cur_y),
            k,
            row,
            seccao,
        );
    }
    cur_y = super::constraint_cola::colas(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        (x, w, cur_y),
        i,
    );
    cur_y = acrescentar(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        (x, w, cur_y),
        i,
        aberta,
    );
    fold.finish(store, scene, hit_index, cur_y)
}

/// A moldura (o molde do `paint_optional_macro`): empurra o cartão para o [`Plano`], se o objecto
/// tiver a pilha.
pub(crate) fn push_constraints_section<'a>(
    plano: &mut Plano<'a>,
    store: &'a WidgetStore,
    inner_x: f32,
    inner_w: f32,
    header_h: f32,
    info: Option<&'a InspectorConstraintsInfo>,
    aberta: crate::state::MenuAdd,
) {
    let Some(info) = info else {
        return;
    };
    emoldurada(
        plano,
        core_ids::INSP_LIVE_CONSTRAINTS_SECTION,
        store,
        inner_x,
        inner_w,
        header_h,
        move |c, t, y| {
            paint_constraints_section(
                c.scene, c.text, t, c.hit, store, inner_x, inner_w, y, info, aberta,
            )
        },
    );
}
