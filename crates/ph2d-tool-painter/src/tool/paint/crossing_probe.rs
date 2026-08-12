//! **Sonda do ENTALHE no CRUZAMENTO** — a reentrância branca que o Enio fotografou numa cruz de
//! aquarela (2026-08-12: *"em vários lugares deste app e principalmente na implementação do traço de
//! FLIP tivemos problemas com o Alpha que criava reentrâncias nos cruzamentos de traços. Parece que o
//! mesmo ocorre com watercolor"*).
//!
//! **A pergunta que ela responde é uma só, e é aritmética.** Duas faixas ortogonais de meia-largura
//! `R` se cruzam. Um ponto na bissetriz, a `s` px de CADA eixo, recebe de cada faixa a MESMA
//! cobertura `f(s)` que receberia dela sozinho. As duas leis possíveis dão números diferentes:
//!
//! * **UNIÃO** (`max`, o envelope): `cov = f(s)` ⇒ a axila fica **igual** ao ombro solitário.
//! * **COMPOSIÇÃO** (cobertura independente): `cov = 1 − (1−f)² = 2f − f²` ⇒ a axila fica **acima**,
//!   com o máximo em `f = 0,5` (0,50 → 0,75).
//!
//! Então a sonda mede `axila(s)` contra `ombro(s)` — o MESMO `s`, o mesmo pincel, longe do
//! cruzamento — e o predito pela composição ao lado. Não precisa de imagem de referência: o oráculo é
//! o próprio ombro da faixa.
//!
//! ⚠️ **`s` corre DENTRO da faixa, e a 1ª versão desta sonda media FORA — e leu zero em tudo.** Com
//! `hardness 0` + `Falloff::Smooth` o perfil chega a zero EM `R`: o ombro macio vive dentro do disco,
//! não além dele. A tabela de zeros passava como "sem diferença" e teria fechado a investigação no
//! lugar errado; o `controle` (quantos px têm tinta) é o que a denunciou.
//!
//! ⚠️ **É exatamente a lei que o FLIP curou em 2026-07-28** (`flip.wgsl`, §"UMA PASSAGEM, UMA
//! COBERTURA"): *"`min` de duas funções lisas tem VINCO na bissetriz do cruzamento — invisível em
//! hardness 1 (máscara binária) e uma costura com pincel macio"*. Lá o desvio entre as duas rotas
//! media 48/255 em hardness 0,4.
//!
//! Rodar: `cargo test -p ph2d-tool-painter --release crossing_probe -- --ignored --nocapture`

use crate::tool::PainterTool;
use ph2d_editor_core::tool::{CanvasPaintTool, PointerPhase, RasterEditTool};
use ph2d_painter_brush::BrushSpec;

use super::accumulate_probe::cp;

const SIZE: u32 = 256;
/// O centro do cruzamento.
const C: f32 = 128.0;
/// Meia-largura da faixa = o raio do pincel.
const R: f32 = 24.0;
/// Onde o ombro SOLITÁRIO é medido — longe do cruzamento, no mesmo `t` perpendicular.
const LONE_X: f32 = 56.0;

fn white(size: u32) -> Vec<u8> {
    vec![255u8; (size * size * 4) as usize]
}

/// O pincel do produto para esta cena: aquarela, macio, um traço largo.
fn wash_brush() -> BrushSpec {
    BrushSpec {
        radius_px: R,
        color: [0.85, 0.15, 0.15],
        space_attenuation: false,
        watercolor: true,
        ..Default::default()
    }
}

fn arm(t: &mut PainterTool, spec: BrushSpec) {
    t.paint.brush = spec;
    for slot in &mut t.paint.brush_by_mode {
        *slot = spec;
    }
}

fn stroke(t: &mut PainterTool, from: [f32; 2], to: [f32; 2]) {
    t.on_canvas_pointer(cp(from, PointerPhase::Down));
    let (dx, dy) = (to[0] - from[0], to[1] - from[1]);
    let n = 24;
    for i in 1..=n {
        let f = i as f32 / n as f32;
        t.on_canvas_pointer(cp([from[0] + dx * f, from[1] + dy * f], PointerPhase::Move));
    }
    t.on_canvas_pointer(cp(to, PointerPhase::Up));
}

/// Opacidade da tinta num ponto: vermelho sobre branco ⇒ o canal VERDE cai com a tinta.
fn alpha_at(t: &PainterTool, x: f32, y: f32) -> f32 {
    let (xi, yi) = (x.round() as u32, y.round() as u32);
    let i = ((yi * SIZE) + xi) as usize * 4;
    (255.0 - f32::from(t.canvas_rgba[i + 1])) / 255.0
}

/// A cruz feita de DOIS traços (o gesto que a foto mostra).
fn cross_two_strokes(spec: BrushSpec) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(white(SIZE), SIZE, SIZE);
    arm(&mut t, spec);
    stroke(&mut t, [40.0, C], [216.0, C]);
    stroke(&mut t, [C, 40.0], [C, 216.0]);
    t
}

/// UM traço que cruza a si mesmo — o caso que no FLIP se comportava DIFERENTE dos dois traços,
/// porque ali o depth é o mesmo e a lei caía na união em vez de compor.
fn cross_one_stroke(spec: BrushSpec) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(white(SIZE), SIZE, SIZE);
    arm(&mut t, spec);
    // Um "L" que volta e atravessa a própria perna: (40,C) → (216,C) → (216,56) → (C,56) → (C,216).
    t.on_canvas_pointer(cp([40.0, C], PointerPhase::Down));
    let legs = [
        ([40.0, C], [216.0, C]),
        ([216.0, C], [216.0, 56.0]),
        ([216.0, 56.0], [C, 56.0]),
        ([C, 56.0], [C, 216.0]),
    ];
    for (from, to) in legs {
        let (dx, dy) = (to[0] - from[0], to[1] - from[1]);
        for i in 1..=24 {
            let f = i as f32 / 24.0;
            t.on_canvas_pointer(cp([from[0] + dx * f, from[1] + dy * f], PointerPhase::Move));
        }
    }
    t.on_canvas_pointer(cp([C, 216.0], PointerPhase::Up));
    t
}

fn report(label: &str, t: &PainterTool) {
    println!("\n  {label}");
    // CONTROLE: a cena tem de conter tinta, senao a tabela abaixo compara dois zeros.
    let painted = t
        .canvas_rgba
        .chunks_exact(4)
        .filter(|p| p[1] != 255)
        .count();
    println!(
        "      (controle: {painted} px com tinta; centro do cruzamento alpha={:.3}; \
         miolo do braco alpha={:.3})",
        alpha_at(t, C, C),
        alpha_at(t, LONE_X, C)
    );
    println!("      s   ombro   axila   composto   axila-ombro");
    // ⚠️ O ombro macio vive DENTRO do disco: com `hardness 0` + `Falloff::Smooth` o perfil chega a
    // zero EM `R`, entao nao ha nada a medir fora da faixa. `s` e a distancia perpendicular ao eixo,
    // de 0 (o eixo) a R (a borda).
    for s in [0.0f32, 4.0, 8.0, 12.0, 16.0, 18.0, 20.0, 22.0] {
        // Ombro solitário: `s` px do eixo da faixa horizontal, longe do cruzamento.
        let lone = alpha_at(t, LONE_X, C + s);
        // Axila: na bissetriz, `s` px de CADA eixo — as duas faixas contribuem o mesmo `f(s)`.
        let pit = alpha_at(t, C + s, C + s);
        let composed = 2.0 * lone - lone * lone;
        println!(
            "   {s:5.1}   {lone:5.3}   {pit:5.3}      {composed:5.3}        {:+.3}",
            pit - lone
        );
    }
}

/// **RENDER-AND-LOOK headless** — o mapa do alfa em volta da quina côncava. A tabela acima diz
/// NÚMEROS; isto diz FORMA, que é o que a foto mostra: se há uma cunha clara entrando pela
/// bissetriz, ela aparece aqui como uma faixa de dígitos baixos entre dois aros escuros.
fn map(label: &str, t: &PainterTool) {
    println!(
        "\n  {label}  (x,y de {C:.0} a {:.0}; digito = alpha*9)",
        C + 40.0
    );
    for y in 0..40u32 {
        let mut row = String::new();
        for x in 0..40u32 {
            let a = alpha_at(t, C + x as f32, C + y as f32);
            let d = (a * 9.0).round().clamp(0.0, 9.0) as u32;
            row.push(if d == 0 {
                '.'
            } else {
                char::from_digit(d, 10).unwrap()
            });
        }
        println!("   {row}");
    }
}

/// A medição. Imprime as duas cenas (dois traços · um traço que se cruza) para a aquarela e o
/// controle DIGITAL, que é a lei que ninguém reportou como quebrada.
#[test]
#[ignore = "sonda; roda com --ignored --nocapture"]
fn measure_the_crossing_notch() {
    println!("\n=== O ENTALHE NO CRUZAMENTO ===");
    println!("  uniao  ⇒ axila == ombro   |   composicao ⇒ axila == composto (> ombro)");

    let wash = wash_brush();
    report("AQUARELA — dois tracos", &cross_two_strokes(wash));
    report(
        "AQUARELA — um traco cruzando a si mesmo",
        &cross_one_stroke(wash),
    );

    let digital = BrushSpec {
        watercolor: false,
        ..wash
    };
    report(
        "DIGITAL (controle) — dois tracos",
        &cross_two_strokes(digital),
    );
    report(
        "DIGITAL (controle) — um traco cruzando a si mesmo",
        &cross_one_stroke(digital),
    );

    println!("\n=== A FORMA (o quadrante inferior-direito da quina) ===");
    map("AQUARELA — dois tracos", &cross_two_strokes(wash));
    map(
        "DIGITAL (controle) — dois tracos",
        &cross_two_strokes(digital),
    );
}
