//! ⭐⭐ **AS PONTES DE TIPO entre o painel e o renderizador** — a luz e a lente.
//!
//! Irmão (`#[path]`) do [`super`], e o corte é por RESPONSABILIDADE: lá mora *o que o painel
//! pede e como a cena o aplica*; aqui *como se traduz um tipo de UI num tipo de device e de
//! volta* — conceitos escritos duas vezes de propósito (o painel não arrasta o `wgpu`), com a
//! ida-e-volta gateada. ⚠️ O gatilho foi o tecto de `700` LOC do gate da workspace, que o
//! `panel.rs` passou (`715`) na integração de 2026-09-25 (rodada 03), quando a `line/3DModeling`
//! trouxe a lente e o modo `Pbr` para cima da `line/sculpt3d`. ⛔ Cura por CORTE, nunca uma entrada
//! no `FILE_OVERAGE_OK`.

/// ⭐⭐ **A PONTE entre os dois tipos de «com que luz»** — o do painel
/// ([`ph2d_panel_sculpt3d::state::LightMode`]) e o do device
/// ([`ph2d_mesh_render::Lighting`]).
///
/// ⛔⛔ **Os dois existem porque o painel NÃO conhece o renderizador** (ele é UI
/// e não arrasta o `wgpu`), logo o conceito está escrito duas vezes de
/// propósito. *O que torna a duplicação honesta é o gate de IDA-E-VOLTA*
/// (`panel_luz_tests`): toda luz do device resolve para um modo do painel e
/// volta ao mesmo, e todo modo do painel é alcançável — sem isso, um terceiro
/// modo nasceria de um lado só e o chip escolheria outra coisa em silêncio.
pub(crate) fn luz_para_o_painel(
    l: ph2d_mesh_render::Lighting,
) -> ph2d_panel_sculpt3d::state::LightMode {
    use ph2d_panel_sculpt3d::state::LightMode;
    match l {
        ph2d_mesh_render::Lighting::Flat => LightMode::Flat,
        ph2d_mesh_render::Lighting::Rig => LightMode::Rig,
        ph2d_mesh_render::Lighting::Pbr => LightMode::Pbr,
        ph2d_mesh_render::Lighting::Matcap(i) => LightMode::Matcap(i),
    }
}

/// A volta — ver [`luz_para_o_painel`].
pub(crate) fn luz_do_painel(
    l: ph2d_panel_sculpt3d::state::LightMode,
) -> ph2d_mesh_render::Lighting {
    use ph2d_panel_sculpt3d::state::LightMode;
    match l {
        LightMode::Flat => ph2d_mesh_render::Lighting::Flat,
        LightMode::Rig => ph2d_mesh_render::Lighting::Rig,
        LightMode::Pbr => ph2d_mesh_render::Lighting::Pbr,
        LightMode::Matcap(i) => ph2d_mesh_render::Lighting::Matcap(i),
    }
}

/// ⭐⭐⭐ **A PONTE entre os dois tipos de «com que lente»** — o do painel
/// ([`ph2d_panel_sculpt3d::state::LensMode`]) e o da câmera
/// ([`ph2d_mesh_render::Lens`]).
///
/// ⛔⛔ **Ela existe pela MESMA razão da irmã da luz, e não por simetria:** o painel é UI e não
/// arrasta o `wgpu`, logo o conceito está escrito duas vezes de propósito. *O que torna a
/// duplicação honesta é o gate de IDA-E-VOLTA nos DOIS sentidos* (`panel_lente_tests`) — sem a
/// volta, uma ponte que colapsasse as duas lentes numa deixaria um chip **morto sob o dedo** com a
/// ida verde por cima.
pub(crate) fn lente_para_o_painel(
    l: ph2d_mesh_render::Lens,
) -> ph2d_panel_sculpt3d::state::LensMode {
    use ph2d_panel_sculpt3d::state::LensMode;
    match l {
        ph2d_mesh_render::Lens::Perspective => LensMode::Perspective,
        ph2d_mesh_render::Lens::Ortho => LensMode::Ortho,
    }
}

/// A volta — ver [`lente_para_o_painel`].
pub(crate) fn lente_do_painel(l: ph2d_panel_sculpt3d::state::LensMode) -> ph2d_mesh_render::Lens {
    use ph2d_panel_sculpt3d::state::LensMode;
    match l {
        LensMode::Perspective => ph2d_mesh_render::Lens::Perspective,
        LensMode::Ortho => ph2d_mesh_render::Lens::Ortho,
    }
}

/// **A PONTE DA LUZ, nos dois sentidos** — ver [`panel_luz_tests`].
#[cfg(test)]
#[path = "panel_luz_tests.rs"]
mod panel_luz_tests;

/// **A PONTE DA LENTE, nos dois sentidos** — ver [`panel_lente_tests`].
#[cfg(test)]
#[path = "panel_lente_tests.rs"]
mod panel_lente_tests;
