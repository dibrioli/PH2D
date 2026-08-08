//! **COMO SE MEDE A ESPESSURA DE UM VÉRTICE** — a sonda que escolhe o método
//! antes de qualquer linha de produto existir.
//!
//! ```text
//! cargo test -p ph2d-sdf --release --test measure_thickness -- --ignored --nocapture
//! ```
//!
//! O `docs/3D/05.1` §2b prescreve *"raio para dentro, contra o campo"*, e há
//! **duas** máquinas que sabem responder isso — o `VoxelField` que o
//! [`ph2d_sdf::bake_ao`] já constrói, e o [`ph2d_mesh::Mesh::raycast`] exato.
//! Escolher por prosa seria palpite; numa ESFERA a resposta certa é conhecida
//! (`2r`, ao dígito), então a escolha é uma medição contra um oráculo externo.

use ph2d_mesh::{Ray, shapes};

/// Marcha para dentro contra o campo até sair, e devolve o comprimento andado.
///
/// É a leitura literal do §2b; o passo é o do voxel, então o erro dela é o erro
/// de discretização do campo — que é exatamente o que a sonda quer conhecer.
fn thickness_by_field(field: &ph2d_sdf::VoxelField, p: [f32; 3], n: [f32; 3]) -> f32 {
    let step = field.step();
    let mut d = step * 0.5;
    let far = field.far();
    while d < far {
        let q = [p[0] - n[0] * d, p[1] - n[1] * d, p[2] - n[2] * d];
        if field.sample(q) > 0.0 {
            return d;
        }
        d += step;
    }
    far
}

#[test]
#[ignore = "sonda de medição; roda sob demanda"]
fn measure_how_to_ask_a_vertex_how_thick_it_is() {
    println!("\n=== A ESPESSURA DE UMA ESFERA: o oraculo e' 2r, ao digito ===");
    println!(
        "{:>7} | {:>9} | {:>9} {:>8} | {:>9} {:>8}",
        "raio", "exato 2r", "raycast", "erro", "campo", "erro"
    );
    for &r in &[1.0f32, 0.45, 0.2] {
        let mut m = shapes::uv_sphere(48, 72, r);
        m.triangulate();
        let mut field = ph2d_sdf::VoxelField::for_bounds(m.bounds(), 150);
        field.voxelize(&m);
        field.flood_fill();

        // O vértice do "equador" — qualquer um serve numa esfera, e é isso que
        // torna a fixture um oráculo em vez de um espelho.
        let (mut ray_worst, mut fld_worst) = (0.0f32, 0.0f32);
        let (mut ray_sum, mut fld_sum, mut n) = (0.0f64, 0.0f64, 0usize);
        for v in (0..m.vert_count()).step_by(37) {
            let p = m.positions()[v];
            let nrm = m.normals()[v];
            // ⚠️ O raio nasce um passo PARA DENTRO: partindo da superfície ele
            // acerta a própria face de origem em `t = 0`.
            let eps = r * 1e-3;
            let o = [
                p[0] - nrm[0] * eps,
                p[1] - nrm[1] * eps,
                p[2] - nrm[2] * eps,
            ];
            let dir = [-nrm[0], -nrm[1], -nrm[2]];
            let by_ray = m
                .raycast(&Ray::new(o, dir))
                .map_or(f32::NAN, |h| h.t + eps);
            let by_field = thickness_by_field(&field, p, nrm);
            let exact = 2.0 * r;
            ray_worst = ray_worst.max((by_ray - exact).abs() / exact);
            fld_worst = fld_worst.max((by_field - exact).abs() / exact);
            ray_sum += f64::from(by_ray);
            fld_sum += f64::from(by_field);
            n += 1;
        }
        println!(
            "{r:>7.2} | {:>9.4} | {:>9.4} {:>7.2}% | {:>9.4} {:>7.2}%",
            2.0 * r,
            ray_sum / n as f64,
            100.0 * f64::from(ray_worst),
            fld_sum / n as f64,
            100.0 * f64::from(fld_worst),
        );
    }

    println!("\n=== O QUE exp(-espessura / D) DA' NA ESCADA DA CENA =19 ===");
    println!("  (D = a distancia que a luz percorre dentro do material)");
    print!("{:>8} |", "D");
    for &r in &[1.0f32, 0.45, 0.2] {
        print!(" {:>16}", format!("r={r:.2} (d={:.2})", 2.0 * r));
    }
    println!();
    for &dist in &[0.1f32, 0.25, 0.5, 1.0, 2.0] {
        print!("{dist:>8.2} |");
        for &r in &[1.0f32, 0.45, 0.2] {
            print!(" {:>16.4}", (-(2.0 * r) / dist).exp());
        }
        println!();
    }
    println!(
        "\n  O canal PRE-INTEGRADO nao tem termo assim: ele REDISTRIBUI a luz da\n  \
         frente, e o teto dele e' a media dela (1/pi = 0,3183). A transmitancia\n  \
         SOMA luz onde o lambert e' zero -- e e' por isso que so' ela faz cera."
    );
}

/// **O PROXY GRÁTIS `2/|κ|` CONTRA O RAIO** — a bifurcação de arquitetura.
///
/// A curvatura de mundo já existe por vértice, é **derivada** (nunca envelhece)
/// e numa esfera vale exatamente `1/r`, então `2/|κ|` é a espessura AO DÍGITO
/// sem bake nenhum. Se ela servisse em geral, o canal inteiro sairia de graça.
#[test]
#[ignore = "sonda de medição; roda sob demanda"]
fn measure_the_free_proxy_against_the_ray() {
    println!("\n=== `2/|kappa|` CONTRA O RAIO, por forma ===");
    println!(
        "{:>22} | {:>9} {:>9} | {:>9} {:>9} | {:>8}",
        "forma", "raio med", "raio p90", "proxy med", "proxy p90", "erro med"
    );
    // A CHAPA: um cilindro achatado — o disco fino que é a forma da folha, da
    // orelha e da mão contra a lanterna.
    let mut slab = shapes::cylinder(64, 1.0, 0.1);
    slab.rebuild();
    let cases: Vec<(&str, ph2d_mesh::Mesh)> = vec![
        ("esfera r=1", shapes::uv_sphere(48, 72, 1.0)),
        ("cubo 2x2x2", shapes::cube(2.0)),
        ("toro R=1 r=0.3", shapes::torus(48, 72, 1.0, 0.3)),
        ("chapa r=1 h=0.1", slab),
    ];
    for (name, mut m) in cases {
        m.triangulate();
        let (mut rays, mut proxies) = (Vec::new(), Vec::new());
        for v in 0..m.vert_count() {
            let p = m.positions()[v];
            let n = m.normals()[v];
            let eps = 1e-3;
            let o = [p[0] - n[0] * eps, p[1] - n[1] * eps, p[2] - n[2] * eps];
            let Some(h) = m.raycast(&Ray::new(o, [-n[0], -n[1], -n[2]])) else {
                continue;
            };
            let k = m.curv_world()[v].abs();
            if k <= 1e-6 {
                proxies.push(f32::INFINITY);
            } else {
                proxies.push(2.0 / k);
            }
            rays.push(h.t + eps);
        }
        if rays.is_empty() {
            println!("{name:>22} | (nenhum raio saiu)");
            continue;
        }
        let pick = |v: &mut Vec<f32>, q: f32| {
            v.sort_by(|a, b| a.partial_cmp(b).unwrap_or(core::cmp::Ordering::Equal));
            v[((v.len() - 1) as f32 * q) as usize]
        };
        let err: Vec<f32> = rays
            .iter()
            .zip(&proxies)
            .map(|(&r, &p)| if p.is_finite() { (p - r).abs() / r } else { 99.0 })
            .collect();
        let mut e = err.clone();
        let (mut a, mut b) = (rays.clone(), proxies.clone());
        println!(
            "{name:>22} | {:>9.3} {:>9.3} | {:>9.3} {:>9.3} | {:>7.0}%",
            pick(&mut a, 0.5),
            pick(&mut a, 0.9),
            pick(&mut b, 0.5),
            pick(&mut b, 0.9),
            100.0 * pick(&mut e, 0.5),
        );
    }
    println!(
        "\n  A CHAPA e' o caso da cera (folha, orelha, mao na lanterna): `kappa`\n  \
         de uma face plana e' ZERO, entao o proxy diz espessura INFINITA onde a\n  \
         peca e' mais fina. E' exatamente onde o canal existe para acender."
    );
}

/// **O QUE O BAKE CUSTA** — o número que decide se ele precisa de `rayon` (e,
/// com ele, de um ADR pela cerca do ADR-0109).
#[test]
#[ignore = "sonda de medição; roda sob demanda"]
fn measure_what_the_thickness_bake_costs() {
    println!("\n=== CUSTO DO BAKE DE ESPESSURA (serial) ===");
    println!(
        "{:>10} | {:>12} | {:>12} | {:>12} | {:>11}",
        "vertices", "raio", "campo", "AO (cones)", "raio/campo"
    );
    for &segs in &[24usize, 72, 144] {
        let mut m = shapes::uv_sphere(segs * 2 / 3, segs, 1.0);
        m.triangulate();

        let mut field = ph2d_sdf::VoxelField::for_bounds(m.bounds(), 150);
        field.voxelize(&m);
        field.flood_fill();

        let t0 = std::time::Instant::now();
        let th: Vec<f32> = (0..m.vert_count())
            .map(|v| {
                let p = m.positions()[v];
                let n = m.normals()[v];
                let eps = 1e-3;
                let o = [p[0] - n[0] * eps, p[1] - n[1] * eps, p[2] - n[2] * eps];
                m.raycast(&Ray::new(o, [-n[0], -n[1], -n[2]]))
                    .map_or(f32::INFINITY, |h| h.t + eps)
            })
            .collect();
        let ms_th = t0.elapsed().as_secs_f64() * 1000.0;

        let t2 = std::time::Instant::now();
        let by_field = ph2d_sdf::bake_thickness(&field, &m);
        let ms_fld = t2.elapsed().as_secs_f64() * 1000.0;
        assert_eq!(by_field.len(), th.len());

        let t1 = std::time::Instant::now();
        let ao = ph2d_sdf::bake_ao(
            &field,
            m.positions(),
            m.normals(),
            ph2d_sdf::AoParams::for_bounds(m.bounds()),
        );
        let ms_ao = t1.elapsed().as_secs_f64() * 1000.0;

        assert_eq!(th.len(), ao.len());
        println!(
            "{:>10} | {:>9.2} ms | {:>9.2} ms | {:>9.2} ms | {:>10.1}x",
            th.len(),
            ms_th,
            ms_fld,
            ms_ao,
            ms_th / ms_fld.max(1e-9)
        );
    }
    println!(
        "\n  O que decide NAO e' a precisao (o raio e' 16x mais exato, e os 0,33%\n  \
         do campo entram num exp(), onde sao invisiveis): e' o CRESCIMENTO.\n  \
         4x os vertices multiplicam o raio por ~14 e o campo por ~3,6, entao\n  \
         numa escultura de verdade a diferenca deixa de ser 15x e vira\n  \
         SEGUNDOS contra MINUTOS.\n\n  \
         O AO (paralelo, ADR-0156) ainda custa ~15x MENOS que o campo -- se um\n  \
         dia estes 36 ms incomodarem, e' esse o numero que justifica o ADR."
    );
}
