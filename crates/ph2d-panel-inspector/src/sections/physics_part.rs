//! **A face de PEÇA do §11** — o que se mostra de um `Collider` que não é um
//! corpo (W-PartFace).
//!
//! A W-Compound deu ao artista o gesto de CRIAR uma peça (*Add Shape to X*), o
//! contorno para vê-la e a ponte para simulá-la. O que faltava era a volta:
//! **selecioná-la e editá-la**. Sem esta face ela caía na vazia, que
//! - dizia **"Not simulated"**, o oposto da verdade (uma peça É simulada, como
//!   forma do corpo ancestral);
//! - mostrava as SEMENTES em vez da forma autorada — medido, uma barra
//!   `0,17 × 0,91` com offset `[0,13, −0,07]`, densidade `3,5` e camada `2`
//!   aparecia como caixa `0,50 × 0,50`, offset `[0, 0]`, densidade `1,00`,
//!   camada `0`;
//! - e **re-oferecia a porta que a criou**, cujo clique reescreve o collider com
//!   os defaults (a forma autorada some, em silêncio).
//!
//! ⚠️ **As rows são exatamente as que a PONTE lê de uma peça**, não as que
//! caberiam na tela: `reconcile_parts` monta o `BodyDesc` a partir do `Collider`
//! (forma · offset · densidade · quique · atrito · camada · sensor) e do marcador
//! `OneWayPlatform`, e passa constantes para tudo que é do CORPO (gravidade,
//! velocidade inicial, CCD, travas, massa, dominância, damping). Um knob de corpo
//! aqui seria um controle que o solver ignora — a lei que esta seção repete desde
//! o W2b.
//!
//! ⚠️ **A ZONA fica de fora pelo mesmo teste:** a ponte não lê `AreaEffector` nem
//! nenhum irmão de uma peça, então marcar *Trigger* numa peça a faz atravessar
//! (isso o solver honra) mas os sete números de zona não teriam leitor. Por isso
//! o bloco de área virou função própria (`physics_rows::paint_area_rows`) e só a
//! face de CORPO a chama.

use super::rows::{num_row, seg_row};
use super::*;
use ph2d_editor_core::screens::hero::InspectorPhysicsInfo;

/// Pinta a face de peça e devolve o `y` final da seção.
#[allow(clippy::too_many_arguments)]
pub(super) fn paint_part_face(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    info: &InspectorPhysicsInfo,
    x: f32,
    w: f32,
    y: f32,
) -> f32 {
    let mut yy = y;
    let h = ROW_H_PX;
    let label_font = TypeToken::Sm.px();
    // O cabeçalho diz o que esta coisa É. Um collider é invisível e a hierarquia
    // pode ter um grupo no meio, então o DONO é nomeado — é a única coisa que o
    // artista não consegue inferir da tela.
    let head = if info.part_owner.is_empty() {
        // Um collider sem corpo nenhum acima: honesto, e o contorno concorda (ele
        // também não o desenha). Não é a mesma coisa que "ainda não é físico".
        "Shape with no body above it \u{00b7} not simulated".to_string()
    } else {
        format!(
            "Shape of {} \u{00b7} simulated as part of it",
            info.part_owner
        )
    };
    paint_text(
        text_system,
        scene,
        &head,
        x,
        yy + (h - label_font) * 0.5,
        label_font,
        w,
        resolve(ColorToken::Text3, theme),
    );
    yy += h;

    yy = seg_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        yy,
        "Collider",
        ids::INSP_LIVE_PHYSICS_COLOR,
        &ids::INSP_PHYS_SHAPE,
        &super::physics::SHAPE_LABELS,
        info.shape_tag,
    );
    yy = super::physics::paint_shape_dims(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        yy,
        info.shape_tag,
    );
    for (label, id) in [
        ("Offset X (m)", ids::INSP_PHYS_OFFSET_X),
        ("Offset Y (m)", ids::INSP_PHYS_OFFSET_Y),
        // A densidade de uma peça é REAL: ela contribui para a massa do corpo
        // composto. O toggle Auto|Manual do W-Mass fica de fora porque o
        // `MassOverride` é do CORPO — uma peça não tem massa própria a sobrepor.
        ("Density", ids::INSP_PHYS_DENSITY),
    ] {
        yy = num_row(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            yy,
            label,
            id,
        );
    }
    yy = super::physics_rows::paint_material_rows(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        yy,
        info.restitution_combine_tag,
        info.friction_combine_tag,
    );
    yy = super::physics_rows::paint_collision_rows(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        yy,
        info.layer,
        info.is_sensor,
        info.one_way,
    );
    super::physics_doors::paint_part_doors(scene, text_system, theme, hit_index, store, x, w, yy)
}
