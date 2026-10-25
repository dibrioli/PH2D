//! ⭐ **A camada apagada leva a linha dela** (decisão do dono 25/10, o After Effects): o binding de
//! um ITEM que saiu (a tinta, o filtro, o item de lista) fica «missing» como o de um objecto
//! morto — escondido, não aplicado, guardado —, e o undo global que devolve o item (o MESMO id)
//! devolve a linha, sem passo nenhum do undo da timeline.

use super::*;
use ph2d_ecs::canal::familia::{LISTA, VETOR};
use ph2d_ecs::canal_lista::PosicoesDosItens;
use ph2d_ecs::canal_vetor as v;
use ph2d_ecs::canal_vetor_ids::{IdsDaForma, IdsDoVetor};
use ph2d_ecs::{CanaisEscritos, FieldKey, ItemId, SimWorld};
use ph2d_timeline::PropKind;

/// O quadro: o apply (a liveness), o upkeep, o snapshot. Devolve (linhas visíveis de `k`, escritas
/// de `k` neste apply).
fn quadro(tl: &mut TimelineState, sim: &mut SimWorld, k: FieldKey) -> (usize, usize) {
    sim.world_mut().insert_resource(CanaisEscritos::default());
    ph2d_timeline::apply_from_doc(sim.world_mut(), &mut tl.doc, 0.5);
    upkeep(tl, sim.world_mut());
    let escritas = sim
        .world_mut()
        .resource_mut::<CanaisEscritos>()
        .drenar(k.familia)
        .iter()
        .filter(|(_, kk, _)| *kk == k)
        .count();
    let mut snap = ph2d_timeline::TimelineViewSnapshot::default();
    snap.rebuild(tl, &ph2d_core::Playhead::new(1.0 / 60.0), false);
    let linhas = snap
        .tracks
        .iter()
        .filter(|t| t.prop == PropKind::Field(k))
        .count();
    (linhas, escritas)
}

/// Um objecto vivo com a chave `k` animada (duas chaves) e o par «sem o item» / «com o item».
fn animado(k: FieldKey) -> (SimWorld, u64, TimelineState) {
    let mut sim = SimWorld::new();
    let e = sim
        .world_mut()
        .spawn((ph2d_ecs::Transform::IDENTITY, Name::new("forma")))
        .id()
        .to_bits();
    let mut tl = TimelineState::new();
    let s = ph2d_anim::RationalTime::from_seconds;
    let f = ph2d_anim::AnimValue::Float;
    for (t, x) in [(0.0, 1.0), (1.0, 2.0)] {
        tl.doc
            .insert_key(e, PropKind::Field(k), s(t), f(x), ph2d_anim::Interp::Linear);
    }
    (sim, e, tl)
}

/// A régua comum: vivo ⇒ visível e aplicado; apagado ⇒ escondido, não aplicado, ainda no
/// documento, sem passo de undo; devolvido ⇒ volta. (Mutação: a liveness ignora o item ⇒ a linha
/// apagada continua visível — RED.)
fn regua(k: FieldKey, com: impl Fn(&mut SimWorld), sem: impl Fn(&mut SimWorld)) {
    let (mut sim, _, mut tl) = animado(k);
    com(&mut sim);
    assert_eq!(
        quadro(&mut tl, &mut sim, k),
        (1, 1),
        "vivo: a linha e a escrita"
    );
    let passos = tl.history.can_undo();

    sem(&mut sim);
    assert_eq!(
        quadro(&mut tl, &mut sim, k),
        (0, 0),
        "apagado: a linha some e a curva não escreve"
    );
    assert!(
        tl.doc
            .bindings()
            .iter()
            .any(|b| b.prop == PropKind::Field(k)),
        "o binding fica no documento, dormente"
    );
    assert_eq!(
        tl.history.can_undo(),
        passos,
        "nenhum passo do undo da timeline"
    );

    com(&mut sim);
    assert_eq!(
        quadro(&mut tl, &mut sim, k),
        (1, 1),
        "o undo global devolve o item com o mesmo id: a linha volta e aplica"
    );
}

fn forma(sim: &mut SimWorld, tintas: &[u32], filtros: &[u32]) {
    let e = sim
        .world_mut()
        .query::<(ph2d_ecs::Entity, &Name)>()
        .iter(sim.world())
        .next()
        .expect("a forma")
        .0
        .to_bits();
    let ids = |x: &[u32]| x.iter().map(|&n| ItemId(n)).collect();
    let f = IdsDaForma {
        tintas: ids(tintas),
        efeitos: Vec::new(),
        filtros: ids(filtros),
    };
    sim.world_mut().insert_resource(IdsDoVetor([(e, f)].into()));
}

#[test]
fn apagar_uma_tinta_animada_esconde_a_linha_e_o_undo_a_devolve() {
    let k = FieldKey::do_item(VETOR, v::PILHA_OPACIDADE, ItemId(20), 0);
    regua(
        k,
        |sim| forma(sim, &[10, 20], &[]),
        |sim| forma(sim, &[10], &[]),
    );
}

#[test]
fn apagar_um_filtro_animado_esconde_a_linha_e_o_undo_a_devolve() {
    let k = FieldKey::do_item(VETOR, v::FILTRO_RAIO, ItemId(40), 0);
    regua(
        k,
        |sim| forma(sim, &[10], &[40, 41]),
        |sim| forma(sim, &[10], &[41]),
    );
}

/// ⭐ A família LISTA pela mesma porta — e o ÚNICO item apagado (a entidade publicada sem item
/// nenhum) também conta como apagado.
#[test]
fn apagar_um_item_de_lista_animado_esconde_a_linha() {
    let k = FieldKey::new(LISTA, 0, 0, 7); // `i:j` = o `ItemId` 7
    let publica = |sim: &mut SimWorld, itens: &[u32]| {
        let e = sim
            .world_mut()
            .query::<(ph2d_ecs::Entity, &Name)>()
            .iter(sim.world())
            .next()
            .expect("o objecto")
            .0
            .to_bits();
        sim.world_mut().insert_resource(PosicoesDosItens(
            itens.iter().map(|&i| ((e, i), 1)).collect(),
            [e].into(),
        ));
    };
    regua(k, |sim| publica(sim, &[7]), |sim| publica(sim, &[]));
}

/// Sem espelho da entidade (um id por atribuir: a família não a publicou) não se decide — vivo.
#[test]
fn sem_espelho_da_entidade_o_item_conta_como_vivo() {
    let k = FieldKey::do_item(VETOR, v::PILHA_OPACIDADE, ItemId(20), 0);
    let (mut sim, _, mut tl) = animado(k);
    sim.world_mut().insert_resource(IdsDoVetor::default());
    sim.world_mut().insert_resource(PosicoesDosItens::default());
    assert_eq!(quadro(&mut tl, &mut sim, k), (1, 1));
    let kl = FieldKey::new(LISTA, 0, 0, 7);
    let (mut sim, _, mut tl) = animado(kl);
    sim.world_mut().insert_resource(PosicoesDosItens::default());
    assert_eq!(quadro(&mut tl, &mut sim, kl), (1, 1));
}
