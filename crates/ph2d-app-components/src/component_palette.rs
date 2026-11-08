//! ⭐ **A paleta de COMPONENTES** — o que o `+` do Inspector abre (ADR-0166, plano F3).
//!
//! # Ela NÃO é um modal novo
//!
//! É o [`ph2d_editor_core::widget::command_palette`], que já é genérico por desenho: ele conhece só
//! um [`PaletteModel`] (título + categorias coloridas + itens com um `NodeId` opaco), e quem o
//! abriu mapeia o id de volta. Já tem scrim, cascata de entrada, busca com **um** predicado
//! servindo o filtro pintado *e* o `Enter`, sub-clusters e promoção a 2 colunas.
//!
//! ⇒ **o que este módulo constrói é o MODELO**, copiando os dois precedentes: a biblioteca de nós
//! do Motion (`render_loop::motion_bridge_library`) e o `Ctrl+K` global.
//!
//! # ⚠️ O filtro por TIPO DE OBJETO, e porque o inaplicável NÃO some
//!
//! Instrução do Enio (2026-08-24): *"nosso modal de objetos deve ter um filtro por tipo de objeto.
//! Exemplo: 9-slice provavelmente não se aplica a nada além de uma sprite de imagem."*
//!
//! O inaplicável fica sob **Show all**, **esmaecido e com a razão nomeada** — ⛔ nunca apagado da
//! lista e ⛔ nunca um no-op silencioso ao clique. *Um componente que existe e é invisível lê-se
//! como defeito; um que aceita o clique e não faz nada é a DIRETIVA §2 violada.*
//!
//! # Os 11 assuntos (plano GameEngineFuncional 25)
//!
//! Grupos, ordem, dicas e desenhos são DADOS em [`ph2d_game_dicas::componente`]; o catálogo
//! (`ComponentCategory`) não muda. O gesto é o do Add Object: 1 clique destaca, 2 escolhem.

use ph2d_component_desc::{Attach, ComponentDesc, ObjectKind, ObjectKinds};
use ph2d_editor_core::NodeId;
use ph2d_editor_core::widget::command_palette::{
    PaletteGroup, PaletteItem, PaletteModel, PaletteSelect, PaletteSub, PaletteTeach, PaletteTip,
};
use ph2d_game_dicas::componente::{self as cd, Assunto};
use ph2d_i18n::tr;

/// O id de item de um componente na paleta — o hash do **nome canónico**.
///
/// ⚠️ **O nome canónico, e não o de exibição:** é ele que o registo indexa, e é por ele que o pick
/// volta a ser um componente. Um rótulo muda quando o produto quiser; o nome canónico é o formato.
#[must_use]
pub const fn item_id(canonical_name: &'static str) -> NodeId {
    ph2d_tool_registry::hash_node_id(canonical_name)
}

/// **Um componente é oferecível?** Três condições, e cada uma barra um defeito diferente.
///
/// 1. **`Authored`** — o artista escolhe-o. `Intrinsic` chega pelo gesto que cria o objeto e
///    `Machinery` é posta por um sistema; oferecer qualquer uma seria oferecer o que não é dele.
/// 2. **Ainda não está lá** — anexar o que já existe é um clique que não faz nada.
/// 3. ⚠️ **A paleta consegue CONSTRUÍ-LO** (`insert_default`): sem `Default` não há valor inicial,
///    e a `Sprite` é exatamente esse caso. Um item que a paleta não consegue anexar não pode estar
///    na paleta — ver o gate `no_authored_component_is_unreachable` do catálogo.
#[must_use]
pub(crate) fn is_offerable(desc: &ComponentDesc, present: &[&str], can_build: bool) -> bool {
    matches!(desc.attach, Attach::Authored { .. })
        && can_build
        && !present.contains(&desc.canonical_name)
}

/// Os tipos de objeto em que este componente tem efeito — `None` quando ele não é `Authored`
/// (e aí não é oferta nenhuma).
fn applies_to(desc: &ComponentDesc) -> Option<ObjectKinds> {
    match desc.attach {
        Attach::Authored { applies_to } => Some(applies_to),
        Attach::Intrinsic | Attach::Machinery => None,
    }
}

/// ⭐ **As palavras de intenção** do componente (doc ObjectsAndComponents 01 §7.3, F6) — `""` quando a
/// tabela não as tem (o gate `every_offered_component_has_intent_words` diz se faltou alguma).
#[must_use]
pub(crate) fn intent_words(desc: &ComponentDesc) -> &'static str {
    let key = desc.keywords_key();
    let words = tr(&key);
    if words == key { "" } else { words }
}

/// Um item da paleta: o nome, a plaquinha `+N` quando traz peças junto (a lista vai na dica), a razão do inaplicável.
fn make_item(desc: &ComponentDesc, applicable: bool, ja: &[&str]) -> PaletteItem {
    let mut label = tr(desc.display_key).to_string();
    let n = cd::trazidos(desc.canonical_name, ja).len();
    let badge = (n > 0).then(|| tr("component_add.plus_n").replace("{n}", &n.to_string()));
    if !applicable {
        label.push_str(tr(
            "app.components.component_palette.not_for_this_object_type_2",
        ));
    }
    PaletteItem {
        badge,
        label,
        id: item_id(desc.canonical_name),
        keywords: intent_words(desc),
    }
}

/// A dica de uma entrada: a frase, as palavras da busca e o que vem junto.
fn make_tip(e: &cd::Entrada, title: &str, desc: Option<&ComponentDesc>, ja: &[&str]) -> PaletteTip {
    let mut body = e.tip.tr().to_string();
    if let Some(d) = desc {
        let words = intent_words(d);
        if !words.is_empty() {
            body.push('\n');
            body.push_str(&tr("component_add.tip.words").replace("{words}", words));
        }
        let brings: Vec<&str> = cd::trazidos(d.canonical_name, ja)
            .iter()
            .map(|(_, b)| cd::nome(b))
            .collect();
        if !brings.is_empty() {
            body.push('\n');
            body.push_str(
                &tr("component_add.tip.also_adds").replace("{names}", &brings.join(", ")),
            );
        }
    }
    PaletteTip {
        id: item_id(e.name),
        title: title.to_string(),
        body,
        picture: e.picture,
    }
}

/// ⭐ **O modelo da paleta para ESTE objeto**, nos 11 assuntos.
///
/// `kind` é o tipo do objeto selecionado; `present` os nomes canónicos que ele **já tem**; `can_build`
/// responde se o registo sabe construir o tipo; `show_all` é a caixa *Show all*. Com ela desligada o
/// inaplicável não aparece; ligada, aparece ESMAECIDO e com a razão.
pub(crate) fn build(
    kind: ObjectKind,
    present: &[&str],
    can_build: &dyn Fn(&str) -> bool,
    show_all: bool,
) -> PaletteModel {
    let mut groups = Vec::new();
    let mut tips = Vec::new();
    for a in Assunto::ALL {
        let (mut applicable, mut other) = (Vec::new(), Vec::new());
        for e in cd::ENTRADAS.iter().filter(|e| e.assunto == a) {
            let Some(d) = ph2d_component_desc::desc_for(e.name) else {
                continue;
            };
            if !is_offerable(d, present, can_build(d.canonical_name)) {
                continue;
            }
            let fits = applies_to(d).is_some_and(|k| k.contains(kind));
            if fits || show_all {
                tips.push(make_tip(e, tr(d.display_key), Some(d), present));
                if fits { &mut applicable } else { &mut other }.push(make_item(d, fits, present));
            }
        }
        // ⚠️ Os aplicáveis PRIMEIRO, sempre — o inaplicável é contexto, não oferta.
        let other_title = tr("app.components.component_palette.not_for_this_object_type");
        let subs: Vec<PaletteSub> = [(None, applicable), (Some(other_title.to_string()), other)]
            .into_iter()
            .filter(|(_, items)| !items.is_empty())
            .map(|(title, items)| PaletteSub { title, items })
            .collect();
        if !subs.is_empty() {
            groups.push(PaletteGroup {
                title: a.title().to_string(),
                color: a.color(),
                subs,
            });
        }
    }
    let mut model = PaletteModel {
        title: tr("app.components.component_palette.add_component").to_string(),
        groups,
        // ⭐ A caixa *Show all* (ADR-0166 / F3): o estado vive na shell; um clique reabre a paleta.
        toggle: Some(ph2d_editor_core::widget::command_palette::PaletteToggle {
            label: tr("app.components.component_palette.show_all").to_string(),
            on: show_all,
        }),
        tips,
        select: None,
    };
    reselect(&mut model, None);
    model
}

/// ⭐ **O destaque**: `keep` se o item ainda está no modelo, senão o 1.º item (dono 08/11) — e o ensino
/// da busca, só com começo que o modelo tem.
pub fn reselect(model: &mut PaletteModel, keep: Option<NodeId>) {
    let first = model
        .groups
        .iter()
        .flat_map(|g| &g.subs)
        .flat_map(|s| &s.items)
        .next();
    let Some(first) = first.map(|it| it.id) else {
        model.select = None;
        return;
    };
    let initial = keep.filter(|id| model.is_item(*id)).unwrap_or(first);
    let teach = cd::TEACH
        .iter()
        .filter(|(_, _, start)| model.is_item(item_id(start)))
        .filter_map(|(text, words, start)| {
            let d = ph2d_component_desc::desc_for(start)?;
            Some(PaletteTeach {
                words: words.tr(),
                text: text.tr().to_string(),
                start: item_id(d.canonical_name),
                start_label: tr("component_add.teach.start").replace("{name}", tr(d.display_key)),
            })
        })
        .collect();
    model.select = Some(PaletteSelect { initial, teach });
}

/// O nome canónico que este id de item nomeia, se algum. É o inverso do [`item_id`], e a única
/// forma de um pick voltar a ser um componente.
///
/// ⚠️ **Varre o CATÁLOGO, não uma lista à mão** — uma segunda lista aqui envelheceria no primeiro
/// componente novo, e o sintoma seria *"o item aparece e não faz nada"*.
/// ⭐ **O item «Animation»** (doc ObjectsAndComponents 01 §6) — não é um componente do MUNDO: a
/// Animação de um objecto é o conjunto dos clips dele na Timeline, então escolhê-lo pede à
/// Timeline o primeiro clip (a shell, por [`crate::component_attach::take_animation_pick`]).
pub const ANIMATION_ITEM: NodeId = item_id("timeline.animation");

/// Põe o item «Animation» à cabeça de *Change over time*, criando o grupo no lugar dele se nenhum
/// componente o abriu, com a dica; e refaz o destaque (pode passar a ser o 1.º).
pub(crate) fn offer_animation(model: &mut PaletteModel) {
    let label = tr("app.components.component_palette.animation");
    let item = PaletteItem {
        badge: None,
        label: label.to_string(),
        id: ANIMATION_ITEM,
        keywords: "",
    };
    if let Some(e) = cd::entrada(cd::ANIMATION) {
        model.tips.push(make_tip(e, label, None, &[]));
    }
    let a = Assunto::Time;
    let title = a.title();
    if let Some(g) = model.groups.iter_mut().find(|g| g.title == title) {
        match g.subs.first_mut() {
            Some(s) if s.title.is_none() => s.items.insert(0, item),
            _ => g.subs.insert(
                0,
                PaletteSub {
                    title: None,
                    items: vec![item],
                },
            ),
        }
    } else {
        let rank = |t: &str| Assunto::ALL.iter().position(|x| x.title() == t);
        let at = model
            .groups
            .iter()
            .position(|g| rank(&g.title) > rank(title));
        model.groups.insert(
            at.unwrap_or(model.groups.len()),
            PaletteGroup {
                title: title.to_string(),
                color: a.color(),
                subs: vec![PaletteSub {
                    title: None,
                    items: vec![item],
                }],
            },
        );
    }
    let keep = model.select.as_ref().map(|s| s.initial);
    reselect(model, keep);
}

#[must_use]
pub(crate) fn name_of_pick(id: NodeId) -> Option<&'static str> {
    ph2d_component_desc::all()
        .map(|d| d.canonical_name)
        .find(|n| item_id(n) == id)
}

#[cfg(test)]
#[path = "component_palette_tests.rs"]
mod tests;

/// ⭐ A busca por intenção (F6) — ver [`crate::component_palette_intent_tests`].
#[cfg(test)]
#[path = "component_palette_intent_tests.rs"]
mod intent_tests;

/// ⭐ A SONDA da paleta — ver [`crate::component_palette_measure`].
#[cfg(test)]
#[path = "component_palette_measure.rs"]
mod measure;

/// ⭐ O censo de alcance nos dois sentidos — ver [`crate::component_reach_tests`].
#[cfg(test)]
#[path = "component_reach_tests.rs"]
mod reach_tests;

/// ⭐ O censo «todo componente que a paleta oferece tem CARTÃO» — ver
/// [`crate::component_card_census_tests`].
#[cfg(test)]
#[path = "component_card_census_tests.rs"]
mod card_census_tests;
