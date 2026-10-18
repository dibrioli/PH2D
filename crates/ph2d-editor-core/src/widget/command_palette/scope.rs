//! ⭐ **Os PREFIXOS da busca** (CodeEditor Onda 2, I14; medição
//! `docs/CodeEditor/ferramentas/medicoes_onda2/I9_I12_I14.md` §I14, o `Ctrl+P` do VS Code corrido).
//!
//! Sem prefixo a paleta é «tudo o que se pode fazer» (e, com algo escrito, os símbolos do documento
//! activo também); os prefixos ESTREITAM: `>` só os comandos, `@` só os símbolos (agrupados por tipo,
//! como o `@:`), `:` só ir a uma linha (`:40`, `:40:5`; um número além do fim prende à última, como no
//! VS Code). Só quem abriu a paleta com [`PaletteScopes`] os tem — a paleta de nós do Motion não.

use super::{PaletteGroup, PaletteItem, PaletteModel, PaletteSub, filter_model, top_match};
use ph2d_a11y::NodeId;
use std::borrow::Cow;

/// Ir a uma linha: o id do item e quantas linhas o documento activo tem.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LineScope {
    pub id: NodeId,
    pub lines: usize,
}

/// O que os prefixos mostram, dado por quem abriu a paleta.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PaletteScopes {
    /// `@` — os símbolos do documento activo (`None` = não há documento com símbolos).
    pub symbols: Option<PaletteGroup>,
    /// `:` — ir a uma linha (`None` = não há documento).
    pub line: Option<LineScope>,
}

impl PaletteScopes {
    /// `id` é um item de algum prefixo (o item da linha conta sempre: o número vem da busca).
    #[must_use]
    pub fn is_item(&self, id: NodeId) -> bool {
        self.line.as_ref().is_some_and(|l| l.id == id)
            || self
                .symbols
                .iter()
                .flat_map(|g| &g.subs)
                .flat_map(|s| &s.items)
                .any(|it| it.id == id)
    }
}

/// O que a paleta mostra para uma busca: o modelo filtrado, o item do `Enter` e a nota que substitui
/// o «nada casa» (a instrução do `:`).
pub struct Scoped<'a> {
    pub model: Cow<'a, PaletteModel>,
    pub top: Option<NodeId>,
    pub note: Option<String>,
}

/// O `:N` ou `:N:C` da busca, preso às linhas que há (1-based). `None` = ainda não é um número.
#[must_use]
pub fn line_target(rest: &str, lines: usize) -> Option<(usize, Option<usize>)> {
    let mut it = rest.trim().splitn(2, ':');
    let line: usize = it.next()?.trim().parse().ok()?;
    let col = match it.next().map(str::trim) {
        Some("") | None => None,
        Some(c) => Some(c.parse::<usize>().ok()?.max(1)),
    };
    Some((line.clamp(1, lines.max(1)), col))
}

fn only(model: &PaletteModel, groups: Vec<PaletteGroup>) -> PaletteModel {
    PaletteModel {
        title: model.title.clone(),
        groups,
        toggle: model.toggle.clone(),
    }
}

/// A busca `query` sobre `model` com os prefixos de `scopes` (sem eles, a lei de sempre).
#[must_use]
pub fn scoped<'a>(
    model: &'a PaletteModel,
    query: &str,
    scopes: Option<&PaletteScopes>,
) -> Scoped<'a> {
    let plain = |q: &str| -> Scoped<'a> {
        let m = if q.is_empty() {
            Cow::Borrowed(model)
        } else {
            Cow::Owned(filter_model(model, q))
        };
        let top = top_match(&m, q);
        Scoped {
            model: m,
            top,
            note: None,
        }
    };
    let Some(sc) = scopes else {
        return plain(query);
    };
    if let Some(rest) = query.strip_prefix('>') {
        return plain(rest.trim_start());
    }
    if let Some(rest) = query.strip_prefix('@') {
        let Some(sym) = &sc.symbols else {
            return Scoped {
                model: Cow::Owned(only(model, Vec::new())),
                top: None,
                note: Some(ph2d_i18n::tr("chrome.palette.no_symbols").to_owned()),
            };
        };
        let mine = only(model, vec![sym.clone()]);
        let rest = rest.trim_start();
        let m = if rest.is_empty() {
            mine
        } else {
            filter_model(&mine, rest)
        };
        // `@` sozinho: o Enter leva ao 1.º símbolo (o do topo do ficheiro).
        let top = top_match(&m, rest).or_else(|| {
            m.groups
                .first()
                .and_then(|g| g.subs.first())
                .and_then(|s| s.items.first())
                .map(|it| it.id)
        });
        return Scoped {
            model: Cow::Owned(m),
            top,
            note: None,
        };
    }
    if let Some(rest) = query.strip_prefix(':') {
        let Some(line) = &sc.line else {
            return Scoped {
                model: Cow::Owned(only(model, Vec::new())),
                top: None,
                note: Some(ph2d_i18n::tr("chrome.palette.no_lines").to_owned()),
            };
        };
        let Some((n, col)) = line_target(rest, line.lines) else {
            return Scoped {
                model: Cow::Owned(only(model, Vec::new())),
                top: None,
                note: Some(ph2d_i18n::tr_with(
                    "chrome.palette.line_prompt",
                    &[("n", &line.lines)],
                )),
            };
        };
        let label = match col {
            Some(c) => ph2d_i18n::tr_with("chrome.palette.goto_line_col", &[("line", &n), ("col", &c)]),
            None => ph2d_i18n::tr_with("chrome.palette.goto_line_n", &[("line", &n)]),
        };
        let group = PaletteGroup {
            title: ph2d_i18n::tr("chrome.palette.goto_line").to_owned(),
            color: ph2d_tokens::ColorToken::NodeCatSource,
            subs: vec![PaletteSub {
                title: None,
                items: vec![PaletteItem {
                    label,
                    id: line.id,
                    keywords: "",
                }],
            }],
        };
        return Scoped {
            model: Cow::Owned(only(model, vec![group])),
            top: Some(line.id),
            note: None,
        };
    }
    let mut s = plain(query);
    if !query.is_empty()
        && let Some(sym) = &sc.symbols
    {
        let found = filter_model(&only(model, vec![sym.clone()]), query);
        if !found.groups.is_empty() {
            let m = s.model.to_mut();
            m.groups.extend(found.groups);
            if s.top.is_none() {
                s.top = top_match(m, query);
            }
        }
    }
    s
}

/// O estado dos prefixos no store: o que se mostra enquanto a paleta está aberta e a busca do
/// último pick (o `:40` escolhido — a paleta já fechou quando o pick é lido).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ScopeState {
    pub scopes: Option<PaletteScopes>,
    pub pick_query: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_tokens::ColorToken;

    fn item(label: &str, n: u64) -> PaletteItem {
        PaletteItem {
            label: label.into(),
            id: NodeId(n),
            keywords: "",
        }
    }

    fn group(title: &str, items: Vec<PaletteItem>) -> PaletteGroup {
        PaletteGroup {
            title: title.into(),
            color: ColorToken::NodeCatSource,
            subs: vec![PaletteSub { title: None, items }],
        }
    }

    fn fixture() -> (PaletteModel, PaletteScopes) {
        let model = PaletteModel {
            title: "Commands".into(),
            groups: vec![group("This tab", vec![item("Save Script", 1), item("Find…", 2)])],
            toggle: None,
        };
        let scopes = PaletteScopes {
            symbols: Some(group("Symbols", vec![item("update", 10), item("speed", 11)])),
            line: Some(LineScope {
                id: NodeId(20),
                lines: 12,
            }),
        };
        (model, scopes)
    }

    fn labels(s: &Scoped<'_>) -> Vec<String> {
        s.model
            .groups
            .iter()
            .flat_map(|g| &g.subs)
            .flat_map(|s| &s.items)
            .map(|i| i.label.clone())
            .collect()
    }

    #[test]
    fn os_prefixos_estreitam() {
        let (m, sc) = fixture();
        let s = |q: &str| scoped(&m, q, Some(&sc));
        assert_eq!(labels(&s("")), ["Save Script", "Find…"], "vazio: só os comandos");
        assert_eq!(labels(&s("upd")), ["update"], "escrito: os símbolos também");
        assert_eq!(labels(&s(">")), ["Save Script", "Find…"]);
        assert_eq!(labels(&s(">upd")), Vec::<String>::new(), "`>` só comandos");
        assert_eq!(labels(&s("@")), ["update", "speed"]);
        assert_eq!((labels(&s("@sp")), s("@sp").top), (vec!["speed".to_owned()], Some(NodeId(11))));
        assert_eq!(s("@").top, Some(NodeId(10)), "`@` + Enter: o 1.º símbolo");
        assert_eq!(s(":3").top, Some(NodeId(20)));
        assert_eq!(s(":").top, None);
        assert!(s(":").note.is_some());
        assert_eq!(line_target("99", 12), Some((12, None)), "prende à última");
        assert_eq!(line_target("5:3", 12), Some((5, Some(3))));
        assert_eq!(line_target("x", 12), None);
        assert_eq!(labels(&scoped(&m, "@", None)), Vec::<String>::new(), "sem prefixos: a lei de sempre");
    }
}
