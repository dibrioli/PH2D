//! **O grupo «Layers» do Ray Sensor** (dono 15/11: *«Layers e Sees deveriam ficar numa seção própria mais
//! próximos»*): a «Layer» em cima, logo por baixo o «Sees» com a frase dele, depois «Rename layers… | Open in
//! tab…». Nasce ABERTO; fechado, o resumo diz a camada e o que ela vê. Auditoria:
//! `docs/GameEngineFuncional/componentes/07_auditoria_arrumacao_2026-11-15.md`.

use super::*;
use crate::camadas_fisica as camadas;
use ph2d_editor_core::ray_edits::InspectorRayInfo;
use ph2d_i18n::{tr, tr_with};

/// A camada do raio, presa às que existem.
fn camada_do_raio(i: &InspectorRayInfo) -> u8 {
    u8::try_from(i.layer)
        .ok()
        .filter(|&c| usize::from(c) < crate::ids::INSP_PHYS_LAYERS)
        .unwrap_or_default()
}

/// O resumo do grupo fechado: «Default · sees Default, Ground».
#[must_use]
pub(crate) fn resumo(camada: usize) -> String {
    let linha = camadas::linha(camada);
    let vistas: Vec<String> = (0..crate::ids::INSP_PHYS_LAYERS)
        .filter(|&j| linha & (1 << j) != 0)
        .map(camadas::rotulo)
        .collect();
    let layer = camadas::rotulo(camada);
    if vistas.is_empty() {
        return tr_with(
            "panel.inspector.ray.layers_sees_nothing",
            &[("layer", &layer.as_str())],
        );
    }
    let seen = vistas.join(tr("panel.inspector.ray.layers_list_sep"));
    tr_with(
        "panel.inspector.ray.layers_summary",
        &[("layer", &layer.as_str()), ("seen", &seen.as_str())],
    )
}

/// O grupo «Layers»: Layer, Sees, os botões. Devolve o `y` seguinte.
#[allow(clippy::too_many_arguments)]
pub(super) fn grupo_das_camadas(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    i: &InspectorRayInfo,
    seccao: ph2d_editor_core::property_row::Seccao,
) -> f32 {
    let titulo = tr("panel.inspector.ray.layers");
    let g = super::grupo_dobravel::grupo_dobravel(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        y,
        crate::ids::INSP_RAY_FOLD_LAYERS,
        titulo,
    );
    let camada = camada_do_raio(i);
    if !g.aberto() {
        super::grupo_dobravel::resumo_do_grupo(
            scene,
            text_system,
            theme,
            x,
            w,
            g.y,
            titulo,
            &resumo(usize::from(camada)),
        );
        return g.fecha(store, scene, hit_index, y);
    }
    let mut t = super::physics_cartao::Tinta {
        scene,
        text: text_system,
        theme,
        hit: hit_index,
        store,
        x,
        w,
    };
    let rotulo = TextKey::new("component.field.ray_sensor.2");
    let cur_y = super::camada_por_nome::linha_da_camada(
        &mut t,
        g.y,
        &super::physics_colide::DO_RAIO,
        camada,
        rotulo.into(),
        seccao,
    );
    let cur_y = sees_e_botoes(&mut t, cur_y, usize::from(camada));
    g.fecha(store, scene, hit_index, cur_y)
}

/// Logo por baixo da Layer, o «Sees» (+ Add layer e a frase); em cartões, depois, os botões no lugar e idioma
/// dos do collider (emenda 4, dono 08/11). Renomeando, os campos de todas as camadas e «Done» tomam o lugar de
/// «Sees» e dos botões.
fn sees_e_botoes(t: &mut super::physics_cartao::Tinta<'_>, y: f32, camada: usize) -> f32 {
    let sees = tr("panel.inspector.ray.sees");
    let quem = &super::physics_colide::DO_RAIO;
    if !crate::cards::active() {
        return super::physics_colide_fila::fila_da_matriz(t, y, quem, camada, sees);
    }
    if super::ray::renomeia_o_raio(t.store) {
        let y = super::physics_colide::rename(t, y, camada);
        let feito = tr("card.insp.phys.layer_rename_done");
        let (r, y) = t.botao(crate::ids::INSP_RAY_LAYER_RENAME_DONE, feito, y);
        t.hit.register(crate::ids::INSP_RAY_LAYER_RENAME_DONE, r);
        return y;
    }
    let y = super::physics_colide_fila::fila_da_matriz(t, y, quem, camada, sees);
    t.dois_botoes(
        y,
        (
            crate::ids::INSP_RAY_LAYER_RENAME,
            tr("card.insp.phys.layer_rename"),
        ),
        (
            crate::ids::INSP_RAY_EDIT_MATRIX,
            tr("card.insp.phys.edit_matrix"),
        ),
    )
}
