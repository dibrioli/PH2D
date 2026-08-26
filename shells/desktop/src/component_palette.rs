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
//! # ⚠️ As cores são REAPROVEITADAS, e isso é uma pergunta em aberto para o Enio
//!
//! Há **12** categorias de componente ([`ComponentCategory::ALL`]) e **7** tokens `NodeCat*` no
//! design system. Este módulo mapeia 12 → 7 com a tabela abaixo, em vez de inventar cinco cores —
//! *escolher cor é decisão de design (§7 do CLAUDE.md), não de quem está a ligar o botão*. A
//! consequência visível é que pares de categorias partilham tinta; se isso incomodar no smoke, a
//! cura é acrescentar tokens ao `ph2d-tokens`, não escrever hex aqui (HR-15).

use ph2d_component_desc::{Attach, ComponentCategory as C, ComponentDesc, ObjectKind, ObjectKinds};
use ph2d_editor::NodeId;
use ph2d_editor::widget::command_palette::{PaletteGroup, PaletteItem, PaletteModel, PaletteSub};
use ph2d_tokens::ColorToken;

/// O id de item de um componente na paleta — o hash do **nome canónico**.
///
/// ⚠️ **O nome canónico, e não o de exibição:** é ele que o registo indexa, e é por ele que o pick
/// volta a ser um componente. Um rótulo muda quando o produto quiser; o nome canónico é o formato.
#[must_use]
pub(crate) const fn item_id(canonical_name: &'static str) -> NodeId {
    ph2d_tool_registry::hash_node_id(canonical_name)
}

/// A tinta de cada categoria. ⚠️ Ver a nota do módulo: 12 categorias, 7 tokens.
fn cat_token(c: C) -> ColorToken {
    match c {
        // O que todo objeto É.
        C::Identity | C::Transform => ColorToken::NodeCatSource,
        // Onde e em que ordem ele sai.
        C::Ordering => ColorToken::NodeCatDistribute,
        // Como o pixel sai.
        C::Rendering | C::Image => ColorToken::NodeCatFx,
        // O que se move.
        C::Animation | C::Anchors => ColorToken::NodeCatTransform,
        // Geometria autorada.
        C::Vector | C::Model3D => ColorToken::NodeCatFocus,
        // O que simula.
        C::Physics => ColorToken::NodeCatOutput,
        // O resto.
        C::Scripting | C::Instancing => ColorToken::NodeCatUtility,
    }
}

/// O rótulo em inglês de cada categoria (HR-15: a UI do app é em inglês).
fn cat_title(c: C) -> &'static str {
    match c {
        C::Identity => "Identity",
        C::Transform => "Transform",
        C::Ordering => "Ordering",
        C::Rendering => "Rendering",
        C::Image => "Image",
        C::Animation => "Animation",
        C::Anchors => "Anchors",
        C::Vector => "Vector",
        C::Physics => "Physics",
        C::Model3D => "3D",
        C::Scripting => "Scripting",
        C::Instancing => "Instancing",
    }
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

/// ⭐ **O que vem JUNTO com este componente, escrito para o artista ler ANTES de clicar.**
///
/// ⚠️ Esta é a correção da crítica medida ao Bevy (discussão #16570, doc 02 §1.4): *«não vejo o que
/// vem junto»*. A dependência automática cura o erro de setup e **cria** um problema de
/// visibilidade — e num editor a UI é o sítio barato de o resolver. Vazio quando não há cascata.
///
/// ⚠️ **Diz os nomes de EXIBIÇÃO, e a lista é FECHADA** (transitiva): anexar *Platform Player*
/// traz `RigidBody`, que traz `Collider` — e o artista tem de ver os dois, não o primeiro.
fn brings_along(desc: &ComponentDesc) -> Vec<&'static str> {
    let mut out: Vec<&'static str> = Vec::new();
    let mut stack: Vec<&'static str> = desc.requires.to_vec();
    while let Some(name) = stack.pop() {
        let Some(d) = ph2d_component_desc::desc_for(name) else {
            continue;
        };
        if out.contains(&d.display_name) {
            continue;
        }
        out.push(d.display_name);
        stack.extend_from_slice(d.requires);
    }
    out
}

/// Um item da paleta, com o rótulo já pronto.
fn make_item(desc: &ComponentDesc, applicable: bool) -> PaletteItem {
    // ⚠️ **A razão E a cascata viajam no RÓTULO**, e não em campos novos do widget genérico: o
    // `PaletteItem` é do `ph2d-editor-core` e serve três consumidores; acrescentar-lhe um
    // `disabled_reason` + um `brings` faria os outros dois carregar dois campos que não usam. O
    // esmaecido pertence ao widget; o *porquê* e o *o-que-vem-junto* pertencem a quem construiu o
    // modelo. E é a MESMA porta, o que é o ponto: um só sítio para tudo o que o item explica.
    let mut label = desc.display_name.to_string();
    let brings = brings_along(desc);
    if !brings.is_empty() {
        label.push_str("  \u{2014}  brings ");
        label.push_str(&brings.join(", "));
    }
    if !applicable {
        label.push_str("  \u{2014}  not for this object type");
    }
    PaletteItem {
        label,
        id: item_id(desc.canonical_name),
    }
}

/// ⭐ **O modelo da paleta para ESTE objeto.**
///
/// `kind` é o tipo do objeto selecionado (derivado do marcador — F0); `present` são os nomes
/// canónicos que ele **já tem**; `can_build` responde se o registo sabe construir aquele tipo;
/// `show_all` é a caixa *Show all* do artista.
///
/// ⚠️ **Com `show_all` desligado o inaplicável não aparece; com ele ligado aparece ESMAECIDO e com
/// a razão** — nunca desaparece sem explicação, e nunca aceita o clique em silêncio.
pub(crate) fn build(
    kind: ObjectKind,
    present: &[&str],
    can_build: &dyn Fn(&str) -> bool,
    show_all: bool,
) -> PaletteModel {
    let mut groups = Vec::new();
    for c in C::ALL {
        let mut applicable = Vec::new();
        let mut other = Vec::new();
        for d in ph2d_component_desc::all() {
            if d.category != c || !is_offerable(d, present, can_build(d.canonical_name)) {
                continue;
            }
            let fits = applies_to(d).is_some_and(|k| k.contains(kind));
            if fits {
                applicable.push(make_item(d, true));
            } else if show_all {
                other.push(make_item(d, false));
            }
        }
        if applicable.is_empty() && other.is_empty() {
            continue;
        }
        // ⚠️ Os aplicáveis PRIMEIRO, sempre — o inaplicável é contexto, não oferta.
        let mut subs = Vec::new();
        if !applicable.is_empty() {
            subs.push(PaletteSub {
                title: None,
                items: applicable,
            });
        }
        if !other.is_empty() {
            subs.push(PaletteSub {
                title: Some("Not for this object type".to_string()),
                items: other,
            });
        }
        groups.push(PaletteGroup {
            title: cat_title(c).to_string(),
            color: cat_token(c),
            subs,
        });
    }
    PaletteModel {
        title: "Add Component".to_string(),
        groups,
        // ⭐ **A caixa *Show all*** (ADR-0166 / F3) — o que revela o inaplicável, esmaecido e com a
        // razão. ⚠️ Ela é do MODELO e não do widget: o estado vive na shell (`AppGfx`), e um
        // clique nela reabre a paleta com o modelo reconstruído.
        toggle: Some(ph2d_editor::widget::command_palette::PaletteToggle {
            label: "Show all".to_string(),
            on: show_all,
        }),
    }
}

/// O nome canónico que este id de item nomeia, se algum. É o inverso do [`item_id`], e a única
/// forma de um pick voltar a ser um componente.
///
/// ⚠️ **Varre o CATÁLOGO, não uma lista à mão** — uma segunda lista aqui envelheceria no primeiro
/// componente novo, e o sintoma seria *"o item aparece e não faz nada"*.
#[must_use]
pub(crate) fn name_of_pick(id: NodeId) -> Option<&'static str> {
    ph2d_component_desc::all()
        .map(|d| d.canonical_name)
        .find(|n| item_id(n) == id)
}

#[cfg(test)]
#[path = "component_palette_tests.rs"]
mod tests;

/// ⭐ O censo de alcance nos dois sentidos — ver [`crate::component_reach_tests`].
#[cfg(test)]
#[path = "component_reach_tests.rs"]
mod reach_tests;
