//! As famílias de um `<text>` resolvidas como o Chromium as resolve no Linux (o oráculo da W18):
//! um nome instalado vale por si; um ausente vale pelo GÉMEO de métricas instalado (Arial →
//! Liberation Sans) ou pelo nome alternativo do Blink (Helvetica → Arial); as genéricas são os nomes
//! das preferências do Chromium (serif = Times New Roman, sans-serif = Arial) e a `monospace` é a do
//! fontconfig; sem nenhuma, a fonte padrão (a de `serif`). Medido no oráculo: t02/t03 (ausente e sem
//! família) e t04 (as genéricas) — doc `docs/MiroClone/09_texto_dos_svg.md`.

use std::collections::BTreeMap;
use std::sync::{Mutex, OnceLock, PoisonError};

use parley::fontique::{Collection, GenericFamily};
use parley::{FontContext, LayoutContext};
use svgtypes::FontFamily as Css;

/// O nome por trás de cada genérica nas preferências do Chromium no Linux.
const SERIF: &str = "Times New Roman";
const SANS: &str = "Arial";
const CURSIVE: &str = "Comic Sans MS";
const FANTASY: &str = "Impact";

/// Famílias de MÉTRICAS iguais: o Chromium aceita uma instalada pela outra pedida.
const METRIC_TWINS: &[&[&str]] = &[
    &["Arial", "Arimo", "Liberation Sans"],
    &["Times New Roman", "Tinos", "Liberation Serif"],
    &["Courier New", "Cousine", "Liberation Mono"],
    &["Symbol", "Symbol Neu"],
    &["Cambria", "Caladea"],
    &["Calibri", "Carlito"],
];

/// O nome que o Blink tenta quando o pedido não existe.
const ALTERNATES: &[(&str, &str)] = &[
    ("Arial", "Helvetica"),
    ("Helvetica", "Arial"),
    ("Courier", "Courier New"),
    ("Courier New", "Courier"),
    ("Times", "Times New Roman"),
    ("Times New Roman", "Times"),
];

/// Uma lista `font-family` resolvida: as famílias instaladas a usar, por ordem, e as pedidas que o
/// sistema não tem e que vinham ANTES da usada (as que o autor queria ver).
#[derive(Clone, Debug, Default)]
pub(crate) struct Resolved {
    pub names: Vec<String>,
    pub missing: Vec<String>,
}

/// O moldador partilhado do processo: a colecção de fontes do sistema custa a abrir (fontconfig) e
/// abre-se UMA vez; as listas resolvidas guardam-se pelo texto do `font-family`.
pub(crate) struct Shaper {
    pub fcx: FontContext,
    pub lcx: LayoutContext<()>,
    resolved: BTreeMap<String, Resolved>,
}

/// Corre `f` com o moldador do processo (um de cada vez).
pub(crate) fn with_shaper<R>(f: impl FnOnce(&mut Shaper) -> R) -> R {
    static SHAPER: OnceLock<Mutex<Shaper>> = OnceLock::new();
    let m = SHAPER.get_or_init(|| {
        Mutex::new(Shaper {
            fcx: FontContext::new(),
            lcx: LayoutContext::new(),
            resolved: BTreeMap::new(),
        })
    });
    f(&mut m.lock().unwrap_or_else(PoisonError::into_inner))
}

impl Shaper {
    /// A lista `font-family` (o valor CSS cru; vazio = nenhuma) resolvida.
    pub(crate) fn resolve(&mut self, family: &str) -> Resolved {
        if let Some(r) = self.resolved.get(family) {
            return r.clone();
        }
        let list = svgtypes::parse_font_families(family).unwrap_or_default();
        let r = resolve_list(&mut self.fcx.collection, &list);
        self.resolved.insert(family.to_owned(), r.clone());
        r
    }

    /// A família instalada que um nome pedido dá (ele, o gémeo ou o alternativo), se alguma.
    #[cfg(test)]
    pub(crate) fn installed(&mut self, name: &str) -> Option<String> {
        named(&mut self.fcx.collection, name)
    }
}

fn resolve_list(col: &mut Collection, list: &[Css]) -> Resolved {
    let mut r = Resolved::default();
    for f in list {
        let found = match f {
            Css::Named(n) => named(col, n),
            Css::Serif => named(col, SERIF).or_else(|| generic(col, GenericFamily::Serif)),
            Css::SansSerif => named(col, SANS).or_else(|| generic(col, GenericFamily::SansSerif)),
            Css::Monospace => generic(col, GenericFamily::Monospace),
            Css::Cursive => named(col, CURSIVE),
            Css::Fantasy => named(col, FANTASY),
        };
        match (found, f) {
            (Some(n), _) if !r.names.contains(&n) => r.names.push(n),
            (None, Css::Named(n)) if r.names.is_empty() && !r.missing.contains(n) => {
                r.missing.push(n.clone());
            }
            _ => {}
        }
    }
    if r.names.is_empty() {
        r.names.extend(
            named(col, SERIF)
                .or_else(|| generic(col, GenericFamily::Serif))
                .or_else(|| generic(col, GenericFamily::SansSerif)),
        );
    }
    r
}

/// Um nome pedido → a família instalada: ele, o gémeo de métricas, ou o alternativo (e o gémeo dele).
fn named(col: &mut Collection, name: &str) -> Option<String> {
    exact_or_twin(col, name).or_else(|| {
        let alt = ALTERNATES
            .iter()
            .find(|(a, _)| a.eq_ignore_ascii_case(name))?
            .1;
        exact_or_twin(col, alt)
    })
}

fn exact_or_twin(col: &mut Collection, name: &str) -> Option<String> {
    exact(col, name).or_else(|| {
        let class = METRIC_TWINS
            .iter()
            .find(|c| c.iter().any(|n| n.eq_ignore_ascii_case(name)))?;
        class.iter().find_map(|n| exact(col, n))
    })
}

fn exact(col: &mut Collection, name: &str) -> Option<String> {
    let id = col.family_id(name)?;
    col.family(id)?;
    col.family_name(id).map(str::to_owned)
}

/// A 1.ª família que o fontconfig dá a uma genérica.
fn generic(col: &mut Collection, g: GenericFamily) -> Option<String> {
    let ids: Vec<_> = col.generic_families(g).collect();
    ids.into_iter().find_map(|id| {
        col.family(id)?;
        col.family_name(id).map(str::to_owned)
    })
}
