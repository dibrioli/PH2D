//! **QUEM ESCREVE DECLARA** — o carimbo de umidade de um dab de tinta é uma
//! escrita no documento, e ela tem de sair pela porta que reporta o sujo.
//!
//! ⚠️ **Report do Enio (2026-08-07):** *"Show Wet só aparece se a água for
//! colocada antes e depois se checar Show Wet. Mas se Show Wet estiver checado e
//! pintar com água pura (Pigment 0) não se vê a água."*
//!
//! As duas frases são UM defeito visto de dois lados. Um dab de tinta escreve no
//! grid por **duas** portas — o `accumulate` carimba `wet[i]` (todo dab, e
//! ANTES do teste de janela) e o `transfer` pousa o pigmento (só quando a janela
//! enche) — e só a segunda devolvia um `TouchedRect`. O véu do Show Wet lê
//! `wet`, então ele descrevia um mundo que ninguém tinha declarado sujo: ficava
//! invisível até um recompose de folha inteira (o toggle) o revelar.
//!
//! **Com Pigment 0 isso é total**, porque o laço de pouso do `transfer` pula
//! todo texel sem pigmento ⇒ retângulo nenhum ⇒ nada suja ⇒ o carimbo de
//! umidade que o `accumulate` acabou de gravar não aparece nunca.
//!
//! ⚠️ **Nada aqui muda o grid** — a correção só DECLARA. O
//! `scripted_session_fingerprint_is_stable` é a prova executável disso.

use ph2d_wet_paint::brush::BrushShape;
use ph2d_wet_paint::painter::Engine;
use ph2d_wet_paint::trail::{Dab, Trail, TrailMode};
use ph2d_wet_paint::tuning::Knob;

const CX: f64 = 200.0;
const CY: f64 = 200.0;
const R: f64 = 24.0;

fn dab() -> Dab {
    Dab {
        x: CX,
        y: CY,
        r: R,
        hardness: 0.5,
        intensity: 1.0,
        water_amount: 0.5,
        dry_gate: 0.0,
        shape: BrushShape::Round,
        dir_x: 1.0,
        dir_y: 0.0,
    }
}

/// A umidade que semeamos antes do dab, para que a ESCRITA seja observável.
///
/// ⚠️ **O oráculo tem de ser a escrita, nunca o valor.** `wet_byte_from_paper` é
/// o INVERSO do dente do papel (`2 − 2·tooth`), e o gate do papel só deixa
/// depositar onde o dente é ALTO — ou seja, exatamente as células que recebem
/// tinta recebem um byte BAIXO, às vezes zero. Num grid virgem (tudo zero) essa
/// escrita seria invisível, e o gate ficaria verde por vácuo.
const SENTINEL: u8 = 7;

/// O que um dab deixou para trás: as células cuja umidade MUDOU, o retângulo que
/// o motor declarou, e o stride do grid (para reconstruir as coordenadas).
type DabTrace = (Vec<usize>, Option<(i32, i32, i32, i32)>, i32);

/// Um dab acumulado num motor novo, com o pigmento no valor dado.
fn one_dab(pigment: f64) -> DabTrace {
    let mut e = Engine::new(400, 400);
    e.tuning.set(Knob::PigmentPerDab, pigment);
    let p = e.sim.gather_params(&e.tuning);
    let tex: Vec<f32> = e.bristle_texture_for_measure();
    let mut t = Trail::default();
    t.start_stroke(CX, CY, [0.2, 0.3, 0.4], TrailMode::Paint);
    t.on_segment(4.0, 0.05 * 2.0 * R);
    let g = e.active_grid_mut();
    g.wet.fill(SENTINEL);
    let acc = t.accumulate_paint(g, &p, &tex, &dab(), false);
    let g = e.active_grid();
    let touched: Vec<usize> = (0..g.cells).filter(|&i| g.wet[i] != SENTINEL).collect();
    (
        touched,
        acc.wrote.map(|r| (r.x0, r.y0, r.x1, r.y1)),
        g.s as i32,
    )
}

/// **O retângulo declarado CONTÉM toda célula que o dab molhou.**
///
/// O oráculo é o GRID, não a aritmética do dab: varre-se o mapa de umidade e
/// exige-se que o retângulo o cubra. Um gate que recomputasse a pegada do
/// pincel seria um espelho da regra que julga.
///
/// **Mutação que sangra:** devolver `wrote: None` do `accumulate_paint_impl`,
/// ou mover o `touch_ext` para depois do teste de janela.
#[test]
fn the_declared_rect_covers_every_cell_the_dab_dampened() {
    for pigment in [600.0f64, 0.0] {
        let (touched, rect, s) = one_dab(pigment);
        assert!(
            !touched.is_empty(),
            "a fixture nao contem o fenomeno: nenhum texel molhado (pigmento {pigment})"
        );
        let (x0, y0, x1, y1) =
            rect.unwrap_or_else(|| panic!("nada declarado com pigmento {pigment}"));
        for i in touched {
            let (cx, cy) = (i as i32 % s, i as i32 / s);
            assert!(
                cx >= x0 && cx <= x1 && cy >= y0 && cy <= y1,
                "a celula molhada ({cx},{cy}) esta FORA do retangulo declarado \
                 ({x0},{y0})..({x1},{y1}) com pigmento {pigment}"
            );
        }
    }
}

/// **Com pigmento ZERO o dab ainda molha, e ainda declara.** É o caso do report:
/// o `transfer` nao pousa nada (o laço de pouso pula todo texel sem pigmento),
/// então antes desta correção o dab escrevia a umidade e devolvia rect nenhum.
///
/// **Mutação que sangra:** a mesma acima — e SÓ este gate a pega quando o
/// pigmento é 0, porque com pigmento o `transfer` ainda devolveria um rect e
/// mascararia a falta.
#[test]
fn a_pigmentless_dab_still_dampens_the_paper_and_says_so() {
    let (touched, rect, _) = one_dab(0.0);
    assert!(
        !touched.is_empty(),
        "com Pigment 0 o dab tem de carimbar a umidade (o mapa que o Show Wet le)"
    );
    assert!(
        rect.is_some(),
        "com Pigment 0 nada seria declarado sujo, e o veu ficaria invisivel ate um \
         recompose de folha inteira"
    );
}
