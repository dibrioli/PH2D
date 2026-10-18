//! ⭐ **Os prefixos `@` e `:` da paleta global** (CodeEditor Onda 2, I14): os sítios do documento de
//! família ACTIVO ([`crate::doc_surface::DocumentSurface::places`]) e a ida até eles (`go_to`). A lei
//! da busca está no widget (`command_palette::scope`); aqui só o que a aba oferece e o pick.

use crate::doc_surface::DocSymbolKind;
use crate::widget::command_palette::{
    LineScope, PaletteGroup, PaletteItem, PaletteScopes, PaletteSub, line_target,
};
use ph2d_a11y::NodeId;
use ph2d_i18n::tr;
use ph2d_tokens::ColorToken;
use ph2d_tool_registry::hash_node_id;

/// O item «Go to line N» (o número vem da busca).
pub const PALETTE_GOTO_LINE: NodeId = hash_node_id("palette.goto_line");
const SYMBOL_BASE: NodeId = hash_node_id("palette.symbol");
/// Quantos símbolos um id sabe dizer (de que recurso é: do espaço dos ids — 2^20 de 2^64).
const SYMBOL_IDS: u64 = 1 << 20;

fn symbol_id(i: usize) -> NodeId {
    NodeId(SYMBOL_BASE.0 ^ (i as u64 + 1))
}

/// O índice do símbolo do item `id`, se é um.
fn symbol_index(id: NodeId) -> Option<usize> {
    let k = id.0 ^ SYMBOL_BASE.0;
    (1..=SYMBOL_IDS).contains(&k).then(|| (k - 1) as usize)
}

/// `id` é um item dos prefixos (o pick é desta paleta).
#[must_use]
pub fn owns(id: NodeId) -> bool {
    id == PALETTE_GOTO_LINE || symbol_index(id).is_some()
}

/// Os prefixos que a aba activa dá agora (sem aba de família: nenhum símbolo, nenhuma linha).
pub fn scopes(hero: &mut crate::HeroScreen) -> PaletteScopes {
    let Some(places) = hero
        .documents
        .active_surface_mut()
        .and_then(|(f, id)| f.places(id))
    else {
        return PaletteScopes::default();
    };
    let mut subs: Vec<PaletteSub> = Vec::new();
    for (kind, key) in [
        (DocSymbolKind::Function, "chrome.palette.symbols_functions"),
        (DocSymbolKind::Property, "chrome.palette.symbols_properties"),
        (DocSymbolKind::Variable, "chrome.palette.symbols_variables"),
    ] {
        let items: Vec<PaletteItem> = places
            .symbols
            .iter()
            .enumerate()
            .filter(|(i, s)| s.kind == kind && (*i as u64) < SYMBOL_IDS)
            .map(|(i, s)| PaletteItem {
                label: match &s.container {
                    Some(c) => format!("{} \u{00b7} {c}", s.name),
                    None => s.name.clone(),
                },
                id: symbol_id(i),
                keywords: "",
            })
            .collect();
        if !items.is_empty() {
            subs.push(PaletteSub {
                title: Some(tr(key).to_owned()),
                items,
            });
        }
    }
    PaletteScopes {
        symbols: (!subs.is_empty()).then(|| PaletteGroup {
            title: tr("chrome.palette.symbols").to_owned(),
            color: ColorToken::NodeCatSource,
            subs,
        }),
        line: Some(LineScope {
            id: PALETTE_GOTO_LINE,
            lines: places.lines,
        }),
    }
}

/// Executa um pick dos prefixos: leva o cursor da aba activa ao símbolo, ou à linha que a busca
/// `query` (a do momento do pick) pedia. `true` = era um deles.
pub fn route(hero: &mut crate::HeroScreen, id: NodeId, query: &str) -> bool {
    if !owns(id) {
        return false;
    }
    let Some((f, tab)) = hero.documents.active_surface_mut() else {
        return true;
    };
    let target = if id == PALETTE_GOTO_LINE {
        let lines = f.places(tab).map_or(1, |p| p.lines);
        query
            .strip_prefix(':')
            .and_then(|rest| line_target(rest, lines))
            .map(|(n, c)| (n - 1, c.map_or(0, |c| c - 1)))
    } else {
        symbol_index(id)
            .and_then(|i| f.places(tab)?.symbols.into_iter().nth(i))
            .map(|s| (s.line, s.col))
    };
    if let Some((line, col)) = target {
        f.go_to(tab, line, col);
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn o_id_de_um_simbolo_volta_ao_indice() {
        for i in [0, 1, 41, 1 << 19] {
            assert_eq!(symbol_index(symbol_id(i)), Some(i));
        }
        assert_eq!(symbol_index(SYMBOL_BASE), None);
        assert!(owns(PALETTE_GOTO_LINE) && !owns(hash_node_id("code_cmd_save")));
    }
}
