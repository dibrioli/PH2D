//! **A cena pronta para o smoke da PILHA de FX RASTER** — `PH2D_BUILD_SMOKE=33`.
//!
//! Módulo irmão do [`crate::build_smoke`] pelo teto de LOC, como o `falloff_smoke`/`contour_smoke`.
//!
//! O FX raster (plano 24) NÃO é um deformador vetorial: ele isola a forma na própria textura, roda
//! a pilha na GPU e recompõe no z dela — pixels, não geometria. **Dezasseis estrelas em quatro
//! fileiras:**
//!
//! 1. **A regressão** (W1/W2): controle nítido · Blur · Glow · Drop Shadow.
//! 2. **Os degraus de DENTRO, e a comparação da W4:** o MESMO Inner Shadow em `Proximity` e em
//!    `Contour` — o 1º mede *quanto de fora há por perto* (as pontas escurecem inteiras, as
//!    reentrâncias quase não), o 2º mede a *distância à borda* (banda de largura constante em toda
//!    a volta). Mais Inner Glow e Color Overlay.
//! 3. **O CONTORNO e a composição:** Outline fino · o STICKER (`Outline → Drop Shadow`) · a PILHA
//!    INTEIRA · e um Outline GROSSO, onde as pontas mostram que ele é uma dilatação de verdade.
//! 4. **Os dois que o campo de distância destravou** — `Feather` (a borda amacia, o miolo fica
//!    intacto: o que um Blur não faz) e `Bevel` (o rebordo ganha luz) — mais **o par de ordem**
//!    (`Glow → Blur` × `Blur → Glow`): os MESMOS dois degraus, trocados. Se a ordem não importasse,
//!    as duas seriam idênticas — e o ponto da W2 é que não são.
//!
//! ⚠️ **Nada aqui prova o SEAM do painel** — a pilha é armada no componente direto (a seção
//! *Filters* do painel é a prova do gesto, coberta pelo seam gate). Esta cena existe para o olho
//! julgar o DESENHO do produtor GPU, como a `line/physics` faz com as cenas de referência.

use ph2d_ecs::{FxOp, VecFilter};
use ph2d_vec_scene::{ShapeKind, VecPathId};

use crate::build_smoke::shape;

/// Os pontos de estrela (5 pontas, raio interno 0.45), reusados em todas.
const STAR_V: &[f64] = &[5.0, 0.45, 0.0];

/// As quatro colunas e as quatro fileiras (canto inferior-esquerdo de cada estrela).
const COLS: [f64; 4] = [-3.6, -1.95, -0.3, 1.35];
const ROWS: [f64; 4] = [2.0, 0.55, -0.9, -2.35];
const SIDE: f64 = 1.25;
/// Quantas estrelas a cena monta.
const STARS: usize = 16;

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
    for i in 0..STARS {
        let (x, y) = (COLS[i % COLS.len()], ROWS[i / COLS.len()]);
        scene.push_path(shape(
            ShapeKind::Star,
            [x, y],
            [x + SIDE, y + SIDE],
            STAR_V,
            [235, 175, 60],
        ));
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
    if ids.len() < STARS {
        eprintln!(
            "[smoke] fx-raster: as dezasseis estrelas ainda não existem — o `sync` não correu"
        );
        return;
    }
    let blur = op(FxOp::BLUR, 0.12, BLACK, 1.0);
    let glow = op(FxOp::GLOW, 0.18, CYAN, 1.0);
    let shadow = FxOp {
        offset: [0.18, -0.18],
        ..op(FxOp::DROP_SHADOW, 0.1, BLACK, 0.6)
    };
    // O MESMO Inner Shadow nos dois modos — é a comparação inteira da wave.
    let inner_prox = FxOp {
        offset: [0.1, -0.1],
        mode: FxOp::MODE_PROXIMITY,
        ..op(FxOp::INNER_SHADOW, 0.12, BLACK, 0.9)
    };
    let inner_cont = FxOp {
        mode: FxOp::MODE_CONTOUR,
        ..inner_prox
    };
    let inner_glow = op(FxOp::INNER_GLOW, 0.12, CYAN, 1.0);
    let outline = op(FxOp::OUTLINE, 0.07, WHITE, 1.0);
    let overlay = op(FxOp::COLOR_OVERLAY, 0.0, SKY, 1.0);
    // `[0]` é o controle (sem pilha). Depois: a regressão, os de dentro, o contorno, a composição.
    let feather = op(FxOp::FEATHER, 0.14, WHITE, 1.0);
    let bevel = FxOp {
        offset: [-0.1, 0.1],
        ..op(FxOp::BEVEL, 0.12, BLACK, 0.9)
    };
    let stacks: [(usize, Vec<FxOp>); 15] = [
        (1, vec![blur]),
        (2, vec![glow]),
        (3, vec![shadow]),
        (4, vec![inner_prox]),
        (5, vec![inner_cont]),
        (6, vec![inner_glow]),
        (7, vec![overlay]),
        (8, vec![outline]),
        (9, vec![outline, shadow]),
        (10, vec![shadow, blur, glow]),
        (11, vec![op(FxOp::OUTLINE, 0.14, WHITE, 1.0)]),
        (12, vec![feather]),
        (13, vec![bevel]),
        (14, vec![glow, blur]),
        (15, vec![blur, glow]),
    ];
    let map = &app.vec_entities;
    let sim = &mut app.gfx.as_mut().expect("gfx").sim;
    // O arm passa pela porta única `set_filter` (a mesma que o bridge do painel usa).
    for (i, ops) in stacks {
        crate::fx_live::set_filter(sim, map, &[ids[i]], Some(VecFilter { ops }));
    }
    eprintln!(
        "[smoke] A PILHA DE FX RASTER — 100% na GPU (plano 24, W3+W4). Dezasseis estrelas:\n\
         \x20 FILEIRA 1 (a regressão): controle nítido · BLUR · GLOW ciano · DROP SHADOW.\n\
         \x20 FILEIRA 2 (os degraus de DENTRO — a comparação da wave):\n\
         \x20  5) INNER SHADOW modo PROXIMITY — o modelo do Photoshop: mede quanto de FORA há por\n\
         \x20     perto, entao as PONTAS escurecem inteiras e as REENTRÂNCIAS quase não escurecem\n\
         \x20     (medido numa cruz: 219 na reentrância contra 155 na aresta reta).\n\
         \x20  6) INNER SHADOW modo CONTOUR — a MESMA sombra pela DISTÂNCIA à borda: banda de\n\
         \x20     largura constante em toda a volta, reentrâncias incluídas (115 contra 104). É o\n\
         \x20     modo DEFAULT, e é a resposta ao 'não projeta sombra nas reentrâncias'.\n\
         \x20  7) INNER GLOW (Contour) · 8) COLOR OVERLAY — a estrela repintada, sem borrar.\n\
         \x20 FILEIRA 3: 9) OUTLINE fino · 10) O STICKER (Outline -> Drop Shadow) ·\n\
         \x20  11) A PILHA INTEIRA (Shadow -> Blur -> Glow) · 12) OUTLINE GROSSO — olhe as PONTAS.\n\
         \x20     O contorno agora é uma DILATAÇÃO sobre um campo de distância: a ponta RECEBE\n\
         \x20     contorno (antes recebia 0,0 px numa quina de 36 graus) e a largura é a mesma na\n\
         \x20     ponta e na aresta. A quina é REDONDA por DERIVAÇÃO — um miter pediria 3,24x a\n\
         \x20     largura numa ponta de estrela, e nenhuma dilatação faz isso (seria 3,24x na\n\
         \x20     aresta também). Miter/bevel sao GEOMETRIA: a pilha de Effects, não esta.\n\
         \x20 FILEIRA 4 (os dois que o campo de distância destravou, e o par de ordem):\n\
         \x20  13) FEATHER — a borda vira uma RAMPA CENTRADA na fronteira, e o MIOLO fica\n\
         \x20      INTACTO. É o que um Blur não faz: ele mistura a COR também (medido, com\n\
         \x20      listras dentro da forma: contraste 195 no feather contra 1 no borrão).\n\
         \x20  14) BEVEL — a face virada para a LUZ clareia e a oposta escurece, morrendo para o\n\
         \x20      miolo (medido em cinza: rim 225 / 30 contra miolo 128; trocar a luz troca os\n\
         \x20      dois). O par Light X/Y é uma DIREÇÃO, não um deslocamento — e é por isso que a\n\
         \x20      tabela ROTULA cada knob em vez de só dizer que ele existe.\n\
         \x20  15) Glow -> Blur · 16) Blur -> Glow — os MESMOS dois degraus, trocados.\n\
         \x20\n\
         \x20 O rim claro de 1 px do Inner Shadow MORREU (um degrau de dentro não move mais um\n\
         \x20 texel de cobertura) e opacidade 0 é no-op em TODO tipo (o Blur apagava a forma).\n\
         \x20 No painel, o card dos dois de dentro tem o chip Mode: Proximity | Contour."
    );
}
