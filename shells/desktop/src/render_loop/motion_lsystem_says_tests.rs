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
        seed: 0,
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
    // ⚠️ **Um NÓ por caso**, e não uma string: o registo passou a ser chaveado pelo nó, e dois
    // casos no mesmo nó mediriam o FLANCO do anterior em vez do próprio (doc 96 §2.3).
    let disse = |driven: &[&str], tropism: f32, n: u32| -> bool {
        let id = ph2d_nodegraph::graph::NodeId(n);
        say(id, driven, tropism);
        already_said(&format!("{n} inert tropism"))
    };
    assert!(
        disse(&[ls::param::TROPISM_ANGLE], 0.0, 1),
        "fio + forca zero tem de ser dito"
    );
    assert!(
        !disse(&[ls::param::TROPISM_ANGLE], 30.0, 2),
        "com forca a serio o aviso mentiria"
    );
    assert!(
        !disse(&[], 0.0, 3),
        "sem fio, o default de toda planta calaria o aviso"
    );
    assert!(
        !disse(&[ls::param::ANGLE], 0.0, 4),
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
        seed: 0,
        slot,
    };
    let nomes = |a: &str| [a.to_string(), String::new(), String::new()];
    let disse = |anchors: &[Anchor], n: u32| -> bool {
        say(
            ph2d_nodegraph::graph::NodeId(n),
            &nomes("folha"),
            anchors,
            3.0,
        );
        already_said(&format!("{n} level 0"))
    };
    assert!(
        disse(&[mark(0, 0.0), mark(0, 0.0)], 1),
        "duas marcas e nenhuma a desenhar tem de ser dito"
    );
    assert!(
        !disse(&[mark(0, 0.0), mark(0, 1.0)], 2),
        "com UMA folha a desenhar o aviso mentiria"
    );
    assert!(
        !disse(&[], 3),
        "sem marca nenhuma quem fala e' o aviso da letra"
    );
    say(
        ph2d_nodegraph::graph::NodeId(4),
        &nomes(""),
        &[mark(0, 0.0)],
        3.0,
    );
    assert!(!already_said("4 level 0"), "sem nome nao ha' o que avisar");
}

/// ⭐⭐⭐ **O REGISTO DOS AVISOS NÃO CRESCE COM O RELÓGIO** — achado §2.3 da auditoria de seis
/// lentes.
///
/// ⚠️ **O defeito não era o aviso: era a CHAVE dele.** O registo era chaveado pela chave de
/// CONTEÚDO da planta, que mistura os **31 params pelos bits** — com o `Generations` animado ela
/// é nova em cada quadro, então o `eprintln!` saía **60×/s** e o `BTreeSet` crescia
/// `~280 B/quadro` **sem varredura nenhuma**. Reproduzido: 320 quadros, 320 impressões.
///
/// A régua é a CONTAGEM de entradas, não o `stderr`: 300 quadros do mesmo nó no mesmo estado
/// mau têm de deixar **uma** entrada.
#[test]
fn the_warning_ledger_does_not_grow_with_the_clock() {
    use crate::render_loop::motion_lsystem_leaves::{
        said_len, say_if_a_wire_drives_an_inert_param as say,
    };
    let id = ph2d_nodegraph::graph::NodeId(7001);
    let antes = said_len();
    for _ in 0..300 {
        say(id, &[ls::param::TROPISM_ANGLE], 0.0);
    }
    assert_eq!(
        said_len() - antes,
        1,
        "300 quadros do MESMO aviso deixaram mais de uma entrada — o registo cresce com o relogio"
    );
}

/// ⭐⭐ **E ELE VOLTA A SAIR quando o artista parte a coisa outra vez** — a metade que o
/// chaveamento por conteúdo comprava por acidente, e que um `insert` sozinho perderia.
///
/// ⚠️ *Um aviso que não volta a sair lê-se exactamente como um aviso que nunca precisou de
/// sair.* Por isso o registo é um FLANCO: entrar no estado mau avisa, **sair esquece**.
#[test]
fn fixing_the_problem_lets_the_warning_speak_again_when_it_returns() {
    use crate::render_loop::motion_lsystem_leaves::{
        already_said, say_if_a_wire_drives_an_inert_param as say,
    };
    let id = ph2d_nodegraph::graph::NodeId(7002);
    let disse = |tropism: f32| {
        say(id, &[ls::param::TROPISM_ANGLE], tropism);
        already_said("7002 inert tropism")
    };
    assert!(disse(0.0), "o 1.o aviso tem de sair");
    assert!(!disse(30.0), "com forca a serio ele cala-se E esquece");
    assert!(
        disse(0.0),
        "voltar ao estado mau tem de voltar a avisar — senao o artista parte a planta em silencio"
    );
}
