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
    let mut piece = if let Some(n) = path.strip_prefix("espinhos:") {
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
    // ⭐⭐⭐ **`PH2D_RECENTER=1` recentra a peça PELA MESMA PORTA que o importador** (2026-09-03).
    // ⛔ A fixtura recentrada à mão (Python, `f64`, seis decimais) não é a peça que o botão do
    // programa vê: o importador chama [`ph2d_mesh::Mesh::recenter`] em `f32`, e a cadeia é
    // caótica nos últimos bits (plano, Parte VI) — medido, o dono exportou `20 658` quads da
    // MESMA escultura, nos MESMOS knobs, onde a sonda sobre a fixtura à mão dava `21 747`.
    // *Uma sonda que recentra de outra maneira mede outra realização do mesmo programa.*
    if std::env::var("PH2D_RECENTER").as_deref() == Ok("1") {
        let c = piece.recenter();
        eprintln!("   RECENTRADA pela porta do importador: centro da caixa era {c:?}");
    }
    // ⭐ **`PH2D_PROBE_SCALE=<s>` escala a peça — o «segundo sorteio»** (2026-09-03). A cadeia
    // é covariante à escala em tudo o que decide (alvo por área, `ALPHA × diagonal`), então
    // uma peça escalada é o MESMO problema com outro ruído de `f32` — e a mesma escultura,
    // nos mesmos knobs, deu *guarda / come / guarda* a ponta maior em três realizações.
    // ⛔ Nunca uma potência de dois: o produto por `0,5` é exacto e devolve a mesma realização.
    if let Some(s) = std::env::var("PH2D_PROBE_SCALE")
        .ok()
        .and_then(|v| v.parse::<f32>().ok())
        .filter(|s| s.is_finite() && *s > 0.0)
    {
        for p in piece.positions_mut() {
            for c in p.iter_mut() {
                *c *= s;
            }
        }
        piece.rebuild();
        eprintln!("   ESCALADA por {s} (segundo sorteio)");
    }
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
    super::piece_signature("ENTRADA", &piece);
    islands("ENTRADA", &piece);
    // ⭐⭐⭐ **A `ENTREGA` NOS TRÊS PONTOS DA CADEIA** — o corte que o §8-octodecies pediu.
    // A coluna já existia e só era impressa na **saída**, então ela dizia *que* a graduação
    // da ponta se perde e nunca *onde*. ⛔ Com um número só, «a fase zero é a culpada» é um
    // palpite com endereço; com os três (`ENTRADA` → `F1` → `SAIDA`) é uma medição.
    super::orientation_and_density("ENTRADA", &piece);

    let mut scene = crate::sculpt3d::Sculpt3dScene::new(&gpu.device, piece.clone(), 1.0);
    scene.viewport = (900, 700);
    // ⭐⭐⭐ **A FASE ZERO, medida ao lado do alvo que o slider pede.** ⛔ *Uma cadeia cuja
    // malha de trabalho é mais grossa que o alvo não pode entregar a densidade pedida* — e
    // as duas grandezas nunca tinham sido impressas na mesma linha.
    {
        let reference = scene.mesh().clone();
        // ⛔⛔ **`edge_for_detail_by_count` e NÃO `edge_for_detail_with`** — o produto trocou
        // de lei em 2026-08-28 (a faixa passou a ser CONTADA e ancorada na ÁREA, porque o
        // piso da outra é a aresta média da malha da cena e o botão deixava de ser
        // idempotente) e esta sonda ficou com a lei velha. *Medido 2026-08-31: ela imprimia
        // `0,03861` enquanto o botão usava `0,03961` — uma sonda que calcula o alvo por
        // outra lei mede outro programa.*
        let target = ph2d_quadflow::edge_for_detail_by_count(&reference, detail);
        // ⛔⛔⛔ **E a fase zero tem de ser A DO PRODUTO.** Este bloco chamava sempre o
        // `remesh_isotropic(ALPHA)`, logo com `PH2D_F1_TARGET=1` a linha `F1` saía
        // **idêntica** à do controlo e o relatório dizia que a env era inerte quando ela
        // estava a mudar a saída. *Um diagnóstico que não corre o caminho que o produto
        // corre acusa o sítio errado.* ⚠️ O `PH2D_ISO_ADAPT` já era honrado por acidente:
        // ele é lido **dentro** do `remesh_isotropic`.
        // ⭐⭐⭐ **É A FUNÇÃO DO PRODUTO, e não um espelho dela** (2026-09-03). ⛔ Aqui viviam
        // dez linhas que *copiavam* a escolha da fase zero, com um comentário a dizer que a
        // espelhavam — e a cópia envelheceu no dia em que a fase zero ganhou a **calota** dos
        // bicos: a sonda media a malha de trabalho de um programa que já não existia. *Uma lei
        // escrita em dois sítios ainda não é uma lei — só uma PORTA é.*
        let mut work =
            crate::sculpt3d::history::retopo_extract::target::phase_zero(&reference, target);
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
        // ⭐⭐⭐ **A GRADE DA CALOTA MEDIDA EM PASSOS DO ALVO** (2026-09-03) — a coluna que
        // decide a wave da calota, e que **nenhuma linha desta sonda tinha**: a `tips` acima
        // mede a malha contra a **mediana dela própria**, logo ela responde *«a ponta é mais
        // grossa que o corpo?»* e nunca *«cabem duas células de calota no bico?»*, que é a
        // pergunta de que o pólo `+1` depende (plano §101).
        let den = ph2d_quadfill::tip_density(&piece, &work, target);
        eprintln!(
            "   F1 CALOTA: pior {:.2} · p50 {:.2} passos do alvo | acima de {:.1}: {} de {}",
            den.worst,
            den.p50,
            ph2d_quadfill::TIP_DENSITY_MAX,
            den.over,
            den.tips,
        );
        islands("F1", &work);
        // ⭐⭐⭐ **O SEGUNDO dos três pontos.** ⚠️ A `ENTREGA` é uma razão entre medianas de
        // aresta-equivalente, logo ela é **adimensional** e comparável entre malhas de
        // densidades diferentes — é por isso que ela pode ser lida antes e depois de um
        // remalhador que muda a contagem de faces em ordens de grandeza.
        super::orientation_and_density("F1", &work);
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
    // ⭐⭐⭐ **A COBERTURA que o botão de facto reporta** — pelo relatório, e não recalculada
    // aqui: *uma sonda que refaz a conta mede a sonda, não o produto.*
    eprintln!(
        "   COBERTURA (casca): p50 {:.3} % | pior {:.3} % | amostras {} {}",
        100.0 * r.coverage_shell_p50,
        100.0 * r.coverage_shell_worst,
        r.coverage_samples,
        if r.coverage_samples == 0 {
            "⛔ NAO MEDIDO"
        } else {
            ""
        }
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
        // ⛔⛔ **Pela PORTA, e a porta mudou em 2026-08-31.** Esta sonda tinha a sua própria
        // cópia do alcance, com o centroide tirado da **média dos vértices** — e imprimia
        // `−6,5 %` na peça do dono enquanto as quatro pontas dela estavam intactas a
        // `−0,1 %`. *O centroide por vértice mede a AMOSTRAGEM: uma retopologia
        // redistribui vértices por construção, logo ele anda sempre.* Ver
        // [`ph2d_quadfill::tips`].
        let (a, b) = (ph2d_quadfill::reach(&piece), ph2d_quadfill::reach(&out));
        eprintln!(
            "   ALCANCE (centroide de AREA): entrada {a:.4} -> saida {b:.4} ({:+.1} %)",
            100.0 * (b / a - 1.0)
        );
        // ⭐⭐⭐ **O DESVIO LOCAL JUNTO DE CADA PONTA** — a régua que o report de 31/08
        // exigiu. ⛔ O suporte por ponta diz *até onde* o bico vai e **nada** sobre a
        // espessura com que lá chega: medido, a ponta partida da peça do dono lê `−5,3 %`
        // de suporte e fecha com um anel **4× mais gordo** que a escultura.
        // ⚠️ **A unidade é a aresta MEDIANA da saída** (2026-09-02), a mesma que o produto
        // passa — uma sonda que dividisse pelo alvo mediria outro número.
        let unit = ph2d_quadfill::median_edge(&out);
        let d = ph2d_quadfill::tip_deviation(&piece, &out, unit);
        eprintln!(
            "   DESVIO na ponta: p50 {:.2} p90 {:.2} max {:.2} quad(s) | {} de {} ponta(s) acima de {:.1} {}",
            d.p50,
            d.p90,
            d.max,
            d.over,
            d.tips,
            ph2d_quadfill::TIP_DEVIATION_MAX,
            if d.tips == 0 { "⛔ NAO MEDIDO" } else { "" }
        );
        // ⭐⭐⭐ **A AMPUTAÇÃO no ÁPICE, e a GRADE no bico — as duas réguas que concordam com a
        // foto** (2026-09-02): a retopologia que o dono aprovou lê `gap ≤ 0,19` e grade
        // `≤ 0,79` em todas as pontas; cada saída que ele reprovou falha pelo menos uma.
        let g = ph2d_quadfill::tip_density(&piece, &out, unit);
        eprintln!(
            "   AMPUTADAS: {} de {} ponta(s) com o bico a mais de {:.1} h da saida (pior gap {:.2} h) \
             | GRADE NA PONTA: pior {:.2} p50 {:.2} ({} de {} acima de {:.1}) {}",
            d.cut,
            d.tips,
            ph2d_quadfill::TIP_GAP_MAX,
            d.apex_max,
            g.worst,
            g.p50,
            g.over,
            g.tips,
            ph2d_quadfill::TIP_DENSITY_MAX,
            if g.tips == 0 { "⛔ NAO MEDIDO" } else { "" }
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
        // ⭐⭐⭐ **QUE PEÇA É ESTA** — ver [`super::piece_signature`]. ⛔ Sem esta linha a
        // tabela do §8-octodecies pôs quatro malhas numa coluna só e **três eram peças
        // diferentes**. *Uma régua que compara ficheiros deve dizer se eles são comparáveis.*
        super::piece_signature("  ", &mesh);
        super::census("   ", &mesh);
        super::local("  ", &mesh);
        super::orientation_and_density("  ", &mesh);
    }
}
