//! **A cena pronta para o smoke do TEXTURE PATTERN** — `PH2D_BUILD_SMOKE=76` (plano 33).
//!
//! Módulo irmão do [`crate::build_smoke`] pelo teto de LOC, como o `knot_smoke`/`twist_smoke`.
//!
//! ⭐ **A arte é SINTETIZADA aqui**, e é de propósito: o smoke não pode pedir um ficheiro ao Enio.
//! Ele corre com um comando só, e a arte é assimétrica nos DOIS eixos (uma barra em cima e uma
//! meia-diagonal), porque um motivo simétrico não deixa ver desfasamento, espelho nem rotação.
//! ⚠️ E ela tem um quadrante **transparente**: um padrão só-opaco esconde a lei do alfa, que é onde
//! a família do Bug #4 do Motion vive.
//!
//! As seis formas da fileira de cima, da esquerda para a direita (e uma **sétima** em baixo):
//!
//! 1. **Grade** (o controlo, e o HERÓI já selecionado) — a repetição simples.
//! 2. **Tijolo 1/2** — as linhas desfasam-se meia célula. O ladrilho assado tem **duas** linhas.
//! 3. **Colmeia** — o mesmo desfasamento, mas com o espaçamento vertical `√3/2` que põe os seis
//!    vizinhos à mesma distância.
//! 4. **Espelho** — a cada repetição a arte inverte; a costura desaparece mesmo em arte não
//!    periódica.
//! 5. **Buraco** (composto, regra `EvenOdd`) — ⚠️ o padrão **não pode** pintar o buraco. Foi a
//!    pedra em que o `fill_multipoint` tropeçou, e o `VectorScene::fill_path` ainda tem o defeito.
//! 6. **Esticada** — a MESMA grade numa forma escalada só num eixo: o padrão **esmaga com ela**, ao
//!    contrário da caneta do traço (bug #27). As duas leis estão certas e são diferentes.
//! 7. ⭐⭐ **Em baixo: a arte é uma FORMA do documento** (W7) — o triângulo ao lado dela. Mexer nos
//!    nós do triângulo re-assa o ladrilho **na hora**, que é o *"pattern fills are dynamic"* do
//!    Figma. ⚠️ O motivo fica **visível de propósito**: escondê-lo é o gesto do olho na Hierarquia,
//!    e uma fonte invisível por omissão seria uma forma que o artista não sabe que tem.

use ph2d_vec_pattern::{PatternMode, TileKind};
use ph2d_vec_scene::{
    FillRule, Paint, PatternFill, PatternSource, Rgba8, VecPath, VecPathId, VecVertex,
};

/// O lado da arte, em pixels.
const ART: u32 = 32;
/// O lado de cada forma, em unidades de mundo.
const BOX: f64 = 2.2;
/// O passo entre formas.
const STEP: f64 = 2.6;

/// A arte de referência: barra em cima, meia-diagonal, um quadrante transparente.
///
/// ⚠️ **Assimétrica nos dois eixos** — um motivo simétrico esconde desfasamento, espelho e rotação,
/// que são metade do que esta cena existe para mostrar.
fn art_rgba() -> Vec<u8> {
    let mut px = Vec::with_capacity((ART * ART * 4) as usize);
    for y in 0..ART {
        for x in 0..ART {
            let c = if y < ART / 8 {
                // A barra do topo: laranja opaco. É ela que denuncia uma rotação ou um espelho.
                [230u8, 140, 60, 255]
            } else if x + y < ART {
                // A meia-diagonal: azul opaco.
                [70, 120, 210, 255]
            } else if x > ART * 3 / 4 && y > ART * 3 / 4 {
                // ⚠️ O quadrante TRANSPARENTE — e com cor por baixo, que é o que todo PNG comum
                // tem. Um assador que componha `0/0` apaga este RGB e a grade deixa de ser
                // byte-idêntica sem que nenhum gate opaco dê por isso.
                [200, 40, 40, 0]
            } else {
                [235, 232, 225, 255]
            };
            px.extend_from_slice(&c);
        }
    }
    px
}

fn rect(cx: f64, cy: f64, half: f64) -> Vec<VecVertex> {
    [
        [cx - half, cy - half],
        [cx + half, cy - half],
        [cx + half, cy + half],
        [cx - half, cy + half],
    ]
    .map(VecVertex::corner)
    .to_vec()
}

fn pattern(source: PatternSource, kind: TileKind, mode: PatternMode, fallback: [u8; 3]) -> Paint {
    let mut f = PatternFill::new(
        source,
        [BOX / 3.0, BOX / 3.0],
        Rgba8::new(fallback[0], fallback[1], fallback[2], 255),
    );
    f.kind = kind;
    f.offset_denom = 2;
    f.mode = mode;
    Paint::Pattern(Box::new(f))
}

pub(crate) fn frame(app: &mut crate::App, f: u32) {
    match f {
        3 => build(app),
        4 => select_hero(app),
        _ => {}
    }
}

fn build(app: &mut crate::App) {
    let Some(gfx) = app.gfx.as_mut() else {
        return;
    };
    let _ = gfx.tools.set_active(&ph2d_editor::ToolId::new("vector"));
    // A arte entra pelo MESMO endereçamento que a autoria usa (`insert_image_rgba8`), senão o smoke
    // provaria um caminho que o produto não tem.
    let source = PatternSource::Image(gfx.asset_db.insert_image_rgba8(ART, ART, art_rgba()));
    let scene = &mut gfx.vec_scene;
    let half = BOX * 0.5;
    let x = |i: usize| -2.5 * STEP + (i as f64) * STEP;

    // 1..4 — as leis de reticulado e de repetição.
    for (i, (kind, mode, fb)) in [
        (TileKind::Grid, PatternMode::Tile, [90, 90, 110]),
        (TileKind::BrickRow, PatternMode::Tile, [110, 90, 90]),
        (TileKind::Hex, PatternMode::Tile, [90, 110, 90]),
        (TileKind::Grid, PatternMode::Mirror, [110, 110, 80]),
    ]
    .into_iter()
    .enumerate()
    {
        scene.push_path(VecPath {
            verts: rect(x(i), 0.0, half),
            closed: true,
            fill: Some(pattern(source, kind, mode, fb)),
            ..VecPath::default()
        });
    }

    // 5 — o COMPOSTO com buraco, regra `EvenOdd`. O contorno de dentro tem de ficar VAZIO.
    let hole = ph2d_vec_scene::Contour {
        verts: rect(x(4), 0.0, half * 0.45),
        closed: true,
    };
    scene.push_path(VecPath {
        verts: rect(x(4), 0.0, half),
        closed: true,
        subpaths: vec![hole],
        fill_rule: FillRule::EvenOdd,
        fill: Some(pattern(
            source,
            TileKind::Grid,
            PatternMode::Tile,
            [80, 100, 120],
        )),
        ..VecPath::default()
    });

    // ⭐⭐ 7 — a ARTE é uma FORMA DO DOCUMENTO (W7, o modelo do Figma). O motivo fica ao lado,
    // visível e editável: mexer nos nós dele re-assa o ladrilho na hora.
    let motivo = scene.push_path(VecPath {
        verts: [[x(5) - 0.4, -3.5], [x(5) + 0.4, -3.5], [x(5), -2.6]]
            .map(VecVertex::corner)
            .to_vec(),
        closed: true,
        fill: Some(Paint::Solid(Rgba8::new(90, 190, 220, 255))),
        ..VecPath::default()
    });
    scene.push_path(VecPath {
        verts: rect(x(4), -3.0, half),
        closed: true,
        fill: Some(pattern(
            PatternSource::Shape(motivo),
            TileKind::BrickRow,
            PatternMode::Tile,
            [70, 90, 110],
        )),
        ..VecPath::default()
    });

    // 6 — a mesma grade numa forma ESTICADA só em x. O padrão esmaga COM ela.
    let mut wide = VecPath {
        verts: rect(x(5) + half, 0.0, half),
        closed: true,
        fill: Some(pattern(
            source,
            TileKind::Grid,
            PatternMode::Tile,
            [120, 80, 110],
        )),
        ..VecPath::default()
    };
    wide.id = VecPathId::default();
    let id = scene.push_path(wide);
    scene.scale_path(id, 1.9, 0.7, [x(5) + half, 0.0]);
}

/// Seleciona a PRIMEIRA forma — o painel abre com o chip **Pattern** aceso.
fn select_hero(app: &mut crate::App) {
    let first: Option<VecPathId> = app
        .gfx
        .as_ref()
        .and_then(|g| g.vec_scene.paths().first().map(|p| p.id));
    if let Some(id) = first {
        app.vec_pen.select_many(&[id]);
    }
    eprintln!(
        "[smoke] texture pattern: 6 formas. (1) GRADE, ja' selecionada - o chip **Pattern** esta' \
         aceso na seccao Fill Type. (2) TIJOLO 1/2: as linhas desfasam meia celula. (3) COLMEIA: o \
         mesmo desfasamento com o espacamento sqrt(3)/2. (4) ESPELHO: a arte inverte a cada \
         repeticao. (5) BURACO (EvenOdd): o miolo tem de ficar VAZIO - se o padrao o pintar, a \
         regra de preenchimento nao viajou. (6) ESTICADA: a mesma grade numa forma escalada so' em \
         x - o padrao ESMAGA com ela, ao contrario do traco. A arte tem um quadrante TRANSPARENTE \
         (canto inferior direito de cada copia): ele tem de deixar ver o fundo, nao pintar vermelho. \
         ⭐ E EM BAIXO: um quadrado cuja ARTE e' o TRIANGULO ao lado dele (uma forma do documento). \
         Mexa nos nos do triangulo com a ferramenta Node -- o padrao tem de mudar NA HORA. \
         ⭐ TODO o ajuste vive no painel, na seccao Pattern: Tile, Offset, Size, Gap, Shift X, \
         Shift Y, Angle e Repeat. As barras SHIFT X/Y deslizam a arte dentro de UMA repeticao \
         (0..100%, e 100 e' o mesmo que 0). No modo Clamp elas somem, com as outras que ele nao le^."
    );
}
