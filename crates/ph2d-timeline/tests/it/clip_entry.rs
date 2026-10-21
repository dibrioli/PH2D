//! ⭐⭐ O clipe de ENTRADA (v20, doc ObjectsAndComponents 01 §14, item 1): a migração v19 sem perda e
//! a Animação de cada objecto a tocar sozinha no Play, no relógio dele.

use ph2d_anim::{AnimValue, Interp, RationalTime};
use ph2d_timeline::{DOC_VERSION, PropKind, SCENE, TimelineDoc};

const V19_BIN: &[u8] = include_bytes!("../fixtures/timeline_doc_v19.bin");
const V19_JSON: &str = include_str!("../fixtures/timeline_doc_v19.json");

fn s(t: f64) -> RationalTime {
    RationalTime::from_seconds(t)
}

fn sem_o_que_a_v20_acrescentou(mut v: serde_json::Value) -> serde_json::Value {
    v.as_object_mut().unwrap().remove("version");
    v.as_object_mut().unwrap().remove("textos"); // v22
    for c in v["clips"].as_array_mut().unwrap() {
        c.as_object_mut().unwrap().remove("autoplay");
    }
    v
}

#[test]
fn um_ficheiro_v19_abre_sem_entrada_marcada_e_sem_perda() {
    let doc = TimelineDoc::from_bytes(V19_BIN).expect("o v19 tem de abrir");
    assert_eq!(doc.version, DOC_VERSION);
    assert!(doc.clips().iter().all(|c| !c.autoplay));
    let antes: serde_json::Value = serde_json::from_str(V19_JSON).unwrap();
    assert_eq!(
        sem_o_que_a_v20_acrescentou(serde_json::to_value(&doc).unwrap()),
        sem_o_que_a_v20_acrescentou(antes)
    );
    let de_novo = TimelineDoc::from_bytes(&doc.to_bytes().unwrap()).unwrap();
    assert_eq!(de_novo, doc);
}

/// ⭐ **O Autoplay do Godot** (dono, 10/10): por omissão NENHUM clipe toca sozinho; ligar um desliga os
/// irmãos; desligar deixa a animação sem nenhum; a Cena também tem o seu; uma variação (Duplicate) não
/// rouba a marca. (Mutações: ligar sem desligar os irmãos; o «por omissão o 1.º» de volta ⇒ RED.)
#[test]
fn o_autoplay_liga_e_desliga_e_por_omissao_nenhum() {
    let heroi = 7;
    let mut doc = TimelineDoc::new();
    let andar = doc.add_clip_for("andar".into(), heroi).unwrap();
    let pular = doc.add_clip_for("pular".into(), heroi).unwrap();
    assert_eq!(doc.entry_clip_of(heroi), None, "por omissão, nenhum");
    assert!(doc.autoplay_entries().is_empty(), "nada toca sozinho");
    assert!(doc.set_autoplay(pular, true));
    assert_eq!(doc.entry_clip_of(heroi), Some(pular));
    assert!(!doc.set_autoplay(pular, true), "já era");
    assert!(doc.set_autoplay(andar, true));
    assert!(!doc.clips()[pular].autoplay, "desliga a irmã");
    let copia = doc.duplicate_clip(andar).unwrap();
    assert!(!doc.clips()[copia].autoplay, "a variação não rouba a marca");
    assert!(doc.set_autoplay(0, true), "a Cena também tem o seu");
    assert_eq!(
        doc.autoplay_entries(),
        vec![(SCENE, 0), (heroi, andar)],
        "a Cena (Arrange vazio: o clipe dela) e depois o objecto"
    );
    assert!(doc.set_autoplay(andar, false));
    assert_eq!(
        doc.entry_clip_of(heroi),
        None,
        "desligado: o Herói não toca nada"
    );
}

/// ⭐ A Cena DIRIGE quem ela arranjou: um objecto cujo clipe está no Arrange não toca a entrada por
/// cima.
#[test]
fn um_objecto_arranjado_pela_cena_nao_toca_a_entrada() {
    let (heroi, vilao) = (7, 8);
    let mut doc = TimelineDoc::new();
    let andar = doc.add_clip_for("andar".into(), heroi).unwrap();
    let rir = doc.add_clip_for("rir".into(), vilao).unwrap();
    doc.set_autoplay(andar, true);
    doc.set_autoplay(rir, true);
    let lane = doc.add_lane("Base".into()).expect("uma lane");
    doc.add_strip(lane, andar, 0.0, 2.0)
        .expect("o clipe do Herói no Arrange");
    assert!(doc.arranged_owners().contains(&heroi));
    let quem: Vec<u64> = doc.autoplay_entries().iter().map(|e| e.0).collect();
    assert_eq!(quem, vec![vilao]);
}

/// O relógio do clipe: sem loop toca uma vez e fica; com o loop do Keys dobra como uma faixa; com
/// ida-e-volta reflecte; o corte de duração vale.
#[test]
fn o_relogio_do_clipe_dobra_pelo_loop_dele() {
    let mut doc = TimelineDoc::new();
    let c = doc.add_clip_for("andar".into(), 7).unwrap();
    assert_eq!(
        doc.clip_local_time(c, 3.5),
        3.5,
        "sem loop: o tempo corre (a pose fica na última chave)"
    );
    doc.set_active(c);
    doc.set_active_loop_for(true, Some((1.0, 3.0)));
    assert_eq!(doc.clip_local_time(c, 0.5), 0.5, "antes do loop: do início");
    assert_eq!(doc.clip_local_time(c, 2.5), 2.5);
    assert!(
        (doc.clip_local_time(c, 3.5) - 1.5).abs() < 1e-9,
        "volta ao início do loop"
    );
    doc.set_active_ping_pong_for(true, true);
    assert!(
        (doc.clip_local_time(c, 3.5) - 2.5).abs() < 1e-9,
        "ida-e-volta reflecte"
    );
}

fn x_de(world: &bevy_ecs::world::World, e: bevy_ecs::entity::Entity) -> f32 {
    world.get::<ph2d_ecs::Transform>(e).unwrap().translation.x
}

/// ⭐⭐ **No relógio dele:** o Herói toca a entrada sozinho; um pedido ([`ph2d_ecs::ClipPlayback`])
/// troca o clipe a partir do instante dele; um pedido do futuro (rewind) não vale. (Mutação:
/// `apply_autoplay` a ignorar o pedido ⇒ RED.)
#[test]
fn cada_objecto_toca_a_sua_animacao_e_um_pedido_troca_o_clipe() {
    let mut world = bevy_ecs::world::World::new();
    let e = world.spawn(ph2d_ecs::Transform::IDENTITY).id();
    let heroi = e.to_bits();
    let mut doc = TimelineDoc::new();
    let andar = doc.add_clip_for("andar".into(), heroi).unwrap();
    let pular = doc.add_clip_for("pular".into(), heroi).unwrap();
    for (clip, fim) in [(andar, 10.0), (pular, -10.0)] {
        doc.set_active(clip);
        doc.insert_key(
            heroi,
            PropKind::TranslationX,
            s(0.0),
            AnimValue::Float(0.0),
            Interp::Linear,
        );
        doc.insert_key(
            heroi,
            PropKind::TranslationX,
            s(1.0),
            AnimValue::Float(fim),
            Interp::Linear,
        );
    }
    doc.set_autoplay(andar, true);

    assert_eq!(
        ph2d_timeline::apply_autoplay(&mut world, &doc, 0.5, |_| false, &mut Vec::new()),
        1
    );
    assert!(
        (x_de(&world, e) - 5.0).abs() < 1e-4,
        "a entrada, no relógio dele"
    );

    world.entity_mut(e).insert(ph2d_ecs::ClipPlayback {
        clip: "pular".into(),
        desde: 2.0,
    });
    ph2d_timeline::apply_autoplay(&mut world, &doc, 2.25, |_| false, &mut Vec::new());
    assert!(
        (x_de(&world, e) + 2.5).abs() < 1e-4,
        "o pedido, desde o instante dele"
    );
    ph2d_timeline::apply_autoplay(&mut world, &doc, 0.25, |_| false, &mut Vec::new());
    assert!(
        (x_de(&world, e) - 2.5).abs() < 1e-4,
        "rewind para antes do pedido: a entrada"
    );
    assert!(
        world.get::<ph2d_ecs::ClipPlayback>(e).is_none(),
        "o pedido do futuro foi largado"
    );
    ph2d_timeline::apply_autoplay(&mut world, &doc, 2.25, |_| false, &mut Vec::new());
    assert!(
        (x_de(&world, e) - 10.0).abs() < 1e-4,
        "a passar outra vez pelo instante, o pedido NÃO revive: a entrada, na última pose"
    );
    assert!(world.get::<ph2d_ecs::ClipPlayback>(e).is_none());
    ph2d_timeline::apply_autoplay(&mut world, &doc, 0.25, |b| b == heroi, &mut Vec::new());
    assert!(
        (x_de(&world, e) - 10.0).abs() < 1e-4,
        "a mão do artista (skip) não é pisada"
    );
}

/// ⛔ Um **Play Clip** de um clipe que a animação não tem: o pedido é largado e DITO uma vez (uma
/// recusa na fila que a shell fala num toast); o objecto continua na entrada. (Mutação: sem o
/// `refusals.push` ⇒ RED.)
#[test]
fn um_pedido_de_um_clipe_que_nao_existe_e_dito_uma_vez() {
    let mut world = bevy_ecs::world::World::new();
    let e = world.spawn(ph2d_ecs::Transform::IDENTITY).id();
    let mut doc = TimelineDoc::new();
    doc.add_clip_for("andar".into(), e.to_bits()).unwrap();
    world.entity_mut(e).insert(ph2d_ecs::ClipPlayback {
        clip: "voar".into(),
        desde: 0.0,
    });
    let mut recusas = Vec::new();
    ph2d_timeline::apply_autoplay(&mut world, &doc, 0.5, |_| false, &mut recusas);
    assert_eq!(recusas, [ph2d_timeline::ClipRefusal::UnknownClip]);
    assert!(world.get::<ph2d_ecs::ClipPlayback>(e).is_none(), "largado");
    ph2d_timeline::apply_autoplay(&mut world, &doc, 0.6, |_| false, &mut recusas);
    assert_eq!(recusas.len(), 1, "dito UMA vez, não a cada quadro");
    assert_ne!(
        ph2d_i18n::tr(ph2d_timeline::ClipRefusal::UnknownClip.message_key()),
        ph2d_timeline::ClipRefusal::UnknownClip.message_key(),
        "a chave está na tabela"
    );
}

/// ⭐ **Com a cena a correr, a animação da CENA toca** quando o Arrange está vazio (2.º smoke do dono, 09/10: o
/// Inimigo, animado pela Cena, ficava parado); com Arrange, é ele que toca e o clipe da Cena não entra.
#[test]
fn com_o_arrange_vazio_a_cena_a_correr_toca_o_clipe_da_cena() {
    let mut world = bevy_ecs::world::World::new();
    let e = world.spawn(ph2d_ecs::Transform::IDENTITY).id();
    let inimigo = e.to_bits();
    let mut doc = TimelineDoc::new();
    doc.set_active(0);
    doc.set_autoplay(0, true);
    doc.insert_key(
        inimigo,
        PropKind::TranslationY,
        s(0.0),
        AnimValue::Float(0.0),
        Interp::Linear,
    );
    doc.insert_key(
        inimigo,
        PropKind::TranslationY,
        s(1.0),
        AnimValue::Float(8.0),
        Interp::Linear,
    );
    ph2d_timeline::apply_autoplay(&mut world, &doc, 0.5, |_| false, &mut Vec::new());
    let y = world.get::<ph2d_ecs::Transform>(e).unwrap().translation.y;
    assert!((y - 4.0).abs() < 1e-4, "o clipe da Cena tocou: y = {y}");

    let lane = doc.add_lane("Base".into()).expect("uma lane");
    doc.add_strip(lane, 0, 0.0, 2.0).expect("o Main no Arrange");
    assert!(
        doc.autoplay_entries().iter().all(|e| e.0 != SCENE),
        "com Arrange, é o Arrange que toca"
    );
}

/// ⭐ Sem Autoplay, o objecto não toca nada com a cena a correr — mas um clipe PEDIDO (Play Clip) toca
/// na mesma. (Mutação: só percorrer quem tem Autoplay ⇒ RED.)
#[test]
fn sem_autoplay_so_toca_o_que_alguem_pedir() {
    let mut world = bevy_ecs::world::World::new();
    let e = world.spawn(ph2d_ecs::Transform::IDENTITY).id();
    let heroi = e.to_bits();
    let mut doc = TimelineDoc::new();
    let pular = doc.add_clip_for("pular".into(), heroi).unwrap();
    doc.set_active(pular);
    doc.insert_key(
        heroi,
        PropKind::TranslationX,
        s(0.0),
        AnimValue::Float(0.0),
        Interp::Linear,
    );
    doc.insert_key(
        heroi,
        PropKind::TranslationX,
        s(1.0),
        AnimValue::Float(10.0),
        Interp::Linear,
    );
    assert_eq!(
        ph2d_timeline::apply_autoplay(&mut world, &doc, 0.5, |_| false, &mut Vec::new()),
        0,
        "sem Autoplay, parado"
    );
    world.entity_mut(e).insert(ph2d_ecs::ClipPlayback {
        clip: "pular".into(),
        desde: 0.0,
    });
    assert_eq!(
        ph2d_timeline::apply_autoplay(&mut world, &doc, 0.5, |_| false, &mut Vec::new()),
        1
    );
    assert!((x_de(&world, e) - 5.0).abs() < 1e-4, "o pedido toca");
}
