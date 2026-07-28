//! **A LEI DE MISTURA por degrau** (plano 24 W6) — a cena `PH2D_BUILD_SMOKE=34`.
//!
//! Irmã da `fx_raster_smoke` (=33) e separada dela de propósito: aquela é o CATÁLOGO (um tipo por
//! estrela, dezasseis slots cheios), e esta é um **A/B** — o mesmo degrau, a mesma cor, a mesma
//! opacidade, e só a LEI diferente. É a única composição em que a pergunta *"o que a lei faz?"* tem
//! resposta visível: um número sozinho não diz nada, dois lado a lado dizem tudo.
//!
//! ⚠️ **A fonte tem de ter ESTRUTURA para a lei ter o que preservar.** Uma estrela de cor chapada
//! sob `Color` sai igual a uma sob `Normal` na METADE dos casos — é o sombreado que a lei respeita
//! ou destrói. Por isso cada estrela leva um **Bevel de base** que lhe dá relevo, e os pares são
//! desenhados POR CIMA dele.
//!
//! Rodar: `cd <worktree> && env PH2D_BUILD_SMOKE=34 cargo run -p ph2d-host-desktop --release`.

use ph2d_ecs::{FxOp, VecFilter};
use ph2d_vec_scene::{ShapeKind, VecPathId};

use crate::build_smoke::shape;

/// Os pontos de estrela (5 pontas, raio interno 0.45) — os mesmos da cena irmã.
const STAR_V: &[f64] = &[5.0, 0.45, 0.0];

const COLS: [f64; 4] = [-3.6, -1.95, -0.3, 1.35];
const ROWS: [f64; 2] = [1.2, -1.0];
const SIDE: f64 = 1.35;
/// Quantas estrelas a cena monta: quatro pares.
const STARS: usize = 8;

/// Os códigos de `ph2d_painter_effects::BlendMode` que a cena exercita. Escritos como CONSTANTES
/// nomeadas e não inline: um número solto num smoke é a coisa que ninguém confere.
const NORMAL: u8 = 0;
const MULTIPLY: u8 = 1;
const OVERLAY_LAW: u8 = 9;
const COLOR_LAW: u8 = 18;

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
            // Um âmbar médio: escuro o bastante para o Screen ter para onde subir, claro o bastante
            // para o Multiply ter para onde descer. Uma cor nas pontas da faixa esconderia metade
            // das leis.
            [190, 140, 70],
        ));
    }
}

/// A base que dá ESTRUTURA à estrela — sem ela `Color` e `Normal` desenham o mesmo, e o par
/// central da cena não discriminaria nada.
fn base_bevel() -> FxOp {
    FxOp {
        radius: 0.16,
        offset: [-0.1, 0.1],
        color: [0.0, 0.0, 0.0, 1.0],
        opacity: 1.0,
        ..FxOp::new(FxOp::BEVEL)
    }
}

/// Um Color Overlay de cor `color` sob a lei `blend`.
fn overlay(color: [f32; 4], blend: u8) -> FxOp {
    FxOp {
        color,
        blend,
        ..FxOp::new(FxOp::COLOR_OVERLAY)
    }
}

/// Um Inner Shadow (modo Contour, o default) sob a lei `blend`.
///
/// ⚠️ **A cor é uma lavanda CLARA, e não o roxo escuro que esta cena teve primeiro.** Medido na
/// sonda: com o roxo escuro as duas leis saíam a 115,4 e 113,1 na borda — 2,3 níveis, invisível,
/// porque uma cor já escura multiplicada dá quase a mesma coisa que uma interpolação até ela. Com
/// a lavanda o par separa: Normal leva a borda a **166,0** (mais CLARA que a base, 148,2 — uma
/// "sombra" que ilumina) e Multiply a **130,0**. A fixture tem de conter o fenômeno.
fn inner(blend: u8) -> FxOp {
    FxOp {
        radius: 0.16,
        color: [0.75, 0.60, 0.95, 1.0],
        opacity: 1.0,
        blend,
        ..FxOp::new(FxOp::INNER_SHADOW)
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
        eprintln!("[smoke] fx-blend: as oito estrelas ainda não existem — o `sync` não correu");
        return;
    }
    const CYAN: [f32; 4] = [0.1, 0.9, 1.0, 1.0];
    const WHITE: [f32; 4] = [1.0, 1.0, 1.0, 1.0];

    // Quatro PARES: em cada um, a MESMA cor e a MESMA opacidade, só a lei muda.
    let stacks: [(usize, Vec<FxOp>); STARS] = [
        // Par 1 — Color Overlay ciano: repinta chapado × mantém o relevo.
        (0, vec![base_bevel(), overlay(CYAN, NORMAL)]),
        (1, vec![base_bevel(), overlay(CYAN, MULTIPLY)]),
        // Par 2 — Color Overlay ciano: repinta chapado × TROCA A MATIZ preservando a luz.
        (2, vec![base_bevel(), overlay(CYAN, NORMAL)]),
        (3, vec![base_bevel(), overlay(CYAN, COLOR_LAW)]),
        // Par 3 — Inner Shadow roxa: lava para o roxo × ESCURECE (o default do Photoshop).
        (4, vec![base_bevel(), inner(NORMAL)]),
        (5, vec![base_bevel(), inner(MULTIPLY)]),
        // Par 4 — Color Overlay branco: apaga tudo × vira MATERIAL (contraste sobre o relevo).
        (6, vec![base_bevel(), overlay(WHITE, NORMAL)]),
        (7, vec![base_bevel(), overlay(WHITE, OVERLAY_LAW)]),
    ];
    let map = &app.vec_entities;
    let sim = &mut app.gfx.as_mut().expect("gfx").sim;
    for (i, ops) in stacks {
        crate::fx_live::set_filter(sim, map, &[ids[i]], Some(VecFilter { ops }));
    }
    eprintln!(
        "[smoke] A LEI DE MISTURA por degrau (plano 24 W6). Quatro PARES — em cada um a MESMA\n\
         \x20cor e a MESMA opacidade, e SÓ a lei muda. Toda estrela leva um BEVEL de base, porque\n\
         \x20uma cor chapada não dá à lei nada para preservar. Os números abaixo saem da sonda\n\
         \x20`measure_the_smoke_scene_pairs` (RTX), medidos ANTES de esta mensagem os afirmar.\n\
         \x20\n\
         \x20 FILEIRA 1 (Color Overlay, o recolorizador):\n\
         \x20  1) NORMAL ciano — recorte CHAPADO: o relevo do bevel some por baixo da tinta.\n\
         \x20     (desvio da luminância no miolo: **0,00** — não sobra estrutura nenhuma.)\n\
         \x20  2) MULTIPLY ciano — a MESMA tinta, e o relevo ATRAVESSA: claro fica claro, escuro\n\
         \x20     fica escuro. (desvio **7,96** contra 11,60 da base — é tingir, não repintar.)\n\
         \x20  3) NORMAL ciano — o controle do par 2, igual ao 1.\n\
         \x20  4) COLOR ciano — a MATIZ é a do ciano e a LUMINOSIDADE é a da estrela: a luma\n\
         \x20     LINEAR da base é **0,2247** e sob Color fica **0,2248**. É o tint/duotone, que\n\
         \x20     a fila do plano 24 listava como item À PARTE — sai daqui sem um décimo tipo.\n\
         \x20\n\
         \x20 FILEIRA 2:\n\
         \x20  5) INNER SHADOW lavanda em NORMAL — a borda vai a **166,0**, mais CLARA que a\n\
         \x20     base (**148,2**). Uma sombra que ILUMINA: a lei antiga interpola até a cor do\n\
         \x20     efeito, então uma cor clara clareia. É o defeito que o default do Photoshop\n\
         \x20     (Multiply) existe para evitar.\n\
         \x20  6) INNER SHADOW lavanda em MULTIPLY — a MESMA cor, e a borda vai a **130,0**:\n\
         \x20     abaixo da base. Uma sombra que escurece.\n\
         \x20  7) COLOR OVERLAY branco em NORMAL — a estrela some (luma **255,0**, desvio 0,00).\n\
         \x20  8) COLOR OVERLAY branco em OVERLAY — vira MATERIAL: o desvio sobe a **15,45**,\n\
         \x20     ACIMA dos 11,60 da base. O branco PUXA o contraste do relevo em vez de o\n\
         \x20     apagar — é o único dos oito que devolve mais estrutura do que recebeu.\n\
         \x20\n\
         \x20 NO PAINEL (é o que fecha o smoke): selecione uma estrela, abra FILTERS, e o card do\n\
         \x20 degrau tem a fileira **Blend** logo abaixo de Color. São VINTE leis, e o chip abre\n\
         \x20 a lista. ⚠️ Ela aparece SÓ em quatro tipos — Inner Shadow, Inner Glow, Bevel e\n\
         \x20 Color Overlay —, os que pousam cor sobre conteúdo que já existe. Num Glow ou numa\n\
         \x20 Drop Shadow o halo entra POR BAIXO, e ali não há com que misturar: o alcance da lei\n\
         \x20 seria uma orla de 1 px (pico 0,25 na rampa de anti-aliasing, medido em\n\
         \x20 `the_blend_of_an_outer_halo_only_reaches_the_antialiased_fringe`)."
    );
}
