//! **O CORPUS DE BENCHMARK DO REMESHER** — a sonda que exporta as malhas de
//! teste para o projeto de bancada (ADR-0161, Passo 0 do briefing de pivô).
//!
//! ⚠️ **Ela vive aqui e não na bancada porque as fixturas de escultura são
//! `pub(super)` deste shell** — elas são desenhadas com os VERBOS do produto
//! (`Crease`, `SnakeHook`), não com relevo escrito à mão nos vértices, e
//! reproduzi-las fora daqui seria uma segunda resposta a *"como esta peça é
//! feita"*. A fase **F0** do plano move isto para o harness; até lá o corpus sai
//! por esta porta.
//!
//! ⚠️ **Nada aqui é código de produto**: é `#[cfg(test)]` inteiro, `#[ignore]`, e
//! escreve **fora** da árvore da engine.
//!
//! ```text
//! cargo test -p ph2d-host-desktop --release --bins dump_the_quad_remesh_corpus -- --ignored --nocapture
//! ```

use std::path::Path;

use ph2d_mesh::{ExportPiece, Mesh, Pose, shapes, write_obj};

/// Onde o corpus é escrito — **fora** do repositório da engine.
///
/// ⚠️ **O caminho é absoluto e está aqui de propósito.** O projeto de bancada é
/// GPL-isolado (ele invoca o oráculo `quadwild-bimdf`) e **não pode** virar um
/// diretório desta árvore; um caminho relativo tornaria fácil, um dia, alguém o
/// puxar para dentro. Ver ADR-0161 §Trilha B.
const CORPUS: &str = "/home/enio/Documentos/Projetos/ph2d-quadbench/corpus";

fn dump(dir: &Path, name: &str, mesh: &Mesh) {
    let obj = write_obj(&[ExportPiece {
        name: Some(name),
        mesh,
        pose: Pose::IDENTITY,
    }]);
    let path = dir.join(format!("{name}.obj"));
    std::fs::write(&path, obj).expect("o corpus e' escrito fora da arvore da engine");
    eprintln!(
        "[corpus] {name}.obj — {} vertices, {} faces, {} triangulos",
        mesh.vert_count(),
        mesh.faces().len(),
        mesh.triangle_count()
    );
}

/// **ESCREVE O CORPUS.** ⚠️ `#[ignore]`: escreve em disco, fora do repo.
#[test]
#[ignore = "sonda de preparacao -- escreve o corpus de benchmark fora da arvore (ADR-0161)"]
fn dump_the_quad_remesh_corpus() {
    let dir = Path::new(CORPUS);
    std::fs::create_dir_all(dir).expect("o diretorio da bancada existe");

    // ── PRIMITIVAS — o controle. Uma esfera lisa não nomeia problema nenhum, e
    // é exatamente por isso que ela entra: um remesher que erra AQUI está
    // quebrado, não meramente pior que o oráculo.
    dump(dir, "cube", &shapes::cube(1.0));
    dump(dir, "sphere_uv_96x144", &shapes::uv_sphere(96, 144, 1.0));
    dump(dir, "torus_64x32", &shapes::torus(64, 32, 1.0, 0.35));
    dump(dir, "sphere_sculpt_98k", &shapes::sculpt_sphere(1.0));

    // ── AS MALHAS COM PATOLOGIA CONHECIDA — as que a bancada existe para medir.
    dump(
        dir,
        "sphere_noisy",
        &shapes::uv_sphere_noisy(96, 144, 1.0, 0.02),
    );
    dump(
        dir,
        "sphere_shuffled",
        &shapes::uv_sphere_shuffled(96, 144, 1.0),
    );

    // ── OS SCULPTS REAIS — desenhados com os verbos do produto.
    //
    // ⚠️ **`hooked_sphere` é A malha do diagnóstico**: o doc dela diz *"a esfera
    // com um BICO puxado longe demais — o modelo que pede um remesh"*, e é a
    // peça das capturas de 2026-08-20 (a protuberância onde a grade colapsou em
    // triângulos e espiral). O gate de regressão do §9 do briefing é sobre ELA.
    dump(dir, "sculpt_hooked", &super::fixtures::hooked_sphere());
    dump(dir, "sculpt_wrinkled", &super::fixtures::wrinkled_sphere());
    dump(dir, "sculpt_ridged", &super::fixtures::ridged_sphere());
    // ⚠️ Esta chega QUEBRADA de propósito (faces arrancadas, beira crua): é o
    // caso que a sanitização do estágio 1 tem de resolver, e nenhum remesher da
    // família aceita entrada não-manifold.
    dump(
        dir,
        "sculpt_punctured",
        &super::fixtures::punctured_sphere(),
    );

    eprintln!("[corpus] escrito em {CORPUS}");
}

/// **RODA O REMESHER ATUAL SOBRE O CORPUS** e escreve a saída ao lado da do
/// oráculo — a linha "(a) remesher atual da PH2D" do §9 do briefing.
///
/// ⚠️ **É a BASELINE, não um gate.** Ela existe para que o plano tenha um número
/// honesto do ponto de partida, medido pela mesma régua que mede o oráculo
/// (`metrics.py` da bancada). ⚠️ `#[ignore]`: escreve em disco, fora do repo.
#[test]
#[ignore = "sonda de preparacao -- baseline do remesher atual sobre o corpus (ADR-0161)"]
fn dump_the_current_remesher_baseline() {
    // ⚠️ **Um diretório por CONFIGURAÇÃO, e a combinação tem nome próprio.** A
    // primeira versão só olhava o `ISO` e a corrida `ISO+MIQ` **sobrescreveu** a
    // do `ISO` sozinho — as duas colunas da tabela passaram a ser a mesma, e a
    // comparação teria mentido sem nada acusar.
    let iso = std::env::var("PH2D_BENCH_ISO").ok().as_deref() == Some("1");
    let miq = std::env::var("PH2D_BENCH_MIQ").ok().as_deref() == Some("1");
    let sub = match (iso, miq) {
        (true, true) => "ours_iso_miq",
        (true, false) => "ours_iso",
        (false, true) => "ours_miq",
        (false, false) => "ours",
    };
    let dir = Path::new(CORPUS).parent().expect("a bancada").join(sub);
    std::fs::create_dir_all(&dir).expect("o diretorio da bancada existe");
    let mut entries: Vec<std::path::PathBuf> = std::fs::read_dir(CORPUS)
        .expect("o corpus foi escrito antes")
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|x| x == "obj"))
        .collect();
    entries.sort();

    for path in entries {
        let name = path
            .file_stem()
            .expect("tem nome")
            .to_string_lossy()
            .to_string();
        let text = std::fs::read_to_string(&path).expect("le o obj");
        let pieces = ph2d_mesh::import_obj(&text).expect("o obj do corpus e' valido");
        let Some(p) = pieces.into_iter().next() else {
            continue;
        };
        let mesh = p.mesh;
        // ⚠️ **O ESTÁGIO F1, ligado por variável de ambiente.** Ele é o passe que
        // a medição do oráculo nomeou (a densidade da saída deixa de depender da
        // entrada), e a sonda tem de poder medir os DOIS lados — senão não há
        // como dizer se ele move a agulha.
        let mut mesh = mesh;
        let iso = std::env::var("PH2D_BENCH_ISO").ok().as_deref() == Some("1");
        if iso {
            let r = ph2d_remesh_iso::remesh_isotropic(&mut mesh, ph2d_remesh_iso::ALPHA);
            eprintln!(
                "[baseline]   F1: {} -> {} v em {} rodadas",
                r.verts_before, r.verts_after, r.rounds
            );
        }
        // ⚠️ Os MESMOS defaults do painel — a baseline tem de ser o que o artista
        // de facto obtém, não um ponto escolhido para favorecer o número.
        let t = std::time::Instant::now();
        let edge = ph2d_quadflow::edge_for_detail(&mesh, 0.5);
        let scale = ph2d_quadflow::ScaleField::adaptive(&mesh, edge, 0.0);
        // ⚠️ **O CAMPO DO F2, injetado no lugar do local.** É assim que o F2 se
        // mede antes de o F5 (quadrangulação por patch) existir: o campo novo
        // entra, o extrator antigo fica, e a diferença é do campo. ⚠️ A conversão
        // para por-vértice PERDE os saltos de período — está declarado na
        // `to_vertex_dirs`, e é por isso que este número é um **piso** do que o
        // F2 vale, não o valor dele.
        let miq = std::env::var("PH2D_BENCH_MIQ").ok().as_deref() == Some("1");
        let (orient, pos) = if miq {
            mesh.triangulate();
            let dual = ph2d_crossfield::Dual::build(&mesh);
            let (field, r) = ph2d_crossfield::solve_miq(&dual);
            let (sing, sum) = ph2d_crossfield::singularities(&mesh, &dual, &field);
            eprintln!(
                "[baseline]   F2: {sing} singularidades (soma {sum}), {} resolucoes, {} inteiros",
                r.solves, r.free_integers
            );
            let dirs = ph2d_crossfield::to_vertex_dirs(&mesh, &dual, &field);
            let orient = ph2d_quadflow::orientation_from(dirs);
            let pos = ph2d_quadflow::solve_position(&mesh, &orient, &scale, 32);
            (orient, pos)
        } else {
            ph2d_quadflow::solve_fields(&mesh, &scale)
        };
        match ph2d_quadflow::extract(&mesh, &orient, &pos, &scale) {
            Ok(q) => {
                let ms = t.elapsed().as_secs_f64() * 1000.0;
                let obj = write_obj(&[ExportPiece {
                    name: Some(&name),
                    mesh: &q.mesh,
                    pose: Pose::IDENTITY,
                }]);
                std::fs::write(dir.join(format!("{name}.obj")), obj).expect("escreve");
                eprintln!(
                    "[baseline] {name}: {} v, {} quads, {} nao-quads, {} buraco(s), {ms:.0} ms",
                    q.mesh.vert_count(),
                    q.quads,
                    q.non_quads,
                    q.holes
                );
            }
            Err(e) => eprintln!("[baseline] {name}: a extracao recusou ({e})"),
        }
    }
    eprintln!("[baseline] escrito em {}", dir.display());
}
