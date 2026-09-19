//! ⭐⭐⭐ **O VERBO QUE TIRA DA CENA, e a ORIGEM QUE ATRAVESSA A LEITURA** (suplente #24, 2026-09-19).
//!
//! ⚠️ **Estes gates VIERAM da shell com a ponte** (a catraca `the_shell_only_shrinks` mandou-a
//! descer). ⛔ A metade que FICOU lá é a que lê a FASE por `include_str!` — ela mede a ordem do
//! quadro, que é composição e não é desta crate.

use ph2d_ecs::{
    DeathCause, Entity, Name, SignalEffect, SignalVerb, SimWorld, Spawned, StableId, Transform,
};
use ph2d_preview_drive::PreviewDrive;

use super::{Som, apply, lido};

/// O som INJECTADO destes gates — um fecho que nunca toca nada. Ver o irmão das LEIS.
fn mudo() -> impl FnMut(&mut SimWorld, Som, Entity) -> bool {
    |_, _, _| false
}

/// Uma cena com **um objecto do documento** e **uma cópia nascida numa corrida** — a mesma partição
/// da fixtura do `the_run_never_enters_the_document`.
fn cena() -> (SimWorld, Entity, Entity) {
    let mut sim = SimWorld::new();
    let w = sim.world_mut();
    let autorado = w
        .spawn((Transform::IDENTITY, Name::new("Parede"), StableId(1)))
        .id();
    let nascido = w
        .spawn((
            Transform::IDENTITY,
            Name::new("Bala"),
            StableId(2),
            Spawned {
                by: 7,
                born_tick: 1,
            },
        ))
        .id();
    (sim, autorado, nascido)
}

fn destruir(target: Entity, source: Entity) -> SignalEffect {
    SignalEffect {
        target,
        verb: SignalVerb::Destroy,
        arg: String::new(),
        source,
    }
}

/// ⭐⭐⭐ **Ele tira quem NASCEU numa corrida e recusa o que é do DOCUMENTO** — as duas metades, e
/// nenhuma basta sozinha.
///
/// ⚠️ **A fronteira é FORÇADA e não escolhida:** apagar um objecto do documento durante a corrida
/// tira-o do documento (a captura vê-o sumido e o `Ctrl+Z` herda a remoção). O precedente é do
/// TOP-20 #14 — *«um projéctil que ele pôs na cena à mão é documento, e apagá-lo destruiria
/// autoria»* — e esta é a quarta leitura da porta [`ph2d_ecs::is_transient`].
///
/// ⚠️ **E ele ANUNCIA, nunca apaga:** o mundo fica INTACTO depois do `apply`. Quem remove é o dreno
/// da `fase_fabrica_e_morte`, por último no quadro — *um moribundo continua visível a toda consulta
/// até ao fim do quadro*.
///
/// **Mutações que devem sangrar:** apagar a guarda `is_transient` · o `apply` a despawnar em vez de
/// devolver o facto · a causa deixar de ser `Killed`.
#[test]
fn o_destroy_tira_quem_nasceu_e_recusa_o_documento() {
    let (mut sim, autorado, nascido) = cena();
    let mut drive = PreviewDrive::default();

    // ⛔ O CONTROLO: o objecto do DOCUMENTO é recusado, e contado como inerte.
    let r = apply(
        &mut sim,
        &[destruir(autorado, autorado)],
        &mut drive,
        &mut mudo(),
    );
    assert_eq!(r.applied, 0, "um alvo de documento foi aplicado");
    assert_eq!(r.inert, 1, "a recusa tem de ser CONTADA");
    assert!(r.mortes.is_empty(), "um alvo de documento produziu morte");

    // ⭐ E a cópia nascida na corrida sai — anunciada, não apagada.
    let r = apply(
        &mut sim,
        &[destruir(nascido, nascido)],
        &mut drive,
        &mut mudo(),
    );
    assert_eq!(r.applied, 1);
    assert_eq!(r.inert, 0);
    assert_eq!(r.mortes.len(), 1, "a cópia nao produziu facto de morte");
    assert_eq!(r.mortes[0].entity, nascido);
    assert_eq!(r.mortes[0].why, DeathCause::Killed);
    assert!(
        r.mortes[0].signal.is_empty(),
        "o sinal de morte e' assunto do `Lifetime`, que ja' o autora"
    );

    // ⚠️ **O mundo fica INTACTO** — as duas ainda lá estão no fim do `apply`.
    assert!(sim.world().get_entity(autorado).is_ok());
    assert!(
        sim.world().get_entity(nascido).is_ok(),
        "o `apply` APAGOU — e quem remove e' o dreno, uma vez por quadro"
    );
}

/// ⭐⭐⭐ **A ORIGEM ATRAVESSA A LEITURA** — era exactamente aqui que ela morria.
///
/// Até esta wave a fase fazia `.map(|s| s.name.to_string())`, e o `source`/`other` do
/// `SignalOrigin` era descartado **uma linha antes** de a tabela precisar deles.
///
/// ⚠️ **As três metades:** o nome chega · quem gritou chega · o outro lado chega **e é outro**. Com
/// os dois bits iguais na fixtura, trocar `quem` por `outro` seria invisível.
///
/// **Mutação que deve sangrar:** o `lido` a devolver `(nome, None, None)`.
#[test]
fn a_origem_atravessa_a_leitura_do_sinal() {
    // ⚠️ Bits de entidades REAIS, tirados de um mundo — `try_from_bits` recusa uma codificação
    // inventada, e um gate com bits inventados mediria o ramo de RECUSA em vez do de passagem.
    let mut sim = SimWorld::new();
    let a = sim.world_mut().spawn(Transform::IDENTITY).id();
    let b = sim.world_mut().spawn(Transform::IDENTITY).id();
    assert_ne!(a, b, "a fixtura precisa de dois sujeitos DIFERENTES");

    let sinal = ph2d_runtime::Signal::from_contact("golpe", a.to_bits(), b.to_bits());
    let (nome, quem, outro) = lido(&sinal);
    assert_eq!(nome, "golpe");
    assert_eq!(quem, Some(a), "quem gritou perdeu-se na leitura");
    assert_eq!(outro, Some(b), "o outro lado perdeu-se na leitura");

    // ⛔ E uma origem SEM sujeito continua sem ele — `None` não é um curinga.
    let (nome, quem, outro) = lido(&ph2d_runtime::Signal::from_control("botao"));
    assert_eq!(nome, "botao");
    assert_eq!(quem, None);
    assert_eq!(outro, None);
}
