//! Os gates da PONTE das secções NAV REGION e NAV AGENT (plano 30, W4).

use super::*;
use ph2d_core::Vec2;
use ph2d_ecs::{Name, Transform};
use ph2d_editor_core::nav_edits::AgentQueixa;
use ph2d_physics_ecs::BodyKind;
use ph2d_topdown::TopDownLaw;

/// Um agente PRONTO: corpo, mover sem teclado, alvo por nome, dentro de uma região.
fn cena(sim: &mut SimWorld) -> (Entity, Entity) {
    let w = sim.world_mut();
    w.spawn((
        Name::new("Arena"),
        Transform::from_translation(Vec2::ZERO),
        NavRegion::default(),
    ));
    w.spawn((
        Name::new("Hero"),
        Transform::from_translation(Vec2::new(3.0, 0.0)),
    ));
    let agente = w
        .spawn((
            Name::new("Chaser"),
            Transform::from_translation(Vec2::new(-2.0, 0.0)),
            RigidBody {
                kind: BodyKind::Kinematic,
            },
            TopDownPlayer::from_law(TopDownLaw {
                default_controls: false,
                ..TopDownLaw::default()
            }),
            NavAgent {
                target: NavTarget::Named(stable_name_id("Hero")),
                ..NavAgent::default()
            },
        ))
        .id();
    let regiao = w.spawn((Name::new("So Regiao"), NavRegion::default())).id();
    (agente, regiao)
}

fn agente_de(sim: &SimWorld, e: Entity) -> InspectorNavAgent {
    build_info(sim.world(), e.to_bits(), 1, true)
        .and_then(|i| i.agent)
        .expect("a secção do agente existe")
}

/// ⭐ **Cada secção existe só para quem TEM o componente dela** (ADR-0166) — e um objecto sem
/// nenhum dos dois não tem instantâneo.
#[test]
fn cada_seccao_existe_so_para_quem_tem_o_componente() {
    let mut sim = SimWorld::new();
    let (agente, regiao) = cena(&mut sim);
    let nu = sim.world_mut().spawn(Transform::default()).id();
    assert!(build_info(sim.world(), nu.to_bits(), 1, true).is_none());
    let i = build_info(sim.world(), regiao.to_bits(), 1, true).unwrap();
    assert!(i.region.is_some() && i.agent.is_none());
    let i = build_info(sim.world(), agente.to_bits(), 1, true).unwrap();
    assert!(i.region.is_none() && i.agent.is_some());
}

/// ⭐⭐⭐ **As perguntas da PONTE chegam ao painel** — corpo, mover, teclado, plataforma, região.
///
/// ⚠️ Cada uma é desligada UMA de cada vez sobre o agente pronto, e a queixa tem de nomear essa —
/// senão um instantâneo que respondesse «sim» a todas deixaria o painel calado sobre um agente que a
/// ponte salta.
///
/// **Mutações que devem sangrar:** `has_body: true` fixo · `mover_reads_keys` a ler outro campo ·
/// `in_region` sempre `true`.
#[test]
fn as_perguntas_da_ponte_chegam_ao_painel() {
    let mut sim = SimWorld::new();
    let (agente, _) = cena(&mut sim);
    assert_eq!(
        agente_de(&sim, agente).queixa(),
        None,
        "o agente pronto não se queixa"
    );

    let mut com_teclado = TopDownPlayer::from_law(TopDownLaw::default());
    com_teclado.default_controls = true;
    sim.world_mut().entity_mut(agente).insert(com_teclado);
    assert_eq!(
        agente_de(&sim, agente).queixa(),
        Some(AgentQueixa::MoverLeTeclado)
    );

    sim.world_mut().entity_mut(agente).remove::<TopDownPlayer>();
    assert_eq!(
        agente_de(&sim, agente).queixa(),
        Some(AgentQueixa::SemMover)
    );

    sim.world_mut().entity_mut(agente).remove::<RigidBody>();
    assert_eq!(
        agente_de(&sim, agente).queixa(),
        Some(AgentQueixa::SemCorpo)
    );

    let mut sim = SimWorld::new();
    let (agente, _) = cena(&mut sim);
    sim.world_mut()
        .entity_mut(agente)
        .insert(PlatformPlayer::default());
    assert_eq!(
        agente_de(&sim, agente).queixa(),
        Some(AgentQueixa::ComPlataforma)
    );

    let mut sim = SimWorld::new();
    let (agente, _) = cena(&mut sim);
    sim.world_mut()
        .entity_mut(agente)
        .insert(Transform::from_translation(Vec2::new(50.0, 0.0)));
    assert_eq!(
        agente_de(&sim, agente).queixa(),
        Some(AgentQueixa::ForaDaRegiao),
        "a região de fábrica tem 10 m de meia-largura; a 50 m ele está fora dela"
    );
}

/// ⭐⭐ **O alvo vai e volta como NOME**, e um nome que ninguém tem lê-se PERDIDO — não vazio.
///
/// **Mutações que devem sangrar:** gravar o hash de `""` em vez de `0` · ler o nome pelos bits.
#[test]
fn o_alvo_vai_e_volta_como_nome() {
    let mut sim = SimWorld::new();
    let (agente, _) = cena(&mut sim);
    let a = agente_de(&sim, agente);
    assert_eq!(a.alvo_modo, NavAlvoModo::Objecto);
    assert_eq!(a.alvo_nome, "Hero");
    assert!(!a.alvo_perdido);

    assert!(apply_nav_edit(
        sim.world_mut(),
        agente.to_bits(),
        &NavFieldEdit::AlvoNome("  Arena ".into())
    ));
    assert_eq!(
        agente_de(&sim, agente).alvo_nome,
        "Arena",
        "quem apara é quem lê"
    );

    assert!(apply_nav_edit(
        sim.world_mut(),
        agente.to_bits(),
        &NavFieldEdit::AlvoNome("Ninguem".into())
    ));
    let a = agente_de(&sim, agente);
    assert!(a.alvo_perdido && a.alvo_nome.is_empty());
    assert_eq!(a.queixa(), Some(AgentQueixa::AlvoPerdido));

    assert!(apply_nav_edit(
        sim.world_mut(),
        agente.to_bits(),
        &NavFieldEdit::AlvoNome("   ".into())
    ));
    assert_eq!(
        sim.world().get::<NavAgent>(agente).unwrap().target,
        NavTarget::Named(0),
        "vazio é ZERO — a convenção de «ninguém» desta casa, nunca o hash de \"\""
    );
    assert_eq!(agente_de(&sim, agente).queixa(), Some(AgentQueixa::SemAlvo));
}

/// ⭐ **Trocar de modo preserva o alvo do modo em que ele JÁ está, e o ponto escreve-se eixo a
/// eixo** — escrever o `x` não pode apagar o `y`.
#[test]
fn o_modo_e_o_ponto_do_alvo() {
    let mut sim = SimWorld::new();
    let (agente, _) = cena(&mut sim);
    let b = agente.to_bits();
    apply_nav_edit(
        sim.world_mut(),
        b,
        &NavFieldEdit::AlvoModo(NavAlvoModo::Objecto),
    );
    assert_eq!(
        agente_de(&sim, agente).alvo_nome,
        "Hero",
        "o mesmo modo não apaga o nome"
    );

    apply_nav_edit(
        sim.world_mut(),
        b,
        &NavFieldEdit::AlvoModo(NavAlvoModo::Ponto),
    );
    apply_nav_edit(sim.world_mut(), b, &NavFieldEdit::AlvoX(4.0));
    apply_nav_edit(sim.world_mut(), b, &NavFieldEdit::AlvoY(-1.5));
    assert_eq!(
        sim.world().get::<NavAgent>(agente).unwrap().target,
        NavTarget::Point([4.0, -1.5])
    );
    assert_eq!(agente_de(&sim, agente).alvo_ponto, [4.0, -1.5]);

    apply_nav_edit(
        sim.world_mut(),
        b,
        &NavFieldEdit::AlvoModo(NavAlvoModo::Nenhum),
    );
    assert_eq!(
        sim.world().get::<NavAgent>(agente).unwrap().target,
        NavTarget::None
    );
}

/// ⚠️ **Os números negativos não existem** e a região lê a máscara inteira.
#[test]
fn os_numeros_do_agente_e_da_regiao() {
    let mut sim = SimWorld::new();
    let (agente, regiao) = cena(&mut sim);
    let (a, r) = (agente.to_bits(), regiao.to_bits());
    apply_nav_edit(sim.world_mut(), a, &NavFieldEdit::Radius(-1.0));
    apply_nav_edit(sim.world_mut(), a, &NavFieldEdit::Arrive(0.4));
    apply_nav_edit(sim.world_mut(), a, &NavFieldEdit::Repath(0.8));
    apply_nav_edit(sim.world_mut(), a, &NavFieldEdit::StuckAfter(2.0));
    apply_nav_edit(sim.world_mut(), a, &NavFieldEdit::Active(false));
    apply_nav_edit(
        sim.world_mut(),
        a,
        &NavFieldEdit::OnArrived("chegou".into()),
    );
    apply_nav_edit(sim.world_mut(), a, &NavFieldEdit::OnNoPath("longe".into()));
    apply_nav_edit(sim.world_mut(), a, &NavFieldEdit::OnStuck("preso".into()));
    let n = sim.world().get::<NavAgent>(agente).unwrap().clone();
    assert_eq!(
        n.radius, 0.0,
        "um raio negativo cai em 0, que é «derivado do colisor»"
    );
    assert_eq!(
        (
            n.arrive_distance,
            n.repath_distance,
            n.stuck_after_s,
            n.active
        ),
        (0.4, 0.8, 2.0, false)
    );
    assert_eq!(
        (
            n.on_arrived.as_str(),
            n.on_no_path.as_str(),
            n.on_stuck.as_str()
        ),
        ("chegou", "longe", "preso")
    );

    apply_nav_edit(sim.world_mut(), r, &NavFieldEdit::HalfW(-3.0));
    apply_nav_edit(sim.world_mut(), r, &NavFieldEdit::HalfH(2.5));
    apply_nav_edit(sim.world_mut(), r, &NavFieldEdit::ObstacleLayers(0b101));
    let g = *sim.world().get::<NavRegion>(regiao).unwrap();
    assert_eq!((g.half_extents, g.obstacle_layers), ([0.0, 2.5], 0b101));
    // ⛔ Uma edição de REGIÃO num objecto que só tem o AGENTE não toca em nada.
    assert!(!apply_nav_edit(
        sim.world_mut(),
        a,
        &NavFieldEdit::HalfW(1.0)
    ));
}

/// ⭐⭐ **A leitura VIVA sai do `NavNow`**, e sem ele não há número nenhum.
#[test]
fn a_leitura_viva_sai_do_nav_now() {
    let mut sim = SimWorld::new();
    let (agente, _) = cena(&mut sim);
    assert_eq!(
        agente_de(&sim, agente).agora,
        None,
        "antes do 1.º tique não há leitura"
    );
    sim.world_mut().entity_mut(agente).insert(NavNow {
        status: NavStatus::MovingPartial,
        remaining: 3.25,
        radius: 0.65,
    });
    assert_eq!(
        agente_de(&sim, agente).agora,
        Some(NavAgora {
            estado: NavEstado::Parcial,
            restante: 3.25,
            raio: 0.65,
        })
    );
}

/// ⭐ **(W5) «Avoid Others» vai e volta** — o componente nasce com o desvio, a edição desliga-o no
/// COMPONENTE, e o retrato do painel lê o que o componente tem.
///
/// **Mutações que devem sangrar:** o braço da aplicação não escrever · o retrato ler uma constante.
#[test]
fn o_desvio_vai_e_volta() {
    let mut sim = SimWorld::new();
    let (agente, _) = cena(&mut sim);
    assert!(agente_de(&sim, agente).avoidance, "nasce com o desvio");
    for quer in [false, true] {
        assert!(apply_nav_edit(
            sim.world_mut(),
            agente.to_bits(),
            &NavFieldEdit::Avoidance(quer)
        ));
        assert_eq!(sim.world().get::<NavAgent>(agente).unwrap().avoidance, quer);
        assert_eq!(agente_de(&sim, agente).avoidance, quer);
    }
}

/// ⭐⭐ **(W6) Os modos `Tag` e `Patrol` vão e voltam** — o modo escreve a variante dele, a tag
/// escolhida chega ao componente, e o NOME escrito na patrulha é o da FORMA (o modo vem primeiro).
///
/// **Mutações que devem sangrar:** o nome a escrever `Named` em patrulha · o modo `Tag` a não
/// preservar a tag · o braço do `AlvoTag` apagado.
#[test]
fn os_modos_da_w6_vao_e_voltam() {
    let mut sim = SimWorld::new();
    let (agente, _) = cena(&mut sim);
    let alvo = |sim: &SimWorld| sim.world().get::<NavAgent>(agente).map(|a| a.target);
    let w = sim.world_mut();
    assert!(apply_nav_edit(
        w,
        agente.to_bits(),
        &NavFieldEdit::AlvoModo(NavAlvoModo::Tag)
    ));
    assert_eq!(alvo(&sim), Some(NavTarget::NearestTagged(0)));
    apply_nav_edit(
        sim.world_mut(),
        agente.to_bits(),
        &NavFieldEdit::AlvoTag(42),
    );
    assert_eq!(alvo(&sim), Some(NavTarget::NearestTagged(42)));
    apply_nav_edit(
        sim.world_mut(),
        agente.to_bits(),
        &NavFieldEdit::AlvoModo(NavAlvoModo::Tag),
    );
    assert_eq!(
        alvo(&sim),
        Some(NavTarget::NearestTagged(42)),
        "repetir o modo apagou a tag"
    );
    let a = agente_de(&sim, agente);
    assert_eq!((a.alvo_modo, a.alvo_tag), (NavAlvoModo::Tag, 42));

    apply_nav_edit(
        sim.world_mut(),
        agente.to_bits(),
        &NavFieldEdit::AlvoModo(NavAlvoModo::Patrulha),
    );
    apply_nav_edit(
        sim.world_mut(),
        agente.to_bits(),
        &NavFieldEdit::AlvoNome(" Ronda ".into()),
    );
    assert_eq!(
        alvo(&sim),
        Some(NavTarget::Patrol(stable_name_id("Ronda"))),
        "na patrulha o nome é o da forma"
    );
}

/// ⭐⭐ **(W6) A patrulha diz quando o nome não é de uma forma DESENHADA** — ninguém com esse nome, ou
/// um objecto sem forma vectorial: as duas pedem a mesma cura.
///
/// **Mutação que deve sangrar:** aceitar um objecto sem `VecPathRef` como forma.
#[test]
fn a_patrulha_diz_quando_o_nome_nao_e_de_uma_forma() {
    let mut sim = SimWorld::new();
    let (agente, _) = cena(&mut sim);
    // «Hero» existe mas não é uma forma desenhada.
    sim.world_mut()
        .get_mut::<NavAgent>(agente)
        .expect("o agente")
        .target = NavTarget::Patrol(stable_name_id("Hero"));
    let a = agente_de(&sim, agente);
    assert_eq!(a.alvo_nome, "Hero");
    assert_eq!(a.queixa(), Some(AgentQueixa::SemForma));
    // O CONTROLO: com uma forma, a queixa some.
    sim.world_mut().spawn((
        Name::new("Ronda"),
        ph2d_ecs::VecPathRef(1),
    ));
    sim.world_mut()
        .get_mut::<NavAgent>(agente)
        .expect("o agente")
        .target = NavTarget::Patrol(stable_name_id("Ronda"));
    assert_eq!(agente_de(&sim, agente).queixa(), None);
}
