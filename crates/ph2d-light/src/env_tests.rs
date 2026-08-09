//! **A SONDA DO AMBIENTE** — o que um piso escalar custa, e o que um ambiente
//! com direção entrega.
//!
//! Fase 0 do item **IBL** do cabeçalho da W10 (`docs/3D/05.1` §5). Ela roda
//! **antes** de existir produto, e o que ela decide é: qual é o modelo de
//! ambiente, quantos termos ele precisa, e quanto de erro o barato tem contra o
//! exato.
//!
//! ## A pergunta
//!
//! Hoje o barro devolve [`crate::AMBIENT`] — **0,35, o mesmo número em toda
//! direção** — para qualquer face virada para longe da luz. Duas faces na sombra,
//! uma olhando para cima e outra para baixo, saem **idênticas**: na região que a
//! lâmpada não alcança, a escultura não tem leitura de forma nenhuma.
//!
//! Um ambiente é a mesma energia **redistribuída por direção**: o que vem de
//! cima é céu, o que vem de baixo é o ricochete do chão. É o que o `05.1` chama
//! de *"80% do parece caro antes de qualquer luz pontual existir"*.
//!
//! ```text
//! cargo test -p ph2d-light --release env_tests -- --ignored --nocapture
//! ```

/// Quantas amostras por eixo a quadratura usa.
///
/// ⚠️ **A integral é o ORÁCULO desta sonda**, então ela não pode ser a parte
/// errada: 512×1024 dá ~524k direções, e o erro da própria quadratura fica
/// abaixo de 1e-4 — três ordens abaixo do que ela vai julgar.
const NTHETA: usize = 512;
const NPHI: usize = 1024;

/// A radiância que chega da direção `d`, para um ambiente de GRADIENTE.
///
/// ⚠️ **O eixo é o `y` do referencial de SOMBREAMENTO, que é o da TELA** — as
/// lâmpadas deste rig são autoradas em azimute/elevação de tela
/// ([`crate::resolve`]), então um ambiente ancorado no MUNDO teria o céu a girar
/// enquanto as lâmpadas ficam paradas. Um estúdio cujo céu e cujas luzes se
/// movem um sem o outro não é um estúdio.
fn radiance(d: [f32; 3], env: Env) -> [f32; 3] {
    let t = d[1]; // -1 (baixo) .. +1 (cima)
    match env {
        Env::Hemisphere { sky, ground } => {
            let u = 0.5 + 0.5 * t;
            [
                ground[0] + (sky[0] - ground[0]) * u,
                ground[1] + (sky[1] - ground[1]) * u,
                ground[2] + (sky[2] - ground[2]) * u,
            ]
        }
        Env::ThreeZone {
            sky,
            horizon,
            ground,
        } => {
            // Duas rampas: chão→horizonte abaixo da linha, horizonte→céu acima.
            // É a forma de um HDRI de estúdio — a banda mais clara é o horizonte,
            // não o zênite.
            if t >= 0.0 {
                let u = t;
                [
                    horizon[0] + (sky[0] - horizon[0]) * u,
                    horizon[1] + (sky[1] - horizon[1]) * u,
                    horizon[2] + (sky[2] - horizon[2]) * u,
                ]
            } else {
                let u = -t;
                [
                    horizon[0] + (ground[0] - horizon[0]) * u,
                    horizon[1] + (ground[1] - horizon[1]) * u,
                    horizon[2] + (ground[2] - horizon[2]) * u,
                ]
            }
        }
    }
}

#[derive(Clone, Copy)]
enum Env {
    Hemisphere {
        sky: [f32; 3],
        ground: [f32; 3],
    },
    ThreeZone {
        sky: [f32; 3],
        horizon: [f32; 3],
        ground: [f32; 3],
    },
}

/// A irradiância EXATA numa normal: `(1/π) ∫ L(d) max(n·d, 0) dω`.
///
/// O `1/π` é a normalização lambertiana — com ela um ambiente de radiância
/// constante `c` devolve exatamente `c`, que é o que faz a comparação com o piso
/// escalar de hoje ser uma comparação de iguais.
fn irradiance_exact(n: [f32; 3], env: Env) -> [f32; 3] {
    let mut acc = [0.0f64; 3];
    for i in 0..NTHETA {
        // θ do zênite; `sin θ dθ dφ` é o elemento de ângulo sólido.
        let theta = (i as f64 + 0.5) * std::f64::consts::PI / NTHETA as f64;
        let (st, ct) = theta.sin_cos();
        for j in 0..NPHI {
            let phi = (j as f64 + 0.5) * std::f64::consts::TAU / NPHI as f64;
            let (sp, cp) = phi.sin_cos();
            // O eixo do gradiente é `y`, então o zênite é `+y`.
            let d = [(st * cp) as f32, ct as f32, (st * sp) as f32];
            let ndl = f64::from(n[0] * d[0] + n[1] * d[1] + n[2] * d[2]);
            if ndl <= 0.0 {
                continue;
            }
            let l = radiance(d, env);
            let w = ndl * st;
            for k in 0..3 {
                acc[k] += f64::from(l[k]) * w;
            }
        }
    }
    let dw = (std::f64::consts::PI / NTHETA as f64) * (std::f64::consts::TAU / NPHI as f64);
    let inv_pi = 1.0 / std::f64::consts::PI;
    [
        (acc[0] * dw * inv_pi) as f32,
        (acc[1] * dw * inv_pi) as f32,
        (acc[2] * dw * inv_pi) as f32,
    ]
}

/// A irradiância pela forma FECHADA de dois termos: `E(n) = c + (2/3)·k·n.y`,
/// onde `L(d) = c + k·d.y`.
///
/// ⚠️ **Para um gradiente LINEAR isto não é uma aproximação, é a resposta.** A
/// convolução de um harmônico zonal de grau `l` com o lóbulo cosseno é uma
/// multiplicação por `Â_l` (Ramamoorthi & Hanrahan 2001), e para `l = 1` o fator
/// é `2/3`. Um ambiente linear em `d.y` **não tem** termo de grau 2 — logo não há
/// o que aproximar, e o erro contra a integral é o da quadratura.
fn irradiance_linear(n: [f32; 3], c: [f32; 3], k: [f32; 3]) -> [f32; 3] {
    let a = 2.0 / 3.0 * n[1];
    [c[0] + k[0] * a, c[1] + k[1] * a, c[2] + k[2] * a]
}

/// Os três coeficientes ZONAIS de um ambiente que só depende de `d.y`.
///
/// ⚠️ **Só três, e os outros seis são ZERO por SIMETRIA, não por economia:** um
/// ambiente invariante por rotação em torno de um eixo tem projeção zonal — todo
/// harmônico com `m ≠ 0` integra a zero contra ele. Avaliar os nove seria
/// multiplicar seis vezes por zero.
///
/// Devolve `(L0, L1, L2)` já convolvidos com o lóbulo cosseno (`Â0 = 1`,
/// `Â1 = 2/3`, `Â2 = 1/4`), de modo que
/// `E(n) = L0 + L1·n.y + L2·(3·n.y² − 1)/2`.
fn zonal_irradiance_coeffs(env: Env) -> ([f32; 3], [f32; 3], [f32; 3]) {
    let mut l0 = [0.0f64; 3];
    let mut l1 = [0.0f64; 3];
    let mut l2 = [0.0f64; 3];
    for i in 0..NTHETA {
        let theta = (i as f64 + 0.5) * std::f64::consts::PI / NTHETA as f64;
        let (st, ct) = theta.sin_cos();
        let d = [0.0f32, ct as f32, st as f32];
        let l = radiance(d, env);
        // Polinômios de Legendre em `cos θ`, pesados pelo ângulo sólido do anel
        // (`2π sin θ dθ`) e pela normalização de cada banda.
        let (p0, p1, p2) = (1.0, ct, 0.5 * (3.0 * ct * ct - 1.0));
        let w = st * std::f64::consts::TAU;
        for k in 0..3 {
            let lv = f64::from(l[k]);
            l0[k] += lv * p0 * w;
            l1[k] += lv * p1 * w;
            l2[k] += lv * p2 * w;
        }
    }
    let dt = std::f64::consts::PI / NTHETA as f64;
    // `(2l+1)/(4π)` normaliza a projeção; `Â_l` a convolve com o cosseno; o
    // `1/π` é o mesmo lambertiano do exato.
    let f = |acc: [f64; 3], band: f64, a_hat: f64| -> [f32; 3] {
        let s = band / (4.0 * std::f64::consts::PI) * a_hat * dt / std::f64::consts::PI;
        [
            (acc[0] * s) as f32,
            (acc[1] * s) as f32,
            (acc[2] * s) as f32,
        ]
    };
    (
        f(l0, 1.0, std::f64::consts::PI),
        f(l1, 3.0, 2.0 * std::f64::consts::PI / 3.0),
        f(l2, 5.0, std::f64::consts::PI / 4.0),
    )
}

fn eval_zonal(n: [f32; 3], c: ([f32; 3], [f32; 3], [f32; 3])) -> [f32; 3] {
    let y = n[1];
    let p2 = 0.5 * (3.0 * y * y - 1.0);
    [
        c.0[0] + c.1[0] * y + c.2[0] * p2,
        c.0[1] + c.1[1] * y + c.2[1] * p2,
        c.0[2] + c.1[2] * y + c.2[2] * p2,
    ]
}

fn lum(c: [f32; 3]) -> f32 {
    0.2126f32.mul_add(c[0], 0.7152f32.mul_add(c[1], 0.0722 * c[2]))
}

/// **O QUE UM PISO ESCALAR CUSTA, E O QUE UM AMBIENTE ENTREGA.**
#[test]
#[ignore = "sonda: rode com --ignored --nocapture"]
fn measure_what_a_directional_ambient_buys() {
    // Um estúdio: céu frio em cima, ricochete quente e escuro embaixo. Os
    // números são o ponto de partida da medição, não a decisão -- a decisão sai
    // da tabela.
    let hemi = Env::Hemisphere {
        sky: [0.62, 0.70, 0.85],
        ground: [0.22, 0.19, 0.16],
    };
    let three = Env::ThreeZone {
        sky: [0.55, 0.64, 0.82],
        horizon: [0.78, 0.76, 0.72],
        ground: [0.20, 0.17, 0.14],
    };

    println!("\n=== O AMBIENTE: o que o piso escalar de hoje NÃO diz ===\n");
    println!(
        "  hoje, para QUALQUER normal:  E = {:.4} (o AMBIENT escalar)\n",
        crate::AMBIENT
    );

    for (name, env) in [("hemisfério (linear)", hemi), ("três zonas", three)] {
        let up = irradiance_exact([0.0, 1.0, 0.0], env);
        let side = irradiance_exact([1.0, 0.0, 0.0], env);
        let down = irradiance_exact([0.0, -1.0, 0.0], env);
        println!("  {name}");
        println!(
            "    para CIMA {:.4} | de LADO {:.4} | para BAIXO {:.4} | contraste cima/baixo {:.2}x",
            lum(up),
            lum(side),
            lum(down),
            lum(up) / lum(down).max(1e-6)
        );
        // A média sobre a esfera de normais -- é ela que diz se o ambiente
        // REDISTRIBUI ou se ele também muda a exposição.
        let mut mean = 0.0f64;
        let mut cnt = 0u32;
        for i in 0..64 {
            let t = (i as f32 + 0.5) / 64.0 * std::f32::consts::PI;
            let (s, c) = t.sin_cos();
            for j in 0..128 {
                let p = (j as f32 + 0.5) / 128.0 * std::f32::consts::TAU;
                let n = [s * p.cos(), c, s * p.sin()];
                mean += f64::from(lum(irradiance_exact(n, env)));
                cnt += 1;
            }
        }
        println!("    média sobre as normais: {:.4}", mean / f64::from(cnt));

        // O barato contra o exato.
        let z = zonal_irradiance_coeffs(env);
        let mut worst = 0.0f32;
        for i in 0..64 {
            let t = (i as f32 + 0.5) / 64.0 * std::f32::consts::PI;
            let (s, c) = t.sin_cos();
            let n = [s, c, 0.0];
            let e = irradiance_exact(n, env);
            let a = eval_zonal(n, z);
            for k in 0..3 {
                worst = worst.max((e[k] - a[k]).abs());
            }
        }
        println!("    zonal de 3 termos vs a integral: pior erro {worst:.6}");

        if let Env::Hemisphere { sky, ground } = env {
            let c = [
                (sky[0] + ground[0]) * 0.5,
                (sky[1] + ground[1]) * 0.5,
                (sky[2] + ground[2]) * 0.5,
            ];
            let k = [
                (sky[0] - ground[0]) * 0.5,
                (sky[1] - ground[1]) * 0.5,
                (sky[2] - ground[2]) * 0.5,
            ];
            let mut worst2 = 0.0f32;
            for i in 0..64 {
                let t = (i as f32 + 0.5) / 64.0 * std::f32::consts::PI;
                let (s, cc) = t.sin_cos();
                let n = [s, cc, 0.0];
                let e = irradiance_exact(n, env);
                let a = irradiance_linear(n, c, k);
                for q in 0..3 {
                    worst2 = worst2.max((e[q] - a[q]).abs());
                }
            }
            println!("    forma fechada de 2 termos vs a integral: pior erro {worst2:.6}");
        }
        println!();
    }
}

// ----------------------------------------------------------------- gates ----

/// **O AMBIENTE REDISTRIBUI, ELE NÃO EXPÕE.**
///
/// A média de [`crate::env_ambient`] sobre TODAS as normais é exatamente o
/// [`crate::AMBIENT`] de sempre, em luminância. ⚠️ Sem esta propriedade o knob
/// seria uma exposição disfarçada: subi-lo clarearia a peça inteira, e o artista
/// não teria como separar *"o ambiente tem direção"* de *"a cena ficou mais
/// clara"* — que são as duas coisas que ele precisa julgar em separado.
///
/// ⚠️ Ela vale em LUMINÂNCIA e **não** por canal, de propósito: a chroma do
/// ambiente é metade do efeito (a sombra de estúdio é fria), e exigi-la neutra
/// por canal proibiria exatamente isso.
#[test]
fn the_ambient_is_redistributed_not_brightened() {
    let mut sum = 0.0f64;
    let mut n = 0u32;
    for i in 0..256u16 {
        let t = (f32::from(i) + 0.5) / 256.0 * std::f32::consts::PI;
        let (s, c) = t.sin_cos();
        for j in 0..512u16 {
            let p = (f32::from(j) + 0.5) / 512.0 * std::f32::consts::TAU;
            sum += f64::from(lum(crate::env_ambient([s * p.cos(), c, s * p.sin()])));
            n += 1;
        }
    }
    let mean = sum / f64::from(n);
    assert!(
        (mean - f64::from(crate::AMBIENT)).abs() < 1.0e-3,
        "a média do ambiente sobre as normais é {mean:.5}, e tinha de ser o \
         AMBIENT ({}) -- o termo está expondo em vez de redistribuir",
        crate::AMBIENT
    );
}

/// **O CÉU É O TOPO DA TELA.**
///
/// ⚠️ **O sinal é a wave inteira, e ele é o que um erro de referencial troca sem
/// o compilador dizer nada.** A normal chega no frame do CANVAS, onde `y` cresce
/// para BAIXO — então uma face virada para o topo da tela tem `y` NEGATIVO. O
/// módulo já pagou esta negação uma vez, no `canvas_normal`, e o comentário de lá
/// nomeia o sintoma: *"a mesma lâmpada acende a pintura por cima e a escultura
/// por baixo"*.
///
/// O número (**2,20×**) é o que a sonda mede contra a integral exata, não um
/// redondo escolhido.
#[test]
fn the_sky_is_the_top_of_the_screen() {
    let up = lum(crate::env_ambient([0.0, -1.0, 0.0]));
    let down = lum(crate::env_ambient([0.0, 1.0, 0.0]));
    assert!(
        up > down,
        "uma face virada para o TOPO da tela recebe {up:.4} e uma virada para \
         BAIXO recebe {down:.4} -- o céu está no lugar errado"
    );
    let ratio = up / down;
    assert!(
        (ratio - 2.20).abs() < 0.02,
        "o contraste cima/baixo é {ratio:.3}x, e a medição do gradiente dá 2,20x"
    );
}

/// **AS CONSTANTES SÃO O QUE A MEDIÇÃO DEU** — derivadas do par céu/chão, não
/// escolhidas.
///
/// ⚠️ **A derivação vive AQUI e não no produto**, e é a diferença entre um número
/// verificado e um número duplicado: o shader e o [`crate::env_ambient`] leem os
/// literais (que é o que a fronteira do device sabe ler), e este gate prova que
/// eles são exatamente `(céu+chão)/2` normalizado e `(2/3)·(céu−chão)/2`
/// normalizado. Quem quiser outro estúdio edita o par, roda a sonda, e traz os
/// dois números de volta.
#[test]
fn the_constants_are_the_measured_gradient() {
    // O estúdio: céu frio em cima, ricochete quente e escuro embaixo.
    const SKY: [f32; 3] = [0.62, 0.70, 0.85];
    const GROUND: [f32; 3] = [0.22, 0.19, 0.16];
    let c = [
        (SKY[0] + GROUND[0]) * 0.5,
        (SKY[1] + GROUND[1]) * 0.5,
        (SKY[2] + GROUND[2]) * 0.5,
    ];
    let k = [
        (SKY[0] - GROUND[0]) * 0.5,
        (SKY[1] - GROUND[1]) * 0.5,
        (SKY[2] - GROUND[2]) * 0.5,
    ];
    let norm = lum(c);
    for i in 0..3 {
        let base = c[i] / norm;
        let slope = (2.0 / 3.0) * k[i] / norm;
        assert!(
            (base - crate::ENV_BASE[i]).abs() < 1.0e-3,
            "ENV_BASE[{i}] é {} e a derivação dá {base:.4}",
            crate::ENV_BASE[i]
        );
        assert!(
            (slope - crate::ENV_SLOPE[i]).abs() < 1.0e-3,
            "ENV_SLOPE[{i}] é {} e a derivação dá {slope:.4}",
            crate::ENV_SLOPE[i]
        );
    }
}
