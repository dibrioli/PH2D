//! Os gates da PONTE da secção RAY SENSOR (suplente #21).

use super::*;
use ph2d_core::Vec2;
use ph2d_ecs::Transform;
use ph2d_physics_ecs::RaySensor;

fn olho(sim: &mut SimWorld, com_voz: bool) -> Entity {
    let e = sim
        .world_mut()
        .spawn((
            Name::new("Olho"),
            Transform::from_translation(Vec2::ZERO),
            RaySensor::default(),
        ))
        .id();
    if com_voz {
        sim.world_mut().entity_mut(e).insert(RaySignals {
            on_enter: "vi".into(),
            on_exit: String::new(),
        });
    }
    e
}

/// ⭐⭐⭐ **Um raio SEM VOZ é um raio legítimo, e a secção dele existe.**
///
/// ⚠️ Os dois componentes são separados de propósito (um objecto que só quer *«há chão?»* não paga
/// dois nomes que nunca usa), logo pedir os DOIS com um `?` esconderia a secção inteira de quem tem
/// só o sensor — e o artista ficaria sem sítio onde escrever o primeiro nome.
///
/// ⛔ E a ausência **não** é um vazio qualquer: ela lê-se como a queixa *«este raio não diz nada»*,
/// que é a terceira da escada.
///
/// **Mutação que deve sangrar:** exigir o `RaySignals` com um `?`.
#[test]
fn um_raio_sem_voz_tem_seccao() {
    let mut sim = SimWorld::new();
    forma(&mut sim, 0, None); // algo para ver: senão a queixa é a camada vazia
    let e = olho(&mut sim, false);
    let i = build_ray_info(
        sim.world(),
        &ph2d_tags::TagTree::new(),
        e.to_bits(),
        1,
        true,
        &[],
    )
    .expect("um raio sem voz tem de ter seccao");
    assert!(i.on_enter.is_empty() && i.on_exit.is_empty());
    assert_eq!(
        i.queixa(),
        Some(ph2d_editor_core::ray_edits::RayQueixa::Mudo),
        "sem nome nenhum a queixa e' "
    );

    // ⛔ O CONTROLO: com voz, a queixa muda — sem ele, um `queixa()` que devolvesse sempre `Mudo`
    // passaria.
    let mut sim2 = SimWorld::new();
    let e2 = olho(&mut sim2, true);
    assert_ne!(
        build_ray_info(
            sim2.world(),
            &ph2d_tags::TagTree::new(),
            e2.to_bits(),
            1,
            true,
            &[]
        )
        .expect("com voz tambem ha' seccao")
        .queixa(),
        Some(ph2d_editor_core::ray_edits::RayQueixa::Mudo)
    );
}

/// ⭐⭐ **A LEITURA VIVA vira um NOME aqui** — o painel mostra texto, e o canal publica uma entidade.
///
/// ⚠️ E um alvo que já não está no mundo devolve nome VAZIO em vez de estourar: o canal publica a
/// entidade do TIQUE, e um corpo apagado entre o tique e o quadro não tem nome. *Isso é honesto, e
/// é o que o `find` do publicador não pode saber.*
#[test]
fn a_leitura_viva_vira_um_nome() {
    let mut sim = SimWorld::new();
    let e = olho(&mut sim, true);
    let parede = sim
        .world_mut()
        .spawn((Name::new("Parede"), Transform::from_translation(Vec2::ZERO)))
        .id();
    let i = build_ray_info(
        sim.world(),
        &ph2d_tags::TagTree::new(),
        e.to_bits(),
        1,
        true,
        &[(parede, 1.75, parede)],
    )
    .expect("ha' seccao");
    assert_eq!(i.sees, "Parede");
    assert!((i.sees_at - 1.75).abs() < 1.0e-6);

    // Um alvo que já saiu da cena: nome vazio, e nada estoura.
    sim.world_mut().despawn(parede);
    let i2 = build_ray_info(
        sim.world(),
        &ph2d_tags::TagTree::new(),
        e.to_bits(),
        1,
        true,
        &[(parede, 1.75, parede)],
    )
    .expect("ha' seccao");
    assert!(i2.sees.is_empty(), "um alvo apagado nao tem nome a mostrar");
}

fn forma(sim: &mut SimWorld, layer: u8, pai: Option<Entity>) -> Entity {
    use ph2d_physics_ecs::{BodyKind, Collider, RigidBody};
    let w = sim.world_mut();
    let e = w
        .spawn((
            Transform::from_translation(Vec2::ZERO),
            Collider {
                layer,
                ..Collider::default()
            },
        ))
        .id();
    match pai {
        Some(p) => {
            w.entity_mut(e).insert(ph2d_ecs::ChildOf(p));
        }
        None => {
            w.entity_mut(e).insert(RigidBody {
                kind: BodyKind::Static,
            });
        }
    }
    e
}

/// ⭐⭐ **«Nothing in this layer» só fala quando é CERTO** (GEF 07, B·3): uma forma de outro corpo
/// numa camada que o raio vê cala-o; as formas do PRÓPRIO corpo não contam (o raio exclui-as); uma
/// camada que a matriz não deixa ver não conta.
///
/// **Mutações que devem sangrar:** contar as formas do corpo dono · ignorar a matriz · voltar a pôr o
/// `Option<&CollisionOverride>` na consulta (o aviso mente sempre, a foto de 15/11).
#[test]
fn a_camada_vazia_so_fala_quando_e_certo() {
    use ph2d_physics_ecs::{BodyKind, RigidBody};
    let mut sim = SimWorld::new();
    let corpo = sim
        .world_mut()
        .spawn((
            Transform::from_translation(Vec2::ZERO),
            RigidBody {
                kind: BodyKind::Dynamic,
            },
        ))
        .id();
    forma(&mut sim, 0, Some(corpo));
    let raio = olho(&mut sim, true);
    sim.world_mut()
        .entity_mut(raio)
        .insert(ph2d_ecs::ChildOf(corpo));
    let vazia = |sim: &SimWorld| {
        build_ray_info(
            sim.world(),
            &ph2d_tags::TagTree::new(),
            raio.to_bits(),
            1,
            true,
            &[],
        )
        .expect("seccao")
        .layer_empty
    };
    assert!(
        vazia(&sim),
        "só a forma do PRÓPRIO corpo: o raio nunca verá nada"
    );

    let chao = forma(&mut sim, 0, None);
    assert!(
        !vazia(&sim),
        "uma forma de outro corpo na camada 0 cala o aviso"
    );

    // A matriz diz que a camada 0 não vê a 0: o chão deixa de contar.
    let mut linhas = [u32::MAX; ph2d_physics_ecs::MAX_LAYERS];
    linhas[0] &= !1;
    ph2d_physics_ecs::excecao::LiveLayerRows::install(sim.world_mut(), linhas);
    assert!(vazia(&sim), "a matriz não deixa a camada 0 ver a 0");

    // Uma exceção própria na forma conta como visível (o aviso não adivinha).
    sim.world_mut()
        .entity_mut(chao)
        .insert(ph2d_physics_ecs::CollisionOverride::default());
    assert!(!vazia(&sim), "com exceção, o aviso cala-se");
}
