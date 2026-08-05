//! **A ORDEM das duas recusas de aparência do `cook_gpu`** (ADR-0154 / esta wave).
//!
//! O `cook_gpu` recusa por dois motivos distintos, e ONDE cada um roda relativo
//! ao plano é load-bearing:
//!
//! - **Live vector (`source.shape`)** — sem rota `geometry_id` no device — recusa
//!   ANTES de `ph2d_gpu_cook::plan(...)`, para o pump da CPU possuir o tick do
//!   zero (sem marchar duas vezes um prefixo sequencial).
//! - **Objeto com sufixo que muda contagem (`source.object` + `suffix_changes_count`)**
//!   recusa DEPOIS do plano — ela PRECISA do plano para saber se o sufixo GPU
//!   reordena/muda a contagem (o que quebraria a partição de texture-run e
//!   pintaria o objeto como quads brancos).
//!
//! Este gate é de shell (o `cook_gpu` exige janela+GPU, então nenhum unit test o
//! alcança). Ele afirma a ORDEM (onde cada recusa roda relativo ao plano), não
//! uma distância em bytes: a propriedade é *live-vector antes de cozinhar* e
//! *objeto-count-changing depois de planejar*, não *na linha N*. A CORRETUDE dos
//! predicados (quais grafos recusam) é o gate de unidade
//! `the_recusal_catches_the_live_vector_but_not_the_object`.

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
    let plan = body
        .find("ph2d_gpu_cook::plan(")
        .expect("cook_gpu still plans");
    let object = body.find("graph_has_object_source(").expect(
        "cook_gpu consults graph_has_object_source — the count-changing cerca was removed",
    );
    let changes_count = body
        .find("suffix_changes_count(")
        .expect("the count-changing cerca consults GpuPlan::suffix_changes_count");
    let route = body.find("gpu_route(").expect("cook_gpu still routes");

    // A recusa do LIVE VECTOR roda ANTES do plano (o pump da CPU possui o tick do
    // zero — nenhum prefixo sequencial marchado duas vezes).
    assert!(
        live_vector < plan,
        "the live-vector recusal must run BEFORE planning (live@{live_vector} vs plan@{plan})"
    );

    // A CERCA de contagem (objeto + `suffix_changes_count`) roda DEPOIS do plano
    // (ela precisa do plano para inspecionar os estágios do sufixo GPU) e ANTES
    // de rotear.
    assert!(
        object > plan && changes_count > plan && object < route,
        "the object count-changing cerca must run AFTER the plan and before routing \
         (object@{object}, changes@{changes_count} vs plan@{plan}, route@{route})"
    );

    // Cada recusa cai para o pump da CPU (`FellThrough`), não `Handled`. A do
    // live-vector vive entre a chamada do predicado e o plano; a da cerca entre
    // a chamada de objeto e o `gpu_route`.
    assert!(
        body[live_vector..plan].contains("GpuOutcome::FellThrough"),
        "the live-vector branch must return GpuOutcome::FellThrough (fall to the CPU render)"
    );
    assert!(
        body[object..route].contains("GpuOutcome::FellThrough"),
        "the count-changing cerca must return GpuOutcome::FellThrough (fall to the CPU render)"
    );
}
