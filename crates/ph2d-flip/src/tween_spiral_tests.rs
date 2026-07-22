//! Gates da espiral (`tween_spiral`).
//!
//! O oráculo de cada um é uma propriedade **do movimento que se vê** — comprimento
//! preservado, ponto fixo parado, extremos fechando — nunca a fórmula reescrita ao lado.

use super::*;
use ph2d_core::Vec2;

/// O ponto do traço no fator `u`, pela fórmula do chamador: rígido + resíduo.
fn at(m: StrokeMotion, a: Vec2, b: Vec2, u: f32) -> Vec2 {
    m.advance(a, u) + m.residual(a, b) * u
}

/// Gira `pts` de `deg` graus em torno de `c` e escala por `k` (o fixture "pose B").
fn pose(pts: &[Vec2], c: Vec2, deg: f32, k: f32) -> Vec<Vec2> {
    let r = deg.to_radians();
    let (s, co) = (r.sin(), r.cos());
    pts.iter()
        .map(|p| {
            let d = *p - c;
            c + Vec2::new(k * (co * d.x - s * d.y), k * (s * d.x + co * d.y))
        })
        .collect()
}

fn len(pts: &[Vec2]) -> f32 {
    pts.windows(2).map(|w| (w[1] - w[0]).length()).sum()
}

/// O braço do fixture: 10 unidades × 10, do ombro na origem para +X.
fn arm() -> Vec<Vec2> {
    (0..=10).map(|i| Vec2::new(i as f32 * 10.0, 0.0)).collect()
}

/// **O gate que justifica a wave.** Um braço que gira quase meia-volta em torno do ombro:
/// com o lerp de coordenadas o meio do caminho é o segmento entre as duas poses, e o braço
/// COLAPSA a 8,7% do comprimento. Com a espiral ele passa pelo arco, girando 85°.
///
/// ⚠️ Nasce VERMELHO no motor do v1 (o controle abaixo mede o mesmo par pelo lerp).
///
/// ⚠️ **170°, não 180°, e a razão é o produto:** a meia-volta EXATA é a singularidade em
/// que o sentido do giro não existe (ver o gate seguinte). Um fixture ali mede a moeda que
/// o `f32` jogou, não a ferramenta.
#[test]
fn a_limb_that_turns_almost_half_a_turn_keeps_its_length_instead_of_collapsing() {
    let (a, shoulder) = (arm(), Vec2::ZERO);
    let b = pose(&a, shoulder, 170.0, 1.0);

    let m = StrokeMotion::fit(&a, &b);
    let mid: Vec<Vec2> = a.iter().zip(&b).map(|(&p, &q)| at(m, p, q, 0.5)).collect();

    let (la, lm) = (len(&a), len(&mid));
    assert!(
        (lm - la).abs() < la * 0.01,
        "o braço encolheu no meio do caminho: {lm:.2} de {la:.2}"
    );
    // E girou de fato 85° — a ponta está no arco, não na corda.
    let tip = *mid.last().expect("não-vazio");
    let want = Vec2::new(
        100.0 * 85f32.to_radians().cos(),
        100.0 * 85f32.to_radians().sin(),
    );
    assert!(
        (tip - want).length() < 1.0,
        "a ponta do meio deveria estar em {want:?} (85° de arco): {tip:?}"
    );
}

/// A referência clássica: o LERP colapsa o mesmo braço. É o CONTROLE do gate acima — sem
/// ele, "manteve o comprimento" poderia ser verdade por acaso do fixture.
#[test]
fn the_control_the_plain_lerp_does_collapse_that_limb() {
    let a = arm();
    let b = pose(&a, Vec2::ZERO, 170.0, 1.0);
    let mid: Vec<Vec2> = a.iter().zip(&b).map(|(&p, &q)| p + (q - p) * 0.5).collect();
    assert!(
        len(&mid) < len(&a) * 0.10,
        "o lerp deveria colapsar (é o defeito que a espiral corrige): {:.2} de {:.2}",
        len(&mid),
        len(&a)
    );
}

/// **A meia-volta EXATA é ambígua, e nenhuma ferramenta resolve isso** — girar 180° para a
/// esquerda e para a direita levam à MESMA pose, então não há informação no par que diga
/// qual. Aqui o desempate acabou saindo do último bit do `π` de `f32` (o fixture de 180°
/// nasceu vermelho apontando a ponta em `(0, −100)`), e está certo assim.
///
/// O que o gate exige é o que de fato importa: **mesmo no caso ambíguo o braço não
/// colapsa** — ele escolhe um lado e percorre o arco inteiro. (O lerp, no mesmo par, some.)
/// Escolher o lado é trabalho do artista, e é a razão de o BetweenIT ter correção manual.
#[test]
fn at_exactly_half_a_turn_the_direction_is_a_coin_flip_but_the_limb_still_never_collapses() {
    let a = arm();
    let b = pose(&a, Vec2::ZERO, 180.0, 1.0);
    let m = StrokeMotion::fit(&a, &b);
    let mid: Vec<Vec2> = a.iter().zip(&b).map(|(&p, &q)| at(m, p, q, 0.5)).collect();
    assert!(
        (len(&mid) - len(&a)).abs() < len(&a) * 0.01,
        "colapsou no caso ambíguo: {:.2} de {:.2}",
        len(&mid),
        len(&a)
    );
    // A ponta está no eixo Y — de um lado OU do outro, e o gate não escolhe qual.
    let tip = *mid.last().expect("não-vazio");
    assert!(
        tip.x.abs() < 1.0 && (tip.y.abs() - 100.0).abs() < 1.0,
        "a ponta deveria estar em (0, ±100): {tip:?}"
    );
}

/// **Translação pura é o caminho do v1, AO BIT.** Não é "quase igual": a variante `Lerp`
/// devolve o próprio ponto e a soma do chamador vira `a + (b−a)·u`, a mesma expressão de
/// antes. É a regressão que protege toda a arte que já existe.
#[test]
fn a_pure_translation_is_bit_identical_to_the_old_lerp() {
    let a: Vec<Vec2> = (0..7)
        .map(|i| Vec2::new(i as f32 * 13.7, (i * i) as f32 * 0.31))
        .collect();
    let t = Vec2::new(37.25, -11.5);
    let b: Vec<Vec2> = a.iter().map(|p| *p + t).collect();

    let m = StrokeMotion::fit(&a, &b);
    assert_eq!(m, StrokeMotion::Lerp, "uma translação não tem ponto fixo");
    for u in [0.0, 0.125, 0.25, 1.0 / 3.0, 0.5, 0.7, 1.0, 1.5, -0.5] {
        for (&p, &q) in a.iter().zip(&b) {
            let old = p + (q - p) * u; // a expressão do v1
            assert_eq!(at(m, p, q, u).to_array(), old.to_array(), "u={u}");
        }
    }
}

/// Os extremos fecham: `u=0` é A e `u=1` é B (dentro do épsilon de `f32`, porque a
/// espiral é aritmética e não uma cópia).
#[test]
fn the_endpoints_close_on_a_and_b() {
    let a: Vec<Vec2> = (0..=8).map(|i| Vec2::new(i as f32 * 7.0, 3.0)).collect();
    let b = pose(&a, Vec2::new(20.0, 5.0), 63.0, 1.7);
    let m = StrokeMotion::fit(&a, &b);
    for (&p, &q) in a.iter().zip(&b) {
        assert!((at(m, p, q, 0.0) - p).length() < 1e-3, "u=0 não é A");
        assert!((at(m, p, q, 1.0) - q).length() < 1e-3, "u=1 não é B");
    }
}

/// **O ponto fixo não se move** — é o que faz o movimento parecer uma articulação em vez
/// de um deslizamento. Um braço preso no ombro tem o ombro parado em TODO inbetween.
#[test]
fn the_pivot_stays_put_through_the_whole_arc() {
    let shoulder = Vec2::new(-4.0, 9.0);
    let a: Vec<Vec2> = (0..=6)
        .map(|i| shoulder + Vec2::new(i as f32 * 9.0, 0.0))
        .collect();
    let b = pose(&a, shoulder, 100.0, 1.4);
    let m = StrokeMotion::fit(&a, &b);
    let StrokeMotion::Spiral { fixed, .. } = m else {
        panic!("um giro em torno do ombro tem ponto fixo");
    };
    assert!(
        (fixed - shoulder).length() < 1e-2,
        "o pivô saiu do ombro: {fixed:?} × {shoulder:?}"
    );
    for u in [0.0, 0.2, 0.5, 0.8, 1.0] {
        // O ponto ancorado NO pivô é o primeiro do traço; ele não pode viajar.
        let p = at(m, a[0], b[0], u);
        assert!(
            (p - shoulder).length() < 1e-2,
            "o pivô andou em u={u}: {p:?}"
        );
    }
}

/// A escala é interpolada **geometricamente** (`σᵗ`), não linearmente: dobrar de tamanho
/// passa por `√2` no meio, não por `1.5`. É o que faz um zoom parecer constante.
#[test]
fn the_scale_grows_geometrically_not_linearly() {
    let a: Vec<Vec2> = (0..=4).map(|i| Vec2::new(i as f32 * 10.0, 0.0)).collect();
    let b = pose(&a, Vec2::new(0.0, 0.0), 30.0, 4.0); // 4× com giro (para haver pivô)
    let m = StrokeMotion::fit(&a, &b);
    let mid: Vec<Vec2> = a.iter().zip(&b).map(|(&p, &q)| at(m, p, q, 0.5)).collect();
    let k = len(&mid) / len(&a);
    assert!(
        (k - 2.0).abs() < 0.05,
        "√4 = 2, e o meio deu {k:.3} (linear daria 2,5)"
    );
}

/// Um traço de UM ponto não define rotação nem escala — e não pode entrar em pânico.
#[test]
fn a_degenerate_stroke_falls_back_instead_of_panicking() {
    assert_eq!(StrokeMotion::fit(&[], &[]), StrokeMotion::Lerp);
    let p = [Vec2::new(1.0, 2.0)];
    assert_eq!(StrokeMotion::fit(&p, &p), StrokeMotion::Lerp);
    // Todos os pontos coincidentes (norm_a = 0).
    let same = [Vec2::new(3.0, 3.0); 5];
    assert_eq!(StrokeMotion::fit(&same, &same), StrokeMotion::Lerp);
    // B colapsado num ponto: σ → 0.
    let a: Vec<Vec2> = (0..5).map(|i| Vec2::new(i as f32, 0.0)).collect();
    assert_eq!(
        StrokeMotion::fit(&a, &[Vec2::new(9.0, 9.0); 5]),
        StrokeMotion::Lerp
    );
}

/// **A régua do `DET_MIN`.** Varre o ângulo de uma rotação quase-degenerada e mede o erro
/// da espiral em `f32` contra a mesma espiral avaliada em `f64` — é o cancelamento do
/// `(p − F) … + F` que decide o piso, e a tabela mostra onde ele explode.
///
/// `cargo test -p ph2d-flip --release the_spiral_ruler -- --ignored --nocapture`
#[test]
#[ignore = "régua: imprime a tabela que justifica DET_MIN"]
fn the_spiral_ruler() {
    let a: Vec<Vec2> = (0..=10).map(|i| Vec2::new(i as f32 * 10.0, 0.0)).collect();
    println!("\n  θ (rad)      det        |erro f32−f64|   arco−corda");
    for e in 1..=8 {
        let theta = 10f32.powi(-e);
        let b = pose(&a, Vec2::new(500.0, 300.0), theta.to_degrees(), 1.0);
        let det = 1.0 - 2.0 * f64::from(theta).cos() + 1.0;
        // A referência: a MESMA espiral, mas avaliada inteiramente em f64.
        let m = StrokeMotion::fit(&a, &b);
        let mut worst = 0.0f64;
        for (i, (&p, &q)) in a.iter().zip(&b).enumerate() {
            let got = m.advance(p, 0.5) + m.residual(p, q) * 0.5;
            let want = reference_f64(&a, &b, i, 0.5);
            worst = worst.max((f64::from(got.x) - want.0).hypot(f64::from(got.y) - want.1));
        }
        let d = (b[10] - a[10]).length();
        println!(
            "  1e-{e}        {det:.3e}   {worst:.3e}        {:.3e}   {}",
            f64::from(d * theta / 8.0),
            if det < DET_MIN { "→ Lerp" } else { "" }
        );
    }
    println!("\n  DET_MIN = {DET_MIN:e}\n");
}

/// A espiral inteira em `f64` — a referência da régua (**não** é o produto: é o mesmo
/// algoritmo sem o degrau de precisão, que é justamente o que a régua quer medir).
fn reference_f64(a: &[Vec2], b: &[Vec2], i: usize, u: f64) -> (f64, f64) {
    let n = a.len();
    let inv = 1.0 / n as f64;
    let (mut ca, mut cb) = ((0.0, 0.0), (0.0, 0.0));
    for k in 0..n {
        ca.0 += f64::from(a[k].x) * inv;
        ca.1 += f64::from(a[k].y) * inv;
        cb.0 += f64::from(b[k].x) * inv;
        cb.1 += f64::from(b[k].y) * inv;
    }
    let (mut dot, mut cross, mut na) = (0.0, 0.0, 0.0);
    for k in 0..n {
        let (ax, ay) = (f64::from(a[k].x) - ca.0, f64::from(a[k].y) - ca.1);
        let (bx, by) = (f64::from(b[k].x) - cb.0, f64::from(b[k].y) - cb.1);
        dot += ax * bx + ay * by;
        cross += ax * by - ay * bx;
        na += ax * ax + ay * ay;
    }
    let theta = libm::atan2(cross, dot);
    let sc = (dot * dot + cross * cross).sqrt() / na;
    let (sin, cos) = libm::sincos(theta);
    let (m00, m01, m10, m11) = (sc * cos, -sc * sin, sc * sin, sc * cos);
    let cx = cb.0 - (m00 * ca.0 + m01 * ca.1);
    let cy = cb.1 - (m10 * ca.0 + m11 * ca.1);
    let det = 1.0 - 2.0 * sc * cos + sc * sc;
    let (fx, fy) = (
        ((1.0 - m11) * cx + m01 * cy) / det,
        (m10 * cx + (1.0 - m00) * cy) / det,
    );
    // S(p) e P(u) saem da MESMA expressão, variando só o expoente — como no produto.
    let spiral = |uu: f64| {
        let (s2, c2) = libm::sincos(theta * uu);
        let k = libm::pow(sc, uu);
        let (dx, dy) = (f64::from(a[i].x) - fx, f64::from(a[i].y) - fy);
        (fx + k * (c2 * dx - s2 * dy), fy + k * (s2 * dx + c2 * dy))
    };
    let s1 = spiral(1.0);
    let (rx, ry) = (f64::from(b[i].x) - s1.0, f64::from(b[i].y) - s1.1);
    let su = spiral(u);
    (su.0 + rx * u, su.1 + ry * u)
}
