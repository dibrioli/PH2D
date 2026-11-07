//! **O cartão Shape de um corpo e as linhas que ele partilha com o «Collision Shape»** (ficha f1). Irmão de
//! [`super::physics_cartao`] pelo tecto de 600 LOC.

use super::physics_cartao::{CARD_PHYS_SHAPE, Tinta};
use super::physics_rows::{
    ONEWAY_LABELS, SENSOR_LABELS, paint_bounce_friction, paint_combine_rows, paint_signal_rows,
    paint_surface_rows,
};
use super::rows::{Campo, par_row, seg_row};
use super::*;
use ph2d_editor_core::screens::hero::InspectorPhysicsInfo;
use ph2d_editor_core::widget::key_mark::Eixos;
use ph2d_editor_core::widget::{Dropdown, DropdownOption, SectionFold, paint_dropdown_chip};
use ph2d_i18n::{TextKey, tr, tr_with};

const KIND_STATIC: u8 = 1;
const SHAPE_BOX: u8 = 1;
/// A folga com que dois lados de `Scale` contam como iguais: a de um `f32` que passou por um campo.
const ESCALA_IGUAL: f32 = 1e-4;

/// O cartão **Shape** de um corpo: `y` intacto para quem não tem corpo (a forma solta tem o cartão dela).
#[allow(clippy::too_many_arguments)]
pub(crate) fn secao_da_forma(
    scene: &mut VectorScene,
    text: &mut TextSystem,
    theme: Theme,
    hit: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    info: &InspectorPhysicsInfo,
) -> f32 {
    let header_h = TypeToken::Md.px() + Spacing::Md.px(); // LITERAL-PX-OK: section header band height
    let header = section_header(store, CARD_PHYS_SHAPE, tr("card.insp.phys.shape"));
    paint_section_header(&header, Rect::new(x, y, w, header_h), scene, text, theme);
    let top = y + header_h;
    let Some(fold) = SectionFold::begin(store, CARD_PHYS_SHAPE, x, w, top, scene, hit) else {
        return top;
    };
    let mut t = Tinta {
        scene: &mut *scene,
        text,
        theme,
        hit: &mut *hit,
        store,
        x,
        w,
    };
    let fim = forma(&mut t, top, info);
    fold.finish(store, scene, hit, fim)
}

fn forma(t: &mut Tinta<'_>, y: f32, info: &InspectorPhysicsInfo) -> f32 {
    let mut y = medidas_e_estado(t, y, info);
    if info.part_count > 0 {
        let n = info.part_count;
        let frase = if n == 1 {
            tr("panel.inspector.physics.plus_1_more_shape_from").to_string()
        } else {
            tr_with("panel.inspector.physics.more_shapes", &[("n", &n)])
        };
        y = t.frase(y, &frase, ColorToken::Text3);
    }
    y = toque(t, y, info);
    if info.is_sensor {
        y = sinais(t, y, info);
    } else {
        if info.kind_tag == KIND_STATIC {
            y = uma_via(t, y, info);
        }
        y = paint_bounce_friction(t.scene, t.text, t.theme, t.hit, t.store, t.x, t.w, y);
    }
    y = camada(t, y, info);
    if info.is_sensor {
        y = dobra_zona(t, y, info);
    }
    y = dobra_offset(t, y);
    y = if info.is_sensor {
        dobra_quique(t, y, info)
    } else {
        dobra_material(t, y, info)
    };
    let uma_via_aqui = !info.is_sensor && info.kind_tag != KIND_STATIC;
    y = dobra_superficie(t, y, info, uma_via_aqui);
    if !info.is_sensor {
        y = dobra_sinais(t, y, info);
    }
    y + SECTION_BOTTOM_PAD_PX
}

/// Shape, as medidas, o estado da forma e o aviso de escala: a abertura das duas faces da forma.
pub(super) fn medidas_e_estado(t: &mut Tinta<'_>, y: f32, info: &InspectorPhysicsInfo) -> f32 {
    let y = seg_row(
        t.scene,
        t.text,
        t.theme,
        t.hit,
        t.store,
        t.x,
        t.w,
        y,
        TextKey::new("component.field.collider.0"),
        core_ids::INSP_LIVE_PHYSICS_GRIP,
        &ids::INSP_PHYS_SHAPE,
        &super::physics::SHAPE_LABELS.map(TextKey::tr),
        info.shape_tag,
    );
    let y = super::physics::paint_shape_dims(
        t.scene,
        t.text,
        t.theme,
        t.hit,
        t.store,
        t.x,
        t.w,
        y,
        info.shape_tag,
    );
    let y = estado_da_forma(t, y, info);
    let y = aviso_de_escala(t, y, info);
    botao_das_alcas(t, y, info)
}

/// Aceso enquanto as alças da forma estão no canvas, e diz como se sai.
fn botao_das_alcas(t: &mut Tinta<'_>, y: f32, info: &InspectorPhysicsInfo) -> f32 {
    let (texto, tipo) = if info.shape_edit_on {
        (tr("card.insp.phys.editing_shape"), ButtonKind::Accent)
    } else {
        (tr("card.insp.phys.edit_shape"), ButtonKind::Default)
    };
    let (r, y) = t.botao_de(ids::INSP_PHYS_EDIT_SHAPE, texto, tipo, y);
    t.hit.register(ids::INSP_PHYS_EDIT_SHAPE, r);
    y
}

fn estado_da_forma(t: &mut Tinta<'_>, y: f32, info: &InspectorPhysicsInfo) -> f32 {
    match info.shape_follows {
        None => y,
        Some(true) => t.frase(y, tr("card.insp.phys.follows"), ColorToken::Text3),
        Some(false) => {
            let y = t.frase(y, tr("card.insp.phys.custom"), ColorToken::Text3);
            let (r, y) = t.botao(ids::INSP_PHYS_FIT_PICTURE, tr("card.insp.phys.fit"), y);
            t.hit.register(ids::INSP_PHYS_FIT_PICTURE, r);
            y
        }
    }
}

/// `Scale` desigual numa forma redonda?
pub(crate) fn esticada(info: &InspectorPhysicsInfo) -> bool {
    info.shape_tag != SHAPE_BOX
        && crate::state::current_inspector_transform()
            .filter(|p| p.entity_bits == info.entity_bits)
            .is_some_and(|p| (p.scale[0].abs() - p.scale[1].abs()).abs() > ESCALA_IGUAL)
}

fn aviso_de_escala(t: &mut Tinta<'_>, y: f32, info: &InspectorPhysicsInfo) -> f32 {
    if !esticada(info) {
        return y;
    }
    let y = t.frase(y, tr("card.insp.phys.stretched"), ColorToken::Warn);
    let (r, y) = t.botao(ids::INSP_PHYS_SCALE_ONE, tr("card.insp.phys.scale_one"), y);
    t.hit.register(ids::INSP_PHYS_SCALE_ONE, r);
    let (r, y) = t.botao(ids::INSP_PHYS_USE_BOX, tr("card.insp.phys.use_box"), y);
    t.hit.register(ids::INSP_PHYS_USE_BOX, r);
    y
}

pub(super) fn toque(t: &mut Tinta<'_>, y: f32, info: &InspectorPhysicsInfo) -> f32 {
    seg_row(
        t.scene,
        t.text,
        t.theme,
        t.hit,
        t.store,
        t.x,
        t.w,
        y,
        tr("card.insp.phys.touch"),
        ids::INSP_LIVE_PHYSICS_SENSOR,
        &ids::INSP_PHYS_SENSOR,
        &SENSOR_LABELS.map(TextKey::tr),
        u8::from(info.is_sensor),
    )
}

pub(super) fn uma_via(t: &mut Tinta<'_>, y: f32, info: &InspectorPhysicsInfo) -> f32 {
    seg_row(
        t.scene,
        t.text,
        t.theme,
        t.hit,
        t.store,
        t.x,
        t.w,
        y,
        tr("panel.inspector.physics.one_way"),
        ids::INSP_LIVE_PHYSICS_ONEWAY,
        &ids::INSP_PHYS_ONEWAY,
        &ONEWAY_LABELS.map(TextKey::tr),
        u8::from(info.one_way),
    )
}

/// Layer em lista com os nomes (os ids e a edição são os de sempre), «Hits» lido da matriz do mundo e o «Rename…».
pub(super) fn camada(t: &mut Tinta<'_>, y: f32, info: &InspectorPhysicsInfo) -> f32 {
    let rotulo = tr("component.field.collider.4");
    let seccao = ph2d_editor_core::property_row::Seccao::medida(t.text, 1, &[rotulo]);
    let linha = ph2d_editor_core::property_row::paint_label_row(
        t.scene,
        t.text,
        t.theme,
        t.x,
        t.w,
        y,
        ph2d_tokens::ROW_H_PX,
        rotulo,
        seccao,
    );
    t.hit.register(ids::INSP_PHYS_LAYER_PICK, linha.control);
    let aberta = matches!(
        t.store.get(ids::INSP_PHYS_LAYER_PICK),
        Some(InteractiveState::Dropdown { open: true, .. })
    );
    let dd = lista_da_camada(info.layer)
        .open(aberta)
        .visual(t.store.dropdown_visual(ids::INSP_PHYS_LAYER_PICK));
    paint_dropdown_chip(&dd, linha.control, t.scene, t.text, t.theme);
    if aberta {
        crate::state_popovers::set_pending_phys_layer_dd(Some((info.layer, linha.control)));
    }
    if let Some(alvo) = ph2d_editor_core::widget::key_mark::paint_decorator_for(
        t.scene,
        t.theme,
        linha.dot,
        &ids::INSP_PHYS_LAYER,
    ) {
        t.hit.register(alvo, linha.dot);
    }
    let y = linha.fim(ph2d_tokens::row_pitch_px());
    let camada = usize::from(info.layer);
    let y = t.frase(y, &crate::camadas_fisica::hits(camada), ColorToken::Text3);
    let (r, y) = t.botao(
        ids::INSP_PHYS_LAYER_RENAME,
        tr("card.insp.phys.layer_rename"),
        y,
    );
    t.hit.register(ids::INSP_PHYS_LAYER_RENAME, r);
    if t.store.is_collapsed(ids::INSP_PHYS_LAYER_RENAMING) {
        return y;
    }
    let dica = camada.to_string();
    let rotulo = tr("card.insp.phys.layer_name");
    let seccao = ph2d_editor_core::property_row::Seccao::medida(t.text, 1, &[rotulo]);
    ph2d_editor_core::property_row::paint_text_row(
        t.scene,
        t.text,
        t.theme,
        t.hit,
        t.store,
        t.x,
        t.w,
        y,
        rotulo,
        ids::INSP_PHYS_LAYER_NAME,
        TextInput::new(ids::INSP_PHYS_LAYER_NAME, "").placeholder(&dica),
        seccao,
    )
}

/// As opções da lista: os nomes, ou 0..7 sem nomes. ⚠️ `pub(crate)`: o passe diferido re-deriva-as.
pub(crate) fn layer_options() -> Vec<DropdownOption<u8>> {
    ids::INSP_PHYS_LAYER
        .iter()
        .enumerate()
        .map(|(i, &id)| DropdownOption::new(id, i as u8, crate::camadas_fisica::rotulo(i)))
        .collect()
}

/// A lista da camada, com a camada do objecto escolhida: o que o chip mostra.
#[must_use]
pub(crate) fn lista_da_camada(camada: u8) -> Dropdown<u8> {
    let mut dd = Dropdown::new(ids::INSP_PHYS_LAYER_PICK, "", layer_options());
    dd.select(camada);
    dd
}

fn sinais(t: &mut Tinta<'_>, y: f32, info: &InspectorPhysicsInfo) -> f32 {
    paint_signal_rows(
        t.scene,
        t.text,
        t.theme,
        t.hit,
        t.store,
        t.x,
        t.w,
        y,
        info.signal_tag,
        &info.signal_tag_path,
    )
}

fn dobra_zona(t: &mut Tinta<'_>, y: f32, info: &InspectorPhysicsInfo) -> f32 {
    let parada = info.force == [0.0, 0.0]
        && [
            info.area_torque,
            info.area_drag,
            info.area_density,
            info.area_form_drag,
        ] == [0.0; 4];
    let resumo = tr(if parada {
        "card.insp.phys.none"
    } else {
        "card.insp.phys.on"
    });
    let titulo = tr("card.insp.phys.zone_force");
    t.dobra(y, ids::INSP_PHYS_FOLD_ZONE, titulo, resumo, |t, y| {
        super::physics_area_rows::paint_area_rows(
            t.scene,
            t.text,
            t.theme,
            t.hit,
            t.store,
            t.x,
            t.w,
            y,
            info.force_world_axes,
        )
    })
}

pub(super) fn dobra_offset(t: &mut Tinta<'_>, y: f32) -> f32 {
    let resumo = format!(
        "{}, {}",
        t.numero(ids::INSP_PHYS_OFFSET_X),
        t.numero(ids::INSP_PHYS_OFFSET_Y)
    );
    let titulo = tr("card.insp.phys.offset");
    t.dobra(y, ids::INSP_PHYS_FOLD_OFFSET, titulo, &resumo, |t, y| {
        let rotulo = tr("component.field.collider.6");
        let (ox, oy) = (
            tr_with("panel.inspector.label_x", &[("label", &rotulo)]),
            tr_with("panel.inspector.label_y", &[("label", &rotulo)]),
        );
        let m = Some(ph2d_editor_core::widget::Unit::Meters);
        par_row(
            t.scene,
            t.text,
            t.theme,
            t.hit,
            t.store,
            t.x,
            t.w,
            y,
            Some((rotulo, Eixos::Xyzw)),
            [
                Campo::new(&ox, ids::INSP_PHYS_OFFSET_X, m),
                Campo::new(&oy, ids::INSP_PHYS_OFFSET_Y, m),
            ],
        )
    })
}

/// Os nomes das duas regras: um só quando concordam.
fn regras(info: &InspectorPhysicsInfo) -> String {
    let nome = |tag: u8| {
        super::physics_rows::COMBINE_LABELS
            .get(usize::from(tag))
            .map_or("", |k| k.tr())
    };
    let (a, b) = (
        nome(info.restitution_combine_tag),
        nome(info.friction_combine_tag),
    );
    if a == b {
        a.to_string()
    } else {
        format!("{a} \u{b7} {b}")
    }
}

pub(super) fn dobra_material(t: &mut Tinta<'_>, y: f32, info: &InspectorPhysicsInfo) -> f32 {
    let resumo = regras(info);
    let titulo = tr("card.insp.phys.material");
    t.dobra(y, ids::INSP_PHYS_FOLD_MATERIAL, titulo, &resumo, |t, y| {
        paint_combine_rows(
            t.scene,
            t.text,
            t.theme,
            t.hit,
            t.store,
            t.x,
            t.w,
            y,
            info.restitution_combine_tag,
            info.friction_combine_tag,
        )
    })
}

fn dobra_quique(t: &mut Tinta<'_>, y: f32, info: &InspectorPhysicsInfo) -> f32 {
    let resumo = format!(
        "{} \u{b7} {}",
        t.numero(ids::INSP_PHYS_RESTITUTION),
        t.numero(ids::INSP_PHYS_FRICTION)
    );
    let titulo = tr("card.insp.phys.bounce_friction");
    t.dobra(y, ids::INSP_PHYS_FOLD_BOUNCE, titulo, &resumo, |t, y| {
        let y = paint_bounce_friction(t.scene, t.text, t.theme, t.hit, t.store, t.x, t.w, y);
        paint_combine_rows(
            t.scene,
            t.text,
            t.theme,
            t.hit,
            t.store,
            t.x,
            t.w,
            y,
            info.restitution_combine_tag,
            info.friction_combine_tag,
        )
    })
}

pub(super) fn dobra_superficie(
    t: &mut Tinta<'_>,
    y: f32,
    info: &InspectorPhysicsInfo,
    com_uma_via: bool,
) -> f32 {
    let parede = tr(if info.no_wall_cling {
        "card.insp.phys.wall_cling_off"
    } else {
        "card.insp.phys.wall_cling_on"
    });
    let resumo = if com_uma_via {
        let via = tr(if info.one_way {
            "card.insp.phys.one_way_on"
        } else {
            "card.insp.phys.one_way_off"
        });
        format!("{via} \u{b7} {parede}")
    } else {
        parede.to_string()
    };
    let titulo = tr("card.insp.phys.surface");
    t.dobra(y, ids::INSP_PHYS_FOLD_SURFACE, titulo, &resumo, |t, y| {
        let y = if com_uma_via { uma_via(t, y, info) } else { y };
        paint_surface_rows(
            t.scene,
            t.text,
            t.theme,
            t.hit,
            t.store,
            t.x,
            t.w,
            y,
            info.no_wall_cling,
        )
    })
}

pub(super) fn dobra_sinais(t: &mut Tinta<'_>, y: f32, info: &InspectorPhysicsInfo) -> f32 {
    let mut nomes: Vec<&str> = [info.signal.as_str(), info.signal_leave.as_str()]
        .into_iter()
        .filter(|s| !s.is_empty())
        .collect();
    nomes.dedup();
    let resumo = if nomes.is_empty() {
        tr("card.insp.phys.none").to_string()
    } else {
        nomes.join(" \u{b7} ")
    };
    let titulo = tr("card.insp.phys.signals");
    t.dobra(y, ids::INSP_PHYS_FOLD_SIGNALS, titulo, &resumo, |t, y| {
        sinais(t, y, info)
    })
}

#[cfg(test)]
mod lista_tests {
    use super::*;

    /// O chip mostra a camada do objecto, com número ou com nome: cada opção carrega o seu próprio valor.
    #[test]
    fn a_lista_da_camada_mostra_a_camada_do_objecto_como_escolhida() {
        let mut nomes: [String; 8] = Default::default();
        nomes[2] = "Enemy".into();
        crate::camadas_fisica::set_current(nomes, [u8::MAX; 8]);
        for i in 0..8u8 {
            let dd = lista_da_camada(i);
            let esperado = if i == 2 {
                "Enemy".to_owned()
            } else {
                i.to_string()
            };
            assert_eq!(dd.selected_label(), Some(esperado.as_str()), "camada {i}");
        }
        crate::camadas_fisica::set_current(Default::default(), [u8::MAX; 8]);
    }
}
