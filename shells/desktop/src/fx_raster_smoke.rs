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
    // ⚠️ **TRÊS estrelas levam TRAÇO armado, e cada uma responde a um report diferente.**
    //
    // A do Outline GROSSO (11): *"com Stroke quebra as pontas"* — a ponta do miter de um traço vai
    // a 3,24 × meia largura do vértice, e o bbox do scratch tem de a conter, senão ela sai CEIFADA.
    //
    // A do BEVEL (13): *"linhas no Bevel"* — uma forma traçada não tinha silhueta exata (a curva
    // autorada passa pelo MEIO da tinta), caía no campo semeado pelo RASTER, e a semente discreta
    // desenhava um pente de hachuras diagonais.
    //
    // A do FEATHER (12) leva traço de **LARGURA ZERO** — a segunda rodada do mesmo report:
    // *"para stroke maior que 0 funciona. Mas para stroke = 0 linhas aparecem"*. Largura zero é
    // **sem traço** (o slider promete isso), mas `stroke.is_some()` continua verdadeiro, então a
    // forma caía no raster sem sequer haver tinta de contorno. O feather lê a DISTÂNCIA do mesmo
    // campo, então o erro aparece nele como rampa ondulada em vez de pente.
    //
    // ⚠️ **Sem estas linhas a cena NÃO CONTÉM os fenómenos**: antes delas nenhuma estrela era
    // traçada *e* biselada, e nenhuma tinha traço de largura zero — os bugs reportados não podiam
    // aparecer em nenhuma das dezasseis. O bevel sem traço continua provado pelos gates e pela
    // sonda `fx_look_probe` (cenas 12/13), que é onde as fotos do antes/depois foram tiradas.
    for (i, rgb, w) in [
        (11usize, [40, 70, 220], 0.06),
        (13, [255, 255, 255], 0.06),
        (12, [255, 255, 255], 0.0),
    ] {
        if let Some(gfx) = app.gfx.as_mut()
            && let Some(path) = gfx.vec_scene.path_mut(ids[i])
        {
            path.stroke = Some(ph2d_vec_scene::StrokeSpec::new(
                ph2d_vec_scene::Rgba8::new(rgb[0], rgb[1], rgb[2], 255),
                w,
            ));
        }
    }
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
         \x20  11) A PILHA INTEIRA (Shadow -> Blur -> Glow) · 12) OUTLINE GROSSO **com TRAÇO azul**\n\
         \x20      — olhe as PONTAS: a ponta do miter de um traço vai a 3,24x meia largura do\n\
         \x20      vértice, e o bbox do scratch tem de a conter (antes ela saía CEIFADA).\n\
         \x20     O contorno agora é uma DILATAÇÃO sobre um campo de distância: a ponta RECEBE\n\
         \x20     contorno (antes recebia 0,0 px numa quina de 36 graus) e a largura é a mesma na\n\
         \x20     ponta e na aresta. A quina é REDONDA por DERIVAÇÃO — um miter pediria 3,24x a\n\
         \x20     largura numa ponta de estrela, e nenhuma dilatação faz isso (seria 3,24x na\n\
         \x20     aresta também). Miter/bevel sao GEOMETRIA: a pilha de Effects, não esta.\n\
         \x20 FILEIRA 4 (os dois que o campo de distância destravou, e o par de ordem):\n\
         \x20  13) FEATHER **com traço de LARGURA ZERO** — a borda vira uma RAMPA CENTRADA na\n\
         \x20      fronteira, e o MIOLO fica INTACTO. É o que um Blur não faz: ele mistura a COR\n\
         \x20      também (medido, com listras dentro da forma: contraste 195 no feather contra 1\n\
         \x20      no borrão). ⚠️ **O traço de largura zero é o caso da 2a rodada do report**:\n\
         \x20      zero significa SEM traço, mas 'stroke.is_some()' continua verdadeiro, entao a\n\
         \x20      forma caia no campo do RASTER sem sequer haver tinta de contorno. A rampa tem\n\
         \x20      de sair LISA, e a estrela nao pode ter contorno visivel nenhum.\n\
         \x20  14) BEVEL **com TRAÇO branco** — a face virada para a LUZ clareia e a oposta\n\
         \x20      escurece, morrendo para o miolo (medido em cinza: rim 225 / 30 contra miolo\n\
         \x20      128; trocar a luz troca os dois). O par Light X/Y é uma DIREÇÃO, não um\n\
         \x20      deslocamento. ⚠️ **O traço é o caso do report 'linhas no Bevel'**: uma forma\n\
         \x20      traçada não tinha silhueta exata e caía no campo do RASTER, cuja semente\n\
         \x20      discreta desenhava um PENTE de hachuras diagonais finas. Agora a silhueta é\n\
         \x20      'preenchimento uniao contorno-do-traco', resolvida pela booleana — o relevo\n\
         \x20      tem de sair LISO, sem hachura, e as pontas com contorno inteiro.\n\
         \x20  15) Glow -> Blur · 16) Blur -> Glow — os MESMOS dois degraus, trocados.\n\
         \x20\n\
         \x20 O rim claro de 1 px do Inner Shadow MORREU (um degrau de dentro não move mais um\n\
         \x20 texel de cobertura) e opacidade 0 é no-op em TODO tipo (o Blur apagava a forma).\n\
         \x20 No painel, o card dos dois de dentro tem o chip Mode: Proximity | Contour."
    );
}
