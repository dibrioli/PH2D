//! ⭐⭐ **QUANTO CUSTA UM PINCEL DO TAMANHO DA PEÇA** — a medição que decide o tecto do raio.
//!
//! Report do dono (2026-09-16), sobre o pincel de plano: *«O radius máximo permitido é pouco»*, e
//! logo a seguir *«o melhor jeito de ver o efeito é com o pincel bem grande, do tamanho da peça a
//! esculpir»*.
//!
//! ```text
//! cargo test -p ph2d-sculpt3d --release --test it mede_o_raio_do_pincel -- --ignored --nocapture
//! ```
//!
//! # A pergunta (`CLAUDE.md` §0.0)
//!
//! O tecto de então era `1/8` da altura do viewport, e o doc dele dizia de que era: *«acima de
//! meio modelo o pincel é um deformador global, que é outra ferramenta»*. ⛔ **Isso é uma opinião
//! de produto, não um recurso**, e o dono desmentiu-a com o uso. Um limite legítimo diz de que
//! recurso ele é e traz a medição ao lado — e o único recurso que um raio maior gasta é o
//! **trabalho por dab**. Esta sonda mede-o, pela porta do produto (`SculptStroke::dab`), com a
//! pegada a crescer até cobrir a peça inteira.
//!
//! # ⭐ Porque o custo não pode explodir, e o que a sonda confirma
//!
//! - a **pegada SATURA na malha**: acima do diâmetro da peça o pincel não tem mais vértice nenhum
//!   para tocar — o dab mais caro possível é o que toca a peça inteira;
//! - o **espaçamento é proporcional ao raio** (`MIN_SPACING_FRACTION` do raio em pixels), logo um
//!   pincel oito vezes maior deposita **oito vezes menos** dabs pelo mesmo arrasto.
//!
//! ⚠️ **`--release` não é preferência** (o motivo do `measure_brush_kernel`), e ⚠️ **leia o
//! `load` ao lado**: é um relógio, e nenhum relógio desta workstation vale acima de `load ~5`.
//! Por isso a asserção é sobre a CONTAGEM (a saturação), nunca sobre os milissegundos.

use std::time::Instant;

use ph2d_mesh::{Mesh, shapes};
use ph2d_sculpt3d::{Brush, Dab, SculptStroke, Symmetry, Verb};

/// O orçamento de um dab (o K1 do `docs/3D/03.5`, o mesmo do `measure_brush_kernel`).
const K1_BUDGET_MS: f64 = 8.0;

/// O olho de fora, a olhar direto para o dab (a mesma convenção do `measure_brush_kernel`).
fn eye_towards(c: [f32; 3]) -> [f32; 3] {
    let l = (c[0] * c[0] + c[1] * c[1] + c[2] * c[2]).sqrt();
    if l > 1e-6 {
        [-c[0] / l, -c[1] / l, -c[2] / l]
    } else {
        [0.0, 0.0, -1.0]
    }
}

/// Mediana de `v`, sem a primeira amostra (ela paga o *first-touch* dos buffers).
fn mediana(mut v: Vec<f64>) -> f64 {
    if v.len() > 1 {
        v.remove(0);
    }
    v.sort_by(f64::total_cmp);
    v[v.len() / 2]
}

/// Um dab do `verbo` de raio `raio` (em raios da esfera, que tem raio `1`) — o custo e quantos
/// vértices ele moveu. O traço recomeça a cada amostra: o dab de trabalho CHEIO é o limite
/// superior (o `measure_brush_kernel` diz porquê).
///
/// ⚠️ **O pincel de plano é inerte no primeiro dab de uma passagem** (espec §1: falta a direcção
/// do traço), então ele é ARMADO com um dab ao lado antes do que se mede — sem isso a sonda
/// cronometraria o dab que não faz nada.
fn um_dab(mesh: &mut Mesh, verbo: Verb, raio: f32, amostras: usize) -> (f64, usize) {
    let pincel = Brush {
        verb: verbo,
        radius: raio,
        // Força minúscula: a sonda mede CUSTO, e deformar mudaria a vizinhança do dab seguinte.
        strength: 1e-4,
        ..Brush::default()
    };
    let n = mesh.vert_count();
    let mut stroke = SculptStroke::default();
    let (mut ms, mut movidos) = (Vec::with_capacity(amostras), 0usize);
    for k in 0..amostras {
        let c = mesh.positions()[(k * 7919 + n / 3) % n];
        stroke.begin(mesh);
        if verbo.exige_esfregar() {
            let ao_lado = [c[0] + 0.05, c[1], c[2]];
            stroke.dab(
                mesh,
                &pincel,
                &Dab::at(ao_lado, raio, eye_towards(ao_lado)),
                Symmetry::default(),
            );
        }
        let t = Instant::now();
        let m = stroke.dab(
            mesh,
            &pincel,
            &Dab::at(c, raio, eye_towards(c)),
            Symmetry::default(),
        );
        ms.push(t.elapsed().as_secs_f64() * 1e3);
        movidos = movidos.max(m);
    }
    (mediana(ms), movidos)
}

#[test]
#[ignore = "medição de relógio: corra em --release com a máquina calma"]
fn mede_o_raio_do_pincel() {
    let load = std::fs::read_to_string("/proc/loadavg").unwrap_or_default();
    println!(
        "\n=== ph2d-sculpt3d :: o custo de UM dab contra o RAIO (load {}) ===",
        load.trim()
    );
    println!(
        "{:>10} {:>8} {:>6} {:>10} {:>10} {:>9}",
        "triangulos", "verbo", "raio", "vertices", "% da peca", "dab ms"
    );
    const RAIOS: [f32; 6] = [0.125, 0.25, 0.5, 1.0, 2.5, 4.0];
    let mut pior = (0.0f64, String::new());
    for alvo in [40_000usize, 400_000, 3_000_000] {
        let mut mesh = shapes::sphere_with_triangles(alvo, 1.0);
        let (tris, verts) = (mesh.triangle_count(), mesh.vert_count());
        for verbo in [Verb::Draw, Verb::Smooth, Verb::Plane] {
            let mut saturado = Vec::new();
            for raio in RAIOS {
                let (ms, movidos) = um_dab(&mut mesh, verbo, raio, 5);
                #[allow(clippy::cast_precision_loss)]
                let frac = 100.0 * movidos as f64 / verts as f64;
                println!(
                    "{tris:>10} {:>8} {raio:>6.3} {movidos:>10} {frac:>9.1}% {ms:>9.3}",
                    verbo.label()
                );
                if ms > pior.0 {
                    pior = (
                        ms,
                        format!("{} a {tris} triangulos, raio {raio}", verbo.label()),
                    );
                }
                // ⚠️ **No pincel de plano a CONTAGEM não é a pegada**: ele move só o que fica do
                // lado errado do plano, e o plano depende do raio até a amostragem da área cobrir
                // a peça (medido: 4351 a 2,5 e 4232 a 4,0). Ali a régua é a coluna do custo.
                if raio >= 2.5 && verbo != Verb::Plane {
                    saturado.push(movidos);
                }
            }
            // ⭐ **A PEGADA SATURA**: acima do diâmetro da peça não há mais vértice nenhum.
            assert!(
                saturado.windows(2).all(|w| w[0] == w[1]),
                "{}: a pegada continuou a crescer acima do diametro da peca ({saturado:?}) — \
                 o custo de um pincel enorme deixaria de ser limitado pela malha",
                verbo.label()
            );
        }
    }
    println!(
        "\nPIOR dab: {:.3} ms ({}) contra o K1 de {K1_BUDGET_MS} ms",
        pior.0, pior.1
    );
}
