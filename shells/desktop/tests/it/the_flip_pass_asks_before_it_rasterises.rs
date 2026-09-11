//! **ARCH-GATE — o Pass A do Flip PERGUNTA antes de rasterizar** (doc 12 §22.3).
//!
//! ⚠️ **Por que arch-gate e não teste de comportamento:** a decisão mora dentro do
//! `composite_layers`, que exige `GpuContext` + `GameRt` + janela. Os gates de unidade cobrem a
//! impressão digital e a lei do skip (`flip_pass_stage_tests.rs`) — e **as duas podem estar perfeitas
//! com o laço nunca perguntando a elas**. É o *registrado ≠ despachado* que este repo já pagou; a
//! costura precisa de alguém olhando a costura.

use std::fs;

/// ⚠️ Via `CARGO_MANIFEST_DIR` — o CWD de um teste é a raiz da CRATE, não do workspace (a convenção
/// do arch-gate vizinho, `the_flip_preview_bakes_through_the_same_door`).
const PASS: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/src/render_loop/flip_pass.rs");

fn src() -> String {
    fs::read_to_string(PASS).unwrap_or_else(|e| panic!("{PASS} ilegivel: {e}"))
}

/// O laço consulta a frescura **ANTES** de rasterizar, e o `continue` é o que torna a resposta
/// load-bearing (perguntar e ignorar seria pior que não perguntar).
#[test]
fn the_stage_loop_asks_before_it_rasterises() {
    let s = src();

    // Controle POSITIVO: as duas âncoras existem. Sem isto um rename deixa o gate verde afirmando
    // nada — a forma de gate vazio que este repo já pegou duas vezes.
    let ask = s
        .find("needs_stage(")
        .unwrap_or_else(|| panic!("`needs_stage(` desapareceu do {PASS} — gate vazio"));
    let raster = s
        .find("stage_layer(")
        .unwrap_or_else(|| panic!("`stage_layer(` desapareceu do {PASS} — gate vazio"));

    assert!(
        ask < raster,
        "a frescura tem de ser consultada ANTES do `stage_layer` — perguntar depois de rasterizar \
         nao economiza nada"
    );

    // A resposta é HONRADA: entre a pergunta e o raster há um `continue` (o corpo do `if !`).
    let entre = &s[ask..raster];
    assert!(
        entre.contains("continue"),
        "a pergunta existe e o laco a IGNORA: falta o `continue` entre `needs_stage` e \
         `stage_layer` (perguntar sem pular e' pior que nao perguntar)"
    );
}

/// ⭐ **A 2ª metade do skip vem do COMPOSITOR, não de nós** — a fatia pode ter sido despejada pelo
/// LRU ou limpa por um rebuild, e um memo sozinho mandaria compor arte velha nesses dois casos.
///
/// ⚠️ Este gate recusa explicitamente o literal `true`: é a mutação mais provável (*"o compositor
/// sempre tem, não tem?"*), e ela é invisível até o dia em que a cena passa do `cache_cap`.
#[test]
fn the_second_half_of_the_skip_is_the_compositors_own_word() {
    let s = src();
    let ask = s.find("needs_stage(").expect("ancora");
    // A chamada inteira, até o `)` do `if`.
    let fim = s[ask..].find(") {").map_or(s.len(), |o| ask + o);
    let chamada = &s[ask..fim];
    assert!(
        chamada.contains("has_slice("),
        "o 3o argumento do `needs_stage` tem de ser a palavra do compositor \
         (`compositor.has_slice(..)`), e nao um palpite nosso: {chamada}"
    );
    assert!(
        !chamada.contains("true"),
        "`true` cravado no lugar do `has_slice` faz o skip mostrar arte despejada: {chamada}"
    );
}

/// A impressão digital é montada com **a câmera da CAMADA**, não com a câmera do frame.
///
/// ⚠️ É a diferença entre paralaxe/fantasma serem vistos e serem ignorados: `layer_cam` já traz o
/// `parallax_model`, o `fold_model` do objeto e o `with_ghost_tint` dobrados. Passar o `cam` cru
/// congelaria toda camada com `depth < 1` e todo fantasma na 1ª pose que eles tiveram.
#[test]
fn the_fingerprint_is_built_from_the_layers_own_camera() {
    let s = src();
    let fp = s
        .find("flip_pass_stage::fingerprint(")
        .unwrap_or_else(|| panic!("`fingerprint(` desapareceu do {PASS} — gate vazio"));
    let fim = s[fp..].find(");").map_or(s.len(), |o| fp + o);
    let chamada = &s[fp..fim];
    assert!(
        chamada.contains("&layer_cam"),
        "a impressao tem de usar `&layer_cam` (paralaxe + fold + tint dobrados), nao o `cam` do \
         frame: {chamada}"
    );
    assert!(
        chamada.contains("l.preview"),
        "sem o preview na impressao o traco VIVO congela no 1o frame do gesto: {chamada}"
    );
}
