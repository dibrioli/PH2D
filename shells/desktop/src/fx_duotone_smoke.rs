//! **DUOTONE + LUMA TO ALPHA** (plano 24 W9) — a cena `PH2D_BUILD_SMOKE=38`.
//!
//! Irmã da `fx_adjust_smoke` (=37), da `fx_morphology_smoke` (=36) e das outras, no mesmo molde: um
//! **A/B**, não um catálogo.
//!
//! ⚠️ **A arte destas duas leis tem de ter VARIAÇÃO de brilho, e isso é a lei, não a cena:** as duas
//! perguntam *quão claro é este texel* — uma manda a resposta para a COR, a outra para a COBERTURA.
//! Sobre uma chapa de cor sólida a resposta é a mesma em todo lado, e o Luma to Alpha sobre branco
//! puro é literalmente a identidade. Por isso cada forma leva um **Bevel** antes: ele é o degrau
//! que esta pilha já tem para esculpir claro-e-escuro dentro de uma silhueta chapada.
//!
//! Rodar: `cd <worktree> && env PH2D_BUILD_SMOKE=38 cargo run -p ph2d-host-desktop --release`.

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
/// A cor das estrelas — um cinza claro, para o relevo do Bevel se ler sem competir com a paleta que
/// o Duotone vai impor.
const STONE: [u8; 3] = [200, 200, 200];

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
            STONE,
        ));
    }
}

/// O degrau que dá VARIAÇÃO de brilho à silhueta chapada — ver o cabeçalho.
fn bevel() -> FxOp {
    FxOp {
        radius: 0.12,
        opacity: 1.0,
        ..FxOp::new(FxOp::BEVEL)
    }
}

/// Uma rampa autorada. As cores chegam em `[0,1]` retos, como a swatch as escreve.
fn duotone(shadow: [f32; 4], highlight: [f32; 4]) -> FxOp {
    FxOp {
        color: shadow,
        color_b: highlight,
        opacity: 1.0,
        ..FxOp::new(FxOp::DUOTONE)
    }
}

fn luma_to_alpha() -> FxOp {
    FxOp {
        opacity: 1.0,
        ..FxOp::new(FxOp::LUMA_TO_ALPHA)
    }
}

/// Um brilho externo, para o par que mostra que estes degraus agem sobre a IMAGEM que chegou.
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
        eprintln!("[smoke] fx-duotone: as oito estrelas ainda não existem — o `sync` não correu");
        return;
    }
    // O par frio→quente do default do "Add", e um segundo par para se ver que a paleta é AUTORADA.
    let cool = [0.10, 0.12, 0.35, 1.0];
    let warm = [1.0, 0.86, 0.62, 1.0];
    let ink = [0.05, 0.20, 0.18, 1.0];
    let lime = [0.75, 1.0, 0.35, 1.0];

    let stacks: [(usize, Vec<FxOp>); STARS] = [
        // Par 1 — O DUOTONE contra o CONTROLE: a mesma estrela esculpida, com e sem a rampa.
        (0, vec![bevel()]),
        (1, vec![bevel(), duotone(cool, warm)]),
        // Par 2 — A PALETA é autorada: a mesma pilha, outras duas pontas.
        (2, vec![bevel(), duotone(ink, lime)]),
        // …e a MESMA rampa invertida (as duas swatches trocadas), que é o negativo dela.
        (3, vec![bevel(), duotone(warm, cool)]),
        // Par 3 — O DUOTONE contra o COLOR OVERLAY, que é a objeção óbvia: um preserva a
        // modelagem, o outro a achata.
        (
            4,
            vec![
                bevel(),
                FxOp {
                    color: warm,
                    opacity: 1.0,
                    ..FxOp::new(FxOp::COLOR_OVERLAY)
                },
            ],
        ),
        (5, vec![bevel(), duotone(cool, warm)]),
        // Par 4 — LUMA TO ALPHA: o brilho vira cobertura. Com o halo AZUL por baixo, o que
        // aparece pelos buracos é o glow — é assim que se vê que a cobertura de facto saiu.
        (6, vec![glow(), bevel()]),
        (7, vec![glow(), bevel(), luma_to_alpha()]),
    ];
    let map = &app.vec_entities;
    let sim = &mut app.gfx.as_mut().expect("gfx").sim;
    for (i, ops) in stacks {
        crate::fx_live::set_filter(sim, map, &[ids[i]], Some(VecFilter { ops }));
    }
    eprintln!(
        "[smoke] DUOTONE + LUMA TO ALPHA (plano 24 W9) — as duas leis que perguntam *quao claro e\n\
         \x20este texel*: uma manda a resposta para a COR (a rampa de duas pontas), a outra para a\n\
         \x20COBERTURA. Toda estrela leva um BEVEL antes, porque uma chapa de cor solida nao tem\n\
         \x20variacao de brilho — e sem variacao as duas leis nao tem o que ler.\n\
         \x20\n\
         \x20 FILEIRA 1:\n\
         \x20  1) CONTROLE — so o Bevel: a estrela esculpida, na cor de pedra.\n\
         \x20  2) + DUOTONE (sombra fria -> luz quente): a MESMA modelagem, outra paleta. O volume\n\
         \x20     sobrevive; e essa a diferenca inteira.\n\
         \x20  3) Outra paleta (tinta escura -> lima): as duas pontas sao AUTORADAS, duas swatches.\n\
         \x20  4) A MESMA rampa INVERTIDA (as duas swatches trocadas) — o negativo dela.\n\
         \x20\n\
         \x20 FILEIRA 2:\n\
         \x20  5) COLOR OVERLAY na cor quente: a estrela vira uma CHAPA. O volume some.\n\
         \x20  6) DUOTONE com a mesma cor na ponta clara: o volume FICA. Lado a lado com a 5, e a\n\
         \x20     resposta a objecao obvia (*ja temos o Color Overlay*). Medido pelo gate: excursao\n\
         \x20     de 188 niveis no verde contra 0.\n\
         \x20  7) CONTROLE do ultimo par — halo azul + bevel, sem o ultimo degrau.\n\
         \x20  8) + LUMA TO ALPHA: o que era escuro fica TRANSPARENTE e o halo azul aparece por\n\
         \x20     tras. E o `luminanceToAlpha` do SVG, com uma divergencia deliberada: a nossa lei\n\
         \x20     ESCALA o alfa (`A' = A x luma`) em vez de o substituir, o que preserva a borda\n\
         \x20     anti-aliased. Medido: sob a lei literal do SVG um texel com 4/255 de cobertura\n\
         \x20     salta para 180/255 — a orla vira um degrau.\n\
         \x20\n\
         \x20 NO PAINEL (e o que fecha o smoke): selecione a estrela 2 e abra FILTERS. O card\n\
         \x20 'Duotone' tem DUAS swatches — **Shadows** e **Highlights** —, e cada uma abre o picker\n\
         \x20 OKLCH POR CONTA PROPRIA: clique a de cima, escolha uma cor, e ela tem de pousar na\n\
         \x20 ponta ESCURA; depois a de baixo, e na CLARA. Confira tambem o card 'Luma to Alpha' da\n\
         \x20 estrela 8: ele tem **Opacity e mais nada**, de proposito."
    );
}
