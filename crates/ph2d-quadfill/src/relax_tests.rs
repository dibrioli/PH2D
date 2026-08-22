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
