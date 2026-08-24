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
    // Uma de cada, construída à mão: é a enumeração que o `Primitive` não oferece.
    let all = [
        (
            "box",
            ph2d_field::Primitive::Box {
                half: [0.1; 3],
                round: 0.0,
            },
        ),
        ("sphere", ph2d_field::Primitive::Sphere { radius: 0.1 }),
        (
            "cylinder",
            ph2d_field::Primitive::Cylinder {
                radius: 0.1,
                half_height: 0.1,
                round: 0.0,
            },
        ),
        (
            "torus",
            ph2d_field::Primitive::Torus {
                major: 0.2,
                minor: 0.05,
            },
        ),
        (
            "extrude",
            ph2d_field::Primitive::Extrude {
                profile: a_square(),
                half_height: 0.1,
                round: 0.0,
            },
        ),
        (
            "revolve",
            ph2d_field::Primitive::Revolve {
                profile: a_square(),
            },
        ),
    ];
    for (key, _) in &all {
        assert!(
            SHAPES.iter().any(|s| s.ends_with(key)),
            "o motor sabe fazer «{key}» e o painel não oferece botão nenhum para ela — é uma \
             feature completa e invisível"
        );
    }
    // …e o controle: o painel não promete o que o motor não tem.
    assert_eq!(
        SHAPES.len(),
        all.len() + 2,
        "o painel oferece formas a mais ou a menos — além das {} primitivas, só as DUAS esculturas",
        all.len()
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

fn a_square() -> ph2d_field::Profile {
    ph2d_field::Profile::new(
        vec![vec![[-0.1, -0.1], [0.1, -0.1], [0.1, 0.1], [-0.1, 0.1]]],
        ph2d_field::FillRule::NonZero,
        1.0e-3,
    )
    .expect("um quadrado é um perfil")
}
