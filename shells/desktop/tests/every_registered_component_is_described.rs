//! **O CENSO de dois lados do catálogo de descritores** (ADR-0166 · plano F0).
//!
//! O `ph2d-component-desc` é side-metadata chaveada pelo **nome canónico** — foi essa a
//! escolha que evitou tocar nos 109 sítios de chamada em 5 crates (DIRETRIZ §1.5.2.1:
//! projete foundational novo para ISOLAMENTO). O preço dessa escolha é a **deriva
//! silenciosa**: renomeie um componente, esqueça o catálogo, e o descritor deixa de ser
//! encontrado sem que nada falhe — a mesma classe do *"componente não registado é
//! DESCARTADO em silêncio"* que o `registry.rs` já avisa.
//!
//! Este arquivo é o que paga esse preço, e **tem de viver aqui**: é a shell que conhece os
//! cinco registradores. Nenhuma crate sozinha vê o registo completo, então um censo dentro
//! de qualquer uma delas seria verdadeiro por vacuidade sobre os componentes das outras
//! ([memória: gate que VARRE uma árvore vive onde a REGRA mora](../../../project-memory/feedback_a_tree_scanning_gate_is_never_reached_by_a_name_filter.md)).
//!
//! ⚠️ **Os dois lados são obrigatórios.** Só *"todo registado tem descritor"* deixa passar
//! um descritor órfão (o tipo foi removido, a linha do catálogo ficou); só *"todo descritor
//! nomeia um registado"* deixa passar um componente novo sem descritor — que é o caso comum
//! e o que mais dói, porque a paleta simplesmente não o mostra.

use ph2d_component_desc::{Attach, ComponentDesc, desc_for};
use ph2d_ecs::scene::{ComponentRegistry, register_ecs_components};

/// O registo **completo** — os CINCO registradores.
///
/// ⚠️ **Deliberadamente diferente do boot.** O `init.rs` chama quatro
/// (`register_script_components` fica de fora, e a consequência está escrita no
/// `catalog/script.rs`: o `LuauScript` não é salvo nem desfeito). O censo mede os tipos que
/// **existem**, não os que hoje arrancam — senão o dia em que alguém ligar o quinto
/// registador o catálogo estaria em falta, e o gate teria estado verde até lá.
fn full_registry() -> ComponentRegistry {
    let mut reg = ComponentRegistry::new();
    register_ecs_components(&mut reg);
    ph2d_render::register_render_components(&mut reg);
    ph2d_script::register_script_components(&mut reg);
    ph2d_physics_ecs::register_physics_components(&mut reg);
    ph2d_field_ecs::register_field_components(&mut reg);
    reg
}

/// **Lado A — todo tipo registado tem descritor.**
///
/// Um componente novo sem linha no catálogo é invisível para a paleta do `+` e para o
/// gating de seção do Inspector: ele existe, salva, desfaz — e o artista não tem como o
/// acrescentar nem como saber que existe.
#[test]
fn every_registered_component_has_a_descriptor() {
    let reg = full_registry();
    let missing: Vec<&str> = reg
        .iter()
        .filter(|e| e.desc.is_none())
        .map(|e| e.canonical_name)
        .collect();
    assert!(
        missing.is_empty(),
        "{} componente(s) registado(s) sem descritor em `ph2d-component-desc`: {:?}\n\
         Acrescente a linha na familia certa de `crates/ph2d-component-desc/src/catalog/` \
         (a lista fica ORDENADA por canonical_name).",
        missing.len(),
        missing,
    );
}

/// **Lado B — todo descritor nomeia um tipo registado.**
///
/// O órfão ao contrário: o tipo saiu do registo (removido, renomeado) e a linha do catálogo
/// ficou. Ela não falha nada — só descreve algo que não existe, e a paleta oferece um
/// componente que ninguém consegue anexar.
#[test]
fn every_descriptor_names_a_registered_component() {
    let reg = full_registry();
    let orphans: Vec<&str> = ph2d_component_desc::all()
        .map(|d| d.canonical_name)
        .filter(|n| reg.get_by_name(n).is_none())
        .collect();
    assert!(
        orphans.is_empty(),
        "{} descritor(es) sem tipo registado: {:?}\n\
         Ou o componente foi removido/renomeado (apague ou corrija a linha do catalogo), \
         ou o registador dele deixou de ser chamado.",
        orphans.len(),
        orphans,
    );
}

/// ⭐ **A lei que o compilador ENSINOU, e por isso existe.**
///
/// A primeira versão do `Attach` tinha duas casas — `Authored` e `Machinery`. Ao converter
/// os registadores para `register_default`, o compilador devolveu **27 dos 109 tipos sem
/// `Default`**, e **17 deles estavam marcados `Authored`**: a paleta oferecê-los-ia e não
/// os conseguiria construir, porque anexar é inserir o **ponto neutro do tipo**. Daí a
/// terceira casa (`Attach::Intrinsic` — dado do artista que chega com o gesto) e daí este
/// gate, que é o que impede a lição de se perder.
///
/// ⚠️ O inverso **não** é lei: um `Intrinsic` ou um `Machinery` PODE ter `Default` (o
/// `RootOrder` tem). O que não pode existir é a promessa sem o meio de a cumprir.
#[test]
fn every_offered_component_can_be_constructed() {
    let reg = full_registry();
    let broken: Vec<&str> = reg
        .iter()
        .filter(|e| {
            e.desc.map(ComponentDesc::is_offered).unwrap_or(false) && e.insert_default.is_none()
        })
        .map(|e| e.canonical_name)
        .collect();
    assert!(
        broken.is_empty(),
        "{} componente(s) OFERECIDO(S) na paleta sem `insert_default`: {:?}\n\
         A paleta insere o ponto NEUTRO do tipo. Ou o tipo ganha `Default` e o registador \
         passa a `register_default::<T>`, ou o descritor dele e' `Attach::Intrinsic` \
         (dado que chega com o gesto) — nunca `Authored`.",
        broken.len(),
        broken,
    );
}

/// **Máquina não tem seção, e um `Machinery` com campos descritos é uma contradição.**
///
/// As quatro pontes de identidade (`VecPathRef` · `PaintedDoc` · `BakedForm` ·
/// `FlipObjectRef`) são ids opacos: descrever campos delas seria construir a tabela que
/// nenhum painel lê. O gate protege a distinção que a terceira variante comprou —
/// `Intrinsic` **pode** ter seção (a `Sprite` tem a maior de todas), `Machinery` não.
///
/// ⚠️ **Ele PARECE tautológico e não é** — e isto está escrito porque a primeira prova de
/// mutação falhou por culpa da mutação, não do gate. O construtor
/// `ComponentDesc::machinery` fixa `fields: &[]`, então nenhuma **chamada** o pode violar:
/// trocar `machinery(..)` por `intrinsic(.., &[])` não o mata, e lido depressa isso lê-se
/// como *"o gate é cego"*. A lei só quebra por **literal de struct** (os campos de
/// `ComponentDesc` são `pub`, e o `core.rs` foi escrito assim na primeira versão) — e com
/// essa mutação ele MATA. *Um construtor que impede o estado inválido não dispensa o gate:
/// ele muda qual é a mutação que o prova.*
#[test]
fn machinery_declares_no_fields() {
    let offenders: Vec<&str> = ph2d_component_desc::all()
        .filter(|d| matches!(d.attach, Attach::Machinery) && !d.fields.is_empty())
        .map(|d| d.canonical_name)
        .collect();
    assert!(
        offenders.is_empty(),
        "Machinery com campos descritos: {:?} — ou ele tem seccao (e e' Intrinsic), \
         ou os campos nao tem leitor.",
        offenders,
    );
}

/// **O descritor que o registo entrega é o MESMO que a busca por nome entrega.**
///
/// Duas portas para a mesma pergunta — `entry.desc` (resolvido uma vez no `register`) e
/// `desc_for(name)` (a busca binária). Se divergirem, o Inspector e a paleta passam a ler
/// tabelas diferentes, e o sintoma seria uma seção que aparece para um e não para o outro.
#[test]
fn the_registry_and_the_lookup_agree() {
    let reg = full_registry();
    for e in reg.iter() {
        let by_lookup = desc_for(e.canonical_name);
        assert_eq!(
            e.desc.map(|d| d.canonical_name),
            by_lookup.map(|d| d.canonical_name),
            "'{}': o descritor do registo e o da busca divergem",
            e.canonical_name,
        );
    }
}
