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

    // ── ⛔⛔ FASE ZERO. Sem ela a mesma cadeia dá `10-12°`, e o defeito e' a entrada.
    ph2d_remesh_iso::remesh_isotropic(&mut mesh, ph2d_remesh_iso::ALPHA);
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
    let (field, _) = ph2d_crossfield::solve_miq(&dual);
    let singular: Vec<u32> = ph2d_crossfield::vertex_index(&mesh, &dual, &field)
        .into_iter()
        .enumerate()
        .filter(|(_, k)| *k != 0)
        .filter_map(|(v, _)| u32::try_from(v).ok())
        .collect();
    let layout = ph2d_trace::trace_patches(&mesh, &dual, &field);
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
             · ⛔ {} degenerados SOBREVIVERAM a' limpeza ({} dissolvidos em {} rondas, parou por {})",
            r.valence,
            fora,
            r.non_disk,
            layout.degenerate().len(),
            r.dissolved,
            r.rounds,
            match r.cleanup_stop {
                0 => "nada a fazer",
                1 => "⛔ a DISSOLUCAO nao removeu parede nenhuma",
                2 => "⛔ a ronda PIORAVA a topologia",
                _ => "⛔ o tecto de rondas",
            }
        );
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
                 pelo lado de ENTRADA) = {} · raio {:.2}x (a peca vai ate' {:.2}x) · \
                 ⭐ FALHA POR {:.3} CELULAS num triangulo de {:.3}",
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
                e.orphan_tri_cells_p50
            );
            // ⭐⭐⭐ **ONDE as celulas falharam** — a coluna que responde ao report do
            // artista (*«furos nas pontas»*), e que nenhuma regua desta linha tinha.
            println!(
                "  ⭐⭐ celulas COLAPSADAS (bigono/monogono) {} · triangulos {} | \
                 ONDE as falhas moram: raio {:.2}x (a peca vai ate' {:.2}x)",
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
        }
        Err(err) => println!("  EXTRACCAO recusada: {err}"),
    }
}
