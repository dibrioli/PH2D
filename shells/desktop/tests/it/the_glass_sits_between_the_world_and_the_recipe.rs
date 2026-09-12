//! ⛔⛔⛔ **O VIDRO JATEADO tem SEIS fios, e nenhum gate de unidade vê um só deles** (Enio,
//! 2026-09-07: *«crie a feature de borrar discretamente o que está por trás do prefab como um vidro
//! jateado»*).
//!
//! As leis vivem em portas com gate próprio — a partição do desenho vectorial
//! (`ph2d-vec-render`), a retenção das peças raster (`present_frost::lift`), o passe de borrão
//! (`ph2d-render`). O que **nenhuma** delas mede é a ORDEM em que o quadro as chama, e é aí que
//! esta feature falha de forma indistinguível a olho:
//!
//! | fio partido | o que o artista vê |
//! |---|---|
//! | o interruptor nunca escrito | nada muda ao abrir a receita |
//! | o documento fica na cena do chrome | os painéis borram junto com o mundo |
//! | a receita nunca codificada | a receita **desaparece** ao abrir |
//! | o vidro corre DEPOIS do chrome | os painéis somem e a receita aparece duas vezes |
//! | o compositor não lê o acumulador | o vidro é feito e deitado fora |
//! | as peças raster não são retidas | um **halo** à volta de cada peça de imagem da receita |
//!
//! ⛔ Ele é textual porque os fios vivem dentro do laço de quadro e do presente, cujas funções têm
//! ~35 argumentos e um `AppGfx` com uma surface de janela real.

use std::path::Path;

/// O corpo do ficheiro sem comentários — senão o censo lê o que o código DIZ sobre si.
fn code_of(rel: &str) -> String {
    let p = Path::new(env!("CARGO_MANIFEST_DIR")).join("src").join(rel);
    let body = std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("{}: {e}", p.display()));
    body.lines()
        .map(|l| match l.find("//") {
            Some(i) => &l[..i],
            None => l,
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// ⭐⭐⭐ **A codificação: o interruptor, o documento fora do chrome, e a receita numa cena própria.**
#[test]
fn the_encoding_splits_the_world_from_the_recipe() {
    let body = code_of("render_loop/mod.rs");
    assert!(
        body.contains("*frosting = ph2d_app_components::master_editing::any_open(sim);"),
        "o interruptor do vidro deixou de ser escrito, ou voltou a perguntar a` vista do VETOR — \
         uma receita feita so' de imagens nao tem forma vectorial nenhuma, e o vidro nao subiria"
    );
    assert!(
        body.contains("frost_doc_scene") && body.contains("vector_scene"),
        "o documento deixou de ter cena propria sob o vidro — os paineis borram com o mundo"
    );
    assert!(
        body.contains("ph2d_vec_render::dispatch_isolated(") && body.contains("frost_front_scene"),
        "a receita ja' nao e' codificada — ela DESAPARECE ao abrir, porque o mundo a salta"
    );
}

/// ⭐⭐⭐ **A ORDEM: o vidro entra ANTES da cena de chrome.**
///
/// O `glass` usa o intermediário do Vello para as próprias faixas. Corrê-lo depois do chrome deixa
/// naquele intermediário a **receita** em vez dos painéis — e o compositor põe a receita no lugar
/// do chrome. ⚠️ *Nenhuma das duas metades está errada; a ordem é que é a lei.*
#[test]
fn the_glass_runs_before_the_chrome_scene() {
    let body = code_of("render_loop/present.rs");
    let glass = body
        .find("present_frost::glass(")
        .expect("o presente nao poe o vidro — o borrao nunca acontece");
    let chrome = body
        .find("vector_scene.inner()")
        .expect("o presente deixou de rasterizar a cena de chrome");
    assert!(
        glass < chrome,
        "o vidro corre DEPOIS do chrome: o intermediario do Vello chega ao compositor com a \
         receita em vez dos paineis"
    );
}

/// ⭐⭐ **O compositor lê o acumulador quando há vidro** — senão o borrão é feito e deitado fora.
#[test]
fn the_compositor_reads_the_world_while_the_glass_is_up() {
    let body = code_of("render_loop/present.rs");
    assert!(
        body.contains("banded || frosting"),
        "o compositor continua a ler a saida do tonemap com o vidro em cima — o quadro sai como \
         se a feature nao existisse"
    );
}

/// ⭐⭐⭐ **As peças raster da receita são RETIDAS pelo fundo** — nas faixas e no passe principal.
///
/// Sem a retenção elas são desenhadas duas vezes: uma no fundo (que o vidro borra) e outra por
/// cima. A nítida cobre a própria silhueta, mas o borrão dela **escapa por fora** — um halo.
#[test]
fn the_raster_pieces_are_held_back_by_every_background_pass() {
    let body = code_of("render_loop/present.rs");
    assert!(
        body.contains("present_frost::lift("),
        "o quadro deixou de perguntar quem sobe para cima do vidro"
    );
    assert_eq!(
        body.matches("held.as_ref()").count(),
        2,
        "as RETIDAS tem de chegar a`s DUAS passagens do fundo — a faixa e o passe principal; \
         uma delas sozinha deixa a peca a desenhar-se atra's do vidro"
    );
}
