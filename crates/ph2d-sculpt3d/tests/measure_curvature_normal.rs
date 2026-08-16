//! **A NORMAL POR COTANGENTES VALE UM CHIP?** — a sonda que decide o `l-mode` do
//! [`Verb::Inflate`], antes de ele existir.
//!
//! ```text
//! cargo test -p ph2d-sculpt3d --release --test measure_curvature_normal -- --ignored --nocapture
//! ```
//!
//! O plano §3 fixa a regra: *cada l-mode nasce como CANDIDATO e só ganha o chip
//! depois de MEDIDO contra o b-mode na mesma cena*. O que o Inflate faz hoje é
//! andar pela **normal do vértice** — a média NÃO-PONDERADA das normais das faces
//! do anel, cujo próprio doc-comment em `ph2d-mesh::normals` já nomeia a dívida
//! (*"ponderar por ÁREA seria estritamente melhor em malha irregular … fica
//! nomeado e não feito"*). O candidato é a **normal de curvatura média por
//! cotangentes** (Meyer/Desbrun/Schröder/Barr 2003), que é o operador de
//! Laplace-Beltrami discreto e não o confunde com distribuição de vértices.
//!
//! # As duas perguntas, e por que a segunda não é implicada pela primeira
//!
//! 1. **Quantos GRAUS separam as duas normais?** É o número que decide se existe
//!    l-mode: abaixo do ruído de `f32` ele seria um chip que não muda um pixel.
//! 2. **Numa malha REGULAR elas coincidem?** É o CONTROLE. Sem ele, um ângulo
//!    grande na malha irregular não distingue *o operador é melhor* de *o
//!    operador está errado*.
//!
//! ⚠️ **A fixture tem de conter o fenômeno.** Numa `uv_sphere` o anel de todo
//! vértice é simétrico em azimute, então a média não-ponderada já aponta radial —
//! as duas concordam por SIMETRIA, não por qualidade. A divergência mora onde as
//! faces do anel têm áreas MUITO diferentes, que é o que uma malha esculpida (e
//! qualquer coisa que saia do dyntopo ou do remesh) de facto é.

use ph2d_mesh::{Mesh, shapes};

/// O ângulo, em graus, entre dois unitários.
fn deg_between(a: [f32; 3], b: [f32; 3]) -> f32 {
    let d = (a[0] * b[0] + a[1] * b[1] + a[2] * b[2]).clamp(-1.0, 1.0);
    d.acos().to_degrees()
}

/// Um retrato de uma malha: as duas normais, com o SINAL separado do EIXO.
///
/// ⚠️ **A separação não é cosmética, é o achado.** `K = 2·κ_H·n` carrega a
/// curvatura COM SINAL: numa cova o `κ_H` é negativo e `K` aponta para DENTRO.
/// Um ângulo cru de 179° entre `K` e a normal não diz *o estimador diverge*, diz
/// *este vértice é côncavo* — e misturar as duas leituras é como uma tabela
/// passa a medir a forma da fixture em vez do operador.
struct Portrait {
    answered: usize,
    total: usize,
    /// Quantos vértices o `K` aponta ao CONTRÁRIO da normal — a fração côncava.
    flipped: usize,
    /// O ângulo depois de dobrar o sinal: `min(θ, 180 − θ)`. É este que responde
    /// *"as duas nomeiam o mesmo EIXO?"*.
    axis_mean: f32,
    axis_p95: f32,
    axis_max: f32,
}

fn portrait(m: &Mesh) -> Portrait {
    let mut degs: Vec<f32> = Vec::new();
    let mut flipped = 0usize;
    for v in 0..m.positions().len() {
        let Some(k) =
            ph2d_mesh::curvature_normal_dir_at(m.positions(), m.faces(), m.adjacency(), v)
        else {
            continue;
        };
        let raw = deg_between(k, m.normals()[v]);
        if raw > 90.0 {
            flipped += 1;
        }
        degs.push(raw.min(180.0 - raw));
    }
    degs.sort_by(f32::total_cmp);
    let n = degs.len();
    #[allow(clippy::cast_precision_loss)] // LITERAL-PX-OK: contagem de vértices.
    let mean = if n == 0 {
        0.0
    } else {
        degs.iter().sum::<f32>() / n as f32
    };
    Portrait {
        answered: n,
        total: m.positions().len(),
        flipped,
        axis_mean: mean,
        axis_p95: degs.get(n * 95 / 100).copied().unwrap_or(0.0),
        axis_max: degs.last().copied().unwrap_or(0.0),
    }
}

fn row(name: &str, m: &Mesh) {
    let p = portrait(m);
    #[allow(clippy::cast_precision_loss)] // LITERAL-PX-OK: contagem de vértices.
    let pct = if p.answered == 0 {
        0.0
    } else {
        p.flipped as f32 * 100.0 / p.answered as f32
    };
    println!(
        "{name:<28} {:>6}/{:<6}  concavos {pct:>5.1}%   EIXO: media {:>6.3}°  p95 {:>6.3}°  max {:>7.3}°",
        p.answered, p.total, p.axis_mean, p.axis_p95, p.axis_max
    );
}

/// **O RETRATO** — quanto as duas normais discordam, por fixture.
#[test]
#[ignore = "sonda: imprime, não afirma"]
fn measure_how_far_the_cotangent_normal_is_from_the_vertex_normal() {
    println!("\n=== normal por COTANGENTES contra a normal do VERTICE ===");
    println!("(o CONTROLE e' a esfera regular: ali elas tem de concordar)\n");
    row(
        "uv_sphere 24x32 (controle)",
        &shapes::uv_sphere(24, 32, 1.0),
    );
    row("uv_sphere 8x12  (grossa)", &shapes::uv_sphere(8, 12, 1.0));
    row("sculpt_sphere (o default)", &shapes::sculpt_sphere(1.0));
    row("torus 32x16", &shapes::torus(32, 16, 1.0, 0.35));
    row(
        "uv_sphere_noisy a=0.02",
        &shapes::uv_sphere_noisy(24, 32, 1.0, 0.02),
    );
    row(
        "uv_sphere_noisy a=0.10",
        &shapes::uv_sphere_noisy(24, 32, 1.0, 0.10),
    );
    row(
        "uv_sphere_shuffled",
        &shapes::uv_sphere_shuffled(24, 32, 1.0),
    );
    println!();
}

/// **E A MALHA QUE O ARTISTA DE FACTO TEM** — depois de esculpir, que é onde o
/// anel deixa de ser simétrico.
///
/// ⚠️ **Sem esta linha a sonda mede o catálogo, não o produto:** as fixtures
/// acima nascem de fórmulas, e uma fórmula tende a produzir anéis regulares.
#[test]
#[ignore = "sonda: imprime, não afirma"]
fn measure_the_same_thing_on_a_sculpted_mesh() {
    use ph2d_sculpt3d::{Brush, Dab, SculptStroke, Symmetry, Verb};

    println!("\n=== depois de ESCULPIR (o anel deixa de ser regular) ===\n");
    let mut m = shapes::uv_sphere(32, 48, 1.0);
    row("antes", &m);

    for (i, verb) in [Verb::Draw, Verb::Inflate, Verb::Crease, Verb::Draw]
        .into_iter()
        .enumerate()
    {
        let b = Brush {
            verb,
            radius: 0.6,
            strength: 1.0,
            ..Brush::default()
        };
        #[allow(clippy::cast_precision_loss)] // LITERAL-PX-OK: índice do passo.
        let a = i as f32 * 1.3;
        let dir = [a.cos() * 0.8, a.sin() * 0.6, -1.0];
        let inv = 1.0 / (dir[0] * dir[0] + dir[1] * dir[1] + dir[2] * dir[2]).sqrt();
        let n = [dir[0] * inv, dir[1] * inv, dir[2] * inv];
        let mut s = SculptStroke::default();
        s.begin(&m);
        for step in 0..6 {
            #[allow(clippy::cast_precision_loss)] // LITERAL-PX-OK: índice do passo.
            let t = step as f32 * 0.12 - 0.3;
            let c = [-n[0] + t * 0.4, -n[1] + t, -n[2]];
            let inv = 1.0 / (c[0] * c[0] + c[1] * c[1] + c[2] * c[2]).sqrt();
            let hit = [c[0] * inv, c[1] * inv, c[2] * inv];
            s.dab(&mut m, &b, &Dab::at(hit, 0.6, hit), Symmetry::default());
        }
    }
    row("depois de 4 tracos", &m);
    println!();
}
