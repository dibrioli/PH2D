//! ⭐⭐⭐ **O INSTRUMENTO DA PEÇA ENTREGUE** — mede um `.obj` que SAIU do botão, e diz
//! **onde** os defeitos estão, não só quantos são.
//!
//! ```text
//! cargo run --release -p ph2d-quadextract --example piece_report -- <ficheiro.obj>
//! ```
//!
//! ⛔⛔ **Ele existe porque «quantos» não responde ao report do artista.** *«Furos nas
//! pontas»* é uma afirmação sobre **posição**, e toda régua desta linha era um total: `14`
//! arestas de bordo não diz se elas estão nas pontas ou espalhadas. ⇒ cada censo aqui sai
//! **com o raio normalizado** (`1,0` = o raio mediano da peça), que é a coordenada em que
//! «ponta» quer dizer alguma coisa.

use std::collections::BTreeMap;

fn load(name: &str) -> ph2d_mesh::Mesh {
    let text = std::fs::read_to_string(name).unwrap_or_else(|e| panic!("{name}: {e}"));
    ph2d_mesh::import_obj(&text)
        .unwrap_or_else(|e| panic!("{name} nao e' um OBJ deste leitor: {e:?}"))
        .into_iter()
        .next()
        .unwrap_or_else(|| panic!("{name} nao tem peca dentro"))
        .mesh
}

fn p50(v: &mut [f32]) -> f32 {
    v.sort_by(f32::total_cmp);
    v.get(v.len() / 2).copied().unwrap_or(0.0)
}

fn main() {
    for name in std::env::args().skip(1) {
        let mesh = load(&name);
        let pos = mesh.positions().to_vec();
        let n = pos.len();

        // O centro e a régua de «ponta»: o raio MEDIANO.
        let c = pos.iter().fold([0.0f64; 3], |a, p| {
            [
                a[0] + f64::from(p[0]),
                a[1] + f64::from(p[1]),
                a[2] + f64::from(p[2]),
            ]
        });
        #[allow(clippy::cast_possible_truncation)]
        let c = [
            (c[0] / n as f64) as f32,
            (c[1] / n as f64) as f32,
            (c[2] / n as f64) as f32,
        ];
        let radius: Vec<f32> = pos
            .iter()
            .map(|p| {
                let d = [p[0] - c[0], p[1] - c[1], p[2] - c[2]];
                d[0].mul_add(d[0], d[1].mul_add(d[1], d[2] * d[2])).sqrt()
            })
            .collect();
        let rmed = p50(&mut radius.clone()).max(1.0e-9);

        // Censo de arestas + valência.
        let mut use_count: BTreeMap<(u32, u32), usize> = BTreeMap::new();
        let mut valence = vec![0usize; n];
        let mut sides: BTreeMap<usize, usize> = BTreeMap::new();
        for f in mesh.faces() {
            let v = f.verts();
            *sides.entry(v.len()).or_default() += 1;
            for k in 0..v.len() {
                let (a, b) = (v[k], v[(k + 1) % v.len()]);
                *use_count
                    .entry(if a < b { (a, b) } else { (b, a) })
                    .or_default() += 1;
                valence[a as usize] += 1;
            }
        }
        let e = use_count.len();
        let fcount = mesh.face_count();
        #[allow(clippy::cast_possible_wrap)]
        let chi = n as i64 - e as i64 + fcount as i64;

        let mut bordo_r: Vec<f32> = Vec::new();
        let mut nm_r: Vec<f32> = Vec::new();
        for (&(a, b), &k) in &use_count {
            let r = 0.5 * (radius[a as usize] + radius[b as usize]) / rmed;
            if k == 1 {
                bordo_r.push(r);
            } else if k >= 3 {
                nm_r.push(r);
            }
        }

        // Irregulares, e onde eles moram.
        let mut irr_r: Vec<f32> = Vec::new();
        let mut irr_by: BTreeMap<usize, usize> = BTreeMap::new();
        for v in 0..n {
            if valence[v] != 4 && valence[v] > 0 {
                *irr_by.entry(valence[v]).or_default() += 1;
                irr_r.push(radius[v] / rmed);
            }
        }

        // ⚠️ **Onde as faces PESSIMAS moram** — a mesma pergunta de posição, para a outra
        // metade do report do artista (*«baixa qualidade da superficie de modo geral»*).
        let mut ruim_r: Vec<f32> = Vec::new();
        for f in mesh.faces() {
            let v = f.verts();
            if v.len() != 4 {
                continue;
            }
            let mut pior = 0.0f32;
            for k in 0..4 {
                let (a, b, c2) = (
                    pos[v[(k + 3) % 4] as usize],
                    pos[v[k] as usize],
                    pos[v[(k + 1) % 4] as usize],
                );
                let e1 = [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
                let e2 = [c2[0] - b[0], c2[1] - b[1], c2[2] - b[2]];
                let l1 = e1[0]
                    .mul_add(e1[0], e1[1].mul_add(e1[1], e1[2] * e1[2]))
                    .sqrt();
                let l2 = e2[0]
                    .mul_add(e2[0], e2[1].mul_add(e2[1], e2[2] * e2[2]))
                    .sqrt();
                if l1 <= 1.0e-12 || l2 <= 1.0e-12 {
                    continue;
                }
                let cs = e1[0].mul_add(e2[0], e1[1].mul_add(e2[1], e1[2] * e2[2])) / (l1 * l2);
                let ang = cs.clamp(-1.0, 1.0).acos().to_degrees();
                pior = pior.max((ang - 90.0).abs());
            }
            if pior > 60.0 {
                let r = v.iter().map(|&i| radius[i as usize]).sum::<f32>() / 4.0 / rmed;
                ruim_r.push(r);
            }
        }

        // ⭐⭐⭐ **A RUGOSIDADE — a régua que esta linha NUNCA teve.**
        //
        // ⛔⛔ O artista reportou duas vezes *«irregularidades da superficie que deveria ser
        // lisa»*, e **nenhuma régua desta linha mede isso**: aspecto e enviesamento falam da
        // forma de cada quad DENTRO do plano dele, e uma grade de quads perfeitamente
        // quadrados pode ondular como um telhado de zinco. ⚠️ *Um quad quadrado e um quad no
        // sítio certo são duas propriedades diferentes, e só uma delas estava a ser medida.*
        //
        // A grandeza é o **ângulo diedro** entre faces vizinhas. Numa peça lisa ele é o que a
        // curvatura obriga (pequeno e a variar devagar); onde a malha ondula, ele **alterna
        // de sinal de face para face**, e é isso que o olho lê como superfície ruim.
        let mut owner: BTreeMap<(u32, u32), Vec<usize>> = BTreeMap::new();
        for (fi, f) in mesh.faces().iter().enumerate() {
            let v = f.verts();
            for k in 0..v.len() {
                let (a, b) = (v[k], v[(k + 1) % v.len()]);
                owner
                    .entry(if a < b { (a, b) } else { (b, a) })
                    .or_default()
                    .push(fi);
            }
        }
        let fnormals = mesh.face_normals().to_vec();
        let mut dihedral: Vec<f32> = Vec::new();
        let mut ruga_r: Vec<f32> = Vec::new();
        for (&(a, b), who) in &owner {
            let [f, g] = who[..] else { continue };
            let (u, w) = (fnormals[f], fnormals[g]);
            let cs = u[0]
                .mul_add(w[0], u[1].mul_add(w[1], u[2] * w[2]))
                .clamp(-1.0, 1.0);
            let ang = cs.acos().to_degrees();
            dihedral.push(ang);
            if ang > 30.0 {
                ruga_r.push(0.5 * (radius[a as usize] + radius[b as usize]) / rmed);
            }
        }

        let shape = ph2d_quadfill::quad_shape(&mesh);
        println!("{name}");
        println!(
            "  {n} verts · {fcount} faces {sides:?} · {e} arestas · X = {chi}  \
             (uma casca fechada da' X = 2)"
        );
        let banda = |v: &[f32]| {
            let mut s = v.to_vec();
            if s.is_empty() {
                return String::from("(nenhum)");
            }
            s.sort_by(f32::total_cmp);
            #[allow(clippy::cast_precision_loss)]
            let acima = s.iter().filter(|r| **r > 1.15).count() as f64 * 100.0 / s.len() as f64;
            format!(
                "raio p50 {:.2}x · p90 {:.2}x · {acima:.0}% acima de 1,15x",
                s[s.len() / 2],
                s[s.len() * 9 / 10]
            )
        };
        let mut rr = radius.clone();
        rr.sort_by(f32::total_cmp);
        println!(
            "  RAIO: p50 {:.3} (a regua) · p90 {:.2}x · p99 {:.2}x · MAX {:.2}x  \
             <- e' isto que «ponta» quer dizer nesta peca",
            rmed,
            rr[rr.len() * 9 / 10] / rmed,
            rr[rr.len() * 99 / 100] / rmed,
            rr[rr.len() - 1] / rmed
        );
        println!(
            "  ⛔ BORDO (os furos): {} arestas · {}",
            bordo_r.len(),
            banda(&bordo_r)
        );
        println!(
            "  ⛔ NAO-MANIFOLD: {} arestas · {}",
            nm_r.len(),
            banda(&nm_r)
        );
        #[allow(clippy::cast_precision_loss)]
        let irr_pct = 100.0 * irr_r.len() as f64 / n as f64;
        println!(
            "  IRREGULARES: {} de {n} ({irr_pct:.2}%) {irr_by:?} · {}",
            irr_r.len(),
            banda(&irr_r)
        );
        println!(
            "  ⭐⭐ FORMA: aspecto p50 {:.2} p99 {:.2} (>4x: {}) | ENVIESAMENTO p50 {:.1}° \
             p99 {:.1}° (>60: {})",
            shape.aspect_p50,
            shape.aspect_p99,
            shape.aspect_over_4,
            shape.skew_p50,
            shape.skew_p99,
            shape.skew_over_60
        );
        println!(
            "  ⛔ FACES PESSIMAS (>60°): {} · {}",
            ruim_r.len(),
            banda(&ruim_r)
        );
        dihedral.sort_by(f32::total_cmp);
        let d = |q: usize| {
            dihedral
                .get(dihedral.len() * q / 100)
                .copied()
                .unwrap_or(0.0)
        };
        #[allow(clippy::cast_precision_loss)]
        let ruga_pct = 100.0 * ruga_r.len() as f64 / dihedral.len().max(1) as f64;
        println!(
            "  ⭐⭐ RUGOSIDADE (diedro entre vizinhas): p50 {:.1}° p90 {:.1}° p99 {:.1}° \
             MAX {:.1}°",
            d(50),
            d(90),
            d(99),
            dihedral.last().copied().unwrap_or(0.0)
        );
        println!(
            "     dobras acima de 30°: {} ({ruga_pct:.2}%) · {}",
            ruga_r.len(),
            banda(&ruga_r)
        );
        println!(
            "     (a barra do oraculo: enviesamento p50 4,8-7,1 | aspecto p50 1,08-1,22 | >60: 0-4)"
        );
    }
}
