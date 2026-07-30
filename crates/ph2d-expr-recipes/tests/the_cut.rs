//! **O CORTE do catálogo — 50 → 31, e nenhuma palavra do artista morreu com ele**
//! (FASE A do plano 12).
//!
//! O Enio reprovou o catálogo e a ordem foi *"eliminar as expressões similares umas às
//! outras"*. A auditoria de 2026-07-29 mediu a matriz de redundância (doc 13 §3) e o corte
//! saiu dela; estes gates são a metade que garante que cortar não foi **esconder**.
//!
//! ⚠️ **Porque isto é um arquivo de gates e não uma nota:** a jornada anterior cortou cinco
//! receitas e herdou os SINÔNIMOS delas, mas não os **RÓTULOS** — medido, `"ramp"`,
//! `"ramp loop"` e `"sway cosine"` davam **zero hits** (doc 13 §7.3), ou seja três nomes que
//! o artista tinha aprendido apontavam para o vazio. O §3.3 do plano proibia exactamente
//! isso (*"cortar sem herdar é esconder capacidade"*) e nada o verificava.

use ph2d_expr_recipes::{Answer, CATALOG, Family, RETIRED, RecipeStack, by_id, search};
use ph2d_expr_recipes::{RowKind, SearchHit};

/// **O rótulo de toda receita aposentada ainda encontra a resposta dela.**
///
/// O gate que faltava, palavra por palavra. Uma aposentada com `Survivor` tem de achar o
/// sobrevivente; uma com `Refusal` tem de achar a recusa que a roteia — e nenhuma das duas
/// pode devolver nada.
///
/// **Mutação que deve sangrar:** apagar os apelidos herdados de qualquer sobrevivente
/// (ex.: tirar `"turbulence"` dos aliases do `shake`).
#[test]
fn every_retired_label_still_finds_its_answer() {
    for r in RETIRED {
        let hits = search(r.label);
        let landed = hits.iter().any(|h| match (h, r.answer) {
            (SearchHit::Recipe(rec), Answer::Survivor(id)) => rec.id == id,
            (SearchHit::Refusal(rf), Answer::Refusal(key)) => rf.key == key,
            _ => false,
        });
        assert!(
            landed,
            "digitar {:?} (a receita {:?}, aposentada) tem de achar {:?} — cortar sem \
             herdar é esconder capacidade.\n  achou: {:?}\n  porque saiu: {}",
            r.label,
            r.id,
            r.answer,
            hits.iter()
                .map(|h| match h {
                    SearchHit::Recipe(rec) => rec.id,
                    SearchHit::Refusal(rf) => rf.key,
                })
                .collect::<Vec<_>>(),
            r.why,
        );
    }
}

/// **Nenhuma aposentada aponta para um card que também já saiu.**
///
/// ⚠️ A aposentadoria em CADEIA é real e aconteceu: o `mirror` foi cortado apontando para o
/// `opposite`, e a FASE A cortou o `opposite`. Sem este gate a tabela mandaria o artista a
/// um card que não existe, e o gate acima passaria (o `search` acharia *alguma* coisa).
///
/// **Mutação que deve sangrar:** apontar o `mirror` de volta para `"opposite"`.
#[test]
fn every_survivor_is_a_recipe_that_still_exists() {
    for r in RETIRED {
        match r.answer {
            Answer::Survivor(id) => assert!(
                by_id(id).is_some(),
                "{:?} aponta para {id:?}, que também foi aposentada — resolva a CADEIA até \
                 o sobrevivente vivo",
                r.id
            ),
            Answer::Refusal(key) => assert!(
                ph2d_expr_recipes::REFUSALS.iter().any(|f| f.key == key),
                "{:?} aponta para a recusa {key:?}, que não existe",
                r.id
            ),
        }
    }
}

/// **Uma aposentada não está no catálogo, e uma do catálogo não está aposentada.**
///
/// As duas metades, porque a tabela e o catálogo são duas listas e a única coisa pior que
/// uma receita cortada sem registro é um registro de uma receita que continua lá.
#[test]
fn the_catalog_and_the_retired_table_do_not_overlap() {
    for r in RETIRED {
        assert!(
            by_id(r.id).is_none(),
            "{:?} está listada como aposentada e continua no catálogo",
            r.id
        );
    }
    for rec in CATALOG {
        assert!(
            !RETIRED.iter().any(|r| r.id == rec.id),
            "{:?} está no catálogo e na tabela de aposentadas",
            rec.id
        );
    }
}

/// **O Shake absorveu o Turbulence sem mover o próprio default, ao BIT.**
///
/// O contrato do parser diz que `octaves = 1` é byte-idêntico ao lowering de uma oitava.
/// Este gate não confia nessa frase: ele compara o TEXTO que o Shake emite hoje com o
/// `wiggle` de dois argumentos, avaliado.
///
/// **Mutação que deve sangrar:** trocar o default do `detail` para 3 (o do turbulence).
#[test]
fn absorbing_turbulence_left_shakes_default_alone() {
    use ph2d_expr::{Bindings, eval};
    use ph2d_expr_parse::parse;

    let now = RecipeStack::of(&["shake"]).to_formula();
    assert!(
        now.contains("wiggle(2, 0.3, 1, 0.5)"),
        "o Shake tem de emitir a forma de QUATRO argumentos com Detail 1: {now}"
    );

    struct B(f32);
    impl Bindings for B {
        fn attr(&self, name: &str) -> f32 {
            match name {
                "time" => self.0,
                "value" => 0.0,
                "__seed" => 100.0,
                _ => 0.0,
            }
        }
        fn param(&self, _: &str) -> f32 {
            0.0
        }
    }
    let four = parse(&now).expect("o catálogo emite texto que o parser aceita");
    let two = parse("value + wiggle(2, 0.3)").expect("a forma de dois argumentos");
    for i in 0..240 {
        let t = i as f32 / 240.0 * 2.0;
        let (a, b) = (eval(&four, &B(t)), eval(&two, &B(t)));
        assert_eq!(
            a, b,
            "Detail 1 tem de ser byte-idêntico ao wiggle de duas oitavas... de dois \
             ARGUMENTOS, em t = {t}"
        );
    }

    // ...e a capacidade do turbulence continua alcançável: Detail 3 muda o desenho.
    let mut row = ph2d_expr_recipes::Row::new("shake").expect("shake existe");
    row.set("detail", ph2d_expr_recipes::KnobValue::Num(3.0));
    let mut stack = RecipeStack::new();
    stack.push(row);
    let detailed = parse(&stack.to_formula()).expect("parseia");
    let moved = (0..240).any(|i| {
        let t = i as f32 / 240.0 * 2.0;
        (eval(&detailed, &B(t)) - eval(&four, &B(t))).abs() > 1e-4
    });
    assert!(
        moved,
        "subir o Detail tem de mudar o tremor — senão o card do turbulence foi cortado E a \
         capacidade dele com ele"
    );
}

/// **O Follow expressa as duas receitas de Link que saíram.**
///
/// ⚠️ Nada foi absorvido aqui, e é isso que o gate mostra: o Follow **já tinha** Multiply e
/// Offset, então ele contém `offset-copy` (Multiply 1) e `opposite` (Multiply −1) no espaço
/// inteiro. Isto corrige uma leitura minha na auditoria — eu escrevi que *"a espinha do Link
/// não é o follow"* porque a matriz compara contra o DEFAULT de B.
#[test]
fn follow_still_expresses_the_two_link_recipes_that_were_cut() {
    use ph2d_expr::{Bindings, eval};
    use ph2d_expr_parse::parse;

    struct B;
    impl Bindings for B {
        fn attr(&self, name: &str) -> f32 {
            match name {
                "Ball.x" => 3.0,
                "value" => 10.0,
                _ => 0.0,
            }
        }
        fn param(&self, _: &str) -> f32 {
            0.0
        }
    }
    let follow = |mult: f32, off: f32| {
        let mut row = ph2d_expr_recipes::Row::new("follow").expect("follow existe");
        row.set(
            "target",
            ph2d_expr_recipes::KnobValue::Link("Ball.x".into()),
        );
        row.set("multiply", ph2d_expr_recipes::KnobValue::Num(mult));
        row.set("offset", ph2d_expr_recipes::KnobValue::Num(off));
        let mut s = RecipeStack::new();
        s.push(row);
        eval(&parse(&s.to_formula()).expect("parseia"), &B)
    };
    // `offset-copy` era `link + offset`.
    assert_eq!(
        follow(1.0, 0.2),
        3.2,
        "Follow com Multiply 1 É o Offset Copy"
    );
    // `opposite` era `2*pivot - link`; com pivot 1 isso é −3 + 2.
    assert_eq!(
        follow(-1.0, 2.0),
        -1.0,
        "Follow com Multiply −1 É o Opposite (pivot = offset/2)"
    );
}

/// **Nenhuma família da galeria está vazia.**
///
/// ⚠️ `Family::ALL` foi de nove para SETE nesta fase, e os variants foram REMOVIDOS em vez
/// de ficarem sem receitas: uma família vazia é uma gaveta que abre para nada, e manter os
/// variants "por enquanto" seria exactamente isso.
#[test]
fn every_family_still_in_the_enum_has_recipes() {
    assert_eq!(
        Family::ALL.len(),
        7,
        "Logic e Field saíram com as receitas delas"
    );
    for f in Family::ALL {
        assert!(
            CATALOG.iter().any(|r| r.family == f),
            "a família {:?} não tem receita nenhuma",
            f.label()
        );
    }
}

/// **O corte não deixou o catálogo sem nenhum dos três KINDS.**
///
/// A pilha só faz sentido se houver o que empilhar: uma fonte que produz, um modificador que
/// dobra, e uma linha de Time que retima. Cortar 19 receitas de uma vez é exactamente o
/// gesto que pode zerar um deles sem ninguém notar.
#[test]
fn the_cut_left_every_kind_of_row_alive() {
    for kind in [RowKind::Value, RowKind::Time, RowKind::Raw] {
        assert!(
            CATALOG.iter().any(|r| r.kind == kind),
            "nenhuma receita sobrou com kind {kind:?}"
        );
    }
    assert!(
        CATALOG.iter().any(|r| r.combine.is_none()),
        "nenhum MODIFICADOR sobrou — a pilha não teria o que dobrar"
    );
    assert!(
        CATALOG.iter().any(|r| r.combine.is_some()),
        "nenhuma FONTE sobrou"
    );
}
