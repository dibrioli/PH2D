//! ⭐⭐ **AS RÉGUAS DA FOTO** — e a fixtura em que elas se calibram.
//!
//! Irmã de [`super::photo_probes`] por RESPONSABILIDADE: aquele módulo pergunta *«o que
//! o artista viu?»*, e este responde *«com que régua se mede isso?»*.
//!
//! ⛔⛔ **Cada uma destas existe porque uma que já havia era CEGA ao defeito da vez:**
//! [`islands`] conta COMPONENTES LIGADOS, e é a única que vê uma almofada (o `χ` conta
//! os dois lados e dá `2`, o bordo é zero, o não-manifold é zero, e a contagem de quads
//! até sobe); [`relief_density`] mede o expoente de `aresta ∼ curvatura`, que é a única
//! forma de ver uma grade rigorosamente uniforme onde se pedia adaptação; e o ALCANCE
//! (a distância máxima ao centroide, impressa pelas sondas) é a única que vê amputação.

/// ⭐⭐⭐ **UMA BOLA COM ESPINHOS** — a fixtura que CONTÉM o fenómeno de 2026-08-29.
///
/// ⛔⛔ **Ela existe porque a peça do artista não está nesta árvore.** As fotos dele mostram
/// uma bola com cinco ou seis **espinhos longos e finos**, e o remesh **amputa-os**: os três
/// piores sítios da saída estão a `r = 1,80`, `1,73` e `1,53` (o corpo tem `r ≈ 1`), com
/// `16` e `12` vértices irregulares cada um e arestas de `0,34` contra uma mediana de
/// `0,031`. ⚠️ *Uma esfera lisa não contém nada disto, e era sobre esferas lisas que toda
/// medição desta linha corria.*
///
/// A construção: uma `uv_sphere` fina, e cada vértice é empurrado para fora ao longo do seu
/// raio por um envelope **gaussiano** centrado em cada direcção de espinho. ⚠️ O expoente do
/// envelope é o que torna o espinho **fino**: `exp(−(θ/σ)²)` com `σ` pequeno dá uma agulha,
/// que é exactamente o que a foto mostra.
pub(super) fn spiked_ball(n: usize, sigma: f32) -> ph2d_mesh::Mesh {
    let mut mesh = ph2d_mesh::shapes::uv_sphere(96, 144, 1.0);
    // ⚠️ **Direcções ESPALHADAS e não num eixo** — a espiral de Fibonacci evita que dois
    // espinhos caiam na mesma fileira de vértices da esfera, o que os tornaria o mesmo
    // espinho gordo em vez de dois finos.
    let golden = std::f32::consts::PI * (3.0 - 5.0f32.sqrt());
    let dirs: Vec<[f32; 3]> = (0..n.max(1))
        .map(|i| {
            let y = 1.0 - 2.0 * (i as f32 + 0.5) / n.max(1) as f32;
            let r = (1.0 - y * y).max(0.0).sqrt();
            let a = golden * i as f32;
            [r * a.cos(), y, r * a.sin()]
        })
        .collect();
    // ⭐⭐⭐ **OS NÚMEROS SÃO OS DA PEÇA DELE**, medidos no `.obj` que ele exportou: o
    // espinho mais longo vai a `r = 1,81` de um corpo de `r ~ 1`, e o **raio local** dele cai
    // de `0,147` (a 65 % do comprimento) para **`0,037`** (a 97 %). O F1 remalha com aresta
    // `0,089`, que e' **2,4x** a espessura da ponta.
    // A 1a redaccao usou `sigma = 0,22` e deu cones GORDOS: a cadeia devolvia casca fechada
    // em todas as densidades e a fixtura **nao continha o fenomeno**.
    let reach = 0.85f32;
    let pos = mesh.positions_mut();
    for p in pos.iter_mut() {
        let len = p[0].mul_add(p[0], p[1].mul_add(p[1], p[2] * p[2])).sqrt();
        if len < 1.0e-6 {
            continue;
        }
        let u = [p[0] / len, p[1] / len, p[2] / len];
        let mut grow = 0.0f32;
        for d in &dirs {
            let c = u[0]
                .mul_add(d[0], u[1].mul_add(d[1], u[2] * d[2]))
                .clamp(-1.0, 1.0);
            let ang = c.acos();
            grow = grow.max(reach * (-(ang / sigma) * (ang / sigma)).exp());
        }
        let k = 1.0 + grow * 1.2;
        for i in 0..3 {
            p[i] = u[i] * len * k;
        }
    }
    mesh.rebuild();
    mesh
}

/// ⭐⭐⭐ **AS ILHAS** — componentes ligados por aresta, e o que cada uma pesa.
///
/// ⛔ **Ela existe por uma foto (Enio, 28/08, «faces completamente soltas»):** a peça dele
/// saiu com o casco fechado (`χ = 2`, zero bordo, zero não-manifold) **e** uma ilha de
/// **duas** faces a flutuar — o MESMO quadrado emitido duas vezes, um virado ao contrário
/// do outro. ⚠️ *Nenhuma régua desta linha a via: todas mediam a malha inteira como um
/// objecto só.*
pub(super) fn islands(tag: &str, mesh: &ph2d_mesh::Mesh) {
    use std::collections::BTreeMap;
    let mut owners: BTreeMap<(u32, u32), Vec<usize>> = BTreeMap::new();
    for (fi, f) in mesh.faces().iter().enumerate() {
        let v = f.verts();
        for k in 0..v.len() {
            let (a, b) = (v[k], v[(k + 1) % v.len()]);
            owners.entry((a.min(b), a.max(b))).or_default().push(fi);
        }
    }
    let mut parent: Vec<usize> = (0..mesh.face_count()).collect();
    fn find(parent: &mut [usize], mut x: usize) -> usize {
        while parent[x] != x {
            parent[x] = parent[parent[x]];
            x = parent[x];
        }
        x
    }
    for fs in owners.values() {
        for w in fs.windows(2) {
            let (a, b) = (find(&mut parent[..], w[0]), find(&mut parent[..], w[1]));
            if a != b {
                parent[a] = b;
            }
        }
    }
    let mut size: BTreeMap<usize, usize> = BTreeMap::new();
    for i in 0..mesh.face_count() {
        let r = find(&mut parent[..], i);
        *size.entry(r).or_default() += 1;
    }
    let mut counts: Vec<usize> = size.values().copied().collect();
    counts.sort_unstable_by(|a, b| b.cmp(a));
    eprintln!(
        "   {tag}: {} ilha(s) -> {:?}",
        counts.len(),
        &counts[..counts.len().min(6)]
    );
    if counts.len() > 1 {
        let pos = mesh.positions();
        for (root, n) in &size {
            if *n > counts[0] / 2 {
                continue;
            }
            let mut c = [0.0f32; 3];
            let mut m = 0usize;
            for i in 0..mesh.face_count() {
                if find(&mut parent[..], i) != *root {
                    continue;
                }
                for &v in mesh.faces()[i].verts() {
                    let p = pos[v as usize];
                    for k in 0..3 {
                        c[k] += p[k];
                    }
                    m += 1;
                }
            }
            let inv = 1.0 / m.max(1) as f32;
            eprintln!(
                "      ilha SOLTA de {n} faces em ({:.3}, {:.3}, {:.3})",
                c[0] * inv,
                c[1] * inv,
                c[2] * inv
            );
        }
    }
}

/// ⭐⭐⭐ **A DENSIDADE SEGUE A FORMA?** — o expoente de `aresta ∼ curvatura^n`.
///
/// ⛔ **Ela é a régua do report de 2026-08-28** (*«as pontas finas… têm menos densidade de
/// faces e perdem detalhes»*), e nenhuma régua desta linha a tinha: todas mediam a aresta
/// **global** (mediana, máxima), que não muda quando a grade ignora a forma.
///
/// `0` = grade **uniforme** · negativo = mais fina onde a forma aperta. ⚠️ A faixa de
/// curvatura entre a 1.ª e a 8.ª banda sai ao lado, porque *um expoente sobre uma faixa de
/// `1,1×` não diz nada.*
pub(super) fn relief_density(tag: &str, mesh: &ph2d_mesh::Mesh) {
    let curv = mesh.curvatures();
    let pos = mesh.positions();
    let mut rows: Vec<(f32, f32)> = Vec::with_capacity(mesh.face_count());
    for f in mesh.faces() {
        let v = f.verts();
        let mut k = 0.0f32;
        let mut e = 0.0f32;
        for i in 0..v.len() {
            k += curv.get(v[i] as usize).copied().unwrap_or(0.0).abs();
            let (a, b) = (pos[v[i] as usize], pos[v[(i + 1) % v.len()] as usize]);
            let d = [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
            e += d[0].mul_add(d[0], d[1].mul_add(d[1], d[2] * d[2])).sqrt();
        }
        let n = v.len() as f32;
        if k > 1.0e-9 {
            rows.push((k / n, e / n));
        }
    }
    if rows.len() < 64 {
        eprintln!("   {tag}: poucas faces com curvatura para medir o expoente");
        return;
    }
    rows.sort_by(|a, b| a.0.total_cmp(&b.0));
    let bands = 8usize;
    let mut xs: Vec<f64> = Vec::new();
    let mut ys: Vec<f64> = Vec::new();
    for b in 0..bands {
        let (lo, hi) = (b * rows.len() / bands, (b + 1) * rows.len() / bands);
        let seg = &rows[lo..hi];
        let mut ks: Vec<f32> = seg.iter().map(|r| r.0).collect();
        let mut es: Vec<f32> = seg.iter().map(|r| r.1).collect();
        ks.sort_by(f32::total_cmp);
        es.sort_by(f32::total_cmp);
        xs.push(f64::from(ks[ks.len() / 2]).ln());
        ys.push(f64::from(es[es.len() / 2]).ln());
    }
    let mx = xs.iter().sum::<f64>() / xs.len() as f64;
    let my = ys.iter().sum::<f64>() / ys.len() as f64;
    let num: f64 = xs.iter().zip(&ys).map(|(x, y)| (x - mx) * (y - my)).sum();
    let den: f64 = xs.iter().map(|x| (x - mx) * (x - mx)).sum();
    eprintln!(
        "   {tag}: aresta ~ curvatura^{:.3}  (0 = UNIFORME) | faixa de curvatura {:.1}x | \
         banda fina {:.5} vs chapada {:.5}",
        if den > 0.0 { num / den } else { 0.0 },
        (xs[xs.len() - 1] - xs[0]).exp(),
        ys[ys.len() - 1].exp(),
        ys[0].exp(),
    );
}

/// A contagem de arestas de uma malha — bordo, não-manifold e `χ`.
pub(super) fn census(tag: &str, mesh: &ph2d_mesh::Mesh) {
    use std::collections::BTreeMap;
    let mut n: BTreeMap<(u32, u32), usize> = BTreeMap::new();
    for f in mesh.faces() {
        let v = f.verts();
        for k in 0..v.len() {
            let (a, b) = (v[k], v[(k + 1) % v.len()]);
            *n.entry((a.min(b), a.max(b))).or_default() += 1;
        }
    }
    // ⚠️ A valência conta **arestas únicas**, não incidências de face.
    let mut deg: BTreeMap<u32, usize> = BTreeMap::new();
    for (a, b) in n.keys() {
        *deg.entry(*a).or_default() += 1;
        *deg.entry(*b).or_default() += 1;
    }
    let bordo = n.values().filter(|c| **c == 1).count();
    let nm = n.values().filter(|c| **c > 2).count();
    let chi = mesh.vert_count() as i64 - n.len() as i64 + mesh.face_count() as i64;
    let worst = deg.values().copied().max().unwrap_or(0);
    let irregular = deg.values().filter(|d| **d != 4).count();
    eprintln!(
        "   {tag}: {} verts {} faces | X = {chi} | {bordo} bordo | {nm} nao-manifold | \
         valencia max {worst} | {irregular} irregulares",
        mesh.vert_count(),
        mesh.face_count(),
    );
}

/// **ONDE os furos estão** — um por linha, com o perímetro e o centro.
///
/// ⚠️ *Uma contagem de arestas de bordo diz que há furo; ela não diz que ele está na
/// PONTA*, que foi a palavra do artista.
pub(super) fn holes(tag: &str, mesh: &ph2d_mesh::Mesh) {
    use std::collections::BTreeMap;
    let mut n: BTreeMap<(u32, u32), usize> = BTreeMap::new();
    for f in mesh.faces() {
        let v = f.verts();
        for k in 0..v.len() {
            let (a, b) = (v[k], v[(k + 1) % v.len()]);
            *n.entry((a.min(b), a.max(b))).or_default() += 1;
        }
    }
    let open: Vec<(u32, u32)> = n
        .into_iter()
        .filter(|(_, c)| *c == 1)
        .map(|(e, _)| e)
        .collect();
    if open.is_empty() {
        return;
    }
    // Componentes ligados das arestas abertas — cada um é um furo.
    let mut next: BTreeMap<u32, Vec<u32>> = BTreeMap::new();
    for &(a, b) in &open {
        next.entry(a).or_default().push(b);
        next.entry(b).or_default().push(a);
    }
    let pos = mesh.positions();
    let mut seen: std::collections::BTreeSet<u32> = std::collections::BTreeSet::new();
    let mut loops = 0usize;
    for &start in next.keys() {
        if !seen.insert(start) {
            continue;
        }
        let mut stack = vec![start];
        let mut verts = vec![start];
        while let Some(v) = stack.pop() {
            for &w in next.get(&v).map_or(&[][..], Vec::as_slice) {
                if seen.insert(w) {
                    verts.push(w);
                    stack.push(w);
                }
            }
        }
        let mut c = [0.0f32; 3];
        for &v in &verts {
            let p = pos[v as usize];
            for k in 0..3 {
                c[k] += p[k];
            }
        }
        let inv = 1.0 / verts.len() as f32;
        loops += 1;
        eprintln!(
            "   {tag}: furo #{loops} com {} vertices, centro ({:.3}, {:.3}, {:.3})",
            verts.len(),
            c[0] * inv,
            c[1] * inv,
            c[2] * inv,
        );
    }
}
