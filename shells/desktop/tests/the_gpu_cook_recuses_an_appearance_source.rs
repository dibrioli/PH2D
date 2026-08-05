//! **O GPU cook recusa um documento de objeto/forma ANTES de planejar** (ADR-0154/0155).
//!
//! A lowering do cook GPU-resident é sprite-only: ela hardcoda `texture_id` no atlas
//! compartilhado e não tem rota `geometry_id` (vetor). Então um documento que traz um
//! `source.object` (o tile dele) ou um `source.shape` (o vetor vivo) desenha como quads
//! brancos do atlas no instante em que um estágio GPU roda (`source → deform → …` é
//! Hybrid) — o report do artista (`Shape → duplicator → rotate` = retângulos brancos).
//!
//! O conserto é `cook_gpu` recusar (`return GpuOutcome::FellThrough`) quando o grafo
//! tem uma fonte de aparência, e recusar **antes** de `ph2d_gpu_cook::plan(...)`, para
//! o pump da CPU possuir o tick do zero (sem marchar duas vezes um prefixo sequencial).
//!
//! Este gate é de shell (o `cook_gpu` exige janela+GPU, então nenhum unit test o
//! alcança). Ele afirma a ORDEM (recusa antes do plano), não uma distância em bytes: a
//! propriedade é *recuse antes de cozinhar*, não *na linha N*. A CORRETUDE do predicado
//! (quais grafos recusam) é o gate de unidade
//! `a_document_bringing_in_an_object_or_shape_recuses_from_the_gpu`.

use std::fs;

#[test]
fn the_gpu_cook_recuses_an_appearance_source_before_planning() {
    let src =
        fs::read_to_string("src/render_loop/motion_bridge_gpu.rs").expect("motion_bridge_gpu.rs");

    // O corpo de `cook_gpu` (depois da sua assinatura — a DEFINIÇÃO do predicado vem
    // antes na fonte, então isto pega só a CHAMADA).
    let body = src
        .split_once("pub(super) fn cook_gpu(")
        .expect("cook_gpu exists")
        .1;

    let recuse = body
        .find("graph_has_appearance_source(")
        .expect("cook_gpu consults graph_has_appearance_source — the recusal was removed");
    let plan = body
        .find("ph2d_gpu_cook::plan(")
        .expect("cook_gpu still plans");

    assert!(
        recuse < plan,
        "the appearance-source recusal must run BEFORE planning/cooking, so the CPU pump \
         owns the tick from scratch (recuse@{recuse} vs plan@{plan})"
    );

    // E a recusa é um FellThrough (cai para o pump da CPU), não um Handled: só o trecho
    // entre a chamada do predicado e o plano deve conter o `FellThrough`.
    let between = &body[recuse..plan];
    assert!(
        between.contains("GpuOutcome::FellThrough"),
        "the appearance-source branch must return GpuOutcome::FellThrough (fall to the CPU render)"
    );
}
