//! **O cartão MACRO** (ADR-0186, W5d; `docs/Macros/02_plano_W5.md` §5): o modo em que a trilha do
//! Motion toca (Faithful · Explanatory), o resumo da gravação, «editada desde a etapa N», o erro
//! e «Edit steps…», que abre o painel MACROS (é lá que as etapas se editam).
//!
//! Uma gravação que não se lê só diz isso: o resumo, as etapas e o erro da reprodução derivam dela.

use super::*;
use ph2d_editor_core::interaction::format_number;
use ph2d_editor_core::macro_edits::{InspectorMacroInfo, MACRO_MODOS, MacroGravacao};
use ph2d_editor_core::property_row::{Seccao, abaixo_do_botao, caixa_do_botao, paint_choice_row};
use ph2d_i18n::{tr, tr_with};

/// Os rótulos dos modos, pela ordem de `ModoDaMacro`.
const MODOS: [&str; MACRO_MODOS] = [
    "panel.inspector.macro.mode.0",
    "panel.inspector.macro.mode.1",
];
/// O que cada modo faz, pela mesma ordem.
const DICAS: [&str; MACRO_MODOS] = [
    "panel.inspector.macro.mode_hint.0",
    "panel.inspector.macro.mode_hint.1",
];

const _: () = assert!(crate::ids::INSP_MACRO_MODO.len() == MACRO_MODOS);

/// As linhas de texto do cartão, pela ordem em que se pintam, com a cor de cada uma.
fn linhas(i: &InspectorMacroInfo) -> Vec<(String, ColorToken)> {
    let (trilhas, duracao_us) = match &i.gravacao {
        MacroGravacao::Ilegivel(chave) => return vec![(tr(chave).to_owned(), ColorToken::Danger)],
        MacroGravacao::Vazia => {
            return vec![(
                tr("panel.inspector.macro.empty").to_owned(),
                ColorToken::Text3,
            )];
        }
        MacroGravacao::Lida {
            trilhas,
            duracao_us,
        } => (trilhas.len(), *duracao_us),
    };
    #[expect(
        clippy::cast_precision_loss,
        reason = "segundos de uma gravação: exacto até 2^53 µs"
    )]
    let segundos = duracao_us as f64 / 1e6;
    let mut out = vec![(
        tr_with(
            "panel.inspector.macro.summary",
            &[
                ("tracks", &trilhas),
                ("steps", &i.etapas),
                ("seconds", &format_number(segundos)),
            ],
        ),
        ColorToken::Text2,
    )];
    if let Some(n) = i.editada_desde {
        out.push((
            tr_with("panel.macros.edited_since", &[("n", &(n + 1))]),
            ColorToken::Text3,
        ));
    }
    if let Some(e) = &i.erro {
        let frase = match &e.detalhe {
            Some(d) => tr_with(e.chave, &[("detail", d)]),
            None => tr(e.chave).to_owned(),
        };
        out.push((frase, ColorToken::Danger));
    }
    out
}

/// Pinta a secção. Devolve o `y` seguinte.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_macro_section(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    i: &InspectorMacroInfo,
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
        ph2d_editor_core::ids::INSP_LIVE_MACRO_SECTION,
        tr("component.macro.name"),
    ) {
        Ok(v) => v,
        Err(y) => return y,
    };
    let rotulo = tr("component.field.macro_fields.2");
    let seccao = Seccao::medida(text_system, 1, &[rotulo]);
    let modos: Vec<(&str, bool, NodeId)> = crate::ids::INSP_MACRO_MODO
        .iter()
        .enumerate()
        .map(|(k, &id)| (tr(MODOS[k]), k == usize::from(i.modo), id))
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
        rotulo,
        &modos,
        seccao,
    );
    if let Some(dica) = DICAS.get(usize::from(i.modo)) {
        cur_y = super::rows::aviso(
            scene,
            text_system,
            theme,
            x,
            w,
            cur_y,
            tr(dica),
            ColorToken::Text3,
        );
    }
    for (frase, cor) in linhas(i) {
        cur_y = super::rows::aviso(scene, text_system, theme, x, w, cur_y, &frase, cor);
    }
    let id = crate::ids::INSP_MACRO_EDITAR_ETAPAS;
    let rotulo = tr("panel.inspector.macro.edit_steps");
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
    fold.finish(store, scene, hit_index, cur_y)
}
