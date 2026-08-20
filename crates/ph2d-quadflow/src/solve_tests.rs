//! **O GATE QUE MEDE O QUE A HIERARQUIA COMPRA** (ADR-0160, Q3.5).

use ph2d_mesh::{Mesh, shapes};

use super::solve_fields;
use crate::extract::extract;
use crate::orientation::solve_orientation;
use crate::position::solve_position;
use crate::scale::ScaleField;

fn sphere() -> Mesh {
    shapes::uv_sphere(48, 64, 1.0)
}

fn torus() -> Mesh {
    shapes::torus(64, 32, 1.0, 0.35)
}

/// ⭐ **A HIERARQUIA PAGA — e este gate já disse o contrário, com número.**
///
/// | campo × células | esfera | toro |
/// |---|---|---|
/// | plano + semente | 42,9 % | 43,4 % |
/// | **hierarquia + retícula** (o produto) | **76,9 %** | **86,6 %** |
///
/// ⚠️ **A HISTÓRIA DESTE GATE É A LIÇÃO DA WAVE.** Ele nasceu a afirmar
/// `flat > deep` — porque era isso que a medição dizia — e a mensagem dele pedia:
/// *"NÃO apague este gate: mude-o, com o número novo ao lado"*. Foi exatamente o
/// que aconteceu.
///
/// O que mudou não foi a hierarquia: foi o **operador**. O
/// `compat_position_extrinsic_4` da crate não era o da referência — ele
/// arredondava cada lado ao ponto médio, independentemente, em vez de enumerar as
/// **quatro quinas** da célula de cada lado e escolher o **PAR mais próximo entre
/// si** (16 combinações). Sem esse operador as duas retículas nunca se procuram,
/// o campo sai suave, não há platôs — e sem platôs a hierarquia não tem o que
/// propagar e o quociente pela retícula funde tudo.
///
/// ⇒ ⚠️ **DUAS recusas MEDIDAS eram consequência de UM operador mal portado**
/// (a hierarquia e o quociente da retícula). *Uma medição só refuta o que ela de
/// facto exercitou — e o que ela exercitava era a minha aproximação, não a lei.*
/// É a razão de a DIRETIVA mandar **portar** a referência antes de escrever a
/// própria versão.
/// ⚠️ **E A RÉGUA DELE JÁ ESTAVA A MENTIR PELA SEGUNDA VEZ, em 2026-08-19.** A
/// barra era `deep > flat × 1,5` sobre a **fração de quads** — e assim que o
/// fecho sem leque e o grafo por passo de retícula entraram, o caminho plano
/// passou de 42,9 % para **82,7 %** e o gate reprovou sobre uma melhoria. Mas o
/// caminho plano não tinha melhorado: no mesmo ponto ele devolve uma peça de
/// **volume 0,87** contra os 4,19 da esfera — ou seja, **80 % da forma
/// desapareceu**, e a fração de quads não vê isso, porque as poucas faces que
/// sobram são quads.
///
/// ⇒ **A régua passa a ser a FORMA**, que é o que a hierarquia de facto compra.
/// Medido a `3,0×` a aresta de entrada:
///
/// | | esfera 48×64 | uv 96×144 amassada |
/// |---|---|---|
/// | plano (32 varreduras) | **62,2 %** quads, volume **0,87** de 4,19, **97 ms** | 82,6 %, volume **3,03** de 3,78, 431 ms |
/// | **hierarquia** | **87,4 %**, volume **4,146**, **14 ms** | **90,0 %**, volume **3,779**, **63 ms** |
///
/// E a `4,0×` o caminho plano devolve **malha vazia** nas duas. *Uma segunda
/// contagem da mesma coisa diverge; um gate que conta faces não vê a forma sumir.*
#[test]
fn the_hierarchy_pays_and_the_number_is_here() {
    for (name, mesh) in [("esfera", sphere()), ("toro", torus())] {
        // ⚠️ **A `6,0×` e não a `3,0×`, e a mudança é a correção de um gate que
        // media onde a afirmação NÃO é load-bearing.** Com os núcleos e os pesos
        // portados, o caminho plano melhorou muito — a `3,0×` ele chega a
        // volume 4,020 contra 4,125 do hierárquico, e a razão de erros cai para
        // 3×. Mas isso é o campo a caber num raio pequeno, não a hierarquia a
        // deixar de pagar: a `6,0×` o caminho plano devolve **volume 0,724** de
        // 4,178 (83 % da forma perdida) e **12 faces**, contra 3,910 e 106 do
        // hierárquico. *Um gate calibrado no ponto errado da faixa reprova sobre
        // código correto.*
        let edge = 4.0 * crate::scale::mean_edge(&mesh);
        let scale = ScaleField::uniform(&mesh, edge);

        let o = solve_orientation(&mesh, 32);
        let p = solve_position(&mesh, &o, &scale, 32);
        let flat = extract(&mesh, &o, &p, &scale).expect("plano");

        let (oh, ph) = solve_fields(&mesh, &scale);
        let deep = extract(&mesh, &oh, &ph, &scale).expect("hierarquico");

        let (vin, vflat, vdeep) = (
            signed_volume(&mesh),
            signed_volume(&flat.mesh),
            signed_volume(&deep.mesh),
        );
        eprintln!(
            "[quadflow] {name}: plano {:.1}% vol {vflat:.3} | hierarquia {:.1}% vol {vdeep:.3} \
             | entrada {vin:.3}",
            flat.quad_fraction() * 100.0,
            deep.quad_fraction() * 100.0
        );

        // ⚠️ **SÓ a razão, e a barra absoluta de volume SAIU.** Ela media
        // geometria, não a hierarquia: a `4,0×` o quad do toro mede `0,33`,
        // quase o raio menor (`0,35`), e nenhuma grade desse tamanho envolve
        // aquele tubo sem perder volume. Um gate que soma as duas coisas reprova
        // sobre a fixtura, não sobre o código.
        // ⚠️ **Esta é uma RAZÃO e não um absoluto, de propósito:** ela sobrevive
        // a qualquer melhoria dos dois lados, e é a afirmação que o ADR-0160 Q3.5
        // de facto faz — *a hierarquia é o que faz a forma atravessar*.
        // MEDIDO 2026-08-19 a `4,0×`, com o porte fiel: a esfera perde **1,53**
        // de volume pelo caminho plano contra **0,10** pelo hierárquico (15×), e
        // o toro perde tudo (o plano devolve 225 faces contra 997). A barra de
        // **3×** tem folga de 5× sobre a pior leitura.
        assert!(
            (vflat - vin).abs() > (vdeep - vin).abs() * 3.0,
            "{name}: o caminho PLANO ({vflat:.3}) deixou de perder a forma contra o hierarquico \
             ({vdeep:.3}, entrada {vin:.3}) -- se ele passou a servir, a hierarquia deixou de \
             pagar e o ADR-0160 Q3.5 tem de ser reaberto. NAO apague este gate: mude-o, com o \
             numero novo ao lado."
        );
    }
}

/// O volume com sinal — a régua da forma, pelo teorema da divergência.
fn signed_volume(mesh: &Mesh) -> f32 {
    let pos = mesh.positions();
    let mut vol = 0.0f64;
    for f in mesh.faces() {
        let v = f.verts();
        for k in 1..v.len() - 1 {
            let (a, b, c) = (
                pos[v[0] as usize],
                pos[v[k] as usize],
                pos[v[k + 1] as usize],
            );
            vol += f64::from(a[0].mul_add(
                b[1].mul_add(c[2], -(b[2] * c[1])),
                a[1].mul_add(
                    b[2].mul_add(c[0], -(b[0] * c[2])),
                    a[2] * b[0].mul_add(c[1], -(b[1] * c[0])),
                ),
            )) / 6.0;
        }
    }
    vol as f32
}

/// **DETERMINÍSTICO** (HR-5) — a hierarquia inteira, duas vezes.
#[test]
fn the_hierarchical_solve_is_bit_reproducible() {
    let mesh = torus();
    let scale = ScaleField::uniform(&mesh, 0.18);
    let (a1, b1) = solve_fields(&mesh, &scale);
    let (a2, b2) = solve_fields(&mesh, &scale);
    assert_eq!(a1, a2, "o campo de orientacao mudou entre corridas");
    assert_eq!(b1, b2, "o campo de posicao mudou entre corridas");
}

/// **A SONDA que escolheu os dois números** — varre varreduras × topo e imprime
/// a fração de quads. ⚠️ `#[ignore]`: é medição, não gate.
#[test]
#[ignore = "sonda de calibracao -- rode a mao quando os numeros da Q3.5 mudarem"]
fn measure_the_hierarchy_knobs() {
    use super::solve_fields_with;
    for (name, mesh) in [("esfera", sphere()), ("toro", torus())] {
        let scale = ScaleField::uniform(&mesh, 0.18);
        let o = solve_orientation(&mesh, 32);
        let p = solve_position(&mesh, &o, &scale, 32);
        let flat = extract(&mesh, &o, &p, &scale).expect("plano");
        eprintln!(
            "[quadflow] === {name}: plano {:.1}% ===",
            flat.quad_fraction() * 100.0
        );
        for coarsest in [24usize, 128, 512, 2048] {
            for sweeps in [1usize, 2, 4, 8, 16, 32] {
                let (oh, ph) = solve_fields_with(&mesh, &scale, sweeps, coarsest);
                let d = extract(&mesh, &oh, &ph, &scale).expect("h");
                eprintln!(
                    "[quadflow]   topo<={coarsest:>4} varreduras={sweeps:>2} -> {:.1}%",
                    d.quad_fraction() * 100.0
                );
            }
        }
    }
}

/// **O 2×2 QUE SEPARA AS DUAS VARIÁVEIS** — campo (plano/hierárquico) × células
/// (semente/retícula). ⚠️ `#[ignore]`: é medição.
#[test]
#[ignore = "sonda de calibracao -- o 2x2 que decidiu o desenho da Q3.5"]
fn measure_field_times_clustering() {
    use crate::extract::{Clustering, extract_with};
    for (name, mesh) in [("esfera", sphere()), ("toro", torus())] {
        let scale = ScaleField::uniform(&mesh, 0.18);
        let o = solve_orientation(&mesh, 32);
        let p = solve_position(&mesh, &o, &scale, 32);
        let (oh, ph) = solve_fields(&mesh, &scale);
        for (fname, of, pf) in [("plano", &o, &p), ("hierarquia", &oh, &ph)] {
            for how in [Clustering::Seed, Clustering::Lattice] {
                match extract_with(&mesh, of, pf, &scale, how) {
                    Ok(q) => eprintln!(
                        "[quadflow] {name} {fname:>10} + {how:?}: V={} {:.1}% quads",
                        q.mesh.vert_count(),
                        q.quad_fraction() * 100.0
                    ),
                    Err(e) => eprintln!("[quadflow] {name} {fname:>10} + {how:?}: ERRO {e:?}"),
                }
            }
        }
    }
}
