//! **Arch-gate: quem apaga lê os números da BORRACHA** (ADR-0114 §4.C).
//!
//! ## O que este gate protege
//!
//! O §4.C deu à borracha um Size/Strength PRÓPRIOS, atrás de um toggle de link (o
//! *Unified Paint Settings* do Blender). A tool resolve o link **uma vez** e publica o
//! resultado no snapshot como `erase_px` / `erase_strength`; o anel do cursor e o apply
//! da borracha leem ESSES campos e mais nenhum.
//!
//! Se o apply voltar a ler `style.width_px` / `style.opacity` (o código pré-§4.C), o
//! resultado é o pior tipo de bug: **o anel mostra um raio e a borracha apaga noutro**,
//! em silêncio, com o toggle na tela dizendo que estão deslinkados. O usuário vê a
//! ferramenta mentir sobre si mesma.
//!
//! ## Por que um gate de TEXTO
//!
//! `flip_erase_apply` deriva o raio de `gfx.camera` — precisa de **janela + GPU**, e o
//! `App` headless nasce com `gfx = None` ([[project_tests]]): o caminho inteiro é
//! inalcançável pelo harness. O anel (`flip_cursor::ring_radius`) É testável e tem os
//! seus gates; este aqui cobre o irmão que o harness não alcança, lendo o arquivo do
//! produto e afirmando a única coisa que importa: **o apply da borracha não fala em
//! `width_px` nem em `opacity`.**
//!
//! Se algum dia a borracha precisar mesmo do número do pincel, ela deve pedi-lo à tool
//! (que é a dona da regra do link), nunca re-derivá-lo aqui.

const SRC: &str = include_str!("../src/flip_erase.rs");

/// O corpo de `flip_erase_apply` — a função que traduz estilo → (raio, força) do apply.
/// Recortar a função (e não o arquivo) importa: o módulo tem testes que legitimamente
/// falam de outras coisas, e um gate sobre o arquivo inteiro viraria falso-positivo.
fn apply_body() -> &'static str {
    let start = SRC
        .find("fn flip_erase_apply")
        .expect("`flip_erase_apply` sumiu — se foi renomeada, atualize este gate");
    let rest = &SRC[start..];
    // Até o começo do módulo de testes (ou o fim do arquivo).
    let end = rest.find("#[cfg(test)]").unwrap_or(rest.len());
    &rest[..end]
}

/// 🔴 **O apply da borracha lê `erase_px` / `erase_strength`, e NÃO os do pincel.**
///
/// Mutação que sangra: trocar `style.erase_px` por `style.width_px` (ou
/// `style.erase_strength` por `style.opacity`) — exatamente o código pré-§4.C.
#[test]
fn the_eraser_apply_reads_the_effective_eraser_numbers() {
    let body = apply_body();

    assert!(
        body.contains("style.erase_px"),
        "o apply da borracha não lê `style.erase_px`: o raio deslinkado nunca chega ao \
         que de fato apaga, e o anel do cursor passa a mentir"
    );
    assert!(
        body.contains("style.erase_strength"),
        "o apply da borracha não lê `style.erase_strength`: a força deslinkada é ignorada"
    );
    assert!(
        !body.contains("style.width_px"),
        "o apply da borracha voltou a ler `style.width_px` (o Size do PINCEL) — com o \
         link desligado ele apaga num raio e o anel desenha outro"
    );
    assert!(
        !body.contains("style.opacity"),
        "o apply da borracha voltou a ler `style.opacity` (a força do PINCEL) — a \
         Strength deslinkada da borracha deixa de valer"
    );
}
