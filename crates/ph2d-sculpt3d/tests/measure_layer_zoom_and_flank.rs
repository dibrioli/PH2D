//! **O ZOOM E O FLANCO** — os dois reports do Enio de 2026-08-17, medidos antes
//! de qualquer hipótese.
//!
//! ```text
//! cargo test -p ph2d-sculpt3d --release --test measure_layer_zoom_and_flank \
//!   -- --ignored --nocapture --test-threads=1
//! ```
//!
//! > *"Quanto mais se aproxima do objeto (zoom), pior o resultado."*
//! > *"Veja se nosso algoritmo da incidência da ferramenta é tão bom quanto
//! > SculptGL e Blender que usam as normais e mesmo nas laterais de um objeto
//! > esférico conseguem bom resultado."*
//!
//! ⚠️ **Nenhuma câmera entra aqui, e é de propósito.** O raio do pincel é medido
//! em pixels de tela e convertido a mundo por uma conta **LINEAR na
//! profundidade** (`Camera::world_radius_for_screen_px`), então *aproximar 4×* é
//! exatamente *raio de mundo 4× menor* — e o que a sonda precisa varrer é o raio,
//! não a câmera. Trazer uma câmera acrescentaria um wgpu ao build para reproduzir
//! uma proporcionalidade.
//!
//! ⚠️ **A `layer_height` é ABSOLUTA de mundo nos DOIS lados** — o
//! `layer.cc:101` lê `brush.height` cru e o RNA declara-a `PROP_DISTANCE`
//! (`rna_brush.cc:3230`, faixa `0..1`, default `0.5`) —, então a razão
//! *altura ÷ raio* é o número que o zoom move. É ele que esta sonda persegue.

use ph2d_mesh::{Mesh, shapes};
use ph2d_sculpt3d::{Brush, Dab, RefMode, SculptStroke, Symmetry, Verb};

fn sphere() -> Mesh {
    shapes::sculpt_sphere(1.0)
}

fn len(q: [f32; 3]) -> f32 {
    (q[0] * q[0] + q[1] * q[1] + q[2] * q[2]).sqrt()
}

fn dist3(a: [f32; 3], b: [f32; 3]) -> f32 {
    len([a[0] - b[0], a[1] - b[1], a[2] - b[2]])
}

/// O espaçamento MEDIANO de aresta — a régua contra a qual todo raio é julgado.
fn median_edge(mesh: &Mesh) -> f32 {
    let (pos, adj) = (mesh.positions(), mesh.adjacency());
    let mut e: Vec<f32> = Vec::new();
    for (i, p) in pos.iter().enumerate().step_by(7) {
        for &j in adj.vert_verts.neighbours(i) {
            e.push(dist3(*p, pos[j as usize]));
        }
    }
    e.sort_by(|a, b| a.partial_cmp(b).unwrap());
    e[e.len() / 2]
}

fn brush(radius: f32, height: f32) -> Brush {
    Brush {
        verb: Verb::Layer,
        mode: RefMode::B,
        radius,
        strength: 0.7,
        hardness: 0.4,
        auto_smooth: 0.0,
        layer_height: height,
        falloff: Verb::Layer.default_falloff(RefMode::B),
        ..Brush::default()
    }
}

/// Esfrega `dabs` vezes no MESMO ponto e devolve `(rest, mesh)`.
fn coat(centre: [f32; 3], eye: [f32; 3], radius: f32, height: f32, dabs: usize) -> (Mesh, Mesh) {
    let rest = sphere();
    let mut mesh = sphere();
    let b = brush(radius, height);
    let mut s = SculptStroke::default();
    s.begin(&mesh);
    for _ in 0..dabs {
        s.dab(
            &mut mesh,
            &b,
            &Dab::at(centre, radius, eye),
            Symmetry::default(),
        );
    }
    (rest, mesh)
}

// ---------------------------------------------------------------------------
// (A) O ZOOM
// ---------------------------------------------------------------------------

/// Aproximar a câmera **encolhe o raio de mundo** e deixa a altura onde estava.
/// A sonda varre o raio com a altura FIXA e mede três coisas independentes:
///
/// * **quantos vértices** a pegada alcança — abaixo de um punhado, o dab deixa
///   de ser uma forma e vira um puxão de vértice;
/// * a razão **altura ÷ raio**, a esbeltez da demão: `1` é um cone, `0,1` é um
///   degrau suave;
/// * o **raio em ARESTAS**, que diz se a malha tem resolução para o desenhar.
#[test]
#[ignore]
fn measure_what_the_zoom_does_to_the_coat() {
    let m = sphere();
    let e = median_edge(&m);
    println!("\n  == (A) O ZOOM: raio de mundo encolhe, a altura fica ==");
    println!("  esfera de fabrica: {} vertices", m.vert_count());
    println!("  aresta mediana: {e:.5}");
    println!("  altura da demao FIXA em 0,1 (o nosso default de hoje)\n");
    println!(
        "  {:>7}  {:>8}  {:>7}  {:>8}  {:>9}  {:>9}",
        "raio", "r/aresta", "verts", "alt/raio", "alt.med", "ondulac."
    );
    println!(
        "  {:->7}  {:->8}  {:->7}  {:->8}  {:->9}  {:->9}",
        "", "", "", "", "", ""
    );

    let centre = [0.0, 0.0, 1.0];
    let eye = [0.0, 0.0, -1.0];
    for r in [0.60f32, 0.40, 0.25, 0.15, 0.09, 0.05, 0.03] {
        let (rest, mesh) = coat(centre, eye, r, 0.1, 12);
        let inner = 0.4 * r; // o disco onde o hardness 0,4 satura a curva
        let mut hs: Vec<f32> = Vec::new();
        let mut n_foot = 0usize;
        for (i, p) in mesh.positions().iter().enumerate() {
            let p0 = rest.positions()[i];
            let d = dist3(p0, centre);
            if d <= r {
                n_foot += 1;
            }
            if d <= inner {
                hs.push(len(*p) - len(p0));
            }
        }
        let (mean, ripple) = if hs.is_empty() {
            (f32::NAN, f32::NAN)
        } else {
            let mean = hs.iter().sum::<f32>() / hs.len() as f32;
            let (lo, hi) = hs
                .iter()
                .fold((f32::MAX, f32::MIN), |(a, b), &v| (a.min(v), b.max(v)));
            (mean, hi - lo)
        };
        println!(
            "  {r:>7.3}  {:>8.2}  {n_foot:>7}  {:>8.3}  {mean:>9.5}  {ripple:>9.5}",
            r / e,
            0.1 / r
        );
    }
    println!(
        "\n  LEITURA: a coluna `alt/raio` e' a ESBELTEZ. Se ela cresce sem teto,\n  \
         aproximar transforma a demao num ESPIGAO — e a cura e' a altura seguir\n  \
         o raio, nao um numero de mundo fixo."
    );
}

// ---------------------------------------------------------------------------
// (B) O FLANCO
// ---------------------------------------------------------------------------

/// O dab caminha do POLO ao EQUADOR com a câmera parada, e a sonda pergunta o
/// que a referência garante e nós talvez não: *quanto do que se moveu está do
/// lado que o artista vê?*
///
/// ⚠️ **O `eye` é fixo** — é a câmera que não se mexe enquanto a mão desce pelo
/// flanco, que é exactamente o gesto do report.
#[test]
#[ignore]
fn measure_what_the_flank_does_to_the_coat() {
    println!("\n  == (B) O FLANCO: o dab desce do polo ao equador ==");
    println!("  camera parada em -Z; a esfera tem raio 1.\n");
    println!(
        "  {:>6}  {:>7}  {:>7}  {:>9}  {:>9}  {:>8}",
        "graus", "verts", "atras", "alt.frente", "alt.atras", "cos.inc"
    );
    println!(
        "  {:->6}  {:->7}  {:->7}  {:->9}  {:->9}  {:->8}",
        "", "", "", "", "", ""
    );

    let eye = [0.0, 0.0, -1.0];
    // ⚠️ **O raio é o do PRODUTO, medido pela câmera enquadrada** (50 px numa
    // tela de 1080 sobre a esfera de fábrica): a primeira versão desta sonda
    // usava 0,30 — quase um terço do modelo —, e um dab desse tamanho alcança o
    // outro lado da casca **por geometria**, não por defeito do pincel.
    let r = 0.16f32;
    for deg in [0.0f32, 30.0, 60.0, 75.0, 85.0, 90.0] {
        let a = deg.to_radians();
        let centre = [a.sin(), 0.0, a.cos()];
        let (rest, mesh) = coat(centre, eye, r, 0.1, 12);
        let mut n = 0usize;
        let mut n_back = 0usize;
        let mut front = 0.0f32;
        let mut back = 0.0f32;
        for (i, p) in mesh.positions().iter().enumerate() {
            let p0 = rest.positions()[i];
            if dist3(p0, centre) > r {
                continue;
            }
            n += 1;
            let lift = len(*p) - len(p0);
            // A normal de uma esfera unitária centrada na origem É a posição.
            let facing = -(p0[0] * eye[0] + p0[1] * eye[1] + p0[2] * eye[2]);
            if facing > 0.0 {
                front = front.max(lift);
            } else {
                n_back += 1;
                back = back.max(lift);
            }
        }
        // O cosseno de incidência no CENTRO do dab: 1 de frente, 0 de perfil.
        let inc = -(centre[0] * eye[0] + centre[1] * eye[1] + centre[2] * eye[2]);
        println!("  {deg:>6.0}  {n:>7}  {n_back:>7}  {front:>9.5}  {back:>9.5}  {inc:>8.4}");
    }
    println!(
        "\n  LEITURA: `atras` > 0 com `alt.atras` > 0 quer dizer que a demao\n  \
         atravessa a peca e levanta a casca do outro lado — o que o front-face\n  \
         do `layer.cc` existe para impedir, e que nasce DESLIGADO na referencia."
    );
}
