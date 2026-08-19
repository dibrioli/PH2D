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

/// ⛔ **MEDIDO E REJEITADO: a hierarquia NÃO paga, e o lever é outro.**
///
/// A Q3 concluiu que a hierarquia multirresolução era pré-requisito da extração.
/// **A medição refutou a conclusão**, e este gate é o registro executável dela.
///
/// | campo × células | esfera | toro |
/// |---|---|---|
/// | plano + semente (**o produto**) | **60,9 %** | **51,9 %** |
/// | hierarquia + semente | 48,8 % | 50,2 % |
/// | plano + retícula | 0 % (1 célula) | 0 % (5 células) |
/// | hierarquia + retícula | 0 % (2 células) | 0 % (9 células) |
///
/// E a varredura de `(topo × varreduras)` — **24 combinações** — nunca passou de
/// **52,3 %**: nenhum ajuste dos dois números faz a hierarquia ganhar.
///
/// ⚠️ **Por que a conclusão da Q3 estava errada:** ela supôs que a extração
/// usava a retícula, e por isso um campo com platôs a ajudaria. Ela **não usa** —
/// o crescimento por semente lê o campo como uma DISTÂNCIA, e um campo mais
/// suave não muda distância nenhuma. *A hipótese não era sobre a hierarquia: era
/// sobre a extração, e eu testei a metade errada.*
///
/// ⚠️ **E o quociente pela retícula — a leitura natural da referência — colapsa**
/// por aritmética, não por afinação: os vértices da entrada distam muito menos
/// que uma célula, então o passo inteiro entre duas retículas vizinhas é `(0,0)`
/// em toda parte.
///
/// ⇒ **O lever da Q4 é a EXTRAÇÃO**, não os campos. A hierarquia fica no repo
/// gateada e correta — ela é o andaime que uma extração baseada em retícula vai
/// precisar —, mas **não está no caminho do produto**, e este gate impede que
/// alguém a ligue sem re-medir.
#[test]
fn the_hierarchy_does_not_pay_yet_and_the_gate_says_so() {
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
        flat.quad_fraction() > deep.quad_fraction(),
        "a hierarquia passou a ganhar do caminho plano ({:.1}% vs {:.1}%) -- a recusa MEDIDA desta \
         wave deixou de valer, e o caminho do produto tem de ser reconsiderado. NAO apague este \
         gate: mude-o, com o numero novo ao lado.",
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
