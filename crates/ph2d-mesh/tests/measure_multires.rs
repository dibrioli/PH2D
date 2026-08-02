//! **A sonda da multiresolução** — o preço de trocar de nível, e a pergunta que
//! decide se divergimos do original.
//!
//! ⚠️ **A pergunta:** o frame do detalhe usa a NORMAL do nível de cima, e ela não
//! é recomputada entre descer e subir — é isso que torna a viagem exata. Mas
//! significa que, se o artista DOBRAR a base enquanto está embaixo, o detalhe
//! volta num frame que descreve a superfície de antes: a tangente segue a base
//! nova (ela sai da previsão), a normal não. **Quanto ele inclina?** Se a
//! resposta for "nada que se veja", portar é o certo; se for grande, há uma
//! divergência a escrever — derivar a normal da PREVISÃO manteria a viagem
//! exata E faria o frame seguir a base.
//!
//! ```text
//! cargo test -p ph2d-mesh --release --test measure_multires -- --ignored --nocapture
//! ```

use ph2d_mesh::{Mesh, Multires, predict, shapes};
use std::time::Instant;

fn ms(f: impl FnOnce()) -> f64 {
    let t = Instant::now();
    f();
    t.elapsed().as_secs_f64() * 1000.0
}

fn norm(v: [f32; 3]) -> f32 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

/// O ângulo, em graus, entre o deslocamento do detalhe e a normal da superfície
/// prevista naquele vértice — **zero significa que o detalhe sai perpendicular à
/// pele**, que é onde o artista o pôs.
fn lean_degrees(m: &Multires, base: &Mesh, v: usize) -> f32 {
    let p = predict(base);
    let d = [
        m.mesh().positions()[v][0] - p.positions[v][0],
        m.mesh().positions()[v][1] - p.positions[v][1],
        m.mesh().positions()[v][2] - p.positions[v][2],
    ];
    // A normal da superfície PREVISTA: a malha do topo com as posições da
    // previsão.
    let mut smooth = m.mesh().clone();
    smooth.positions_mut().copy_from_slice(&p.positions);
    smooth.rebuild();
    let n = smooth.normals()[v];
    let (dl, nl) = (norm(d), norm(n));
    if dl < 1e-6 || nl < 1e-6 {
        return 0.0;
    }
    let c = ((d[0] * n[0] + d[1] * n[1] + d[2] * n[2]) / (dl * nl)).clamp(-1.0, 1.0);
    c.acos().to_degrees()
}

#[test]
#[ignore = "sonda: mede, não afirma"]
fn measure_whether_the_detail_leans_when_the_base_bends() {
    const V: usize = 200;
    println!("\n  O DETALHE INCLINA quando a base dobra?");
    println!("  (esfera 12x18, nível 1; um vértice empurrado 0,25 na normal)");
    println!("   o que se faz na base            inclinação    comprimento");

    for (label, bend) in [
        ("nada", 0.0f32),
        ("dobra suave  (0,10)", 0.10),
        ("dobra media  (0,25)", 0.25),
        ("dobra forte  (0,50)", 0.50),
    ] {
        let mut m = Multires::new(shapes::uv_sphere(12, 18, 1.0));
        m.add_level();
        // O detalhe.
        let n = m.mesh().normals()[V];
        let p = m.mesh().positions()[V];
        m.mesh_mut().positions_mut()[V] =
            [p[0] + n[0] * 0.25, p[1] + n[1] * 0.25, p[2] + n[2] * 0.25];
        m.mesh_mut().rebuild();

        m.lower();
        // Dobra a base: um cisalhamento proporcional a `y`, que gira a
        // superfície localmente sem transladá-la.
        for q in m.mesh_mut().positions_mut() {
            q[0] += bend * q[1];
        }
        m.mesh_mut().rebuild();
        let base = m.mesh().clone();
        m.higher();

        let d = [
            m.mesh().positions()[V][0] - predict(&base).positions[V][0],
            m.mesh().positions()[V][1] - predict(&base).positions[V][1],
            m.mesh().positions()[V][2] - predict(&base).positions[V][2],
        ];
        println!(
            "  {label:<30}  {:>9.2}°   {:>11.5}",
            lean_degrees(&m, &base, V),
            norm(d)
        );
    }
    println!();
}

#[test]
#[ignore = "sonda: mede, não afirma"]
fn measure_the_cost_of_switching_levels() {
    println!("\n  TROCAR DE NÍVEL — o preço por tamanho do topo");
    println!("   topo V      descer (ms)   subir (ms)   subdividir (ms)");
    for (rings, segs) in [(16, 24), (32, 48), (64, 96), (96, 144)] {
        let mut m = Multires::new(shapes::uv_sphere(rings, segs, 1.0));
        let t_add = ms(|| {
            m.add_level();
        });
        let top = m.mesh().vert_count();
        // Aquece.
        m.lower();
        m.higher();
        let t_down = ms(|| {
            m.lower();
        });
        let t_up = ms(|| {
            m.higher();
        });
        println!("  {top:>8}   {t_down:>11.2}   {t_up:>10.2}   {t_add:>15.2}");
    }
    println!();
}

#[test]
#[ignore = "sonda: mede, não afirma"]
fn measure_what_the_stack_costs_in_memory() {
    println!("\n  A PILHA — o que os DETALHES custam além das malhas");
    println!("   níveis     topo V     detalhes (MB)");
    let mut m = Multires::new(shapes::uv_sphere(32, 48, 1.0));
    for level in 0..4 {
        println!(
            "  {:>7}   {:>8}   {:>14.2}",
            level + 1,
            m.mesh().vert_count(),
            m.detail_bytes() as f64 / (1024.0 * 1024.0)
        );
        if level < 3 {
            m.add_level();
            // Um detalhe qualquer, para o encode alocar de verdade.
            m.lower();
            m.higher();
        }
    }
    println!();
}
