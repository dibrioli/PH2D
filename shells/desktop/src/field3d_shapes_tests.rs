//! Os gates do CATÁLOGO de formas (W100) — ver [`super`].

use super::{Family, Make, SHAPES, available, shape_at, slot_of};

/// ⭐⭐⭐ **A CORRENTE QUE FECHA O BURACO: toda primitiva que o motor sabe fazer tem uma linha aqui.**
///
/// ⚠️ Este gate herdou o trabalho do `every_primitive_the_engine_can_make_has_a_button`, e a razão
/// dele é a W53: o `Extrude`/`Revolve` existiam **desde a W3** e nenhum botão os alcançava — uma
/// família de features inteira, completa e invisível. A corrente é
/// `Primitive` novo ⇒ erro de compilação em `Primitive::kind` ⇒ variante nova em `PrimitiveKind`
/// ⇒ `PrimitiveKind::ALL` não compila sem ela ⇒ **este laço reprova até haver a linha**.
#[test]
fn every_primitive_the_engine_can_make_is_in_the_catalogue() {
    for k in ph2d_field::PrimitiveKind::ALL {
        assert!(
            SHAPES.iter().any(|s| s.key.ends_with(k.key())),
            "a primitiva {k:?} não tem linha no catálogo - ela é inalcançável pela paleta"
        );
    }
}

/// ⭐⭐⭐ **A chave é a IDENTIDADE, então nenhuma se repete.**
///
/// ⚠️ Duas linhas com a mesma chave dariam **o mesmo id de item** na paleta, e o pick resolveria
/// sempre na primeira — a segunda forma seria pintada, clicável, e criaria a outra. É o modo de
/// falha mais caro que existe (parece funcionar), e com 60 linhas por vir uma colisão de
/// copiar-e-colar é o erro esperado, não o improvável.
#[test]
fn no_two_shapes_share_a_key() {
    for (i, a) in SHAPES.iter().enumerate() {
        for b in SHAPES.iter().skip(i + 1) {
            assert_ne!(a.key, b.key, "duas formas partilham a chave {}", a.key);
        }
    }
}

/// ⭐⭐ **O construtor da linha é o que decide, e não a posição** — a lei que a W100 comprou.
///
/// ⚠️ **Este é o gate que a W53 não podia ter.** Antes, as quatro entradas não-primitivas eram
/// alcançadas por `SHAPES.len() - 4`, `- 3`, `- 2`, `- 1`: acrescentar uma forma **no fim** fazia o
/// *Extrude* passar a abrir o diálogo de escultura, sem erro nenhum. Aqui a pergunta é sobre o
/// [`Make`] da própria linha, então a lista pode crescer em qualquer sítio.
#[test]
fn only_the_formula_shapes_build_from_a_radius() {
    for (slot, shape) in SHAPES.iter().enumerate() {
        let built = shape_at(slot, 0.5);
        match shape.make {
            Make::Formula(_) => assert!(
                built.is_some(),
                "{} diz-se de fórmula e não construiu nada",
                shape.key
            ),
            // ⚠️ Um contorno desenhado e um arquivo vivem **fora do mundo**: quem os trata é o
            // braço próprio, e um `Some` aqui seria uma forma nascida do nada no sítio errado.
            Make::Extrude | Make::Revolve | Make::Sculpt | Make::SculptScene => assert!(
                built.is_none(),
                "{} não sai de um raio - o `shape_at` tem de recusar",
                shape.key
            ),
        }
    }
}

/// ⭐ **Fora da lista não há forma nenhuma** — e é `None`, não a primeira.
#[test]
fn a_slot_past_the_catalogue_builds_nothing() {
    assert!(shape_at(SHAPES.len(), 0.5).is_none());
    assert!(slot_of("panel.model3d.add.nao_existe").is_none());
}

/// ⭐⭐ **A DISPONIBILIDADE é a lei da W34**, e cada `Make` responde por si.
///
/// ⚠️ O controlo é o que faz o gate valer: com as duas condições **desligadas**, as de fórmula
/// continuam disponíveis. Sem ele, um `available` que devolvesse sempre `false` passaria a metade
/// de cima e o defeito seria *"nenhum botão faz nada"*.
#[test]
fn only_what_needs_a_selection_waits_for_one() {
    for shape in SHAPES {
        let sempre = available(shape, false, false);
        match shape.make {
            Make::Formula(_) | Make::Sculpt => assert!(
                sempre,
                "{} não depende de nada e devia estar sempre disponível",
                shape.key
            ),
            Make::Extrude | Make::Revolve => {
                assert!(!sempre, "{} precisa de um contorno", shape.key);
                assert!(available(shape, false, true), "{} com contorno", shape.key);
            }
            Make::SculptScene => {
                assert!(!sempre, "{} precisa de escultura na cena", shape.key);
                assert!(available(shape, true, false), "{} com escultura", shape.key);
            }
        }
    }
}

/// ⭐ **Toda família da paleta tem título e tinta próprios.**
///
/// ⚠️ Duas famílias com a mesma tinta leem-se como uma só na paleta, e o título é o que separa os
/// grupos — é a cor que *ensina o mapa* do catálogo (a lição que a biblioteca do Motion registou).
#[test]
fn each_family_has_its_own_title_and_colour() {
    for (i, a) in Family::ALL.iter().enumerate() {
        for b in Family::ALL.iter().skip(i + 1) {
            assert_ne!(a.title(), b.title(), "{a:?} e {b:?} têm o mesmo título");
            assert_ne!(a.color(), b.color(), "{a:?} e {b:?} têm a mesma tinta");
        }
    }
}

/// ⭐⭐ **Uma forma nova nasce com o `round` que tem direito** — e não a zero.
///
/// ⚠️ Este é o módulo cujo argumento **é** o arredondamento: uma caixa de aresta viva ao nascer
/// esconde exatamente aquilo que ele faz melhor que o Blender. A propriedade é sobre a **família**
/// (toda forma que aceita `round` nasce com um), então ela vale para as 60 que vêm, não para as 4
/// que existem.
#[test]
fn every_new_shape_that_can_round_is_born_round() {
    for (slot, shape) in SHAPES.iter().enumerate() {
        let Some(prim) = shape_at(slot, 0.5) else {
            continue;
        };
        let Some(r) = ph2d_field::NodeShape::Leaf(prim).radius() else {
            // Sem `round` no modelo (esfera, toro) — a ausência é do documento, não desta lei.
            continue;
        };
        assert!(
            r > 0.0,
            "{} nasce de aresta viva - o módulo do arredondamento a esconder o que faz",
            shape.key
        );
    }
}
