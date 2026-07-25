//! **O alcance do Gap Closure é ZOOM-INVARIANTE** (Enio 2026-07-25, doc 06 §8).
//!
//! O Gap mede unidades de MUNDO (como o Size do pincel, §4.C.6), não px de tela. A régua
//! do alcance converte MUNDO→LOCAL (a geometria do desenho é local, ADR-0111) pela escala
//! do objeto e **NADA MAIS** — em particular, NÃO multiplica pelo `px_to_world` da câmera.
//! Foi exatamente esse fator que fazia o alcance encolher ao aproximar a câmera: um vão de
//! tamanho fixo em documento passava a ocupar mais px de tela e "saía de alcance" (o
//! *"de perto some"*).
//!
//! ## Por que um gate de TEXTO
//!
//! A régua vive em `fill_click` (o clique real) e `flip_gap_helpers_tick` (o overlay ao
//! vivo) — as DUAS têm de usar a mesma fórmula, senão a tela mostra um vão que o clique não
//! fecha. Ambas exigem `gfx` (câmera + janela), então nenhum unit test as alcança. Este
//! gate lê o fonte do produto e afirma a propriedade: a linha do alcance converte por
//! `obj_scale` (mundo→local, legítimo) e **não** por `px_to_world` (o zoom).
//!
//! Reintroduzir `* px_to_world` na régua traz o "de perto some" de volta — e este gate
//! fica vermelho. (Controle positivo: `px_to_world` CONTINUA no `fill_click` para a
//! `precision` e o debug; o gate isola a linha do alcance, não o arquivo.)

/// A linha de atribuição do alcance do Gap num fonte, isolada por um marcador único.
///
/// `marker` é a âncora textual do início da atribuição (`gap_reach:` ou `let reach =`);
/// devolvemos o trecho até o `;` — só a EXPRESSÃO do alcance, não o arquivo, para o
/// `px_to_world` legítimo da `precision`/debug não dar falso-positivo.
fn reach_expr<'a>(src: &'a str, file: &str, marker: &str) -> &'a str {
    let start = src.find(marker).unwrap_or_else(|| {
        panic!("`{marker}` sumiu de {file} — se a régua do Gap foi renomeada, atualize o gate")
    });
    let rest = &src[start..];
    let end = rest
        .find(';')
        .unwrap_or_else(|| panic!("a atribuição do alcance em {file} não fecha com `;`?"));
    &rest[..end]
}

#[test]
fn the_gap_reach_is_zoom_invariant() {
    // O clique real.
    let click = include_str!("../src/flip_fill.rs");
    let click_reach = reach_expr(click, "flip_fill.rs", "gap_reach:");
    // Controle positivo: o `px_to_world` legítimo (precision/debug) EXISTE no arquivo.
    assert!(
        click.contains("px_to_world"),
        "controle positivo falhou: `px_to_world` devia existir em flip_fill.rs (precision/debug)"
    );
    assert!(
        !click_reach.contains("px_to_world"),
        "o alcance do Gap no CLIQUE voltou a depender do zoom (`px_to_world`) — o \
         \"de perto some\". A régua e' mundo->local por `obj_scale` e mais nada: `{click_reach}`"
    );
    assert!(
        click_reach.contains("obj_scale"),
        "o alcance do clique tem de converter mundo->local por `obj_scale`: `{click_reach}`"
    );

    // O overlay ao vivo.
    let live = include_str!("../src/flip_gap_live.rs");
    let live_reach = reach_expr(live, "flip_gap_live.rs", "let reach =");
    assert!(
        !live_reach.contains("px_to_world"),
        "o alcance do Gap no OVERLAY voltou a depender do zoom (`px_to_world`): `{live_reach}`"
    );
    assert!(
        live_reach.contains("obj_scale"),
        "o alcance do overlay tem de converter mundo->local por `obj_scale`: `{live_reach}`"
    );
}
