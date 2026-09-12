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
//! inalcançável pelo harness. O anel (`ph2d_app_flip::cursor::ring_radius`) É testável e tem os
//! seus gates; este aqui cobre o irmão que o harness não alcança, lendo o arquivo do
//! produto e afirmando a única coisa que importa: **o apply da borracha não fala em
//! `width_px` nem em `opacity`.**
//!
//! Se algum dia a borracha precisar mesmo do número do pincel, ela deve pedi-lo à tool
//! (que é a dona da regra do link), nunca re-derivá-lo aqui.

const SRC: &str = include_str!("../../src/erase.rs");

/// O corpo do APPLY da borracha — a função que traduz estilo → (raio, força). Recortar a
/// função (e não o arquivo) importa: o módulo tem testes que legitimamente falam de outras
/// coisas, e um gate sobre o arquivo inteiro viraria falso-positivo.
///
/// ⚠️ **W2/L5 2.ª volta:** ela chamava-se `flip_erase_apply` enquanto era um método de
/// `App`; hoje é `pub fn apply(state, flip, playhead, camera, win, w2l, x, y)`. A agulha
/// segue a LEI (*o apply da borracha*) e não o nome antigo — mas o RECORTE passou a ter de
/// parar na função seguinte, porque `apply` deixou de ser a última do ficheiro (HOWTO
/// §2.13: uma fronteira nova muda o que está em volta, não só o nome).
///
/// ⚠️⚠️ **E a 1.ª redacção desta correcção mordeu a MESMA lei que ela cita:** eu escrevi a
/// agulha `pub fn apply(` e a função é `pub(crate) fn apply(` — *ancorar no MODIFICADOR é
/// o defeito, e ele reaparece dentro da própria cura*. A agulha diz `fn apply(`.
fn apply_body() -> &'static str {
    let start = SRC
        .find("fn apply(")
        .expect("o apply da borracha sumiu de `erase.rs` — se foi renomeado, actualize este gate");
    let rest = &SRC[start..];
    // Até a PRÓXIMA função de topo (ou o módulo de testes, ou o fim).
    let end = rest[1..]
        .find("\npub(crate) fn ")
        .or_else(|| rest[1..].find("\npub fn "))
        .map(|o| o + 1)
        .into_iter()
        .chain(rest.find("#[cfg(test)]"))
        .min()
        .unwrap_or(rest.len());
    let body = &rest[..end];
    // Controlo POSITIVO: um recorte que colapsou para quase nada aprovaria por vacuidade.
    assert!(
        body.len() > 400,
        "o recorte do apply da borracha deu {} bytes — perdeu o sujeito",
        body.len()
    );
    body
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
