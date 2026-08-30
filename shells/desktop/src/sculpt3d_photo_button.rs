//! ⭐⭐⭐ **AS SONDAS QUE CORREM A PORTA DO PRODUTO** — o botão, e a fase zero sozinha.
//!
//! Irmã de [`super::photo_probes`] por RESPONSABILIDADE: aquele módulo mede a FOTO, e
//! este corre o **caminho real** e mede o que sai dele.
//!
//! ⚠️⚠️ **A porta importa e já mordeu:** `Sculpt3dScene::quad_remesh` é o motor **LOCAL**
//! e `quad_remesh_global` é o de omissão — uma sonda que chame o primeiro mede a
//! ferramenta errada e o relatório fica plausível. `PH2D_PROBE_LOCAL=1` é o único caminho
//! para o local, e o default é o que o artista clica.

// ⚠️ **`local` ja' e' o nome de uma variavel aqui** (o MOTOR local, `PH2D_PROBE_LOCAL`),
// e sao coisas diferentes: um e' o motor de retopologia, o outro e' a regua de forma.
use super::{census, holes, islands, local as local_shape, relief_density, spiked_ball};

/// ⭐⭐⭐ **SONDA — A PEÇA DO ARTISTA PELO BOTÃO, e não por uma cópia da ordem dele.**
///
/// ⛔ **Ela existe porque as outras sondas desta linha NÃO são o botão.** O
/// [`ph2d_quadchain::quads_from_mesh`] remalha para o `target_edge` que recebe e corre a
/// cadeia **uma** vez; o botão remalha com [`ph2d_remesh_iso::ALPHA`] **fixo**, tira o alvo
/// do slider ([`ph2d_quadflow::edge_for_detail_with`]) e corre **duas ou três** tentativas
/// que uma medição escolhe. *Duas ordens diferentes com o mesmo nome dão dois números, e o
/// que o artista vê é o da que ele carrega.*
///
/// ⚠️ Ela chama [`crate::sculpt3d::Sculpt3dScene::quad_remesh`] — a **porta** —, então
/// nenhuma lei é reescrita aqui.
///
/// ```text
/// \
///   env PH2D_PIECE=/caminho/peca.obj PH2D_DETAIL=0.5 \
///   cargo test -p ph2d-host-desktop --release --bins \
///   the_artists_piece_through_the_button -- --ignored --nocapture
/// ```
#[test]
#[ignore = "sonda -- a peca do artista pelo BOTAO (PH2D_PIECE=<obj>)"]
fn the_artists_piece_through_the_button() {
    let Ok(path) = std::env::var("PH2D_PIECE") else {
        eprintln!("sem PH2D_PIECE -- nada a medir");
        return;
    };
    let Ok(gpu) = ph2d_gpu::GpuContext::new(ph2d_gpu::GpuContext::default_instance(), None) else {
        eprintln!("no GPU adapter on this machine -- nothing to assert");
        return;
    };
    // ⭐⭐⭐ **A FIXTURA SINTÉTICA `espinhos:<n>`** — ver [`spiked_ball`]. ⛔ Ela existe porque
    // *a fixtura só prova o que ela contém*: a peça que o artista fotografou em 28/08 é a
    // saída partida, e a **entrada** dele não está nesta árvore. Sem uma peça com pontas
    // finas, toda medição desta wave mediria uma bola.
    let piece = if let Some(n) = path.strip_prefix("espinhos:") {
        spiked_ball(
            n.parse().unwrap_or(6),
            std::env::var("PH2D_SPIKE_SIGMA")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(0.10f32),
        )
    } else {
        let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"));
        ph2d_mesh::import_obj(&text)
            .unwrap_or_else(|e| panic!("{path} nao e' um OBJ deste leitor: {e:?}"))
            .into_iter()
            .next()
            .unwrap_or_else(|| panic!("{path} nao tem peca dentro"))
            .mesh
    };
    let detail: f32 = std::env::var("PH2D_DETAIL")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0.5);
    let adapt: f32 = std::env::var("PH2D_ADAPT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0.0);

    // ⭐⭐ **O CONTROLO vem ANTES de acusar a cadeia** — se a entrada já é aberta ou
    // não-manifold, o que sai dela não foi a cadeia que abriu.
    census("ENTRADA", &piece);
    islands("ENTRADA", &piece);

    let mut scene = crate::sculpt3d::Sculpt3dScene::new(&gpu.device, piece.clone(), 1.0);
    scene.viewport = (900, 700);
    // ⭐⭐⭐ **A FASE ZERO, medida ao lado do alvo que o slider pede.** ⛔ *Uma cadeia cuja
    // malha de trabalho é mais grossa que o alvo não pode entregar a densidade pedida* — e
    // as duas grandezas nunca tinham sido impressas na mesma linha.
    {
        let reference = scene.mesh().clone();
        let target = ph2d_quadflow::edge_for_detail_with(
            &reference,
            detail,
            ph2d_quadflow::GLOBAL_FLOOR_IN_INPUT_EDGES,
        );
        let mut work = reference.clone();
        ph2d_remesh_iso::remesh_isotropic(&mut work, ph2d_remesh_iso::ALPHA);
        work.triangulate();
        eprintln!(
            "   CENA: {} verts {} faces | aresta media {:.5} | alvo do slider {:.5}",
            reference.vert_count(),
            reference.face_count(),
            ph2d_quadflow::mean_edge(&reference),
            target,
        );
        eprintln!(
            "   F1 (ALPHA={:.4}): {} verts {} faces | aresta media {:.5} | ALVO/F1 = {:.2}x",
            ph2d_remesh_iso::ALPHA,
            work.vert_count(),
            work.face_count(),
            ph2d_quadflow::mean_edge(&work),
            target / ph2d_quadflow::mean_edge(&work),
        );
        census("F1", &work);
        // ⭐⭐⭐ **A MESMA régua na FASE ZERO** — é o corte que diz se a ponta morre
        // no remalhador ou a jusante dele. *Sem os dois lados, «a cadeia amputa» é uma
        // acusação sem endereço.*
        super::tips("F1", &piece, &work);
        islands("F1", &work);
    }
    // ⭐⭐⭐ **QUAL das duas portas** — o painel tem um dropdown, e o `Global` é o de
    // omissão ([`ph2d_panel_sculpt3d::state::RetopoMode`]). ⛔ `PH2D_PROBE_LOCAL=1` corre a
    // outra. *Chamar a `quad_remesh` directamente é correr o motor LOCAL, que é o que o
    // artista tem quando o dropdown diz `Fast` — e não o que ele tem por omissão.*
    let local = std::env::var("PH2D_PROBE_LOCAL").as_deref() == Ok("1");
    eprintln!(
        "   MOTOR: {}",
        if local {
            "Fast (local)"
        } else {
            "Even Grid (global)"
        }
    );
    // ⭐⭐⭐ **QUANTOS CLIQUES** — o artista carrega outra vez quando o resultado parece
    // errado, e o alvo do 2.º clique sai da malha que o 1.º deixou. *Um botão cujo repetir
    // destrói a peça é um defeito de produto, não uma escolha do artista.*
    let presses: usize = std::env::var("PH2D_PRESSES")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(1);
    let mut r = None;
    for press in 1..=presses {
        let clock = std::time::Instant::now();
        let one = if local {
            scene.quad_remesh(detail, adapt)
        } else {
            scene.quad_remesh_global(detail, adapt)
        }
        .unwrap_or_else(|e| panic!("o botao recusou no clique {press}: {e:?}"));
        eprintln!(
            "   CLIQUE {press}: alvo {:.5} -> {} quads ({} nao-quads) em {:.0} ms",
            one.edge,
            one.quads,
            one.non_quads,
            clock.elapsed().as_secs_f32() * 1000.0,
        );
        census(&format!("  apos {press}"), scene.mesh());
        r = Some(one);
    }
    let r = r.expect("pelo menos um clique");
    let out = scene.mesh().clone();

    eprintln!(
        "BOTAO d={detail:.2} a={adapt:.2} -> {} quads ({} nao-quads) [campo {}]",
        r.quads,
        r.non_quads,
        if r.aligned { "alinhado" } else { "liso" }
    );
    eprintln!(
        "   alvo {:.5} | aresta mediana {:.2}x max {:.2}x | a maior atravessa {:.1} % da peca",
        r.edge,
        r.edge_median_ratio,
        r.edge_max_ratio,
        100.0 * r.edge_max_span
    );
    eprintln!(
        "   forma: aspecto p50 {:.2} p99 {:.2} | enviesamento p50 {:.1} p99 {:.1} | >60 {} | dobras {}",
        r.shape.aspect_p50,
        r.shape.aspect_p99,
        r.shape.skew_p50,
        r.shape.skew_p99,
        r.shape.skew_over_60,
        r.folded,
    );
    census("SAIDA", &out);
    islands("SAIDA", &out);
    local_shape("SAIDA", &out);
    super::orientation_and_density("SAIDA", &out);
    // ⭐⭐⭐ **UMA MEDIÇÃO POR PONTA** — a régua que a foto de 30/08 exigiu (seta VERDE
    // e seta VERMELHA na mesma peça). O ALCANCE é um extremo GLOBAL e não a vê.
    super::tips("SAIDA", &piece, &out);

    relief_density("SAIDA", &out);
    // ⭐ **`PH2D_DUMP=<ficheiro>` escreve a saída** — é o que permite medir a SECÇÃO do
    // espinho fora daqui, com a mesma régua que comparou as três malhas do artista.
    if let Ok(path) = std::env::var("PH2D_DUMP") {
        let text = ph2d_mesh::write_obj(&[ph2d_mesh::ExportPiece {
            name: Some("out"),
            mesh: &out,
            pose: ph2d_mesh::Pose::default(),
        }]);
        let _ = std::fs::write(&path, text);
        eprintln!("   DUMP: {path}");
    }
    // ⭐⭐⭐ **O ALCANCE da saída contra o da entrada** — a régua da AMPUTAÇÃO, e a única
    // que a peça do artista de 2026-08-29 move. ⛔ Nenhuma régua de topologia ou de forma
    // a vê: uma ponta comida sai fechada, com quads bonitos.
    {
        let reach = |m: &ph2d_mesh::Mesh| -> f32 {
            let pos = m.positions();
            let n = pos.len().max(1) as f32;
            let mut c = [0.0f32; 3];
            for q in pos {
                for k in 0..3 {
                    c[k] += q[k] / n;
                }
            }
            pos.iter().fold(0.0f32, |acc, q| {
                let d = [q[0] - c[0], q[1] - c[1], q[2] - c[2]];
                acc.max(d[0].mul_add(d[0], d[1].mul_add(d[1], d[2] * d[2])).sqrt())
            })
        };
        eprintln!(
            "   ALCANCE: entrada {:.4} -> saida {:.4} ({:+.1} %)",
            reach(&piece),
            reach(&out),
            100.0 * (reach(&out) / reach(&piece) - 1.0)
        );
    }
    holes("SAIDA", &out);
}

/// ⭐⭐⭐ **SONDA — a FASE ZERO preserva a topologia que recebe?**
///
/// ⛔⛔ **Reproduzido com a peça do artista em 2026-08-29:** ela entra `χ = 2`, fechada,
/// zero não-manifold — e a `remesh_isotropic` devolve **`χ = 6` com uma aresta não-manifold**.
/// A jusante o `ph2d-gridmap` entra em `index out of bounds` (`assembly.rs:193`), que é o
/// estouro que este repo tinha **sem endereço** desde 26/08.
///
/// ⚠️ **Ela é PURA — não precisa de GPU**, e por isso pode virar gate assim que a fixtura
/// sintética reproduzir o fenómeno.
///
/// ```text
/// \
///   cargo test -p ph2d-host-desktop --release --bins \
///   does_phase_zero_keep_the_topology -- --ignored --nocapture
/// ```
#[test]
#[ignore = "sonda -- a fase zero preserva a topologia?"]
fn does_phase_zero_keep_the_topology() {
    let mut cases: Vec<(String, ph2d_mesh::Mesh)> = Vec::new();
    for sigma in [0.30f32, 0.20, 0.14, 0.10, 0.07, 0.05] {
        cases.push((format!("espinhos sigma={sigma:.2}"), spiked_ball(6, sigma)));
    }
    if let Ok(path) = std::env::var("PH2D_PIECE")
        && let Ok(text) = std::fs::read_to_string(&path)
        && let Ok(pieces) = ph2d_mesh::import_obj(&text)
        && let Some(p) = pieces.into_iter().next()
    {
        cases.push((format!("ARTISTA {path}"), p.mesh));
    }
    for (name, piece) in cases {
        eprintln!("── {name}");
        census("  entrada", &piece);
        let mut work = piece.clone();
        ph2d_remesh_iso::remesh_isotropic(&mut work, ph2d_remesh_iso::ALPHA);
        work.triangulate();
        census("  fase zero", &work);
        islands("  fase zero", &work);
        // ⭐⭐⭐ **O RAIO MÁXIMO é a régua da AMPUTAÇÃO** — o que a ponta perde mede-se em
        // distância ao centro, e nenhuma régua de topologia a vê.
        let reach = |m: &ph2d_mesh::Mesh| -> f32 {
            let pos = m.positions();
            let n = pos.len().max(1) as f32;
            let mut c = [0.0f32; 3];
            for q in pos {
                for k in 0..3 {
                    c[k] += q[k] / n;
                }
            }
            pos.iter().fold(0.0f32, |acc, q| {
                let d = [q[0] - c[0], q[1] - c[1], q[2] - c[2]];
                acc.max(d[0].mul_add(d[0], d[1].mul_add(d[1], d[2] * d[2])).sqrt())
            })
        };
        eprintln!(
            "  ALCANCE: entrada {:.4} -> fase zero {:.4} ({:+.1} %)",
            reach(&piece),
            reach(&work),
            100.0 * (reach(&work) / reach(&piece) - 1.0)
        );
    }
}

/// ⭐⭐⭐ **A RÉGUA LOCAL sobre FICHEIROS, lado a lado** — o CONTROLO que decide se
/// «torcida» é defeito ou é a superfície.
///
/// ⛔⛔ **Sem ele o número não significa nada.** Um quad que cobre uma região
/// curva é **legitimamente** não-plano: numa ponta afiada a superfície vira
/// depressa, e a torção mede isso tão bem como mede um defeito. *A pergunta não é
/// «há torção?», é «há MAIS do que na malha de que ninguém se queixou?».*
///
/// ⚠️ **O controlo certo é a saída da ferramenta que o artista ACEITOU** — ele
/// mandou a peça dele passada pelo QRemeshify e disse *«preserva as pontas»*. Ela
/// é o denominador honesto; uma barra escolhida por mim não seria.
///
/// ```text
///   PH2D_MESHES=/a.obj,/b.obj \
///   cargo test -p ph2d-host-desktop --release --bins \
///   the_local_ruler_across_files -- --ignored --nocapture
/// ```
#[test]
#[ignore = "sonda -- a regua local sobre N ficheiros (PH2D_MESHES=a.obj,b.obj)"]
fn the_local_ruler_across_files() {
    let Ok(list) = std::env::var("PH2D_MESHES") else {
        eprintln!("sem PH2D_MESHES -- nada a medir");
        return;
    };
    for path in list.split(',').map(str::trim).filter(|s| !s.is_empty()) {
        let Ok(text) = std::fs::read_to_string(path) else {
            eprintln!("── {path}: NAO ABRE");
            continue;
        };
        let Ok(pieces) = ph2d_mesh::import_obj(&text) else {
            eprintln!("── {path}: NAO E' UM OBJ VALIDO");
            continue;
        };
        let Some(p) = pieces.into_iter().next() else {
            eprintln!("── {path}: sem pecas");
            continue;
        };
        let name = std::path::Path::new(path)
            .file_name()
            .map_or_else(|| path.to_string(), |s| s.to_string_lossy().to_string());
        eprintln!("── {name}");
        let mesh = p.mesh;
        // ⚠️ **A contagem de faces vai JUNTO**, sempre: uma malha mais fina tem
        // quads menores, e um quad menor cobre menos curvatura — comparar a torção
        // de duas densidades diferentes sem a dizer é a armadilha de 28/08.
        eprintln!("   {} verts {} faces", mesh.vert_count(), mesh.face_count());
        super::census("   ", &mesh);
        super::local("  ", &mesh);
        super::orientation_and_density("  ", &mesh);
    }
}
