//! Escolha de família de fonte para o texto vetorial (ADR-0108).
//!
//! Enumera as fontes do sistema (fontique, o mesmo backend que o parley do
//! `ph2d-text` usa) e resolve uma família em um [`VariableFont`] do motor de texto
//! vetorial. `None` = a fonte embutida (InterVariable), que NÃO toca o fontique —
//! assim o custo de escanear as fontes do sistema (50–200 ms) só paga quando o
//! usuário abre o seletor de fonte, nunca no startup nem ao digitar no default.
//!
//! Tudo `thread_local` (o editor é single-thread na main): o `Collection` não
//! precisa ser `Send`/`Sync`, e o cache família→`VariableFont` evita reparsar.

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::sync::Arc;

use fontique::{Collection, CollectionOptions, SourceCache};
use ph2d_vector_font::VariableFont;

/// A fonte embutida (InterVariable) como `Arc`, parseada 1× — o default (`family =
/// None`) e o fallback quando uma família do sistema não parseia. NÃO constrói o
/// `Collection` do fontique.
fn embedded() -> Arc<VariableFont> {
    thread_local! {
        static EMB: RefCell<Option<Arc<VariableFont>>> = const { RefCell::new(None) };
    }
    EMB.with(|c| {
        c.borrow_mut()
            .get_or_insert_with(|| {
                Arc::new(
                    VariableFont::new(ph2d_text::inter_variable_ttf().to_vec())
                        .expect("a InterVariable embutida sempre parseia"),
                )
            })
            .clone()
    })
}

/// O catálogo de fontes do sistema + cache de parse. Construído preguiçosamente na
/// 1ª chamada que precisa do sistema ([`families`] ou [`resolve`] de uma família).
struct FontDb {
    collection: Collection,
    cache: SourceCache,
    /// Nomes de família ordenados e deduplicados (o que o seletor cicla).
    families: Vec<String>,
    /// família → fonte parseada (`None` = a família não deu uma fonte usável).
    /// `BTreeMap` (não `HashMap`): determinismo, ADR-0022 / HR-5.
    loaded: BTreeMap<String, Option<Arc<VariableFont>>>,
}

impl FontDb {
    fn new() -> Self {
        let mut collection = Collection::new(CollectionOptions {
            system_fonts: true,
            ..Default::default()
        });
        let mut families: Vec<String> = collection.family_names().map(str::to_owned).collect();
        families.sort_unstable();
        families.dedup();
        Self {
            collection,
            cache: SourceCache::default(),
            families,
            loaded: BTreeMap::new(),
        }
    }

    /// Resolve uma família em um `VariableFont` (cacheado). Escolhe a fonte VARIÁVEL
    /// da família (tem eixos → o slider Weight funciona); senão a primeira. `None` se
    /// a família não existe, não carrega ou não parseia.
    fn load(&mut self, family: &str) -> Option<Arc<VariableFont>> {
        if let Some(cached) = self.loaded.get(family) {
            return cached.clone();
        }
        let parsed = self.collection.family_by_name(family).and_then(|fam| {
            let fonts = fam.fonts();
            let chosen = fonts
                .iter()
                .find(|f| !f.axes().is_empty())
                .or_else(|| fonts.first())?;
            let blob = chosen.load(Some(&mut self.cache))?;
            VariableFont::new(blob.data().to_vec()).ok().map(Arc::new)
        });
        self.loaded.insert(family.to_owned(), parsed.clone());
        parsed
    }
}

thread_local! {
    static DB: RefCell<Option<FontDb>> = const { RefCell::new(None) };
}

/// Roda `f` com o catálogo, construindo-o (escaneia o sistema) na 1ª vez.
fn with_db<R>(f: impl FnOnce(&mut FontDb) -> R) -> R {
    DB.with(|c| f(c.borrow_mut().get_or_insert_with(FontDb::new)))
}

/// A fonte de uma sessão de texto: a família escolhida (do sistema) ou a embutida.
/// Chamado a cada regen — barato após o 1º load (lookup + `Arc::clone`).
#[must_use]
pub(crate) fn resolve(family: Option<&str>) -> Arc<VariableFont> {
    match family {
        None => embedded(),
        Some(name) => with_db(|db| db.load(name)).unwrap_or_else(embedded),
    }
}

/// Cicla a família escolhida por `dir` (+1 próxima / −1 anterior) sobre a lista
/// `[Embutida] ++ famílias`, com wrap. `None` = a entrada embutida. Constrói o
/// catálogo na 1ª chamada (é quando o usuário abre o seletor).
#[must_use]
pub(crate) fn cycle_family(current: Option<&str>, dir: i32) -> Option<String> {
    with_db(|db| {
        let len = db.families.len() + 1; // +1 = a entrada embutida (índice 0)
        let cur = match current {
            None => 0,
            Some(name) => db
                .families
                .iter()
                .position(|f| f == name)
                .map_or(0, |i| i + 1),
        };
        let next = (cur as i32 + dir).rem_euclid(len as i32) as usize;
        (next != 0).then(|| db.families[next - 1].clone())
    })
}

/// O rótulo a exibir para a família corrente (a embutida tem nome próprio).
#[must_use]
pub(crate) fn display_name(family: Option<&str>) -> String {
    family.map_or_else(|| "Inter (bundled)".to_owned(), str::to_owned)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A embutida resolve sem tocar o fontique (não constrói o `Collection`).
    #[test]
    fn resolve_none_is_the_embedded_font() {
        assert!(resolve(None).units_per_em() > 0);
    }

    /// O fontique enumera as fontes do sistema neste ambiente (Linux/fontconfig).
    /// Se falhar (0 famílias), a escolha de fonte estaria morta — vale travar.
    #[test]
    fn the_system_exposes_font_families() {
        let n = with_db(|db| db.families.len());
        assert!(
            n > 0,
            "fontique deveria enumerar fontes do sistema (achou {n})"
        );
    }

    /// Ciclar +1 da embutida e −1 de volta retorna à embutida (o índice 0 da lista
    /// `[Embutida] ++ famílias`). Não entra em pânico com/sem fontes.
    #[test]
    fn cycling_round_trips_back_to_embedded() {
        let next = cycle_family(None, 1);
        assert_eq!(cycle_family(next.as_deref(), -1), None);
    }

    /// Uma família enumerada resolve num `VariableFont` usável (ou cai na embutida —
    /// nunca em pânico). Cobre o caminho de load/parse real.
    #[test]
    fn a_system_family_resolves_to_a_usable_font() {
        if let Some(fam) = cycle_family(None, 1) {
            assert!(resolve(Some(&fam)).units_per_em() > 0);
        }
    }
}
