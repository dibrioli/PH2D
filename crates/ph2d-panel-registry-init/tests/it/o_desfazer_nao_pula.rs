//! ⭐ **Ctrl+Z e Ctrl+Y não pulam os cartões** (`docs/CartoesUI/10`, smoke 3, defeito B), por gesto real
//! sobre a `Mesa` do C2. O restauro respawna o mundo: os bits mudam e nada mais. Medido na app, antes da
//! cura: um quadro sem seleção punha os 7 cartões a sair, e a lista ia de 681 a 450 px e voltava em 27
//! quadros. A porta é `ph2d_host_presos::depois_do_restauro`.

use super::*;
use ph2d_editor_core::panel::cards::entra_sai;

/// O que a mão vê da lista, a cada quadro: a altura dela, a altura e o `y` de cada cartão, a rolagem,
/// e a altura de cada solto.
#[derive(Debug, PartialEq)]
struct Lista {
    h: Option<f32>,
    cartoes: Vec<(NodeId, f32)>,
    y: Vec<Option<f32>>,
    rolagem: f32,
    soltos: Vec<Option<f32>>,
}

fn lista(m: &mut Mesa, soltos: &[NodeId]) -> Lista {
    let y = [
        v::VECTOR_WIDTH_NUM,
        v::VECTOR_FILL_OPACITY_NUM,
        insp::ids::INSP_TRANSFORM_POS_X,
    ]
    .map(|id| m.h.hit_index.rect_for(id).map(|r| r.y))
    .to_vec();
    let h = m.h.store.panel_rect(INSP_PANEL).map(|r| r.h);
    let rolagem = m.h.store.panel_scroll(INSP_PANEL);
    let cartoes =
        m.h.store
            .card_list_mut(INSP_PANEL)
            .items
            .iter()
            .map(|i| (i.key, i.h))
            .collect();
    let soltos = soltos
        .iter()
        .map(|k| m.h.store.panel_rect(loose_rect_of(*k)).map(|r| r.h))
        .collect();
    Lista {
        h,
        cartoes,
        y,
        rolagem,
        soltos,
    }
}

fn id_estavel(i: usize) -> ph2d_ecs::StableId {
    ph2d_ecs::StableId(900 + i as u64)
}

fn depois_do_restauro(m: &mut Mesa) {
    let antes = ph2d_host_presos::antes_do_restauro(m.sim.world());
    depois_com(m, &antes);
}

fn depois_com(m: &mut Mesa, antes: &ph2d_host_presos::AntesDoRestauro) {
    let mut sel = ph2d_app_vec::selection_sync::VecSelSync::default();
    ph2d_host_presos::depois_do_restauro(
        &mut m.h, &m.sim, &m.scene, &m.map, &mut m.pen, &mut sel, &m.tools, antes,
    );
}

/// O respawn do restauro: as três formas com bits novos, e o resto igual.
fn respawn(m: &mut Mesa) {
    respawn_sem(m, &[]);
}

/// O respawn de um passo em que `fora` não existia ainda.
fn respawn_sem(m: &mut Mesa, fora: &[usize]) {
    for i in 0..3 {
        let velho = ph2d_ecs::Entity::from_bits(m.bits[i]);
        if m.sim.world().get_entity(velho).is_ok() {
            m.sim.world_mut().despawn(velho);
        }
        if fora.contains(&i) {
            continue;
        }
        let e = m
            .sim
            .world_mut()
            .spawn((
                Transform::default(),
                Name::new(nome(i)),
                VecPathRef(m.ids[i]),
                id_estavel(i),
            ))
            .id();
        m.bits[i] = e.to_bits();
        m.map.insert(m.ids[i], e.to_bits());
    }
}

/// A lista assentou: as molas pararam e dois quadros seguidos são iguais.
fn assenta(m: &mut Mesa) {
    for _ in 0..400 {
        m.quadro();
        if !m.h.store.card_list_mut(INSP_PANEL).stack.moving() {
            return;
        }
    }
    panic!("a lista não assentou");
}

fn mesa() -> Mesa {
    let mut m = Mesa::nova();
    for sec in [insp::CARD_TRANSFORM, insp::CARD_APPEARANCE] {
        m.h.store.set_collapsed(sec, false);
    }
    assenta(&mut m);
    m
}

/// ⭐ O desfazer que não muda a estrutura: nenhum quadro difere, em nenhum cartão. *Mutação: a régua é
/// viva: o buraco de um quadro, abaixo, é vermelho.*
#[test]
fn an_undo_that_keeps_the_cards_moves_nothing() {
    let mut m = mesa();
    let foto = lista(&mut m, &[]);
    assert!(foto.cartoes.len() >= 3, "{foto:?}");
    // Ctrl+Z e Ctrl+Y são o mesmo gesto: dois respawns.
    for _ in 0..2 {
        respawn(&mut m);
        m.escolhe(&[0]);
        for q in 0..40 {
            m.quadro();
            assert_eq!(lista(&mut m, &[]), foto, "quadro {q}");
            assert!(
                entra_sai::saindo(INSP_PANEL).is_empty(),
                "quadro {q}: há cartões a sair"
            );
        }
    }
}

/// A régua: o buraco de um quadro sem seleção (a app antes da cura) pula os cartões.
#[test]
fn one_frame_without_a_selection_is_what_made_the_cards_jump() {
    let mut m = mesa();
    let foto = lista(&mut m, &[]);
    insp::set_current_inspector_transform(None);
    insp::set_current_inspector_name(None);
    appearance::set_current_appearance(None);
    appearance::set_current_vector_style(None);
    m.h.store.set_card_subject(INSP_PANEL, None);
    m.h.gizmo.clear_all_selection();
    pinta(&mut m.h, 1);
    assert!(
        !entra_sai::saindo(INSP_PANEL).is_empty(),
        "a régua não vê o buraco"
    );
    m.escolhe(&[0]);
    m.quadro();
    let h = lista(&mut m, &[]).h.expect("a lista");
    assert!(
        h < foto.h.expect("a lista") - 20.0,
        "o buraco não mexeu: {h} × {foto:?}"
    );
}

/// O cartão preso, com os bits novos ligados pela chave: o quadro seguinte mostra o mesmo.
/// *Mutação: tirar `chave::resolve` de `depois_do_restauro` ⇒ o Fill preso de A perde a camada por um
/// quadro (na app: 145 → 99 → 145 px).*
#[test]
fn an_undo_does_not_flicker_a_pinned_card() {
    let mut m = mesa();
    for i in 0..3 {
        let e = ph2d_ecs::Entity::from_bits(m.bits[i]);
        m.sim.world_mut().entity_mut(e).insert(id_estavel(i));
    }
    let k = m.prende(CARD_FILL, 0, &[1]);
    depois_do_restauro(&mut m); // o quadro da app liga os bits à chave durável
    assenta(&mut m);
    let foto = lista(&mut m, &[k]);
    for _ in 0..2 {
        respawn(&mut m);
        depois_do_restauro(&mut m);
        assert_eq!(
            m.h.gizmo.iter_selected().collect::<Vec<_>>(),
            [m.bits[1]],
            "o gizmo não voltou"
        );
        assert_eq!(
            m.h.store.loose_card(k).and_then(|c| c.pin),
            Some(m.bits[0]),
            "o alfinete não voltou"
        );
        for q in 0..20 {
            m.quadro();
            assert_eq!(lista(&mut m, &[k]), foto, "quadro {q}");
        }
    }
}

/// O cartão de um componente: o que o mundo do passo tem publica-se com os bits de agora.
fn publica_a_fisica(m: &Mesa, tem: bool) {
    use super::super::o_inspector_armado::fisica_armada;
    let f = ph2d_editor_core::screens::hero::InspectorPhysicsInfo {
        entity_bits: m.bits[0],
        ..fisica_armada()
    };
    insp::set_current_inspector_physics(tem.then_some(f));
}

/// Larga o que a thread publicou, mesmo quando o teste cai.
struct SemFisica;
impl Drop for SemFisica {
    fn drop(&mut self) {
        insp::set_current_inspector_physics(None);
    }
}

/// ⭐ O desfazer que TIRA um cartão de verdade (o componente nasceu no passo desfeito): só ele sai, com
/// mola; nenhum outro cartão entra na lista de saída; o refazer abre-o com mola e a lista volta à mesma.
/// *Mutação: `entra_sai::itens` sem `saindo` ⇒ a saída chega ao fim num quadro.*
#[test]
fn an_undo_that_takes_a_card_away_closes_it_on_the_spring_and_the_redo_opens_it() {
    use ph2d_editor_core::ids::INSP_LIVE_PHYSICS_SECTION as FISICA;
    let _limpa = SemFisica;
    // Só com o que cabe: com a lista a transbordar, a altura dela é a da janela e não mede a mola.
    let mut m = Mesa::nova();
    for i in 0..3 {
        let e = ph2d_ecs::Entity::from_bits(m.bits[i]);
        m.sim.world_mut().entity_mut(e).insert(id_estavel(i));
    }
    m.h.store.set_collapsed(FISICA, false);
    publica_a_fisica(&m, true);
    assenta(&mut m);
    let cheia = lista(&mut m, &[]);
    assert!(
        cheia.cartoes.iter().any(|(k, _)| *k == FISICA),
        "a régua: o cartão da física não está na lista: {cheia:?}"
    );
    let altura = |l: &Lista| l.h.expect("a lista");

    // Ctrl+Z.
    let antes = ph2d_host_presos::antes_do_restauro(m.sim.world());
    respawn(&mut m);
    depois_com(&mut m, &antes);
    publica_a_fisica(&m, false);
    m.quadro();
    assert_eq!(entra_sai::saindo(INSP_PANEL), [FISICA], "só a física sai");
    let um = lista(&mut m, &[]);
    assert_eq!(um.rolagem, cheia.rolagem, "a rolagem saltou");
    let mut ultima = altura(&um);
    for q in 0..400 {
        if !m.h.store.card_list_mut(INSP_PANEL).stack.moving() {
            break;
        }
        m.quadro();
        let agora = altura(&lista(&mut m, &[]));
        assert!(
            agora <= ultima + 0.5,
            "quadro {q}: voltou a subir, {ultima} → {agora}"
        );
        ultima = agora;
    }
    let vazia = lista(&mut m, &[]);
    assert!(entra_sai::saindo(INSP_PANEL).is_empty(), "ficou a sair");
    assert!(
        !vazia.cartoes.iter().any(|(k, _)| *k == FISICA),
        "não saiu: {vazia:?}"
    );
    assert!(
        altura(&cheia) - altura(&vazia) > ph2d_card_stack::metrics::CARD_HEAD_H,
        "a régua: a altura da lista não mediu a saída: {} → {}",
        altura(&cheia),
        altura(&vazia)
    );
    assert!(
        altura(&um) - altura(&vazia) > 1.0,
        "a saída chegou ao fim num quadro: {} → {} (fim {})",
        altura(&cheia),
        altura(&um),
        altura(&vazia)
    );
    let outros = |l: &Lista| -> Vec<NodeId> {
        l.cartoes
            .iter()
            .map(|(k, _)| *k)
            .filter(|k| *k != FISICA)
            .collect()
    };
    assert_eq!(outros(&vazia), outros(&cheia), "outro cartão saiu com ela");

    // Ctrl+Y.
    let antes = ph2d_host_presos::antes_do_restauro(m.sim.world());
    respawn(&mut m);
    depois_com(&mut m, &antes);
    publica_a_fisica(&m, true);
    m.quadro();
    let entra = altura(&lista(&mut m, &[]));
    assenta(&mut m);
    let de_volta = lista(&mut m, &[]);
    assert!(
        altura(&de_volta) - entra > 1.0,
        "a entrada chegou ao fim num quadro: {entra} (fim {})",
        altura(&de_volta)
    );
    assert_eq!(de_volta, cheia, "o refazer não devolveu a mesma lista");
}

/// O desfazer que leva o OBJECTO do cartão preso: o solto fica, sem bits; o refazer prende-o outra vez,
/// aos bits novos, e nenhum outro objecto o apanhou entretanto.
/// *Mutação: tirar `chave::resolve` de `depois_do_restauro` ⇒ o alfinete fica com bits mortos.*
#[test]
fn an_undo_that_takes_the_pinned_object_away_and_a_redo_that_brings_it_back() {
    let mut m = mesa();
    for i in 0..3 {
        let e = ph2d_ecs::Entity::from_bits(m.bits[i]);
        m.sim.world_mut().entity_mut(e).insert(id_estavel(i));
    }
    let k = m.prende(CARD_FILL, 0, &[1]);
    depois_do_restauro(&mut m);
    assenta(&mut m);
    let n = m.h.store.loose_cards_of(INSP_PANEL).len();
    let velho = m.bits[0];

    let antes = ph2d_host_presos::antes_do_restauro(m.sim.world());
    respawn_sem(&mut m, &[0]);
    depois_com(&mut m, &antes);
    assert_eq!(
        m.h.store.loose_cards_of(INSP_PANEL).len(),
        n,
        "o solto sumiu"
    );
    assert_eq!(
        m.h.store.loose_card(k).and_then(|c| c.pin),
        None,
        "o alfinete ficou com os bits mortos de {velho:x}"
    );
    m.quadros(3);
    assert_eq!(m.h.store.loose_card(k).and_then(|c| c.pin), None);

    let antes = ph2d_host_presos::antes_do_restauro(m.sim.world());
    respawn(&mut m);
    depois_com(&mut m, &antes);
    assert_ne!(m.bits[0], velho, "a régua: o refazer troca os bits");
    assert_eq!(
        m.h.store.loose_card(k).and_then(|c| c.pin),
        Some(m.bits[0]),
        "o refazer não prendeu o solto ao objecto de volta"
    );
    m.quadros(3);
    assert_eq!(m.h.store.loose_card(k).and_then(|c| c.pin), Some(m.bits[0]));
}

/// ⭐ A proporção do Scale anda com o objecto pela porta do restauro: a corrente fechada sobre A guarda
/// o par de A, e os bits novos herdam-no; a de quem o passo levou sai.
/// *Mutação: tirar `reata_travas_de_escala` de `depois_do_restauro` ⇒ os bits velhos ficam.*
#[test]
fn the_scale_chain_of_an_object_survives_the_respawn() {
    let mut m = mesa();
    for i in 0..3 {
        let e = ph2d_ecs::Entity::from_bits(m.bits[i]);
        m.sim.world_mut().entity_mut(e).insert(id_estavel(i));
    }
    assert!(!insp::scale_lock_on(), "a régua: a corrente começa aberta");
    clica(&mut m.h, insp::SCALE_LOCK);
    let velho = m.bits[0];
    assert_eq!(
        insp::travas_de_escala(),
        [velho],
        "a corrente fechou sobre A"
    );

    let antes = ph2d_host_presos::antes_do_restauro(m.sim.world());
    respawn(&mut m);
    depois_com(&mut m, &antes);
    assert_ne!(m.bits[0], velho);
    assert_eq!(
        insp::travas_de_escala(),
        [m.bits[0]],
        "a proporção ficou nos bits mortos"
    );

    let antes = ph2d_host_presos::antes_do_restauro(m.sim.world());
    respawn_sem(&mut m, &[0]);
    depois_com(&mut m, &antes);
    assert_eq!(
        insp::travas_de_escala(),
        Vec::<u64>::new(),
        "a de A devia sair"
    );

    // Abre: a corrente é da thread. Pelo evento: um 2.º clique no mesmo sítio seria um duplo clique.
    m.h.apply_event(ph2d_editor_core::interaction::WidgetEvent::Click(
        insp::SCALE_LOCK,
    ));
    assert!(!insp::scale_lock_on());
}

/// ⛔ `App` sem placa não tem `gfx`, e o `apply_project_with` volta na guarda dele: nenhum teste chega
/// lá. O que se pode provar sem placa é a forma: a única porta do restauro na shell lê o `antes` ANTES
/// do `restore`, e chama `depois_do_restauro` DEPOIS de o mapa novo e a seleção voltarem. A lei de cada
/// metade está nos testes acima. *Mutação: tirar a chamada, ou movê-la para antes do `pen.select_many`.*
#[test]
fn the_only_restore_door_of_the_shell_calls_the_two_halves_in_order() {
    let raiz = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../shells/desktop/src");
    let src = std::fs::read_to_string(raiz.join("undo_app.rs")).expect("undo_app.rs");
    let corpo = src
        .split_once("fn apply_project_with(")
        .expect("a porta do restauro")
        .1;
    let onde = |agulha: &str| {
        assert_eq!(
            corpo.matches(agulha).count(),
            1,
            "«{agulha}» em apply_project_with"
        );
        corpo.find(agulha).expect("contada")
    };
    let ordem = [
        "ph2d_host_presos::antes_do_restauro(gfx.sim.world())",
        "state.restore(s, reg)",
        "self.vec.entities = map;",
        "self.vec.pen.select_many(&alive);",
        "ph2d_host_presos::depois_do_restauro(",
    ]
    .map(onde);
    assert!(ordem.is_sorted(), "a ordem do restauro mudou: {ordem:?}");

    // E é a única: fora dos testes, ninguém mais chama `ProjectState::restore`.
    let mut pilha = vec![raiz];
    let mut portas = Vec::new();
    while let Some(d) = pilha.pop() {
        for e in std::fs::read_dir(d).expect("a pasta").flatten() {
            let p = e.path();
            if p.is_dir() {
                pilha.push(p);
            } else if p.extension().is_some_and(|x| x == "rs")
                && !p.to_string_lossy().ends_with("_tests.rs")
                && !p.to_string_lossy().contains("/tests")
                && std::fs::read_to_string(&p).is_ok_and(|s| {
                    s.contains(".restore(s, reg)")
                        || s.contains(".restore(sim, reg)")
                        || s.contains(".restore(&mut sim, &reg)")
                        || s.contains(".restore(&mut gfx.sim")
                })
            {
                portas.push(p.file_name().expect("nome").to_string_lossy().into_owned());
            }
        }
    }
    assert_eq!(
        portas,
        ["undo_app.rs"],
        "outra porta do restauro passa ao lado"
    );
}
