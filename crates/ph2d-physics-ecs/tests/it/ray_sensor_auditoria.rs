//! **Os gates que faltavam ao Ray Sensor** (auditoria GEF 07, 15/11, `componentes/07_auditoria_2026-11-15.md`):
//! C1 (parede que contém a origem), C3 (a marca bloqueada), C4 (trocar «See all hits» a meio), C5 (o lado do
//! herói sob a espada) e C8 (até onde vai a família). Cada um tem o controlo na mesma fixtura e diz a mutação
//! que o sangra.

use crate::ray_sensor_versao_c::{caixa, com_tag, marca, para, parede, raio, ve};
use ph2d_core::Vec2;
use ph2d_ecs::{ChildOf, Entity, Name, SimWorld, StableId, Transform};
use ph2d_physics_ecs::{PhysicsBridge, PlatformPlayer, RaySensor, RaySignals, SeeFilter};
use ph2d_tags::TagTree;

/// O raio olha para a direita a partir da origem, só para a tag `Inimigo`.
fn olho(tree: &TagTree, walls_block: bool, all_hits: bool) -> RaySensor {
    RaySensor {
        only_see: SeeFilter::OnlyTag,
        see_tag: tree.find("Inimigo").expect("tag").0,
        walls_block,
        all_hits,
        ..para(0.0)
    }
}

/// ⭐⭐ **C1 — a origem DENTRO de uma parede sem a tag, com «Walls block»: o raio não vê através dela.**
/// Controlos: sem «Walls block» ele atravessa e vê o alvo; com «Everything» a parede que o contém é
/// ignorada (Godot `hit_from_inside = false`), e ele vê o alvo. O caminho «See all hits» igual.
///
/// **Mutação que sangra:** o `accept` ignorar `dentro` (a forma que contém a origem sempre fora).
#[test]
fn walls_block_com_a_origem_dentro_de_uma_parede_tapa() {
    for all_hits in [false, true] {
        let mut tree = TagTree::new();
        let inimigo = tree.create("Inimigo").expect("tag");
        let cena = |r: RaySensor| {
            let mut sim = SimWorld::new();
            caixa(&mut sim, "Muro", 0.0, 0.5, false);
            let alvo = parede(&mut sim, "Alvo", 4.0);
            com_tag(&mut sim, alvo, 1, inimigo);
            let e = raio(&mut sim, Vec2::ZERO, None, r);
            let mut b = PhysicsBridge::new();
            b.set_tag_tree(&tree);
            b.dispatch(&mut sim, true, 1);
            (b.ray_sensor_hits().get(&e).map(|h| h.body), marca(&b), alvo)
        };
        let (visto, (fim, hit, _), _) = cena(olho(&tree, true, all_hits));
        assert_eq!(visto, None, "all_hits = {all_hits}: a parede tapa");
        assert!(
            fim.abs() < 1.0e-3 && hit.is_none(),
            "a linha pára na origem: {fim} {hit:?}"
        );
        let (visto, _, alvo) = cena(olho(&tree, false, all_hits));
        assert_eq!(visto, Some(alvo), "sem Walls block atravessa");
        let (visto, _, alvo) = cena(RaySensor {
            all_hits,
            ..para(0.0)
        });
        assert_eq!(
            visto,
            Some(alvo),
            "Everything: a parede que o contém é ignorada"
        );
    }
}

/// ⭐⭐ **C3 — a parede que tapa NÃO é um acerto no desenho**: a linha pára nela, sem `hit` (estado
/// `Clear`), como o painel e os sinais dizem («não vê»). Nos dois caminhos (um acerto e «See all hits»).
///
/// **Mutação que sangra:** no caminho de um acerto, `parou` vir do acerto cru (a parede).
#[test]
fn a_marca_bloqueada_nao_e_um_acerto() {
    for all_hits in [false, true] {
        let mut tree = TagTree::new();
        let inimigo = tree.create("Inimigo").expect("tag");
        let mut sim = SimWorld::new();
        parede(&mut sim, "Parede", 2.0);
        let alvo = parede(&mut sim, "Alvo", 4.0);
        com_tag(&mut sim, alvo, 1, inimigo);
        let e = raio(&mut sim, Vec2::ZERO, None, olho(&tree, true, all_hits));
        let mut b = PhysicsBridge::new();
        b.set_tag_tree(&tree);
        b.dispatch(&mut sim, true, 1);
        assert_eq!(b.ray_sensor_hits().get(&e), None, "all_hits = {all_hits}");
        let (fim, hit, _) = marca(&b);
        assert!(
            (fim - 1.75).abs() < 1.0e-3,
            "a linha acaba na parede: {fim}"
        );
        assert_eq!(hit, None, "all_hits = {all_hits}: sem acerto no desenho");
        assert_eq!(
            b.ray_marks()[0].state,
            ph2d_physics_ecs::ProbeState::Clear,
            "all_hits = {all_hits}"
        );
    }
}

/// ⭐⭐ **C4 — trocar «See all hits» a meio não inventa arestas.** Duas paredes; ligado vê as duas.
/// Desligar: sai só a de longe (a de perto continua vista). Religar: entra só a de longe.
///
/// **Mutação que sangra:** voltar a saltar as arestas de um acerto no tique da troca (`fora(e)`).
#[test]
fn trocar_all_hits_no_meio_nao_inventa_arestas() {
    let mut sim = SimWorld::new();
    let perto = parede(&mut sim, "Perto", 2.0);
    let longe = parede(&mut sim, "Longe", 4.0);
    let r = raio(
        &mut sim,
        Vec2::ZERO,
        None,
        RaySensor {
            all_hits: true,
            ..para(0.0)
        },
    );
    sim.world_mut().entity_mut(r).insert(RaySignals {
        on_enter: "vi".into(),
        on_exit: "perdi".into(),
    });
    let mut b = PhysicsBridge::new();
    b.dispatch(&mut sim, true, 1);
    assert_eq!(b.ray_enters(), &[(r, perto), (r, longe)]);
    let troca = |sim: &mut SimWorld, v: bool| {
        if let Some(mut s) = sim.world_mut().get_mut::<RaySensor>(r) {
            s.all_hits = v;
        }
    };
    troca(&mut sim, false);
    b.dispatch(&mut sim, true, 2);
    assert_eq!(b.ray_exits(), &[(r, longe)], "desligar: sai só a de longe");
    assert!(b.ray_enters().is_empty(), "desligar: ninguém entra");
    assert_eq!(b.ray_sensor_hits().get(&r).map(|h| h.body), Some(perto));
    b.dispatch(&mut sim, true, 3);
    assert!(
        b.ray_exits().is_empty() && b.ray_enters().is_empty(),
        "parado: nada"
    );
    troca(&mut sim, true);
    b.dispatch(&mut sim, true, 4);
    assert_eq!(
        b.ray_enters(),
        &[(r, longe)],
        "religar: entra só a de longe"
    );
    assert!(b.ray_exits().is_empty(), "religar: ninguém sai");
}

/// ⭐⭐ **C5 — «Turns with facing» sob a ESPADA vira com o HERÓI**: o raio é filho de uma espada com corpo
/// próprio, filha do herói; o lado é o do personagem da cadeia. Controlo: desligada, olha para a direita.
///
/// **Mutação que sangra:** ler o `player_state` do corpo dono (a espada) em vez de subir a cadeia.
#[test]
fn o_raio_debaixo_da_espada_vira_com_o_heroi() {
    for ligada in [true, false] {
        let (mut sim, mut bridge, p) = crate::platform_scene::scene(0.0, 0.0);
        let pose = |sim: &SimWorld| sim.world().get::<Transform>(p).expect("pose").translation;
        let (px, py) = (pose(&sim).x, pose(&sim).y);
        let esq = parede(&mut sim, "Esquerda", px - 3.0);
        let dir = parede(&mut sim, "Direita", px + 3.0);
        for w in [esq, dir] {
            if let Some(mut t) = sim.world_mut().get_mut::<Transform>(w) {
                t.translation.y = py;
            }
        }
        // A espada: um gatilho fino com corpo próprio (não empurra o herói), filho dele.
        let espada = caixa(&mut sim, "Espada", 0.0, 0.1, true);
        if let Some(mut t) = sim.world_mut().get_mut::<Transform>(espada) {
            t.translation = Vec2::ZERO;
        }
        sim.world_mut().entity_mut(espada).insert(ChildOf(p));
        let r = raio(
            &mut sim,
            Vec2::ZERO,
            Some(espada),
            RaySensor {
                turns_with_facing: ligada,
                ..para(0.0)
            },
        );
        for t in 1..=6u64 {
            bridge.set_player_input(
                p,
                ph2d_platformer::PlayerInput {
                    drive: -1.0,
                    ..ph2d_platformer::PlayerInput::default()
                },
            );
            bridge.dispatch(&mut sim, true, t);
        }
        assert_eq!(
            bridge.player_view(p).map(|v| v.facing),
            Some(-1.0),
            "fixtura"
        );
        let visto = bridge.ray_sensor_hits().get(&r).map(|h| h.body);
        assert_eq!(
            visto,
            Some(if ligada { esq } else { dir }),
            "ligada = {ligada}"
        );
    }
}

/// O herói (personagem) num veículo: `Veiculo` ⊃ {`Cabine`, `Heroi` ⊃ {`Espada`, o raio}}; `Muro` ao fundo.
fn veiculo(up_to: impl Fn(&SimWorld, [Entity; 4]) -> u64) -> (Option<Entity>, [Entity; 4]) {
    let mut sim = SimWorld::new();
    let v = caixa(&mut sim, "Veiculo", 0.0, 0.25, false);
    let cabine = caixa(&mut sim, "Cabine", 3.0, 0.25, false);
    let heroi = caixa(&mut sim, "Heroi", 0.0, 0.5, false);
    let espada = caixa(&mut sim, "Espada", 1.5, 0.25, false);
    let muro = parede(&mut sim, "Muro", 6.0);
    for (e, pai) in [(cabine, v), (heroi, v), (espada, heroi)] {
        sim.world_mut().entity_mut(e).insert(ChildOf(pai));
    }
    for (e, id) in [(v, 11), (cabine, 12), (heroi, 13), (espada, 14), (muro, 15)] {
        sim.world_mut().entity_mut(e).insert(StableId(id));
    }
    sim.world_mut()
        .entity_mut(heroi)
        .insert(PlatformPlayer::default());
    let todos = [v, cabine, espada, muro];
    let ate = up_to(&sim, todos);
    let r = raio(
        &mut sim,
        Vec2::new(0.75, 0.0),
        Some(heroi),
        RaySensor {
            ignore_up_to: ate,
            ..para(0.0)
        },
    );
    let _ = sim.world_mut().get::<Name>(r);
    (ve(&mut sim, r, &TagTree::new()), todos)
}

/// ⭐⭐⭐ **C8 (dono 15/11) — até onde a família vai.** Automático: pára no PERSONAGEM — o raio do herói
/// ignora a espada e vê a cabine do veículo. «Ignore up to» = o veículo: ignora a árvore inteira dele e
/// vê o muro. Um topo fora da cadeia (o muro) não conta: volta ao automático.
///
/// **Mutações que sangram:** o automático subir ao corpo mais alto · ignorar `ignore_up_to` · aceitar
/// um topo que não é antepassado.
#[test]
fn a_familia_para_no_personagem_e_no_topo_nomeado() {
    let (visto, [_, cabine, _, _]) = veiculo(|_, _| 0);
    assert_eq!(
        visto,
        Some(cabine),
        "automático: pára no herói, vê a cabine"
    );
    let (visto, [_, _, _, muro]) = veiculo(|_, _| 11);
    assert_eq!(visto, Some(muro), "até ao veículo: a cabine é família");
    let (visto, [_, cabine, _, _]) = veiculo(|_, _| 13);
    assert_eq!(visto, Some(cabine), "até ao herói = o automático");
    let (visto, [_, cabine, _, _]) = veiculo(|_, _| 15);
    assert_eq!(visto, Some(cabine), "o muro não está acima: automático");
}
