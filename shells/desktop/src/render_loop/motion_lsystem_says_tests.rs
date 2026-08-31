//! **O QUE O APP DIZ** — os três avisos que separam «não funciona» de «não funciona PORQUÊ».
//!
//! ⚠️ **Este arquivo existe por um TETO DE LOC** (HR-18, 600 para `shells/`), e o corte é por
//! responsabilidade: o irmão mede o que a folha DESENHA, e este o que o app **explica** quando
//! ela não desenha.
//!
//! ⛔⛔ Os três nasceram de reports do Enio em que o silêncio era metade da causa: *"só apareceu
//! em seu exemplo"* (a letra que a gramática não emite) · *"LFO não funciona animando Tropism
//! Angle"* (um fio num param que outro mantém inerte) · *"as folhas não aparecem"* + *"nada
//! muda"* (o `First Level` a apagar todas as marcas de uma letra).
//!
//! ⚠️ **A régua é sempre a DECISÃO, nunca o canal**: o aviso sai no `stderr`, que um gate não
//! lê — o que se mede é `already_said`, e cada gate tem de calar nos casos vizinhos, senão o
//! aviso vira ruído por quadro.

use ph2d_node_source_lsystem as ls;

/// ⛔⛔ **O nome posto numa letra que a gramática não emite tem de ser DITO.**
///
/// Report do Enio (2026-08-30): *"só apareceu em seu exemplo, ao trocar o tipo de árvore não
/// aparece mais"*. Os moldes de planta trazem `J`, mas **uma gramática escrita à mão pode não
/// trazer letra nenhuma** — e aí o campo fica cheio, nada nasce, e o artista não tem como saber
/// porquê. *Um controlo com valor lá dentro e efeito nenhum parece ligado: é a pior espécie de
/// morto.*
///
/// ⚠️ **A metade que se gateia é a DECISÃO, não o canal** — o aviso sai no `stderr`, que um teste
/// não lê; por isso a lei vive numa função pura ([`crate::render_loop::motion_lsystem_leaves::unanswered_slots`]) e é ela que se mede.
#[test]
fn a_letter_with_a_name_and_no_anchor_is_reported() {
    let anchor = |slot: usize| crate::render_loop::motion_lsystem_leaves::Anchor {
        p: [0.0, 0.0],
        rot: 0.0,
        grow: 1.0,
        slot,
    };
    let names = |a: &str, b: &str, c: &str| [a.to_string(), b.to_string(), c.to_string()];

    // Nome posto, letra ausente da gramática ⇒ acusa.
    assert_eq!(
        crate::render_loop::motion_lsystem_leaves::unanswered_slots(&names("folha", "", ""), &[]),
        vec![0],
        "um nome sem ancora nenhuma tem de ser acusado"
    );
    // A letra existe ⇒ cala.
    assert!(
        crate::render_loop::motion_lsystem_leaves::unanswered_slots(
            &names("folha", "", ""),
            &[anchor(0)]
        )
        .is_empty(),
        "com a ancora la' o aviso seria ruido"
    );
    // ⚠️ **Por SLOT, nunca «há âncoras?»** — uma gramática com `J` e um nome em `K` é exactamente
    // o caso do report, e uma régua que só perguntasse «esta planta tem âncoras?» ficaria muda.
    assert_eq!(
        crate::render_loop::motion_lsystem_leaves::unanswered_slots(
            &names("folha", "flor", ""),
            &[anchor(0)]
        ),
        vec![1],
        "o slot que tem ancora cala e o que nao tem acusa"
    );
    // Campo vazio nunca acusa: não pedir objecto nenhum é o estado normal.
    assert!(
        crate::render_loop::motion_lsystem_leaves::unanswered_slots(&names("", "", ""), &[])
            .is_empty()
    );
}
/// ⭐⭐ **UM FIO NUM PARAM INERTE É DITO** — report do Enio (2026-08-30): *"LFO não funciona
/// animando Tropism Angle"*.
///
/// ⚠️ **A régua é a DECISÃO, não o canal** (o aviso sai no `stderr`, que um gate não lê), e ela
/// tem de reprovar nos DOIS sentidos: calar quando não há fio (senão toda planta de fábrica
/// gritaria) e calar quando a força existe (senão o aviso mente).
#[test]
fn a_wire_on_an_inert_param_is_reported() {
    use crate::render_loop::motion_lsystem_leaves::{
        already_said, say_if_a_wire_drives_an_inert_param as say,
    };
    let disse = |driven: &[&str], tropism: f32, chave: &str| -> bool {
        say(chave, driven, tropism);
        already_said(&format!("{chave} inert tropism"))
    };
    assert!(
        disse(&[ls::param::TROPISM_ANGLE], 0.0, "w1"),
        "fio + forca zero tem de ser dito"
    );
    assert!(
        !disse(&[ls::param::TROPISM_ANGLE], 30.0, "w2"),
        "com forca a serio o aviso mentiria"
    );
    assert!(
        !disse(&[], 0.0, "w3"),
        "sem fio, o default de toda planta calaria o aviso"
    );
    assert!(
        !disse(&[ls::param::ANGLE], 0.0, "w4"),
        "um fio NOUTRO param nao diz nada sobre o tropismo"
    );
}
/// ⭐⭐⭐ **O `First Level` QUE APAGA TUDO É DITO** — o silêncio que gerou DOIS reports do Enio
/// no mesmo minuto (*"as folhas não aparecem"* e *"nada muda"*).
///
/// ⚠️ **Ele tem de calar nos três casos vizinhos**, senão vira ruído por quadro: sem nome, sem
/// marca daquela letra (aí quem fala é o aviso da letra), e quando pelo menos uma se desenha.
#[test]
fn a_first_level_that_hides_every_leaf_is_reported() {
    use crate::render_loop::motion_lsystem_leaves::{
        Anchor, already_said, say_if_the_level_hid_every_leaf as say,
    };
    let mark = |slot: usize, grow: f32| Anchor {
        p: [0.0, 0.0],
        rot: 0.0,
        grow,
        slot,
    };
    let nomes = |a: &str| [a.to_string(), String::new(), String::new()];
    let disse = |anchors: &[Anchor], chave: &str| -> bool {
        say(chave, &nomes("folha"), anchors, 3.0);
        already_said(&format!("{chave} level 0"))
    };
    assert!(
        disse(&[mark(0, 0.0), mark(0, 0.0)], "h1"),
        "duas marcas e nenhuma a desenhar tem de ser dito"
    );
    assert!(
        !disse(&[mark(0, 0.0), mark(0, 1.0)], "h2"),
        "com UMA folha a desenhar o aviso mentiria"
    );
    assert!(
        !disse(&[], "h3"),
        "sem marca nenhuma quem fala e' o aviso da letra"
    );
    say("h4", &nomes(""), &[mark(0, 0.0)], 3.0);
    assert!(!already_said("h4 level 0"), "sem nome nao ha' o que avisar");
}
