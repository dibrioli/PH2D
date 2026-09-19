//! **O RAIO persistente** (suplente #21) — os gates da lei.
//!
//! ⚠️ **Eles medem as TRÊS grandezas que a sonda nomeou** como o buraco que a composição não
//! exprime — **ordem · métrica · direcção** ([`mede_o_que_a_composicao_ja_da_ao_raio`]) —, mais as
//! duas metades que só a fiação tem: a aresta e o rebobinar.

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Name, SimWorld, StableId, Transform};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, PhysicsBridge, RaySensor, RaySignals, RigidBody,
    SignalTagFilter,
};
use ph2d_tags::TagTree;

fn parede(sim: &mut SimWorld, nome: &str, x: f32) -> Entity {
    sim.world_mut()
        .spawn((
            Name::new(nome),
            RigidBody {
                kind: BodyKind::Static,
            },
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x: 0.25,
                    half_y: 1.0,
                },
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(x, 0.0)),
        ))
        .id()
}

/// Um olho **sem corpo** — a forma mais magra que este componente aceita.
fn olho(sim: &mut SimWorld, em: Vec2, dir: Vec2, reach: f32) -> Entity {
    sim.world_mut()
        .spawn((
            Name::new("Olho"),
            Transform::from_translation(em),
            RaySensor {
                origin: Vec2::ZERO,
                dir,
                reach,
                layer: 0,
            },
        ))
        .id()
}

fn anda(sim: &mut SimWorld, bridge: &mut PhysicsBridge, ate: u64) {
    bridge.dispatch(sim, true, ate);
}

/// ⭐⭐⭐ **ORDEM e MÉTRICA — as duas colunas que a composição deixa vazias.**
///
/// A sonda mediu o outro lado: uma barra sensor sobre as duas paredes devolve **um** elemento
/// (`["Barra"]`) e **zero** contactos, logo nem a ordem nem a métrica existem por ali.
///
/// **Mutações que devem sangrar:** devolver o mais LONGE · deitar fora a distância · deitar fora a
/// normal.
#[test]
fn um_raio_ve_o_mais_perto_e_diz_a_que_distancia() {
    let mut sim = SimWorld::new();
    let perto = parede(&mut sim, "Perto", 2.0);
    let _longe = parede(&mut sim, "Longe", 5.0);
    let e = olho(&mut sim, Vec2::ZERO, Vec2::new(1.0, 0.0), 20.0);
    let mut bridge = PhysicsBridge::new();
    anda(&mut sim, &mut bridge, 1);

    let h = bridge
        .ray_sensor_hits()
        .get(&e)
        .copied()
        .expect("o raio tem de ver alguma coisa");
    assert_eq!(h.body, perto, "o raio devolveu a parede LONGE");
    // A face de cá da parede de `x = 2` com meia-largura `0,25` está em `1,75`.
    assert!(
        (h.distance - 1.75).abs() < 1.0e-3,
        "distancia lida: {} (esperava 1,75 — a FACE, nao o centro)",
        h.distance
    );
    assert!(
        (h.point[0] - 1.75).abs() < 1.0e-3,
        "ponto lido: {:?}",
        h.point
    );
    // ⭐ A normal é o que torna a REFLEXÃO exprimível, e é a coluna que a composição nunca dá.
    assert!(
        h.normal[0] < -0.9,
        "a normal tem de apontar CONTRA o raio: {:?}",
        h.normal
    );
}

/// ⭐⭐⭐ **DIRECÇÃO — a terceira coluna, e a que uma FORMA não pode ter.**
///
/// Uma barra sensor centrada na origem é **simétrica por construção** e apanha a parede de trás; a
/// sonda mediu-o. Um raio aponta.
///
/// **Mutação que deve sangrar:** o cast a ignorar o sentido de `dir`.
#[test]
fn um_raio_nao_ve_o_que_esta_atras() {
    let mut sim = SimWorld::new();
    let frente = parede(&mut sim, "Frente", 2.0);
    let _atras = parede(&mut sim, "Atras", -3.0);
    let e = olho(&mut sim, Vec2::ZERO, Vec2::new(1.0, 0.0), 20.0);
    let mut bridge = PhysicsBridge::new();
    anda(&mut sim, &mut bridge, 1);

    assert_eq!(
        bridge.ray_sensor_hits().get(&e).map(|h| h.body),
        Some(frente),
        "o raio viu para tras — uma FORMA faria isso, um raio nao"
    );

    // ⛔ O CONTROLO: virado ao contrário, ele vê a de trás e **não** a da frente.
    if let Some(mut r) = sim.world_mut().get_mut::<RaySensor>(e) {
        r.dir = Vec2::new(-1.0, 0.0);
    }
    let mut b2 = PhysicsBridge::new();
    b2.dispatch(&mut sim, true, 1);
    assert_eq!(
        b2.ray_sensor_hits().get(&e).map(|h| h.body),
        Some(_atras),
        "virado ao contrario ele tem de ver a de TRAS — sem isto a 1.ª metade \
         passaria com um raio que simplesmente nao alcanca"
    );
}

/// ⭐⭐ **Os dois vectores são LOCAIS, logo o raio RODA com o objecto.**
///
/// É isto que faz a mira de uma torreta seguir a torreta sem uma segunda lei — e escrevê-los em
/// mundo obrigaria o artista a reescrevê-los sempre que o objecto virasse.
///
/// ⚠️⚠️ **A ORIGEM é `(1, 0)` e não `(0, 0)`, e a 1.ª redacção deste gate tinha-a a zero:** ali
/// rodá-la é um **no-op**, logo a mutação que apagava a rotação da origem **SOBREVIVEU**. *Um
/// corpus no ponto NEUTRO de um knob não testa esse knob* — a lei que esta casa já pagou no
/// `Accumulate` do apagador de deslocamento e no `Step Scale` do L-System.
///
/// ⭐ **A fixtura discrimina as DUAS metades de uma vez:** com a pose a `+90°`, o `(1, 0)` local da
/// origem tem de virar `(0, 1)` em mundo e o `(1, 0)` local da direcção tem de virar «para cima».
/// A parede está em `x = 0`, logo um raio que nasça em `(1, 0)` — a origem **não** rodada — passa
/// ao lado dela, e um que aponte para `+x` também.
///
/// **Mutações que devem sangrar:** aplicar a pose só à origem, ou só à direcção.
#[test]
fn um_raio_roda_com_o_objecto() {
    let mut sim = SimWorld::new();
    let cima = parede(&mut sim, "Cima", 0.0);
    if let Some(mut t) = sim.world_mut().get_mut::<Transform>(cima) {
        t.translation = Vec2::new(0.0, 3.0);
    }
    let e = sim
        .world_mut()
        .spawn((
            Name::new("Olho"),
            Transform::from_translation(Vec2::ZERO),
            RaySensor {
                origin: Vec2::new(1.0, 0.0),
                dir: Vec2::new(1.0, 0.0),
                reach: 5.0,
                layer: 0,
            },
        ))
        .id();
    let mut bridge = PhysicsBridge::new();
    anda(&mut sim, &mut bridge, 1);
    assert!(
        bridge.ray_sensor_hits().get(&e).is_none(),
        "de pe' o raio nasce em (1, 0) e aponta para +X — nao ha' nada la'"
    );

    // Um quarto de volta: a origem vai para `(0, 1)` e o rumo passa a ser «para cima».
    if let Some(mut t) = sim.world_mut().get_mut::<Transform>(e) {
        t.rotation = std::f32::consts::FRAC_PI_2;
    }
    let mut b2 = PhysicsBridge::new();
    b2.dispatch(&mut sim, true, 1);
    let h = b2
        .ray_sensor_hits()
        .get(&e)
        .copied()
        .expect("o raio nao rodou com o objecto");
    assert_eq!(h.body, cima);
    // ⭐ E a DISTÂNCIA prova que ele nasceu em `(0, 1)` e não em `(0, 0)`: a face de baixo da parede
    // está em `y = 2`, logo um raio nascido na origem do objecto leria `2,0`.
    assert!(
        (h.distance - 1.0).abs() < 1.0e-3,
        "distancia lida: {} — esperava 1,0 (nascido em y = 1, face em y = 2)",
        h.distance
    );
}

/// ⭐⭐ **O corpo dono é EXCLUÍDO — e o gate do motor mede o que acontece sem isso**
/// (`the_caster_can_exclude_itself`: *«um personagem acha-se no chão para sempre»*).
///
/// **Mutação que deve sangrar:** passar `None` no lugar do handle.
#[test]
fn um_raio_nao_se_ve_a_si_mesmo() {
    let mut sim = SimWorld::new();
    let alvo = parede(&mut sim, "Alvo", 3.0);
    let eu = sim
        .world_mut()
        .spawn((
            Name::new("Eu"),
            RigidBody {
                kind: BodyKind::Static,
            },
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x: 0.5,
                    half_y: 0.5,
                },
                ..Collider::default()
            },
            Transform::from_translation(Vec2::ZERO),
            RaySensor {
                origin: Vec2::ZERO,
                dir: Vec2::new(1.0, 0.0),
                reach: 20.0,
                layer: 0,
            },
        ))
        .id();
    let mut bridge = PhysicsBridge::new();
    anda(&mut sim, &mut bridge, 1);
    assert_eq!(
        bridge.ray_sensor_hits().get(&eu).map(|h| h.body),
        Some(alvo),
        "o raio nasceu DENTRO do proprio corpo e achou-se a si mesmo"
    );
}

/// ⭐ **Um sensor num objecto SEM corpo não tem o que excluir** — e a resposta certa ali é não
/// excluir nada. O risco §5.2 do plano, MEDIDO em vez de suposto.
#[test]
fn um_raio_sem_corpo_nao_exclui_ninguem() {
    let mut sim = SimWorld::new();
    let alvo = parede(&mut sim, "Alvo", 2.0);
    let e = olho(&mut sim, Vec2::ZERO, Vec2::new(1.0, 0.0), 20.0);
    assert!(
        sim.world().get::<RigidBody>(e).is_none(),
        "a fixtura tem de ser SEM corpo, senao mede outra coisa"
    );
    let mut bridge = PhysicsBridge::new();
    anda(&mut sim, &mut bridge, 1);
    assert_eq!(
        bridge.ray_sensor_hits().get(&e).map(|h| h.body),
        Some(alvo),
        "um olho sem corpo tem de ver na mesma"
    );
}

/// ⭐⭐⭐ **O raio GRITA ao passar a ver e ao deixar de ver — dois nomes, duas perguntas.**
///
/// ⚠️ E o filtro por tag vale aqui **sem uma linha nova**, porque os dois canais passam pelo
/// `signal_events`, que é onde o `SignalTagFilter` mora.
///
/// **Mutações que devem sangrar:** publicar o mesmo nome nos dois extremos · saltar o filtro.
#[test]
fn o_raio_grita_ao_ver_e_ao_deixar_de_ver() {
    let mut sim = SimWorld::new();
    let alvo = parede(&mut sim, "Alvo", 2.0);
    let e = olho(&mut sim, Vec2::ZERO, Vec2::new(1.0, 0.0), 20.0);
    sim.world_mut().entity_mut(e).insert(RaySignals {
        on_enter: "vi".into(),
        on_exit: "perdi".into(),
    });
    let tree = TagTree::new();
    let mut bridge = PhysicsBridge::new();

    anda(&mut sim, &mut bridge, 1);
    let sinais = bridge.signal_events(&sim, &tree);
    assert_eq!(
        sinais.iter().map(|s| s.name.as_str()).collect::<Vec<_>>(),
        vec!["vi"],
        "o raio tinha de gritar a ENTRADA no tique em que passou a ver"
    );
    assert_eq!(sinais[0].source, e, "quem grita e' quem OLHA");
    assert_eq!(sinais[0].other, alvo, "e o `other` e' quem foi VISTO");

    // O alvo sai da cena: o raio deixa de ver.
    sim.world_mut().despawn(alvo);
    anda(&mut sim, &mut bridge, 2);
    assert_eq!(
        bridge
            .signal_events(&sim, &tree)
            .iter()
            .map(|s| s.name.as_str())
            .collect::<Vec<_>>(),
        vec!["perdi"],
        "sem o alvo o raio tinha de gritar a SAIDA"
    );

    // ⛔ E o tique seguinte é SILÊNCIO — uma saída é uma aresta, não um estado.
    anda(&mut sim, &mut bridge, 3);
    assert!(
        bridge.signal_events(&sim, &tree).is_empty(),
        "a saida repetiu-se — ela e' uma ARESTA, e um sinal por tique seria ruido"
    );
}

/// ⭐⭐ **Trocar de alvo é uma SAÍDA e uma ENTRADA, nunca um silêncio.**
///
/// Quem escuta *«vi inimigo»* tem de ouvir de novo quando o inimigo passa a ser outro — senão a
/// porta reage ao primeiro e ignora todos os seguintes.
///
/// **Mutação que deve sangrar:** o braço que compara `anterior.body == h.body` a devolver sempre
/// `true`.
#[test]
fn trocar_de_alvo_e_uma_saida_e_uma_entrada() {
    let mut sim = SimWorld::new();
    let perto = parede(&mut sim, "Perto", 2.0);
    let longe = parede(&mut sim, "Longe", 5.0);
    let e = olho(&mut sim, Vec2::ZERO, Vec2::new(1.0, 0.0), 20.0);
    let mut bridge = PhysicsBridge::new();
    anda(&mut sim, &mut bridge, 1);
    assert_eq!(
        bridge.ray_sensor_hits().get(&e).map(|h| h.body),
        Some(perto)
    );

    sim.world_mut().despawn(perto);
    anda(&mut sim, &mut bridge, 2);
    assert_eq!(
        bridge.ray_exits(),
        &[(e, perto)],
        "a saida do alvo antigo tem de soar"
    );
    assert_eq!(
        bridge.ray_enters(),
        &[(e, longe)],
        "e a entrada do novo, no MESMO tique"
    );
}

/// ⭐⭐ **A cerca por TAG vale para o raio, e custou zero linhas de filtro.**
///
/// **Mutação que deve sangrar:** o `passa(...)` a devolver sempre `true` no braço dos raios.
#[test]
fn a_cerca_de_tag_vale_para_o_raio() {
    let mut tree = TagTree::new();
    let inimigo = tree.create("Inimigo").expect("cria a tag");
    let mut sim = SimWorld::new();
    let alvo = parede(&mut sim, "Alvo", 2.0);
    // ⚠️ A identidade é lida por ACERTO (a lei que a wave das Tags pagou): sem um `StableId` no
    // mundo, a consulta de pertença devolve «ninguém» e o gate mediria a ausência da fixtura.
    sim.world_mut().entity_mut(alvo).insert(StableId(1));
    let e = olho(&mut sim, Vec2::ZERO, Vec2::new(1.0, 0.0), 20.0);
    sim.world_mut().entity_mut(e).insert((
        RaySignals {
            on_enter: "vi".into(),
            on_exit: String::new(),
        },
        SignalTagFilter(inimigo.0),
    ));
    let mut bridge = PhysicsBridge::new();
    anda(&mut sim, &mut bridge, 1);
    assert!(
        bridge.signal_events(&sim, &tree).is_empty(),
        "o alvo NAO tem a tag — o raio tinha de ficar calado"
    );

    // ⭐ O CONTROLO: com a tag posta, o mesmo raio grita.
    let mut sim2 = SimWorld::new();
    let alvo2 = parede(&mut sim2, "Alvo", 2.0);
    sim2.world_mut()
        .entity_mut(alvo2)
        .insert((StableId(1), ph2d_ecs::tags::Tags::from_ids([inimigo])));
    let e2 = olho(&mut sim2, Vec2::ZERO, Vec2::new(1.0, 0.0), 20.0);
    sim2.world_mut().entity_mut(e2).insert((
        RaySignals {
            on_enter: "vi".into(),
            on_exit: String::new(),
        },
        SignalTagFilter(inimigo.0),
    ));
    let mut b2 = PhysicsBridge::new();
    b2.dispatch(&mut sim2, true, 1);
    assert_eq!(
        b2.signal_events(&sim2, &tree)
            .iter()
            .map(|s| s.name.as_str())
            .collect::<Vec<_>>(),
        vec!["vi"],
        "com a tag posta o raio tem de gritar — sem este controlo a metade de cima \
         passaria com um raio que simplesmente nao ve' nada"
    );
}

/// ⭐⭐⭐ **REBOBINAR faz o raio voltar a não ver nada — o QUARTO mapa desta família.**
///
/// ⚠️ Sem isto o defeito é MUDO e ao contrário do que se espera: a 2.ª corrida começa com o raio a
/// «já ver», logo a entrada **não** volta a soar — uma porta que abriu ao ver o herói fica calada
/// para sempre depois do primeiro Reset. *Os três mapas irmãos foram esquecidos ali um de cada vez,
/// e o último custou um report do dono.*
///
/// **Mutação que deve sangrar:** tirar o `ray_hits.clear()` do `rebuild_from_rest`.
#[test]
fn rebobinar_faz_o_raio_voltar_a_nao_ver_nada() {
    let mut sim = SimWorld::new();
    let alvo = parede(&mut sim, "Alvo", 2.0);
    let e = olho(&mut sim, Vec2::ZERO, Vec2::new(1.0, 0.0), 20.0);
    sim.world_mut().entity_mut(e).insert(RaySignals {
        on_enter: "vi".into(),
        on_exit: String::new(),
    });
    let tree = TagTree::new();
    let mut bridge = PhysicsBridge::new();

    anda(&mut sim, &mut bridge, 5);
    assert_eq!(bridge.ray_sensor_hits().get(&e).map(|h| h.body), Some(alvo));
    assert_eq!(bridge.signal_events(&sim, &tree).len(), 1, "a 1.ª corrida");

    // O relógio volta ao princípio.
    bridge.dispatch(&mut sim, true, 0);
    // E anda outra vez: a entrada tem de soar DE NOVO.
    bridge.dispatch(&mut sim, true, 5);
    assert_eq!(
        bridge
            .signal_events(&sim, &tree)
            .iter()
            .map(|s| s.name.as_str())
            .collect::<Vec<_>>(),
        vec!["vi"],
        "depois de rebobinar a entrada tem de soar outra vez — o mapa ficou com o \
         que o raio via no fim da 1.ª corrida"
    );
}
