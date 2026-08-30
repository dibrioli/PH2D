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

/// **A fonte corrente publica o eixo `wght`?** — o FACTO de que a fileira *Weight* do painel
/// depende, e que ninguém publicava.
///
/// # A pergunta certa é a TERCEIRA, e as duas óbvias estão refutadas
///
/// Sem tabela `fvar` o `skrifa` ignora a localização de eixo (`font.axes().location(..)` sobre um
/// conjunto vazio devolve o default), então numa fonte ESTÁTICA o slider de Weight é **inerte** —
/// pintado, arrastável, e sem efeito nenhum na letra.
///
/// ⛔ **A cura óbvia — copiar a regra da secção AXES — está REFUTADA por medição.** A AXES some
/// com `names.is_empty()`, e `names` vem de [`variation_axes`], **que filtra o `wght` fora**. Numa
/// fonte variável **só de peso** — a espécie mais comum, e há uma nesta máquina
/// (`/usr/share/fonts/cantarell/Cantarell-VF.otf`, `fvar = ['wght']`) — aquela lista fica vazia, a
/// AXES esconde-se **com razão**, e a mesma regra esconderia um Weight **vivo e correcto**.
///
/// ⇒ *"há eixos ALÉM do peso?"* e *"há o eixo do peso?"* são duas perguntas, e o painel só tinha
/// resposta para a primeira. Esta é a segunda, e ela sai do `fvar` da fonte — nunca da lista que
/// já o excluiu.
#[must_use]
pub(crate) fn has_weight_axis(family: Option<&str>) -> bool {
    resolve(family)
        .axes()
        .iter()
        .any(|a| a.tag() == AxisTag::WEIGHT)
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

    /// Quantas famílias do sistema o gate abaixo parseia. Cada uma custa um parse (cacheado), e
    /// o que ele afirma é uma LEI sobre todas — não a existência de uma peça rara.
    const SCAN_LIMIT: usize = 120;

    /// **A lista de eixos NUNCA responde à pergunta do peso — e é essa a lei que faltava.**
    ///
    /// A cura óbvia para o *Weight* inerte era copiar a regra da secção AXES (`axes.is_empty()`).
    /// Este gate refuta-a sem precisar de presumir que existe uma fonte só-de-peso na máquina:
    /// sobre **toda** família do sistema, `variation_axes` é `axes()` MENOS o `wght`. Logo, numa
    /// fonte cujo único eixo é o peso a lista é **vazia** e o peso está **vivo** — as duas
    /// perguntas divergem por construção, e não por acaso de corpus.
    ///
    /// ⚠️ E ele afirma a metade que o painel não pode afirmar: [`has_weight_axis`] tem de ler o
    /// `fvar` da fonte, e não a lista já filtrada.
    #[test]
    fn the_extra_axes_list_can_never_answer_the_weight_question() {
        // A embutida: variável, COM peso, e o peso não entra na lista de extras.
        assert!(
            has_weight_axis(None),
            "a InterVariable embutida deixou de expor `wght` — ou a leitura do fvar partiu"
        );

        let mut scanned = 0usize;
        let mut statics = 0usize;
        let mut weight_only = 0usize;
        for name in pickable_families().iter().flatten().take(SCAN_LIMIT) {
            let axes = resolve(Some(name)).axes();
            let has_w = axes.iter().any(|a| a.tag() == AxisTag::WEIGHT);
            let extra = variation_axes(Some(name));
            assert_eq!(
                has_weight_axis(Some(name)),
                has_w,
                "`has_weight_axis` discorda do `fvar` de {name} — ele deixou de ler a fonte"
            );
            assert!(
                extra.iter().all(|a| a.tag != AxisTag::WEIGHT),
                "o `wght` de {name} vazou para a lista de eixos EXTRAS (ele tem slider próprio)"
            );
            assert_eq!(
                extra.len(),
                axes.len() - usize::from(has_w),
                "a lista de eixos extras de {name} não é `axes` menos o peso. Se ela passar a \
                 incluí-lo, a secção AXES duplica o Weight; se passar a tirar mais alguma coisa, \
                 um eixo real fica inalcançável."
            );
            if axes.is_empty() {
                statics += 1;
                assert!(
                    !has_weight_axis(Some(name)),
                    "{name} é ESTÁTICA e foi dada como tendo peso — o slider ficaria inerte"
                );
            }
            if has_w && axes.len() == 1 {
                weight_only += 1;
                assert!(
                    extra.is_empty() && has_weight_axis(Some(name)),
                    "{name} é variável SÓ no peso: a lista de extras tem de ser vazia (a AXES \
                     esconde-se) e o peso tem de continuar vivo"
                );
            }
            scanned += 1;
        }

        // A metade JUSTA: sem ela um sistema sem fontes passaria provando nada.
        assert!(
            scanned > 0,
            "nenhuma família do sistema foi parseada — este gate não mediu nada"
        );
        // A fixtura do DEFEITO tem de existir de facto. ⚠️ Ela é universal (toda instalação com
        // fontconfig/CoreText tem estáticas); se um dia não for, o gate diz-o em voz alta em vez
        // de ficar verde por vacuidade.
        assert!(
            statics > 0,
            "nenhuma das {scanned} famílias parseadas é ESTÁTICA — a fixtura do defeito não \
             existe nesta máquina, e o ramo `!has_weight_axis` nunca foi exercido"
        );
        eprintln!(
            "[vec_font] {scanned} famílias parseadas · {statics} estáticas · \
             {weight_only} variáveis só-de-peso"
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
