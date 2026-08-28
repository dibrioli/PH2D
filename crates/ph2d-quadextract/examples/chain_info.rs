//! ⭐⭐⭐ **A CADEIA INTEIRA DA CASA**, com a **fase zero** honrada — e a única
//! medição em que a barra do oráculo (`4,8°`–`7,1°` de enviesamento) é comparável.
//!
//! ```text
//! cargo run --release -p ph2d-quadextract --example chain_info -- [peca] [densidade]
//! ```
//!
//! ⛔ **`--release`.** O solver contínuo gasta `160 000` rondas de Gauss–Seidel; em
//! `debug` isto é minutos por peça.

fn median_edge(mesh: &ph2d_mesh::Mesh) -> f32 {
    let pos = mesh.positions();
    let mut e: Vec<f32> = Vec::new();
    for f in mesh.faces() {
        let v = f.verts();
        for k in 0..v.len() {
            let (a, b) = (pos[v[k] as usize], pos[v[(k + 1) % v.len()] as usize]);
            let d = [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
            e.push(d[0].mul_add(d[0], d[1].mul_add(d[1], d[2] * d[2])).sqrt());
        }
    }
    e.sort_by(f32::total_cmp);
    e[e.len() / 2]
}

fn aspect(mesh: &ph2d_mesh::Mesh) -> (f32, f32) {
    let pos = mesh.positions();
    let mut a: Vec<f32> = Vec::new();
    for f in mesh.faces() {
        let v = f.verts();
        let mut lo = f32::MAX;
        let mut hi = 0.0f32;
        for k in 0..v.len() {
            let (p, q) = (pos[v[k] as usize], pos[v[(k + 1) % v.len()] as usize]);
            let d = [p[0] - q[0], p[1] - q[1], p[2] - q[2]];
            let l = d[0].mul_add(d[0], d[1].mul_add(d[1], d[2] * d[2])).sqrt();
            lo = lo.min(l);
            hi = hi.max(l);
        }
        a.push(hi / lo.max(1.0e-20));
    }
    a.sort_by(f32::total_cmp);
    (a[a.len() / 2], a[(a.len() - 1) * 99 / 100])
}

/// ⭐⭐⭐ **A BANDA DE RAIO de um conjunto de vértices** — a coordenada em que «ponta»
/// quer dizer alguma coisa.
///
/// ⛔ Toda régua deste instrumento era um TOTAL, e o report do artista de 2026-08-25 é
/// sobre POSIÇÃO. Devolve `p50 / p90` em múltiplos do raio **mediano** da peça, para se
/// ler contra o `p99` dela.
fn radius_band(pos: &[[f32; 3]], who: &[u32]) -> String {
    if pos.is_empty() {
        return String::from("(peca vazia)");
    }
    let c = pos.iter().fold([0.0f64; 3], |a, p| {
        [
            a[0] + f64::from(p[0]),
            a[1] + f64::from(p[1]),
            a[2] + f64::from(p[2]),
        ]
    });
    #[allow(clippy::cast_precision_loss)]
    let inv = 1.0 / pos.len() as f64;
    let c = [c[0] * inv, c[1] * inv, c[2] * inv];
    let r = |i: usize| {
        let p = pos[i];
        let d = [
            f64::from(p[0]) - c[0],
            f64::from(p[1]) - c[1],
            f64::from(p[2]) - c[2],
        ];
        d[0].mul_add(d[0], d[1].mul_add(d[1], d[2] * d[2])).sqrt()
    };
    let mut all: Vec<f64> = (0..pos.len()).map(r).collect();
    all.sort_by(f64::total_cmp);
    let med = all[all.len() / 2].max(1.0e-12);
    let p99 = all[all.len() * 99 / 100] / med;
    if who.is_empty() {
        return format!("(nenhum) [a peca vai ate' {p99:.2}x]");
    }
    let mut v: Vec<f64> = who.iter().map(|&i| r(i as usize) / med).collect();
    v.sort_by(f64::total_cmp);
    format!(
        "raio p50 {:.2}x p90 {:.2}x [a peca vai ate' {p99:.2}x]",
        v[v.len() / 2],
        v[v.len() * 9 / 10]
    )
}

/// ⭐⭐⭐ **A RÉGUA DE FIDELIDADE — quanto a saída se afasta da ESCULTURA.**
///
/// ⛔⛔ Ela existe por causa da terceira queixa do artista (2026-08-25): *«quanto mais densa
/// a malha gerada, maiores as irregularidades da superfície que deveria ser lisa»*. ⚠️ **Nenhuma
/// régua desta cadeia a media** — todas falam da FORMA dos quads (aspecto, enviesamento, área),
/// e nenhuma da **distância entre a peça que sai e a peça que ele fez**.
///
/// ⚠️ **Duas colunas, e a segunda é a que a luz mostra.** O desvio de POSIÇÃO diz se a peça
/// mudou de sítio; o desvio de NORMAL diz se ela ficou **facetada** — e é a normal que o
/// sombreado revela. *Uma peça pode estar a `0,1 %` de distância e parecer um diamante.*
///
/// ⭐ **E a coluna que decide a hipótese é o F1**: a saída é medida contra a peça crua **e**
/// contra a remalhada. Se o erro contra a crua não descer quando a grade fica mais fina,
/// mas o erro contra a remalhada descer, então o tecto é o **substrato** — o F1 corre a uma
/// densidade FIXA (`ALPHA × diagonal`) qualquer que seja a densidade pedida.
fn fidelity(raw: &ph2d_mesh::Mesh, f1: &ph2d_mesh::Mesh, out: &ph2d_mesh::Mesh) {
    let b = raw.bounds();
    let diag = {
        let d = [
            b.max[0] - b.min[0],
            b.max[1] - b.min[1],
            b.max[2] - b.min[2],
        ];
        d[0].mul_add(d[0], d[1].mul_add(d[1], d[2] * d[2])).sqrt()
    };
    let seed = diag * 0.05;
    let dev = |against: &ph2d_mesh::Mesh| {
        let mut v: Vec<f32> = out
            .positions()
            .iter()
            .map(|&p| {
                let q = ph2d_remesh_iso::project_onto(against, p, seed);
                let d = [p[0] - q[0], p[1] - q[1], p[2] - q[2]];
                d[0].mul_add(d[0], d[1].mul_add(d[1], d[2] * d[2])).sqrt() / diag
            })
            .collect();
        v.sort_by(f32::total_cmp);
        (
            v.get(v.len() / 2).copied().unwrap_or(0.0) * 100.0,
            v.get(v.len() * 95 / 100).copied().unwrap_or(0.0) * 100.0,
            v.last().copied().unwrap_or(0.0) * 100.0,
        )
    };
    let (r50, r95, rmax) = dev(raw);
    let (f50, f95, fmax) = dev(f1);

    // ⛔⛔⛔ **ESTA COLUNA É TAUTOLÓGICA DEPOIS DO ACABAMENTO, e o aviso já estava escrito.**
    //
    // O doc do [`ph2d_quadfill::detail_lost`] regista que `saída → referência` dá ~zero
    // **mesmo numa malha destruída** — medido em 2026-08-21: `0,0000` na destruída contra
    // `0,0015` na boa, *a destruída a pontuar melhor*. E desde 2026-08-26 o acabamento
    // **pousa** cada vértice na referência, por construção. ⇒ **`0,000` aqui não é uma
    // vitória: é a definição da operação.**
    //
    // ⚠️ Ela FICA porque continua a separar «a saída vive sobre o F1» de «vive sobre a
    // escultura» — que foi o achado do §18 —, mas ⛔ **a régua de fidelidade a sério é a de
    // baixo**, e é ela que responde *«todo pedaço que o artista esculpiu tem malha nova por
    // perto?»*.
    println!(
        "  ⚠️ SOBRE-O-QUE (tautologica com acabamento): contra a ESCULTURA p50 {r50:.3} p95 {r95:.3} max {rmax:.3} | contra o F1 p50 {f50:.3} p95 {f95:.3} max {fmax:.3}"
    );
    let (lost95, lostmax) = ph2d_quadfill::detail_lost(raw, out);
    let (relief, conf) = ph2d_quadfill::follows_relief(raw, out);
    println!(
        "  ⭐⭐⭐ FIDELIDADE a serio (referencia → saida, % da diagonal): DETALHE PERDIDO p95 {:.3} max {:.3}",
        lost95 * 100.0,
        lostmax * 100.0
    );
    println!(
        "  ⭐⭐⭐ OBEDECE AO RELEVO: {relief:.1}° (confianca {conf:.2}) — ⚠️ 22,5° = «nao olhou»"
    );

    // ⭐⭐⭐ **A RUGOSIDADE DAS TRÊS MALHAS, lado a lado** — e as duas primeiras são o controlo.
    //
    // ⛔⛔ **Sozinha, a rugosidade da saída não acusa ninguém.** Uma escultura rugosa dá uma
    // saída rugosa, e isso é a cadeia a **fazer o seu trabalho**. A pergunta é se a rugosidade
    // que sai é a que **entra** — ⇒ mede-se a crua, a remalhada e a saída, com a contagem de
    // faces ao lado, porque ⚠️ **a dobra entre vizinhas encolhe com a densidade**: comparar
    // 40 000 triângulos com 2 000 quads sem essa coluna é comparar duas réguas diferentes.
    for (rotulo, m) in [("crua ", raw), ("F1   ", f1), ("saida", out)] {
        let (p50, p95, pmax, over) = roughness(m);
        println!(
            "  ⭐⭐ RUGOSIDADE {rotulo}: p50 {p50:.1}° p95 {p95:.1}° max {pmax:.1}° · {over} arestas acima de 30° · {} faces",
            m.face_count()
        );
    }
}

/// A **dobra entre faces vizinhas**, em graus — `(p50, p95, max, quantas acima de 30°)`.
///
/// ⚠️ É esta grandeza que o sombreado mostra: uma peça pode estar a `0,1 %` de distância da
/// original e parecer um diamante. ⛔ E ela **depende da densidade** — só se compara entre
/// malhas de contagem parecida, ou com a contagem escrita ao lado.
fn roughness(m: &ph2d_mesh::Mesh) -> (f32, f32, f32, usize) {
    let n = m.face_normals();
    let mut owner: std::collections::BTreeMap<(u32, u32), Vec<usize>> =
        std::collections::BTreeMap::new();
    for (fi, f) in m.faces().iter().enumerate() {
        let v = f.verts();
        for k in 0..v.len() {
            let (a, b) = (v[k], v[(k + 1) % v.len()]);
            owner
                .entry(if a < b { (a, b) } else { (b, a) })
                .or_default()
                .push(fi);
        }
    }
    let mut kink: Vec<f32> = owner
        .values()
        .filter(|w| w.len() == 2)
        .map(|w| {
            let (a, b) = (n[w[0]], n[w[1]]);
            a[0].mul_add(b[0], a[1].mul_add(b[1], a[2] * b[2]))
                .clamp(-1.0, 1.0)
                .acos()
                .to_degrees()
        })
        .collect();
    kink.sort_by(f32::total_cmp);
    let at = |q: usize| kink.get(kink.len() * q / 100).copied().unwrap_or(0.0);
    (
        at(50),
        at(95),
        kink.last().copied().unwrap_or(0.0),
        kink.iter().filter(|k| **k > 30.0).count(),
    )
}

fn main() {
    let mut args = std::env::args().skip(1);
    let piece = args.next().unwrap_or_else(|| String::from("esfera"));
    let scale: f32 = args.next().and_then(|s| s.parse().ok()).unwrap_or(1.0);
    // ⭐⭐ **UM CAMINHO DE FICHEIRO É UMA PEÇA.** ⚠️ *Uma régua só prova o que a
    // fixtura contém* — as formas analíticas acima não têm relevo, e a queixa do
    // artista («não é fiel à curvatura») é **sobre relevo**. Medi-la nas peças que
    // ele de facto viu exige carregar as peças que ele de facto viu.
    let mut mesh = if piece.ends_with(".obj") {
        let text = std::fs::read_to_string(&piece)
            .unwrap_or_else(|e| panic!("nao consegui ler {piece}: {e}"));
        let pieces = ph2d_mesh::import_obj(&text)
            .unwrap_or_else(|e| panic!("{piece} nao e' um OBJ que este leitor entenda: {e:?}"));
        let first = pieces
            .into_iter()
            .next()
            .unwrap_or_else(|| panic!("{piece} nao tem uma peca dentro"));
        first.mesh
    } else {
        match piece.as_str() {
            "toro" => ph2d_mesh::shapes::torus(64, 32, 1.0, 0.35),
            "esfera-fina" => ph2d_mesh::shapes::uv_sphere(96, 144, 1.0),
            "esfera-irregular" => ph2d_mesh::shapes::uv_sphere_shuffled(48, 72, 1.0),
            // ⭐ AS PONTAS. O corpus não tem nenhuma verdadeiramente aguda, e o report
            // do artista (2026-08-25) nomeia «ponta, chifres» como o pior caso. *Uma
            // fixtura que não contém o fenómeno não o pode medir.*
            "octaedro" => ph2d_mesh::shapes::octahedron(1.0),
            "cilindro" => ph2d_mesh::shapes::cylinder(64, 0.5, 1.5),
            _ => ph2d_mesh::shapes::uv_sphere(24, 36, 1.0),
        }
    };
    mesh.triangulate();
    let (a50, a99) = aspect(&mesh);
    println!(
        "{piece}: {} faces cruas, aspecto p50 {a50:.2} p99 {a99:.2}",
        mesh.face_count()
    );
    // ⭐⭐⭐ **A ESCULTURA COMO ELE A FEZ** — guardada para a régua de FIDELIDADE.
    //
    // ⛔ A queixa dele de 2026-08-25 — *«quanto mais densa a malha gerada, maiores as
    // irregularidades da superfície que deveria ser lisa»* — não é sobre a forma dos quads,
    // e **nenhuma régua desta cadeia a media**. Ela é sobre a distância entre a saída e a
    // peça que ele esculpiu, e ⚠️ o denominador tem de ser a peça CRUA, nunca a remalhada:
    // medir contra o F1 responde *«o extractor seguiu o F1?»*, que é outra pergunta.
    let raw = mesh.clone();

    // ── ⛔⛔ FASE ZERO. Sem ela a mesma cadeia dá `10-12°`, e o defeito e' a entrada.
    // ⭐⭐⭐ **A DENSIDADE DA FASE ZERO, varrível** — `PH2D_ALPHA`.
    //
    // ⛔ Ela é uma **constante** e o produto nunca a move, qualquer que seja a densidade que
    // o artista peça. ⚠️ *Se o relevo já morre aqui, nenhum peso de alinhamento a jusante o
    // pode recuperar* — e é isso que esta env existe para responder, com a régua
    // `follows_relief` do lado.
    let alpha = std::env::var("PH2D_ALPHA")
        .ok()
        .and_then(|v| v.parse::<f32>().ok())
        .unwrap_or(ph2d_remesh_iso::ALPHA);
    let f1 = ph2d_remesh_iso::remesh_isotropic(&mut mesh, alpha);
    println!(
        "  ⭐⭐⭐ MANIFOLD na porta: {} arestas mas ⇒ {} · {} vertices partidos, {} copias",
        f1.manifold.bad_edges_before,
        f1.manifold.bad_edges_after,
        f1.manifold.split_verts,
        f1.manifold.copies
    );
    mesh.triangulate();
    let (a50, a99) = aspect(&mesh);
    println!(
        "  F1 (fase zero): {} faces, aspecto p50 {a50:.2} p99 {a99:.2}  (a assinatura do F1 e' 1,16 / 1,58)",
        mesh.face_count()
    );

    // ⚠️ O `h` sobe para ANTES do campo: a lei da feição mede-se em múltiplos do passo alvo
    // da grade, e a detecção corre antes do F2 porque é ela que o restringe.
    let h = median_edge(&mesh) * scale;
    let mut dual = ph2d_crossfield::Dual::build(&mesh);
    // ⭐⭐⭐ **O BORDO COMO FEIÇÃO** — a única feição EXACTA, sem limiar nenhum.
    //
    // ⚠️ Separado do bloco da curvatura de propósito: aquele tem cinco coeficientes e pode
    // marcar a mais; este não tem nenhum. *Misturá-los faria a varredura de um responder
    // pelo outro.* ⭐ **LIGADO por omissão** (`PH2D_BOUNDARY_FEATURE=0` bissecta): inerte em
    // peça fechada por construção, e as duas com bordo eram as duas piores do corpus.
    if std::env::var("PH2D_BOUNDARY_FEATURE").as_deref() != Ok("0") {
        let (be, loops) = ph2d_mesh::boundary_feature_edges(&mesh);
        let cr = dual.constrain(&mesh, &be);
        println!(
            "  BORDO como feicao: {loops} lacos ⇒ {} arestas ⇒ {} faces fixas, {} conflitos",
            be.len(),
            cr.faces,
            cr.conflicts
        );
    }

    // ⭐⭐ **AS LINHAS DE FEIÇÃO** (obra B, `SPEC_restricoes_por_eliminacao.md` §3) — o 1.º
    // dos três consumidores. ⛔ Nasce DESLIGADA: a régua desta obra é «a peça ficou melhor»,
    // e enquanto a tabela não estiver escrita ela não entra no caminho de ninguém.
    if std::env::var("PH2D_FEATURE_EDGES").as_deref() == Ok("1") {
        // ⚠️ Os cinco coeficientes entram por ENV para que a varredura corra sobre a régua
        // que decide — **a peça no fim da cadeia** — e não sobre a contagem de
        // singularidades, que é só o sinal de alarme do gate nº7.
        let num = |k: &str, d: f32| {
            std::env::var(k)
                .ok()
                .and_then(|v| v.parse::<f32>().ok())
                .unwrap_or(d)
        };
        let base = ph2d_mesh::FeatureOptions::default();
        let opts = ph2d_mesh::FeatureOptions {
            r1_in_h: num("PH2D_FEATURE_R1", base.r1_in_h),
            half_window_in_h: num("PH2D_FEATURE_WIN", base.half_window_in_h),
            min_anisotropy: num("PH2D_FEATURE_ANISO", base.min_anisotropy),
            ..base
        };
        let (fd, fr) = ph2d_mesh::feature_dirs(&mesh, h, opts);
        let (fe, er) = ph2d_mesh::feature_edges(
            &mesh,
            &fd,
            num("PH2D_FEATURE_COS", ph2d_mesh::FEATURE_EDGE_MIN_COS),
        );
        let cr = dual.constrain(&mesh, &fe);
        println!(
            "  FEICAO: {} vertices marcados ({} recusados pela janela) ⇒ {} arestas \
             ({:.2}% da peca) ⇒ {} faces fixas, {} conflitos",
            fr.marked,
            fr.rejected_window,
            er.kept,
            er.sparsity_pct(),
            cr.faces,
            cr.conflicts
        );
    }
    // ⭐⭐⭐ **O PESO DO ALINHAMENTO AO RELEVO, varrível.**
    //
    // ⛔ Ele shipa a `0,03` desde 2026-08-22, e o número foi escolhido pelo **campo do
    // oráculo** — não pela régua `follows_relief`, que só entrou neste instrumento em
    // 2026-08-26. ⚠️ *Um número escolhido por uma régua tem de ser reconferido quando outra
    // régua chega.* `PH2D_ALIGN_WEIGHT` varre-o.
    let (field, _) = match std::env::var("PH2D_ALIGN_WEIGHT")
        .ok()
        .and_then(|v| v.parse::<f32>().ok())
    {
        Some(w) => {
            ph2d_crossfield::solve_miq_aligned(&dual, ph2d_crossfield::Rounding::default(), w)
        }
        None => ph2d_crossfield::solve_miq(&dual),
    };
    let singular: Vec<u32> = ph2d_crossfield::vertex_index(&mesh, &dual, &field)
        .into_iter()
        .enumerate()
        .filter(|(_, k)| *k != 0)
        .filter_map(|(v, _)| u32::try_from(v).ok())
        .collect();
    let layout = ph2d_trace::trace_patches(&mesh, &dual, &field);

    // ⭐⭐⭐ **PREGAR TAMBÉM OS CANTOS DO LAYOUT** (`PH2D_PIN_CORNERS=1`).
    //
    // Hoje só as singularidades do CAMPO vão para o G5 com pedido de ponto inteiro. Os
    // **cantos do layout** — os extremos de cada arco — não vão, e a §23.14 mediu que
    // praticamente nenhum arco sai isolinha. ⚠️ *Pregar os extremos num inteiro não
    // torna um arco isolinha* (isso é a restrição de a componente atravessada ser zero);
    // é a **pré-condição** dela, e é a mudança mais pequena que a régua consegue medir.
    //
    // ⛔ Instrumento apenas — nada disto entra no produto sem o número ao lado.
    let singular: Vec<u32> = if std::env::var("PH2D_PIN_CORNERS").as_deref() == Ok("1") {
        let mut set: std::collections::BTreeSet<u32> = singular.into_iter().collect();
        let before = set.len();
        set.extend(layout.corners.iter().flatten().copied());
        println!(
            "  ⭐ PIN_CORNERS: {before} singulares do campo + cantos do layout ⇒ {} pedidos",
            set.len()
        );
        set.into_iter().collect()
    } else {
        singular
    };
    // ⭐⭐⭐ **AS CONDIÇÕES DE VALIDADE DO PATCH** (o achado da ordem das fases, §2.3):
    // disco · valência `3`–`6` · convexidade. ⚠️ **As duas primeiras já eram MEDIDAS pelo
    // traçado** (`TraceReport::valence`, `non_disk`) e por um método do layout
    // (`degenerate()`), e **nenhum instrumento as imprimia** — a terceira ainda não existe.
    // *É o terceiro contador vermelho, num dia só, que ninguém estava a ler.*
    {
        let r = &layout.report;
        let fora: usize = r
            .valence
            .iter()
            .filter(|(k, _)| **k < 3 || **k > 6)
            .map(|(_, n)| *n)
            .sum();
        println!(
            "  ⭐⭐⭐ CONDICOES DO PATCH: valencia {:?} · ⛔ {} fora de 3..6 · {} NAO-DISCO \
             · χ dos patches {:?} · ⛔ {} degenerados SOBREVIVERAM a' limpeza ({} dissolvidos em {} rondas, parou por {}) · poda: {} tocos · ⭐ {} rondas de CORTE",
            r.valence,
            fora,
            r.non_disk,
            {
                // ⭐ **O χ de cada patch, em histograma.** ⚠️ `1` é disco · `≤ 0` é anel ou
                // pior · `≥ 2` é PARTIDO — e as duas classes pedem cortes diferentes:
                // um anel abre-se ligando duas voltas de bordo, um partido não tem caminho
                // entre as componentes. *Um contador de «não-disco» junta as duas.*
                let mut h: std::collections::BTreeMap<i64, usize> =
                    std::collections::BTreeMap::new();
                for c in &layout.chi {
                    *h.entry(*c).or_default() += 1;
                }
                h
            },
            layout.degenerate().len(),
            r.dissolved,
            r.rounds,
            match r.cleanup_stop {
                0 => "nada a fazer",
                1 => "⛔ a DISSOLUCAO nao removeu parede nenhuma",
                2 => "⛔ a ronda PIORAVA a topologia",
                _ => "⛔ o tecto de rondas",
            },
            r.pruned,
            r.opened_rings
        );
    }
    // ⭐ **O RETRATO de cada patch degenerado**, e não o total: lados · laços de fronteira ·
    // `χ`. ⚠️ *«5 degenerados» junta pelo menos duas avarias com curas opostas* — a lasca de
    // poucos lados cura-se FUNDINDO, e o que tem laços a mais cura-se CORTANDO.
    {
        let bad = layout.degenerate();
        let retrato: Vec<(usize, usize, usize, i64)> = bad
            .iter()
            .map(|&p| {
                (
                    p,
                    layout.side_arcs[p].len(),
                    layout.loops_per_patch.get(p).copied().unwrap_or(1),
                    layout.chi.get(p).copied().unwrap_or(1),
                )
            })
            .collect();
        println!("  ⭐ DEGENERADOS (patch, lados, lacos, χ): {retrato:?}");
        // ⭐⭐⭐ **A RÉGUA DO VÃO** — a que distância as duas fronteiras passam uma da
        // outra. ⚠️ `1` é ESTRANGULADO (cortar ali só acrescenta um toco); um vão maior é
        // um anel gordo, e esse corta-se.
        let gaps = ph2d_trace::patches::ring_gaps(&mesh, &layout);
        println!("  ⭐⭐⭐ VAO (patch, lados, VAO, faces, TAMANHOS das fronteiras): {gaps:?}");
    }
    let (cut, cr) = ph2d_gridmap::cut_along_patches(&mesh, &layout);
    let (combed, comb) = ph2d_gridmap::comb_patches(&mesh, &layout, &cut);
    println!(
        "  F2+F3+G1+G2: {} patches ({} discos), {} costuras ({} com salto, ⛔ {} SEM salto \
         = nao acopladas) · ⛔ NAO-DISCOS: {} anel {} partido, {} abertos aqui, \
         ⛔⛔ {} POR ABRIR (a fase seguinte NAO os parametriza) · ⭐ {} PARTIDOS separados",
        cr.patches,
        cr.discs,
        comb.seams,
        comb.jumps,
        comb.seams.saturating_sub(comb.jumps),
        cr.not_discs[0],
        cr.not_discs[1],
        cr.opened,
        cr.unopened,
        cr.split_patches
    );

    let t = std::time::Instant::now();
    let pin = std::env::args().nth(3).as_deref() != Some("sem-singularidades");
    // ⭐ O caminho SOLDADO (a costura entra por eliminação, não por peso).
    // ⚠️ **O interruptor é lido pela porta da crate**, não aqui: as duas portas leram-no
    // com sentidos opostos até 2026-08-24.
    let welded = ph2d_gridmap::welded_enabled();
    // ⚠️ As duas alavancas que o diagnostico das DOBRAS precisa de poder mexer sem
    // recompilar: quantas rondas o continuo corre, e se as singularidades sao pregadas.
    let num = |k: &str, d: usize| {
        std::env::var(k)
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(d)
    };
    let base_opts = ph2d_gridmap::RoundOptions::default();
    let opts = ph2d_gridmap::RoundOptions {
        pin_singularities: pin,
        // ⛔⛔ **O interruptor só pode DESLIGAR, e a 1.ª redacção lia-o ao contrário:**
        // ela punha `false` sempre que a env não estivesse posta, o que **sobrepunha o
        // default do produto** e fazia este instrumento medir o comportamento antigo com
        // ar de estar a medir o novo. ⚠️ *Um instrumento que não herda o default do
        // produto não mede o produto* — e o sintoma foi ler `14 bordo` no dia em que o
        // default já dava `10`.
        pin_lone_singularities: std::env::var("PH2D_PIN_LONE").as_deref() != Ok("0")
            && base_opts.pin_lone_singularities,
        welded_rounds: num("PH2D_G3_ROUNDS", base_opts.welded_rounds),
        sweeps: num("PH2D_G3_SWEEPS", base_opts.sweeps),
        ..base_opts
    };
    let (map, r) = if welded {
        ph2d_gridmap::round_welded(&mesh, &cut, &combed, h, opts, &singular)
    } else {
        ph2d_gridmap::round_to_integers(&mesh, &cut, &combed, h, opts, &singular)
    };
    println!(
        "  caminho: {}",
        if welded {
            "SOLDADO (eliminacao)"
        } else {
            "penalizado"
        }
    );
    println!("  modalidade das singularidades: {pin}");
    println!(
        "  ⭐⭐⭐ AMARRAS DOS ARCOS: ligadas={} | {} grupos entraram · ⛔ {} RECUSADOS \
         (⛔ {} por a classe ser DEPENDENTE do sistema dos fechos · {} por ela SER uma \
         incognita LIVRE dele · {} por a componente ja' estar pregada)",
        ph2d_gridmap::arcline_enabled(),
        r.tie_groups,
        r.tie_refused,
        r.tie_refused_why[0],
        r.tie_refused_why[1],
        r.tie_refused_why[2]
    );
    println!(
        "  ⭐⭐⭐ GANHO do denominador FINGIDO (H/H_fingida, 1.0 = estava certo): p50 {:.2}x max {:.2}x",
        r.tie_gain.0, r.tie_gain.1
    );
    println!(
        "  ⭐⭐ RAIZES de classe SIMPLES (a relax_class tambem as escrevia): {}",
        r.tie_plain_roots
    );
    println!(
        "  ⛔⛔⛔ EIXOS que a escada SALTOU por estarem amarrados (nunca viram inteiro): {}",
        r.tie_axes_skipped
    );
    println!(
        "  ⭐⭐⭐ TRANSLACOES fraccionarias no fim: {} LIVRES · {} \
DEPENDENTES (a substituicao recebeu uma livre torta) · {} ORFAS (ninguem as escreve)",
        r.frac_shift_free, r.frac_shift_dep, r.frac_shift_orphan
    );
    println!(
        "  ⭐⭐⭐ das LIVRES fraccionarias, {} tinham sido PREGADAS (=> alguem as mexeu depois)",
        r.frac_shift_free_pinned
    );
    println!(
        "  ⭐⭐⭐ ... e por EIXO: {} com os DOIS pregados · {} com UM so' · {} com NENHUM",
        r.frac_shift_free_pinned, r.frac_shift_free_half, r.frac_shift_free_loose
    );
    println!(
        "  ⭐⭐⭐ EQUACOES DE CICLO: {} com o valor FRACCIONARIO (pior {:.4}) · {} termos de \
entrada fraccionarios · pior |coef| {:.3}",
        r.arc_cycle_frac.0, r.arc_cycle_frac.2, r.arc_cycle_frac.1, r.arc_cycle_frac.3
    );
    println!(
        "  ⛔⛔⛔ DONOS de equacao de ciclo OBSOLETOS (o mapa tem valor de uma ronda antiga): {}",
        r.arc_cycle_stale
    );

    // ⭐⭐⭐ **O DESLOCAMENTO `δ` DE CADA MEMBRO AMARRADO, e quanto dele e' FRACCIONARIO.**
    //
    // ⚠️ Um membro amarrado vale `σ·raiz + δ`, e a escada gulosa **nao o prega** (o eixo
    // esta' congelado pela amarra). ⇒ ele so' cai em inteiro se o `δ` for inteiro — e o `δ`
    // sai de `e·off_A − e·off_B`, que e' linear nos shifts e **nao tem razao nenhuma para
    // o ser**. *Se esta coluna nao for zero, as translacoes deixam de ser inteiras por
    // CONSTRUCAO, e a extraccao recebe fraccionarias.*
    if ph2d_gridmap::arcline_enabled() {
        let (w_d, _) = ph2d_gridmap::weld(&cut, &combed);
        let (m_d, _) = ph2d_gridmap::solve_welded(
            &mesh,
            &cut,
            &combed,
            h,
            ph2d_gridmap::weld_solve_driver::ROUNDS,
        );
        let t_d = ph2d_gridmap::arcline::build_arc_ties(&cut, &w_d, &m_d);
        let mut fracs: Vec<f32> = Vec::new();
        let (mut membros, mut inteiros) = (0usize, 0usize);
        for g in 0..t_d.groups() {
            let Some((root, mem)) = t_d.group(g) else {
                continue;
            };
            for &x in mem {
                if x == root {
                    continue;
                }
                let (_, _, delta) = t_d.of(x);
                let f = (delta - delta.round()).abs();
                membros += 1;
                if f <= 1.0e-4 {
                    inteiros += 1;
                }
                fracs.push(f);
            }
        }
        fracs.sort_by(f32::total_cmp);
        let p50 = fracs.get(fracs.len() / 2).copied().unwrap_or(0.0);
        let mx = fracs.last().copied().unwrap_or(0.0);
        println!(
            "  ⭐⭐⭐ O DESLOCAMENTO δ dos membros amarrados: {membros} membros nao-raiz \
⇒ ⭐ {inteiros} com δ INTEIRO · ⛔ {} FRACCIONARIO (parte fracc. p50 {p50:.3} max {mx:.3})",
            membros - inteiros
        );
    }

    // ⭐⭐⭐ **O PREÇO POR GRUPO** — `PH2D_ARC_GROUP_SCAN=1`.
    //
    // ⚠️ *A pergunta que decide a forma da cura:* as dobras que a restrição paga estão
    // **espalhadas** por todos os arcos, ou **concentradas** em poucos? Concentrado ⇒ há
    // um subconjunto que compra quase todo o alinhamento por quase nada. **Medir antes de
    // construir** — e cada linha aqui é uma resolução contínua inteira, por isso a sonda
    // vive atrás de uma env.
    if std::env::var("PH2D_ARC_GROUP_SCAN").as_deref() == Ok("1") {
        let (w_scan, _) = ph2d_gridmap::weld(&cut, &combed);
        let (m_scan, _) = ph2d_gridmap::solve_welded(
            &mesh,
            &cut,
            &combed,
            h,
            ph2d_gridmap::weld_solve_driver::ROUNDS,
        );
        let all = ph2d_gridmap::arcline::build_arc_ties(&cut, &w_scan, &m_scan);
        let n = all.groups();
        println!("  ⭐⭐⭐ VARREDURA POR GRUPO ({n} grupos, uma resolucao continua cada):");
        let base = ph2d_gridmap::solve_welded_with(
            &mesh,
            &cut,
            &combed,
            h,
            ph2d_gridmap::weld_solve_driver::ROUNDS,
            None,
            None,
        )
        .1;
        println!(
            "    [nenhum] dobras {} (antes do arredondamento)",
            base.folded_after
        );
        for g in 0..n {
            let only = all.keep_groups(&[g]);
            let membros = only.group(0).map_or(0, |t| t.1.len());
            let rep = ph2d_gridmap::solve_welded_with(
                &mesh,
                &cut,
                &combed,
                h,
                ph2d_gridmap::weld_solve_driver::ROUNDS,
                Some(&only),
                None,
            )
            .1;
            println!(
                "    [grupo {g:>2}] {membros:>2} membros · entrou={} · dobras {} (delta {:+}) · nao-finitos {}",
                rep.tie_groups,
                rep.folded_after,
                rep.folded_after as i64 - base.folded_after as i64,
                rep.nonfinite
            );
        }
    }
    println!(
        "  ⛔⛔ NAO-FINITOS no mapa continuo: {} no fim · {} logo apos a 1a ronda",
        r.nonfinite.0, r.nonfinite.1
    );
    println!(
        "  ⛔⛔ ESTOUROU na ronda {} (0 = nunca), com movimento {:.3e}",
        r.nonfinite_round.0, r.nonfinite_round.1
    );
    println!(
        "  ⛔⛔ QUEM estourou: {}",
        [
            "classe",
            "amarra",
            "livre",
            "ciclo de arco",
            "nenhum (ja' vinha torto)"
        ][r.nonfinite_who.min(4)]
    );
    println!(
        "  ⛔⛔⛔ PREGOS com passo NAO-FINITO: {} | ⭐ EQUACOES DE CICLO (A3) que entraram: {}",
        r.nonfinite_pins, r.arc_cycles
    );

    // ⭐⭐⭐ **QUANTOS singulares o CORTE de facto duplica.** O caminho soldado deriva os
    // vértices singulares dos FECHOS do grafo de cópias — e um vértice que o corte não
    // duplicou não tem cópias, logo não tem fecho, logo **nunca é pregado num inteiro**.
    // ⚠️ O doc daquele passo afirma que as duas contagens batem *«8 para 8, 12 para 12»*;
    // esta coluna é o que testa a afirmação numa peça em que ela pode não bater.
    {
        let mut copias: std::collections::BTreeMap<u32, usize> = std::collections::BTreeMap::new();
        for origin in &cut.origin {
            for &g in origin {
                *copias.entry(g).or_default() += 1;
            }
        }
        let (mut duplicados, mut unicos, mut ausentes) = (0usize, 0usize, 0usize);
        for v in &singular {
            match copias.get(v) {
                None => ausentes += 1,
                Some(1) => unicos += 1,
                Some(_) => duplicados += 1,
            }
        }
        println!(
            "  ⭐⭐⭐ SINGULARES contra o CORTE: {} duplicados (tem fecho) · ⛔ {} com UMA \
             cópia só (nunca pregados) · {} ausentes — de {}",
            duplicados,
            unicos,
            ausentes,
            singular.len()
        );
    }
    println!(
        "  G3+G5 ({:.1} s): {} costuras de arvore + {} de CICLO + {} singularidades (de {}, ⛔ {} AUSENTES DO CORTE, ⭐ {} SOLTOS pregados, {} copias, {} ambiguas) ⇒ {} inteiros \
         | degrau1 {} degrau2 {} | {} visitas | passo pior {:.4} soma {:.3} | passou-as-costuras {}",
        t.elapsed().as_secs_f64(),
        r.tree_seams,
        r.cycle_seams,
        r.singular_pinned,
        singular.len(),
        r.singular_absent,
        r.singular_loose_pinned,
        r.singular_copies,
        r.ambiguous_seams,
        r.pinned,
        r.level1,
        r.level2,
        r.visits,
        r.worst_step,
        r.sum_step,
        r.switched_to_seams
    );
    println!(
        "  ⭐ distancia a inteiro DEPOIS: {:.3e}  (tem de ser 0) | costura p50 {:.4}→{:.4} max {:.4}→{:.4} | angulo p50 {:.1}°",
        r.shift_frac_max,
        r.seam_before.0,
        r.seam_after.0,
        r.seam_before.1,
        r.seam_after.1,
        r.solve.angle_p50
    );
    println!(
        "  ⭐⭐ DOBRAS: {} no continuo (endurecimento: {} passagens, {} ⇒ {}) · \
         ⭐⭐⭐ o ARREDONDAMENTO leva-as de {} para {}, e {} de {} pregos criaram alguma \
         · 2a tentativa: {} correram, ⭐ {} ganharam",
        r.folded_after,
        r.stiffen_passes,
        r.folded_before,
        r.folded_after,
        r.folded_before_rounding,
        r.folded_after_rounding,
        r.pins_that_folded,
        r.pinned,
        r.second_tries,
        r.second_tries_won
    );
    println!(
        "  REGUAS do mapa: alinhamento p50 {:.3} p95 {:.3} | angulo p50 {:.1}° p95 {:.1}° | ESCALA p50 {:.3} p95 {:.3} | {} triangulos, {} saltados, {} pares",
        r.solve.align_p50,
        r.solve.align_p95,
        r.solve.angle_p50,
        r.solve.angle_p95,
        r.solve.scale_p50,
        r.solve.scale_p95,
        r.solve.triangles,
        r.solve.skipped,
        r.solve.pairs
    );

    // ⭐⭐⭐ **O SALTO DA GRADE AO DAR UMA VOLTA** — a régua do espiral (§23.9).
    //
    // ⚠️ Ela corre **sobre o mapa final**: as translações são o que ela compõe, e o G5
    // move-as. *Medi-la no contínuo seria medir outro mapa.*
    {
        let al = ph2d_gridmap::measure_alignment(&cut, &combed, &map);
        println!(
            "  ⭐⭐⭐ ESPIRAL DA GRADE (holonomia de ciclo): {} ciclos planos ⇒ ⛔ {} ESPIRALAM (nenhuma familia fecha), \
             {} fecham numa familia (o tubo, normal), ⭐ {} fecham nas duas \
             | DERIVA p50 {:.2} p90 {:.2} max {:.2} soma {:.1} celulas (volta maior: {:.0}) \
             | {} ciclos que RODAM | ⛔ {} fraccionarios | {} costuras soltas",
            al.flat_cycles,
            al.spiral_cycles,
            al.one_family_cycles,
            al.closed_cycles,
            al.drift_p50,
            al.drift_p90,
            al.drift_max,
            al.drift_sum,
            al.span_max,
            al.turning_cycles,
            al.fractional,
            al.loose
        );
        // ⭐⭐⭐ A leitura que NÃO depende da árvore de expansão que a régua escolheu.
        println!(
            "  ⭐⭐⭐ RETICULADO das holonomias (nao depende da arvore): ordem {} | PERIODO da familia u: {} celulas · da familia v: {} — ⛔ 0 com ordem >= 1 = essa familia NAO PODE fechar em volta nenhuma",
            al.lattice_rank, al.u_period, al.v_period
        );
        // ⭐⭐⭐ Os ciclos que RODAM: ali o invariante é o PONTO FIXO — onde o cone está
        // na carta — e o denominador da inversa é `2`.
        println!(
            "  ⭐⭐⭐ CONES (ponto fixo dos {} ciclos que rodam): ⭐ {} num ponto INTEIRO · ⛔ {} a MEIA CELULA · distancia a' grade p50 {:.3} max {:.3}",
            al.turning_cycles, al.cone_on_lattice, al.cone_half, al.cone_frac_p50, al.cone_frac_max
        );
    }

    // ⭐⭐⭐ **O QUE O F4 EXIGIRIA, contra o que o mapa livre fez.** A rota da extracção
    // não o chama (§23.13); esta coluna mede o tamanho da discordância **antes** de
    // alguém construir a restrição. ⚠️ *Uma restrição que o mapa já satisfaz não muda
    // nada, e teria custado uma wave a descobri-lo.*
    match layout.to_layout(h) {
        Err(e) => println!("  ⛔ o layout nao passa a porta do F4: {e:?}"),
        Ok(l) => match ph2d_quantize::quantize_within(&l, ph2d_quantize::Budget::new(256, 512)) {
            Err(e) => println!("  ⛔ o F4 RECUSOU este layout: {e:?}"),
            Ok((q, _)) => {
                // ⭐⭐⭐ **ONDE O DESVIO ENTRA: antes ou depois do arredondamento?**
                //
                // A escada gulosa move cada cone no máximo **meia célula**; um arco tem
                // dois extremos ⇒ o G5 sozinho não pode desalinhar mais de **uma**. E o
                // desvio medido é `0,61`–`0,96`. ⚠️ *Os dois números são compatíveis, e
                // é exactamente por isso que a pergunta tem de ser medida em vez de
                // deduzida.*
                //
                // ⛔ Se o contínuo já desalinha, a cura não é na escada — é no G3.
                let (cont, _) =
                    ph2d_gridmap::solve_welded(&mesh, &cut, &combed, h, opts.welded_rounds);
                let ac = ph2d_gridmap::measure_arc_quantization(&cut, &cont, &q.arc);
                println!(
                    "  ⭐⭐⭐ ANTES do arredondamento (o G3 continuo): ⛔ {} de {} nao sao isolinhas \
                     (atravessam p50 {:.2} max {:.2}) | discordancia p50 {:.2} soma {:.0}",
                    ac.off_axis, ac.arcs, ac.across_p50, ac.across_max, ac.diff_p50, ac.diff_sum
                );
                let aq = ph2d_gridmap::measure_arc_quantization(&cut, &map, &q.arc);
                println!(
                    "  ⭐⭐⭐ O MAPA contra o F4: {} arcos ({} costuras sem arco) ⇒ ⭐ {} CONCORDAM ({:.0}%) \
                     | discordancia p50 {:.2} max {:.2} soma {:.0} arestas de quad \
                     | ⛔⛔ {} NAO SAO ISOLINHAS (atravessam p50 {:.2} max {:.2} celulas)",
                    aq.arcs,
                    aq.cut_only,
                    aq.agree,
                    100.0 * f64::from(aq.agree_fraction()),
                    aq.diff_p50,
                    aq.diff_max,
                    aq.diff_sum,
                    aq.off_axis,
                    aq.across_p50,
                    aq.across_max
                );
            }
        },
    }

    // ⭐⭐⭐ **O PORTÃO DA WAVE DOS ARCOS:** as equações «este arco é uma isolinha» são
    // consistentes entre si? Um conjunto de diferenças com sinal só se pode eliminar se
    // **toda volta fechar**. ⛔ *Descobrir um conflito depois de refazer o relaxador
    // custaria a wave inteira.*
    {
        let (w, _) = ph2d_gridmap::weld(&cut, &combed);
        // ⭐⭐⭐ **A PREMISSA DO A3, medida no mapa QUE A OBRA VAI USAR.**
        //
        // O A3 quer que as equações de arco que FECHAM CICLO possuam um escalar de
        // **translação** — é isso que as torna «uma condição sobre as translações» em vez
        // de uma condição solta. ⚠️ Uma equação sem termo de translação nenhum **não tem
        // pivô** dessa família, e o desenho do A3 falha nela.
        //
        // ⛔⛔ **E ela mede-se AQUI, sobre o mapa vivo desta corrida** — com
        // `PH2D_GRIDMAP_ARCLINE=1` esse mapa é o **restringido**. *O portão da §23.16 leu
        // a premissa no mapa LIVRE e disse `0` conflitos; com a restrição activa a esfera
        // dá `2`. Um portão que valida a premissa num ponto não a valida no ponto onde a
        // obra a vai usar.*
        {
            let eqs = ph2d_gridmap::arc_equations(&cut, &w, &map);
            let mut sem_shift = 0usize;
            let mut shifts: Vec<usize> = Vec::new();
            for e in &eqs {
                let n = e
                    .terms
                    .iter()
                    .filter(|t| matches!(t.0, ph2d_gridmap::Var::Shift(_)))
                    .count();
                if n == 0 {
                    sem_shift += 1;
                }
                shifts.push(n);
            }
            // ⭐⭐⭐ **O CRUZAMENTO que decide o desenho:** as equações que FECHAM CICLO
            // têm termo de translação? Só essas precisam de um pivô dessa família — as
            // que eliminam já possuem o escalar de uma classe. ⚠️ *Contar as duas
            // populações em separado não responde à pergunta; emparelhá-las responde.*
            {
                let ties = ph2d_gridmap::build_arc_ties(&cut, &w, &map);
                let (mut com, mut sem) = (0usize, 0usize);
                for &k in ties.cycle_equations() {
                    let n = eqs.get(k).map_or(0, |e: &ph2d_gridmap::ArcEquation| {
                        e.terms
                            .iter()
                            .filter(|t| matches!(t.0, ph2d_gridmap::Var::Shift(_)))
                            .count()
                    });
                    if n > 0 { com += 1 } else { sem += 1 }
                }
                println!(
                    "  ⭐⭐⭐ CICLO x TRANSLACAO: das {} equacoes que FECHAM CICLO, ⭐ {} tem pivo de translacao · ⛔ {} NAO TEM (o A3 falha nessas)",
                    ties.cycle_equations().len(),
                    com,
                    sem
                );
            }
            shifts.sort_unstable();
            println!(
                "  ⭐⭐⭐ PREMISSA DO A3 (neste mapa): {} equacoes | termos de TRANSLACAO por equacao: p50 {} max {} | ⛔ {} SEM translacao nenhuma (sem pivo dessa familia)",
                eqs.len(),
                shifts.get(shifts.len() / 2).copied().unwrap_or(0),
                shifts.last().copied().unwrap_or(0),
                sem_shift
            );
        }
        let al = ph2d_gridmap::measure_arc_lines(&cut, &w, &map);
        println!(
            "  ⭐⭐⭐ PORTAO DOS ARCOS: {} equacoes sobre {} escalares ⇒ ⭐ {} ELIMINAM · {} fecham ciclo \
             | ⛔⛔⛔ {} CONFLITOS DE SINAL | desacordo numerico p50 {:.3} max {:.3} celulas \
             | ⚠️ {} ambiguas (perto de 45°) | {} saltadas",
            al.arcs,
            al.scalars,
            al.eliminated,
            al.cycles,
            al.sign_conflicts,
            al.offset_p50,
            al.offset_max,
            al.ambiguous,
            al.skipped
        );
    }

    let (tris, uv) = ph2d_gridmap::corner_map(&cut, &map);
    // ⭐ **CONTROLO INDEPENDENTE da ponte**: contar as dobras aqui, sem passar pela
    // extraccao. Se os dois numeros discordarem, o defeito e' do `corner_map` (uma
    // ordem de cantos trocada inverteria metade das areas sem erro nenhum a acusar).
    let folded = uv
        .iter()
        .filter(|t| {
            let d = (t[1][0] - t[0][0]) * (t[2][1] - t[0][1])
                - (t[1][1] - t[0][1]) * (t[2][0] - t[0][0]);
            d < 0.0
        })
        .count();
    // ⭐ **ONDE as dobras moram** — o teste da hipotese de que tudo o que falha na peca
    // e' UM mecanismo, e que ele vive na ponta.
    let folded_where: Vec<u32> = uv
        .iter()
        .zip(&tris)
        .filter(|(t, _)| {
            let d = (t[1][0] - t[0][0]) * (t[2][1] - t[0][1])
                - (t[1][1] - t[0][1]) * (t[2][0] - t[0][0]);
            d < 0.0
        })
        .flat_map(|(_, v)| v.iter().copied())
        .collect();
    println!(
        "  ponte: {} triangulos, {} DOBRADOS no dominio ({:.1}%) · {}  <- controlo independente",
        tris.len(),
        folded,
        100.0 * folded as f64 / tris.len().max(1) as f64,
        radius_band(mesh.positions(), &folded_where)
    );
    println!(
        "  ⭐⭐ ONDE as SINGULARIDADES do campo moram: {}",
        radius_band(mesh.positions(), &singular)
    );
    let cm = ph2d_quadextract::CornerMap {
        pos: mesh.positions(),
        tris: &tris,
        uv: &uv,
    };
    match ph2d_quadextract::extract(&cm, None) {
        Ok((out, e)) => {
            // ⭐⭐⭐ **A EXPERIÊNCIA: e se a saída fosse POUSADA na escultura?**
            //
            // ⛔⛔ Medido 2026-08-26: a fidelidade contra o F1 é **`0,000`** — a saída vive
            // **sobre** a malha remalhada — e a fidelidade contra a ESCULTURA fica cravada em
            // `0,10 %` de p95 num intervalo de **16×** de densidade. ⇒ *pedir uma malha mais
            // fina não a aproxima nem um pouco da peça que o artista fez*, e o número de
            // vincos acima de 30° **sobe** (`93 → 143`), porque a grade fina resolve cada
            // faceta do F1. É a 3.ª queixa dele, com mecanismo.
            //
            // ⚠️ **É uma medição, não um produto** — `PH2D_SNAP_TO_SCULPT=1`. Mover vértices
            // pode dobrar quads, e por isso as réguas de FORMA saem para os dois.
            let out = if std::env::var("PH2D_SNAP_TO_SCULPT").as_deref() == Ok("1") {
                let b = raw.bounds();
                let d = [
                    b.max[0] - b.min[0],
                    b.max[1] - b.min[1],
                    b.max[2] - b.min[2],
                ];
                let seed = d[0].mul_add(d[0], d[1].mul_add(d[1], d[2] * d[2])).sqrt() * 0.05;
                let normals = out.normals().to_vec();
                let moved: Vec<[f32; 3]> = out
                    .positions()
                    .iter()
                    .enumerate()
                    .map(|(i, &p)| {
                        // ⚠️ `project_facing` e não `project_onto`: numa ponta fina o pé mais
                        // próximo pode estar do OUTRO lado da peça, e a face entre os dois
                        // vira uma lasca. A direcção é a normal do vértice da saída.
                        ph2d_remesh_iso::project_facing(&raw, p, seed, Some(normals[i]))
                    })
                    .collect();
                ph2d_mesh::Mesh::from_parts(moved, out.faces().to_vec()).unwrap_or(out)
            } else {
                out
            };

            // ⭐⭐⭐ **E SE A SAÍDA LEVASSE O ACABAMENTO QUE O IRMÃO DELA LEVA?**
            //
            // ⛔⛔ Medido 2026-08-26: a cadeia do `fill` corre `SMOOTHING_ROUNDS = 6` passos
            // de Laplaciano tangencial com reprojeção **desde sempre**, e a cadeia da
            // EXTRACÇÃO entrega a malha **crua**. *Dois caminhos para o mesmo produto, e só um
            // com acabamento.*
            //
            // ⚠️ **A lei é a da casa, não uma minha.** A 1.ª versão desta experiência usava o
            // Laplaciano INTEIRO e reprojecção COM direcção — e a segunda é uma recusa
            // **medida** do `finish.rs` (com direcção, as dobras foram de `1` para `10` e a
            // aresta máxima de `2,58×` para `5,85×`). *Uma experiência que reescreve a lei em
            // vez de a chamar mede outra coisa.*
            let out = {
                // ⚠️⚠️ **O DEFAULT É O DO PRODUTO, e não zero.** Em 2026-08-26 esta linha
                // já pôs uma sonda a medir o comportamento ANTIGO enquanto imprimia como se
                // fosse o novo (o `pin_lone_singularities` do `chain_info`). *Um instrumento
                // cujo default diverge do produto responde por outro programa.*
                let rounds: usize = std::env::var("PH2D_OUT_RELAX")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(ph2d_quadfill::SMOOTHING_ROUNDS);
                let mut m = out;
                ph2d_quadfill::smooth(&mut m, &raw, rounds);
                m
            };
            let shape = ph2d_quadfill::quad_shape(&out);
            println!(
                "  EXTRACCAO: residuo de translacao p50 {:.3e} max {:.3e} · ⭐ {} \
                 FRACCIONARIAS (⭐ {} lados < 1/100 de celula, {} < 1/10 · rotacao {:.3e}) | {} nos ({} vertice, {} aresta, {} face) | {} dobras",
                e.shift_residual_p50,
                e.shift_residual,
                e.shift_fractional,
                e.tiny_edges,
                e.short_edges,
                e.rot_residual,
                e.vertex_nodes + e.edge_nodes + e.face_nodes,
                e.vertex_nodes,
                e.edge_nodes,
                e.face_nodes,
                e.folded_faces
            );
            let mut cnt: std::collections::BTreeMap<(u32, u32), usize> =
                std::collections::BTreeMap::new();
            for f in out.faces() {
                let v = f.verts();
                for k in 0..v.len() {
                    let (a, b) = (v[k], v[(k + 1) % v.len()]);
                    *cnt.entry(if a < b { (a, b) } else { (b, a) }).or_default() += 1;
                }
            }
            println!(
                "  triangulos degenerados {} | celulas: {} fechadas {} abandonadas {} nao-fechadas | anel {:?} | cantos {:?} | arestas: {} bordo, {} nao-manifold",
                e.degenerate_faces,
                e.cells_closed,
                e.cells_abandoned,
                e.cells_unclosed,
                e.ring_len
                    .iter()
                    .enumerate()
                    .filter(|(_, n)| **n > 0)
                    .map(|(i, n)| (i, *n))
                    .collect::<Vec<_>>(),
                e.ring_distinct
                    .iter()
                    .enumerate()
                    .filter(|(_, n)| **n > 0)
                    .map(|(i, n)| (i, *n))
                    .collect::<Vec<_>>(),
                cnt.values().filter(|c| **c == 1).count(),
                cnt.values().filter(|c| **c >= 3).count()
            );
            // ⭐ **A peça sai em disco quando se pede**, para o `piece_report` a medir com
            // as réguas de POSIÇÃO que este instrumento não tem. *Duas perguntas — «a cadeia
            // fez o quê» e «a peça está como» — e o segundo instrumento precisa da peça.*
            if let Ok(path) = std::env::var("PH2D_CHAIN_DUMP") {
                let text = ph2d_mesh::write_obj(&[ph2d_mesh::ExportPiece {
                    mesh: &out,
                    name: Some("Piece"),
                    pose: ph2d_mesh::Pose::default(),
                }]);
                std::fs::write(&path, text).unwrap_or_else(|e| panic!("{path}: {e}"));
                println!("  (peca gravada em {path})");
            }
            println!(
                "  ⭐⭐ SANEAMENTO: {} arestas colapsadas + {} tardias · {} faces MORTAS · \
                 {} triangulos degenerados no dominio · ⛔⛔ {} TRANSICOES INEXACTAS \
                 (⛔ {} delas SEM APROXIMAR)",
                e.collapsed_edges,
                e.late_collapsed,
                e.dead_faces,
                e.degenerate_faces,
                e.inexact_transitions,
                e.far_fallbacks
            );
            println!(
                "  ⭐⭐⭐ ORFAS (o sintoma mais A MONTANTE de um furo): {} sem parceira ({} com NO' la' / ⭐ {} sobre uma ARESTA) + \
                 {} sem saida do triangulo ({} achatado / {} com a ORIGEM FORA / {} so' \
                 pelo lado de ENTRADA) = {} · raio {:.2}x (o p99 da peca e' {:.2}x) · \
                 ⭐ FALHA POR {:.3} CELULAS num triangulo de {:.3} · ⭐⭐ RESGATADAS pela face gemea: {} · num CANTO: {} · ⭐⭐ pelo LEQUE: {}",
                e.orphan_no_partner,
                e.orphan_no_partner_node_exists,
                e.orphan_no_partner_on_edge,
                e.orphan_no_exit,
                e.orphan_no_exit_flat,
                e.orphan_no_exit_o_outside,
                e.orphan_no_exit_entry_only,
                e.orphan,
                e.orphan_radius_p50,
                e.piece_radius_p99,
                e.orphan_miss_cells_p50,
                e.orphan_tri_cells_p50,
                e.orphan_rescued_across_edge,
                e.orphan_on_corner,
                e.orphan_rescued_in_fan
            );
            println!(
                "  ⛔⛔⛔ POR QUE o resgate pela gemea nao disparou: {} sem estar sobre \
aresta · {} sobre aresta SEM GEMEA (bordo) · {} com gemea mas SEM A CHAVE la' (destas, \
{} tem porta no mesmo ponto com OUTRA DIRECCAO) · {} eram a propria porta",
                e.rescue_why.0, e.rescue_why.1, e.rescue_why.2, e.rescue_why.3, e.rescue_why.4
            );
            println!(
                "  ⭐⭐⭐ QUAL CONVENCAO de direccao acharia a parceira: x.dir={} · oposta={} \
· com troca de sinal={} · oposta dessa={} (esta ultima e' a que o codigo usa)",
                e.rescue_would[0], e.rescue_would[1], e.rescue_would[2], e.rescue_would[3]
            );
            let f = e.rescue_by_fold;
            println!(
                "  ⭐⭐⭐ QUAL ACERTOU x QUEM ESTA' DOBRADO (d2 / oposta): nenhuma \
{}/{} · so' a gemea {}/{} · so' a face {}/{} · as DUAS {}/{} | ambiguas {}",
                f[0], f[1], f[2], f[3], f[4], f[5], f[6], f[7], e.rescue_ambiguous
            );
            println!(
                "  ⭐⭐⭐ PASSE MUTUO (cada lado nomeia o outro): {} pares ligados · {} candidatas SEM correspondencia",
                e.rescue_mutual.0, e.rescue_mutual.1
            );
            println!(
                "  ⭐⭐⭐ das SEM CHAVE, em que cardinal RELATIVO ao d2 ha' porta: +0={} +1={} +2={} +3={}",
                e.rescue_offset[0], e.rescue_offset[1], e.rescue_offset[2], e.rescue_offset[3]
            );
            println!(
                "  ⭐⭐⭐ SEM PORTA NENHUMA ali: {} · destas, gemea DEGENERADA {} · face de ca' degenerada {}",
                e.rescue_no_port.0, e.rescue_no_port.1, e.rescue_no_port.2
            );
            println!(
                "  ⭐⭐⭐ ... destas, {} caem num CANTO da gemea · {} com a gemea a ter portas NOUTROS pontos",
                e.rescue_no_port_where.0, e.rescue_no_port_where.1
            );
            // ⭐⭐⭐ **ONDE as celulas falharam** — a coluna que responde ao report do
            // artista (*«furos nas pontas»*), e que nenhuma regua desta linha tinha.
            println!(
                "  ⭐⭐ celulas COLAPSADAS (bigono/monogono) {} · triangulos {} | \
                 ONDE as falhas moram: raio {:.2}x (o p99 da peca e' {:.2}x)",
                e.degenerate_cells, e.triangles, e.cells_failed_radius_p50, e.node_radius_p99
            );
            println!(
                "  ⭐ SAIDA: {} verts, {} quads, X = {} | ordem limpa {:?} | orfas {} disputadas {} fugidas {}",
                out.vert_count(),
                e.quads,
                ph2d_quadextract::euler_characteristic(&out),
                e.port_step,
                e.orphan,
                e.contested,
                e.runaway
            );
            println!(
                "  ⭐⭐ FORMA: aspecto p50 {:.2} p99 {:.2} (>4x: {}) | ENVIESAMENTO p50 {:.1}° p99 {:.1}° (>60: {}) | area spread {:.2}",
                shape.aspect_p50,
                shape.aspect_p99,
                shape.aspect_over_4,
                shape.skew_p50,
                shape.skew_p99,
                shape.skew_over_60,
                shape.area_spread
            );
            println!(
                "     (a barra do oraculo: enviesamento p50 4,8-7,1 | aspecto p50 1,08-1,22 | >60: 0-4)"
            );
            fidelity(&raw, &mesh, &out);
        }
        Err(err) => println!("  EXTRACCAO recusada: {err}"),
    }
}
