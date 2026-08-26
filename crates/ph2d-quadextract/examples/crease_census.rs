//! ⭐⭐⭐ **OBRA A do [`ACHADO_ordem_das_fases.md`](../../../docs/3D/quad-remesh/ACHADO_ordem_das_fases.md):
//! quanto do VINCO é que o passo zero destrói?**
//!
//! ```text
//! cargo run --release -p ph2d-quadextract --example crease_census -- <peça|ficheiro.obj>
//! ```
//!
//! ⛔ **Ela decide se as outras cinco obras daquela tabela valem alguma coisa.** A hipótese
//! é que a remalha isotrópica — que não sabe que vinco existe — alisa as quinas antes de
//! qualquer fase as poder usar. *Se ela não as alisar, a reordenação inteira é palpite.*
//!
//! A régua é o **ângulo diedro** entre faces vizinhas, que é como o alvo define um vinco
//! (`quadwild-2021` §4.1: limiar sobre o diedro). Aqui não se escolhe limiar nenhum: sai a
//! **distribuição**, e é ela que diz se a cauda aguda sobrevive.

use std::collections::BTreeMap;

fn load(name: &str) -> ph2d_mesh::Mesh {
    let mut mesh = if name.ends_with(".obj") {
        let text = std::fs::read_to_string(name).unwrap_or_else(|e| panic!("{name}: {e}"));
        ph2d_mesh::import_obj(&text)
            .unwrap_or_else(|e| panic!("{name} nao e' um OBJ deste leitor: {e:?}"))
            .into_iter()
            .next()
            .unwrap_or_else(|| panic!("{name} nao tem peca dentro"))
            .mesh
    } else {
        match name {
            "cilindro" => ph2d_mesh::shapes::cylinder(64, 0.5, 1.5),
            "toro" => ph2d_mesh::shapes::torus(64, 32, 1.0, 0.35),
            "octaedro" => ph2d_mesh::shapes::octahedron(1.0),
            _ => ph2d_mesh::shapes::uv_sphere(24, 36, 1.0),
        }
    };
    mesh.triangulate();
    mesh
}

/// Os ângulos diedros de toda aresta interior, em graus.
fn dihedrals(mesh: &ph2d_mesh::Mesh) -> Vec<f32> {
    let n = mesh.face_normals();
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
    let mut out = Vec::new();
    for who in owner.values() {
        let [f, g] = who[..] else { continue };
        let (u, w) = (n[f], n[g]);
        let c = u[0]
            .mul_add(w[0], u[1].mul_add(w[1], u[2] * w[2]))
            .clamp(-1.0, 1.0);
        out.push(c.acos().to_degrees());
    }
    out
}

/// ⚠️ **A cauda AGUDA é o que interessa, e ela tem de ser medida em FRACÇÃO** — a remalha
/// muda a contagem de arestas, então comparar contagens absolutas compararia duas malhas de
/// tamanhos diferentes. *Uma cauda que encolhe porque a malha encolheu não é uma cauda que
/// se perdeu.*
/// ⭐⭐⭐ **A TOPOLOGIA da malha: `χ`, bordo e NÃO-MANIFOLD.**
///
/// ⛔ **Uma aresta não-manifold parte o mapa de meias-arestas** (`(a,b) -> face`, uma face
/// por aresta dirigida): com três faces a reclamar a mesma, o mapa guarda uma e as outras
/// desaparecem. ⚠️ *É esse mapa que a travessia de fronteira do layout usa para pivotar* —
/// e é por isso que esta coluna pertence ao mesmo censo que os vincos.
fn topology(rotulo: &str, mesh: &ph2d_mesh::Mesh) {
    let mut n: BTreeMap<(u32, u32), usize> = BTreeMap::new();
    for f in mesh.faces() {
        let v = f.verts();
        for k in 0..v.len() {
            let (a, b) = (v[k], v[(k + 1) % v.len()]);
            *n.entry(if a < b { (a, b) } else { (b, a) }).or_default() += 1;
        }
    }
    let bordo = n.values().filter(|c| **c == 1).count();
    let nm = n.values().filter(|c| **c >= 3).count();
    #[allow(clippy::cast_possible_wrap)]
    let chi = mesh.vert_count() as i64 - n.len() as i64 + mesh.face_count() as i64;
    println!(
        "  {rotulo:<10} topologia: χ = {chi} (fechada da' 2) · {bordo} de bordo · \
         ⛔ {nm} NAO-MANIFOLD"
    );
}

fn report(rotulo: &str, mesh: &ph2d_mesh::Mesh) {
    let mut d = dihedrals(mesh);
    d.sort_by(f32::total_cmp);
    let n = d.len().max(1);
    let q = |p: usize| d.get(d.len() * p / 100).copied().unwrap_or(0.0);
    let frac = |lim: f32| {
        #[allow(clippy::cast_precision_loss)]
        {
            100.0 * d.iter().filter(|a| **a >= lim).count() as f64 / n as f64
        }
    };
    println!(
        "  {rotulo:<10} {:>6} arestas · diedro p50 {:>5.1}° p90 {:>5.1}° p99 {:>5.1}° \
         MAX {:>5.1}° | ⭐ acima de 30°: {:>5.2}% · 45°: {:>5.2}% · 60°: {:>5.2}%",
        d.len(),
        q(50),
        q(90),
        q(99),
        d.last().copied().unwrap_or(0.0),
        frac(30.0),
        frac(45.0),
        frac(60.0)
    );
}

/// ⭐⭐ **A RESOLUÇÃO NA PONTA contra a do corpo** — a outra metade da pergunta.
///
/// ⚠️ Preservar o vinco e **amostrá-lo o suficiente** são coisas diferentes: o alvo faz as
/// duas (`quadwild-2021` §4.2 afina o alvo local entre `0,3×` e `3×` perto dos vincos), e
/// só a segunda é sobre a ponta. *Uma quina que sobrevive com três triângulos em cima dela
/// sobreviveu ao censo e não ao produto.*
fn resolution(rotulo: &str, mesh: &ph2d_mesh::Mesh) {
    let pos = mesh.positions();
    let c = pos.iter().fold([0.0f64; 3], |a, p| {
        [
            a[0] + f64::from(p[0]),
            a[1] + f64::from(p[1]),
            a[2] + f64::from(p[2]),
        ]
    });
    #[allow(clippy::cast_precision_loss)]
    let inv = 1.0 / pos.len().max(1) as f64;
    let c = [c[0] * inv, c[1] * inv, c[2] * inv];
    let r = |p: [f32; 3]| {
        let d = [
            f64::from(p[0]) - c[0],
            f64::from(p[1]) - c[1],
            f64::from(p[2]) - c[2],
        ];
        d[0].mul_add(d[0], d[1].mul_add(d[1], d[2] * d[2])).sqrt()
    };
    let mut all: Vec<f64> = pos.iter().map(|p| r(*p)).collect();
    all.sort_by(f64::total_cmp);
    let med = all[all.len() / 2].max(1.0e-12);

    let (mut e_body, mut n_body, mut e_tip, mut n_tip) = (0.0f64, 0usize, 0.0f64, 0usize);
    for f in mesh.faces() {
        let v = f.verts();
        for k in 0..v.len() {
            let (a, b) = (pos[v[k] as usize], pos[v[(k + 1) % v.len()] as usize]);
            let d = [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
            let l = f64::from(d[0].mul_add(d[0], d[1].mul_add(d[1], d[2] * d[2])).sqrt());
            let rr = 0.5 * (r(a) + r(b)) / med;
            if rr > 1.15 {
                e_tip += l;
                n_tip += 1;
            } else {
                e_body += l;
                n_body += 1;
            }
        }
    }
    #[allow(clippy::cast_precision_loss)]
    let (mb, mt) = (e_body / n_body.max(1) as f64, e_tip / n_tip.max(1) as f64);
    println!(
        "  {rotulo:<10} aresta media: corpo {mb:.4} · ⭐ PONTA {mt:.4} ({:.2}x o corpo)          | {} lados na ponta de {}",
        mt / mb.max(1.0e-12),
        n_tip,
        n_body + n_tip
    );
}

fn main() {
    for name in std::env::args().skip(1) {
        let mesh = load(&name);
        println!("{name}");
        report("ANTES", &mesh);
        resolution("ANTES", &mesh);
        topology("ANTES", &mesh);
        let mut work = mesh.clone();
        ph2d_remesh_iso::remesh_isotropic(&mut work, ph2d_remesh_iso::ALPHA);
        work.triangulate();
        report("DEPOIS", &work);
        resolution("DEPOIS", &work);
        topology("DEPOIS", &work);
    }
}
