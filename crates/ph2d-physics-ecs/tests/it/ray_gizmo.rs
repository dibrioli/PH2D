//! **O RAIO no CANVAS** (suplente #21, W5) — os gates da LINHA que a ponte publica.
//!
//! ⚠️ **Eles medem outra coisa que o [`super::ray_sensor`]:** lá a pergunta é *«o raio viu o quê?»*
//! e aqui é *«o que é que o artista VÊ?»*. A segunda não se deduz da primeira — um raio pode estar
//! rigorosamente certo e ser desenhado com o comprimento errado, ou não ser desenhado de todo.
//!
//! # ⭐⭐⭐ Porque estes gates medem a MARCA e não o caminho de Vello
//!
//! O pintor é **um só** (`ph2d_app_physics::overlay::probes::probe_marks`), já existe e já tem os
//! gates de geometria dele — a linha, a ponta do alcance, o tique do acerto, os três estados. O que
//! a W5 acrescenta é um [`ProbeKind`] e **a publicação**, logo é a publicação que se mede aqui; o
//! lado do ecrã tem o gate irmão na crate do desenho.
//!
//! ⚠️ **Uma régua que conta QUANTAS marcas nunca vê QUAL linha** — a família que o `edge_max` global
//! e o `χ` cego à almofada já pagaram neste repo. Por isso nenhum destes gates se contenta com
//! `!marks.is_empty()`: cada um afirma um NÚMERO da geometria.

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Name, SimWorld, Transform};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, PhysicsBridge, ProbeKind, ProbeShape, ProbeState, RaySensor,
    RaySignals, RigidBody,
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

/// A única marca publicada, desmontada nos cinco números que o pintor consome.
fn a_marca(bridge: &PhysicsBridge) -> (ProbeState, [f32; 2], [f32; 2], f32, Option<f32>) {
    let marcas = bridge.ray_marks();
    assert_eq!(marcas.len(), 1, "esperava UMA marca, li {}", marcas.len());
    let m = marcas[0];
    assert_eq!(m.kind, ProbeKind::Sensor, "a marca e' de um raio autorado");
    match m.shape {
        ProbeShape::Ray {
            origin,
            dir,
            reach,
            hit,
            skin,
        } => {
            // ⛔ **Sem `skin`, e a ausência é a LEI:** o sensor do personagem nasce no CENTRO do
            // corpo (o `exclude_body` precisa disso) e desenhá-lo dali enterraria metade da linha
            // sob o contorno; aqui a origem é **autorada**, e encolher o desenho esconderia
            // exactamente o número que o artista está a mexer.
            assert_eq!(skin, 0.0, "um raio autorado nao tem pele a descontar");
            (m.state, origin, dir, reach, hit)
        }
        outra => panic!("um sensor de raio desenha-se como uma LINHA, li {outra:?}"),
    }
}

/// ⭐⭐⭐ **A linha desenhada É o raio que foi lançado** — origem, rumo e alcance em MUNDO.
///
/// A fixtura discrimina as três metades de uma vez: a pose está a `+90°` e a origem local é
/// `(1, 0)`, logo **quem não rodar a origem** publica `(1, 0)` em vez de `(0, 1)`, e **quem não
/// rodar a direcção** publica `(1, 0)` em vez de `(0, 1)`. A parede está por cima, a `y = 3`.
///
/// ⚠️ **O `hit` é a distância que o MOTOR devolveu**, e o gate prende-a ao que o painel lê: as duas
/// superfícies descrevem o mesmo acerto, e no dia em que discordarem uma delas está a mentir.
///
/// **Mutações que devem sangrar:** publicar a origem sem rodar · publicar o rumo sem rodar ·
/// publicar o alcance de outra fonte · deitar fora a distância do acerto.
#[test]
fn a_linha_desenhada_e_o_raio_que_foi_lancado() {
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
    if let Some(mut t) = sim.world_mut().get_mut::<Transform>(e) {
        t.rotation = std::f32::consts::FRAC_PI_2;
    }
    let mut bridge = PhysicsBridge::new();
    bridge.dispatch(&mut sim, true, 1);

    let (estado, origem, dir, reach, hit) = a_marca(&bridge);
    assert_eq!(estado, ProbeState::Hit, "ele ve' a parede de cima");
    assert!(
        (origem[0] - 0.0).abs() < 1.0e-4 && (origem[1] - 1.0).abs() < 1.0e-4,
        "a origem LOCAL (1, 0) a +90 graus nasce em (0, 1): li {origem:?}"
    );
    assert!(
        dir[0].abs() < 1.0e-4 && (dir[1] - 1.0).abs() < 1.0e-4,
        "o rumo LOCAL (1, 0) a +90 graus aponta para cima: li {dir:?}"
    );
    assert!(
        (reach - 5.0).abs() < 1.0e-6,
        "o alcance desenhado e' o autorado: li {reach}"
    );
    let d = hit.expect("acertou, logo a marca leva a distancia");
    let lido = bridge
        .ray_sensor_hits()
        .get(&e)
        .expect("o painel le' o mesmo acerto")
        .distance;
    assert!(
        (d - lido).abs() < 1.0e-6 && (d - 1.0).abs() < 1.0e-3,
        "o desenho e o painel descrevem o MESMO acerto: desenho {d}, painel {lido}"
    );
}

/// ⭐⭐⭐ **A direcção da marca é UNITÁRIA, e isto é uma correcção medida.**
///
/// A porta do motor normaliza **por dentro** (`the_reach_is_in_metres_whatever_the_direction_length`),
/// logo um `dir` autorado de `(1, 1)` com `reach = 3` casta **3 m**. Quem desenha calcula
/// `origem + dir·reach` ⇒ com o vector cru a linha mediria `3 · √2 = 4,243` e o overlay mentiria
/// sobre o alcance **exactamente para o artista que o está a afinar**.
///
/// **Mutação que deve sangrar:** publicar `gira(r.dir)` sem normalizar.
#[test]
fn a_direccao_da_marca_e_unitaria_senao_a_linha_mente_sobre_o_alcance() {
    let mut sim = SimWorld::new();
    olho(&mut sim, Vec2::ZERO, Vec2::new(1.0, 1.0), 3.0);
    let mut bridge = PhysicsBridge::new();
    bridge.dispatch(&mut sim, true, 1);

    let (_, origem, dir, reach, _) = a_marca(&bridge);
    let n = (dir[0] * dir[0] + dir[1] * dir[1]).sqrt();
    assert!(
        (n - 1.0).abs() < 1.0e-5,
        "o rumo tem de ser unitario: |{dir:?}| = {n}"
    );
    // E a consequência, na grandeza que o artista vê: o comprimento do que se desenha.
    let fim = [origem[0] + dir[0] * reach, origem[1] + dir[1] * reach];
    let comprimento = (fim[0] * fim[0] + fim[1] * fim[1]).sqrt();
    assert!(
        (comprimento - 3.0).abs() < 1.0e-4,
        "a linha desenhada mede o ALCANCE (3 m), nao 3·|dir| = 4,243: li {comprimento}"
    );
}

/// ⭐⭐ **Um raio que não vê nada CONTINUA desenhado** — e este é o argumento inteiro do overlay de
/// sondas, trazido para cá: *o que se desenha é ONDE ELE OLHA, e a intensidade diz o que aconteceu*.
///
/// ⚠️ Desenhar só o que acertou deixaria invisível justamente o raio cujo alcance é curto demais —
/// que é o caso em que o artista precisa de o ver.
///
/// **Mutação que deve sangrar:** publicar a marca dentro do braço do acerto.
#[test]
fn um_raio_que_nao_ve_nada_continua_desenhado() {
    let mut sim = SimWorld::new();
    // A parede está a 2 m e o alcance é 1 m: ele olha e não alcança.
    parede(&mut sim, "Longe", 2.0);
    let e = olho(&mut sim, Vec2::ZERO, Vec2::new(1.0, 0.0), 1.0);
    let mut bridge = PhysicsBridge::new();
    bridge.dispatch(&mut sim, true, 1);

    assert!(
        bridge.ray_sensor_hits().get(&e).is_none(),
        "controlo: a 1 m ele nao alcanca a parede de 2 m"
    );
    let (estado, _, _, reach, hit) = a_marca(&bridge);
    assert_eq!(
        estado,
        ProbeState::Clear,
        "perguntou e nao achou — nao e' `Idle`, que quer dizer «nao perguntei»"
    );
    assert_eq!(hit, None, "sem acerto nao ha' tique de acerto");
    assert!(
        (reach - 1.0).abs() < 1.0e-6,
        "e o ALCANCE continua desenhado, que e' o numero a afinar: li {reach}"
    );
}

/// ⛔ **Um raio DEGENERADO não se desenha** — as três entradas com que não se faz uma linha.
///
/// ⚠️ **As duas últimas coincidem HOJE com o que a porta do motor recusa, e isso é um FACTO e não
/// uma lei:** lá a razão é *«o parry responderia sobre um raio que não existe»* e aqui é *«não se
/// faz um vector unitário de um vector nulo, e um `NaN` chega ao caminho de Vello como um ponto que
/// não existe»*. Duas razões, o mesmo conjunto.
///
/// **Mutação que deve sangrar:** tirar a guarda do `raio_em_mundo`.
#[test]
fn um_raio_degenerado_nao_se_desenha() {
    for (nome, dir, reach) in [
        ("direccao nula", Vec2::ZERO, 2.0),
        ("direccao NaN", Vec2::new(f32::NAN, 0.0), 2.0),
        ("alcance NaN", Vec2::new(1.0, 0.0), f32::NAN),
        ("alcance negativo", Vec2::new(1.0, 0.0), -2.0),
    ] {
        let mut sim = SimWorld::new();
        parede(&mut sim, "Perto", 1.0);
        olho(&mut sim, Vec2::ZERO, dir, reach);
        let mut bridge = PhysicsBridge::new();
        bridge.dispatch(&mut sim, true, 1);
        assert!(
            bridge.ray_marks().is_empty(),
            "{nome}: nao ha' linha que desenhar, e uma marca daqui vira um ponto NaN no canvas"
        );
    }

    // ⭐ CONTROLO POSITIVO: o mesmo arranjo com números sãos publica a marca. Sem ele, uma guarda
    // que recusasse TUDO passaria nos quatro casos acima.
    let mut sim = SimWorld::new();
    parede(&mut sim, "Perto", 1.0);
    olho(&mut sim, Vec2::ZERO, Vec2::new(1.0, 0.0), 2.0);
    let mut bridge = PhysicsBridge::new();
    bridge.dispatch(&mut sim, true, 1);
    assert_eq!(
        bridge.ray_marks().len(),
        1,
        "controlo: o raio sao desenha-se"
    );
}

/// ⭐⭐⭐ **Com o solver DESARMADO a linha segue o corpo que a mão arrastou, e cala-se sobre o que
/// vê.**
///
/// ⚠️ O toggle *Physics* do transporte **nasce desmarcado**, logo este é o caminho de OMISSÃO do
/// app: sem esta metade um `RaySensor` seria invisível na configuração de fábrica.
///
/// ⚠️ **E o estado é `Idle` porque a lei NÃO correu.** Publicar uma resposta aqui seria descrever
/// uma corrida que não está a acontecer — e ela passaria pelos canais de aresta, emitindo um
/// *«vi o herói»* com a física desligada.
///
/// **Mutações que devem sangrar:** não re-derivar no `hold` · castar no `hold` · publicar `Clear`.
#[test]
fn com_o_solver_desarmado_a_linha_segue_o_corpo_e_cala_se() {
    let mut sim = SimWorld::new();
    let alvo = parede(&mut sim, "Alvo", 2.0);
    let e = olho(&mut sim, Vec2::ZERO, Vec2::new(1.0, 0.0), 20.0);
    let mut bridge = PhysicsBridge::new();

    anda(&mut sim, &mut bridge, 3);
    assert_eq!(
        bridge.ray_sensor_hits().get(&e).map(|h| h.body),
        Some(alvo),
        "controlo: com o solver armado ele ve' a parede"
    );

    // A mão arrasta o olho para `y = 5` e o artista desmarca o Physics.
    if let Some(mut t) = sim.world_mut().get_mut::<Transform>(e) {
        t.translation = Vec2::new(0.0, 5.0);
    }
    bridge.hold(&mut sim, 4);

    let (estado, origem, _, reach, hit) = a_marca(&bridge);
    assert_eq!(
        estado,
        ProbeState::Idle,
        "sem passo a lei nao correu — `Clear` diria «perguntei e nao achei»"
    );
    assert_eq!(hit, None, "e nao ha' acerto a desenhar");
    assert!(
        (origem[1] - 5.0).abs() < 1.0e-4,
        "a linha segue o corpo que a mao arrastou: li y = {}",
        origem[1]
    );
    assert!(
        (reach - 20.0).abs() < 1.0e-6,
        "e o alcance continua la', que e' o que se esta' a afinar"
    );
    assert!(
        bridge.ray_sensor_hits().is_empty(),
        "com o solver desarmado nao ha' leitura: uma de pe' descreve uma corrida que acabou"
    );
}

/// ⛔⛔ **Desarmar o Physics APAGA as arestas do raio** — a terceira metade da frase que o `hold` já
/// escreve para o contacto e para o gatilho, e uma CORRECÇÃO da W2.
///
/// As arestas são limpas no topo do `dispatch`, e o `hold` é chamado **em vez** dele: desarmar no
/// quadro em que um raio acabou de ver alguém deixava a entrada de pé, e o dreno do shell re-emitia
/// o sinal **em todo quadro, para sempre**.
///
/// **Mutação que deve sangrar:** tirar o `discard_ray_history()` do `hold`.
#[test]
fn desarmar_a_fisica_apaga_as_arestas_do_raio() {
    let mut sim = SimWorld::new();
    parede(&mut sim, "Alvo", 2.0);
    let e = olho(&mut sim, Vec2::ZERO, Vec2::new(1.0, 0.0), 20.0);
    sim.world_mut().entity_mut(e).insert(RaySignals {
        on_enter: "vi".into(),
        on_exit: String::new(),
    });
    let tree = TagTree::new();
    let mut bridge = PhysicsBridge::new();

    anda(&mut sim, &mut bridge, 1);
    assert_eq!(
        bridge.signal_events(&sim, &tree).len(),
        1,
        "controlo: a entrada soa no tique em que ele passa a ver"
    );

    // O artista desmarca o Physics no quadro seguinte.
    bridge.hold(&mut sim, 2);
    assert!(
        bridge.signal_events(&sim, &tree).is_empty(),
        "com a fisica desarmada a entrada nao pode continuar a soar — ela ficaria de pe' e o \
         dreno do shell re-emitia-a em TODO quadro"
    );
}

/// ⭐ **Rebobinar apaga a linha desenhada** — «rebobinar é RENASCER», na grandeza do canvas.
///
/// O laço de replay **não casta**, logo sem esta limpeza o canvas continuaria a desenhar os raios
/// onde os corpos estavam antes do Reset — um overlay que mente sobre o que o produto mede.
///
/// **Mutação que deve sangrar:** tirar o `ray_marks.clear()` do `rebuild_from_rest`.
#[test]
fn rebobinar_apaga_a_linha_desenhada() {
    let mut sim = SimWorld::new();
    parede(&mut sim, "Alvo", 2.0);
    olho(&mut sim, Vec2::ZERO, Vec2::new(1.0, 0.0), 20.0);
    let mut bridge = PhysicsBridge::new();

    anda(&mut sim, &mut bridge, 5);
    assert_eq!(bridge.ray_marks().len(), 1, "controlo: a corrida desenha");

    bridge.dispatch(&mut sim, true, 0);
    assert!(
        bridge.ray_marks().is_empty(),
        "depois do Reset a linha do tique anterior descreve uma corrida que deixou de existir"
    );

    // E ela volta assim que o relógio anda outra vez — a limpeza não é uma amputação.
    bridge.dispatch(&mut sim, true, 1);
    assert_eq!(
        bridge.ray_marks().len(),
        1,
        "e o quadro seguinte re-deriva: apagar nao pode ser apagar para sempre"
    );
}

fn anda(sim: &mut SimWorld, bridge: &mut PhysicsBridge, ate: u64) {
    bridge.dispatch(sim, true, ate);
}
