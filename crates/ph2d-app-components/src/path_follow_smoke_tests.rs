//! Os gates da CENA do seguidor de caminho (suplente #23).
//!
//! ⚠️⚠️ **Eles correm pelo caminho do PRODUTO** — a mesma [`monta_em`] que a shell chama e a mesma
//! [`crate::path_follow_bridge::a_escrever`] que o quadro corre. *Uma cena medida por uma cópia
//! dela afirma sobre código que o dono não vê.*

use super::*;
use ph2d_ecs::{TimerRuntime, World};

/// A cena montada, e os três empréstimos que ela usa.
fn cena() -> (
    ph2d_ecs::SimWorld,
    ph2d_vec_scene::VecScene,
    ph2d_vec_entities::entities::VecEntityMap,
    Montada,
) {
    // ⚠️ **Os três à mão, e não o `entities::setup()`**: aquele vive atrás da feature
    // `test-support` da crate dele, e ligá-la daqui poria uma feature de teste na árvore do
    // produto por causa de três construtores vazios.
    let mut sim = ph2d_ecs::SimWorld::new();
    let mut c = ph2d_vec_scene::VecScene::new();
    let mut m = ph2d_vec_entities::entities::VecEntityMap::new();
    let montada = monta_em(&mut sim, &mut c, &mut m, 1).expect("a `=1` existe");
    (sim, c, m, montada)
}

/// Põe todos os relógios da cena a meio do período.
///
/// ⚠️⚠️ **A reconciliação vem PRIMEIRO, e é o que uma cena NÃO faz por si:** ela declara os
/// `Timers` (a config, que é o que o ficheiro guarda) e quem cria o `TimerRuntime` é o quadro. Sem
/// esta linha o gate mede um mundo onde relógio nenhum nasceu — e a lista de pedidos vem **VAZIA**,
/// que é exactamente o que a 1.ª corrida deste ficheiro leu.
fn a_meio(world: &mut World) {
    ph2d_ecs::timer::reconcile_timers(world);
    let mut q = world.query::<&mut TimerRuntime>();
    for mut rt in q.iter_mut(world) {
        for s in &mut rt.0 {
            s.elapsed_us = PERIODO_US / 2;
        }
    }
}

/// ⭐⭐⭐ **A PISTA tem o NOME que os seguidores procuram** — e o gate lê os DOIS lados.
///
/// ⚠️ Escrito duas vezes, um erro de letra daria uma cena em que nada anda, com a forma à vista e o
/// painel a dizer *«no object in the scene has that name»*.
#[test]
fn a_pista_tem_o_nome_que_os_seguidores_procuram() {
    let (mut sim, _c, _m, _) = cena();
    let world = sim.world_mut();
    let mut q = world.query::<(&Name, &ph2d_ecs::VecPathRef)>();
    let nomes: Vec<String> = q.iter(world).map(|(n, _)| n.0.clone()).collect();
    assert_eq!(nomes, vec![PISTA.to_owned()], "a forma desenhada da cena");

    let mut q = world.query::<&PathFollow>();
    let seguidores: Vec<&PathFollow> = q.iter(world).collect();
    assert_eq!(seguidores.len(), 3, "três seguidores na cena");
    for pf in seguidores {
        assert_eq!(pf.caminho, PISTA, "um seguidor aponta a outra pista");
    }
}

/// ⭐⭐⭐ **OS TRÊS ANDAM NA PISTA E O CONTROLO NÃO** — a medição do §5.0 posta na cena.
///
/// ⚠️⚠️ **A régua é a DISTÂNCIA À CURVA, e não um ponto previsto** — e a 1.ª redacção deste gate
/// media um ponto: ela esperava os três no TOPO a meio do período e o vai-e-volta estava no FIM.
/// *Ele estava certo e a minha régua é que descrevia o irmão dele* — a meio do período a dobra da
/// W8 devolve `1,0`, que é a ponta da pista. ⇒ o que se afirma é o que a cena existe para mostrar:
/// **eles estão SOBRE a pista** (à distância do `Side Offset` de cada um), e o cinzento não.
///
/// ⭐ E o desvio do CONTROLO é o RAIO inteiro, que é o número fechado da sonda: numa circunferência
/// centrada na origem todo ponto da corda está a exactamente `r` do arco.
#[test]
fn os_tres_andam_na_pista_e_o_controlo_corta_pelo_meio() {
    let (mut sim, c, _m, _) = cena();
    a_meio(sim.world_mut());

    let cozido = c.paths()[0].cooked();
    let arco = ph2d_vec_scene::arc_path::ArcPath::from_contour(&cozido.verts, cozido.closed)
        .expect("a pista tem segmentos");
    let distancia = |p: [f32; 2]| {
        let p = [f64::from(p[0]), f64::from(p[1])];
        let (q, _) = arco.frame_at(arco.closest_arc(p));
        (q[0] - p[0]).hypot(q[1] - p[1])
    };

    let pedidos = crate::path_follow_bridge::a_escrever(&mut sim, &c);
    assert_eq!(pedidos.len(), 3, "três seguidores pedem");
    let mut sitios = Vec::new();
    for p in &pedidos {
        let nome = sim
            .world()
            .get::<Name>(p.entity)
            .map_or(String::new(), |n| n.0.clone());
        let lado = f64::from(sim.world().get::<PathFollow>(p.entity).unwrap().lado);
        let d = distancia(p.mundo);
        assert!(
            (d - lado.abs()).abs() < 1e-3,
            "«{nome}» está a {d:.4} da pista e o Side Offset dele é {lado}"
        );
        sitios.push((nome, p.mundo));
    }
    // ⭐ **E o VAI-E-VOLTA está NOUTRO sítio da pista** — sem isto, três seguidores idênticos
    // passariam este gate e a cena não mostraria diferença nenhuma.
    let ping = sitios
        .iter()
        .find(|(n, _)| n.starts_with("Vai-e-volta"))
        .expect("o vai-e-volta está na cena");
    let ronda = sitios
        .iter()
        .find(|(n, _)| n == "Ronda")
        .expect("a ronda está na cena");
    let entre = (ping.1[0] - ronda.1[0]).hypot(ping.1[1] - ronda.1[1]);
    assert!(
        entre > 0.5,
        "o ciclo não faz diferença nenhuma a meio do período ({entre:.4})"
    );

    // ⛔ **O CONTROLO** — ele não pede nada à ponte (não tem o componente), e o tween põe-no na
    // CORDA. O desvio é o RAIO inteiro, que é o número que a sonda do §5.0 mediu.
    let mut drive = ph2d_preview_drive::PreviewDrive::default();
    crate::tween_bridge::drive_tweens(&mut sim, &mut drive, &pedidos);
    let world = sim.world_mut();
    let mut q = world.query::<(&Name, &Transform)>();
    let ctrl = q
        .iter(world)
        .find(|(n, _)| n.0.starts_with("Controlo"))
        .map(|(_, t)| *t)
        .expect("o controlo está na cena");
    let fora = distancia([ctrl.translation.x, ctrl.translation.y]);
    assert!(
        (fora - RAIO).abs() < 1e-3,
        "o controlo devia sair da pista pelo RAIO e saiu {fora:.6}"
    );
}

/// ⚠️ **TODOS no mesmo período** — com ritmos diferentes o olho compara velocidades em vez de
/// comparar percursos, e a cena deixa de ensinar o que promete.
#[test]
fn todos_correm_no_mesmo_periodo() {
    let (mut sim, _c, _m, _) = cena();
    let world = sim.world_mut();
    let mut q = world.query::<&Timers>();
    for ts in q.iter(world) {
        for t in &ts.0 {
            assert_eq!(t.duration_us, PERIODO_US);
            assert!(
                t.repeat && t.autostart,
                "o relógio da cena tem de correr só"
            );
            assert!(t.signal.is_empty(), "um nome aqui poria avisos na tela");
        }
    }
}

/// ⭐⭐⭐ **A CENA NÃO ABRE COM QUEIXA** — se abrisse, o passo (1) mandaria o dono olhar para uma
/// secção a dizer que falta alguma coisa.
///
/// ⚠️ **E o CONTROLO negativo está no mesmo gate:** apagar o nome faz a queixa aparecer. Sem ele,
/// um `queixa()` que devolvesse sempre `None` passaria.
#[test]
fn a_cena_abre_sem_queixa_e_a_queixa_existe() {
    let (mut sim, _c, _m, montada) = cena();
    let bits = montada.escolhido;
    let i = crate::path_follow_inspector::build_path_follow_info(sim.world_mut(), bits, 1, true)
        .expect("quem nasce escolhido TEM o componente");
    assert_eq!(i.queixa(), None, "a cena abre a queixar-se: {i:?}");
    assert_eq!(i.caminho, PISTA);

    sim.world_mut()
        .get_mut::<PathFollow>(Entity::from_bits(montada.escolhido))
        .unwrap()
        .caminho
        .clear();
    let i = crate::path_follow_inspector::build_path_follow_info(sim.world_mut(), bits, 1, true)
        .unwrap();
    assert!(
        i.queixa().is_some(),
        "sem nome, o painel tem de dizer porquê"
    );
}

/// ⚠️ **O CHÃO nasce PRIMEIRO** — a ordem das raízes é congelada pela ordem de criação, e um fundo
/// criado por último desenha por cima de tudo (o report do dono no #13).
#[test]
fn o_chao_nasce_primeiro() {
    let (mut sim, _c, _m, _) = cena();
    let world = sim.world_mut();
    let mut q = world.query::<(ph2d_ecs::Entity, &Name)>();
    let mut nomes: Vec<(ph2d_ecs::Entity, String)> =
        q.iter(world).map(|(e, n)| (e, n.0.clone())).collect();
    nomes.sort_by_key(|(e, _)| e.index());
    assert_eq!(nomes.first().map(|(_, n)| n.as_str()), Some("Ground"));
}
