//! **GROW / SHRINK** (plano 24 W7) — a cena `PH2D_BUILD_SMOKE=36`.
//!
//! Irmã da `fx_turbulence_smoke` (=35) e da `fx_blend_smoke` (=34), no mesmo molde: um **A/B**, não
//! um catálogo. Quatro pares, e em cada um a MESMA estrela — só uma coisa difere.
//!
//! ⚠️ **O terceiro par é o headline, e não é decoração:** ele mostra as MESMAS duas operações em
//! ORDENS trocadas. Um degrau que medisse a FORMA em vez da IMAGEM desenharia os dois iguais (a
//! morfologia recortaria o contorno de volta à silhueta nos dois casos), e a pilha deixaria de
//! compor exatamente onde ela promete compor.
//!
//! Rodar: `cd <worktree> && env PH2D_BUILD_SMOKE=36 cargo run -p ph2d-host-desktop --release`.

use ph2d_ecs::{FxOp, VecFilter};
use ph2d_vec_scene::{ShapeKind, VecPathId};

use crate::build_smoke::shape;

/// Os pontos de estrela — os mesmos das cenas irmãs.
const STAR_V: &[f64] = &[5.0, 0.45, 0.0];

const COLS: [f64; 4] = [-3.6, -1.95, -0.3, 1.35];
const ROWS: [f64; 2] = [1.2, -1.0];
const SIDE: f64 = 1.35;
/// Quantas estrelas a cena monta: quatro pares.
const STARS: usize = 8;

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
    for i in 0..STARS {
        let x = COLS[i % 4];
        let y = ROWS[i / 4];
        gfx.vec_scene.push_path(shape(
            ShapeKind::Star,
            [x, y],
            [x + SIDE, y + SIDE],
            STAR_V,
            [190, 140, 70],
        ));
    }
}

/// Um degrau de morfologia — o `grow` é o único knob, e o SINAL é a operação.
fn morph(grow: f32) -> FxOp {
    FxOp {
        grow,
        opacity: 1.0,
        ..FxOp::new(FxOp::MORPHOLOGY)
    }
}

/// Um contorno escuro, para o par da ORDEM.
fn outline(width: f32) -> FxOp {
    FxOp {
        radius: width,
        color: [0.05, 0.05, 0.08, 1.0],
        opacity: 1.0,
        ..FxOp::new(FxOp::OUTLINE)
    }
}

/// Um borrão, para o par do CHOKE.
fn blur(radius: f32) -> FxOp {
    FxOp {
        radius,
        opacity: 1.0,
        ..FxOp::new(FxOp::BLUR)
    }
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
    if ids.len() < STARS {
        eprintln!(
            "[smoke] fx-morphology: as oito estrelas ainda não existem — o `sync` não correu"
        );
        return;
    }
    let stacks: [(usize, Vec<FxOp>); STARS] = [
        // Par 1 — O SINAL: o mesmo knob nas duas pontas.
        (0, vec![morph(-0.10)]),
        (1, vec![morph(0.10)]),
        // Par 2 — O ELEMENTO: crescer pouco × crescer muito. As pontas ARREDONDAM, porque a régua
        // é a distância euclidiana (um disco). Com o retângulo do `feMorphology` elas ficariam
        // quadradas e o alcance na diagonal seria 1,41× maior.
        (2, vec![morph(0.05)]),
        (3, vec![morph(0.30)]),
        // Par 3 — A ORDEM (o headline): as MESMAS duas operações, trocadas.
        (4, vec![outline(0.05), morph(0.08)]),
        (5, vec![morph(0.08), outline(0.05)]),
        // Par 4 — O USO: o *choke* clássico. Encolher ANTES de borrar mantém o macio dentro da
        // silhueta em vez de o deixar vazar para fora dela.
        (6, vec![blur(0.10)]),
        (7, vec![morph(-0.10), blur(0.10)]),
    ];
    let map = &app.vec_entities;
    let sim = &mut app.gfx.as_mut().expect("gfx").sim;
    for (i, ops) in stacks {
        crate::fx_live::set_filter(sim, map, &[ids[i]], Some(VecFilter { ops }));
    }
    eprintln!(
        "[smoke] GROW / SHRINK (plano 24 W7) — o `feMorphology` do SVG (dilate/erode) num knob\n\
         \x20COM SINAL, que é como o AE (Simple Choker), o Blender (Dilate/Erode) e o Illustrator\n\
         \x20(Offset Path) o embrulham. Quatro PARES: a MESMA estrela, uma coisa de diferença.\n\
         \x20\n\
         \x20 Os números saem dos gates de GPU (RTX), medidos ANTES desta mensagem. Eles descrevem\n\
         \x20 ONDE A BORDA FICOU — que é o que a operação É.\n\
         \x20\n\
         \x20 FILEIRA 1:\n\
         \x20  1) SHRINK -0,10 — a silhueta AFINA. O meio do slider é o zero, e ele é byte-idêntico\n\
         \x20     a não haver degrau nenhum (o artista atravessa o zero a arrastar).\n\
         \x20  2) GROW +0,10 — a mesma estrela ENGORDA. Medido: pedir `r` move o contorno `r`.\n\
         \x20  3) GROW +0,05 — pouco: a estrela mal muda de silhueta.\n\
         \x20  4) GROW +0,30 — muito: repare que as PONTAS ARREDONDAM. É a assinatura do disco\n\
         \x20     euclidiano — medido na quina de uma caixa, crescer 10 px alcança **10,00 px** na\n\
         \x20     diagonal; o elemento RETANGULAR do SVG alcançaria 14,14 e deixaria a quina\n\
         \x20     quadrada.\n\
         \x20\n\
         \x20 FILEIRA 2 (a ORDEM e o USO):\n\
         \x20  5) Outline -> Grow: a morfologia recebe a imagem JÁ contornada e ENGORDA O TRAÇO.\n\
         \x20  6) Grow -> Outline: a forma engorda primeiro e o traço é desenhado à volta dela —\n\
         \x20     traço FINO sobre estrela GORDA. As MESMAS duas operações, desenhos diferentes:\n\
         \x20     é a tese da pilha. ⚠️ Medido no meio-plano: o contorno sozinho acaba em 71,50 e\n\
         \x20     depois de Grow(3) em 74,50 — ele ENGORDOU 3 px. Um degrau que medisse a FORMA o\n\
         \x20     teria RECORTADO de volta a 67, e os dois cards desta fileira sairiam iguais.\n\
         \x20  7) Blur sozinho — o macio VAZA para fora da silhueta.\n\
         \x20  8) Shrink -> Blur — o *choke*: encolher antes de borrar mantém o macio DENTRO. É\n\
         \x20     exatamente o que o Choke do Photoshop e o Simple Choker do AE existem para fazer.\n\
         \x20\n\
         \x20 NO PAINEL (é o que fecha o smoke): selecione uma estrela, abra FILTERS, e o card\n\
         \x20 'Grow / Shrink' tem UM knob — **Amount**, com o zero no MEIO do curso e o sinal no\n\
         \x20 readout. Arraste-o de ponta a ponta: a forma tem de afinar e engordar continuamente,\n\
         \x20 e ao passar pelo meio tem de ficar EXATAMENTE como estava."
    );
}
