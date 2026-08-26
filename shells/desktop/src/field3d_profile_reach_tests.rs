//! ⭐ **A FAMÍLIA DO PERFIL ALCANÇA O PAINEL?** — os gates de alcance das formas desenhadas
//! (W53), separados do isolamento com que tinham partilhado arquivo.
//!
//! ⚠️ A razão do corte é o teto de LOC do shell (HR-18), e a fronteira **não é arbitrária**: estes
//! gates perguntam *o painel oferece o que o motor sabe fazer?*, e os do arquivo irmão perguntam
//! *o isolamento diz-se e tem volta?*. Duas leis, dois arquivos.

/// ⭐⭐ **TODA FORMA QUE O MOTOR SABE FAZER TEM BOTÃO.**
///
/// # A lei que faltava, e por que a da W34 não a apanhava
///
/// `Primitive::Extrude` e `Primitive::Revolve` existem no motor **desde a W3**, medidos contra
/// oráculos independentes — e **nenhum botão os alcançava**: só as cenas de smoke os construíam. O
/// plano do módulo chama-lhes a razão de existir (*"é aqui que o fluxo do MoI renasce"*).
///
/// ⚠️ **A lei da W34 tem uma exclusão escrita** que os deixava de fora: a tabela dela cobre só as
/// fileiras que **dependem da seleção**, e as formas foram postas de lado como *"ações sempre
/// disponíveis"*. A pergunta certa para esta fileira é outra — *o painel oferece tudo o que o motor
/// sabe fazer?* — e a exclusão da outra lei escondia-a.
///
/// ⭐ A régua é o **construtor de nome** do documento (`ph2d_field_ecs::shape_name`), que é a lista
/// que o motor de facto tem: uma primitiva nova aparece aqui **sozinha**, no dia em que nascer.
#[test]
fn every_primitive_the_engine_can_make_has_a_button() {
    use crate::field3d_scene::panel::SHAPES;
    use ph2d_field::PrimitiveKind;
    // ⭐⭐⭐ **DERIVADA, e não escrita à mão** (2026-08-26).
    //
    // ⛔ Até aqui esta lista era literal — *«uma de cada, construída à mão: é a enumeração que o
    // `Primitive` não oferece»* — e a contagem no fim só a defendia **de si mesma**. O doc deste
    // gate promete que *«uma primitiva nova aparece aqui sozinha»* e isso era **falso**: um
    // `Primitive` novo compilava, ficava sem botão, e este gate ficava **verde**. Hoje a enumeração
    // existe ([`PrimitiveKind`]) e a corrente fecha no compilador.
    let mut seen: Vec<usize> = Vec::new();
    for k in PrimitiveKind::ALL {
        let slot = SHAPES.iter().position(|s| s.ends_with(k.key()));
        assert!(
            slot.is_some(),
            "o motor sabe fazer «{}» e o painel não oferece botão nenhum para ela — é uma feature \
             completa e invisível, que é o defeito que a W53 pagou",
            k.key()
        );
        // ⛔ **E um botão PRÓPRIO.** Sem esta metade, duas famílias com a mesma chave passavam: a
        // segunda encontrava o botão da primeira e o gate dizia que estava tudo alcançável — uma
        // prova de mutação mostrou-o.
        let slot = slot.expect("acabou de ser afirmado");
        assert!(
            !seen.contains(&slot),
            "«{}» aponta para o mesmo botão de outra família (slot {slot}) — duas formas a partilhar \
             um botão é uma delas inalcançável",
            k.key()
        );
        seen.push(slot);
    }
    // …e o controle: o painel não promete o que o motor não tem.
    assert_eq!(
        SHAPES.len(),
        PrimitiveKind::ALL.len() + 2,
        "o painel oferece formas a mais ou a menos — além das {} primitivas, só as DUAS esculturas",
        PrimitiveKind::ALL.len()
    );
}

/// ⭐ **Os dois botões de perfil só aparecem com um contorno FECHADO escolhido** — a lei da W34
/// aplicada à segunda família cuja disponibilidade não é constante.
#[test]
fn the_profile_buttons_appear_only_with_a_closed_outline_selected() {
    use crate::field3d_scene::panel::{EXTRUDE_SLOT, REVOLVE_SLOT, SHAPES, adds_for};
    let without = adds_for(false, false);
    assert!(
        !without
            .iter()
            .any(|c| c.key == SHAPES[EXTRUDE_SLOT] || c.key == SHAPES[REVOLVE_SLOT]),
        "sem contorno escolhido, «Extrude» e «Revolve» são botões que não têm o que extrudar"
    );
    let with = adds_for(false, true);
    assert!(
        with.iter().any(|c| c.key == SHAPES[EXTRUDE_SLOT])
            && with.iter().any(|c| c.key == SHAPES[REVOLVE_SLOT]),
        "com um contorno escolhido, os dois têm de aparecer"
    );
}

/// ⚠️ **Os quatro slots derivados não colidem** — os dois do perfil e os dois da escultura saem todos
/// de `SHAPES.len()`, e um `-3` trocado por um `-4` faria dois botões serem o mesmo. É a mesma
/// família de cerca que o `SCULPT_SLOT` já tinha, com o dobro dos membros.
#[test]
fn the_four_derived_slots_are_distinct_and_in_range() {
    use crate::field3d_scene::panel::{
        EXTRUDE_SLOT, REVOLVE_SLOT, SCULPT_SCENE_SLOT, SCULPT_SLOT, SHAPES,
    };
    let slots = [EXTRUDE_SLOT, REVOLVE_SLOT, SCULPT_SLOT, SCULPT_SCENE_SLOT];
    for (i, a) in slots.iter().enumerate() {
        assert!(*a < SHAPES.len(), "o slot {a} está fora da lista");
        for b in slots.iter().skip(i + 1) {
            assert_ne!(a, b, "dois slots derivados caíram no mesmo botão");
        }
    }
    // E cada um aponta para a chave que o nome dele promete.
    assert!(SHAPES[EXTRUDE_SLOT].ends_with("extrude"));
    assert!(SHAPES[REVOLVE_SLOT].ends_with("revolve"));
    assert!(SHAPES[SCULPT_SLOT].ends_with("sculpt"));
    assert!(SHAPES[SCULPT_SCENE_SLOT].ends_with("sculpt_scene"));
}
