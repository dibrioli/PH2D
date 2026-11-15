//! ⭐ **O grupo «Filter» do Ray Sensor** (GEF 07, versão C, dono 15/11: «abranger todas as
//! possibilidades») — irmão de [`super::ray`] por tecto de LOC, e por ASSUNTO: aqui mora o QUEM o raio
//! vê; lá, o ONDE ele olha.
//!
//! Nasce FECHADO, com o resumo à direita do título (o molde do «Speed change» do Projectile). A tag e
//! «Walls block» só existem com «Only see» ≠ Everything: sem filtro de tag elas não decidem nada, e um
//! controlo que não decide é um controlo morto.

use super::*;
use ph2d_editor_core::property_row::{Seccao, paint_label_row};
use ph2d_editor_core::ray_edits::{InspectorRayInfo, RaySee};
use ph2d_editor_core::widget::key_mark::Rotulo;
use ph2d_editor_core::widget::{Dropdown, DropdownOption, paint_dropdown_chip};
use ph2d_i18n::{tr, tr_with};

/// A ordem das peças de «Only see» — a mesma do [`crate::ids::INSP_RAY_ONLY_SEE`].
pub(crate) const MODOS: [(RaySee, &str); 3] = [
    (RaySee::Everything, "panel.inspector.ray.see_everything"),
    (RaySee::OnlyTag, "panel.inspector.ray.see_only_tag"),
    (RaySee::ExceptTag, "panel.inspector.ray.see_except_tag"),
];

/// As opções da tag: a árvore inteira do projecto (a mesma porta do Nav Agent e do filtro dos sinais).
pub(crate) fn opcoes_da_tag() -> Vec<DropdownOption<u64>> {
    super::nav_tag_row::opcoes_da_arvore(
        &crate::ids::INSP_RAY_SEE_TAG_OPT,
        crate::ids::INSP_RAY_SEE_TAG_MORE,
    )
}

/// O nome da tag do filtro; `None` quando está por escolher (o filtro não filtra: C2) e «a deleted tag»
/// quando foi apagada (com «Only» ninguém passa).
fn nome_da_tag(tag: u64) -> Option<String> {
    (tag != 0).then(|| {
        crate::state::current_tag_tree()
            .into_iter()
            .find(|r| r.id == tag)
            .map_or_else(
                || tr("panel.inspector.ray.filter_tag_gone").to_string(),
                |r| r.label,
            )
    })
}

/// ⭐ **O resumo do grupo fechado** — só o que foge do de fábrica; de fábrica diz «Ignores me».
#[must_use]
pub(crate) fn resumo(i: &InspectorRayInfo) -> String {
    let mut partes: Vec<String> = Vec::new();
    let chave_do_modo = match i.only_see {
        RaySee::Everything => None,
        RaySee::OnlyTag => Some("panel.inspector.ray.filter_only_x"),
        RaySee::ExceptTag => Some("panel.inspector.ray.filter_not_x"),
    };
    if let (Some(chave), Some(tag)) = (chave_do_modo, nome_da_tag(i.see_tag)) {
        partes.push(tr_with(chave, &[("tag", &tag.as_str())]));
    }
    if i.ignore_parts && !i.ignore_up_to.is_empty() {
        partes.push(tr_with(
            "panel.inspector.ray.filter_up_to_x",
            &[("name", &i.ignore_up_to.as_str())],
        ));
    }
    for (liga, chave) in [
        (!i.ignore_body, "panel.inspector.ray.filter_sees_me"),
        (!i.ignore_parts, "panel.inspector.ray.filter_sees_my_parts"),
        (i.hit_triggers, "panel.inspector.ray.filter_sees_triggers"),
        (i.starts_inside, "panel.inspector.ray.filter_from_inside"),
        (i.all_hits, "panel.inspector.ray.filter_all_hits"),
    ] {
        if liga {
            partes.push(tr(chave).to_string());
        }
    }
    if partes.is_empty() {
        return tr("panel.inspector.ray.filter_ignores_me").to_string();
    }
    partes.join(tr("panel.inspector.ray.filter_sep"))
}

/// A linha da tag: o chip e a lista da árvore do projecto, num popover diferido.
#[allow(clippy::too_many_arguments)]
fn linha_da_tag(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    tag: u64,
    seccao: Seccao,
) -> f32 {
    let rotulo = TextKey::new("component.field.ray_sensor.7");
    let linha = paint_label_row(
        scene,
        text_system,
        theme,
        x,
        w,
        y,
        ROW_H_PX,
        tr("component.field.ray_sensor.7"),
        seccao,
    );
    hit_index.register(crate::ids::INSP_RAY_SEE_TAG, linha.control);
    let aberto = matches!(
        store.get(crate::ids::INSP_RAY_SEE_TAG),
        Some(InteractiveState::Dropdown { open: true, .. })
    );
    let mut dd = Dropdown::new(crate::ids::INSP_RAY_SEE_TAG, "", opcoes_da_tag())
        .placeholder(tr("panel.inspector.nav.pick_a_tag_u"))
        .open(aberto)
        .visual(store.dropdown_visual(crate::ids::INSP_RAY_SEE_TAG));
    if tag != 0 {
        dd.select(tag);
    }
    paint_dropdown_chip(&dd, linha.control, scene, text_system, theme);
    if aberto {
        crate::state_popovers::set_pending_ray_tag_dd(Some(linha.control));
    }
    if let Some(alvo) = ph2d_editor_core::widget::key_mark::paint_decorator_campo(
        scene,
        theme,
        linha.dot,
        &[crate::ids::INSP_RAY_SEE_TAG],
        rotulo.into(),
    ) {
        hit_index.register(alvo, linha.dot);
    }
    linha.fim(ph2d_tokens::row_pitch_px())
}

/// Pinta o grupo «Filter». Devolve o `y` seguinte.
#[allow(clippy::too_many_arguments)]
pub(super) fn grupo_do_filtro(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    i: &InspectorRayInfo,
    seccao: Seccao,
) -> f32 {
    let titulo = tr("panel.inspector.ray.filter");
    let g = super::grupo_dobravel::grupo_dobravel(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        y,
        crate::ids::INSP_RAY_FOLD_FILTER,
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
    let caixa = |id, n: &'static str, on| (id, Rotulo::from(TextKey::new(n)), on);
    let mut cur_y = ph2d_editor_core::property_row::paint_check_rows(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        g.y,
        &[
            caixa(
                crate::ids::INSP_RAY_IGNORE_BODY,
                "component.field.ray_sensor.4",
                i.ignore_body,
            ),
            caixa(
                crate::ids::INSP_RAY_IGNORE_PARTS,
                "component.field.ray_sensor.5",
                i.ignore_parts,
            ),
        ],
        seccao,
    );
    // «Ignore up to» (C8, dono 15/11) só com «Ignore my other parts»: sem ela não decide nada.
    if i.ignore_parts {
        let fantasma = if i.ignore_up_to_auto.is_empty() {
            tr("panel.inspector.ray.up_to_auto").to_string()
        } else {
            tr_with(
                "panel.inspector.ray.up_to_auto_x",
                &[("name", &i.ignore_up_to_auto.as_str())],
            )
        };
        cur_y = super::conta_gotas_row::linha(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            cur_y,
            (
                TextKey::new("component.field.ray_sensor.12"),
                crate::ids::INSP_RAY_UP_TO,
                TextInput::new(crate::ids::INSP_RAY_UP_TO, "").placeholder(&fantasma),
            ),
            (crate::ids::INSP_RAY_UP_TO_PICK, i.up_to_pick_armed),
            seccao,
        );
    }
    let nomes: Vec<&str> = MODOS.iter().map(|&(_, k)| tr(k)).collect();
    let segmentos: Vec<(&str, bool, ph2d_a11y::NodeId)> = MODOS
        .iter()
        .zip(nomes)
        .zip(crate::ids::INSP_RAY_ONLY_SEE)
        .map(|((&(m, _), nome), id)| (nome, i.only_see == m, id))
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
        TextKey::new("component.field.ray_sensor.6"),
        &segmentos,
        seccao,
    );
    if i.only_see != RaySee::Everything {
        cur_y = linha_da_tag(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            cur_y,
            i.see_tag,
            seccao,
        );
        cur_y = ph2d_editor_core::property_row::paint_check_row(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            cur_y,
            caixa(
                crate::ids::INSP_RAY_WALLS_BLOCK,
                "component.field.ray_sensor.8",
                i.walls_block,
            ),
            seccao,
        );
    }
    cur_y = ph2d_editor_core::property_row::paint_check_rows(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        &[
            caixa(
                crate::ids::INSP_RAY_HIT_TRIGGERS,
                "component.field.ray_sensor.9",
                i.hit_triggers,
            ),
            caixa(
                crate::ids::INSP_RAY_STARTS_INSIDE,
                "component.field.ray_sensor.10",
                i.starts_inside,
            ),
            caixa(
                crate::ids::INSP_RAY_ALL_HITS,
                "component.field.ray_sensor.11",
                i.all_hits,
            ),
        ],
        seccao,
    );
    g.fecha(store, scene, hit_index, cur_y)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_editor_core::screens::hero::InspectorTagRow;

    fn de_fabrica() -> InspectorRayInfo {
        InspectorRayInfo {
            entity_bits: 1,
            angle: 0.0,
            reach: 1.0,
            layer: 0,
            turns_with_facing: false,
            ignore_body: true,
            ignore_parts: true,
            ignore_up_to: String::new(),
            ignore_up_to_off: false,
            ignore_up_to_auto: String::new(),
            up_to_pick_armed: false,
            only_see: RaySee::Everything,
            see_tag: 0,
            see_tag_exists: true,
            walls_block: true,
            hit_triggers: false,
            starts_inside: false,
            all_hits: false,
            layer_empty: false,
            on_enter: String::new(),
            on_exit: String::new(),
            sees: String::new(),
            sees_at: 0.0,
            sees_all: Vec::new(),
            clock_playing: true,
            selected_count: 1,
        }
    }

    /// ⭐ O resumo do grupo fechado muda com as opções (controlo: o de fábrica diz «Ignores me»).
    ///
    /// **Mutação que deve sangrar:** o resumo devolver sempre «Ignores me».
    #[test]
    fn o_resumo_do_filter_muda_com_as_opcoes() {
        crate::state::set_current_tag_tree(vec![InspectorTagRow {
            id: 7,
            path: "Enemy".into(),
            label: "Enemy".into(),
            depth: 0,
        }]);
        assert_eq!(resumo(&de_fabrica()), "Ignores me");
        let mut r = de_fabrica();
        r.only_see = RaySee::OnlyTag;
        r.see_tag = 7;
        r.hit_triggers = true;
        assert_eq!(resumo(&r), "Only Enemy, Sees triggers");
        r.only_see = RaySee::ExceptTag;
        r.see_tag = 0;
        r.hit_triggers = false;
        r.ignore_body = false;
        r.all_hits = true;
        // C2: sem tag o filtro não filtra ninguém — o resumo não promete um «Not …» (a queixa pede a tag).
        assert_eq!(resumo(&r), "Sees me, All hits");
        r.only_see = RaySee::OnlyTag;
        r.see_tag = 9;
        assert_eq!(resumo(&r), "Only a deleted tag, Sees me, All hits");
        // C8: «Sees my parts» depende só da sua caixa; com ambas desligadas dizem-se as duas.
        let mut p = de_fabrica();
        p.ignore_body = false;
        p.ignore_parts = false;
        assert_eq!(resumo(&p), "Sees me, Sees my parts");
        let mut topo = de_fabrica();
        topo.ignore_up_to = "Truck".into();
        assert_eq!(resumo(&topo), "Up to Truck");
        crate::state::set_current_tag_tree(Vec::new());
    }
}
