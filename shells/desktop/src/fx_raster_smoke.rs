//! **A cena pronta para o smoke do FX RASTER** (Blur / Glow / Drop Shadow) — `PH2D_BUILD_SMOKE=33`.
//!
//! Módulo irmão do [`crate::build_smoke`] pelo teto de LOC, como o `falloff_smoke`/`contour_smoke`.
//!
//! O FX raster (plano 24) NÃO é um deformador vetorial: ele isola a forma na própria textura, borra
//! na CPU e recompõe no z dela — pixels, não geometria. Quatro estrelas, e cada uma prova uma coisa:
//!
//! - **Esquerda (controle):** estrela nítida, sem filtro. É o "antes" — a borda dura.
//! - **Blur:** a MESMA estrela, borrada (`Replace` — a imagem toma o lugar do desenho). A borda
//!   vira uma RAMPA suave; é o que separa este produto de um simples alfa reduzido.
//! - **Glow:** a silhueta borrada e tingida de ciano ATRÁS da estrela (que segue nítida por cima,
//!   `Below`). A cor da forma é descartada — só a cobertura importa.
//! - **Drop Shadow:** como o Glow, preta a 60% e DESLOCADA — a sombra cai para baixo-direita e a
//!   estrela paira sobre ela.
//!
//! ⚠️ **Nada aqui prova o SEAM do painel** — o filtro é armado no componente direto (a seção
//! *Filters* do painel é a prova do gesto, coberta pelo seam gate). Esta cena existe para o olho
//! julgar o DESENHO do produtor GPU, como a `line/physics` faz com as cenas de referência.

use ph2d_ecs::VecFilter;
use ph2d_vec_scene::{ShapeKind, VecPathId};

use crate::build_smoke::shape;

/// Os pontos de estrela (5 pontas, raio interno 0.45), reusados nas quatro.
const STAR_V: &[f64] = &[5.0, 0.45, 0.0];

pub(crate) fn frame(app: &mut crate::App, f: u32) {
    match f {
        3 => build(app),
        4 => arm(app),
        _ => {}
    }
}

fn build(app: &mut crate::App) {
    let Some(gfx) = app.gfx.as_mut() else {
        return;
    };
    let _ = gfx.tools.set_active(&ph2d_editor::ToolId::new("vector"));
    let scene = &mut gfx.vec_scene;
    // Quatro estrelas numa fileira, da esquerda para a direita. Preenchidas para o borrão/glow/
    // sombra terem corpo para espalhar.
    scene.push_path(shape(ShapeKind::Star, [-3.4, -0.8], [-1.9, 0.8], STAR_V, [235, 175, 60]));
    scene.push_path(shape(ShapeKind::Star, [-1.6, -0.8], [-0.1, 0.8], STAR_V, [235, 175, 60]));
    scene.push_path(shape(ShapeKind::Star, [0.2, -0.8], [1.7, 0.8], STAR_V, [235, 175, 60]));
    scene.push_path(shape(ShapeKind::Star, [2.0, -0.8], [3.5, 0.8], STAR_V, [110, 200, 235]));
}

fn arm(app: &mut crate::App) {
    let ids: Vec<VecPathId> = app
        .gfx
        .as_ref()
        .expect("gfx")
        .vec_scene
        .paths()
        .iter()
        .map(|p| p.id)
        .collect();
    // [0] controle (sem filtro) · [1] Blur · [2] Glow · [3] Drop Shadow.
    let (Some(&blur), Some(&glow), Some(&shadow)) = (ids.get(1), ids.get(2), ids.get(3)) else {
        eprintln!("[smoke] fx-raster: as quatro estrelas ainda não existem — o `sync` não correu");
        return;
    };
    let map = &app.vec_entities;
    let sim = &mut app.gfx.as_mut().expect("gfx").sim;
    // O arm passa pela porta única `set_filter` (a mesma que o bridge do painel usa).
    crate::fx_live::set_filter(
        sim,
        map,
        &[blur],
        Some(VecFilter {
            kind: VecFilter::BLUR,
            radius: 0.12,
            offset: [0.0, 0.0],
            color: [0.0, 0.0, 0.0, 1.0],
            opacity: 1.0,
        }),
    );
    crate::fx_live::set_filter(
        sim,
        map,
        &[glow],
        Some(VecFilter {
            kind: VecFilter::GLOW,
            radius: 0.18,
            offset: [0.0, 0.0],
            color: [0.1, 0.9, 1.0, 1.0],
            opacity: 1.0,
        }),
    );
    crate::fx_live::set_filter(
        sim,
        map,
        &[shadow],
        Some(VecFilter {
            kind: VecFilter::DROP_SHADOW,
            radius: 0.1,
            offset: [0.18, -0.18],
            color: [0.0, 0.0, 0.0, 1.0],
            opacity: 0.6,
        }),
    );
    eprintln!(
        "[smoke] FX RASTER — o produtor GPU (plano 24): a forma isolada, rasterizada num scratch,\n\
         \x20 lida de volta e borrada na CPU. Quatro estrelas, da esquerda para a direita:\n\
         \x20  1) CONTROLE — nítida, borda dura (o \"antes\").\n\
         \x20  2) BLUR — a mesma estrela borrada (radius 0.12 mundo). A borda vira RAMPA.\n\
         \x20  3) GLOW — silhueta ciano borrada (radius 0.18) ATRÁS da estrela, que fica nítida.\n\
         \x20  4) DROP SHADOW — preta 60%, borrada (radius 0.1) e deslocada [0.18,-0.18].\n\
         \x20\n\
         \x20 Dê ZOOM: o borrão CRESCE na tela (o raio é de MUNDO). Nenhum filtro armado = cena\n\
         \x20 byte-idêntica à de sempre. A seção *Filters* do painel arma isto pela UI."
    );
}
