//! **A TURBULÊNCIA** (plano 24 W6b) — a cena `PH2D_BUILD_SMOKE=35`.
//!
//! Irmã da `fx_blend_smoke` (=34) e construída no mesmo molde: um **A/B**, não um catálogo. Quatro
//! pares, e em cada um a MESMA forma com o MESMO efeito — só um knob difere. Um campo de ruído
//! sozinho não diz nada ("é assim mesmo?"); dois lado a lado dizem o que o knob faz.
//!
//! ⚠️ **Toda estrela leva um CONTORNO por baixo da turbulência**, e não é decoração: a turbulência
//! deforma o que RECEBEU, então com o contorno na pilha antes dela é a LINHA que ondula — e uma
//! linha fina torna visível um deslocamento de poucos pixels que numa silhueta cheia passa
//! despercebido. É também a demonstração de que ela COMPÕE: o degrau anterior é a entrada dela.
//!
//! Rodar: `cd <worktree> && env PH2D_BUILD_SMOKE=35 cargo run -p ph2d-host-desktop --release`.

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

/// O contorno que dá uma LINHA para a turbulência deformar.
fn base_outline() -> FxOp {
    FxOp {
        radius: 0.04,
        color: [0.05, 0.05, 0.08, 1.0],
        opacity: 1.0,
        ..FxOp::new(FxOp::OUTLINE)
    }
}

/// Uma turbulência com os quatro knobs explícitos — a cena inteira é a variação deles.
fn turb(amount: f32, scale: f32, detail: u8, mode: u8) -> FxOp {
    FxOp {
        radius: amount,
        scale,
        detail,
        mode,
        // A MESMA semente em todas: um par que trocasse de desenho junto com o knob não isolaria
        // knob nenhum.
        seed: 3,
        opacity: 1.0,
        ..FxOp::new(FxOp::TURBULENCE)
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
            "[smoke] fx-turbulence: as oito estrelas ainda não existem — o `sync` não correu"
        );
        return;
    }
    let stacks: [(usize, Vec<FxOp>); STARS] = [
        // Par 1 — AMOUNT: o controle (zero = a forma intacta) contra a forma liquefeita.
        (
            0,
            vec![base_outline(), turb(0.0, 0.25, 3, FxOp::MODE_SMOOTH)],
        ),
        (
            1,
            vec![base_outline(), turb(0.25, 0.25, 3, FxOp::MODE_SMOOTH)],
        ),
        // Par 2 — SIZE: o mesmo deslocamento, ondulações miúdas × largas.
        (
            2,
            vec![base_outline(), turb(0.08, 0.12, 3, FxOp::MODE_SMOOTH)],
        ),
        (
            3,
            vec![base_outline(), turb(0.08, 0.50, 3, FxOp::MODE_SMOOTH)],
        ),
        // Par 3 — DETAIL: uma oitava (onda limpa) × seis (grão dentro da onda).
        (
            4,
            vec![base_outline(), turb(0.08, 0.30, 1, FxOp::MODE_SMOOTH)],
        ),
        (
            5,
            vec![base_outline(), turb(0.08, 0.30, 6, FxOp::MODE_SMOOTH)],
        ),
        // Par 4 — MODO: a soma com sinal × a soma dos módulos (os dois `type` do feTurbulence).
        (
            6,
            vec![base_outline(), turb(0.10, 0.30, 3, FxOp::MODE_SMOOTH)],
        ),
        (
            7,
            vec![base_outline(), turb(0.10, 0.30, 3, FxOp::MODE_CREASED)],
        ),
    ];
    let map = &app.vec_entities;
    let sim = &mut app.gfx.as_mut().expect("gfx").sim;
    for (i, ops) in stacks {
        crate::fx_live::set_filter(sim, map, &[ids[i]], Some(VecFilter { ops }));
    }
    eprintln!(
        "[smoke] A TURBULÊNCIA (plano 24 W6b) — o feTurbulence + feDisplacementMap do SVG num\n\
         \x20degrau só, que é como o AE (Turbulent Displace) o embrulha. Quatro PARES: em cada um\n\
         \x20a MESMA estrela com o MESMO contorno por baixo, e SÓ um knob difere.\n\
         \x20\n\
         \x20 Os números saem da sonda `measure_the_smoke_scene_pairs` (RTX), medidos ANTES desta\n\
         \x20 mensagem. Eles descrevem o CAMPO (o desvio de uma aresta reta ao zoom da cena), não\n\
         \x20 a estrela — o que a estrela mostra é o mesmo campo dobrado sobre o contorno dela.\n\
         \x20\n\
         \x20 FILEIRA 1:\n\
         \x20  1) AMOUNT 0 — o CONTROLE. Desvio **0,00 px**: a forma tem de sair intacta, e o\n\
         \x20     degrau existe na pilha. É o que prova que um knob zerado não é um efeito.\n\
         \x20  2) AMOUNT 0,25 — a mesma pilha, liquefeita: desvio **15,60 px**, 6 ondulações.\n\
         \x20  3) SIZE 0,12 — ondulações MIÚDAS: **8** delas na mesma altura (desvio 1,79 px).\n\
         \x20  4) SIZE 0,50 — o MESMO Amount, ondulações LARGAS: **3** (desvio 1,58 px). O knob\n\
         \x20     governa o TAMANHO, e quase não mexe em quão longe a tinta anda.\n\
         \x20\n\
         \x20 FILEIRA 2:\n\
         \x20  5) DETAIL 1 — uma oitava: onda limpa (rugosidade **0,444**).\n\
         \x20  6) DETAIL 6 — seis: grão fino DENTRO da mesma onda (rugosidade **1,048**, 2,4×).\n\
         \x20  7) SMOOTH — o `fractalNoise` do SVG: a soma COM SINAL (rugosidade 0,635).\n\
         \x20  8) CREASED — o `turbulence`: a soma dos MÓDULOS. Onde cada oitava cruza o zero\n\
         \x20     fica um VINCO — rugosidade **0,850** e **10** ondulações contra 4, com a mesma\n\
         \x20     semente e o mesmo tamanho. É a diferença entre nuvem e fumaça.\n\
         \x20\n\
         \x20 NO PAINEL (é o que fecha o smoke): selecione uma estrela, abra FILTERS, e o card da\n\
         \x20 turbulência tem **Amount · Size · Detail · Seed** mais os dois chips de Mode. Os\n\
         \x20 três knobs do ruído aparecem em BLOCO e SÓ neste tipo (um Size sem Detail descreve\n\
         \x20 metade de um campo). Arraste o **Seed**: o desenho troca e a estatística não —\n\
         \x20 é o botão de *me dá outro*.\n\
         \x20 ⚠️ E arraste o **Amount** de um degrau qualquer da pilha: o padrão da turbulência\n\
         \x20 NÃO pode andar por baixo da forma. A grade dele é ancorada na FORMA, não na\n\
         \x20 textura (`the_noise_is_pinned_to_the_shape_not_to_the_scratch`, desvio 0,0000 px)."
    );
}
