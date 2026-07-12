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
use ph2d_vector_font::{AxisTag, VariableFont, VariableFontAxis};

/// Um eixo de variação de uma fonte (fora o `wght`, que tem slider próprio) — nome
/// legível + range + default. Alimenta a seção Axes do painel.
pub(crate) struct AxisDesc {
    pub tag: AxisTag,
    pub name: String,
    pub min: f32,
    pub max: f32,
    pub default: f32,
}

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
    /// Fontes IMPORTADAS de arquivo (nome de exibição → fonte). Independentes do
    /// scan do sistema; entram no ciclo antes das famílias do sistema.
    static IMPORTED: RefCell<BTreeMap<String, Arc<VariableFont>>> =
        const { RefCell::new(BTreeMap::new()) };
}

/// Roda `f` com o catálogo, construindo-o (escaneia o sistema) na 1ª vez.
fn with_db<R>(f: impl FnOnce(&mut FontDb) -> R) -> R {
    DB.with(|c| f(c.borrow_mut().get_or_insert_with(FontDb::new)))
}

/// Importa uma fonte de bytes de arquivo (`.ttf`/`.otf`) sob o rótulo `name`.
/// Devolve `Some(name)` se parseou (para virar a família corrente), `None` se os
/// bytes não são uma fonte válida. Não toca o `Collection` do sistema.
#[must_use]
pub(crate) fn import(name: String, bytes: Vec<u8>) -> Option<String> {
    let font = Arc::new(VariableFont::new(bytes).ok()?);
    IMPORTED.with(|m| m.borrow_mut().insert(name.clone(), font));
    Some(name)
}

/// A fonte de uma sessão de texto: a família escolhida (do sistema) ou a embutida.
/// Chamado a cada regen — barato após o 1º load (lookup + `Arc::clone`).
#[must_use]
pub(crate) fn resolve(family: Option<&str>) -> Arc<VariableFont> {
    let Some(name) = family else {
        return embedded();
    };
    // Importadas primeiro (não constroem o scan do sistema), depois o sistema.
    if let Some(f) = IMPORTED.with(|m| m.borrow().get(name).cloned()) {
        return f;
    }
    with_db(|db| db.load(name)).unwrap_or_else(embedded)
}

/// A lista selecionável de fontes, na ordem canônica `[Embutida] ++ importadas ++
/// famílias do sistema` (`None` = a embutida). É a MESMA ordem que o ciclo `<`/`>`
/// e o dropdown usam, então o índice publicado no preview casa com o que a shell
/// aplica. Constrói o catálogo do sistema na 1ª chamada (é quando o usuário abre o
/// seletor / o dropdown).
#[must_use]
pub(crate) fn pickable_families() -> Vec<Option<String>> {
    let imported: Vec<String> = IMPORTED.with(|m| m.borrow().keys().cloned().collect());
    with_db(|db| {
        let mut names: Vec<Option<String>> =
            Vec::with_capacity(1 + imported.len() + db.families.len());
        names.push(None);
        names.extend(imported.iter().cloned().map(Some));
        names.extend(db.families.iter().cloned().map(Some));
        names
    })
}

/// Cicla a família escolhida por `dir` (+1 próxima / −1 anterior) sobre
/// [`pickable_families`], com wrap. `None` = a entrada embutida.
#[must_use]
pub(crate) fn cycle_family(current: Option<&str>, dir: i32) -> Option<String> {
    let names = pickable_families();
    let cur = names
        .iter()
        .position(|n| n.as_deref() == current)
        .unwrap_or(0);
    let next = (cur as i32 + dir).rem_euclid(names.len() as i32) as usize;
    names[next].clone()
}

/// O rótulo a exibir para a família corrente (a embutida tem nome próprio).
#[must_use]
pub(crate) fn display_name(family: Option<&str>) -> String {
    family.map_or_else(|| "Inter (bundled)".to_owned(), str::to_owned)
}

/// Os eixos de variação de `family` ALÉM do peso (`wght`), na ordem que a fonte os
/// expõe — o que a seção Axes do painel mostra. Vazio para fontes estáticas ou que só
/// têm peso. Resolve a fonte (cacheada) e lê o `fvar`.
#[must_use]
pub(crate) fn variation_axes(family: Option<&str>) -> Vec<AxisDesc> {
    resolve(family)
        .axes()
        .iter()
        .filter(|a| a.tag() != AxisTag::WEIGHT)
        .map(|a| AxisDesc {
            tag: a.tag(),
            name: a.name().to_owned(),
            min: a.min(),
            max: a.max(),
            default: a.default(),
        })
        .collect()
}

/// A lista `(tag, default)` dos eixos extras de `family` — o estado inicial de
/// `VecTextEdit::extra_axes` / o default corrente da shell quando a família muda.
#[must_use]
pub(crate) fn seed_extra_axes(family: Option<&str>) -> Vec<(AxisTag, f32)> {
    variation_axes(family)
        .into_iter()
        .map(|a| (a.tag, a.default))
        .collect()
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

    /// A InterVariable embutida expõe o eixo `opsz` (Optical Size) além do peso — a
    /// seção Axes do painel mostra um campo pra ele. Trava a enumeração de eixos extras
    /// (sem `wght`, que tem slider próprio).
    #[test]
    fn the_bundled_font_exposes_the_optical_size_axis() {
        let axes = variation_axes(None);
        assert!(
            axes.iter().any(|a| a.tag == AxisTag::OPTICAL_SIZE),
            "a Inter embutida tem opsz"
        );
        assert!(
            axes.iter().all(|a| a.tag != AxisTag::WEIGHT),
            "o peso NAO entra nos eixos extras (tem slider proprio)"
        );
        // seed_extra_axes casa 1-a-1 com variation_axes, no default de cada eixo.
        let seed = seed_extra_axes(None);
        assert_eq!(seed.len(), axes.len());
        assert!(
            seed.iter()
                .zip(&axes)
                .all(|((t, v), d)| *t == d.tag && *v == d.default)
        );
    }

    /// Uma fonte importada resolve pelo nome e entra no ciclo antes das do sistema
    /// (ciclar +1 da embutida cai nela). Usa os bytes da embutida como arquivo-teste.
    #[test]
    fn an_imported_font_resolves_and_leads_the_cycle() {
        let name = import(
            "ZZImportTest".to_owned(),
            ph2d_text::inter_variable_ttf().to_vec(),
        )
        .expect("bytes da embutida parseiam");
        assert_eq!(name, "ZZImportTest");
        assert!(resolve(Some("ZZImportTest")).units_per_em() > 0);
        assert_eq!(cycle_family(None, 1), Some("ZZImportTest".to_owned()));
        assert!(
            import("bad".to_owned(), vec![0, 1, 2, 3]).is_none(),
            "lixo não parseia"
        );
    }
}
