//! **A FORMA DE CADA QUAD** — a régua que faltava, e que a foto de 2026-08-22
//! («péssimo») pede.
//!
//! ⛔⛔ **Todas as réguas geométricas desta linha medem UM extremo.**
//! [`ph2d_quadfill::FillReport::edge_max`] é a **aresta mais longa da malha
//! inteira**; `edge_median` é a **mediana de todas as arestas**. Nenhuma das duas
//! olha para um quad de cada vez — e a foto mostra justamente um defeito
//! **por-face**: quads esmagados, finos numa direcção e compridos na outra, em
//! faixas.
//!
//! ⚠️ **Um quad de `0,02 × 0,30` não move nenhuma das duas.** A aresta longa dele
//! (`0,30`) está muito abaixo da maior da malha, e a curta (`0,02`) afunda-se na
//! mediana de dezenas de milhares. *Duas grandezas globais não somam a uma
//! grandeza local* — e foi por isso que a malha de 2026-08-22 passou em `edge_max
//! ≤ 20 %` e o artista escreveu «péssimo» a olhar para ela.
//!
//! ⭐ **As três grandezas por-face, e o que cada uma vê:**
//!
//! | grandeza | o que é | o que ela apanha na foto |
//! |---|---|---|
//! | **aspecto** | aresta mais longa ÷ mais curta **do mesmo quad** | a faixa de quads esmagados |
//! | **enviesamento** | maior desvio de 90° nos quatro cantos | o losango, que tem aspecto `1` e é péssimo |
//! | **espalhamento de área** | área p99 ÷ área p50 | a orelha grossa ao lado da calota fina |
//!
//! ⚠️ **As três são precisas juntas.** Um quad pode ter aspecto `1,0` e ser um
//! losango de 20° (aspecto é cego ao ângulo); pode ter os quatro cantos a 90° e
//! ser um rectângulo `1 × 10` (o enviesamento é cego à razão); e uma malha pode
//! ter as duas perfeitas e ainda assim pôr metade dos quads numa ponta da peça.

use ph2d_mesh::Mesh;
use ph2d_quadfill::QuadShape;

/// ⭐⭐ **A RÉGUA MORA NO PRODUTO, não nesta sonda** — ver
/// [`ph2d_quadfill::quad_shape`].
///
/// ⛔ **Ela nasceu aqui e MUDOU-SE no mesmo dia**, de propósito: uma régua que só
/// existe numa sonda `#[ignore]` é uma régua que o relatório do botão não tem, e
/// foi exactamente por o relatório não a ter que a malha de 2026-08-22 passou em
/// tudo e o artista escreveu «péssimo». *O sítio de uma régua é o caminho que ela
/// julga* ([[feedback_a_rule_only_exists_if_it_is_on_the_path_of_who_executes_it]]).
///
/// O que fica aqui é só a **apresentação** — a linha legível que as sondas
/// imprimem.
fn measure(mesh: &Mesh) -> Shown {
    Shown(ph2d_quadfill::quad_shape(mesh), mesh.faces().len())
}

/// A forma, mais a contagem de faces, formatadas numa linha.
struct Shown(QuadShape, usize);

fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn len(d: [f32; 3]) -> f32 {
    d[0].mul_add(d[0], d[1].mul_add(d[1], d[2] * d[2])).sqrt()
}

fn pct(sorted: &[f32], p: f32) -> f32 {
    if sorted.is_empty() {
        return 0.0;
    }
    let i = ((sorted.len() - 1) as f32 * p).round() as usize;
    sorted[i.min(sorted.len() - 1)]
}

/// ⭐⭐⭐ **A BARRA VEM DO ORÁCULO, não de mim.** Mede a forma dos quads que o
/// binário de referência entrega, peça a peça — é contra estes números que os
/// nossos se leem.
///
/// ⛔ **Sem este lado, qualquer barra que eu escrevesse seria um palpite.**
/// «aspecto ≤ 4» soa razoável e não é medição nenhuma; o que interessa é se o
/// estado da arte entrega `2` ou `40`, porque isso decide se a nossa malha está
/// *pior que a referência* ou *tão boa quanto ela e o defeito é outro*.
///
/// ⚠️ Sai em silêncio quando a bancada não está nesta máquina. ⛔ *Skip gracioso
/// não é verde* — isto é sonda, não gate.
///
/// ```text
/// \
///   cargo test -p ph2d-host-desktop --release --bins \
///   what_shape_are_the_oracles_quads -- --ignored --nocapture
/// ```
#[test]
#[ignore = "sonda -- le a bancada GPL-isolada fora da arvore"]
fn what_shape_are_the_oracles_quads() {
    const BENCH: &str = "/home/enio/Documentos/Projetos/ph2d-quadbench/ref";
    let root = std::path::Path::new(BENCH);
    let Ok(dirs) = std::fs::read_dir(root) else {
        eprintln!("[oraculo] a bancada nao esta nesta maquina — nada a medir");
        return;
    };
    let mut names: Vec<String> = dirs
        .filter_map(|e| Some(e.ok()?.file_name().to_string_lossy().into_owned()))
        .collect();
    names.sort();
    for piece in names {
        // ⭐⭐⭐ **AS DUAS SAÍDAS DELE, lado a lado** — e a diferença entre elas é a
        // pergunta mais barata desta investigação.
        //
        // O oráculo grava a malha **antes** do alisamento final e **depois**. Se as
        // duas medirem igual, a qualidade está no LAYOUT e um alisador nosso não a
        // compra; se a crua for má e a alisada boa, ⭐ **o alisador é a cura** — e um
        // alisador de quads com projecção de volta à superfície é coisa que se
        // escreve, sem tocar em código dele.
        for (rotulo, sufixo) in [
            ("cru   ", "_rem_p0_123_quadrangulation.obj"),
            ("ALISADO", "_rem_p0_123_quadrangulation_smooth.obj"),
        ] {
            let path = root.join(&piece).join(format!("{piece}{sufixo}"));
            let Ok(text) = std::fs::read_to_string(&path) else {
                continue;
            };
            let Some(piece_mesh) = ph2d_mesh::import_obj(&text).ok().and_then(|mut v| v.pop())
            else {
                continue;
            };
            eprintln!("  {piece:<18} {rotulo} {}", measure(&piece_mesh.mesh));
        }
    }
}

/// ⭐⭐⭐ **A FORMA DOS NOSSOS QUADS**, nas três fixturas e nos três níveis do
/// slider — a régua que a foto de 2026-08-22 pede e que não existia.
///
/// ⛔ **O contexto:** nesse dia a `edge_max` da orelha caiu de `57 %` da peça para
/// `5,5 %`, o relatório ficou verde em toda a coluna, e a foto seguinte veio com a
/// palavra **«péssimo»**. *Uma grandeza global melhorou e o artista viu a mesma
/// doença* — porque a doença é por-face e nada aqui a media.
///
/// ⚠️ **A sonda imprime também o CONTROLO da entrada.** A malha isotrópica que o F1
/// entrega tem a forma que tem; se o aspecto já estiver lá, nada a jusante o criou.
///
/// ```text
/// \
///   cargo test -p ph2d-host-desktop --release --bins \
///   what_shape_are_our_quads -- --ignored --nocapture
/// ```
#[test]
#[ignore = "sonda -- a forma por-face da nossa saida"]
fn what_shape_are_our_quads() {
    for (name, reference) in [
        ("ORELHA", crate::sculpt3d::fixtures::eared_sphere()),
        ("GANCHO", crate::sculpt3d::fixtures::hooked_sphere()),
        ("ENRUGADA", crate::sculpt3d::fixtures::wrinkled_sphere()),
    ] {
        eprintln!("── {name} ──");
        let diag = {
            let b = reference.bounds();
            let d = [
                b.max[0] - b.min[0],
                b.max[1] - b.min[1],
                b.max[2] - b.min[2],
            ];
            d[0].mul_add(d[0], d[1].mul_add(d[1], d[2] * d[2])).sqrt()
        };
        let mut work = reference.clone();
        ph2d_remesh_iso::remesh_isotropic(&mut work, ph2d_remesh_iso::ALPHA);
        work.triangulate();
        eprintln!("  CONTROLO (o F1, triangulos)  {}", measure(&work));
        let dual = ph2d_crossfield::Dual::build(&work);
        let (field, _) = ph2d_crossfield::solve_miq(&dual);
        let layout = ph2d_trace::trace_patches(&work, &dual, &field);
        for detail in [0.25f32, 0.5, 1.0] {
            let target = ph2d_quadflow::edge_for_detail_with(
                &reference,
                detail,
                ph2d_quadflow::GLOBAL_FLOOR_IN_INPUT_EDGES,
            );
            let Ok(spec) = layout.to_layout(target) else {
                eprintln!("  d={detail:.2} | o layout RECUSOU");
                continue;
            };
            let Ok((quant, _)) =
                ph2d_quantize::quantize_within(&spec, ph2d_quantize::Budget::new(256, 512))
            else {
                eprintln!("  d={detail:.2} | a quantizacao RECUSOU");
                continue;
            };
            designed_step(&layout, &quant, target);
            // ⭐⭐⭐ **A VARREDURA que escolhe o [`ph2d_quadfill::SQUARE_ROUNDS`].**
            //
            // ⚠️ **As quatro colunas da direita são o PREÇO**, e estão aqui porque
            // uma relaxação que só olhasse para o ângulo poderia comprá-lo com
            // qualquer das outras: `dobras` (a face virada ao contrário), `aresta`
            // (alguma coisa esticou), `detalhe` (a forma escorreu para longe da
            // escultura) e o tempo. *Uma cura sem a coluna do preço é meia medição.*
            for square in [0usize, 2, 4, 8, 16] {
                let t0 = std::time::Instant::now();
                let Ok((out, r)) = ph2d_quadfill::fill_with(
                    &work,
                    &reference,
                    &layout,
                    &quant,
                    ph2d_quadfill::SMOOTHING_ROUNDS,
                    square,
                ) else {
                    eprintln!("  d={detail:.2} q={square:<2} | a montagem RECUSOU");
                    continue;
                };
                let ms = t0.elapsed().as_secs_f32() * 1000.0;
                let (_, p95) = ph2d_quadfill::detail_lost(&reference, &out);
                eprintln!(
                    "  d={detail:.2} q={square:<2} {} | dobras {:>5} aresta {:>5.1}% detalhe {:>6.3}% {ms:>7.0}ms",
                    measure(&out),
                    r.folded_local,
                    100.0 * r.edge_max / diag,
                    100.0 * p95,
                );
                if square == 0 && detail > 0.9 {
                    worst_faces(&out, &layout, &quant);
                }
            }
        }
    }
}

/// ⭐⭐⭐ **DE QUE FASE NASCE O ESMAGAMENTO** — a pergunta que separa o **F4** (a
/// quantização escolheu mal quantos segmentos cada lado leva) do **F5** (a
/// construção pôs os pontos mal dentro de um orçamento correcto).
///
/// ⛔ **É a pergunta que não se pode responder olhando para a malha final**, porque
/// nela as duas fases já se misturaram. Aqui mede-se o **PASSO DESENHADO** de cada
/// lado — `comprimento ÷ segmentos` — que é uma grandeza puramente do F4 e existe
/// antes de qualquer ponto ser colocado.
///
/// ⚠️ **A leitura:** se o passo desenhado já vier torto, um quad esmagado é
/// obrigatório e nenhuma mudança na construção o pode curar — ele foi **encomendado
/// assim**. Se o passo vier certo e a malha sair torta, a dívida é do F5.
///
/// ⭐ **O denominador é o alvo**, então `1,00` é um lado perfeito. ⚠️ **O que
/// interessa não é o desvio de cada lado — é o desvio ENTRE os lados do mesmo
/// patch**: dois lados a `0,7` fazem um patch fino mas de quads quadrados; um a
/// `0,7` com o vizinho a `1,4` faz quads `2:1` em todo o interior.
fn designed_step(
    layout: &ph2d_trace::PatchLayout,
    quant: &ph2d_quantize::Quantization,
    target: f32,
) {
    let mut ratios: Vec<f32> = Vec::new();
    let mut all: Vec<f32> = Vec::new();
    let mut starved = 0usize;
    for (pp, sides) in layout.side_arcs.iter().enumerate() {
        let step: Vec<f32> = sides
            .iter()
            .map(|s| {
                let l: f32 = s.iter().map(|&(a, _)| layout.arc_length[a as usize]).sum();
                let n: u32 = s.iter().map(|&(a, _)| quant.arc[a as usize]).sum();
                l / f32::from(u16::try_from(n.max(1)).unwrap_or(u16::MAX))
            })
            .collect();
        for s in &step {
            all.push(s / target);
        }
        // ⚠️ **Num leque de `n` lados, os lados VIZINHOS é que fazem a célula** — o
        // sector `j` é uma grade `e_{j+1} × e_j`. Comparar o lado 0 com o 2 num
        // pentágono não descreve célula nenhuma.
        let n = step.len();
        for j in 0..n {
            let (a, b) = (step[j], step[(j + 1) % n]);
            ratios.push(a.max(b) / b.min(a).max(1.0e-9));
        }
        // Um lado que pediu muito mais do que recebeu: o piso de `1` segmento a
        // morder um lado que geometricamente queria dez.
        for (i, s) in sides.iter().enumerate() {
            let l: f32 = s.iter().map(|&(a, _)| layout.arc_length[a as usize]).sum();
            let n: u32 = s.iter().map(|&(a, _)| quant.arc[a as usize]).sum();
            if l / target > 2.0 * f32::from(u16::try_from(n.max(1)).unwrap_or(u16::MAX)) {
                starved += 1;
                if starved <= 6 {
                    eprintln!(
                        "        patch {pp:<3} lado {i}: comprimento {l:.3} queria {:.1} segmentos e recebeu {n}",
                        l / target
                    );
                }
            }
        }
    }
    ratios.sort_by(f32::total_cmp);
    all.sort_by(f32::total_cmp);
    eprintln!(
        "      PASSO DESENHADO: p05 {:.2} p50 {:.2} p95 {:.2} do alvo \
         | ⭐razao entre lados VIZINHOS p50 {:.2} p95 {:.2} max {:.1} \
         | {starved} lados esfomeados",
        pct(&all, 0.05),
        pct(&all, 0.50),
        pct(&all, 0.95),
        pct(&ratios, 0.50),
        pct(&ratios, 0.95),
        ratios.last().copied().unwrap_or(0.0),
    );
}

/// Os dez piores quads: onde estão, e a que patch pertencem.
fn worst_faces(out: &Mesh, layout: &ph2d_trace::PatchLayout, quant: &ph2d_quantize::Quantization) {
    let pos = out.positions();
    let mut rank: Vec<(f32, usize, [f32; 3])> = Vec::new();
    for (fi, f) in out.faces().iter().enumerate() {
        let v = f.verts();
        let n = v.len();
        if n < 3 {
            continue;
        }
        let p: Vec<[f32; 3]> = v.iter().map(|&i| pos[i as usize]).collect();
        let (mut lo, mut hi) = (f32::MAX, 0.0f32);
        for k in 0..n {
            let e = len(sub(p[(k + 1) % n], p[k]));
            lo = lo.min(e);
            hi = hi.max(e);
        }
        let mut c = [0.0f32; 3];
        for q in &p {
            for k in 0..3 {
                c[k] += q[k] / n as f32;
            }
        }
        rank.push((hi / lo.max(1.0e-12), fi, c));
    }
    rank.sort_by(|a, b| b.0.total_cmp(&a.0));
    for (a, fi, c) in rank.iter().take(10) {
        eprintln!(
            "      aspecto {a:>8.1}  face {fi:<7} em ({:.2}, {:.2}, {:.2})",
            c[0], c[1], c[2]
        );
    }
    // ⭐ Quantos patches e como o orçamento de segmentos ficou distribuído — o
    // número que diz se a doença é do TRAÇADO (patches maus) ou da QUANTIZAÇÃO.
    let mut thin = 0usize;
    for (pp, e) in quant.corners.iter().enumerate() {
        let _ = pp;
        if e.iter().copied().min().unwrap_or(9) <= 1 {
            thin += 1;
        }
    }
    eprintln!(
        "      {thin} de {} patches com um raio de leque a 1 (o sector de UMA celula)",
        layout.side_arcs.len()
    );
}

impl std::fmt::Display for Shown {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = &self.0;
        write!(
            f,
            "{:>6} faces | aspecto p50 {:.2} p99 {:>6.1} max {:>7.1} (>4x: {:>5}) \
             | enviesamento p50 {:>4.0}° p99 {:>4.0}° max {:>4.0}° (>60°: {:>5}) \
             | area p99/p50 {:>6.1}",
            self.1,
            s.aspect_p50,
            s.aspect_p99,
            s.aspect_max,
            s.aspect_over_4,
            s.skew_p50,
            s.skew_p99,
            s.skew_max,
            s.skew_over_60,
            s.area_spread,
        )
    }
}

/// ⭐⭐⭐ **A RÉGUA ESTÁ NO CAMINHO DO BOTÃO** — o tripwire desta sessão inteira.
///
/// ⛔⛔ **É a lição de 2026-08-22, escrita como asserção.** Nesse dia a malha da
/// orelha passou em **todas** as réguas desta linha — `100 %` de quads, casca
/// fechada, `χ` exacta, densidade no alvo, `edge_max` a `5,5 %` da peça depois de
/// ter estado a `57 %` — e a foto seguinte veio com a palavra **«péssimo»**. *A
/// régua que via o defeito não existia, e depois de existir passou uma hora a viver
/// só dentro de uma sonda `#[ignore]`.*
///
/// ⚠️ **Este gate não julga a malha: julga o RELATÓRIO.** Ele afirma duas coisas
/// que nenhuma outra asserção do repo afirma — que a forma por-face é **medida** no
/// caminho do produto, e que ela **chega à linha** que o artista lê. *Uma régua fora
/// do caminho de quem a executa não existe*
/// ([[feedback_a_rule_only_exists_if_it_is_on_the_path_of_who_executes_it]]).
#[test]
fn the_report_carries_the_shape_of_every_quad() {
    let reference = crate::sculpt3d::fixtures::wrinkled_sphere();
    let mut work = reference.clone();
    ph2d_remesh_iso::remesh_isotropic(&mut work, ph2d_remesh_iso::ALPHA);
    work.triangulate();
    let target = ph2d_quadflow::edge_for_detail_with(
        &reference,
        0.25,
        ph2d_quadflow::GLOBAL_FLOOR_IN_INPUT_EDGES,
    );
    let dual = ph2d_crossfield::Dual::build(&work);
    let (field, _) = ph2d_crossfield::solve_miq(&dual);
    let layout = ph2d_trace::trace_patches(&work, &dual, &field);
    let spec = layout.to_layout(target).expect("o layout fecha");
    let (quant, _) = ph2d_quantize::quantize_within(&spec, ph2d_quantize::Budget::new(256, 512))
        .expect("quantiza");
    let (_, r) = ph2d_quadfill::fill(
        &work,
        &reference,
        &layout,
        &quant,
        ph2d_quadfill::SMOOTHING_ROUNDS,
    )
    .expect("monta");
    assert!(
        r.shape.skew_p50 > 0.0 && r.shape.aspect_p50 >= 1.0,
        "o relatorio da montagem nao mediu a forma: {:?}",
        r.shape
    );
    // ⭐ E ela chega à linha que o artista lê.
    let line = crate::sculpt3d::history::remesh::QuadRemeshReport {
        verts: 0,
        quads: r.quads,
        non_quads: r.non_quads,
        edge: 0.0,
        ms: 0.0,
        holes: 0,
        irregular: r.irregular,
        edge_max_ratio: f32::NAN,
        edge_median_ratio: f32::NAN,
        edge_max_span: f32::NAN,
        shape: r.shape,
        aligned: true,
        folded: 0,
    };
    let text = crate::sculpt3d::history::retopo_global::retopo_line(&line);
    assert!(
        text.contains("enviesamento"),
        "a linha do relatorio nao diz o enviesamento -- foi a coluna que faltava em 2026-08-22:\n{text}"
    );
}

/// ⛔⛔ **VERMELHO E COM ENDEREÇO — os nossos quads não são quadrados como os dele.**
///
/// ⭐ **A barra sai do oráculo**, medida com o mesmo código sobre a saída dele na
/// bancada (2026-08-22): enviesamento p50 `5–6°`, e **0 a 4** faces em milhares com
/// um canto pior que 60°.
///
/// | peça (`d = 1,0`) | enviesamento p50 | p99 | faces `> 60°` |
/// |---|---|---|---|
/// | ⭐ oráculo, orelha | **`6°`** | `20°` | **`0` de 4 658** |
/// | ⛔ nós, orelha | `27°` | `79°` | **`9 159` de 78 403 (12 %)** |
/// | ⛔ nós, gancho | `28°` | `86°` | `1 872` de 8 772 (21 %) |
/// | ⛔ nós, enrugada | `18°` | `87°` | `8 281` de 29 468 (28 %) |
///
/// # ⭐⭐⭐ A causa, medida — e ela NÃO é onde os pontos estão
///
/// ⛔ **Uma relaxação por ajuste de quadrado foi construída, medida e não curou
/// isto** ([`ph2d_quadfill::SQUARE_ROUNDS`]). Dezasseis rondas na orelha levaram o
/// aspecto máximo de `122,7` a `30,3` — e o enviesamento p50 de `27°` para… `26°`,
/// comprando **576 dobras contra 171**. *Se mover vértices dezasseis vezes não move
/// a mediana, endireitar um quad desendireita o vizinho:* o esmagamento está na
/// **conectividade**, não nas posições.
///
/// ⭐⭐ **E a sonda [`super::field_follow`] nomeia qual conectividade.** Medindo o
/// desvio da grade ao campo cruzado, por família de linhas:
///
/// | | só a família `u` | ⭐ **as duas famílias** |
/// |---|---|---|
/// | oráculo, gancho | `5,1°` | **`7,6°`** — mal se move |
/// | ⛔ nós, gancho | `9,9°` | ⛔ **`19,2°`** — mais do dobro |
///
/// ⇒ **A nossa primeira família de linhas segue o campo; a segunda não fica
/// ortogonal a ela.** É a assinatura da interpolação transfinita: ela casa com a
/// fronteira do patch e **enviesa no meio**. ⚠️ E [`ph2d_quadfill::fill_with`] nem
/// sequer **recebe** o campo — uma fase que não o tem entre os argumentos não o pode
/// seguir no interior.
///
/// ⇒ **A cura nomeada:** o interior de um patch tem de nascer de uma
/// parametrização **alinhada ao campo**, não de uma interpolação da fronteira.
#[test]
#[ignore = "VERMELHO -- a cura e' a parametrizacao alinhada ao campo, ver o doc"]
fn the_quads_are_as_square_as_the_oracles() {
    for (name, reference) in [
        ("ORELHA", crate::sculpt3d::fixtures::eared_sphere()),
        ("GANCHO", crate::sculpt3d::fixtures::hooked_sphere()),
        ("ENRUGADA", crate::sculpt3d::fixtures::wrinkled_sphere()),
    ] {
        let mut work = reference.clone();
        ph2d_remesh_iso::remesh_isotropic(&mut work, ph2d_remesh_iso::ALPHA);
        work.triangulate();
        let target = ph2d_quadflow::edge_for_detail_with(
            &reference,
            1.0,
            ph2d_quadflow::GLOBAL_FLOOR_IN_INPUT_EDGES,
        );
        let dual = ph2d_crossfield::Dual::build(&work);
        let (field, _) = ph2d_crossfield::solve_miq(&dual);
        let layout = ph2d_trace::trace_patches(&work, &dual, &field);
        let spec = layout.to_layout(target).expect("o layout fecha");
        let (quant, _) =
            ph2d_quantize::quantize_within(&spec, ph2d_quantize::Budget::new(256, 512))
                .expect("quantiza");
        let (_, r) = ph2d_quadfill::fill(
            &work,
            &reference,
            &layout,
            &quant,
            ph2d_quadfill::SMOOTHING_ROUNDS,
        )
        .expect("monta");
        let faces = (r.quads + r.non_quads).max(1);
        eprintln!(
            "[f5] {name}: enviesamento p50 {:.0}° p99 {:.0}° · {} de {faces} acima de 60°",
            r.shape.skew_p50, r.shape.skew_p99, r.shape.skew_over_60
        );
        assert!(
            r.shape.skew_p50 <= 10.0,
            "{name}: enviesamento mediano {:.0}°, barra 10° (o oraculo entrega 5 a 6)",
            r.shape.skew_p50
        );
        #[allow(clippy::cast_precision_loss)]
        let bad = r.shape.skew_over_60 as f32 / faces as f32;
        assert!(
            bad <= 0.001,
            "{name}: {:.1}% das faces com um canto pior que 60°, barra 0,1% (o oraculo entrega 0)",
            100.0 * bad
        );
    }
}
