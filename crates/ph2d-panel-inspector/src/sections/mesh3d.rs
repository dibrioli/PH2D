//! ⭐⭐⭐ **O que o Inspector mostra do CATAVENTO** — a malha 3D viva de um sprite
//! (`docs/3D/02.2`, rota B).
//!
//! # ⭐⭐⭐ A QUEIXA é a razão de esta secção existir
//!
//! Três números cabiam numa tabela genérica. O que não cabe é dizer que **um catavento num sprite
//! que nunca foi assado não faz absolutamente nada** — a fase do quadro salta-o, e não há um pixel
//! na tela que o explique. É a lei que os pincéis da escultura pagaram cinco vezes: *um controlo
//! que não faz nada e não diz porquê é indistinguível de um controlo partido*, e o artista conclui
//! que a FERRAMENTA não funciona em vez de que falta a ENTRADA dela.
//!
//! ⛔⛔ **A ordem NÃO vive aqui**, e é isso que a torna testável: ela é a porta
//! [`InspectorMesh3dInfo::queixa`], e o gate dela corre **sem um device**. *Dois `if` dentro de um
//! pintor só se medem com uma janela, e um gate `#[ignore]` é um gate que o CI nunca corre.*
//!
//! # ⚠️ Os ângulos são oferecidos em GRAUS
//!
//! O componente guarda RADIANOS (é o que a lei consome), e as duas conversões vivem no
//! [`ph2d_editor_core::mesh3d_edits`] — nunca neste pintor. *Uma unidade convertida no sítio onde
//! ela é desenhada é a segunda resposta à pergunta «que unidade é esta».*

use super::rows::aviso;
use super::*;
use ph2d_editor_core::mesh3d_edits::{
    InspectorMesh3dInfo, MESH3D_ANGLE_STEP as PASSO_ANG, MESH3D_SPIN_STEP as PASSO_SPIN,
    Mesh3dQueixa,
};
use ph2d_editor_core::widget::{SectionFold, Unit};
use ph2d_i18n::tr;

/// A chave da frase de cada queixa.
///
/// ⚠️ Um `match` e não uma tabela indexada: a posição de uma variante não é a tag de nada aqui, e
/// uma tabela criaria essa dependência do nada.
const fn chave_da_queixa(q: Mesh3dQueixa) -> &'static str {
    match q {
        Mesh3dQueixa::SemForma => "panel.inspector.mesh3d.bake_this_sprite_first",
        Mesh3dQueixa::Parado => "panel.inspector.mesh3d.it_is_standing_still",
    }
}

/// O corpo da secção — os CONTROLOS.
#[allow(clippy::too_many_arguments)]
fn corpo(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    i: &InspectorMesh3dInfo,
) -> f32 {
    let mut cur_y = y;
    // ⚠️ **A QUEIXA primeiro** — quem não vê nada mexer não quer afinar um ângulo.
    if let Some(q) = i.queixa() {
        cur_y = aviso(
            scene,
            text_system,
            theme,
            x,
            w,
            cur_y,
            tr(chave_da_queixa(q)),
            ColorToken::Text3,
        );
    }
    let seccao = ph2d_editor_core::property_row::Seccao::medida(
        text_system,
        1,
        &[
            tr("panel.inspector.mesh3d.yaw"),
            tr("panel.inspector.mesh3d.pitch"),
            tr("panel.inspector.mesh3d.spin"),
        ],
    );
    // ⚠️ **A UNIDADE é um CHIP do campo, nunca texto no rótulo** — há gate a prová-lo, e a razão é
    // que dois sítios a dizer a mesma unidade divergem no dia em que um deles mudar.
    for (label, id, passo, unidade) in [
        (
            tr("panel.inspector.mesh3d.yaw"),
            crate::ids::INSP_MESH3D_YAW,
            PASSO_ANG,
            Some(Unit::Degrees),
        ),
        (
            tr("panel.inspector.mesh3d.pitch"),
            crate::ids::INSP_MESH3D_PITCH,
            PASSO_ANG,
            Some(Unit::Degrees),
        ),
        (
            // ⭐ **`PerSecond` (`1/s`) e não uma variante nova:** voltas por segundo É uma
            // frequência, e o enum já a tem. *Uma unidade nova cujo sufixo termina em `s` teria de
            // entrar ANTES do `Seconds` na ordem do `parse_suffix`* — a armadilha que aquele
            // ficheiro documenta —, e nada aqui a justificava.
            tr("panel.inspector.mesh3d.spin"),
            crate::ids::INSP_MESH3D_SPIN,
            PASSO_SPIN,
            Some(Unit::PerSecond),
        ),
    ] {
        cur_y = super::rows::fields_row(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            cur_y,
            label,
            &[id],
            passo,
            unidade,
            seccao,
        );
    }
    cur_y
}

/// Pinta a secção. Devolve o `y` seguinte.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_mesh3d_section(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    info: &InspectorMesh3dInfo,
) -> f32 {
    let header_h = TypeToken::Md.px() + Spacing::Md.px(); // LITERAL-PX-OK: banda do cabeçalho
    let header = section_header(
        store,
        ph2d_editor_core::ids::INSP_LIVE_MESH3D_SECTION,
        tr("panel.inspector.mesh3d.live_mesh"),
    );
    paint_section_header(
        &header,
        Rect::new(x, y, w, header_h),
        scene,
        text_system,
        theme,
    );
    let Some(fold) = SectionFold::begin(
        store,
        ph2d_editor_core::ids::INSP_LIVE_MESH3D_SECTION,
        x,
        w,
        y + header_h,
        scene,
        hit_index,
    ) else {
        return y + header_h;
    };
    let cur_y = corpo(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        y + header_h,
        info,
    );
    fold.finish(store, scene, hit_index, cur_y)
}
