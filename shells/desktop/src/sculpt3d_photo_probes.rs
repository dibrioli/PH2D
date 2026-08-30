//! **AS SONDAS DA FOTO** — o que o artista viu, em números.
//!
//! ⭐⭐ **O corte contra o [`super::retopo_field_probes`] foi forçado pela HR-18**
//! (683 contra 600), e é de ASSUNTO: lá as sondas do **campo** (o relevo, o peso do
//! alinhamento); aqui as do **defeito geométrico** — a aresta que atravessa a peça e
//! o leque que se fecha num ponto.
//!
//! ⛔ **Elas nascem de três fotos e da palavra «muito ruim»** (2026-08-22), sobre
//! uma saída que passava em **todas** as réguas desta linha: 100 % de quads, casca
//! fechada, `χ` exacta, densidade no alvo, 15 irregulares, valência máxima 6 e a
//! forma preservada a `0,085 %` da diagonal. *Uma peça pode passar em toda asserção
//! e estar visivelmente destruída.*

/// **A SONDA DAS FOTOS** — irmã pelo teto de LOC da shell (HR-18, 600): ver
/// [`measure`]. ⭐ Ela é a única régua desta linha que vê AMPUTAÇÃO.
#[path = "sculpt3d_photo_measure.rs"]
mod measure;

/// **AS RÉGUAS** — irmã pelo teto de LOC da shell (HR-18, 600), cortada por
/// RESPONSABILIDADE: ver [`rulers`].
#[path = "sculpt3d_photo_rulers.rs"]
mod rulers;

/// **AS SONDAS DA PORTA DO PRODUTO** — irmã pela mesma razão: ver [`button`].
#[path = "sculpt3d_photo_button.rs"]
mod button;

/// ⭐⭐⭐ **O CAMPO ACORDA NA PONTA?** — irmã pela mesma razão: ver [`field_wakes`].
#[path = "sculpt3d_photo_field_wakes.rs"]
mod field_wakes;

use rulers::{
    census, holes, islands, local, orientation_and_density, relief_density, spiked_ball, tips,
};

/// ⭐⭐⭐ **VERDE desde 2026-08-22** — a aresta de **56 %** que o artista fotografou
/// mede hoje **12,4 % · 7,8 % · 5,5 %** nos três níveis do slider, e as dobras
/// caíram de **2 204 para 171**. O que a curou foi o **campo**: `ALIGN_WEIGHT`
/// deixou de ser zero, e o número dele saiu do gabarito do oráculo (`PLAN.md`
/// §4-tritricies), não de uma varredura no fim da cadeia.
///
/// ⛔ **O que ela foi** (as fotos de 2026-08-22).
///
/// ⚠️ **O artista mandou três fotos e a palavra «muito ruim», e NENHUMA régua desta
/// linha piscou:** 100 % de quads · casca fechada · característica de Euler exacta ·
/// densidade no alvo (mediana `1,02×`) · 15 irregulares · valência máxima **6** · e
/// a forma preservada (`detail_lost` p95 `0,085 %`). *Uma peça pode passar em toda
/// asserção e estar visivelmente destruída* — é a terceira vez que esta linha paga
/// esta lição, e desta vez a régua **existia** e não estava apontada para esta peça.
///
/// ⭐ **A barra é a que o relatório já define** (`QuadRemeshReport::edge_max_span`,
/// `≤ 0,20`), e ela só corria sobre a `wrinkled_sphere` — que mede `6 %`.
///
/// | fixtura | aresta máxima, em fração da peça | dobras (d = 1,0) |
/// |---|---|---|
/// | ⛔ **orelha** | **56,5 % · 56,3 % · 57,0 %** nos três níveis do slider | ⛔ **2 204** (4,85 %) |
/// | gancho | 9,2 % · 8,0 % · 11,6 % | 19 |
/// | enrugada | 10,0 % · 6,4 % · 6,1 % | 70 |
///
/// ⭐⭐ **O que se sabe da aresta, medido:** ela liga dois pontos **ANTIPODAIS**,
/// ambos a raio `1,00` — na parte lisa da esfera, não no vinco da orelha — e o
/// comprimento dela é `1,98`, o **diâmetro**. ⚠️ Ela **não encolhe** quando o alvo
/// encolhe 10×: *não é falta de resolução, é um ponto no sítio errado.*
///
/// ⭐ **E o CONTROLO ilibou as fases de montante:** a maior aresta da fixtura é
/// `1,8 %` e a da saída do F1 é `3,3 %`. A cadeia cria-a.
///
/// ⛔ **Uma hipótese foi construída, medida e REJEITADA:** as falhas de amostragem
/// do achatamento (**3 329**, `8,4 %`, e **zero** nas outras duas fixturas) pareciam
/// a causa — o recurso delas é um ponto de Coons no **espaço**, que passa perto do
/// centro da peça. Curá-las com um recuo para dentro do domínio levou as falhas a
/// **0** e ⛔ **não moveu a aresta**, com as dobras a **subir 25 %**. *Elas
/// acompanham o defeito naquela peça; não o causam.*
///
/// # ⭐⭐⭐ A CAUSA, achada em 2026-08-22 — **um patch que dá três voltas à peça**
///
/// ⛔ **`patch 1` da orelha tem SEIS lados de comprimento**
/// `6,78 · 0,27 · 2,15 · 6,80 · 0,09 · 2,15` — perímetro **18,2**, que é **520 % da
/// diagonal da peça**. Os dois lados de `6,8` são quase a circunferência inteira
/// (`6,28`); os de `0,09` e `0,27` são lascas.
///
/// ⭐ **E o número separa-o de tudo o resto com margem:** em três fixturas e
/// dezenas de patches, o segundo maior perímetro é **230 %**. *Um contra 2,3× de
/// folga não é uma cauda de distribuição: é outra coisa.*
///
/// ⇒ **O mecanismo, do perímetro à aresta:** as lascas recebem `2` segmentos e os
/// lados longos `40`. A lei do leque (`L_i = e_{i−1} + e_{i+1}`) resolve isso com
/// raios `[1, 39, 1, 1, 39, 1]` — **quatro dos seis raios valem `1`**. Um raio de
/// `1` faz o sector ter **uma célula de fundo**, e o quad dessa célula liga um ponto
/// lá longe de um lado a um ponto lá longe do seguinte. Num patch que dá três voltas
/// à peça, esse quad **atravessa-a** — e as duas pontas saem antipodais, ambas a
/// raio `1,00`, exactamente como a foto mostra.
///
/// ⚠️ **É por isso que ela não encolhe com o slider:** a `d = 1,0` os lados longos
/// passam a `384` segmentos e os raios continuam `[6, 384, 10, 1, 379, 4]` — o
/// sector continua a ter **um** de fundo.
///
/// # ⭐⭐⭐ E o patch é um ANEL CORTADO E ABERTO — a cadeia causal fecha
///
/// Os lados de `patch 1`, pelos arcos que os compõem:
///
/// ```text
///     [16,17,4,5]   [6]   [7]   [8..14]   [15]   [7]
///        6,78      0,27  2,15    6,80     0,09   2,15
/// ```
///
/// ⭐⭐ **O arco `7` aparece DUAS vezes** — e os dois lados de `2,15` são ele, ida e
/// volta. Os ângulos nos cantos confirmam-no: `[74, 77, ⭐14, 118, 70, ⭐26]`, com os
/// dois *hairpins* exactamente nas pontas da ponte. ⇒ **Este patch é o anel que a
/// ponte abriu** (`PLAN.md` §4-sexvicies), e os lados de `6,78`/`6,80` são as duas
/// fronteiras dele — quase a circunferência da esfera cada uma.
///
/// ⭐⭐ **E o controlo mostra que a ponte não é opcional:** com ela desligada, a
/// orelha **RECUSA** nos três níveis do slider (`GenusLost`) — o layout dela não
/// fecha de outra maneira. *A ponte é o que a faz existir; o defeito é como o patch
/// dela é preenchido.*
///
/// # ⇒ Onde o conserto mora, por eliminação
///
/// | candidato | porque NÃO |
/// |---|---|
/// | a quantização dar mais segmentos à lasca | ⛔ a densidade dela **está certa**: `0,27` de comprimento com `2` segmentos é `0,135` cada, contra um alvo de `0,167` |
/// | `dissolve` o patch | ⛔ ele **junta** patches, e este já dá três voltas à peça |
/// | desligar a ponte | ⛔ a orelha deixa de fechar |
///
/// ⭐ **Sobra o PREENCHIMENTO.** A lei do leque (`L_i = e_{i−1} + e_{i+1}`) é a da
/// referência e está certa para um `n`-gono de verdade; ⛔ **um anel cortado não é
/// um hexágono** — duas das seis arestas dele são a *mesma* curva. O preenchimento
/// certo para ele é uma **faixa**: uma grade entre as duas fronteiras, com a ponte
/// como costura. *É o que o `fill_rectangle` já faz para `n = 4`, e é preciso
/// reconhecer este caso para lá o mandar.*
///
/// ```text
/// \
///   cargo test -p ph2d-host-desktop --release --bins \
///   the_ear_does_not_ship_an_edge_across_the_piece -- --ignored --nocapture
/// ```
///
/// ```text
/// \
///   cargo test -p ph2d-host-desktop --release --bins \
///   the_ear_does_not_ship_an_edge_across_the_piece -- --ignored --nocapture
/// ```
#[test]
fn the_ear_does_not_ship_an_edge_across_the_piece() {
    let reference = crate::sculpt3d::fixtures::eared_sphere();
    let b = reference.bounds();
    let d = [
        b.max[0] - b.min[0],
        b.max[1] - b.min[1],
        b.max[2] - b.min[2],
    ];
    let diag = d[0].mul_add(d[0], d[1].mul_add(d[1], d[2] * d[2])).sqrt();
    let mut work = reference.clone();
    ph2d_remesh_iso::remesh_isotropic(&mut work, ph2d_remesh_iso::ALPHA);
    work.triangulate();
    let dual = ph2d_crossfield::Dual::build(&work);
    let (field, _) = ph2d_crossfield::solve_miq(&dual);
    let layout = ph2d_trace::trace_patches(&work, &dual, &field);
    for detail in [0.25f32, 0.5, 1.0] {
        let target = ph2d_quadflow::edge_for_detail_with(
            &reference,
            detail,
            ph2d_quadflow::GLOBAL_FLOOR_IN_INPUT_EDGES,
        );
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
        let span = r.edge_max / diag;
        eprintln!(
            "[f5] orelha d={detail:.2}: aresta maxima {:.1} % da peca ({} quads, {} dobras)",
            100.0 * span,
            r.quads,
            r.folded_local,
        );
        // ⚠️ **A barra é a mesma do `QuadRemeshReport::edge_max_span`**, e ⛔ ela
        // não se afrouxa: ela afirma *"nada atravessa a peça"*, que é um facto
        // absoluto e não uma preferência.
        assert!(
            span <= 0.20,
            "d={detail:.2}: uma aresta atravessa {:.1} % da peca (barra 20 %)",
            100.0 * span
        );
    }
}
