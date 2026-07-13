//! **A binding DESTACADA** — `entity = 0`, o sentinel que o `resolve_entities` escreve para
//! "esta não tem objeto vivo; ache-o pelo `wire_id`".
//!
//! Um projeto recém-carregado é feito SÓ disso: toda binding chega destacada e o `upkeep` do
//! frame as recola pelo nome. Então o caminho que estes gates cobrem é o Ctrl+O inteiro, e ele
//! tem duas minas — as duas armadas há semanas, as duas invisíveis enquanto o único caminho que
//! escrevia o sentinel era código morto:
//!
//! 1. **`0` não é um nulo no bevy.** O índice é `NonZero<u32>`, então `Entity::from_bits(0)`
//!    **entra em pânico** em vez de devolver uma entidade morta. O apply do frame decodifica
//!    TODA binding (é ele quem decide `missing`), então o primeiro frame depois do load
//!    derrubava o app.
//! 2. **O save apagava a identidade.** O `stamp_wire_ids` sobrescrevia o `wire_id` com o hash do
//!    objeto vivo — e NULL quando não há objeto vivo. Uma track dormente (objeto deletado, ou
//!    ainda destacada pelo load) perdia a única coisa que a fazia reencontrar o dono.

use ph2d_anim::RationalTime;
use ph2d_ecs::{Entity, Name, SimWorld, World};
use ph2d_timeline::{
    AnimValue, Interp, PropKind, TimelineDoc, WireId, apply_from_doc, refresh_and_heal_bindings,
    resolve_entities, stamp_wire_ids,
};

/// O hash do nome, como o shell o faz (FNV-1a — `timeline_persist::wire_id_for_name`).
fn wire_of_name(name: &str) -> WireId {
    const OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
    const PRIME: u64 = 0x0000_0100_0000_01b3;
    let mut h = OFFSET;
    for b in name.as_bytes() {
        h ^= u64::from(*b);
        h = h.wrapping_mul(PRIME);
    }
    WireId(if h == 0 { 1 } else { h })
}

/// O que o shell passa pro `stamp_wire_ids`: o nome do objeto vivo, ou NULL se não há um.
/// Decodifica com `try_from_bits` — bits destacados (`0`) fariam `from_bits` entrar em pânico.
fn wire_of(world: &World, bits: u64) -> WireId {
    Entity::try_from_bits(bits)
        .and_then(|e| world.get::<Name>(e))
        .map_or(WireId::NULL, |n| wire_of_name(n.as_str()))
}

fn keyed(doc: &mut TimelineDoc, entity: u64) {
    doc.upsert_key(
        entity,
        PropKind::TranslationX,
        RationalTime::from_seconds(0.0),
        AnimValue::Float(1.0),
        Interp::Linear,
    );
}

/// **O frame seguinte ao Ctrl+O não pode derrubar o app.**
///
/// O documento que o load instala tem TODA binding destacada. O `apply_from_doc` roda antes do
/// `upkeep` no mesmo frame (é ele quem levanta o `missing` que o `upkeep` cura), então ele vê os
/// bits zerados — e `Entity::from_bits(0)` entra em pânico: *"Attempted to initialize invalid
/// bits as an entity"*. Abrir qualquer projeto com uma track keyada matava o processo.
///
/// FALSIFICADO trocando `try_from_bits` por `from_bits` no `apply.rs`: panic, não falha de
/// asserção.
#[test]
fn the_apply_survives_the_document_the_loader_installs() {
    let mut sim = SimWorld::new();
    let hero = sim.world_mut().spawn(Name::new("hero")).id().to_bits();
    let mut doc = TimelineDoc::new();
    keyed(&mut doc, hero);

    resolve_entities(&mut doc, |_| None); // exatamente o que o `install_from_project` faz
    assert_eq!(doc.bindings()[0].entity, 0, "destacada");

    apply_from_doc(sim.world_mut(), &mut doc, 0.0); // …e o que o frame faz em seguida
    assert!(
        doc.bindings()[0].missing,
        "bits indecodificáveis É o que missing quer dizer"
    );
}

/// E, curada pelo nome, a animação volta a dirigir o objeto — o load inteiro, ponta a ponta.
#[test]
fn the_healed_binding_drives_the_object_the_load_spawned() {
    let mut sim = SimWorld::new();
    let old = sim.world_mut().spawn(Name::new("hero")).id().to_bits();
    let mut doc = TimelineDoc::new();
    keyed(&mut doc, old);
    stamp_wire_ids(&mut doc, |b| wire_of(sim.world(), b));
    resolve_entities(&mut doc, |_| None); // o load: destaca tudo

    // O `apply_project` respawnou o mundo: MESMO nome, bits NOVOS.
    let mut sim2 = SimWorld::new();
    for _ in 0..3 {
        sim2.world_mut().spawn(());
    }
    let reborn = sim2
        .world_mut()
        .spawn((Name::new("hero"), ph2d_ecs::Transform::default()))
        .id();
    assert_ne!(reborn.to_bits(), old, "bits novos");

    apply_from_doc(sim2.world_mut(), &mut doc, 0.0); // frame: levanta missing
    let by_wire = |w: WireId| (w == wire_of_name("hero")).then_some(reborn.to_bits());
    assert_eq!(
        refresh_and_heal_bindings(&mut doc, |b| wire_of(sim2.world(), b), by_wire),
        1,
        "o upkeep recolou pelo NOME"
    );
    assert_eq!(doc.bindings()[0].entity, reborn.to_bits());
    assert!(!doc.bindings()[0].missing);
}

/// **Um save NUNCA apaga a identidade de uma track dormente.**
///
/// `wire_of` só sabe responder por objeto VIVO — de um deletado (ou de uma binding ainda
/// destacada) ele devolve NULL. Escrever esse NULL por cima queimaria a única coisa que a track
/// ainda tem, e ela nunca mais recolaria: nem pelo undo que respawna o objeto, nem recriando um
/// objeto de mesmo nome, nem em sessão nenhuma.
///
/// O bug real: animar `hero` → deletar `hero` (a track sobrevive dormente, por design) →
/// **Ctrl+S** → a identidade some do documento E do arquivo. O save destruía o mecanismo em que
/// o módulo inteiro se apoia, em silêncio.
#[test]
fn a_save_never_erases_a_dormant_tracks_identity() {
    let mut sim = SimWorld::new();
    let hero = sim.world_mut().spawn(Name::new("hero")).id();
    let mut doc = TimelineDoc::new();
    keyed(&mut doc, hero.to_bits());
    stamp_wire_ids(&mut doc, |b| wire_of(sim.world(), b));
    let identity = doc.bindings()[0].wire_id;
    assert!(!identity.is_null());

    // O objeto morre. A track fica dormente — é o design.
    sim.world_mut().despawn(hero);
    apply_from_doc(sim.world_mut(), &mut doc, 0.0);
    assert!(doc.bindings()[0].missing);

    // Ctrl+S com o objeto morto.
    stamp_wire_ids(&mut doc, |b| wire_of(sim.world(), b));
    assert_eq!(
        doc.bindings()[0].wire_id,
        identity,
        "o save preservou a identidade da track dormente"
    );

    // …e por isso o undo (que respawna com bits novos e o mesmo nome) ainda a cura.
    let reborn = sim.world_mut().spawn(Name::new("hero")).id();
    let by_wire = |w: WireId| (w == wire_of_name("hero")).then_some(reborn.to_bits());
    assert_eq!(
        refresh_and_heal_bindings(&mut doc, |b| wire_of(sim.world(), b), by_wire),
        1,
        "a track voltou com o objeto — o que o save teria tornado impossível"
    );
}

/// O mesmo, para a binding que o LOAD acabou de destacar: salvar antes de o `upkeep` recolar
/// (um Ctrl+S no primeiro frame depois de um Ctrl+O) não pode nem entrar em pânico nem apagar a
/// identidade que veio do arquivo.
#[test]
fn saving_before_the_heal_neither_panics_nor_forgets() {
    let mut sim = SimWorld::new();
    let hero = sim.world_mut().spawn(Name::new("hero")).id().to_bits();
    let mut doc = TimelineDoc::new();
    keyed(&mut doc, hero);
    stamp_wire_ids(&mut doc, |b| wire_of(sim.world(), b));
    let identity = doc.bindings()[0].wire_id;
    resolve_entities(&mut doc, |_| None); // o load

    stamp_wire_ids(&mut doc, |b| wire_of(sim.world(), b)); // Ctrl+S imediato: bits = 0
    assert_eq!(
        doc.bindings()[0].wire_id,
        identity,
        "a identidade do arquivo sobreviveu ao save prematuro"
    );
}
