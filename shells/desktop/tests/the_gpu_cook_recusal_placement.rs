//! **A ORDEM das recusas de aparência do `cook_gpu`** (ADR-0154 / esta wave).
//!
//! O `cook_gpu` recusa por três motivos de aparência, e ONDE cada um roda relativo
//! ao plano é load-bearing:
//!
//! - **Live vector SHAPE (`source.shape`)** — sem rota `geometry_id` no device —
//!   recusa ANTES de `ph2d_gpu_cook::plan(...)`, para o pump da CPU possuir o tick
//!   do zero (sem marchar duas vezes um prefixo sequencial). Sinal: por-TIPO de nó
//!   (`graph_has_live_vector_source`).
//! - **Objeto que resolve para um VETOR VIVO (`source.object` → `geometry_id`)** —
//!   recusa também ANTES do plano, mas por CONTEÚDO: se um `source.object` é um
//!   vetor depende do que o artista NOMEOU, então a recusa varre os externals
//!   publicados (`cook_publishes_live_geometry`). Um objeto de sprite puro
//!   (`texture_id`) NÃO recusa — fica no stamp de GPU (o ponto desta wave).
//! - **Objeto com sufixo que muda contagem (`suffix_changes_count`)** recusa
//!   DEPOIS do plano — ela PRECISA do plano para saber se o sufixo GPU
//!   reordena/muda a contagem (o que quebraria a partição de texture-run).
//!
//! Este gate é de shell (o `cook_gpu` exige janela+GPU, então nenhum unit test o
//! alcança). Ele afirma a ORDEM (onde cada recusa roda relativo ao plano), não
//! uma distância em bytes. A CORRETUDE dos predicados (quais grafos recusam) são os
//! gates de unidade `the_recusal_catches_the_live_vector_but_not_the_object` e
//! `the_object_recusal_is_content_aware_a_live_vector_recuses_but_a_sprite_stays`.

use std::fs;

#[test]
fn the_recusals_run_in_the_right_place_relative_to_the_plan() {
    let src =
        fs::read_to_string("src/render_loop/motion_bridge_gpu.rs").expect("motion_bridge_gpu.rs");

    // O corpo de `cook_gpu` (depois da sua assinatura — as DEFINIÇÕES dos
    // predicados vêm antes na fonte, então isto pega só as CHAMADAS).
    let body = src
        .split_once("pub(super) fn cook_gpu(")
        .expect("cook_gpu exists")
        .1;

    let live_vector = body.find("graph_has_live_vector_source(").expect(
        "cook_gpu consults graph_has_live_vector_source — the live-vector recusal was removed",
    );
    let live_geo = body.find("cook_publishes_live_geometry(").expect(
        "cook_gpu consults cook_publishes_live_geometry — the content-aware object recusal was removed",
    );
    // ⚠️ A recusa do SUBSTEP (doc 89, folha 13): o device marcha o PLANO INTEIRO e o substep da
    // CPU é POR-ZONA, então um documento substepado tem UMA resposta só enquanto o device não
    // souber marchar por-zona. Como as irmãs de vetor vivo, ela roda ANTES do plano.
    let substeps = body.find("graph_asks_for_substeps(").expect(
        "cook_gpu consults graph_asks_for_substeps — a recusa do substep foi removida, e sem ela \
         os dois produtores mostram quadros diferentes",
    );
    let plan = body
        .find("ph2d_gpu_cook::plan(")
        .expect("cook_gpu still plans");
    let changes_count = body
        .find("suffix_changes_count(")
        .expect("the count-changing cerca consults GpuPlan::suffix_changes_count");
    let route = body.find("gpu_route(").expect("cook_gpu still routes");

    // As DUAS recusas de vetor vivo (shape por-tipo, objeto por-conteúdo) rodam
    // ANTES do plano — o pump da CPU possui o tick do zero, sem marchar duas vezes
    // um prefixo sequencial, e a GPU nunca tenta desenhar um `geometry_id`.
    assert!(
        substeps < plan,
        "a recusa do substep tem de rodar ANTES do plano (substeps@{substeps} vs plan@{plan})"
    );
    assert!(
        live_vector < plan && live_geo < plan,
        "the live-vector recusals must run BEFORE planning \
         (shape@{live_vector}, object@{live_geo} vs plan@{plan})"
    );

    // A CERCA de contagem roda DEPOIS do plano (ela precisa do plano para inspecionar
    // os estágios do sufixo GPU) e ANTES de rotear.
    assert!(
        changes_count > plan && changes_count < route,
        "the object count-changing cerca must run AFTER the plan and before routing \
         (changes@{changes_count} vs plan@{plan}, route@{route})"
    );

    // Cada recusa de vetor vivo cai para o pump da CPU (`FellThrough`), não
    // `Handled`. As duas vivem entre a chamada do predicado e o plano.
    assert!(
        body[live_vector..plan].contains("GpuOutcome::FellThrough"),
        "the live-vector recusals must return GpuOutcome::FellThrough (fall to the CPU render)"
    );
    // A cerca de contagem cai entre a checagem de contagem e o `gpu_route`.
    assert!(
        body[changes_count..route].contains("GpuOutcome::FellThrough"),
        "the count-changing cerca must return GpuOutcome::FellThrough (fall to the CPU render)"
    );
}
