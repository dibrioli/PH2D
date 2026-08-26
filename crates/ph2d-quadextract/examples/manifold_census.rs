//! ⭐⭐⭐ **O RETRATO DA ARESTA AMBÍGUA** — a sonda que faltava antes de escrever a cura.
//!
//! ```text
//! cargo run --release -p ph2d-quadextract --example manifold_census -- <peça|ficheiro.obj>
//! ```
//!
//! ⛔⛔ **Por que ela existe:** em 2026-08-25 esta linha construiu **quatro** reparações
//! não-manifold e as quatro saíram piores que o defeito
//! (`docs/3D/quad-remesh/ACHADO_ordem_das_fases.md` §12). Todas as quatro foram desenhadas a
//! partir do **nome** do defeito — *«uma aleta»*, *«duas folhas»* — e nenhuma a partir da
//! estrutura medida. ⚠️ *Duas vezes no mesmo dia a cura foi inferida do nome e refutada pela
//! medição.* Esta sonda inverte a ordem.
//!
//! ⭐ O que ela responde, e cada resposta escolhe uma cura DIFERENTE:
//!
//! | o que a aresta é | como se lê aqui | a cura que isso escolhe |
//! |---|---|---|
//! | uma **aleta** | 3 faces, uma com diedro ~0 com as outras | deitar a face fora |
//! | duas **folhas coladas** | 4 faces, dois pares | **soldar** (as folhas são a mesma) |
//! | uma face **repetida** | 2 faces com o MESMO conjunto de vértices | deduplicar |
//! | um **beliscão** | 3+ faces e vértices coincidentes à volta | soldar por posição |
//!
//! ⚠️ **A coluna `cópias` é a lei da partição aplicada localmente** — quantas cópias o
//! [`ph2d_mesh::split_non_manifold`] faria naquele vértice. É o que liga esta sonda à
//! reparação que já existe.

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
            _ => ph2d_mesh::shapes::uv_sphere(24, 36, 1.0),
        }
    };
    mesh.triangulate();
    mesh
}

fn edge_faces(mesh: &ph2d_mesh::Mesh) -> BTreeMap<(u32, u32), Vec<u32>> {
    let mut out: BTreeMap<(u32, u32), Vec<u32>> = BTreeMap::new();
    for (fi, f) in mesh.faces().iter().enumerate() {
        let v = f.verts();
        for k in 0..v.len() {
            let (a, b) = (v[k], v[(k + 1) % v.len()]);
            #[allow(clippy::cast_possible_truncation)]
            out.entry(if a < b { (a, b) } else { (b, a) })
                .or_default()
                .push(fi as u32);
        }
    }
    out
}

fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1].mul_add(b[2], -(a[2] * b[1])),
        a[2].mul_add(b[0], -(a[0] * b[2])),
        a[0].mul_add(b[1], -(a[1] * b[0])),
    ]
}
fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0].mul_add(b[0], a[1].mul_add(b[1], a[2] * b[2]))
}
fn norm(a: [f32; 3]) -> f32 {
    dot(a, a).sqrt()
}

/// ⭐ **A direcção da face vista DA aresta** — o que separa duas faces que partilham `(a,b)`.
///
/// ⚠️ Não é a normal: duas folhas coladas podem ter normais opostas e estar **no mesmo
/// sítio**. O que as separa é para onde o terceiro vértice aponta, medido no plano
/// perpendicular à aresta. *É este vector que dá o ângulo em torno da aresta.*
fn spoke(pos: &[[f32; 3]], a: u32, b: u32, f: &ph2d_mesh::Face) -> [f32; 3] {
    let e = sub(pos[b as usize], pos[a as usize]);
    let el = norm(e).max(1e-20);
    let e = [e[0] / el, e[1] / el, e[2] / el];
    let c = f
        .verts()
        .iter()
        .copied()
        .find(|&v| v != a && v != b)
        .unwrap_or(a);
    let d = sub(pos[c as usize], pos[a as usize]);
    let t = dot(d, e);
    let r = [
        d[0] - t * e[0],
        d[1] - t * e[1],
        d[2] - t * e[2],
    ];
    let rl = norm(r).max(1e-20);
    [r[0] / rl, r[1] / rl, r[2] / rl]
}

/// Quantas componentes o anel de `v` tem quando **só as arestas de duas faces ligam** —
/// a lei da partição, medida num vértice só.
fn ring_components(mesh: &ph2d_mesh::Mesh, ef: &BTreeMap<(u32, u32), Vec<u32>>, v: u32) -> usize {
    let faces: Vec<u32> = mesh
        .faces()
        .iter()
        .enumerate()
        .filter(|(_, f)| f.verts().contains(&v))
        .filter_map(|(i, _)| u32::try_from(i).ok())
        .collect();
    let local: BTreeMap<u32, usize> = faces.iter().enumerate().map(|(i, &f)| (f, i)).collect();
    let mut parent: Vec<usize> = (0..faces.len()).collect();
    fn root(p: &mut [usize], mut a: usize) -> usize {
        while p[a] != a {
            p[a] = p[p[a]];
            a = p[a];
        }
        a
    }
    for (&(x, y), who) in ef {
        if who.len() != 2 || (x != v && y != v) {
            continue;
        }
        if let (Some(&i), Some(&j)) = (local.get(&who[0]), local.get(&who[1])) {
            let (ri, rj) = (root(&mut parent, i), root(&mut parent, j));
            if ri != rj {
                parent[rj] = ri;
            }
        }
    }
    let mut seen = std::collections::BTreeSet::new();
    for i in 0..faces.len() {
        let r = root(&mut parent, i);
        seen.insert(r);
    }
    seen.len()
}

fn main() {
    let mut args = std::env::args().skip(1);
    let name = args.next().unwrap_or_else(|| String::from("esfera"));
    // ⭐⭐⭐ **`f1` mede a malha DEPOIS do remalhe** — a pergunta *«quem cria o defeito, o
    // ficheiro ou nós?»*. ⚠️ Sem este modo a sonda só sabe acusar o ficheiro.
    let after_f1 = args.next().is_some_and(|a| a == "f1");
    let mut mesh = load(&name);
    if after_f1 {
        // ⛔ Com a cura DESLIGADA, senão a sonda mede a cura em vez do remalhe.
        // SAFETY-ish: é um exemplo, e a variável só governa este processo.
        unsafe { std::env::set_var("PH2D_DOUBLED_REPAIR", "0") };
        ph2d_remesh_iso::remesh_isotropic(&mut mesh, ph2d_remesh_iso::ALPHA);
        mesh.triangulate();
        println!("[depois do F1]");
    }
    let mesh = mesh;
    let pos = mesh.positions();
    let ef = edge_faces(&mesh);

    // O centro e o raio médio — para dizer ONDE o defeito mora, em unidades da peça.
    let c = pos.iter().fold([0.0f32; 3], |a, p| {
        [a[0] + p[0], a[1] + p[1], a[2] + p[2]]
    });
    let n = pos.len().max(1) as f32;
    let c = [c[0] / n, c[1] / n, c[2] / n];
    let rmean = pos.iter().map(|p| norm(sub(*p, c))).sum::<f32>() / n;

    let bad: Vec<(&(u32, u32), &Vec<u32>)> = ef.iter().filter(|(_, w)| w.len() >= 3).collect();
    let border = ef.values().filter(|w| w.len() == 1).count();
    println!(
        "{name}: {} vertices, {} faces · {} arestas · bordo {border} · NAO-MANIFOLD {} \
         · raio medio {rmean:.4}",
        pos.len(),
        mesh.face_count(),
        ef.len(),
        bad.len()
    );

    // ⭐ **Vértices coincidentes** — a pergunta que separa «beliscão» de «aleta».
    let mut by_pos: BTreeMap<[i64; 3], Vec<u32>> = BTreeMap::new();
    for (i, p) in pos.iter().enumerate() {
        let q = [
            (f64::from(p[0]) * 1e6).round() as i64,
            (f64::from(p[1]) * 1e6).round() as i64,
            (f64::from(p[2]) * 1e6).round() as i64,
        ];
        #[allow(clippy::cast_possible_truncation)]
        by_pos.entry(q).or_default().push(i as u32);
    }
    // ⭐⭐⭐ **O BORDO MEDE-SE PELO COMPRIMENTO E PELOS LAÇOS, nunca pela contagem de
    // arestas.** ⚠️ *Que número imprimiria a resposta contrária?* Um remalhe que reamostra
    // a mesma curva a um passo mais fino sobe a contagem sem tocar no buraco — `38 → 107`
    // lê-se como «alargou» e pode ser «cortou mais fino». O que não pode mudar é **quantos
    // buracos há** e **que perímetro têm**.
    {
        let mut nxt: BTreeMap<u32, Vec<u32>> = BTreeMap::new();
        let mut length = 0.0f32;
        for (&(x, y), who) in &ef {
            if who.len() != 1 {
                continue;
            }
            length += norm(sub(pos[x as usize], pos[y as usize]));
            nxt.entry(x).or_default().push(y);
            nxt.entry(y).or_default().push(x);
        }
        let mut seen: std::collections::BTreeSet<u32> = std::collections::BTreeSet::new();
        let mut loops = 0usize;
        for &v in nxt.keys() {
            if !seen.insert(v) {
                continue;
            }
            loops += 1;
            let mut stack = vec![v];
            while let Some(u) = stack.pop() {
                for &w in nxt.get(&u).into_iter().flatten() {
                    if seen.insert(w) {
                        stack.push(w);
                    }
                }
            }
        }
        println!("  BORDO: {loops} lacos, perimetro {length:.4} (em raios medios: {:.4})", length / rmean);
    }

    let dup_groups = by_pos.values().filter(|g| g.len() > 1).count();
    let dup_verts: usize = by_pos.values().filter(|g| g.len() > 1).map(Vec::len).sum();
    println!("  vertices COINCIDENTES: {dup_groups} grupos, {dup_verts} vertices");

    // ⭐ Faces repetidas — o mesmo CONJUNTO de vértices duas vezes.
    let mut by_set: BTreeMap<[u32; 3], usize> = BTreeMap::new();
    for f in mesh.faces() {
        let mut v = [f.verts()[0], f.verts()[1], f.verts()[2]];
        v.sort_unstable();
        *by_set.entry(v).or_default() += 1;
    }
    // ⭐⭐⭐ **A ORIENTAÇÃO das cópias decide a cura**, e as duas leituras são a mesma
    // contagem: cópias com a MESMA orientação são lixo (a segunda não acrescenta
    // superfície nenhuma); com orientação OPOSTA são uma folha de espessura zero, e apagar
    // uma delas **abre** a peça. ⚠️ *Sem esta coluna, deduplicar é um palpite.*
    let mut by_cycle: BTreeMap<[u32; 3], usize> = BTreeMap::new();
    for f in mesh.faces() {
        let v = f.verts();
        let mut c = [v[0], v[1], v[2]];
        // A rotação canónica do ciclo — preserva o sentido.
        while c[0] != *c.iter().min().unwrap_or(&c[0]) {
            c = [c[1], c[2], c[0]];
        }
        *by_cycle.entry(c).or_default() += 1;
    }
    let same_winding: usize = by_cycle.values().filter(|&&c| c > 1).map(|c| c - 1).sum();
    let repeated_sets = by_set.values().filter(|&&c| c > 1).count();
    let extra_sets: usize = by_set.values().filter(|&&c| c > 1).map(|c| c - 1).sum();
    println!(
        "  faces REPETIDAS: {repeated_sets} conjuntos ({extra_sets} copias a mais) · \
         destas, {same_winding} com a MESMA orientacao, {} com orientacao OPOSTA",
        extra_sets - same_winding
    );

    // ⭐⭐⭐ **A CURA, medida no mesmo sítio que o retrato.** Uma sonda que descreve o
    // defeito e não mede o remédio deixa as duas metades em ficheiros diferentes.
    {
        let mut cured = mesh.clone();
        let r = ph2d_mesh::drop_doubled_faces(&mut cured);
        println!(
            "  CURA (folhas de espessura zero): {} pares espelhados, {} repeticoes puras · \
             ambiguas {} -> {} · bordo {} -> {}{}",
            r.mirror_pairs,
            r.same_winding_dropped,
            r.bad_edges_before,
            r.bad_edges_after,
            r.border_before,
            r.border_after,
            if r.refused { "  ⛔ RECUSADA" } else { "" }
        );
    }

    if bad.is_empty() {
        println!("  ⭐ nenhuma aresta ambigua.");
        return;
    }

    println!(
        "\n{:>4} {:>7} {:>7} | {:>6} | {:>6} {:>6} | {:>28} | raio/medio",
        "#", "vA", "vB", "faces", "copA", "copB", "angulos em torno (graus)"
    );
    for (i, entry) in bad.iter().enumerate() {
        let (&(a, b), who) = *entry;
        let ra = norm(sub(pos[a as usize], c)) / rmean;
        let rb = norm(sub(pos[b as usize], c)) / rmean;
        let faces: Vec<&ph2d_mesh::Face> = who
            .iter()
            .map(|&f| &mesh.faces()[f as usize])
            .collect();
        // Os ângulos em torno da aresta, ordenados — é o retrato do leque.
        let e = sub(pos[b as usize], pos[a as usize]);
        let el = norm(e).max(1e-20);
        let axis = [e[0] / el, e[1] / el, e[2] / el];
        let s0 = spoke(pos, a, b, faces[0]);
        let mut ang: Vec<f32> = faces
            .iter()
            .map(|f| {
                let s = spoke(pos, a, b, f);
                let x = dot(s, s0).clamp(-1.0, 1.0);
                let y = dot(cross(s0, s), axis);
                let t = y.atan2(x).to_degrees();
                if t < 0.0 { t + 360.0 } else { t }
            })
            .collect();
        ang.sort_by(|x, y| x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal));
        let txt = ang
            .iter()
            .map(|t| format!("{t:.1}"))
            .collect::<Vec<_>>()
            .join(" ");
        println!(
            "{:>4} {a:>7} {b:>7} | {:>6} | {:>6} {:>6} | {txt:>28} | {ra:.3}/{rb:.3}",
            i,
            who.len(),
            ring_components(&mesh, &ef, a),
            ring_components(&mesh, &ef, b),
        );
        // ⚠️ As faces cruas — para se ver se duas partilham o TERCEIRO vértice, que é o
        // que distingue duas folhas coladas de um leque a sério.
        let thirds: Vec<u32> = faces
            .iter()
            .map(|f| {
                f.verts()
                    .iter()
                    .copied()
                    .find(|&v| v != a && v != b)
                    .unwrap_or(a)
            })
            .collect();
        let areas: Vec<f32> = faces
            .iter()
            .map(|f| {
                let v = f.verts();
                let (p0, p1, p2) = (pos[v[0] as usize], pos[v[1] as usize], pos[v[2] as usize]);
                norm(cross(sub(p1, p0), sub(p2, p0))) * 0.5
            })
            .collect();
        println!(
            "       terceiros {thirds:?} · areas {}",
            areas
                .iter()
                .map(|x| format!("{x:.3e}"))
                .collect::<Vec<_>>()
                .join(" ")
        );
    }
}
