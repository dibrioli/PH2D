//! Gates do **RASTRO RE-COZIDO** (`Source: Resampled`) — a cauda montada de um
//! leque de tempo em vez de um ring (doc 89, folha 07, `SUPERAR:` S1).
//!
//! ⚠️ **A lei está numa função só, e é ela que estes gates falsificam:**
//! [`echo_offsets`] devolve o deslocamento de cada geração, e os dois leitores —
//! o [`time_fans`], que o converte em mapas de tempo, e o `eval`, que dele tira a
//! IDADE — não podem discordar. Um gate que medisse só o desenho não veria a
//! discordância: ele veria uma cauda plausível, pousada nos instantes errados.

use super::*;

/// ⭐ **A REDUÇÃO.** Sem eco para a frente, a escada é exactamente a que o ring
/// produz: `-(L−1)s, …, −s, 0`, cauda primeiro e cabeça por último.
#[test]
fn with_no_forward_echo_the_ladder_is_the_one_the_ring_makes() {
    for length in [1usize, 2, 3, 8, 32] {
        for spacing in [1usize, 2, 5] {
            let got = echo_offsets(length, spacing, 0);
            let want: Vec<i32> = (1..length)
                .rev()
                .map(|k| -((k * spacing) as i32))
                .chain(std::iter::once(0))
                .collect();
            assert_eq!(got, want, "L={length} s={spacing}");
        }
    }
}

/// ⭐ **O ECO PARA A FRENTE** — o que a AE faz por RE-RENDERIZAR e um ring não
/// pode conter. Com `forward = f`, `f` das gerações são futuras e o resto é
/// passado; a cabeça continua a ser a última linha desenhada.
#[test]
fn forward_echoes_land_ahead_of_the_head() {
    let got = echo_offsets(5, 2, 2);
    // 4 ecos: 2 à frente (+2, +4) e 2 atrás (−2, −4), por idade decrescente com
    // o passado à frente no empate, e a cabeça no fim.
    assert_eq!(got, vec![-4, 4, -2, 2, 0]);
    assert_eq!(
        got.last(),
        Some(&0),
        "a cabeca viva desenha por ULTIMO, sempre"
    );
    // Todas as gerações, o que quer que o Forward faça.
    for f in 0..=4 {
        assert_eq!(echo_offsets(5, 2, f).len(), 5, "forward={f}");
    }
}

/// `forward` maior que os ecos que existem vira "todos à frente" — nunca um erro,
/// nunca uma cauda mais curta.
#[test]
fn a_forward_beyond_the_tail_simply_puts_every_echo_ahead() {
    let all = echo_offsets(4, 1, 99);
    assert_eq!(all.len(), 4);
    assert!(
        all.iter().all(|&g| g >= 0),
        "com forward saturado nada pode ficar atras: {all:?}"
    );
    assert_eq!(forward_of(99.0, 4), 3, "clampado aos ecos que ha'");
    assert_eq!(forward_of(-3.0, 4), 0, "um negativo e' zero, nao um panico");
    assert_eq!(forward_of(f32::NAN, 4), 0);
}

/// ⚠️ **O vão dos alvos NÃO segue o Forward.** `Fade` é sempre *"o que um eco a
/// `L−1` passos parece"*, de que lado for — senão o número mudaria de sentido ao
/// arrastar o Forward, que é a doença que este repo chama de bug de design.
#[test]
fn the_authored_span_does_not_move_with_the_forward_split() {
    let span = authored_span(9, 3);
    assert_eq!(span, 24, "(9−1)·3");
    for f in 0..=8 {
        let offs = echo_offsets(9, 3, f);
        assert_eq!(offs.len(), 9);
        assert!(
            offs.iter().all(|&g| g.unsigned_abs() <= span),
            "forward={f}: uma geracao passou do vao autorado"
        );
    }
}

/// ⭐ **`at_age(span, 1)` É `per_tick(span)`** — a generalização contém o caso
/// antigo, expressão por expressão. É isso que faz existir uma lei e não duas.
#[test]
fn the_absolute_decay_at_one_tick_is_the_per_tick_rate() {
    let d = Decay {
        alpha_max: 1.0,
        fade: 0.1,
        shrink: 0.65,
        hue_shift: 90.0,
        saturation: 0.4,
        spin: 45.0,
    };
    for span in [1u32, 2, 7, 31] {
        let a = d.per_tick(span);
        let b = d.at_age(span, 1);
        assert_eq!(a.fade.to_bits(), b.fade.to_bits(), "span={span} fade");
        assert_eq!(a.shrink.to_bits(), b.shrink.to_bits(), "span={span} shrink");
        assert_eq!(
            a.saturation.to_bits(),
            b.saturation.to_bits(),
            "span={span} sat"
        );
        assert_eq!(a.hue_shift.to_bits(), b.hue_shift.to_bits());
        assert_eq!(a.spin.to_bits(), b.spin.to_bits());
    }
}

/// E no VÃO INTEIRO o alvo é atingido — a propriedade que o slider promete.
#[test]
fn the_absolute_decay_reaches_the_authored_target_at_the_span() {
    let d = Decay {
        alpha_max: 1.0,
        fade: 0.2,
        shrink: 0.5,
        hue_shift: 120.0,
        saturation: 0.3,
        spin: 90.0,
    };
    let span = 16u32;
    let end = d.at_age(span, span);
    assert!((end.fade - 0.2).abs() < 1e-5, "{}", end.fade);
    assert!((end.shrink - 0.5).abs() < 1e-5, "{}", end.shrink);
    assert!((end.saturation - 0.3).abs() < 1e-5, "{}", end.saturation);
    assert!((end.hue_shift - 120.0).abs() < 1e-3, "{}", end.hue_shift);
    assert!((end.spin - 90.0).abs() < 1e-3, "{}", end.spin);
    // Idade zero é a identidade — a cabeça viva não desbota.
    let head = d.at_age(span, 0);
    assert_eq!(head.fade, 1.0);
    assert_eq!(head.shrink, 1.0);
    assert_eq!(head.spin, 0.0);
}

/// **O leque VIRA a cauda**: cada fatia entra uma vez, com a idade da sua
/// geração escrita na coluna que o renderer lê, e a cabeça por último.
#[test]
fn the_resampled_tail_is_one_row_per_slice_aged_by_its_generation() {
    let at = |x: f32| Stream::new(1).with("P", Column::Vec2(vec![[x, 0.0]]));
    let (a, b, c) = (at(-2.0), at(-1.0), at(0.0));
    let fan = [&a, &b, &c];
    let offsets = echo_offsets(3, 1, 0);
    assert_eq!(offsets, vec![-2, -1, 0]);
    let out = step_resampled(&fan, &offsets, Decay::new(0.25, 1.0), authored_span(3, 1));

    assert_eq!(out.count(), 3, "uma linha por fatia");
    let Some(Column::Scalar(age)) = out.get(AGE) else {
        panic!("a coluna de idade tem de existir")
    };
    assert_eq!(
        age.as_slice(),
        &[2.0, 1.0, 0.0],
        "cauda primeiro, cabeca no fim"
    );
    let Some(Column::Vec2(p)) = out.get("P") else {
        panic!("P")
    };
    assert_eq!(p.as_slice(), &[[-2.0, 0.0], [-1.0, 0.0], [0.0, 0.0]]);

    // O `fade` autorado é atingido na PONTA, e a cabeça fica intocada.
    let Some(Column::Vec2(_)) = out.get("P") else {
        unreachable!()
    };
    let alpha = match out.get("tint") {
        Some(Column::Vec4(v)) => v.iter().map(|c| c[3]).collect::<Vec<_>>(),
        _ => panic!("o rastro materializa `tint`"),
    };
    assert!((alpha[0] - 0.25).abs() < 1e-5, "a ponta: {}", alpha[0]);
    assert_eq!(alpha[2], 1.0, "a cabeca viva nao desbota");
}

/// ⚠️ **Uma fatia VAZIA é pulada, não desenhada** — o instante `t − k·s` de antes
/// do começo de uma cena não tem elemento nenhum, e uma linha fantasma ali seria
/// um eco pousado na origem.
#[test]
fn an_empty_slice_draws_nothing_instead_of_landing_at_the_origin() {
    let full = Stream::new(1).with("P", Column::Vec2(vec![[3.0, 0.0]]));
    let empty = Stream::empty();
    let fan = [&empty, &full, &full];
    let out = step_resampled(&fan, &echo_offsets(3, 1, 0), Decay::NEUTRAL, 2);
    assert_eq!(out.count(), 2, "a fatia vazia nao vira linha");
}
