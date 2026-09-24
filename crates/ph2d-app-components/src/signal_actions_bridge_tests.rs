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

// ─────────────────────────────────────────────────────────────────────────────
// ⭐⭐⭐ **O VERBO QUE RECOMEÇA A CORRIDA** (o FIM DE JOGO, 2026-09-19)
// ─────────────────────────────────────────────────────────────────────────────

fn recomecar(target: Entity, source: Entity) -> SignalEffect {
    SignalEffect {
        target,
        verb: SignalVerb::RestartRun,
        arg: String::new(),
        source,
    }
}

/// ⭐⭐⭐ **Ele ANUNCIA, e nunca age** — o mesmo idioma da morte, na 2.ª vez que ele se usa.
///
/// ⚠️ **As duas metades:** o pedido sai no relatório, **e** o mundo fica intocado. Sem a segunda,
/// uma ponte que rebobinasse aqui passaria — e rebobinar no meio da resolução mediria um mundo que
/// os efeitos seguintes ainda não tinham visto.
///
/// **Mutações que devem sangrar:** tirar o braço do `match` · pôr a ponte a agir em vez de anunciar.
#[test]
fn o_verbo_do_recomeco_anuncia_e_nao_toca_no_mundo() {
    let (mut sim, autorado, nascido) = cena();
    let antes = sim.world().entities().len();
    let mut drive = PreviewDrive::default();
    let r = apply(
        &mut sim,
        &[recomecar(autorado, autorado)],
        &mut drive,
        &mut mudo(),
    );
    assert!(r.recomecar, "o pedido tem de sair no relatorio");
    assert_eq!(r.applied, 1, "ele nunca e' inerte: a corrida existe sempre");
    assert_eq!(
        sim.world().entities().len(),
        antes,
        "a ponte tocou no mundo — ela ANUNCIA, e quem rebobina e' a shell"
    );
    assert!(
        sim.world().get_entity(nascido).is_ok(),
        "a copia nascida na corrida so' sai pelo dreno da shell"
    );
}

/// ⭐⭐⭐ **DEZ pedidos no mesmo quadro são UM recomeço** — e quem o diz é o TIPO.
///
/// ⚠️ Dez inimigos a morrer juntos, cada um com *«ao morrer → recomeça»*, pedem dez vezes. *Uma
/// contagem aqui convidaria o dreno a rebobinar dez vezes, e a décima mediria um mundo que a
/// primeira já tinha refeito.*
///
/// **Mutação que deve sangrar:** trocar o `bool` por um `usize` que soma.
#[test]
fn dez_pedidos_no_mesmo_quadro_sao_um_recomeco() {
    let (mut sim, autorado, _) = cena();
    let efeitos: Vec<SignalEffect> = (0..10).map(|_| recomecar(autorado, autorado)).collect();
    let mut drive = PreviewDrive::default();
    let r = apply(&mut sim, &efeitos, &mut drive, &mut mudo());
    // ⛔ O tipo é a lei: um booleano não sabe contar até dez.
    assert!(r.recomecar);
    assert_eq!(
        r.applied, 10,
        "os dez efeitos aplicaram-se; o que e' UM e' a CORRIDA"
    );
}

/// ⚠️ **E ele não contamina o relatório de quem não o pediu** — o CONTROLO.
///
/// Sem esta metade, um `recomecar` que nascesse `true` faria toda a tabela do app recomeçar a
/// corrida a cada sinal, e os dois gates acima ficariam verdes.
///
/// **Mutação que deve sangrar:** `recomecar: true` no `Default` do relatório.
#[test]
fn os_outros_verbos_nao_pedem_recomeco() {
    let (mut sim, autorado, nascido) = cena();
    let mut drive = PreviewDrive::default();
    let outros: Vec<SignalEffect> = SignalVerb::ALL
        .iter()
        .filter(|v| **v != SignalVerb::RestartRun)
        .map(|v| SignalEffect {
            target: nascido,
            verb: *v,
            arg: String::new(),
            source: autorado,
        })
        .collect();
    assert_eq!(outros.len(), 11, "piso de populacao: os outros onze");
    let r = apply(&mut sim, &outros, &mut drive, &mut mudo());
    assert!(
        !r.recomecar,
        "um verbo que nao e' o `Restart Run` pediu o recomeco"
    );
    assert_eq!(
        r.mortes.len(),
        1,
        "controlo positivo: o `Destroy` continua a anunciar a morte da copia"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// ⭐⭐⭐ **OS VERBOS DA VIDA** (plano 28, W2b, 2026-09-24)
// ─────────────────────────────────────────────────────────────────────────────

fn vida(verb: SignalVerb, arg: &str, target: Entity, source: Entity) -> SignalEffect {
    SignalEffect {
        target,
        verb,
        arg: arg.into(),
        source,
    }
}

/// Uma cena com um objecto que TEM vida e outro que não tem.
fn cena_com_vida() -> (SimWorld, Entity, Entity) {
    let (mut sim, autorado, nascido) = cena();
    sim.world_mut()
        .entity_mut(autorado)
        .insert(ph2d_physics_ecs::Health::default());
    (sim, autorado, nascido)
}

/// ⭐⭐⭐ **O `Damage` e o `Heal` ANUNCIAM um pedido e nunca tocam na vida** — o idioma do `Destroy`
/// e do `Restart Run`, a terceira vez que ele se usa. A vida anda por TIQUE dentro do anel de
/// checkpoints; um verbo que a escrevesse aqui seria esquecido pelo primeiro scrub.
///
/// **Mutações que devem sangrar:** tirar o braço do `match` · trocar `Cura` por `Dano`.
#[test]
fn os_verbos_da_vida_anunciam_um_pedido() {
    use ph2d_physics_ecs::PedidoDeVida;
    let (mut sim, alvo, _) = cena_com_vida();
    let mut drive = PreviewDrive::default();
    let r = apply(
        &mut sim,
        &[
            vida(SignalVerb::Damage, "12", alvo, alvo),
            vida(SignalVerb::Heal, " 4.5 ", alvo, alvo),
        ],
        &mut drive,
        &mut mudo(),
    );
    assert_eq!(
        r.pedidos_de_vida,
        vec![
            (alvo, PedidoDeVida::Dano(12.0)),
            (alvo, PedidoDeVida::Cura(4.5))
        ]
    );
    assert_eq!((r.applied, r.inert), (2, 0));
}

/// ⚠️ **Um número ilegível, nulo, negativo ou não finito é INERTE, e um alvo sem vida também** — o
/// painel conta-os; a ponte não inventa um valor. ⛔ Ler `"abc"` como `0` ou `"-3"` como uma cura
/// seria o «aceita e mente».
///
/// **Mutações que devem sangrar:** aceitar `≤ 0` · aceitar alvo sem `Health`.
#[test]
fn um_argumento_ilegivel_ou_um_alvo_sem_vida_e_inerte() {
    let (mut sim, alvo, sem_vida) = cena_com_vida();
    let mut drive = PreviewDrive::default();
    let mut efeitos: Vec<SignalEffect> = ["", "abc", "-3", "0", "NaN", "inf"]
        .iter()
        .map(|a| vida(SignalVerb::Damage, a, alvo, alvo))
        .collect();
    efeitos.push(vida(SignalVerb::Heal, "-1", alvo, alvo));
    efeitos.push(vida(SignalVerb::Damage, "5", sem_vida, sem_vida));
    let r = apply(&mut sim, &efeitos, &mut drive, &mut mudo());
    assert!(
        r.pedidos_de_vida.is_empty(),
        "passou um pedido inválido: {:?}",
        r.pedidos_de_vida
    );
    assert_eq!((r.applied, r.inert), (0, 8));
}
