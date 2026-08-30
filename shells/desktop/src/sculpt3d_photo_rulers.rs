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

/// ⭐⭐⭐ **A RÉGUA LOCAL, impressa** — ver [`ph2d_quadfill::LocalShape`].
///
/// ⚠️ **As colunas vêm em PARES de numerador e denominador**, e isso é a razão de
/// esta sonda existir: *«12 defeitos na ponta»* significa coisas opostas se a ponta
/// tem `20` faces ou `2 000`. O `QuadShape` resume em percentis e por isso não pode
/// responder à frase do artista.
pub(super) fn local(tag: &str, mesh: &ph2d_mesh::Mesh) {
    let (s, per) = ph2d_quadfill::local_shape(mesh);
    let n = per.len().max(1);
    #[allow(clippy::cast_precision_loss)]
    let pct = |k: usize| 100.0 * k as f32 / n as f32;
    eprintln!(
        "   {tag}: LOCAL {} defeito(s) em {} faces ({:.2} %) | gravatas {} torcidas {} lascas {} \
         | torcao max {:.1} p99 {:.1}",
        s.defects,
        per.len(),
        pct(s.defects),
        s.bowties,
        s.warped,
        s.slivers,
        s.warp_max,
        s.warp_p99,
    );
    #[allow(clippy::cast_precision_loss)]
    let densidade = if s.faces_at_tip == 0 {
        0.0
    } else {
        100.0 * s.defects_at_tip as f32 / s.faces_at_tip as f32
    };
    eprintln!(
        "   {tag}: LOCAL na PONTA {}/{} faces ({:.2} %) contra {:.2} % na peca inteira",
        s.defects_at_tip,
        s.faces_at_tip,
        densidade,
        pct(s.defects),
    );
    // ⚠️ **Os piores, com o RAIO ao lado** — sem ele a lista não diz se eles estão
    // onde o artista aponta.
    let mut idx: Vec<usize> = (0..per.len()).filter(|&i| per[i].is_defect()).collect();
    idx.sort_by(|&a, &b| per[b].warp_deg.total_cmp(&per[a].warp_deg));
    for &i in idx.iter().take(6) {
        let d = per[i];
        eprintln!(
            "      face {i}: torcao {:.1} | {:?} | quadratura {:.3} | raio {:.2}",
            d.warp_deg, d.kind, d.squareness, d.radial
        );
    }
}

/// ⭐⭐⭐ **A ORIENTAÇÃO e a DENSIDADE POR RAIO** — as duas que sobram depois de a
/// torção, a gravata e a lasca serem ilibadas (medido 2026-08-30).
///
/// ⛔⛔ **A orientação é a que literalmente se lê como BURACO.** Duas faces
/// vizinhas com enrolamento oposto apontam para lados contrários; num viewport
/// sombreado uma delas é vista **por dentro** e sai preta. ⚠️ **E nenhuma régua
/// deste repo a mede:** `χ` não muda, o bordo é zero, o não-manifold conta arestas
/// com `≠ 2` faces — e aqui as duas faces estão lá. *A assinatura é a aresta
/// interior percorrida no MESMO sentido pelas duas.*
///
/// ⛔ **A densidade por raio é a outra queixa dele** — *«as pontas têm menos
/// densidade de faces»*. Ela mede a área mediana de face por casca radial: se a
/// última casca tem faces muito maiores, a ponta é grosseira, e isso não move
/// nenhuma mediana global.
pub(super) fn orientation_and_density(tag: &str, mesh: &ph2d_mesh::Mesh) {
    use std::collections::BTreeMap;
    let faces = mesh.faces();
    let pos = mesh.positions();

    // ⚠️ A chave é a aresta NÃO-ORIENTADA; o valor conta cada SENTIDO.
    let mut dir: BTreeMap<(u32, u32), (usize, usize)> = BTreeMap::new();
    for f in faces {
        let v = f.verts();
        for k in 0..v.len() {
            let (a, b) = (v[k], v[(k + 1) % v.len()]);
            let e = if a < b { (a, b) } else { (b, a) };
            let slot = dir.entry(e).or_insert((0, 0));
            if a < b {
                slot.0 += 1;
            } else {
                slot.1 += 1;
            }
        }
    }
    let mut inconsistent = 0usize;
    let mut interior = 0usize;
    for (fwd, bwd) in dir.values() {
        if fwd + bwd == 2 {
            interior += 1;
            // ⭐ Numa superfície orientada, toda aresta interior é percorrida
            // **uma vez em cada sentido**. `2/0` ou `0/2` é enrolamento oposto.
            if *fwd == 2 || *bwd == 2 {
                inconsistent += 1;
            }
        }
    }
    #[allow(clippy::cast_precision_loss)]
    let pct = 100.0 * inconsistent as f32 / interior.max(1) as f32;
    eprintln!(
        "   {tag}: ORIENTACAO {inconsistent} de {interior} arestas interiores viradas ({pct:.3} %)"
    );

    // A densidade por casca radial.
    let mut c = [0.0f64; 3];
    let cent: Vec<[f32; 3]> = faces
        .iter()
        .map(|f| {
            let v = f.verts();
            let mut p = [0.0f32; 3];
            for &i in v {
                let q = pos[i as usize];
                for k in 0..3 {
                    p[k] += q[k];
                }
            }
            #[allow(clippy::cast_precision_loss)]
            let n = v.len() as f32;
            [p[0] / n, p[1] / n, p[2] / n]
        })
        .collect();
    for p in &cent {
        for k in 0..3 {
            c[k] += f64::from(p[k]);
        }
    }
    #[allow(clippy::cast_precision_loss)]
    let n = cent.len().max(1) as f64;
    let mid = [(c[0] / n) as f32, (c[1] / n) as f32, (c[2] / n) as f32];
    let area = |f: &ph2d_mesh::Face| -> f32 {
        let v = f.verts();
        let mut s = [0.0f32; 3];
        for k in 0..v.len() {
            let a = pos[v[k] as usize];
            let b = pos[v[(k + 1) % v.len()] as usize];
            s[0] += a[1].mul_add(b[2], -(a[2] * b[1]));
            s[1] += a[2].mul_add(b[0], -(a[0] * b[2]));
            s[2] += a[0].mul_add(b[1], -(a[1] * b[0]));
        }
        0.5 * s[0].mul_add(s[0], s[1].mul_add(s[1], s[2] * s[2])).sqrt()
    };
    let d: Vec<f32> = cent
        .iter()
        .map(|p| {
            let q = [p[0] - mid[0], p[1] - mid[1], p[2] - mid[2]];
            q[0].mul_add(q[0], q[1].mul_add(q[1], q[2] * q[2])).sqrt()
        })
        .collect();
    let far = d.iter().copied().fold(0.0f32, f32::max).max(1.0e-9);
    let mut shells: [Vec<f32>; 5] = Default::default();
    for (i, f) in faces.iter().enumerate() {
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let s = ((d[i] / far) * 5.0).floor().min(4.0) as usize;
        shells[s].push(area(f).sqrt());
    }
    let mut linha = String::new();
    for (k, s) in shells.iter_mut().enumerate() {
        s.sort_by(f32::total_cmp);
        let med = if s.is_empty() { 0.0 } else { s[s.len() / 2] };
        linha.push_str(&format!(
            " [{:.1}-{:.1}] {med:.4}×{}",
            k as f32 * 0.2,
            (k + 1) as f32 * 0.2,
            s.len()
        ));
    }
    eprintln!("   {tag}: ARESTA-EQUIVALENTE por casca radial:{linha}");
    // ⭐⭐⭐ **A RAZÃO PONTA/CORPO, num número só** — a coluna que o report do
    // artista pede, e a única com alvo DERIVADO: a saída do QRemeshify que ele
    // aprovou mede `0,59`, e a nossa de 30/08 media `1,18`.
    //
    // ⚠️ **Pela porta partilhada** ([`ph2d_quadfill::tip_body_ratio`]) — a mesma
    // que mede o PEDIDO do campo de passo. Recalcular aqui daria dois números que
    // ninguém pode dividir um pelo outro.
    let raiz_area: Vec<f32> = faces.iter().map(|f| area(f).sqrt()).collect();
    let (razao, amostra) = ph2d_quadfill::tip_body_ratio(&cent, &raiz_area);
    eprintln!(
        "   {tag}: ENTREGA razao ponta/corpo {razao:.3}  (alvo derivado do oraculo \
         aprovado: 0,59 · <1 afina na ponta, >1 engrossa) [amostra da ponta: {amostra} faces]"
    );
}
