//! O gate que autoriza o sombreamento no dispositivo a existir.

use super::{Sample, on_cpu, on_device};

/// A grelha de direcções — normais, observadores e luzes que exercitam o lóbulo inteiro.
fn amostras() -> Vec<Sample> {
    let norm = |v: [f32; 3]| {
        let k = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
        [v[0] / k, v[1] / k, v[2] / k]
    };
    let mut out = Vec::new();
    // ⚠️ **Rasantes incluídos de propósito**: é lá que o `N·V → 0` põe o `4·ndv` do denominador a
    // aproximar-se de zero e que um `clamp` diferente entre os dois motores se vê.
    for nz in [1.0_f32, 0.92, 0.6, 0.2, 0.03] {
        for ang in [0.0_f32, 0.7, 1.4, 2.6, 3.9, 5.1] {
            let s = (1.0 - nz * nz).max(0.0).sqrt();
            let n = norm([s * ang.cos(), s * ang.sin(), nz]);
            for vz in [1.0_f32, 0.75, 0.35, 0.08] {
                let sv = (1.0 - vz * vz).max(0.0).sqrt();
                let v = norm([sv * (ang + 1.1).cos(), sv * (ang + 1.1).sin(), vz]);
                for lz in [0.95_f32, 0.5, 0.12, -0.3] {
                    let sl = (1.0 - lz * lz).max(0.0).sqrt();
                    out.push(Sample {
                        n,
                        v,
                        l: norm([sl * (ang + 2.3).cos(), sl * (ang + 2.3).sin(), lz]),
                    });
                }
            }
        }
    }
    out
}

/// Os materiais: o de omissão, um metal, um espelho, um verniz, um fosco e um emissivo.
fn materiais() -> Vec<(&'static str, ph2d_material::OpenPbr)> {
    let d = ph2d_material::OpenPbr::default;
    vec![
        ("omissão", d()),
        (
            "metal",
            ph2d_material::OpenPbr {
                base_metalness: 1.0,
                base_color: [0.94, 0.78, 0.35],
                specular_roughness: 0.18,
                ..d()
            },
        ),
        (
            "espelho",
            ph2d_material::OpenPbr {
                specular_roughness: 0.02,
                base_color: [0.05; 3],
                ..d()
            },
        ),
        (
            "verniz",
            ph2d_material::OpenPbr {
                coat_weight: 1.0,
                coat_roughness: 0.08,
                coat_color: [0.9, 0.95, 1.0],
                base_color: [0.6, 0.1, 0.1],
                ..d()
            },
        ),
        (
            "fosco",
            ph2d_material::OpenPbr {
                specular_roughness: 0.9,
                base_diffuse_roughness: 0.8,
                base_color: [0.35, 0.55, 0.3],
                ..d()
            },
        ),
        (
            "emissivo",
            ph2d_material::OpenPbr {
                emission_luminance: 3.0,
                emission_color: [1.0, 0.4, 0.15],
                coat_weight: 0.5,
                ..d()
            },
        ),
        // ⛔⛔⛔ **OS DOIS DA SUBSUPERFÍCIE, e até 2026-09-19 esta lista NÃO OS TINHA.**
        //
        // Os seis de cima partem todos de `OpenPbr::default()`, que tem `subsurface_weight = 0` ⇒
        // **a paridade de materiais nunca tinha corrido uma linha do caminho da subsuperfície**, e
        // o gémeo da borda mole ia ser construído contra uma régua que não o vê. *É a mesma lei do
        // Fujii, um bloco acima: um corpus no ponto NEUTRO de um knob não testa esse knob.*
        //
        // ⚠️ **São DOIS porque são DOIS caminhos** e não um grau: o maciço integra o perfil de
        // Burley sobre a curvatura, a parede fina nega a normal e é lambertiana do lado de lá.
        (
            "jade (maciço)",
            ph2d_material::OpenPbr {
                subsurface_weight: 1.0,
                geometry_thin_walled: false,
                subsurface_color: [0.75, 0.35, 0.35],
                base_color: [0.75, 0.35, 0.35],
                subsurface_radius: 0.4,
                ..d()
            },
        ),
        (
            "folha (fina)",
            ph2d_material::OpenPbr {
                subsurface_weight: 1.0,
                geometry_thin_walled: true,
                subsurface_color: [0.35, 0.75, 0.2],
                base_color: [0.2, 0.5, 0.1],
                ..d()
            },
        ),
    ]
}

/// ⭐⭐⭐ **O MATERIAL DO DISPOSITIVO É O DA CPU** — as QUATRO respostas, sobre `720` direcções e
/// `8` materiais.
///
/// # A barra, e de que recurso ela é
///
/// As duas leis são a **mesma expressão** na mesma ordem, em `f32`; o que as separa é o hardware
/// contrair uma multiplicação e uma soma num `fma` onde a CPU as faz em dois passos, e as funções
/// transcendentes (`pow`, `acos`, `sqrt`) terem `ulp` próprio em cada implementação.
///
/// ⚠️ **A barra é RELATIVA e não absoluta**: a resposta directa de um espelho a `α = 4e-4` chega a
/// centenas (o lóbulo é estreito e alto), e uma barra absoluta ali mediria o pico em vez da lei.
/// ⛔ E não é o bit: pedir igualdade exacta reprovaria sobre `fma`, que é **melhor** aritmética.
///
/// ⚠️ **O CONTROLO vem primeiro** — sem ele, uma grelha que devolvesse zeros dos dois lados passava.
///
/// # ⭐⭐⭐ A prova de mutação, e o que ela ensinou sobre o CORPUS
///
/// A mutação não foi inventada: é o **erro que este port de facto cometeu**. A primeira redacção do
/// WGSL escreveu as duas constantes de Fujii *de cabeça* e as duas saíram erradas — o `FUJII_2` por
/// um factor (`0,0626` contra `0,0725`). Reposta a errada, o gate lê **`6,167e-3`**, `60×` a barra.
///
/// ⚠️⚠️ **E ela acende em UM material de seis.** O `FUJII_2` só entra multiplicado pela
/// `base_diffuse_roughness`, que no material de omissão é **`0`** — logo cinco dos seis materiais
/// são **cegos** a ela e liam `~1e-7` com o erro lá dentro. *Um corpus no ponto NEUTRO de um knob
/// não testa esse knob*, e é por isso que a lista de materiais aqui varre os extremos em vez de
/// variar a cor.
///
/// ⇒ a cura que ficou não foi corrigir os números: foi **derivá-los da mesma expressão** que o Rust
/// usa. *Uma constante lê-se do ficheiro, nunca da memória.*
#[test]
#[ignore = "precisa de GPU"]
fn o_material_do_dispositivo_e_o_da_cpu() {
    let am = amostras();
    assert!(
        am.len() >= 480,
        "grelha pobre: {} (5 normais × 6 azimutes × 4 vistas × 4 luzes)",
        am.len()
    );

    let mut pior_global = 0.0f64;
    let mut linhas = Vec::new();
    for (nome, m) in materiais() {
        let s = m.prepare();
        let cpu = on_cpu(&s, &am);
        let Some(gpu) = on_device(&s, &am) else {
            println!("sem adaptador — saltado");
            return;
        };

        // ⭐ O CONTROLO: a lei tem de PRODUZIR alguma coisa nesta grelha.
        let vivos = cpu
            .iter()
            .filter(|a| a.direct.iter().chain(&a.indirect).any(|c| *c > 1e-4))
            .count();
        assert!(
            vivos * 4 > am.len(),
            "o material «{nome}» só respondeu em {vivos} de {} amostras — a grelha não o exercita, \
             e a comparação abaixo não afirmaria nada",
            am.len()
        );

        let mut pior = 0.0f64;
        let mut onde = 0usize;
        for (i, (a, b)) in cpu.iter().zip(&gpu).enumerate() {
            // ⭐ **As QUATRO respostas**, e a quarta é a que a borda mole lê.
            for (x, y) in a
                .direct
                .iter()
                .chain(&a.indirect)
                .chain(&a.emission)
                .chain(&a.direct_sss)
                .zip(
                    b.direct
                        .iter()
                        .chain(&b.indirect)
                        .chain(&b.emission)
                        .chain(&b.direct_sss),
                )
            {
                let rel = f64::from((x - y).abs()) / f64::from(x.abs().max(*y).max(1e-3));
                if rel > pior {
                    pior = rel;
                    onde = i;
                }
            }
        }
        linhas.push(format!(
            "  {nome:>9} · pior desvio relativo {pior:.3e} · na amostra {onde}"
        ));
        pior_global = pior_global.max(pior);
    }
    for l in &linhas {
        println!("{l}");
    }

    // ⚠️ `1e-4` é a folga de `f32` sob contracção: `~1e-7` por operação, e a composição encadeia
    // dezenas delas com divisões pelo meio. ⛔ Uma LEI diferente — uma associação trocada, um
    // `clamp` em falta, um coeficiente mal transcrito — move **ordens de grandeza**: as duas
    // constantes de Fujii que esta wave escreveu de cabeça estavam erradas no 6.º dígito e uma
    // delas por um FACTOR.
    assert!(
        pior_global < 1e-4,
        "o pior desvio relativo entre os dois motores é {pior_global:.3e} — acima da folga de \
         `f32`. Não é precisão: é uma LEI diferente."
    );
}
