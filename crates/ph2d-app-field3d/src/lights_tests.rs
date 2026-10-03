//! Os gates do [`super`].

use super::*;

/// A lâmpada do rig de omissão, em espaço de VISTA (o `y` do canvas virado) e como radiância
/// (`π × cor × intensidade`) — o que o sombreamento traçado do Render recebia até 03/10.
fn lampada_do_rig() -> Vec<([f32; 3], [f32; 3])> {
    ph2d_light::resolve(&ph2d_light::LightRig::default())
        .map(|r| {
            r.lamps()
                .iter()
                .map(|l| {
                    (
                        [l.dir[0], -l.dir[1], l.dir[2]],
                        l.tint.map(|t| t * core::f32::consts::PI),
                    )
                })
                .collect()
        })
        .unwrap_or_default()
}

/// ⭐ **O `π` da luz-objecto é o MESMO da lâmpada de estúdio** — uma luz branca de força `1` a UMA
/// unidade entrega o que a lâmpada do rig entregava.
///
/// ⛔ Sem isto, a escala de intensidade do modelador seria uma segunda escala, e o artista teria de
/// aprender duas.
#[test]
fn a_light_of_one_at_one_unit_is_the_lamp_the_rig_had() {
    let l = ph2d_field_ecs::FieldLight::default();
    let do_rig = lampada_do_rig();
    assert_eq!(do_rig.len(), 1, "o rig de omissão tem UMA lâmpada acesa");
    for c in 0..3 {
        assert!(
            (radiance_at_one(l)[c] - do_rig[0].1[c]).abs() < 1.0e-6,
            "canal {c}: {:?} contra {:?}",
            radiance_at_one(l),
            do_rig[0].1
        );
    }
}

/// ⚠️ **Força e cor negativas não escurecem a peça** — elas são coadas, porque a subtracção de luz
/// não existe e o que ela produziria é um pixel `NaN` a jusante.
#[test]
fn a_negative_light_is_no_light_never_a_dark_one() {
    let mau = ph2d_field_ecs::FieldLight {
        intensity: -5.0,
        color: [-1.0, 0.5, -0.2],
    };
    assert_eq!(radiance_at_one(mau), [0.0; 3]);
}

/// ⭐⭐⭐ **O SÍTIO DA PRIMEIRA LUZ É A DIRECÇÃO DO RIG QUE ELA SUBSTITUI** — e sai da porta, nunca de
/// um literal.
///
/// ⚠️ **A primeira redacção era um `[f32; 3]` const em MUNDO**, e isso é a mesma classe de erro que o
/// sinal de `y` desta casa já pagou: a direcção do rig é de **ECRÃ**, e onde «superior-esquerda» cai
/// no mundo depende de para onde a câmera olha.
#[test]
fn the_first_light_is_born_where_the_rig_lamp_shone_from() {
    let cam = ph2d_field_render::Orbit::default();
    let p = opening_place(&cam);
    let (right, up, toward_eye) = cam.basis();
    let d = [0, 1, 2].map(|i| p[i] - cam.target[i]);
    let dot = |a: [f32; 3], b: [f32; 3]| a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
    // ⚠️ **Normalizada**: o que se compara é a DIRECÇÃO — a distância é a
    // [`opening_distance`], e ela tem lei própria.
    let n = dot(d, d).sqrt();
    assert!(
        (n - opening_distance(&cam)).abs() < 1.0e-5,
        "a luz nasceu a {n} e a lei pede {}",
        opening_distance(&cam)
    );
    let em_vista = [dot(d, right) / n, dot(d, up) / n, dot(d, toward_eye) / n];
    let lampada = lampada_do_rig()[0].0;
    for c in 0..3 {
        assert!(
            (em_vista[c] - lampada[c]).abs() < 1.0e-5,
            "a luz nasce em {em_vista:?} e a lâmpada do rig vinha de {lampada:?}"
        );
    }
    // ⭐⭐ **E a FORÇA é `r²`** — é ela que faz a peça receber, no centro, o que a lâmpada do rig
    // lhe dava. *Uma luz num sítio novo com a força velha seria uma cena que escurece sozinha.*
    let (_, lampada_nova) = opening_light(&cam);
    let r = opening_distance(&cam);
    assert!(
        (lampada_nova.intensity - r * r).abs() < 1.0e-5,
        "a força é {} e a lei pede {}",
        lampada_nova.intensity,
        r * r
    );
    // ⭐ **E com uma câmera OUTRA, o mundo muda e a vista não** — é isto que um literal não faz.
    let mut outra = ph2d_field_render::Orbit::from_yaw_pitch(1.1, -0.4);
    outra.target = [0.3, -0.2, 0.7];
    let q = opening_place(&outra);
    assert!(
        (0..3).any(|i| (q[i] - p[i]).abs() > 1.0e-3),
        "o sítio não seguiu a câmera: {q:?} contra {p:?}"
    );
}

/// ⭐⭐⭐ **O PAINEL DE UMA LUZ OFERECE EXACTAMENTE O QUE ELA FAZ** — a lei W34, sobre o objecto novo.
///
/// ⛔ **A rotação e a escala NÃO são oferecidas**, e isso é a metade que interessa: um ponto não tem
/// orientação nem tamanho, e três sliders de ângulo que não movem um pixel são o controlo morto que
/// aquela lei proíbe por escrito.
#[test]
fn the_panel_of_a_light_offers_no_angle_and_no_size() {
    let mut world = bevy_ecs::world::World::new();
    let luz = ph2d_field_ecs::add_light(
        &mut world,
        [1.0, 2.0, 3.0],
        ph2d_field_ecs::FieldLight::default(),
    );
    let linhas = ph2d_field_ecs::params_of(&world, luz);
    let chaves: Vec<ph2d_field::Param> = linhas.iter().map(|(p, _)| *p).collect();
    assert_eq!(
        chaves,
        vec![
            ph2d_field::Param::Pos(0),
            ph2d_field::Param::Pos(1),
            ph2d_field::Param::Pos(2),
            ph2d_field::Param::Light(0),
            ph2d_field::Param::Light(1),
            ph2d_field::Param::Light(2),
            ph2d_field::Param::Light(3),
        ],
        "as linhas de uma luz mudaram"
    );
    // ⭐ **A POSIÇÃO é a da entidade**, e não uma cópia: o que o painel mostra é a pose.
    assert!((linhas[0].1.value - 1.0).abs() < 1.0e-6);
    assert!((linhas[2].1.value - 3.0).abs() < 1.0e-6);
    // ⭐ **E o gesto de a mover é o MESMO** — a porta que move uma forma move uma lâmpada.
    ph2d_field_ecs::set_param(&mut world, luz, ph2d_field::Param::Pos(0), -4.0).expect("mover");
    assert_eq!(of_the_world(&mut world)[0].world[0], -4.0);
}

/// ⭐⭐ **O OLHO DA HIERARQUIA APAGA A LUZ E NÃO A MARCA** — e de graça, porque é o mesmo componente
/// que esconde uma forma. *Uma luz que só se desliga apagando-a é uma luz que ninguém experimenta.*
///
/// # ⚠️ As duas metades, e a segunda entrou com o gizmo (14/09)
///
/// A recolha do MÓDULO tem **todas** as luzes; quem filtra pelo olho é o [`lamps_of`], que é o
/// consumidor a quem o olho diz respeito. ⛔ Filtrar na recolha tirava a luz apagada também do
/// **canvas**, e uma luz sem marca só se voltaria a acender pela Hierarquia — que é de onde o gizmo
/// a tirou. *A primeira redacção deste gate afirmava exactamente isso, e foi o gizmo que a corrigiu.*
#[test]
fn the_hierarchy_eye_switches_the_light_off_and_not_the_mark() {
    let mut world = bevy_ecs::world::World::new();
    let luz = ph2d_field_ecs::add_light(
        &mut world,
        [1.0, 1.0, 1.0],
        ph2d_field_ecs::FieldLight::default(),
    );
    assert_eq!(lamps_of(&of_the_world(&mut world)).len(), 1);
    world
        .entity_mut(luz)
        .insert(ph2d_ecs::Visibility { hidden: true });
    let recolhidas = of_the_world(&mut world);
    assert_eq!(
        recolhidas.len(),
        1,
        "a luz apagada saiu da recolha — o canvas fica sem a marca dela"
    );
    assert!(!recolhidas[0].on, "o olho não chegou à recolha");
    assert!(
        lamps_of(&recolhidas).is_empty(),
        "a luz escondida continuou a acender"
    );
    // ⭐ E o controlo: sem o `hidden`, ela volta a acender.
    world
        .entity_mut(luz)
        .insert(ph2d_ecs::Visibility { hidden: false });
    assert_eq!(lamps_of(&of_the_world(&mut world)).len(), 1);
}

/// ⚠️ **A ordem das luzes é ESTÁVEL** — a soma de `f32` não é associativa, e sem isto o mesmo
/// documento daria duas imagens conforme a ordem em que o ECS calhou de arrumar os arquétipos.
#[test]
fn the_lights_come_out_in_a_stable_order() {
    let mut world = bevy_ecs::world::World::new();
    let a = ph2d_field_ecs::add_light(
        &mut world,
        [1.0, 0.0, 0.0],
        ph2d_field_ecs::FieldLight::default(),
    );
    let b = ph2d_field_ecs::add_light(
        &mut world,
        [2.0, 0.0, 0.0],
        ph2d_field_ecs::FieldLight::default(),
    );
    let _ = ph2d_field_ecs::add_light(
        &mut world,
        [3.0, 0.0, 0.0],
        ph2d_field_ecs::FieldLight::default(),
    );
    let antes: Vec<f32> = of_the_world(&mut world)
        .iter()
        .map(|l| l.world[0])
        .collect();
    assert_eq!(antes.len(), 3);
    // ⚠️ **A perturbação é a que importa:** inserir um componente MUDA o arquétipo da entidade, e é
    // isso que reordena uma varredura de ECS. *Sem esta metade o gate só media que três é três.*
    world.entity_mut(b).insert(ph2d_ecs::Name::new("Zzz"));
    world
        .entity_mut(a)
        .insert(ph2d_ecs::Visibility { hidden: false });
    let depois: Vec<f32> = of_the_world(&mut world)
        .iter()
        .map(|l| l.world[0])
        .collect();
    assert_eq!(
        antes, depois,
        "a ordem mudou ao tocar nas luzes — a soma de `f32` não é associativa, logo isto são duas \
         imagens do mesmo documento"
    );
    // ⛔ **E ela NÃO é a ordem de criação**, que é o que se leria por engano: os bits de uma
    // entidade não crescem com o `spawn`. O que a lei promete é ser a **mesma** todas as vezes.
    assert_ne!(
        antes,
        vec![1.0, 2.0, 3.0],
        "a ordem calhou de ser a de criação: este assert existe para o leitor seguinte não a \
         confundir com a lei — se ela passar a sê-lo, o comentário acima é que está errado"
    );
}

/// ⭐⭐⭐ **UMA LUZ É UMA LINHA DA HIERARQUIA** — e o gate mede-o pela **mesma query** que a casa usa,
/// não por uma lista de componentes escrita à mão.
///
/// # ⛔⛔ O defeito que ele fecha
///
/// A primeira redacção do `add_light` dava `Name` e `FieldPose` e mais nada. A luz existia, iluminava
/// a peça e **não tinha linha nenhuma**: não se podia escolher, renomear, esconder nem apagar. *Uma
/// luz que não aparece na Hierarquia não é um objecto 3D — é uma variável global com uma posição*, e
/// era exactamente o oposto da ordem do dono.
///
/// ⚠️ **A query é `With<Transform>, Without<ChildOf>`** (`ph2d_ecs::HierarchyWalkState`), e este gate
/// escreve-a por extenso de propósito: se ela mudar do outro lado, o que se lê aqui é o gate a ficar
/// verde sobre uma luz invisível. *A cerca contra isso é o piso de população logo abaixo.*
#[test]
fn a_light_is_a_row_of_the_hierarchy() {
    use bevy_ecs::prelude::{With, Without};
    let mut world = bevy_ecs::world::World::new();
    let doc = ph2d_field::FieldDoc::new(
        vec![ph2d_field_eval::leaf(
            ph2d_field::Primitive::Sphere { radius: 0.3 },
            ph2d_field::Xform::IDENTITY,
        )],
        ph2d_field::NodeId(0),
    )
    .expect("a peça");
    ph2d_field_ecs::spawn_doc(&mut world, &doc, "Model");
    let luz = ph2d_field_ecs::add_light(
        &mut world,
        [1.0, 1.0, 1.0],
        ph2d_field_ecs::FieldLight::default(),
    );

    let mut q = world.query_filtered::<bevy_ecs::entity::Entity, (
        With<ph2d_ecs::Transform>,
        Without<bevy_ecs::hierarchy::ChildOf>,
    )>();
    let raizes: Vec<bevy_ecs::entity::Entity> = q.iter(&world).collect();
    // ⚠️ **PISO DE POPULAÇÃO**: a peça tem de estar lá também, senão um dia em que a query deixe de
    // casar seja o que for este gate leria «zero raízes» e passaria por vacuidade.
    assert_eq!(raizes.len(), 2, "raízes: {raizes:?}");
    assert!(raizes.contains(&luz), "a luz não é uma raiz da Hierarquia");
    // ⭐ **E ela tem NOME** — é a linha que o artista lê.
    assert_eq!(
        world.get::<ph2d_ecs::Name>(luz).map(|n| n.0.clone()),
        Some("Light".to_string())
    );
    // ⭐ **A segunda não se chama «Light» outra vez** — duas linhas iguais são duas linhas que o
    // artista não consegue distinguir.
    let luz2 =
        ph2d_field_ecs::add_light(&mut world, [2.0; 3], ph2d_field_ecs::FieldLight::default());
    assert_eq!(
        world.get::<ph2d_ecs::Name>(luz2).map(|n| n.0.clone()),
        Some("Light 2".to_string())
    );
}
