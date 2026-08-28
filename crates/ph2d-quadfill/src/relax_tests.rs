//! **OS GATES DA LEI DO QUADRADO** — ver [`super::nearest_square`].
//!
//! ⚠️ **Eles não constroem malha nenhuma, e é esse o ponto.** A projecção é a
//! única parte desta crate que é matemática pura; medi-la através de uma malha
//! misturaria um erro de sinal com a parametrização, a reprojecção e o
//! amortecimento — e um erro de sinal aqui produz uma malha **plausível** e
//! errada, que é a categoria que esta linha já pagou várias vezes.

use super::nearest_square;

/// Os quatro cantos de um quadrado de raio `r` rodado de `ang`, na mão directa.
fn square(r: f32, ang: f32) -> [[f32; 2]; 4] {
    let mut out = [[0.0f32; 2]; 4];
    for (k, o) in out.iter_mut().enumerate() {
        #[allow(clippy::cast_precision_loss)]
        let a = ang + std::f32::consts::FRAC_PI_2 * k as f32;
        *o = [r * a.cos(), r * a.sin()];
    }
    out
}

fn dist(a: [[f32; 2]; 4], b: [[f32; 2]; 4]) -> f32 {
    (0..4)
        .map(|k| (a[k][0] - b[k][0]).hypot(a[k][1] - b[k][1]))
        .fold(0.0f32, f32::max)
}

/// ⭐⭐ **PONTO FIXO — um quadrado já quadrado não se mexe.**
///
/// ⛔ **Sem este lado, a lei podia estar a encolher tudo** e o gate irmão — que só
/// pede que um losango melhore — continuaria verde: encolher um losango para um
/// ponto também *melhora* o ângulo dele.
#[test]
fn a_square_is_its_own_nearest_square() {
    for turn in [0.0f32, 0.3, 1.0, -2.2] {
        for r in [0.01f32, 1.0, 50.0] {
            let s = square(r, turn);
            let got = nearest_square(s);
            assert!(
                dist(s, got) <= 1.0e-5 * r,
                "raio {r} volta {turn}: o quadrado andou {:.2e}",
                dist(s, got)
            );
        }
    }
}

/// ⭐⭐⭐ **A FORMA FECHADA ACHA MESMO O MAIS PRÓXIMO** — verificada contra uma
/// busca por força bruta, que é uma segunda implementação independente.
///
/// ⛔⛔ **Este gate existe porque o irmão abaixo é uma TAUTOLOGIA, e a prova de
/// mutação mostrou-o** (2026-08-22). `a_rhombus_becomes_a_square` afirma *"a saída
/// é um quadrado"* — mas `wₖ = h·iᵏ` é um quadrado **para qualquer `h`**, incluindo
/// um `h` errado. Com um sinal trocado em `a.y`, três dos quatro gates deste
/// ficheiro continuavam verdes; rodar a fixtura **não** os salvou, porque o defeito
/// não estava na fixtura. *Um gate que afirma o que a construção já garante não
/// pode ficar vermelho pela razão que ele diz medir.*
///
/// ⭐ **A cura é o ORÁCULO, não outra fixtura:** varre-se o ângulo, para cada
/// ângulo o raio óptimo sai por projecção, e o mínimo dessa varredura é o resíduo
/// que a forma fechada tem de igualar. ⚠️ *A varredura não sabe nada da álgebra que
/// está a julgar* — é isso que a torna testemunha.
#[test]
fn the_closed_form_finds_the_nearest_square() {
    // ⚠️ Fixturas ASSIMÉTRICAS: uma simetria cancela erros de sinal, e foi assim
    // que a primeira versão deste ficheiro passou por engano.
    for (name, z) in [
        (
            "losango deitado",
            [[1.0f32, 0.0], [0.0, 0.25], [-1.0, 0.0], [0.0, -0.25]],
        ),
        (
            "trapezio torto",
            [[1.0f32, 0.2], [-0.1, 0.9], [-1.2, -0.3], [0.3, -0.8]],
        ),
        (
            "quase degenerado",
            [[1.0f32, 0.0], [0.9, 0.05], [-1.0, 0.0], [-0.9, -0.05]],
        ),
        (
            "muito rodado",
            [[0.3f32, 1.1], [-1.0, 0.4], [-0.2, -1.0], [0.9, -0.5]],
        ),
    ] {
        // Centra, que é a pré-condição da lei.
        let c = [
            0.25 * (z[0][0] + z[1][0] + z[2][0] + z[3][0]),
            0.25 * (z[0][1] + z[1][1] + z[2][1] + z[3][1]),
        ];
        let mut zc = z;
        for p in &mut zc {
            *p = [p[0] - c[0], p[1] - c[1]];
        }
        let residual = |w: [[f32; 2]; 4]| -> f32 {
            (0..4)
                .map(|k| (w[k][0] - zc[k][0]).powi(2) + (w[k][1] - zc[k][1]).powi(2))
                .sum()
        };
        let mine = residual(nearest_square(zc));
        // ── A TESTEMUNHA: varre o ângulo; para cada um, o raio óptimo é a
        // projecção `r* = ¼ Σ ⟨zₖ, uₖ⟩`, e as duas mãos entram as duas.
        let mut best = f32::MAX;
        for step in 0..36_000 {
            #[allow(clippy::cast_precision_loss)]
            let ang = std::f32::consts::TAU * step as f32 / 36_000.0;
            for hand in [1.0f32, -1.0] {
                let unit = |k: usize| {
                    #[allow(clippy::cast_precision_loss)]
                    let a = hand.mul_add(std::f32::consts::FRAC_PI_2 * k as f32, ang);
                    [a.cos(), a.sin()]
                };
                let r: f32 = 0.25
                    * (0..4)
                        .map(|k| {
                            let u = unit(k);
                            zc[k][0].mul_add(u[0], zc[k][1] * u[1])
                        })
                        .sum::<f32>();
                let mut w = [[0.0f32; 2]; 4];
                for (k, o) in w.iter_mut().enumerate() {
                    let u = unit(k);
                    *o = [r * u[0], r * u[1]];
                }
                best = best.min(residual(w));
            }
        }
        assert!(
            mine <= best + 1.0e-4,
            "{name}: a forma fechada deu residuo {mine:.6} e a busca achou {best:.6} \
             -- a lei NAO esta a achar o quadrado mais proximo"
        );
    }
}

/// ⭐⭐⭐ **O LOSANGO VIRA QUADRADO** — a afirmação que dá razão ao módulo.
///
/// ⚠️ **Ela é NECESSÁRIA e não suficiente** — ver
/// [`the_closed_form_finds_the_nearest_square`], que é quem prova que o quadrado é
/// o *certo*. Este gate fica porque descreve o produto em palavras do artista: *o
/// losango que o Laplaciano não sabia ver sai quadrado.*
///
/// ⚠️ **A fixtura tem de CONTER o fenómeno** ([[reference_topic_fixture_discipline]]):
/// um losango tem as **quatro arestas iguais** e é por isso um ponto fixo do
/// Laplaciano — é exactamente a forma que o alisador antigo não sabia ver, e é ela
/// que a foto de 2026-08-22 mostra aos milhares.
#[test]
fn a_rhombus_becomes_a_square() {
    // Esticado num eixo e encolhido no outro: as quatro arestas continuam iguais,
    // os cantos são 2·atan(0.25) ≈ 28° e o suplemento.
    //
    // ⚠️⚠️ **E ele está RODADO de propósito.** Com o losango alinhado aos eixos
    // (`[1,0] [0,0.25] [-1,0] [0,-0.25]`) uma troca de sinal em `a.y` **CANCELA-SE
    // por simetria** e este gate sobrevive à mutação — medido em 2026-08-22: dos
    // quatro gates deste ficheiro, só o do ponto fixo morria. *Uma fixtura simétrica
    // é cega a metade dos erros de sinal que a lei pode ter*
    // ([[reference_topic_fixture_discipline]]).
    let rhombus = {
        let (s, c) = 0.7f32.sin_cos();
        let mut q = [[1.0f32, 0.0], [0.0, 0.25], [-1.0, 0.0], [0.0, -0.25]];
        for p in &mut q {
            *p = [c.mul_add(p[0], -(s * p[1])), s.mul_add(p[0], c * p[1])];
        }
        q
    };
    let got = nearest_square(rhombus);
    // Os quatro cantos do resultado a 90°, e o raio o mesmo em todos.
    let r: Vec<f32> = got.iter().map(|p| p[0].hypot(p[1])).collect();
    let lo = r.iter().copied().fold(f32::MAX, f32::min);
    let hi = r.iter().copied().fold(0.0f32, f32::max);
    assert!(
        hi - lo <= 1.0e-5,
        "os quatro cantos nao ficaram a igual distancia do centro: {r:?}"
    );
    assert!(
        lo > 0.1,
        "a lei encolheu o losango para um ponto: raio {lo}"
    );
    for k in 0..4 {
        let (a, b) = (got[k], got[(k + 1) % 4]);
        let c = a[0].mul_add(b[0], a[1] * b[1]) / (a[0].hypot(a[1]) * b[0].hypot(b[1]));
        let deg = c.clamp(-1.0, 1.0).acos().to_degrees();
        assert!(
            (deg - 90.0).abs() <= 0.01,
            "canto {k}: {deg:.2}° do centro, esperado 90°"
        );
    }
}

/// ⭐⭐ **A MÃO INVERSA — um quad DOBRADO recebe o quadrado do lado certo.**
///
/// ⛔ **É a metade que impede a cura de agravar o defeito.** Um quad cuja volta se
/// inverteu no plano dele tem harmónico directo quase nulo; pedir-lhe na mesma o
/// quadrado da mão directa puxaria os quatro cantos para **perto do centro** — a
/// face colapsaria em vez de se endireitar. *Medido aqui: sem a escolha por
/// módulo, o raio do resultado cai para ~zero.*
#[test]
fn a_flipped_quad_gets_the_square_of_its_own_handedness() {
    let mut s = square(1.0, 0.0);
    s.reverse();
    let got = nearest_square(s);
    let lo = got
        .iter()
        .map(|p| p[0].hypot(p[1]))
        .fold(f32::MAX, f32::min);
    assert!(
        lo > 0.9,
        "o quad da mao inversa colapsou para o centro: raio {lo}"
    );
    assert!(
        dist(s, got) <= 1.0e-5,
        "a mao inversa nao e' ponto fixo: andou {:.2e}",
        dist(s, got)
    );
}

/// ⚠️ **O centro do resultado é o centro da entrada** — a translação é o harmónico
/// zero e a lei não lhe toca.
///
/// ⛔ Sem isto, uma lei que devolvesse o quadrado certo **no sítio errado** passaria
/// nos três gates acima e arrastaria a malha inteira a cada ronda.
#[test]
fn the_centre_does_not_move() {
    let skewed = [[1.0f32, 0.2], [-0.1, 0.9], [-1.2, -0.3], [0.3, -0.8]];
    let got = nearest_square(skewed);
    let c = |q: [[f32; 2]; 4]| {
        [
            0.25 * (q[0][0] + q[1][0] + q[2][0] + q[3][0]),
            0.25 * (q[0][1] + q[1][1] + q[2][1] + q[3][1]),
        ]
    };
    let (a, b) = (c(skewed), c(got));
    assert!(
        (a[0] - b[0]).hypot(a[1] - b[1]) <= 1.0e-5,
        "o centro andou de {a:?} para {b:?}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// ⭐⭐⭐ A LEI DO ALINHAMENTO AO RELEVO — ver [`super::steer`].
// ─────────────────────────────────────────────────────────────────────────────

/// A orientação da GRADE de um quadrado de harmónico `h`, dobrada em `[0°, 90°)`.
///
/// ⚠️ **É a aresta, não o harmónico** — os cantos de `h·iᵏ` estão a `arg(h) + 90°k` e as
/// arestas entre eles correm a `arg(h) + 45° + 90°k`. *Medir o harmónico daria um gate
/// verde com a grade rodada 45°.*
fn grid_angle(h: [f32; 2]) -> f32 {
    let a = h[1].atan2(h[0]) + std::f32::consts::FRAC_PI_4;
    a.rem_euclid(std::f32::consts::FRAC_PI_2)
}

/// A menor diferença entre dois ângulos de grade, em `[0°, 45°]`.
fn grid_gap(a: f32, b: f32) -> f32 {
    let q = std::f32::consts::FRAC_PI_2;
    let d = (a - b).rem_euclid(q);
    d.min(q - d)
}

/// ⭐⭐⭐ **PESO ZERO NÃO TOCA — ao bit.**
///
/// ⛔ **É esta metade que faz a esfera continuar a ver a lei antiga.** Numa superfície sem
/// direcção preferida a anisotropia é `0`, e sem esta garantia o alinhamento poria uma
/// costura onde a forma não pede nenhuma. ⚠️ *Bit a bit, e não «quase igual»* — a linha
/// `x0` da varredura de 2026-08-28 tem de ser idêntica em toda coluna à corrida sem
/// alinhamento nenhum, e um `1e-7` de deriva estragaria essa comparação.
#[test]
fn a_zero_weight_hint_leaves_the_square_untouched() {
    let h = [0.7f32, -0.3];
    for f in [[1.0f32, 0.0], [0.0, 1.0], [-0.4, 0.9], [0.0, 0.0]] {
        let got = super::steer(h, f, 0.0);
        assert!(
            got[0].to_bits() == h[0].to_bits() && got[1].to_bits() == h[1].to_bits(),
            "peso zero mexeu o harmonico: {h:?} -> {got:?} com alvo {f:?}"
        );
    }
}

/// ⭐⭐⭐ **COM PESO CHEIO A GRADE FICA NA DIRECÇÃO PEDIDA** — e a régua é módulo `90°`.
///
/// ⚠️ **A varredura cobre uma volta inteira do alvo**, de propósito: a dobragem em
/// `[−45°, 45°]` é onde um erro de sinal ou um `rem_euclid` trocado se esconde, e ele só
/// aparece nos quadrantes que uma fixtura única não visita.
#[test]
fn full_weight_lands_the_grid_on_the_asked_direction() {
    let h = [0.6f32, 0.25];
    for k in 0..24 {
        #[allow(clippy::cast_precision_loss)]
        let t = std::f32::consts::TAU * k as f32 / 24.0;
        let f = [t.cos(), t.sin()];
        let got = super::steer(h, f, 1.0);
        let gap = grid_gap(grid_angle(got), t);
        assert!(
            gap <= 1.0e-4,
            "com peso 1 a grade ficou a {:.3}° do alvo (alvo {:.1}°)",
            gap.to_degrees(),
            t.to_degrees()
        );
    }
}

/// ⭐⭐ **O TAMANHO NÃO É NEGOCIADO** — a rotação é uma rotação.
///
/// ⛔ Sem este lado, uma implementação que devolvesse `f` normalizado a `|h|` errado
/// passaria no gate acima e encolheria a malha a cada ronda. *O tamanho vem dos quatro
/// pontos; só a orientação vem do relevo.*
#[test]
fn steering_is_a_rotation_and_keeps_the_size() {
    let h = [0.6f32, 0.25];
    let r0 = h[0].hypot(h[1]);
    for k in 0..24 {
        #[allow(clippy::cast_precision_loss)]
        let t = std::f32::consts::TAU * k as f32 / 24.0;
        for w in [0.13f32, 0.5, 1.0] {
            let got = super::steer(h, [t.cos(), t.sin()], w);
            let r = got[0].hypot(got[1]);
            assert!(
                (r - r0).abs() <= 1.0e-5 * r0.max(1.0),
                "o raio mudou de {r0} para {r} (peso {w}, alvo {:.1}°)",
                t.to_degrees()
            );
        }
    }
}

/// ⭐⭐⭐ **O QUADRADO NUNCA RODA MAIS DE 45°** — porque rodar `90°` é a mesma grade.
///
/// ⚠️ **É o gate que impede o alinhamento de VIRAR a grade de lado.** Sem a dobragem, um
/// alvo a `80°` faria o quadrado dar quase um quarto de volta para «obedecer» a uma
/// direcção que ele já obedecia; a malha inteira giraria por nada e o relevo pioraria
/// exactamente onde ele é mais forte.
///
/// ⛔⛔ **A 1.ª redacção deste gate era uma TAUTOLOGIA, e a prova de mutação apanhou-a**
/// (2026-08-28): ela media a rotação com o [`grid_gap`], que devolve um valor em
/// `[0°, 45°]` **por construção** — a asserção não podia falhar, e com a dobragem removida
/// ela ficou verde. ⭐ A régua certa é o ângulo **CRU** entre `h` e o `h` rodado: uma
/// rotação de `80°` lê-se `80°` ali, e `45°` no outro.
#[test]
fn steering_never_turns_a_square_by_more_than_an_eighth_turn() {
    let h = [0.6f32, 0.25];
    for k in 0..64 {
        #[allow(clippy::cast_precision_loss)]
        let t = std::f32::consts::TAU * k as f32 / 64.0;
        for w in [0.25f32, 1.0] {
            let got = super::steer(h, [t.cos(), t.sin()], w);
            // O ângulo CRU entre os dois harmónicos, dobrado só em `(-180°, 180°]`.
            let raw = (got[1].atan2(got[0]) - h[1].atan2(h[0]) + std::f32::consts::PI)
                .rem_euclid(std::f32::consts::TAU)
                - std::f32::consts::PI;
            assert!(
                raw.abs() <= std::f32::consts::FRAC_PI_4 + 1.0e-4,
                "o quadrado rodou {:.1}°, mais que os 45° que uma grade admite (peso {w}, alvo {:.1}°)",
                raw.to_degrees(),
                t.to_degrees()
            );
        }
    }
}

/// ⭐⭐ **O PESO INTERPOLA, e é monótono** — meio peso anda menos de meio caminho? Não: anda
/// **exactamente** metade do desvio, e é isso que faz a anisotropia ser uma confiança e não
/// um interruptor.
///
/// ⛔ Sem ele, uma implementação que tratasse qualquer `w > 0` como `1` passaria nos outros
/// quatro gates — e as linhas `x0,5`, `x1` e `x2` da varredura mediriam a mesma coisa.
#[test]
fn half_the_weight_covers_half_the_gap() {
    let h = [0.6f32, 0.25];
    let a0 = grid_angle(h);
    for k in 0..16 {
        #[allow(clippy::cast_precision_loss)]
        let t = std::f32::consts::TAU * k as f32 / 16.0;
        let f = [t.cos(), t.sin()];
        let full = grid_gap(grid_angle(super::steer(h, f, 1.0)), a0);
        let half = grid_gap(grid_angle(super::steer(h, f, 0.5)), a0);
        assert!(
            (half - full * 0.5).abs() <= 1.0e-4,
            "meio peso andou {:.3}° e o caminho inteiro e' {:.3}°",
            half.to_degrees(),
            full.to_degrees()
        );
    }
}
