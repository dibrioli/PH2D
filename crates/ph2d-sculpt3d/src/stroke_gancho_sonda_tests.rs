//! ⛔⛔⛔⛔ **A PERGUNTA QUE O REPORT *«algumas vezes correto, algumas vezes
//! bugado»* (2026-09-19) OBRIGA A FAZER: alguma condição da máscara ainda tira
//! barro que JÁ ESTÁ A ANDAR?**
//!
//! Porque é isso, e só isso, que rasga: *quem já andava pára enquanto o vizinho
//! continua*. A cura de hoje deu memória à condição da **normal**; as outras
//! duas — o tecto do passeio e a razão `superfície/ar` — continuam a julgar
//! **vivo**, e num gancho a peça muda de forma debaixo delas.
//!
//! ⚠️ **Esta sonda mora como filha do [`super`] porque o `alcance` é campo
//! privado do traço** — ela lê o contador que a
//! [`crate::dab_alcance::Alcance`] mantém sob `cfg(test)`, que é a única
//! maneira de separar *«a máscara cortou»* de *«a máscara cortou o que já se
//! mexia»*.

use super::*;

fn esfera() -> ph2d_mesh::Mesh {
    ph2d_mesh::shapes::sculpt_sphere(1.0)
}

fn norma(v: [f32; 3]) -> f32 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

/// Um arrasto de gancho, e o que a máscara tira a quem já anda, dab a dab.
/// Devolve `[tecto, razão, normal]` somados sobre o traço.
fn tirado_ao_traco(verbo: Verb, raio: f32, len: f32, dabs: usize, rumo: [f32; 3]) -> [usize; 3] {
    let mut m = esfera();
    let b = Brush {
        verb: verbo,
        radius: raio,
        strength: 1.0,
        surface_only: true,
        ..Brush::default()
    };
    let mut st = SculptStroke::default();
    st.begin(&m);
    let n = norma(rumo).max(1e-9);
    let u = [rumo[0] / n, rumo[1] / n, rumo[2] / n];
    let passo = len / dabs as f32;
    let mut centro = [0.0f32, 0.0, 1.0];
    let mut soma = [0usize; 3];
    for _ in 0..dabs {
        centro = [
            centro[0] + u[0] * passo,
            centro[1] + u[1] * passo,
            centro[2] + u[2] * passo,
        ];
        st.dab(
            &mut m,
            &b,
            &Dab::hooking(
                centro,
                raio,
                [0.0, 0.0, -1.0],
                [u[0] * passo, u[1] * passo, u[2] * passo],
            ),
            Symmetry::default(),
        );
        let t = st.alcance.tirou_do_traco_no_teste();
        for k in 0..3 {
            soma[k] += t[k];
        }
    }
    soma
}

/// ⭐⭐⭐⭐ **A TABELA QUE SEPARA AS TRÊS CONDIÇÕES** — quantos vértices que o
/// traço já capturava cada uma retirou.
///
/// ```text
/// bash scripts/ph2d-run.sh cargo test -p ph2d-sculpt3d --lib \
///   diag_quem_tira_barro_que_ja_anda -- --ignored --nocapture --test-threads=1
/// ```
#[test]
#[ignore = "sonda: imprime a tabela da atribuicao, nao afirma nada"]
fn diag_quem_tira_barro_que_ja_anda() {
    println!("\n== QUEM TIRA BARRO QUE JA' ANDA (somado sobre o traco) ==");
    println!("   esfera de escultura · mascara LIGADA · olho em -z\n");
    println!(
        "{:>12} {:>6} {:>6} {:>6} {:>12} | {:>7} {:>7} {:>7}",
        "verbo", "raio", "len", "dabs", "rumo", "tecto", "razao", "normal"
    );
    println!("{}", "-".repeat(78));
    let rumos: [(&str, [f32; 3]); 3] = [
        ("tangencial", [1.0, 0.0, 0.0]),
        ("45° p/ olho", [1.0, 0.0, 1.0]),
        ("obliquo", [1.0, 0.6, 0.35]),
    ];
    for verbo in [Verb::SnakeHook, Verb::Move] {
        for raio in [0.12f32, 0.25, 0.45] {
            for (nome, rumo) in rumos {
                for (len, dabs) in [(0.9f32, 24usize), (2.4, 64)] {
                    let t = tirado_ao_traco(verbo, raio, len, dabs, rumo);
                    let marca = if t.iter().any(|&x| x > 0) { " ⛔" } else { "" };
                    println!(
                        "{:>12} {raio:>6.2} {len:>6.2} {dabs:>6} {nome:>12} | {:>7} {:>7} {:>7}{marca}",
                        format!("{verbo:?}"),
                        t[0],
                        t[1],
                        t[2]
                    );
                }
            }
        }
    }
    println!(
        "\n   ⛔ marca as celulas em que a mascara AINDA tira barro que ja' anda —\n   \
         cada uma dessas e' um rasgo no meio do traco."
    );
}
