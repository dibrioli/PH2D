//! **O que o Inspector mostra do MOTION INPUT** (ciclo 19 do Motion) — o objecto que publica uma
//! entrada para o grafo.
//!
//! A maquete B (dono 15/11, `docs/GameEngineFuncional/componentes/08_motion_input.md`): os nomes que
//! o grafo lê são a ESCOLHA principal (chips que escrevem o nome, mais «New…» para um nome novo; a
//! caixa de texto só aparece para um nome que o grafo ainda não lê); depois a fonte, o campo que a
//! fonte lê (uma linha que não mexe nada é um controlo morto), a QUEIXA com a saída dela, e a leitura
//! do que chega ao grafo AGORA (MI-11).

use super::*;
use ph2d_editor_core::motion_input_edits::{
    InspectorMotionInputInfo, MOTION_INPUT_SOURCES, MotionInputAoVivo, MotionInputOutro,
    MotionInputQueixa, SOURCE_ACTION, SOURCE_PROPERTY, SOURCE_SPEED,
};
use ph2d_editor_core::property_row::{Seccao, paint_choice_row};
use ph2d_i18n::{tr, tr_with};

/// Os rótulos das fontes, pela ordem de `MotionSource::ALL`.
const FONTES: [&str; MOTION_INPUT_SOURCES] = [
    "panel.inspector.motion_input.source.0",
    "panel.inspector.motion_input.source.1",
    "panel.inspector.motion_input.source.2",
    "panel.inspector.motion_input.source.3",
    "panel.inspector.motion_input.source.4",
    "panel.inspector.motion_input.source.5",
    "panel.inspector.motion_input.source.6",
    "panel.inspector.motion_input.source.7",
];

const _: () = assert!(crate::ids::INSP_MI_SOURCE.len() == MOTION_INPUT_SOURCES);

#[must_use]
const fn chave_da_queixa(q: MotionInputQueixa) -> &'static str {
    match q {
        MotionInputQueixa::NomeVazio => "panel.inspector.motion_input.empty_name",
        MotionInputQueixa::GrafoSemEntradas => "panel.inspector.motion_input.graph_has_no_input",
        MotionInputQueixa::NomeNaoLido => "panel.inspector.motion_input.name_not_read",
        MotionInputQueixa::SemVida => "panel.inspector.motion_input.no_health",
        MotionInputQueixa::SemAlvo => "panel.inspector.motion_input.no_from",
        MotionInputQueixa::SemCorpo => "panel.inspector.motion_input.no_body",
        MotionInputQueixa::AcaoDesconhecida => "panel.inspector.motion_input.unknown_action",
        MotionInputQueixa::CampoVazio => "panel.inspector.motion_input.no_property",
        MotionInputQueixa::CampoAusente => "panel.inspector.motion_input.property_gone",
    }
}

/// A fileira «Input»: um chip por nome do grafo (o aceso é o nome actual) e «New…» no fim.
#[allow(clippy::too_many_arguments)]
fn nomes_do_grafo(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    rotulo: TextKey,
    i: &InspectorMotionInputInfo,
    seccao: Seccao,
) -> f32 {
    let atual = i.name.trim();
    let mut chips: Vec<(&str, bool, NodeId)> = i
        .nomes_do_grafo
        .iter()
        .zip(crate::ids::INSP_MI_GRAFO)
        .map(|(n, id)| (n.as_str(), n == atual, id))
        .collect();
    chips.push((
        tr("panel.inspector.motion_input.new"),
        false,
        crate::ids::INSP_MI_NEW,
    ));
    paint_choice_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        y,
        rotulo,
        &chips,
        seccao,
    )
}

/// O número de uma leitura, com duas casas (a régua das frações `0..1`).
fn numero(v: f32) -> String {
    format!("{v:.2}")
}

fn valor_na_frase(v: MotionInputAoVivo) -> String {
    match v {
        MotionInputAoVivo::Numero(n) => numero(n),
        MotionInputAoVivo::Cor(_) => tr("panel.inspector.motion_input.a_color").to_string(),
        MotionInputAoVivo::Disparos(n) => n.to_string(),
    }
}

/// MI-11 — **o que chega ao grafo AGORA**: a vida lida, o que este objecto manda, os disparos, e
/// quem TAPA o nome (o grafo lê a primeira linha). `Text1` porque muda a cada quadro.
fn frases_ao_vivo(i: &InspectorMotionInputInfo) -> Vec<String> {
    use ph2d_editor_core::motion_input_edits::{SOURCE_DAMAGE, SOURCE_HEALTH};
    let mut f = Vec::new();
    match (i.source, i.ao_vivo) {
        (SOURCE_HEALTH, Some(MotionInputAoVivo::Numero(v))) => f.push(tr_with(
            "panel.inspector.motion_input.reads_health",
            &[("v", &numero(v))],
        )),
        // Parada, a Property lê o campo no alvo (a mesma porta da ponte); o rótulo é o da lista.
        (SOURCE_PROPERTY, Some(MotionInputAoVivo::Numero(v))) if !i.a_correr => {
            if let Some((rotulo, _)) = i.campos_do_alvo.iter().find(|(_, k)| *k == i.property) {
                f.push(tr_with(
                    "panel.inspector.motion_input.reads_property",
                    &[("c", rotulo), ("v", &numero(v))],
                ));
            }
        }
        (SOURCE_DAMAGE, Some(MotionInputAoVivo::Disparos(0))) => {
            f.push(tr("panel.inspector.motion_input.not_fired").to_string());
        }
        (SOURCE_DAMAGE, Some(MotionInputAoVivo::Disparos(n))) => f.push(tr_with(
            "panel.inspector.motion_input.fired",
            &[("n", &n.to_string())],
        )),
        (_, Some(v)) if i.a_correr => f.push(tr_with(
            "panel.inspector.motion_input.sends",
            &[("v", &valor_na_frase(v))],
        )),
        _ => {}
    }
    if let Some((quem, v)) = &i.tapado_por {
        let v = valor_na_frase(*v);
        f.push(match quem {
            MotionInputOutro::Script => tr_with(
                "panel.inspector.motion_input.covered_by_script",
                &[("v", &v)],
            ),
            MotionInputOutro::Linha => {
                tr_with("panel.inspector.motion_input.covered_by_line", &[("v", &v)])
            }
            MotionInputOutro::Objeto(o) => tr_with(
                "panel.inspector.motion_input.covered_by_object",
                &[("o", o), ("v", &v)],
            ),
        });
    }
    f
}

/// A queixa e, na que tem saída, o botão dela («Add Health»).
#[allow(clippy::too_many_arguments)]
fn queixa(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    i: &InspectorMotionInputInfo,
) -> f32 {
    let Some(q) = i.queixa() else {
        return y;
    };
    let cur_y = super::rows::aviso(
        scene,
        text_system,
        theme,
        x,
        w,
        y,
        tr(chave_da_queixa(q)),
        ColorToken::Warn,
    );
    // A saída de cada queixa que tem uma: Health, o corpo da Speed (o Grounded pede um Player, que não é
    // um anexo de um clique), e o conta-gotas para um From que não acha ninguém.
    let (id, chave) = match q {
        MotionInputQueixa::SemVida => (
            crate::ids::INSP_MI_ADD_HEALTH,
            "panel.inspector.motion_input.add_health",
        ),
        MotionInputQueixa::SemCorpo if i.source == SOURCE_SPEED => (
            crate::ids::INSP_MI_ADD_BODY,
            "panel.inspector.motion_input.add_body",
        ),
        MotionInputQueixa::SemAlvo => (
            crate::ids::INSP_MI_PICK_OBJECT,
            "panel.inspector.motion_input.pick_object",
        ),
        _ => return cur_y,
    };
    let rotulo = tr(chave);
    let r = ph2d_editor_core::property_row::caixa_do_botao(text_system, x, w, cur_y, rotulo);
    let btn = Button::new(id, rotulo)
        .kind(ButtonKind::Default)
        .visual(store.button_visual(id));
    paint_button(&btn, r, scene, text_system, theme);
    hit_index.register(id, r);
    ph2d_editor_core::property_row::abaixo_do_botao(r)
}

/// Pinta a secção. Devolve o `y` seguinte.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_motion_input_section(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    i: &InspectorMotionInputInfo,
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
        ph2d_editor_core::ids::INSP_LIVE_MOTION_INPUT_SECTION,
        tr("component.motion_input.name"),
    ) {
        Ok(v) => v,
        Err(y) => return y,
    };
    let chaves = [
        TextKey::new("component.field.motion_input_fields.1"),
        TextKey::new("component.field.motion_input_fields.2"),
        TextKey::new("component.field.motion_input_fields.3"),
        TextKey::new("component.field.motion_input_fields.4"),
        TextKey::new("panel.inspector.motion_input.name_label"),
        TextKey::new("component.field.motion_input_fields.5"),
        TextKey::new("component.field.motion_input_fields.6"),
        TextKey::new("component.field.motion_input_fields.7"),
        TextKey::new("component.field.motion_input_fields.8"),
        TextKey::new("component.field.motion_input_fields.9"),
        TextKey::new("component.field.motion_input_fields.10"),
        TextKey::new("component.field.motion_input_fields.11"),
        TextKey::new("component.field.motion_input_fields.12"),
    ];
    let rotulos = chaves.map(TextKey::tr);
    let seccao = Seccao::medida(text_system, 1, &rotulos);
    let tem_chips = !i.nomes_do_grafo.is_empty();
    if tem_chips {
        cur_y = nomes_do_grafo(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            cur_y,
            chaves[0],
            i,
            seccao,
        );
    }
    // A caixa só para um nome que o grafo ainda não lê (ou sem grafo, onde ela é a fileira «Input»).
    if !tem_chips || !i.nomes_do_grafo.iter().any(|n| n == i.name.trim()) {
        let nome = crate::ids::INSP_MI_NAME;
        cur_y = super::anim_rows::text_row(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            cur_y,
            if tem_chips { chaves[4] } else { chaves[0] },
            nome,
            TextInput::new(nome, "").placeholder(tr("panel.inspector.motion_input.name_hint")),
            seccao,
        );
    }
    let fontes: Vec<(&str, bool, NodeId)> = crate::ids::INSP_MI_SOURCE
        .iter()
        .enumerate()
        .map(|(k, &id)| (tr(FONTES[k]), k == usize::from(i.source), id))
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
        chaves[1],
        &fontes,
        seccao,
    );
    let linhas = super::motion_input_linhas::lista;
    if i.le_from() {
        let xy = (x, w, cur_y);
        cur_y = super::motion_input_linhas::from(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            xy,
            i,
            seccao,
        );
    }
    for (fonte, l) in [
        (SOURCE_ACTION, crate::popovers::motion_input::Lista::Action),
        (
            SOURCE_PROPERTY,
            crate::popovers::motion_input::Lista::Property,
        ),
    ] {
        if i.source == fonte {
            let xy = (x, w, cur_y);
            cur_y = linhas(
                scene,
                text_system,
                theme,
                hit_index,
                store,
                xy,
                i,
                l,
                seccao,
            );
        }
    }
    if i.le_valor() {
        cur_y = super::rows::fields_row(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            cur_y,
            chaves[2],
            &[crate::ids::INSP_MI_VALUE],
            0.01, // LITERAL-PX-OK: passo de arrasto de uma fracção 0..1
            None,
            seccao,
        );
    }
    if i.le_cor() {
        cur_y = super::color_tint::bloco_de_cores(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            cur_y,
            &[(crate::ids::INSP_MI_COLOR, chaves[3], i.color)],
        );
    }
    if i.le_numero() {
        let xy = (x, w, cur_y);
        cur_y = super::motion_input_linhas::grupo_shape(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            xy,
            i,
            seccao,
        );
    }
    cur_y = queixa(scene, text_system, theme, hit_index, store, x, w, cur_y, i);
    for frase in frases_ao_vivo(i) {
        cur_y = super::rows::aviso(
            scene,
            text_system,
            theme,
            x,
            w,
            cur_y,
            &frase,
            ColorToken::Text1,
        );
    }
    let resto = i
        .nomes_do_grafo
        .len()
        .saturating_sub(crate::ids::INSP_MI_GRAFO.len());
    if resto > 0 {
        cur_y = super::rows::aviso(
            scene,
            text_system,
            theme,
            x,
            w,
            cur_y,
            &tr_with(
                "panel.inspector.motion_input.more_names",
                &[("n", &resto.to_string())],
            ),
            ColorToken::Text3,
        );
    }
    fold.finish(store, scene, hit_index, cur_y)
}
