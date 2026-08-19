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
#[test]
fn the_hierarchy_pays_and_the_number_is_here() {
    let mesh = sphere();
    let scale = ScaleField::uniform(&mesh, 0.18);

    let o = solve_orientation(&mesh, 32);
    let p = solve_position(&mesh, &o, &scale, 32);
    let flat = extract(&mesh, &o, &p, &scale).expect("plano");

    let (oh, ph) = solve_fields(&mesh, &scale);
    let deep = extract(&mesh, &oh, &ph, &scale).expect("hierarquico");

    eprintln!(
        "[quadflow] plano {:.1}% | hierarquia {:.1}%",
        flat.quad_fraction() * 100.0,
        deep.quad_fraction() * 100.0
    );
    assert!(
        deep.quad_fraction() > flat.quad_fraction() * 1.5,
        "a hierarquia ({:.1}%) deixou de ganhar do caminho plano ({:.1}%) com folga -- MEDIDO em \
         2026-08-19: 76,9% contra 42,9%. NAO apague este gate: mude-o, com o numero novo ao lado.",
        deep.quad_fraction() * 100.0,
        flat.quad_fraction() * 100.0
    );
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
