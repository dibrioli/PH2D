//! **A TRADUÇÃO do vocabulário** entre a tool (o que o painel mostra) e o documento (o que a
//! geometria guarda) — irmão do [`super::vector_bridge`] pelo teto de 600 LOC da shell (HR-18).
//!
//! ⚠️ O corte é por RESPONSABILIDADE: ali mora *o que a ponte FAZ neste frame* (visibilidade,
//! read-back do picker, restyle, publicação), aqui *como um nome de um lado vira o nome do outro*.
//! É a mesma linha que o `layout_live::frame_style` traça, e pelo mesmo motivo — os `match` são
//! exaustivos, então um membro novo num dos vocabulários **não compila** até alguém dizer o que
//! ele é do outro lado, em vez de cair num `_ =>` que o desenharia errado.

use ph2d_vec_scene::{LineCap, LineJoin};

/// Map the UI-facing `StrokeCap`/`StrokeJoin` to the geometry enums.
pub(super) fn line_cap(c: ph2d_tool_vector::StrokeCap) -> LineCap {
    use ph2d_tool_vector::StrokeCap;
    match c {
        StrokeCap::Butt => LineCap::Butt,
        StrokeCap::Round => LineCap::Round,
        StrokeCap::Square => LineCap::Square,
    }
}
pub(super) fn line_join(j: ph2d_tool_vector::StrokeJoin) -> LineJoin {
    use ph2d_tool_vector::StrokeJoin;
    match j {
        StrokeJoin::Miter => LineJoin::Miter,
        StrokeJoin::Round => LineJoin::Round,
        StrokeJoin::Bevel => LineJoin::Bevel,
    }
}

/// Map the geometry `VertexKind` to the panel's UI-facing `VertexType`.
/// O que a seleção de vértices tem em comum, no vocabulário do painel. `Mixed` viaja porque nenhum
/// chip descreve uma seleção de tipos diferentes — publicar o tipo do PRIMÁRIO fazia o painel
/// afirmar um deles (auditoria do plano 25, item 5).
pub(super) fn vertex_sel_of(sel: ph2d_vec_edit::SelectedKind) -> ph2d_tool_vector::VertexSel {
    use ph2d_tool_vector::VertexSel;
    match sel {
        ph2d_vec_edit::SelectedKind::Uniform(k) => VertexSel::Uniform(vertex_type_of(k)),
        ph2d_vec_edit::SelectedKind::Mixed => VertexSel::Mixed,
    }
}

fn vertex_type_of(k: ph2d_vec_scene::VertexKind) -> ph2d_tool_vector::VertexType {
    use ph2d_tool_vector::VertexType;
    use ph2d_vec_scene::VertexKind;
    match k {
        VertexKind::Corner => VertexType::Corner,
        VertexKind::Smooth => VertexType::Smooth,
        VertexKind::Symmetric => VertexType::Symmetric,
    }
}
