//! **As linhas da onda 2 do MOTION INPUT** (GEF 08): o From (caixa pelo nome + conta-gotas), as listas Action
//! e Property, e o grupo dobrável «Shape» (Remap · From | To · Curve · Smooth) — irmão de
//! [`super::motion_input`] pelo tecto de linhas dos painéis.

use super::*;
use crate::popovers::motion_input::{Lista, set_pending};
use ph2d_editor_core::motion_input_edits::{CURVE_PRESETS, InspectorMotionInputInfo};
use ph2d_editor_core::property_row::{Seccao, paint_label_row};
use ph2d_editor_core::widget::key_mark::Rotulo;
use ph2d_editor_core::widget::{Dropdown, paint_dropdown_chip};
use ph2d_i18n::{tr, tr_with};

/// O From — a linha do «Aim at» do Ray Sensor (caixa pelo nome + conta-gotas); vazio = o PAI («Hero (parent)»).
#[allow(clippy::too_many_arguments)]
pub(super) fn from(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    (x, w, y): (f32, f32, f32),
    i: &InspectorMotionInputInfo,
    seccao: Seccao,
) -> f32 {
    super::conta_gotas_row::linha(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        y,
        (
            TextKey::new("component.field.motion_input_fields.5"),
            crate::ids::INSP_MI_FROM,
            TextInput::new(crate::ids::INSP_MI_FROM, "").placeholder(o_pai(i)),
        ),
        (crate::ids::INSP_MI_FROM_PICK, i.from_pick_armed),
        seccao,
        None,
    )
}

/// O From vazio diz o pai pelo nome («Hero (parent)»); sem nome, «Parent».
#[must_use]
pub(crate) fn o_pai(i: &InspectorMotionInputInfo) -> String {
    if i.pai.trim().is_empty() {
        return tr("panel.inspector.motion_input.parent").to_string();
    }
    tr_with(
        "panel.inspector.motion_input.from_this",
        &[("o", &i.pai.trim())],
    )
}

/// O nome do objecto que a fonte lê, para as frases: o From, ou o pai.
#[must_use]
pub(crate) fn o_alvo(i: &InspectorMotionInputInfo) -> String {
    match (i.from.trim(), i.pai.trim()) {
        ("", "") => tr("panel.inspector.motion_input.the_parent").to_string(),
        ("", p) => p.to_string(),
        (f, _) => f.to_string(),
    }
}

/// A lista suspensa de `lista` (Action ou Property), com o popover diferido.
#[allow(clippy::too_many_arguments)]
pub(super) fn lista(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    (x, w, y): (f32, f32, f32),
    i: &InspectorMotionInputInfo,
    lista: Lista,
    seccao: Seccao,
) -> f32 {
    let (rotulo, vazio) = match lista {
        Lista::Action => (
            "component.field.motion_input_fields.6",
            "panel.inspector.motion_input.action_pick",
        ),
        Lista::Property => (
            "component.field.motion_input_fields.7",
            "panel.inspector.motion_input.property_pick",
        ),
    };
    let id = lista.id();
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
    hit_index.register(id, linha.control);
    let aberto = matches!(
        store.get(id),
        Some(InteractiveState::Dropdown { open: true, .. })
    );
    let (opcoes, sel) = lista.opcoes(i);
    let mut dd = Dropdown::new(id, "", opcoes)
        .placeholder(tr(vazio))
        .open(aberto)
        .visual(store.dropdown_visual(id));
    if let Some(n) = sel {
        dd.select(n);
    }
    paint_dropdown_chip(&dd, linha.control, scene, text_system, theme);
    // A escolha da ação/propriedade não se anima: a coluna leva o ponto, e no cartão nada (D17).
    if ph2d_editor_core::widget::card_ink::card_body().is_none() {
        ph2d_editor_core::widget::paint_decorator_dot(scene, theme, linha.dot);
    }
    if aberto {
        set_pending(Some((lista, linha.control)));
    }
    linha.fim(ph2d_tokens::row_pitch_px())
}

/// O nome da predefinição da curva gravada (`None` = uma curva que nenhuma predefinição escreve).
fn nome_da_curva(curve: &str) -> Option<&'static str> {
    CURVE_PRESETS
        .iter()
        .find(|(_, t)| *t == curve.trim())
        .map(|(k, _)| tr(k))
}

fn numero_curto(v: f32) -> String {
    let s = format!("{v:.2}");
    let s = s.trim_end_matches('0').trim_end_matches('.');
    ph2d_editor_core::widget::zero_sem_sinal(s).replace('-', "\u{2212}")
}

/// ⭐ **O resumo do grupo fechado**: «Off» de fábrica; senão a faixa, a curva (se não é a Linear) e o Smooth.
#[must_use]
pub(crate) fn resumo(i: &InspectorMotionInputInfo) -> String {
    let mut partes: Vec<String> = Vec::new();
    if i.remap {
        partes.push(tr_with(
            "panel.inspector.motion_input.remap_summary",
            &[
                ("a", &numero_curto(i.remap_from)),
                ("b", &numero_curto(i.remap_to)),
            ],
        ));
        match nome_da_curva(&i.curve) {
            Some(_) if i.curve.trim().is_empty() => {}
            Some(n) => partes.push(n.to_string()),
            None => partes.push(tr("panel.inspector.motion_input.curve_custom").to_string()),
        }
    }
    if i.smooth > 0.0 {
        partes.push(tr_with(
            "panel.inspector.motion_input.smooth_s",
            &[("s", &numero_curto(i.smooth))],
        ));
    }
    if partes.is_empty() {
        tr("panel.inspector.motion_input.shape_off").to_string()
    } else {
        partes.join(" \u{00b7} ")
    }
}

/// ⭐ O grupo «Shape», fechado de fábrica com o resumo na linha do título (o molde do «Filter» do Ray).
#[allow(clippy::too_many_arguments)]
pub(super) fn grupo_shape(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    (x, w, y): (f32, f32, f32),
    i: &InspectorMotionInputInfo,
    seccao: Seccao,
) -> f32 {
    let titulo = tr("panel.inspector.motion_input.shape");
    let g = super::grupo_dobravel::grupo_dobravel(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        y,
        crate::ids::INSP_MI_FOLD_SHAPE,
        titulo,
    );
    if !g.aberto() {
        super::grupo_dobravel::resumo_do_grupo(
            scene,
            text_system,
            theme,
            x,
            w,
            g.y,
            titulo,
            &resumo(i),
        );
        return g.fecha(store, scene, hit_index, y);
    }
    let mut cur_y = ph2d_editor_core::property_row::paint_check_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        g.y,
        (
            crate::ids::INSP_MI_REMAP,
            Rotulo::from(TextKey::new("component.field.motion_input_fields.8")),
            i.remap,
        ),
        seccao,
    );
    // A faixa e a curva só decidem com a caixa (sem ela, o número passa): sem ela, somem.
    if i.remap {
        // From | To na mesma linha (fora do cartão, as duas linhas de hoje).
        cur_y = super::rows::par_row(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            cur_y,
            None,
            [
                super::rows::Campo::new(
                    TextKey::new("component.field.motion_input_fields.9"),
                    crate::ids::INSP_MI_REMAP_FROM,
                    None,
                ),
                super::rows::Campo::new(
                    TextKey::new("component.field.motion_input_fields.10"),
                    crate::ids::INSP_MI_REMAP_TO,
                    None,
                ),
            ],
        );
        let gravada = CURVE_PRESETS.iter().position(|(_, t)| *t == i.curve.trim());
        let chips: Vec<(&str, bool, NodeId)> = CURVE_PRESETS
            .iter()
            .zip(crate::ids::INSP_MI_CURVE)
            .enumerate()
            .map(|(n, ((k, _), id))| (tr(k), gravada == Some(n), id))
            .collect();
        cur_y = ph2d_editor_core::property_row::paint_choice_row(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            cur_y,
            TextKey::new("component.field.motion_input_fields.11"),
            &chips,
            seccao,
        );
    }
    cur_y = super::rows::fields_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        TextKey::new("component.field.motion_input_fields.12"),
        &[crate::ids::INSP_MI_SMOOTH],
        0.05, // LITERAL-PX-OK: passo de arrasto em segundos
        Some(ph2d_editor_core::widget::Unit::Seconds),
        seccao,
    );
    g.fecha(store, scene, hit_index, cur_y)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ⭐ O resumo do Shape fechado muda com as opções: «Off» de fábrica, a faixa, a curva e o Smooth.
    #[test]
    fn o_resumo_do_shape_segue_as_opcoes() {
        let mut i = InspectorMotionInputInfo {
            entity_bits: 1,
            name: "hp".into(),
            source: 2,
            value: 0.0,
            color: [1.0; 4],
            nomes_do_grafo: Vec::new(),
            tem_vida: true,
            from: String::new(),
            alvo_achado: true,
            tem_corpo: false,
            action: String::new(),
            acoes_do_mapa: Vec::new(),
            property: String::new(),
            campos_do_alvo: Vec::new(),
            campos_par: Vec::new(),
            pai: String::new(),
            alvo_bits: None,
            remap: false,
            remap_from: 0.0,
            remap_to: 900.0,
            curve: String::new(),
            smooth: 0.0,
            from_pick_armed: false,
            selected_count: 1,
            a_correr: false,
            ao_vivo: None,
            tapado_por: None,
        };
        assert_eq!(resumo(&i), "Off");
        i.remap = true;
        assert_eq!(resumo(&i), "0\u{2013}900 \u{2192} 0\u{2013}1");
        i.curve = CURVE_PRESETS[1].1.into();
        i.smooth = 0.3;
        assert_eq!(
            resumo(&i),
            "0\u{2013}900 \u{2192} 0\u{2013}1 \u{b7} Ease In \u{b7} Smooth 0.3 s"
        );
        i.curve = "c1 0:0:H 1:1:L".into();
        assert!(resumo(&i).contains("Custom curve"));
        i.remap = false;
        assert_eq!(resumo(&i), "Smooth 0.3 s", "sem Remap a curva não decide");
        // Um zero não tem sinal; o menos é o tipográfico.
        i.remap = true;
        i.smooth = 0.0;
        i.curve = String::new();
        i.remap_from = -0.001;
        i.remap_to = -5.0;
        assert_eq!(resumo(&i), "0\u{2013}\u{2212}5 \u{2192} 0\u{2013}1");
    }
}
