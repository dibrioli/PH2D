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
    FillRule, Paint, PatternFill, PatternSource, Rgba8, StrokeSpec, VecPath, VecPathId, VecVertex,
};

/// O lado da arte, em pixels.
const ART: u32 = 32;
/// O lado de cada forma, em unidades de mundo.
const BOX: f64 = 2.2;
/// O passo entre formas.
const STEP: f64 = 2.6;
/// A largura do contorno, em unidades de mundo — a mesma ordem de grandeza dos outros smokes
/// vetoriais desta casa (`0,012`–`0,02`), subida porque aqui ela tem de **ler-se por cima de uma
/// arte com detalhe**, e não por cima de um preenchimento chapado.
const STROKE_W: f64 = 0.03; // LITERAL-PX-OK: largura no domínio do documento

/// ⛔⛔ **TODA forma desta cena NASCE COM CONTORNO** (Enio, 2026-08-27: *"o contorno funciona com as
/// shapes que eu desejo, mas não funcionam com os teus desenhos"*).
///
/// ⚠️ **Era o smoke, não o produto** — e foi o report que fechou uma caça de três mensagens. A
/// ferramenta de forma escreve `path.stroke = Some(..)` **sempre**
/// ([`ph2d_vec_edit`](../../../crates/ph2d-vec-edit/src/shape.rs)), então toda forma que o artista
/// desenha tem contorno; estas nasciam de `..VecPath::default()`, que é `stroke: None`. E o
/// `restyle_selected_strokes` **recusa por desenho** quem não tem um (*"ganhar um traço do nada
/// seria a UI inventando geometria"*) ⇒ a secção *Stroke* ficava **pintada e inerte** só aqui, o
/// que se lê exactamente como *"o padrão anulou o contorno"*.
///
/// ⚠️⚠️ **A lição é da CENA, não do padrão:** uma cena de smoke montada por código não herda o que
/// a ferramenta de autoria garante — *ela tem de nascer no estado em que o artista a encontraria*,
/// senão o smoke mede um objecto que o produto nunca produz.
fn contorno() -> StrokeSpec {
    StrokeSpec::new(Rgba8::new(35, 35, 45, 255), STROKE_W)
}

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

/// A lei de um padrão, com o tamanho de cópia pedido — a porta única, para o preenchimento e o
/// traço nascerem da MESMA conta (plano 35, wave D).
fn lei(
    source: PatternSource,
    kind: TileKind,
    mode: PatternMode,
    fallback: [u8; 3],
    lado: f64,
) -> PatternFill {
    let mut f = PatternFill::new(
        source,
        [lado, lado],
        Rgba8::new(fallback[0], fallback[1], fallback[2], 255),
    );
    f.kind = kind;
    f.offset_denom = 2;
    f.mode = mode;
    f
}

fn pattern(source: PatternSource, kind: TileKind, mode: PatternMode, fallback: [u8; 3]) -> Paint {
    Paint::Pattern(Box::new(lei(source, kind, mode, fallback, BOX / 3.0)))
}

/// ⭐⭐ **UM CONTORNO COM PADRÃO** (plano 35) — a faixa recebe o ladrilho.
///
/// ⚠️ **A largura é DERIVADA do ladrilho, não escolhida**: a `STROKE_W` de `0,03` é fina demais
/// para o motivo se ler (menos de um décimo de uma cópia), e um smoke em que a feature é invisível
/// não é um smoke. Aqui a faixa fica em `1,2 ×` o lado de uma cópia — larga o bastante para se ver
/// **o que** repete, estreita o bastante para se ver **que** repete ao longo do perímetro (~24
/// cópias nos `8,8` de contorno de uma destas formas).
///
/// ⛔ E ela **não** manda no padrão: engrossar a linha muda a faixa, nunca o motivo (plano 35 §2.3).
fn contorno_com_padrao(source: PatternSource, mode: PatternMode, fallback: [u8; 3]) -> StrokeSpec {
    let lado = BOX / 6.0;
    let mut s = StrokeSpec::new(
        Rgba8::new(fallback[0], fallback[1], fallback[2], 255),
        lado * 1.2,
    );
    s.paint = ph2d_vec_scene::StrokePaint::Pattern(Box::new(lei(
        source,
        TileKind::Grid,
        mode,
        fallback,
        lado,
    )));
    s
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
            stroke: Some(contorno()),
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
        stroke: Some(contorno()),
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
        stroke: Some(contorno()),
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
        stroke: Some(contorno()),
        ..VecPath::default()
    });

    // ⭐⭐ 8 — **SÓ CONTORNO, com padrão** (plano 35). Sem `fill` nenhum: é o caso que prova que a
    // faixa é o sujeito, e é também o que obriga o `Clamp` a enquadrar pela caixa do TRAÇO — um
    // enquadramento pela do preenchimento não teria o que ler aqui.
    scene.push_path(VecPath {
        verts: rect(x(0), -3.0, half),
        closed: true,
        fill: None,
        stroke: Some(contorno_com_padrao(source, PatternMode::Tile, [40, 40, 55])),
        ..VecPath::default()
    });

    // ⭐⭐ 9 — **OS DOIS**, com leis DIFERENTES (grade no preenchimento, espelho no traço). É esta
    // forma que faz aparecer a fileira `Fill | Stroke` no topo da secção *Pattern*: com um alvo só
    // não há escolha a oferecer, e o chip não é pintado.
    scene.push_path(VecPath {
        verts: rect(x(1), -3.0, half),
        closed: true,
        fill: Some(pattern(
            source,
            TileKind::Grid,
            PatternMode::Tile,
            [100, 100, 120],
        )),
        stroke: Some(contorno_com_padrao(
            source,
            PatternMode::Mirror,
            [40, 55, 40],
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
        stroke: Some(contorno()),
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
         ⭐ TODO o ajuste vive no painel, na seccao Pattern: Tile, Offset, Width, Height, \
         Lock Aspect, Gap, Shift X, Shift Y, Angle e Repeat. Com o CADEADO ligado (o default) mexer \
         num eixo leva o outro; desligado, a arte ACHATA de proposito. As barras SHIFT X/Y deslizam a arte dentro de UMA repeticao \
         (0..100%, e 100 e' o mesmo que 0). No modo Clamp elas somem, com as outras que ele nao le^. \
         ⭐ E TODA forma desta cena nasce COM CONTORNO (escuro, fino) -- antes nasciam sem nenhum, e \
         a seccao Stroke ficava inerte SO' AQUI. Troque Fill Type entre Solid e Pattern: o contorno \
         tem de continuar la', e a largura/cor dele tem de responder ao painel. \
         ⭐⭐ E EMBAIXO A' ESQUERDA, DUAS FORMAS NOVAS: a 1a e' SO' CONTORNO, e o contorno dela e' \
         feito da arte (sem preenchimento nenhum). A 2a tem padrao NOS DOIS -- grade no miolo, \
         espelhado no contorno. Selecione a 2a: a seccao Stroke ganha a fileira **Type** \
         (Solid | Pattern) e a seccao Pattern ganha, no topo, a fileira **Target** \
         (Fill | Stroke) -- e' ela que diz qual dos dois os knobs abaixo estao a editar. Na 1a \
         forma o Target NAO aparece, porque com um alvo so' nao ha' escolha. Engrosse o contorno \
         com a barra Width: a faixa engrossa e o MOTIVO nao muda de tamanho."
    );
}
