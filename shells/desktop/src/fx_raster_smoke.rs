//! **A cena pronta para o smoke da PILHA de FX RASTER** — `PH2D_BUILD_SMOKE=33`.
//!
//! Módulo irmão do [`crate::build_smoke`] pelo teto de LOC, como o `falloff_smoke`/`contour_smoke`.
//!
//! O FX raster (plano 24) NÃO é um deformador vetorial: ele isola a forma na própria textura, roda
//! a pilha na GPU e recompõe no z dela — pixels, não geometria. Sete estrelas em duas fileiras:
//!
//! **Em cima, os degraus SOZINHOS** (a regressão da W1 — cada tipo continua a fazer o que fazia):
//! controle nítido · Blur · Glow · Drop Shadow.
//!
//! **Embaixo, a W2** — o que a pilha entrega e um filtro único não:
//! - **A PILHA INTEIRA** (`Drop Shadow → Blur → Glow`): três degraus encadeados numa forma só.
//! - **O PAR DE ORDEM** (`Glow → Blur` × `Blur → Glow`): os MESMOS dois degraus, trocados. Se a
//!   ordem não importasse, as duas estrelas seriam idênticas — e o ponto da wave inteira é que não
//!   são. Blur DEPOIS lava o halo junto com a forma; Blur ANTES engorda a silhueta de que o halo
//!   nasce, e o halo fica com borda própria.
//!
//! ⚠️ **Nada aqui prova o SEAM do painel** — a pilha é armada no componente direto (a seção
//! *Filters* do painel é a prova do gesto, coberta pelo seam gate). Esta cena existe para o olho
//! julgar o DESENHO do produtor GPU, como a `line/physics` faz com as cenas de referência.

use ph2d_ecs::{FxOp, VecFilter};
use ph2d_vec_scene::{ShapeKind, VecPathId};

use crate::build_smoke::shape;

/// Os pontos de estrela (5 pontas, raio interno 0.45), reusados em todas.
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
    // Fileira de cima: os quatro degraus sozinhos (a regressão da W1).
    for (i, col) in [-3.4, -1.6, 0.2, 2.0].iter().enumerate() {
        let tint = if i == 3 {
            [110, 200, 235]
        } else {
            [235, 175, 60]
        };
        scene.push_path(shape(
            ShapeKind::Star,
            [*col, 0.5],
            [col + 1.5, 2.1],
            STAR_V,
            tint,
        ));
    }
    // Fileira de baixo: a pilha inteira + o par de ordem.
    for col in [-2.5, -0.7, 1.1] {
        scene.push_path(shape(
            ShapeKind::Star,
            [col, -2.1],
            [col + 1.5, -0.5],
            STAR_V,
            [235, 175, 60],
        ));
    }
}

/// Um degrau com raio e cor explícitos (os outros campos vêm do default do tipo).
fn op(kind: u8, radius: f32, color: [f32; 4], opacity: f32) -> FxOp {
    FxOp {
        radius,
        color,
        opacity,
        ..FxOp::new(kind)
    }
}

const CYAN: [f32; 4] = [0.1, 0.9, 1.0, 1.0];
const BLACK: [f32; 4] = [0.0, 0.0, 0.0, 1.0];

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
    if ids.len() < 7 {
        eprintln!("[smoke] fx-raster: as sete estrelas ainda não existem — o `sync` não correu");
        return;
    }
    let blur = op(FxOp::BLUR, 0.12, BLACK, 1.0);
    let glow = op(FxOp::GLOW, 0.18, CYAN, 1.0);
    let shadow = FxOp {
        offset: [0.18, -0.18],
        ..op(FxOp::DROP_SHADOW, 0.1, BLACK, 0.6)
    };
    // `[0]` é o controle (sem pilha). Depois: os três singles, a pilha inteira e o par de ordem.
    let stacks: [(usize, Vec<FxOp>); 6] = [
        (1, vec![blur]),
        (2, vec![glow]),
        (3, vec![shadow]),
        (4, vec![shadow, blur, glow]),
        (5, vec![glow, blur]),
        (6, vec![blur, glow]),
    ];
    let map = &app.vec_entities;
    let sim = &mut app.gfx.as_mut().expect("gfx").sim;
    // O arm passa pela porta única `set_filter` (a mesma que o bridge do painel usa).
    for (i, ops) in stacks {
        crate::fx_live::set_filter(sim, map, &[ids[i]], Some(VecFilter { ops }));
    }
    eprintln!(
        "[smoke] A PILHA DE FX RASTER — 100% na GPU (plano 24, W2). Sete estrelas:\n\
         \x20 CIMA (os degraus sozinhos, a regressão da W1):\n\
         \x20  1) CONTROLE — nítida, borda dura (o \"antes\").\n\
         \x20  2) BLUR — a mesma estrela borrada (radius 0.12 mundo). A borda vira RAMPA.\n\
         \x20  3) GLOW — halo ciano (radius 0.18) POR BAIXO da estrela, que fica nítida.\n\
         \x20  4) DROP SHADOW — preta 60%, borrada (radius 0.1) e deslocada [0.18,-0.18].\n\
         \x20 BAIXO (o que a W2 acrescenta):\n\
         \x20  5) A PILHA INTEIRA — Drop Shadow -> Blur -> Glow, TRÊS degraus numa forma só.\n\
         \x20  6) Glow -> Blur   — o borrão vem DEPOIS: lava o halo junto com a estrela.\n\
         \x20  7) Blur -> Glow   — o borrão vem ANTES: o halo nasce da silhueta JÁ engordada.\n\
         \x20 As duas últimas têm os MESMOS dois degraus, trocados de ordem. Se parecerem iguais,\n\
         \x20 a pilha não está compondo — e é isso que esta wave entrega.\n\
         \x20\n\
         \x20 Dê ZOOM: o borrão CRESCE na tela (o raio é de MUNDO). Nenhuma pilha armada = cena\n\
         \x20 byte-idêntica à de sempre. A seção *Filters* do painel arma isto pela UI:\n\
         \x20 selecione uma forma, \"Add Blur\"/\"Add Glow\"/\"Add Drop Shadow\", e use as setas\n\
         \x20 do card para reordenar."
    );
}
