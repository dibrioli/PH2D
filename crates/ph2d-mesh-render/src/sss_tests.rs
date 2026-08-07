//! Os gates da **tabela** do SSS pré-integrado.
//!
//! ⚠️ O oráculo é a **PROPRIEDADE FÍSICA**, nunca a fórmula: cada um destes
//! afirma algo que se pode dizer sobre luz entrando em carne, e nenhum re-escreve
//! a integral para comparar com ela mesma. O que o passe DESENHA é oráculo de
//! GPU e vive em `tests/gpu_render.rs`.

use super::*;

/// **Superfície PLANA é o difuso de sempre.** Sem curvatura não há vizinhança
/// mais clara de onde a luz possa vir, então a pré-integração tem de reduzir
/// literalmente a `max(N·L, 0)` — e é isso que faz `scatter = 0` não mover um
/// pixel.
#[test]
fn a_flat_surface_is_plain_lambert() {
    for i in 0..21 {
        let n_dot_l = -1.0 + i as f32 * 0.1;
        let d = integrate(n_dot_l, 0.0);
        let want = n_dot_l.max(0.0);
        for c in d {
            assert!(
                (c - want).abs() < 1e-6,
                "N·L {n_dot_l}: {c} deveria ser o lambert {want}"
            );
        }
    }
}

/// **A LUZ VAZA PARA O LADO ESCURO** — a razão de o efeito existir.
///
/// Exatamente no terminador (`N·L = 0`) o difuso de Lambert é zero. Com
/// espalhamento ele **não** é: metade da vizinhança está iluminada, e a luz que
/// entrou ali sai aqui. Este é o gate que morre se alguém trocar a integral por
/// um `max(cos, 0)` disfarçado.
#[test]
fn light_bleeds_past_the_terminator() {
    // MEDIDO em `t = 1`: R 0,070 · G 0,044 · B 0,035, contra um lambert de
    // ZERO. A barra é 0,03 — bem acima do degrau de 8 bits (0,0039), e bem
    // abaixo do medido, para não ser um gate que falha por afinação do perfil.
    // ⚠️ A primeira barra que escrevi era 0,1, sem medir nada.
    let d = integrate(0.0, 1.0);
    assert!(
        d[0] > 0.03,
        "no terminador o vermelho deveria vazar (medido 0,070), e deu {}",
        d[0]
    );
    // E além do terminador, no lado escuro, ainda sobra alguma.
    let past = integrate(-0.2, 1.0);
    assert!(
        past[0] > 0.0,
        "logo depois do terminador o vermelho deveria sobrar, e deu {}",
        past[0]
    );
}

/// **O VERMELHO VAZA MAIS QUE O AZUL** — e é só isso que faz pele parecer pele.
///
/// ⚠️ O oráculo é a ORDEM dos três canais, não um valor: ele sobrevive a
/// qualquer re-normalização do perfil, e morre no instante em que alguém
/// colapsar as seis gaussianas numa média (o que pareceria uma simplificação
/// inocente).
#[test]
fn red_travels_further_than_blue() {
    let d = integrate(0.0, 1.0);
    assert!(
        d[0] > d[1] && d[1] > d[2],
        "no terminador esperava vermelho > verde > azul, e deu {d:?}"
    );
}

/// **Quanto mais curvo, mais macio** — o terminador de uma superfície fina
/// espalha mais que o de uma grossa. Monotonicidade, que é o que um artista
/// espera de um slider.
#[test]
fn a_tighter_curve_bleeds_more() {
    let mut last = 0.0;
    for i in 0..8 {
        let t = i as f32 * 0.25;
        let d = integrate(0.0, t)[0];
        assert!(
            d >= last - 1e-6,
            "t {t}: o vazamento caiu de {last} para {d}"
        );
        last = d;
    }
}

/// **A tabela é indexada por `|κ|`, e a integral prova por quê.** `D` é par em
/// `x`, então a resposta não pode depender do sinal do raio.
///
/// ⚠️ Este gate existe porque eu ia alegar o contrário: que o sinal da curvatura
/// era um argumento a favor do canal por-vértice. Ele é — para o **Cavity**, não
/// para esta tabela.
#[test]
fn the_response_does_not_know_the_sign_of_the_curvature() {
    for i in 0..9 {
        let n_dot_l = -1.0 + i as f32 * 0.25;
        for t in [0.25f32, 1.0, 2.0] {
            let a = integrate(n_dot_l, t);
            let b = integrate(n_dot_l, -t);
            for c in 0..3 {
                assert!(
                    (a[c] - b[c]).abs() < 1e-6,
                    "N·L {n_dot_l}, t {t}, canal {c}: {} contra {}",
                    a[c],
                    b[c]
                );
            }
        }
    }
}

/// **A resolução da tabela é MEDIDA, não escolhida** — o erro de uma consulta
/// bilinear contra a integral exata tem de caber no próprio degrau de 8 bits.
///
/// Se ele não coubesse, dobrar [`LUT_SIZE`] compraria qualidade; como ele cabe,
/// dobrar não muda um byte do que sobe para o device.
#[test]
fn the_table_is_fine_enough() {
    let lut = bake_lut();
    let n = LUT_SIZE as usize;
    let fetch = |x: usize, y: usize, c: usize| f32::from(lut[(y * n + x) * 4 + c]) / 255.0;

    let mut worst = 0.0f32;
    // Amostra ENTRE os nós, que é onde a interpolação erra — amostrar nos nós
    // mediria a quantização e chamaria isso de erro de resolução.
    for yi in 0..n - 1 {
        for xi in 0..n - 1 {
            let (fx, fy) = (0.5f32, 0.5f32);
            let n_dot_l = lut_n_dot_l(xi) + fx * (lut_n_dot_l(xi + 1) - lut_n_dot_l(xi));
            let t = lut_t(yi) + fy * (lut_t(yi + 1) - lut_t(yi));
            let exact = integrate(n_dot_l, t);
            for (c, want) in exact.iter().enumerate() {
                let bilinear = (1.0 - fx) * (1.0 - fy) * fetch(xi, yi, c)
                    + fx * (1.0 - fy) * fetch(xi + 1, yi, c)
                    + (1.0 - fx) * fy * fetch(xi, yi + 1, c)
                    + fx * fy * fetch(xi + 1, yi + 1, c);
                worst = worst.max((bilinear - want).abs());
            }
        }
    }
    // O degrau da quantização de 8 bits é 1/255 = 0,00392. A tabela não pode
    // errar mais que ele — senão a resolução, e não o formato, seria o limite.
    assert!(
        worst <= 1.0 / 255.0,
        "a consulta bilinear erra {worst:.5}, acima do degrau de 8 bits (0,00392) \
         — a resolucao virou o limite e LUT_SIZE precisa subir"
    );
    println!("erro maximo da tabela: {worst:.5} (degrau de 8 bits: 0.00392)");
}

/// **O teto do eixo cobre o regime que o MATERIAL ocupa** — e não a assíntota,
/// que é outra coisa e fica bem mais longe.
///
/// ⚠️ **Este gate substitui um que eu escrevi errado.** O primeiro afirmava que
/// *"a curva já assentou em `T_MAX`"*, e a sonda mostrou que ela não assenta
/// antes de `t ≈ 24`: a resposta tende a `1/π`, devagar. Perseguir a assíntota
/// seria o teto errado — `t = scatter/R` é *quantos raios de curvatura a luz
/// percorre*, e a borda de uma orelha (o caso mais extremo de pele real) é
/// `t ≈ 0,7`.
///
/// Então o que se afirma é o que decide: o teto está **muito além** do regime
/// físico, e o que acontece acima dele é saturação honesta.
#[test]
fn the_ceiling_is_far_past_the_physical_regime() {
    /// A borda de uma orelha: raio ~3 mm, espalhamento da pele ~2 mm.
    const THINNEST_SKIN: f32 = 0.7;
    // ⚠️ **`const {}` e não um `assert!` de runtime, e o clippy tem razão:** as
    // duas são constantes, então a afirmação pode ser cobrada na COMPILAÇÃO.
    // Baixar o teto abaixo do regime da pele deixa de ser um teste vermelho e
    // passa a ser um build quebrado, que é a única forma de ninguém o ignorar.
    const {
        assert!(T_MAX >= 4.0 * THINNEST_SKIN);
    }
    // E ele não é gratuito: a curva ainda ANDA dentro da tabela, então as linhas
    // não estão sendo gastas num platô. (Se ela tivesse assentado bem antes do
    // teto, a resolução estaria sendo desperdiçada.)
    let half = integrate(0.0, T_MAX * 0.5)[0];
    let full = integrate(0.0, T_MAX)[0];
    assert!(
        full - half > 4.0 / 255.0,
        "entre metade do teto e o teto a curva move {:.4}, menos que um degrau de \
         8 bits — a metade de cima da tabela seria platô",
        full - half
    );
}

/// A porta clampa o que o device não sabe recusar, e **a divisão pelo teto
/// acontece só nela**.
#[test]
fn the_door_clamps_and_owns_the_ceiling() {
    let raw = SssRaw::pack(SssParams {
        strength: 3.0,
        scatter: -1.0,
    });
    assert!(
        (raw.params[0] - 1.0).abs() < 1e-6,
        "forca {}",
        raw.params[0]
    );
    assert!(raw.params[1] >= 0.0, "scatter {}", raw.params[1]);

    let raw = SssRaw::pack(SssParams {
        strength: 0.5,
        scatter: T_MAX,
    });
    assert!(
        (raw.params[1] - 1.0).abs() < 1e-6,
        "um scatter igual ao teto deveria mapear a coordenada 1, e deu {}",
        raw.params[1]
    );
}

/// **O espalhamento nasce da PEÇA** — a lição que o bake e o AO de tela já
/// pagaram.
#[test]
fn the_scatter_is_a_fraction_of_the_piece() {
    let small = SssParams::for_bounds(ph2d_mesh::Aabb {
        min: [-0.5; 3],
        max: [0.5; 3],
    });
    let big = SssParams::for_bounds(ph2d_mesh::Aabb {
        min: [-50.0, -0.5, -0.5],
        max: [50.0, 0.5, 0.5],
    });
    assert!((small.scatter - SCATTER_FRACTION).abs() < 1e-6, "{small:?}");
    assert!(
        (big.scatter - 100.0 * SCATTER_FRACTION).abs() < 1e-4,
        "{big:?}"
    );
}

/// O uniform tem o tamanho que o WGSL declara.
#[test]
fn the_uniform_is_one_vec4() {
    assert_eq!(SssRaw::SIZE, 16);
}
