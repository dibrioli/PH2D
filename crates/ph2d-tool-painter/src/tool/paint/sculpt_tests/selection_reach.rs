//! **A marquee é o alcance da espátula — inclusive para a BOLA** (Enio, 2026-08-07, com duas fotos:
//! *"Impasto:inflate:Filter Layer em uma seleção, a deformação vaza/extrapola a seleção"*).
//!
//! O irmão `the_sculpt_respects_the_selection` (em [`super::super::sculpt_tests`]) já afirma isto para um
//! traço — **com o verbo Smooth**, e é por isso que ele ficou verde sobre o defeito: todo verbo por-texel
//! escreve `pre + k·Δ` com `k = amount[i]`, e o `amount` já nasce atenuado pela seleção, então o
//! `continue` do `render_sculpt` os confina de graça. **Medido:** Smooth e Sharpen vazam ZERO texel em
//! qualquer raio, como traço e como filtro — *a fixture daquele gate não continha o verbo que vaza*.
//!
//! O Blob é outra coisa: uma **dilatação**, cuja razão de existir é crescer além dos texels tocados. Os
//! números do defeito, pela porta do produto: **7 px** além da borda com o knob Smooth em 0, **23 px** com
//! ele no máximo, e **6 px** num traço que terminou 20 px DENTRO da seleção.

use super::super::sculpt_filter::FilterScope;
use super::super::sculpt_tests::{arm_sculpt, sculpt_canvas};
use crate::tool::PainterTool;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};

const SMOOTH: u8 = 0;
const INFLATE: u8 = 7;
const SIZE: u32 = 200;
/// A borda da seleção: tudo com `x >= 100` está FORA.
const EDGE: usize = 100;

fn cp(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

/// `(texels mudados DENTRO, texels mudados FORA, o x mais distante que se mexeu)`.
fn split(before: &[f32], now: &[f32]) -> (usize, usize, usize) {
    let (mut inside, mut outside, mut max_x) = (0usize, 0usize, 0usize);
    for (i, (&a, &b)) in now.iter().zip(before.iter()).enumerate() {
        if (a - b).abs() <= 1e-4 {
            continue;
        }
        let x = i % SIZE as usize;
        if x >= EDGE {
            outside += 1;
            max_x = max_x.max(x);
        } else {
            inside += 1;
        }
    }
    (inside, outside, max_x)
}

/// Uma pincelada de impasto REAL -- a rota de dab do produto, o deposito do produto -- terminando 10 px
/// DENTRO da borda da selecao, sobre tela nua. E a unica fixture que contem a materia: `mats`/`covers`/
/// `rgba` so existem porque o deposito os escreveu.
fn deposited_near_the_edge() -> (PainterTool, crate::tool::RtLayerId) {
    let mut t = PainterTool::default();
    let n = (SIZE * SIZE) as usize;
    t.set_source(vec![255u8; n * 4], SIZE, SIZE);
    let b = ph2d_painter_brush::BrushSpec {
        radius_px: 16.0,
        hardness: 1.0,
        falloff: ph2d_painter_brush::Falloff::Smooth,
        strength: 1.0,
        color: [0.1, 0.2, 0.9],
        space_attenuation: false,
        impasto: true,
        ..Default::default()
    };
    t.paint.brush = b;
    for slot in &mut t.paint.brush_by_mode {
        *slot = b;
    }
    t.set_paint_tool_mode("brush");
    t.set_brush_impasto_depth(1.0);
    let layer = t.layers.active().expect("uma camada");
    t.on_canvas_pointer(cp([40.0, 100.0], PointerPhase::Down));
    let mut x = 44.0;
    while x <= 90.0 {
        t.on_canvas_pointer(cp([x, 100.0], PointerPhase::Move));
        x += 4.0;
    }
    t.on_canvas_pointer(cp([90.0, 100.0], PointerPhase::Up));
    (t, layer)
}

/// **O filtro não cresce para fora da marquee** — o caso que o Enio fotografou, no PIOR ajuste.
///
/// O knob **Smooth** do Inflate está no MÁXIMO de propósito: ele borra a superfície absoluta onde a bola
/// agiu, alargando a janela por cima dela, e era ele quem levava o vazamento de 7 para **23 px** (foi o
/// *"isso acontece com smooth também"* do report). Se a confinação fosse aplicada ANTES do blur, este
/// gate seria o que sangraria — o blur esfregaria o resultado por cima da fronteira outra vez.
///
/// **Mutação que sangra:** tirar o bloco de confinação do `render_inflate` — 4600 texels fora, `x_max`
/// 122 contra a borda em 99.
#[test]
fn the_filter_does_not_inflate_past_the_selection() {
    let (mut t, layer, before) = sculpt_canvas(SIZE);
    t.set_rect_selection(0, 0, EDGE as u32, SIZE);
    assert!(
        t.selection_restricts_paint(),
        "fixture: sem selecao viva este gate nao e sobre nada"
    );
    arm_sculpt(&mut t, INFLATE, 0.5, 1.0);
    t.set_sculpt_smooth(1.0); // o pior caso, e a metade "smooth" do report
    assert!(t.filter_sculpt_layer(FilterScope::Layer), "o filtro rodou");
    let now = t.heights.get(&layer).expect("relevo");
    let (inside, outside, max_x) = split(&before, now);
    assert!(
        inside > 1000,
        "o filtro moveu so {inside} texels DENTRO — ele nao esta funcionando"
    );
    assert_eq!(
        outside,
        0,
        "a bola cresceu {outside} texels FORA da selecao (ate x={max_x}, borda em {})",
        EDGE - 1
    );
}

/// **E um TRAÇO também não** — o vazamento nunca foi do Filter Layer: os dois dirigem o mesmo
/// `render_inflate`, e um traço que termina 20 px dentro da borda empurrava a forma 6 px para fora dela.
///
/// **Mutação que sangra:** a mesma — 218 texels fora, `x_max` 105.
#[test]
fn a_stroke_does_not_inflate_past_the_selection() {
    let (mut t, layer, before) = sculpt_canvas(SIZE);
    t.set_rect_selection(0, 0, EDGE as u32, SIZE);
    arm_sculpt(&mut t, INFLATE, 0.5, 1.0);
    // O traço inteiro fica DENTRO, e para a 20 px da borda: o que cruzar a marquee foi a bola, não a mão.
    t.on_canvas_pointer(cp([50.0, 100.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([65.0, 100.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([80.0, 100.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([80.0, 100.0], PointerPhase::Up));
    let now = t.heights.get(&layer).expect("relevo");
    let (inside, outside, max_x) = split(&before, now);
    assert!(inside > 500, "o traco moveu so {inside} texels DENTRO");
    assert_eq!(
        outside, 0,
        "a bola cresceu {outside} texels FORA da selecao (ate x={max_x}) num traco que parou em x=80"
    );
}

/// **E a MATÉRIA não atravessa a marquee tampouco.**
///
/// O Blob é o único verbo que MOVE matéria: uma forma que cresce sobre tela nua sem levar a tinta junto
/// não cresceu (a luz pesa por cobertura). Então confinar a altura e deixar o argmax de pé daria cobertura
/// e cor crescendo onde o relevo não cresce — a doença de *duas coisas que devem concordar sobre onde a
/// forma acaba, discordando*, que esta linha já pagou como a ESCADA da silhueta (2026-07-16).
///
/// ⚠️ **A fixture é um DEPÓSITO REAL, e três tentativas minhas antes dela não continham o fenômeno** — a
/// advecção exige os planos `covers`/`mats`/`rgba`, e só o depósito do produto os escreve; um relevo
/// sintético deixa `matter_ok` falso e o bloco inteiro não roda. O que denunciou foi o **CONTROLE**: com
/// ele mudo (`0` texels movidos DENTRO), os três zeros lá fora não eram sobre nada.
///
/// **Mutação que sangra:** não zerar o `sbuf` na confinação (confinar só a altura) — **60 texels** de
/// cobertura E de cor atravessam, até x=106.
#[test]
fn the_matter_does_not_cross_the_selection_either() {
    let (mut t, layer) = deposited_near_the_edge();
    let covers = (**t.covers.get(&layer).expect("cobertura")).clone();
    let rgba = (*t.canvas_rgba).clone();
    t.set_rect_selection(0, 0, EDGE as u32, SIZE);
    arm_sculpt(&mut t, INFLATE, 0.5, 1.0);
    assert!(t.filter_sculpt_layer(FilterScope::Layer), "o filtro rodou");
    let cov = t.covers.get(&layer).expect("cobertura");
    let now = t.canvas_rgba.clone();
    let (mut inside, mut cov_out, mut rgb_out, mut max_x) = (0usize, 0usize, 0usize, 0usize);
    for y in 0..SIZE as usize {
        for x in 0..SIZE as usize {
            let i = y * SIZE as usize + x;
            if x < EDGE {
                inside += usize::from(cov[i] != covers[i]);
                continue;
            }
            if cov[i] != covers[i] {
                cov_out += 1;
                max_x = max_x.max(x);
            }
            rgb_out += usize::from(now[i * 4..i * 4 + 4] != rgba[i * 4..i * 4 + 4]);
        }
    }
    assert!(
        inside > 100,
        "CONTROLE: a materia moveu so {inside} texels DENTRO — a fixture nao contem o fenomeno, \
         e os zeros la fora nao seriam sobre nada"
    );
    assert_eq!(
        cov_out, 0,
        "a bola levou cobertura para {cov_out} texels fora da selecao (ate x={max_x})"
    );
    assert_eq!(rgb_out, 0, "e cor para {rgb_out} deles");
}

/// **Uma seleção que cobre TUDO é byte-idêntica a nenhuma seleção** — o guarda de regressão da wave.
///
/// É ele que prova que o mundo que o Enio aprovou não se moveu: a confinação tem um `continue` exato em
/// `s >= 1`, então dentro da marquee (e sem marquee nenhuma) a bola escreve o que sempre escreveu, ao bit.
/// Sem esse `continue`, `p + 1·(h − p)` **não é** `h` em `f32` sempre que a subtração cancela.
///
/// ⚠️ O gate compara os DOIS lados que a bola escreve — altura E cobertura —, porque confinar um só é
/// precisamente o modo de falha do gate acima.
///
/// **Mutação que sangra:** trocar o `continue` por um lerp incondicional.
#[test]
fn a_selection_that_covers_everything_is_the_same_as_no_selection() {
    let run = |select_all: bool| {
        let (mut t, layer, _) = sculpt_canvas(SIZE);
        if select_all {
            t.set_rect_selection(0, 0, SIZE, SIZE);
            assert!(
                t.selection_restricts_paint(),
                "fixture: a selecao esta viva"
            );
        }
        arm_sculpt(&mut t, INFLATE, 0.5, 1.0);
        assert!(t.filter_sculpt_layer(FilterScope::Layer), "o filtro rodou");
        let h = (**t.heights.get(&layer).expect("relevo")).clone();
        let c = (**t.covers.get(&layer).expect("cobertura")).clone();
        (h, c)
    };
    let (h_free, c_free) = run(false);
    let (h_all, c_all) = run(true);
    let diff = h_free
        .iter()
        .zip(h_all.iter())
        .filter(|(a, b)| a.to_bits() != b.to_bits())
        .count();
    assert_eq!(
        diff, 0,
        "{diff} alturas divergem de uma selecao que nao restringe nada"
    );
    assert_eq!(
        c_free, c_all,
        "a cobertura divergiu de uma selecao que cobre tudo"
    );
}

/// **E o verbo por-texel segue sem precisar de nada disto** — o CONTROLE que separa *"o Blob vazava"* de
/// *"o sculpt vazava"*, e a razão de a cura morar no `render_inflate` e não no `render_sculpt`.
///
/// Sem ele a wave inteira poderia ter sido escrita no lugar errado (confinar a saída de TODO verbo), o que
/// aplicaria a seleção **duas vezes** no Smooth — uma no `amount`, outra na saída — e é a exata armadilha
/// de duplo-escalamento que o doc do `filter_sculpt_layer` já nomeia.
#[test]
fn the_per_texel_verbs_never_needed_confining() {
    let (mut t, layer, before) = sculpt_canvas(SIZE);
    t.set_rect_selection(0, 0, EDGE as u32, SIZE);
    arm_sculpt(&mut t, SMOOTH, 1.0, 1.0); // raio MÁXIMO: o borrão lê bem para fora da marquee
    assert!(t.filter_sculpt_layer(FilterScope::Layer), "o filtro rodou");
    let now = t.heights.get(&layer).expect("relevo");
    let (inside, outside, _) = split(&before, now);
    assert!(inside > 1000, "o Smooth moveu so {inside} texels DENTRO");
    assert_eq!(
        outside, 0,
        "o Smooth mudou {outside} texels fora — o `amount` deixou de confina-lo"
    );
}
