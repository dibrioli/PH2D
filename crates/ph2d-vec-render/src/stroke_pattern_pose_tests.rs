//! **OS GATES DA ESTAMPA DO TRAÇO SOB A POSE DA ENTIDADE** (ITEM B) — irmão do
//! [`super::pattern_stroke_tests`] pelo teto de LOC, e o corte é por SUJEITO: ali a estampa é a
//! tinta de uma FAIXA, aqui é o que lhe acontece quando a forma tem um `Transform` **não-uniforme**
//! (alcançável por estados de UI / Smart Animate).
//!
//! # A lei que estes gates defendem
//!
//! *Numa faixa, a banda e o que está dentro dela escalam pelo MESMO fator, e esse fator é o da
//! CANETA (`√|det|`).* Ela foi curada na rota do BAKE em `a14b6a0cb` e ficou viva nesta, a do
//! DESENHO — medido, `asp/aut = 2,7143×` a `rot·(1,9 · 0,7)` e `3,0000×` a `(3, 1)`.
//!
//! # ⚠️ A régua tem DUAS metades, e só a segunda apanha a construção errada
//!
//! Des-esticar o ladrilho **desloca-o** — e uma cura que só uniformize a escala passa num gate de
//! aspecto enquanto deixa a estampa fora da forma. É no `Clamp` que isso se vê (ali desenha-se
//! **uma** cópia enquadrada), e por isso a posição é medida ao lado do aspecto, sempre.

use crate::stroke_uniform::{PatternFrame, is_conformal};
use ph2d_vec_scene::{
    PatternFill, PatternSource, Rgba8, StrokePaint, StrokeSpec, VecPath, VecPathId, VecScene,
    VecVertex, VecXforms, Xform,
};
use ph2d_vector::{Affine, BezPath, ImageQuality, Point, Shape, StableImage, VectorScene};

// ── A FIXTURA, e ela contém o fenómeno por CONSTRUÇÃO ────────────────────────────────

/// A arte assada: **8 × 4 px, não-quadrada**. ⚠️ Uma arte quadrada não contém o fenómeno — o
/// esticão mede-se num aspecto, e um aspecto de `1` fica `1` depois de esticado.
const ART_PX: [u32; 2] = [8, 4];
/// O rasto de UMA cópia no mundo: **`4 × 2`, não-quadrado** — pela mesma razão.
const ART_SIZE: [f64; 2] = [4.0, 2.0];
/// A caixa da forma: **`20 × 10`, não-quadrada** — senão o `Clamp` enquadraria um quadrado e o
/// enquadramento deixaria de distinguir os espaços em que a caixa se mede.
const FORMA: [[f64; 2]; 4] = [[0.0, 0.0], [20.0, 0.0], [20.0, 10.0], [0.0, 10.0]];

fn ladrilho() -> crate::PatternTile {
    let px = (ART_PX[0] * ART_PX[1]) as usize;
    crate::PatternTile {
        image: StableImage::from_rgba(
            std::sync::Arc::new(vec![200u8; px * 4]),
            ART_PX[0],
            ART_PX[1],
        )
        .expect("RGBA 8x4"),
        cells: [1, 1],
        tile_px: ART_PX,
        quality: ImageQuality::Medium,
        wrap_seam: 0,
    }
}

fn estampa(modo: ph2d_vec_pattern::PatternMode) -> PatternFill {
    let mut p = PatternFill::new(
        PatternSource::Shape(1),
        ART_SIZE,
        Rgba8::new(200, 30, 30, 255),
    );
    // ⚠️ A origem NÃO é a do pivô do afim, de propósito: com `origin == pivot` a âncora ficaria
    // parada em qualquer construção e o gate da posição não mediria nada.
    p.origin = [1.0, 3.0];
    p.mode = modo;
    p
}

fn cena(modo: ph2d_vec_pattern::PatternMode) -> (VecScene, VecPathId) {
    let mut scene = VecScene::default();
    let mut s = StrokeSpec::new(Rgba8::new(9, 9, 9, 255), 1.0);
    s.paint = StrokePaint::Pattern(Box::new(estampa(modo)));
    let id = scene.push_path(VecPath {
        verts: FORMA.map(VecVertex::corner).to_vec(),
        closed: true,
        stroke: Some(s),
        ..VecPath::default()
    });
    (scene, id)
}

fn bp_local() -> BezPath {
    let mut b = BezPath::new();
    b.move_to((FORMA[0][0], FORMA[0][1]));
    for v in &FORMA[1..] {
        b.line_to((v[0], v[1]));
    }
    b.close_path();
    b
}

/// A colocação **LOCAL**, tal como o desenho a calculava antes desta cura.
fn colocacao_local(modo: ph2d_vec_pattern::PatternMode) -> Affine {
    let t = ladrilho();
    let b = bp_local().bounding_box();
    Affine::new(estampa(modo).placement_in(t.cells, t.tile_px, ([b.x0, b.y0], [b.x1, b.y1])))
}

/// ⭐ **O ORÁCULO: a lei de ANTES desta cura** — `transform · colocação_local`, que é exactamente o
/// que o `stroke_uniform_image` pré-compunha. O gate mede as duas metades **contra ele**: no
/// conforme tem de bater ao bit, no não-conforme tem de bater na POSIÇÃO e divergir na ESCALA.
fn oraculo(transform: Affine, modo: ph2d_vec_pattern::PatternMode) -> Affine {
    transform * colocacao_local(modo)
}

/// O afim de PINCEL que de facto chegou ao Vello, lido do encoding.
///
/// ⚠️ **Ele é `f32` ali dentro**, e é isso que o GPU vê — comparar em `f64` mediria uma precisão
/// que o desenho não tem. A fixtura emite **um** desenho, e o par `[geometria, pincel]` é afirmado.
fn pincel_no_encoding(alvo: &VectorScene) -> [f32; 6] {
    let xs = &alvo.inner().encoding().transforms;
    assert_eq!(
        xs.len(),
        2,
        "a fixtura deixou de emitir UM desenho ({} transforms) - `.last()` deixa de ser o pincel",
        xs.len()
    );
    let t = xs[1];
    [
        t.matrix[0],
        t.matrix[1],
        t.matrix[2],
        t.matrix[3],
        t.translation[0],
        t.translation[1],
    ]
}

#[allow(clippy::cast_possible_truncation)] // e' precisamente o truncamento que o Vello faz
fn como_encoding(m: Affine) -> [f32; 6] {
    m.as_coeffs().map(|c| c as f32)
}

/// Desenha a cena pela porta do produto, com `xf` como `Transform` da ENTIDADE.
fn desenhar(modo: ph2d_vec_pattern::PatternMode, xf: Xform) -> VectorScene {
    let (scene, id) = cena(modo);
    let mut tiles = crate::PatternTiles::new();
    tiles.insert((id, crate::PatternSlot::Stroke), ladrilho());
    let mut xforms = VecXforms::new();
    xforms.insert(id, xf);
    let mut target = VectorScene::new();
    crate::dispatch(
        &scene,
        &ph2d_vec_scene::VecViewState::default(),
        &xforms,
        &crate::LiveGeometry::new(),
        &crate::FxImages::new(),
        &crate::WidgetSkins::new(),
        &tiles,
        &crate::BrushArts::new(),
        &crate::DilatedPaints::new(),
        Affine::IDENTITY,
        &mut target,
    );
    target
}

// ── As três grandezas que se medem sobre um afim de pincel ────────────────────────────

/// O ASPECTO do ladrilho tal como ele sai na tela: as duas arestas da célula, em unidades de mundo.
fn aspecto(brush: [f32; 6]) -> f64 {
    let c = brush.map(f64::from);
    (c[0].hypot(c[1]) * f64::from(ART_PX[0])) / (c[2].hypot(c[3]) * f64::from(ART_PX[1]))
}

/// A ÂNCORA: o ponto de mundo onde o canto do ladrilho assenta. Em espaço de imagem ele é
/// `(0, altura)` — ver `ph2d_vec_pattern::placement`, cuja linha `0` é o TOPO do desenho.
fn ancora(brush: [f32; 6]) -> [f64; 2] {
    let p = Affine::new(brush.map(f64::from)) * Point::new(0.0, f64::from(ART_PX[1]));
    [p.x, p.y]
}

/// A forma de TELA vista de dentro do ladrilho: em `[0, w] × [0, h]` ela está **coberta** por UMA
/// cópia, que é a promessa do [`ph2d_vec_pattern::PatternMode::Clamp`].
fn forma_dentro_do_ladrilho(brush: [f32; 6], transform: Affine) -> ([f64; 2], [f64; 2]) {
    let b = (Affine::new(brush.map(f64::from)).inverse() * (transform * bp_local())).bounding_box();
    ([b.x0, b.y0], [b.x1, b.y1])
}

// ── ⭐⭐⭐ OS GATES ────────────────────────────────────────────────────────────────────

/// ⭐⭐⭐ **METADE DE CIMA — um afim CONFORME desenha o que desenhava, ao bit.**
///
/// É a metade que protege o caso comum (rotação, translação, escala uniforme), e ela é forte por
/// construção: a moldura de um afim conforme devolve a colocação **sem lhe tocar** e a caixa **sem
/// transformar a geometria**, então o `stroke_uniform_image` recebe literalmente os mesmos
/// argumentos de antes desta cura. Aqui isso é medido **na saída**, contra o oráculo que escreve a
/// lei antiga.
///
/// ⚠️ **Os dois modos entram**, porque só o `Clamp` lê a caixa: sem ele, uma cura que medisse a
/// caixa no espaço errado passaria neste gate.
#[test]
fn a_conformal_pose_encodes_exactly_what_it_encoded_before_the_cure() {
    for modo in [
        ph2d_vec_pattern::PatternMode::Tile,
        ph2d_vec_pattern::PatternMode::Clamp,
    ] {
        for (rotulo, coeffs) in [
            ("identidade", [1.0, 0.0, 0.0, 1.0, 0.0, 0.0]),
            ("translacao", [1.0, 0.0, 0.0, 1.0, 37.0, -11.0]),
            ("escala uniforme 3x", [3.0, 0.0, 0.0, 3.0, 0.0, 0.0]),
            ("rotacao + escala", {
                let m = Affine::translate((5.0, -2.0)) * Affine::rotate(0.6) * Affine::scale(2.5);
                m.as_coeffs()
            }),
            ("reflexao", [-2.0, 0.0, 0.0, 2.0, 0.0, 0.0]),
        ] {
            let m = Affine::new(coeffs);
            assert!(
                is_conformal(m),
                "a fixtura `{rotulo}` nao e' conforme - ela nao mede o caso comum"
            );
            let visto = pincel_no_encoding(&desenhar(modo, Xform(coeffs)));
            assert_eq!(
                visto,
                como_encoding(oraculo(m, modo)),
                "{modo:?}/{rotulo}: o afim de pincel deixou de ser `transform * colocacao` - o \
                 caminho comum mudou de pixel"
            );
        }
    }
}

/// ⛔⛔⛔ **METADE DE BAIXO — sob um afim NÃO-CONFORME o ladrilho não estica E não se move.**
///
/// As duas afirmações vivem no mesmo teste de propósito: **uma cura que só uniformize a escala
/// passa na primeira e falha na segunda**, e foi essa a bifurcação que a rota do BAKE deixou
/// aberta em `a14b6a0cb`. Medido, com a caixa medida no espaço errado: aspecto `1,0000×` (aprovado)
/// e a âncora do `Clamp` a saltar `(1,268, −2,196)`.
///
/// ⚠️ **O CONTROLO é a metade que torna isto uma medição**: o oráculo (a lei de antes) tem de
/// esticar `3,00×`, senão a fixtura não contém o fenómeno e o gate aprova-se sozinho.
#[test]
fn a_non_uniform_pose_neither_stretches_the_tile_nor_moves_it() {
    for modo in [
        ph2d_vec_pattern::PatternMode::Tile,
        ph2d_vec_pattern::PatternMode::Clamp,
    ] {
        for (rotulo, coeffs, esticao) in [
            ("(3, 1)", [3.0, 0.0, 0.0, 1.0, 0.0, 0.0], 3.0),
            (
                "rot * (1,9 . 0,7)",
                (Affine::rotate(0.4) * Affine::scale_non_uniform(1.9, 0.7)).as_coeffs(),
                1.9 / 0.7,
            ),
        ] {
            let m = Affine::new(coeffs);
            assert!(
                !is_conformal(m),
                "a fixtura `{rotulo}` e' conforme - ela nao contem o fenomeno"
            );
            let antes = como_encoding(oraculo(m, modo));
            let depois = pincel_no_encoding(&desenhar(modo, Xform(coeffs)));
            let aut = aspecto(como_encoding(colocacao_local(modo)));

            // ⚠️ CONTROLO: a lei ANTIGA de facto estica. Sem esta linha, um ladrilho que já
            // saísse quadrado faria as duas metades abaixo passarem sem medir nada.
            assert!(
                (aspecto(antes) / aut - esticao).abs() < 1e-4,
                "{modo:?}/{rotulo}: o oraculo devia esticar {esticao:.4}x e esticou {:.4}x - a \
                 fixtura nao contem o fenomeno",
                aspecto(antes) / aut
            );
            // ⭐ METADE 1 — o ASPECTO volta ao autorado.
            assert!(
                (aspecto(depois) / aut - 1.0).abs() < 1e-4,
                "{modo:?}/{rotulo}: o ladrilho saiu a {:.4}x o aspecto autorado - a banda nao \
                 estica e o motivo dentro dela esticou",
                aspecto(depois) / aut
            );
            // ⛔ METADE 2 — e a POSIÇÃO não se mexe. É esta que uma cura só-de-escala falha.
            let (a, b) = (ancora(antes), ancora(depois));
            assert!(
                (a[0] - b[0]).abs() < 1e-3 && (a[1] - b[1]).abs() < 1e-3,
                "{modo:?}/{rotulo}: a ancora do ladrilho saltou de {a:?} para {b:?} - des-esticar \
                 deslocou a estampa, e no Clamp isso deixa a forma por pintar"
            );
        }
    }
}

/// ⛔⛔ **E O `Clamp` CONTINUA A COBRIR A FORMA QUE ENQUADRA.**
///
/// O modo promete *"mostra a imagem uma vez"* e enquadra **cobrindo** — a cura não pode transformar
/// isso numa cópia pequena com borda esticada à volta (que é o report do Enio de 2026-08-27,
/// *"clamp deixa tudo em branco"*, ressuscitado por outra porta).
///
/// ⚠️ **A cobertura mede-se no espaço do LADRILHO, nunca em eixos de tela**: um ladrilho rodado
/// cobre uma forma rodada, e uma caixa alinhada aos eixos diria que não.
///
/// ⭐ **É este o gate que a construção «só compor a parte conforme» reprova**: medida, ela deixa a
/// forma a ocupar `−0,29 .. 13,56` de um ladrilho de `0 .. 8` — quase o dobro do que existe.
#[test]
fn the_clamped_tile_still_covers_the_shape_it_frames() {
    let modo = ph2d_vec_pattern::PatternMode::Clamp;
    for (rotulo, coeffs) in [
        ("conforme 3x", [3.0, 0.0, 0.0, 3.0, 0.0, 0.0]),
        ("(3, 1)", [3.0, 0.0, 0.0, 1.0, 0.0, 0.0]),
        (
            "rot * (1,9 . 0,7)",
            (Affine::rotate(0.4) * Affine::scale_non_uniform(1.9, 0.7)).as_coeffs(),
        ),
    ] {
        let m = Affine::new(coeffs);
        let brush = pincel_no_encoding(&desenhar(modo, Xform(coeffs)));
        let (lo, hi) = forma_dentro_do_ladrilho(brush, m);
        let (w, h) = (f64::from(ART_PX[0]), f64::from(ART_PX[1]));
        assert!(
            lo[0] >= -1e-3 && lo[1] >= -1e-3 && hi[0] <= w + 1e-3 && hi[1] <= h + 1e-3,
            "{rotulo}: a forma ocupa {lo:?}..{hi:?} de um ladrilho de [0,{w}]x[0,{h}] - o Clamp \
             deixou de a cobrir"
        );
        // ⚠️ CONTROLO: a pergunta é real. Sem uma forma MAIOR que uma cópia autorada, o
        // enquadramento seria inerte e este gate ficaria verde sobre um `Clamp` que não enquadra.
        let uma_copia = ART_SIZE[0] * f64::from(uniform_k(m));
        let largura = (m * bp_local()).bounding_box().width();
        assert!(
            largura > uma_copia * 1.5,
            "{rotulo}: a forma ({largura:.3}) nao e' maior que uma copia autorada \
             ({uma_copia:.3}) - o enquadramento nao e' exercido"
        );
    }
}

#[allow(clippy::cast_possible_truncation)] // so' para a mensagem do controlo
fn uniform_k(m: Affine) -> f32 {
    crate::stroke_uniform::uniform_scale(m) as f32
}

// ── A SONDA que escolheu a construção ─────────────────────────────────────────────────

/// ⭐ **SONDA do ITEM B — as TRÊS construções, lado a lado.**
///
/// `hoje` = `transform · colocação` · **`A`** = só compor a parte conforme · **`B`** = `A` mais a
/// caixa medida na forma des-esticada (a que shipou). Imprime aspecto, âncora e cobertura.
///
/// ⚠️ **Foi ela que fechou a bifurcação com número**, e o número que decide não é o aspecto: `A` e
/// `B` dão `1,0000×` os dois, e é no `Clamp` que `A` desloca a âncora e perde a cobertura.
#[test]
#[ignore = "sonda: imprime, nao afirma"]
fn measure_the_three_constructions_of_the_stroke_pattern_brush() {
    let t = ladrilho();
    let bp = bp_local();
    let (tw, th) = (f64::from(ART_PX[0]), f64::from(ART_PX[1]));
    for modo in [
        ph2d_vec_pattern::PatternMode::Tile,
        ph2d_vec_pattern::PatternMode::Clamp,
    ] {
        let p = estampa(modo);
        for (rotulo, m) in [
            ("CONFORME scale(3)", Affine::scale(3.0)),
            (
                "CONFORME rot+scale",
                Affine::rotate(0.6) * Affine::scale(2.0),
            ),
            ("NAO-CONF (3,1)", Affine::scale_non_uniform(3.0, 1.0)),
            (
                "NAO-CONF rot*(1.9,0.7)",
                Affine::rotate(0.4) * Affine::scale_non_uniform(1.9, 0.7),
            ),
        ] {
            let frame = PatternFrame::of(m, p.origin);
            let caixa = |xf: Affine| {
                let b = (xf * bp.clone()).bounding_box();
                ([b.x0, b.y0], [b.x1, b.y1])
            };
            let local = Affine::new(p.placement_in(t.cells, t.tile_px, caixa(Affine::IDENTITY)));
            let esticada = Affine::new(p.placement_in(t.cells, t.tile_px, frame.box_of(&bp)));
            let aut = aspecto(como_encoding(local));
            println!("\n== {modo:?} / {rotulo} ==");
            // ⚠️ **O que se mede é o que o VELLO recebe**, e não o argumento: no caminho conforme
            // a composição é feita por ele (`pen_xf = transform`), no partido não (`IDENTITY`).
            // Sem isto a sonda leria a colocação LOCAL sobre geometria de TELA e diria disparates.
            let vello =
                |b: Affine| crate::stroke_uniform::pen_for(&ph2d_vector::Stroke::new(1.0), m).1 * b;
            for (nome, brush) in [
                // A lei de antes desta cura.
                ("hoje", m * local),
                // ⛔ **A**: a parte conforme com a caixa medida no espaço LOCAL.
                ("A   ", vello(frame.brush_for(local).probe_affine())),
                // ⭐ **B**: a que shipou — a caixa vem da forma des-esticada.
                ("B   ", vello(frame.brush_for(esticada).probe_affine())),
            ] {
                let e = como_encoding(brush);
                let (lo, hi) = forma_dentro_do_ladrilho(e, m);
                println!(
                    "   {nome} asp={:.4} ({:.4}x o autorado)  ancora={:?}  forma no ladrilho \
                     {:.2}..{:.2} x {:.2}..{:.2} de {tw}x{th}",
                    aspecto(e),
                    aspecto(e) / aut,
                    ancora(e).map(|v| (v * 1000.0).round() / 1000.0),
                    lo[0],
                    hi[0],
                    lo[1],
                    hi[1],
                );
            }
        }
    }
}
