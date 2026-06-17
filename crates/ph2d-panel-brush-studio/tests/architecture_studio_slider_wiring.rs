//! Executable gate for the 8-site Brush Studio slider wiring.
//!
//! Por que existe: adicionar um slider exige fiar ~8 sites em 3 crates; faltar
//! UM = clique/drag dropado EM SILÊNCIO (compila e passa nos unit tests). Foi a
//! causa nº 1 do "compila mas nada funciona" (sliders de Wet Mix inertes, reset
//! não registrado). Este gate lê o FONTE (não compila) e prova as costuras que
//! a compilação não vê. Ver docs/IntegracaoMultiAgente/DIRETIVA_IMPLEMENTACAO.md §2.
//!
//! Invariantes (sliders do Brush Studio):
//!  - A ⊆ D  : todo slider que o painel torna VIVO (is_studio_slider) é de fato
//!             despachado pela tool — senão arrastar o slider não faz nada.
//!  - A ⊆ P  : todo slider vivo é REGISTRADO em populate() — senão o clique é
//!             dropado (a classe do "botão reset não funciona" / pills mortas).
//!  - D\A ⊆ DORMANT : todo slider que a tool despacha mas o painel NÃO expõe é
//!             um stub dormente DOCUMENTADO (Cerca de Chesterton). Hoje:
//!             Roundness + AlphaThreshold (campos que o engine não lê — exigem
//!             Stamp ABI 96B congelado; ver sections.rs / event.rs). Qualquer
//!             fio morto ACIDENTAL novo cai aqui e força uma decisão explícita.

use std::collections::BTreeSet;
use std::fs;

const STUDIO_PREFIX: &str = "PAINTER_STUDIO_";

/// Sliders despachados-mas-não-expostos, dormentes de propósito (engine não lê
/// o campo; precisa do Stamp ABI 96B congelado). NÃO são bug — são stubs
/// forward. Se um destes for WIRADO no painel sem o engine ler o campo, vira um
/// no-op silencioso; mantenha-os fora da UI até o engine aplicar.
const DORMANT: &[&str] = &["SHAPE_ROUNDNESS_SLIDER", "ALPHA_THRESHOLD_SLIDER"];

fn read(rel_from_manifest: &str) -> String {
    let path = format!("{}/{}", env!("CARGO_MANIFEST_DIR"), rel_from_manifest);
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("não consegui ler {path}: {e}"))
}

/// Remove comentários de linha `//` para não contar idents citados em prosa.
fn strip_line_comments(src: &str) -> String {
    src.lines()
        .map(|l| match l.find("//") {
            Some(i) => &l[..i],
            None => l,
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Todos os identificadores que terminam em `_SLIDER` no fonte (sem comentários).
fn slider_idents(src: &str) -> Vec<String> {
    let clean = strip_line_comments(src);
    let mut out = Vec::new();
    let bytes = clean.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let c = bytes[i];
        let is_ident = c.is_ascii_alphanumeric() || c == b'_';
        if is_ident {
            let start = i;
            while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
                i += 1;
            }
            let tok = &clean[start..i];
            if tok.ends_with("_SLIDER") {
                out.push(tok.to_string());
            }
        } else {
            i += 1;
        }
    }
    out
}

/// Sliders VIVOS no painel = `ids::X_SLIDER` em event.rs (is_studio_slider).
/// Nomes curtos (o painel re-exporta `PAINTER_STUDIO_X as X`).
fn live_panel_sliders() -> BTreeSet<String> {
    slider_idents(&read("src/event.rs")).into_iter().collect()
}

/// Sliders REGISTRADOS em populate.rs.
fn registered_sliders() -> BTreeSet<String> {
    slider_idents(&read("src/populate.rs")).into_iter().collect()
}

/// Sliders do STUDIO despachados pela tool = idents `PAINTER_STUDIO_*_SLIDER`
/// em trait_impls.rs (brush_studio_param_for_slider + o special-case do grain
/// depth). Exclui `PAINTER_SIDEBAR_*`. Normalizado para o nome curto.
fn tool_dispatched_studio_sliders() -> BTreeSet<String> {
    slider_idents(&read("../ph2d-tool-painter/src/tool/trait_impls.rs"))
        .into_iter()
        .filter(|t| t.starts_with(STUDIO_PREFIX))
        .map(|t| t[STUDIO_PREFIX.len()..].to_string())
        .collect()
}

#[test]
fn every_live_slider_is_dispatched_by_the_tool() {
    let live = live_panel_sliders();
    let dispatched = tool_dispatched_studio_sliders();
    let dead: Vec<_> = live.difference(&dispatched).collect();
    assert!(
        dead.is_empty(),
        "FIO MORTO: sliders VIVOS no painel (event.rs) que a tool NÃO despacha \
         (arrastar não faz nada) → {dead:?}. Adicione ao brush_studio_param_for_slider \
         (ou ao special-case) em trait_impls.rs."
    );
}

#[test]
fn every_live_slider_is_registered_in_populate() {
    let live = live_panel_sliders();
    let registered = registered_sliders();
    let unregistered: Vec<_> = live.difference(&registered).collect();
    assert!(
        unregistered.is_empty(),
        "CLIQUE DROPADO: sliders VIVOS (event.rs) NÃO registrados em populate.rs \
         (não viram widget hit-testável) → {unregistered:?}. Registre via pct()/slider_chip()/bipolar()."
    );
}

#[test]
fn dispatched_but_unexposed_sliders_are_documented_dormant() {
    let live = live_panel_sliders();
    let dispatched = tool_dispatched_studio_sliders();
    let dormant: BTreeSet<String> = DORMANT.iter().map(|s| s.to_string()).collect();

    // Stubs que a tool despacha mas o painel não expõe.
    let unexposed: BTreeSet<String> = dispatched.difference(&live).cloned().collect();
    let undocumented: Vec<_> = unexposed.difference(&dormant).collect();
    assert!(
        undocumented.is_empty(),
        "STUB NÃO-DOCUMENTADO: a tool despacha estes sliders mas o painel não os \
         expõe, e não estão na allowlist DORMANT → {undocumented:?}. Ou wire o painel \
         (8 sites) OU adicione à DORMANT com a razão (engine não lê o campo)."
    );

    // Mantém a allowlist honesta: cada DORMANT precisa ainda existir no dispatch.
    let stale: Vec<_> = dormant.difference(&dispatched).collect();
    assert!(
        stale.is_empty(),
        "DORMANT obsoleto: {stale:?} não é mais despachado pela tool — remova da allowlist."
    );
}
