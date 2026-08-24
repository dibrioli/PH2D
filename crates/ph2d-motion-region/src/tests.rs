//! Os gates do domínio — a lei que quatro geradores passam a partilhar.

use super::*;

/// Um par de uniformes decorrelacionados, sem depender do hash de nó nenhum.
fn draws(n: usize) -> impl Iterator<Item = (f32, f32)> {
    (0..n).map(|i| {
        let u = ((i as u32).wrapping_mul(2_654_435_761) >> 8) as f32 / (1u32 << 24) as f32;
        let v = ((i as u32).wrapping_mul(40_503) >> 8) as f32 / (1u32 << 24) as f32;
        (u, v)
    })
}

/// ⭐ **O RETÂNGULO É O DE SEMPRE, AO BIT** — a expressão que os nós tinham
/// escrita à mão, byte a byte.
///
/// ⚠️ Este é o gate que torna a lei adoptável: se o default movesse um ULP, toda
/// cena salva mudaria de layout no dia em que o param nascesse.
#[test]
fn the_rect_draw_is_bit_for_bit_the_hand_written_one() {
    let (w, h) = (4.0_f32, 2.5_f32);
    let d = Region::of(0.0, w, h, 0.0);
    for (u, v) in draws(512) {
        let got = d.sample(u, v);
        let want = [(u - 0.5) * w, (v - 0.5) * h];
        assert_eq!(
            got.map(f32::to_bits),
            want.map(f32::to_bits),
            "u={u} v={v}: {got:?} contra {want:?}"
        );
    }
}

/// E o retângulo aceita a caixa inteira — um reticulado não perde um ponto.
#[test]
fn the_rect_cut_removes_nothing() {
    let d = Region::of(0.0, 4.0, 4.0, 0.0);
    for r in 0..9 {
        for c in 0..9 {
            let p = [(c as f32 - 4.0) * 0.5, (r as f32 - 4.0) * 0.5];
            assert!(d.contains(p), "{p:?} caiu fora da propria caixa");
        }
    }
}

/// ⭐ **O SORTEIO NO DISCO É UNIFORME POR ÁREA** — metade da área está dentro de
/// `R/√2`, e é lá que metade dos pontos tem de cair.
///
/// ⚠️ **A régua é a fração de ÁREA e não a de raio**, que é precisamente o erro que
/// um `r = R·u` comete: ele poria ~71% dos pontos no disco interior.
#[test]
fn the_disc_draw_is_area_uniform_not_radius_uniform() {
    let d = Region::of(1.0, 4.0, 4.0, 0.0);
    let n = 20_000;
    let inside = draws(n)
        .filter(|&(u, v)| {
            let p = d.sample(u, v);
            d.radial(p) <= std::f32::consts::FRAC_1_SQRT_2
        })
        .count();
    let frac = inside as f32 / n as f32;
    assert!(
        (frac - 0.5).abs() < 0.03,
        "metade da AREA tem de levar metade dos pontos: {frac:.4}"
    );
    // E o controlo: nenhum ponto sai do disco.
    for (u, v) in draws(4_000) {
        assert!(d.contains(d.sample(u, v)), "um dardo saiu do disco");
    }
}

/// ⭐ **O ANEL NUNCA PÕE UM PONTO NO BURACO** — e o buraco tem o tamanho pedido.
#[test]
fn the_ring_draw_never_lands_in_the_hole() {
    for inner in [0.25_f32, 0.6, 0.9] {
        let d = Region::of(2.0, 6.0, 6.0, inner);
        let mut closest = f32::MAX;
        for (u, v) in draws(8_000) {
            let r = d.radial(d.sample(u, v));
            closest = closest.min(r);
            assert!(r <= 1.0 + 1e-4, "saiu do anel: r={r}");
        }
        assert!(
            closest >= inner - 1e-3,
            "inner={inner}: o mais perto caiu em {closest:.4}"
        );
        // E ele encosta no buraco — senão a banda estaria a ser sorteada torta.
        assert!(
            closest < inner + 0.05,
            "inner={inner}: o sorteio nao chega a' borda de dentro ({closest:.4})"
        );
    }
}

/// ⭐⭐ **O QUE O SORTEIO PRODUZ, O CORTE ACEITA** — a lei que liga as duas
/// perguntas desta crate, e a que o seno aproximado quebrava.
///
/// ⚠️ Com `(cos, sin)` parabólicos crus `c² + s²` chega a `1,004`: um dardo de raio
/// cheio aterrava `0,2%` fora, e o `contains` recusava um ponto que o `sample`
/// tinha acabado de produzir. *Uma aproximação boa para um ângulo é errada para um
/// raio* — ver `unit_cycles`.
#[test]
fn everything_the_draw_produces_the_cut_accepts() {
    for d in [
        Region::of(0.0, 4.0, 2.0, 0.0),
        Region::of(1.0, 4.0, 4.0, 0.0),
        Region::of(1.0, 7.0, 1.5, 0.0),
        Region::of(2.0, 5.0, 5.0, 0.3),
        Region::of(2.0, 5.0, 5.0, 0.95),
    ] {
        for (u, v) in draws(4_000) {
            let p = d.sample(u, v);
            assert!(
                d.contains(p),
                "{d:?} sorteou {p:?} (r={}) e o proprio corte recusou",
                d.radial(p)
            );
        }
    }
}

/// ⭐ **UMA CASCA É UM ANEL** — a afirmação que apagou o param `fill`.
///
/// Com `inner` alto a banda fica fina: a área que sobra é `1 − inner²`, e é ela que
/// decide quantos pontos de uma rede sobrevivem ao corte.
#[test]
fn a_shell_is_a_ring_with_a_big_hole() {
    let solid = Region::of(1.0, 4.0, 4.0, 0.0);
    let shell = Region::of(2.0, 4.0, 4.0, 0.9);
    let lattice: Vec<[f32; 2]> = (0..41)
        .flat_map(|r| (0..41).map(move |c| [(c as f32 - 20.0) * 0.1, (r as f32 - 20.0) * 0.1]))
        .collect();
    let kept = |d: &Region| lattice.iter().filter(|p| d.contains(**p)).count();
    let (a, b) = (kept(&solid), kept(&shell));
    assert!(a > 0 && b > 0, "as duas guardam alguma coisa");
    // A banda de 10% do raio guarda ~1 − 0,81 = 19% do que o disco guarda.
    let frac = b as f32 / a as f32;
    assert!(
        (frac - 0.19).abs() < 0.05,
        "a casca tinha de guardar ~19% do disco: {frac:.4} ({b} de {a})"
    );
}

/// ⭐ **NUM ANEL O CORAÇÃO É O MEIO DA BANDA** — as duas fronteiras leem `0` e o
/// meio lê `1`, enquanto o disco tem uma fronteira só.
#[test]
fn the_ring_has_two_edges_and_the_disc_has_one() {
    let ring = Region::of(2.0, 4.0, 4.0, 0.5);
    // raio 0,5 e raio 1,0 são as duas bordas; 0,75 é o meio.
    let at = |r: f32| ring.depth([r * 2.0, 0.0]);
    assert!(at(0.5).abs() < 1e-5, "borda de dentro: {}", at(0.5));
    assert!(at(1.0).abs() < 1e-5, "borda de fora: {}", at(1.0));
    assert!((at(0.75) - 1.0).abs() < 1e-5, "o meio: {}", at(0.75));
    // E o buraco lê NEGATIVO — não `0`, que é a borda.
    assert!(at(0.1) < 0.0, "o buraco tinha de ler fora: {}", at(0.1));

    let disc = Region::of(1.0, 4.0, 4.0, 0.0);
    assert!(
        (disc.depth([0.0, 0.0]) - 1.0).abs() < 1e-6,
        "o centro do disco"
    );
    assert!(disc.depth([2.0, 0.0]).abs() < 1e-5, "a borda do disco");
}

/// ⚠️ **`falloff = 0` devolve `1,0` AO BIT** — o default reduz ao nó que shipava.
#[test]
fn a_zero_falloff_is_exactly_one_everywhere() {
    let d = Region::of(2.0, 3.0, 5.0, 0.4);
    for (u, v) in draws(1_000) {
        let p = d.sample(u, v);
        assert_eq!(d.density(p, 0.0).to_bits(), 1.0_f32.to_bits());
        // E fora da região também — um consumidor pode perguntar em qualquer sítio.
        assert_eq!(
            d.density([u * 9.0, v * 9.0], 0.0).to_bits(),
            1.0_f32.to_bits()
        );
    }
}

/// E com `falloff = 1` a densidade desce do coração até ao piso, sem o furar.
#[test]
fn a_full_falloff_grades_from_the_core_to_the_floor() {
    let d = Region::of(1.0, 4.0, 4.0, 0.0);
    assert!(
        (d.density([0.0, 0.0], 1.0) - 1.0).abs() < 1e-6,
        "o coracao vale 1"
    );
    let edge = d.density([2.0, 0.0], 1.0);
    assert!(
        (edge - MIN_DENSITY).abs() < 1e-5,
        "a borda tinha de encostar no piso: {edge}"
    );
    // Monotónica pelo caminho, e nunca abaixo do piso.
    let mut prev = f32::MAX;
    for k in 0..=40 {
        let v = d.density([k as f32 * 0.05, 0.0], 1.0);
        assert!(v >= MIN_DENSITY - 1e-6, "furou o piso em {k}: {v}");
        assert!(v <= prev + 1e-6, "subiu ao afastar-se em {k}: {v} > {prev}");
        prev = v;
    }
}

/// ⚠️ **Um `Circle` numa caixa não-quadrada é uma ELIPSE** — a forma herda a caixa,
/// não a substitui. Os dois semi-eixos leem `1` na régua.
#[test]
fn a_circle_in_a_wide_box_is_an_ellipse() {
    let d = Region::of(1.0, 8.0, 2.0, 0.0);
    assert!((d.radial([4.0, 0.0]) - 1.0).abs() < 1e-5, "o semi-eixo x");
    assert!((d.radial([0.0, 1.0]) - 1.0).abs() < 1e-5, "o semi-eixo y");
    assert!(
        !d.contains([4.0, 1.0]),
        "a esquina da caixa fica FORA da elipse"
    );
}

/// ⚠️ **Params doentes caem no retângulo, sem pânico** — eles chegam de um fio.
#[test]
fn a_driven_param_can_be_anything_and_the_answer_is_the_rectangle() {
    for bad in [f32::NAN, f32::INFINITY, -7.0, 99.0] {
        let d = Region::of(bad, 4.0, 4.0, bad);
        assert!(d.is_rect() || d.contains([0.0, 0.0]), "bad={bad}");
        let p = d.sample(0.3, 0.7);
        assert!(p[0].is_finite() && p[1].is_finite(), "bad={bad} deu {p:?}");
    }
    // Extensão zero: a caixa não tem interior, e a resposta é a linha — não o vazio.
    let flat = Region::of(0.0, 4.0, 0.0, 0.0);
    assert!(
        flat.contains([1.0, 0.0]),
        "uma caixa achatada ainda tem a linha"
    );
    // NaN na extensão vira zero, não `∞`.
    let nan = Region::of(1.0, f32::NAN, 4.0, 0.0);
    assert!(nan.sample(0.5, 0.5).iter().all(|v| v.is_finite()));
}

/// O `inner` é aparado abaixo de `1`: um anel de banda zero dividiria por zero no
/// sorteio por área.
#[test]
fn the_hole_can_never_swallow_the_band() {
    let d = Region::of(2.0, 4.0, 4.0, 1.0);
    for (u, v) in draws(256) {
        let p = d.sample(u, v);
        assert!(p[0].is_finite() && p[1].is_finite(), "u={u}: {p:?}");
        assert!(d.contains(p), "o sorteio saiu da propria regiao: {p:?}");
    }
}

/// ⭐⭐ **NENHUM PONTO DE REDE CAI FORA DA CAIXA CONSTRUÍDA DA PRÓPRIA EXTENSÃO** — e é
/// isto que torna o ramo do `Rect` no [`carve`] uma poupança de custo, e não uma cerca.
///
/// ⚠️ **A pergunta é de ARREDONDAMENTO, não de álgebra.** O ponto de fora está em
/// `((n−1) − (n−1)/2)·g` e a meia-extensão em `((n−1)·g)/2`: mesma quantidade real, duas
/// árvores de operações. Se elas divergissem por um ULP, um `contains` incondicional
/// comeria a coluna de fora de toda grade — em silêncio.
///
/// ⚠️ **Este gate nasceu de uma MUTAÇÃO SOBREVIVENTE:** apagar o ramo não moveu um bit,
/// o que provou que a afirmação de então (*«o ramo compra a identidade»*) era sobre
/// nada. O que se pode afirmar é isto, e mede-se.
#[test]
fn no_lattice_point_ever_falls_outside_its_own_box() {
    let mut checked = 0_u32;
    for cols in 3..60_usize {
        for gi in 1..400_u32 {
            let gap = gi as f32 * 0.01;
            let cx = (cols as f32 - 1.0) * 0.5;
            let outer = (cols as f32 - 1.0 - cx) * gap;
            let d = Region::of(0.0, (cols as f32 - 1.0) * gap, 1.0, 0.0);
            assert!(
                d.contains([outer, 0.0]),
                "cols={cols} gap={gap}: o ponto de fora ({outer}) caiu fora da propria caixa"
            );
            checked += 1;
        }
    }
    assert_eq!(
        checked, 22_743,
        "a varredura tem de cobrir o que a nota diz"
    );
}

/// A escada dos rótulos e a dos números andam juntas.
#[test]
fn every_shape_number_has_a_label() {
    assert_eq!(SHAPE_LABELS.len(), 3);
    for (i, l) in SHAPE_LABELS.iter().enumerate() {
        assert!(!l.is_empty(), "rotulo {i} vazio");
    }
    assert_eq!(SHAPE_RECT, 0);
    assert_eq!(SHAPE_CIRCLE, 1);
    assert_eq!(SHAPE_RING, 2);
}
