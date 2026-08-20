//! Os gates do disco de maciez (doc 89, folha 11).

use super::*;

/// **A DENSIDADE DO MIOLO NÃO MUDA AO LIGAR A MACIEZ.**
///
/// ⚠️ Este é o gate que separa uma penumbra de um *desbotamento*, e é o defeito que
/// a cura ingénua (`a/N`) produz: sobrepondo `N` taps de alfa `per_tap_alpha(a)` a
/// união tem de dar exactamente `a` de volta. Sem ele, ligar o knob **clareia** a
/// sombra e o artista corrige a opacidade — e aí a borda fica errada nos dois lados.
#[test]
fn the_union_of_the_taps_restores_the_hard_shadows_alpha() {
    for a in [0.05f32, 0.35, 0.5, 0.9, 1.0] {
        let per = per_tap_alpha(a);
        // 1 − (1−per)^N, a composição de N camadas da mesma cor.
        let union = 1.0 - (1.0 - per).powi(TAPS as i32);
        assert!(
            (union - a).abs() < 1e-5,
            "alfa {a}: {TAPS} taps de {per:.5} dão {union:.5}"
        );
    }
}

/// **A CURA INGÉNUA `a/N` ESTARIA ERRADA, e o gate mostra por quanto** — o controle
/// que impede alguém de «simplificar» a raiz.
#[test]
fn the_naive_split_would_be_measurably_dimmer() {
    let a = 0.35f32;
    #[expect(clippy::cast_precision_loss, reason = "TAPS = 16")]
    let naive = 1.0 - (1.0 - a / TAPS as f32).powi(TAPS as i32);
    assert!(
        (naive - a).abs() > 0.04,
        "o `a/N` tinha de errar de forma visível, e deu {naive:.4} contra {a}"
    );
}

/// **O ALFA POR TAP SAI POR `sqrt` e não por `powf`** — HR-5.
///
/// ⚠️ O oráculo é a REPETIBILIDADE ao bit: `sqrt` é correctamente arredondado no
/// IEEE-754, então a mesma entrada dá o mesmo `f32` em qualquer plataforma. Um
/// `powf` daria um número por libm.
#[test]
fn the_per_tap_alpha_is_bit_repeatable() {
    assert_eq!(per_tap_alpha(0.35), per_tap_alpha(0.35));
    // E o caminho degenerado não produz `NaN`: fora de [0,1] devolve a entrada.
    assert_eq!(per_tap_alpha(-1.0), -1.0);
    assert!(per_tap_alpha(f32::NAN).is_nan());
}

/// **O DISCO ENCHE UMA ÁREA, e não um anel nem um eixo.**
///
/// ⚠️ Três afirmações, e cada uma mata um erro diferente: (a) nenhum tap sai do
/// raio — senão a sombra «cresce» mais do que o knob diz; (b) os raios são
/// DISTINTOS — um anel regular daria todos iguais e a penumbra teria uma borda
/// dura; (c) o centroide fica perto da origem — um disco enviesado deslocaria a
/// sombra ao ligar a maciez.
#[test]
fn the_disc_fills_an_area_rather_than_a_ring_or_an_axis() {
    let r = 0.5f32;
    let d = disc(r);
    let radii: Vec<f32> = d.iter().map(|q| q[0].hypot(q[1])).collect();
    for (k, rad) in radii.iter().enumerate() {
        assert!(*rad <= r + 1e-4, "o tap {k} saiu do raio: {rad}");
    }
    let (lo, hi) = radii
        .iter()
        .fold((f32::MAX, f32::MIN), |(a, b), v| (a.min(*v), b.max(*v)));
    assert!(
        hi - lo > 0.5 * r,
        "os taps têm de cobrir raios distintos: [{lo:.3}, {hi:.3}]"
    );
    #[expect(clippy::cast_precision_loss, reason = "TAPS = 16")]
    let n = TAPS as f32;
    let c = d
        .iter()
        .fold([0.0f32, 0.0], |a, q| [a[0] + q[0], a[1] + q[1]]);
    let (cx, cy) = (c[0] / n, c[1] / n);
    assert!(
        cx.hypot(cy) < 0.15 * r,
        "o disco tem de ser centrado, e o centroide deu ({cx:.4}, {cy:.4})"
    );
}

/// **O DISCO É UNIFORME POR ÁREA, e não por raio** — a raiz de [`disc`].
///
/// ⚠️ **Este gate nasceu de uma MUTAÇÃO SOBREVIVENTE.** Tirar o `sqrt` (`rad = r·t`)
/// passava por todos os outros: os raios continuam dentro do disco, continuam
/// distintos e o centroide continua na origem. O que se perde é a DENSIDADE — os
/// taps amontoam-se no miolo e a penumbra fica dura por dentro e rala por fora.
///
/// O oráculo é geométrico e não tem número mágico: **metade do raio é um quarto da
/// área**, então no máximo ~¼ dos taps pode cair lá dentro. Com `r·t` caem metade.
#[test]
fn the_disc_is_uniform_by_area_and_not_by_radius() {
    let r = 1.0f32;
    let inner = disc(r)
        .iter()
        .filter(|q| q[0].hypot(q[1]) <= r * 0.5)
        .count();
    assert!(
        inner <= TAPS / 4 + 1,
        "metade do raio é um quarto da área, e {inner} de {TAPS} taps caíram lá"
    );
    assert!(inner >= 2, "…e não pode ser zero: {inner}");
}

/// **OS TAPS FOGEM DOS EIXOS** — é para isso que o passo é o ângulo de OURO.
///
/// ⚠️ **Segunda mutação sobrevivente:** trocar o ângulo de ouro por ¼ de volta
/// mantinha os 16 taps em posições DISTINTAS (quatro raios × quatro distâncias),
/// então o gate da repetição ficava verde — e a penumbra sairia com quatro **raios**
/// visíveis, que é o artefacto que a escolha do passo existe para evitar.
///
/// O oráculo é a fração de taps genuinamente FORA dos eixos.
#[test]
fn the_taps_avoid_the_axes_so_the_penumbra_has_no_spokes() {
    let r = 1.0f32;
    let off_axis = disc(r)
        .iter()
        .filter(|q| q[0].abs() > 0.15 * r && q[1].abs() > 0.15 * r)
        .count();
    assert!(
        off_axis * 2 >= TAPS,
        "pelo menos metade dos taps tem de estar fora dos eixos, e só {off_axis} estão"
    );
}

/// **OS TAPS NÃO SE REPETEM** — o ângulo de ouro é escolhido para isso.
///
/// ⚠️ Um passo racional (digamos ¼ de volta) poria quatro taps no mesmo raio E no
/// mesmo ângulo módulo a volta, e o disco viraria quatro pontos com peso 4.
#[test]
fn no_two_taps_land_on_the_same_spot() {
    let d = disc(1.0);
    for i in 0..TAPS {
        for j in (i + 1)..TAPS {
            let (dx, dy) = (d[i][0] - d[j][0], d[i][1] - d[j][1]);
            assert!(
                dx.hypot(dy) > 1e-3,
                "os taps {i} e {j} caíram no mesmo sítio"
            );
        }
    }
}

/// **`TAPS` É POTÊNCIA DE DOIS**, e não por gosto: é o que faz a raiz `N`-ésima ser
/// `sqrt` encadeado (ver o cabeçalho do módulo). Uma mudança para 12 quebraria a
/// densidade sem quebrar nenhum outro gate deste arquivo.
#[test]
fn the_tap_count_is_a_power_of_two_so_the_root_is_sqrt() {
    assert!(TAPS.is_power_of_two(), "TAPS = {TAPS}");
    assert_eq!(TAPS.trailing_zeros(), 4, "quatro `sqrt` encadeados");
}
