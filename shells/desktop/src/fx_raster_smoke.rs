//! **A cena pronta para o smoke da PILHA de FX RASTER** — `PH2D_BUILD_SMOKE=33`.
//!
//! Módulo irmão do [`crate::build_smoke`] pelo teto de LOC, como o `falloff_smoke`/`contour_smoke`.
//!
//! O FX raster (plano 24) NÃO é um deformador vetorial: ele isola a forma na própria textura, roda
//! a pilha na GPU e recompõe no z dela — pixels, não geometria. **Doze estrelas em três fileiras:**
//!
//! **Em cima, a regressão** (o que a W1/W2 já entregavam): controle nítido · Blur · Glow · Drop
//! Shadow.
//!
//! **No meio, o CATÁLOGO da W3** — os quatro tipos novos, cada um sozinho: Inner Shadow · Inner
//! Glow · Outline · Color Overlay.
//!
//! **Embaixo, a COMPOSIÇÃO** — o que a pilha entrega e um filtro único não:
//! - **A PILHA INTEIRA** (`Drop Shadow → Blur → Glow`): três degraus encadeados numa forma só.
//! - **O PAR DE ORDEM** (`Glow → Blur` × `Blur → Glow`): os MESMOS dois degraus, trocados. Se a
//!   ordem não importasse, as duas estrelas seriam idênticas — e o ponto da W2 é que não são.
//! - **O STICKER** (`Outline → Drop Shadow`): o contorno duro e a sombra por baixo dele. É o
//!   desenho que nenhum dos dois faz sozinho, e o que uma Gaussiana nunca desenha.
//!
//! ⚠️ **Nada aqui prova o SEAM do painel** — a pilha é armada no componente direto (a seção
//! *Filters* do painel é a prova do gesto, coberta pelo seam gate). Esta cena existe para o olho
//! julgar o DESENHO do produtor GPU, como a `line/physics` faz com as cenas de referência.

use ph2d_ecs::{FxOp, VecFilter};
use ph2d_vec_scene::{ShapeKind, VecPathId};

use crate::build_smoke::shape;

/// Os pontos de estrela (5 pontas, raio interno 0.45), reusados em todas.
const STAR_V: &[f64] = &[5.0, 0.45, 0.0];

/// As quatro colunas e as três fileiras (canto inferior-esquerdo de cada estrela; lado 1.4).
const COLS: [f64; 4] = [-3.6, -1.9, -0.2, 1.5];
const ROWS: [f64; 3] = [1.5, -0.1, -1.7];
const SIDE: f64 = 1.4;

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
    // Ordem de push = a ordem dos índices que o `arm` endereça: fileira a fileira, da esquerda.
    for y in ROWS {
        for x in COLS {
            scene.push_path(shape(
                ShapeKind::Star,
                [x, y],
                [x + SIDE, y + SIDE],
                STAR_V,
                [235, 175, 60],
            ));
        }
    }
}

/// Um degrau com raio, cor e opacidade explícitos (o resto vem do default do tipo).
fn op(kind: u8, radius: f32, color: [f32; 4], opacity: f32) -> FxOp {
    FxOp {
        radius,
        color,
        opacity,
        ..FxOp::new(kind)
    }
}

const CYAN: [f32; 4] = [0.1, 0.9, 1.0, 1.0];
const WHITE: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
const BLACK: [f32; 4] = [0.0, 0.0, 0.0, 1.0];
const SKY: [f32; 4] = [0.35, 0.75, 1.0, 1.0];

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
    if ids.len() < COLS.len() * ROWS.len() {
        eprintln!("[smoke] fx-raster: as doze estrelas ainda não existem — o `sync` não correu");
        return;
    }
    let blur = op(FxOp::BLUR, 0.12, BLACK, 1.0);
    let glow = op(FxOp::GLOW, 0.18, CYAN, 1.0);
    let shadow = FxOp {
        offset: [0.18, -0.18],
        ..op(FxOp::DROP_SHADOW, 0.1, BLACK, 0.6)
    };
    let inner_shadow = FxOp {
        offset: [0.12, -0.12],
        ..op(FxOp::INNER_SHADOW, 0.1, BLACK, 0.9)
    };
    let inner_glow = op(FxOp::INNER_GLOW, 0.12, CYAN, 1.0);
    let outline = op(FxOp::OUTLINE, 0.07, WHITE, 1.0);
    let overlay = op(FxOp::COLOR_OVERLAY, 0.0, SKY, 1.0);
    // `[0]` é o controle (sem pilha). Depois: a regressão, o catálogo novo e as composições.
    let stacks: [(usize, Vec<FxOp>); 11] = [
        (1, vec![blur]),
        (2, vec![glow]),
        (3, vec![shadow]),
        (4, vec![inner_shadow]),
        (5, vec![inner_glow]),
        (6, vec![outline]),
        (7, vec![overlay]),
        (8, vec![shadow, blur, glow]),
        (9, vec![glow, blur]),
        (10, vec![blur, glow]),
        (11, vec![outline, shadow]),
    ];
    let map = &app.vec_entities;
    let sim = &mut app.gfx.as_mut().expect("gfx").sim;
    // O arm passa pela porta única `set_filter` (a mesma que o bridge do painel usa).
    for (i, ops) in stacks {
        crate::fx_live::set_filter(sim, map, &[ids[i]], Some(VecFilter { ops }));
    }
    eprintln!(
        "[smoke] A PILHA DE FX RASTER — 100% na GPU (plano 24, W3: SETE tipos). Doze estrelas:\n\
         \x20 CIMA (a regressão da W1/W2 — cada tipo continua a fazer o que fazia):\n\
         \x20  1) CONTROLE — nítida, borda dura (o \"antes\").\n\
         \x20  2) BLUR — a mesma estrela borrada. A borda vira RAMPA.\n\
         \x20  3) GLOW — halo ciano POR BAIXO da estrela, que fica nítida.\n\
         \x20  4) DROP SHADOW — preta 60%, borrada e deslocada [0.18,-0.18].\n\
         \x20 MEIO (o CATÁLOGO novo — os quatro tipos desta wave, sozinhos):\n\
         \x20  5) INNER SHADOW — a sombra cai PARA DENTRO, deslocada: a estrela lê como um\n\
         \x20     RECORTE. Medido: a borda de dentro escurece para 158 e o miolo fica em 255.\n\
         \x20  6) INNER GLOW — o mesmo sem deslocamento, em ciano: um brilho que abraça a borda\n\
         \x20     por dentro. Nada disto vaza para FORA da silhueta (0 texels, byte a byte).\n\
         \x20  7) OUTLINE — contorno BRANCO de borda DURA. Não é um glow forte: medido, a borda\n\
         \x20     para exactamente na largura pedida (3,5 px para 4 · 7,5 px para 8) e a\n\
         \x20     transição cabe em ~1 px, contra os ~3 sigma de um glow.\n\
         \x20  8) COLOR OVERLAY — a MESMA estrela repintada de azul, sem borrar e sem mover um\n\
         \x20     texel de cobertura. É pontual: um dispatch, margem zero (medido: 6 overlays\n\
         \x20     custam 0,282 ms contra 0,646 ms de 6 borrões).\n\
         \x20 BAIXO (a COMPOSIÇÃO — o que só uma pilha entrega):\n\
         \x20  9) A PILHA INTEIRA — Drop Shadow -> Blur -> Glow, TRÊS degraus numa forma só.\n\
         \x20 10) Glow -> Blur   — o borrão vem DEPOIS: lava o halo junto com a estrela.\n\
         \x20 11) Blur -> Glow   — o borrão vem ANTES: o halo nasce da silhueta JÁ engordada.\n\
         \x20 12) O STICKER — Outline -> Drop Shadow: o contorno duro, e a sombra por baixo DELE.\n\
         \x20 As estrelas 10 e 11 têm os MESMOS dois degraus, trocados de ordem: se parecerem\n\
         \x20 iguais, a pilha não está compondo.\n\
         \x20\n\
         \x20 Dê ZOOM: o borrão CRESCE na tela (o raio é de MUNDO). No painel, a seção *Filters*\n\
         \x20 arma tudo isto pela UI — sete botões \"Add\", e cada card oferece SÓ os controles do\n\
         \x20 tipo dele (o Color Overlay não tem Radius; só as sombras têm Offset)."
    );
}
