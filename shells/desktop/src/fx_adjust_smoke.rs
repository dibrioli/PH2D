//! **COLOR ADJUST** (plano 24 W8) — a cena `PH2D_BUILD_SMOKE=37`.
//!
//! Irmã da `fx_morphology_smoke` (=36), da `fx_turbulence_smoke` (=35) e da `fx_blend_smoke` (=34),
//! no mesmo molde: um **A/B**, não um catálogo. Quatro pares, e em cada um a MESMA estrela — só
//! uma coisa difere.
//!
//! ⚠️ **A estrela é COLORIDA de propósito, e isso é a lei e não a cena:** matiz e saturação são
//! rotação e escala do CROMA, e um pixel sem croma não tem o que girar. Numa arte cinzenta só o
//! Brilho move alguma coisa — o quarto par mostra exactamente isso.
//!
//! Rodar: `cd <worktree> && env PH2D_BUILD_SMOKE=37 cargo run -p ph2d-host-desktop --release`.

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
/// A cor da estrela — âmbar, croma médio. ⚠️ Uma cor VIVA (o vermelho puro) fica na borda do
/// gamut e o giro dela é CORTADO na volta a 8 bits (medido: 0,641 do croma sobrevive); esta gira
/// limpa em toda direção.
const AMBER: [u8; 3] = [190, 140, 70];
/// A estrela do último par — CINZA, para mostrar onde a lei tem pontos fixos.
const GREY: [u8; 3] = [150, 150, 150];

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
        // As duas últimas são o par do CINZA.
        let rgb = if i >= STARS - 2 { GREY } else { AMBER };
        gfx.vec_scene.push_path(shape(
            ShapeKind::Star,
            [x, y],
            [x + SIDE, y + SIDE],
            STAR_V,
            rgb,
        ));
    }
}

/// Um degrau de ajuste de cor — a matiz em VOLTAS (a unidade do modelo; o painel mostra graus).
fn adjust(hue: f32, sat: f32, bright: f32) -> FxOp {
    FxOp {
        hue,
        sat,
        bright,
        opacity: 1.0,
        ..FxOp::new(FxOp::COLOR_ADJUST)
    }
}

/// Um brilho externo AZUL, para o par da ORDEM — a cor dele é autorada, então ela só gira se o
/// ajuste vier DEPOIS.
fn glow() -> FxOp {
    FxOp {
        radius: 0.16,
        color: [0.25, 0.45, 1.0, 1.0],
        opacity: 1.0,
        ..FxOp::new(FxOp::GLOW)
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
        eprintln!("[smoke] fx-adjust: as oito estrelas ainda não existem — o `sync` não correu");
        return;
    }
    let stacks: [(usize, Vec<FxOp>); STARS] = [
        // Par 1 — A MATIZ: o mesmo knob nas duas pontas (−90° e +90°).
        (0, vec![adjust(-0.25, 0.0, 0.0)]),
        (1, vec![adjust(0.25, 0.0, 0.0)]),
        // Par 2 — A SATURAÇÃO: drenada até o cinza × croma dobrado.
        (2, vec![adjust(0.0, -1.0, 0.0)]),
        (3, vec![adjust(0.0, 1.0, 0.0)]),
        // Par 3 — A ORDEM: as MESMAS duas operações, trocadas. O halo do Glow tem cor AUTORADA,
        // então ele só gira se o ajuste o receber já desenhado.
        (4, vec![adjust(0.35, 0.0, 0.0), glow()]),
        (5, vec![glow(), adjust(0.35, 0.0, 0.0)]),
        // Par 4 — OS PONTOS FIXOS, numa estrela CINZA: a matiz não a move, o brilho move.
        (6, vec![adjust(0.35, 0.8, 0.0)]),
        (7, vec![adjust(0.0, 0.0, -0.45)]),
    ];
    let map = &app.vec_entities;
    let sim = &mut app.gfx.as_mut().expect("gfx").sim;
    for (i, ops) in stacks {
        crate::fx_live::set_filter(sim, map, &[ids[i]], Some(VecFilter { ops }));
    }
    eprintln!(
        "[smoke] COLOR ADJUST (plano 24 W8) — o `feColorMatrix` do SVG (hueRotate + saturate + o\n\
         \x20brilho que a matrix exprime) na ficha *Hue/Saturation* que Photoshop, AE, Krita e\n\
         \x20Blender desenharam. Quatro PARES: a MESMA estrela, uma coisa de diferença.\n\
         \x20\n\
         \x20 ⚠️ A LEI NÃO É NOVA — é a MESMA do ajuste Hue/Saturation que o PAINTER já ship, pelo\n\
         \x20 MESMO kernel WGSL (extraído para um arquivo compartilhado ao ganhar este 2º\n\
         \x20 consumidor). Medido pelo gate contra a implementação de CPU daquela crate: pior\n\
         \x20 divergência **1 nível de byte** em 5 combinações × 9 cores.\n\
         \x20\n\
         \x20 FILEIRA 1:\n\
         \x20  1) HUE −90° — a matiz gira para um lado. O meio do slider é o zero, e ele é\n\
         \x20     byte-idêntico a não haver degrau nenhum (o artista atravessa o zero a arrastar).\n\
         \x20  2) HUE +90° — a mesma estrela para o outro lado. ⚠️ A rotação é RÍGIDA: o croma\n\
         \x20     não escorre. Medido no giro inteiro, razão croma-depois/antes **0,989..1,010**\n\
         \x20     — e é uma rotação em OKLab, não a matriz YIQ do SVG nem a matiz do HSL, porque\n\
         \x20     em HSL um pixel quase-cinza tem matiz mal definida e sai SALPICADO (o defeito\n\
         \x20     que o Enio já viu num fundo cinzento, e cuja cura foi o OKLab).\n\
         \x20  3) SATURATION −1 — a estrela sai CINZA (o croma vai a zero, medido < 0,01).\n\
         \x20  4) SATURATION +1 — o croma DOBRA. A cor fica mais viva sem mudar de matiz.\n\
         \x20\n\
         \x20 FILEIRA 2 (a ORDEM e os PONTOS FIXOS):\n\
         \x20  5) Adjust -> Glow: a estrela gira, e o halo AZUL entra depois, com a cor autorada.\n\
         \x20  6) Glow -> Adjust: o ajuste recebe a imagem JÁ com o halo, e gira OS DOIS — o halo\n\
         \x20     deixa de ser azul. As MESMAS duas operações, desenhos diferentes: é a tese da\n\
         \x20     pilha, e é o que prova que este degrau ajusta a IMAGEM que chegou.\n\
         \x20  7) Estrela CINZA + matiz e saturação FORTES: **não acontece nada**, e está certo —\n\
         \x20     girar a matiz de um pixel sem croma é girar nada. (Gate mede: ≤1 nível.)\n\
         \x20  8) A MESMA estrela cinza + BRILHO −0,45: escurece. O brilho é o único dos três que\n\
         \x20     move um acromático, e é por isso que os três são oferecidos juntos.\n\
         \x20\n\
         \x20 NO PAINEL (é o que fecha o smoke): selecione uma estrela, abra FILTERS, e o card\n\
         \x20 'Color Adjust' tem TRÊS knobs — **Hue** (em GRAUS, −180..180), **Saturation** e\n\
         \x20 **Brightness** (−1..1) —, todos com o zero no MEIO do curso e o sinal no readout.\n\
         \x20 Arraste cada um de ponta a ponta: ao passar pelo meio a forma tem de ficar\n\
         \x20 EXACTAMENTE como estava. Nas pontas do Brightness: preto puro e branco puro."
    );
}
