//! ⭐ **A sonda headless das cenas** — os números que a mensagem ao dono promete, medidos antes dela.

use bevy_ecs::entity::Entity;
use ph2d_app_constraints::smoke::{self, FOLGA_DA_PUPILA, ROUTERS};
use ph2d_constraints_ecs::evaluate;
use ph2d_ecs::{Name, SimWorld, Transform, world_transform};
use ph2d_preview_drive::PreviewDrive;

fn cena(n: u32) -> (SimWorld, PreviewDrive) {
    let mut sim = SimWorld::new();
    assert!(
        smoke::encenar(sim.world_mut(), n),
        "a cena {n} não respondeu"
    );
    let mut drive = PreviewDrive::default();
    evaluate(&mut sim, &mut drive);
    drive.settle();
    (sim, drive)
}

fn chamada(sim: &mut SimWorld, nome: &str) -> Entity {
    let w = sim.world_mut();
    let mut q = w.query::<(Entity, &Name)>();
    q.iter(w)
        .find(|(_, n)| n.0 == nome)
        .map(|(e, _)| e)
        .unwrap_or_else(|| panic!("a cena não tem «{nome}»"))
}

fn mundo(sim: &SimWorld, e: Entity) -> Transform {
    world_transform(sim.world(), e).expect("pose mundial")
}

#[test]
fn every_level_up_to_the_router_ceiling_answers() {
    let teto = ROUTERS[0].max_level;
    for n in 1..=teto {
        let mut sim = SimWorld::new();
        assert!(
            smoke::encenar(sim.world_mut(), n),
            "a cena {n} está abaixo do tecto e é muda"
        );
    }
    assert!(!smoke::encenar(SimWorld::new().world_mut(), teto + 1));
}

/// Cena 2: arrastar a bola por um círculo largo à volta do olho — a pupila nunca sai da folga e a seta
/// aponta sempre para a bola.
#[test]
fn the_pupil_never_leaves_the_eye_and_the_arrow_aims_at_the_ball() {
    let (mut sim, mut drive) = cena(2);
    let olho = chamada(&mut sim, "Olho");
    let bola = chamada(&mut sim, "Bola");
    let pupila = chamada(&mut sim, "Pupila");
    let seta = chamada(&mut sim, "Seta");
    let mut maior = 0.0_f64;
    for k in 0..72 {
        let a = f64::from(k) * std::f64::consts::TAU / 72.0;
        let r = if k % 2 == 0 { 3.0 } else { 0.3 };
        #[expect(clippy::cast_possible_truncation, reason = "o Transform é f32")]
        {
            let mut t = sim.world_mut().get_mut::<Transform>(bola).unwrap();
            t.translation.x = (r * a.cos()) as f32;
            t.translation.y = (r * a.sin()) as f32;
        }
        evaluate(&mut sim, &mut drive);
        drive.settle();
        let (o, p, b, s) = (
            mundo(&sim, olho),
            mundo(&sim, pupila),
            mundo(&sim, bola),
            mundo(&sim, seta),
        );
        let d = f64::from(p.translation.x - o.translation.x)
            .hypot(f64::from(p.translation.y - o.translation.y));
        maior = maior.max(d);
        if r < FOLGA_DA_PUPILA {
            assert!(
                (p.translation - b.translation).length() < 1e-5,
                "dentro da folga a pupila É a bola"
            );
        }
        let alvo = f64::from(b.translation.y - s.translation.y)
            .atan2(f64::from(b.translation.x - s.translation.x));
        let erro = ph2d_constraints::wrap_pi(f64::from(s.rotation) - alvo).abs();
        assert!(
            erro < 1e-5,
            "a seta erra a bola por {erro} rad no passo {k}"
        );
    }
    assert!(
        maior <= FOLGA_DA_PUPILA + 1e-5,
        "a pupila saiu do olho: {maior}"
    );
    assert!(
        maior >= FOLGA_DA_PUPILA - 1e-5,
        "a pupila nunca chegou à borda: {maior}"
    );
}

/// Cena 3: os dois ponteiros autorados a 170°.
#[test]
fn the_limited_pointer_stops_at_45_and_the_half_copy_lands_at_180() {
    let (mut sim, _) = cena(3);
    let limitado = chamada(&mut sim, "Ponteiro com limite");
    let meio = chamada(&mut sim, "Ponteiro a meio");
    let graus = |e| f64::from(mundo(&sim, e).rotation).to_degrees();
    assert!(
        (graus(limitado) - 45.0).abs() < 1e-4,
        "limite: {}",
        graus(limitado)
    );
    assert!(
        (graus(meio).abs() - 180.0).abs() < 1e-4,
        "meia cópia: {}",
        graus(meio)
    );
}

/// Cena 2, report do dono 25/10 (*«Inside e Surface estão na mesma posição»*): com a bola FORA do
/// olho os dois põem a pupila na borda (é a mesma resposta); com a bola DENTRO da folga, o Inside
/// deixa-a na bola e o Surface empurra-a até à borda — e o Outside também.
#[test]
fn inside_and_surface_differ_only_with_the_ball_inside_the_eye() {
    use ph2d_app_constraints::inspector::apply_all;
    use ph2d_constraints_ecs::Constraints;
    use ph2d_ecs::canal_constraint::MODO;
    use ph2d_editor_core::constraint_edits::ConstraintFieldEdit as E;
    let pupila_com = |modo: f32, bola_x: f32| {
        let (mut sim, mut drive) = cena(2);
        let bola = chamada(&mut sim, "Bola");
        let pupila = chamada(&mut sim, "Pupila");
        let id = sim.world().get::<Constraints>(pupila).unwrap().items[1].id;
        apply_all(&mut sim, &[(pupila.to_bits(), E::Campo(id, MODO, modo))]);
        let mut t = sim.world_mut().get_mut::<Transform>(bola).unwrap();
        t.translation.x = bola_x;
        t.translation.y = 0.0;
        evaluate(&mut sim, &mut drive);
        drive.settle();
        f64::from(mundo(&sim, pupila).translation.x)
    };
    let fora = 3.0;
    assert!((pupila_com(0.0, fora) - FOLGA_DA_PUPILA).abs() < 1e-5);
    assert!(
        (pupila_com(2.0, fora) - FOLGA_DA_PUPILA).abs() < 1e-5,
        "fora, iguais"
    );
    let dentro = 0.2;
    assert!(
        (pupila_com(0.0, dentro) - f64::from(dentro)).abs() < 1e-5,
        "Inside move-a"
    );
    assert!(
        (pupila_com(2.0, dentro) - FOLGA_DA_PUPILA).abs() < 1e-5,
        "Surface à borda"
    );
    assert!(
        (pupila_com(1.0, dentro) - FOLGA_DA_PUPILA).abs() < 1e-5,
        "Outside à borda"
    );
}

/// Cena 2, report do dono 25/10 (*«Outside: a pupila fica sempre em cima da bola»*): o Outside é
/// «nunca mais PERTO que a distância» — com a bola fora do círculo a pupila segue-a (nada a
/// corrigir); com a bola DENTRO, a pupila vai à borda, pelo raio, em qualquer raio e direcção; a
/// meia força fica a meio caminho entre a bola e a borda.
#[test]
fn outside_pushes_the_pupil_to_the_edge_whenever_the_ball_is_inside() {
    use ph2d_app_constraints::inspector::apply_all;
    use ph2d_constraints_ecs::Constraints;
    use ph2d_ecs::canal_constraint::MODO;
    use ph2d_editor_core::constraint_edits::ConstraintFieldEdit as E;
    let (mut sim, mut drive) = cena(2);
    let bola = chamada(&mut sim, "Bola");
    let pupila = chamada(&mut sim, "Pupila");
    let ids: Vec<u32> = sim
        .world()
        .get::<Constraints>(pupila)
        .unwrap()
        .items
        .iter()
        .map(|i| i.id)
        .collect();
    apply_all(&mut sim, &[(pupila.to_bits(), E::Campo(ids[1], MODO, 1.0))]);
    let em = |sim: &mut SimWorld, drive: &mut PreviewDrive, x: f32, y: f32| {
        let mut t = sim.world_mut().get_mut::<Transform>(bola).unwrap();
        t.translation.x = x;
        t.translation.y = y;
        for _ in 0..3 {
            ph2d_app_constraints::quadro(sim, &mut ph2d_vec_scene::VecScene::new(), drive, None);
            drive.settle();
        }
        let p = mundo(sim, pupila).translation;
        (f64::from(p.x), f64::from(p.y))
    };
    for (x, y) in [(0.05, 0.0), (0.3, 0.2), (-0.1, -0.4), (0.0, 0.59)] {
        let (px, py) = em(&mut sim, &mut drive, x, y);
        assert!(
            (px.hypot(py) - FOLGA_DA_PUPILA).abs() < 1e-4,
            "bola em ({x}, {y}) ⇒ pupila a {}",
            px.hypot(py)
        );
    }
    let (px, py) = em(&mut sim, &mut drive, 2.0, 1.0);
    assert!(
        (px - 2.0).abs() < 1e-5 && (py - 1.0).abs() < 1e-5,
        "fora, segue a bola"
    );
    apply_all(&mut sim, &[(pupila.to_bits(), E::Forca(ids[1], 0.5))]);
    let (px, _) = em(&mut sim, &mut drive, 0.2, 0.0);
    assert!(
        (px - (0.2 + FOLGA_DA_PUPILA) / 2.0).abs() < 1e-4,
        "meia força ⇒ {px}"
    );
}

/// Cena 4: o braço com a IK na mão e um Copy Rotation ABAIXO — a mão roda com o Rodador (no Blender o
/// Copy Rotation some); descer a IK na lista põe a ponta da mão na bola.
#[test]
fn the_arm_scene_turns_the_hand_below_the_ik() {
    use ph2d_app_constraints::inspector::apply_all;
    use ph2d_app_constraints::smoke::OSSO;
    use ph2d_constraints_ecs::Constraints;
    use ph2d_editor_core::constraint_edits::ConstraintFieldEdit as E;
    let (mut sim, mut drive) = cena(4);
    let ombro = chamada(&mut sim, "Ombro");
    let mao = chamada(&mut sim, "Antebraço");
    let bola = chamada(&mut sim, "Bola");
    let rodador = chamada(&mut sim, "Rodador");
    let passo = |sim: &mut SimWorld, drive: &mut PreviewDrive| {
        for _ in 0..3 {
            ph2d_app_constraints::quadro(sim, &mut ph2d_vec_scene::VecScene::new(), drive, None);
            drive.settle();
        }
    };
    let longe = |a: Transform, b: Transform| {
        f64::from((a.translation.x - b.translation.x).hypot(a.translation.y - b.translation.y))
    };
    let confere = |sim: &SimWorld| {
        let (m, r) = (mundo(sim, mao), mundo(sim, rodador));
        assert!(
            ph2d_constraints::wrap_pi(f64::from(m.rotation - r.rotation)).abs() < 1e-5,
            "a mão roda com o Rodador: {} ≠ {}",
            m.rotation.to_degrees(),
            r.rotation.to_degrees()
        );
        assert!((longe(m, mundo(sim, ombro)) - f64::from(OSSO)).abs() < 1e-4);
        assert!(
            (longe(m, mundo(sim, bola)) - f64::from(OSSO)).abs() < 1e-4,
            "o cotovelo é o da IK"
        );
    };
    passo(&mut sim, &mut drive);
    confere(&sim);
    sim.world_mut()
        .get_mut::<Transform>(bola)
        .unwrap()
        .translation
        .x = -0.5;
    sim.world_mut()
        .get_mut::<Transform>(rodador)
        .unwrap()
        .rotation = 30f32.to_radians();
    passo(&mut sim, &mut drive);
    confere(&sim);
    let antes = (mundo(&sim, ombro), mundo(&sim, mao));
    passo(&mut sim, &mut drive);
    assert_eq!(
        (mundo(&sim, ombro), mundo(&sim, mao)),
        antes,
        "parado, não salta"
    );
    // Descer a IK: agora ela vence, e a ponta da mão toca a bola.
    let ik = sim.world().get::<Constraints>(mao).unwrap().items[0].id;
    apply_all(&mut sim, &[(mao.to_bits(), E::Mover(ik, 1))]);
    passo(&mut sim, &mut drive);
    let m = mundo(&sim, mao);
    let (s, c) = m.rotation.sin_cos();
    let b = mundo(&sim, bola).translation;
    let ponta = (
        m.translation.x + c * OSSO * m.scale.x,
        m.translation.y + s * OSSO * m.scale.x,
    );
    assert!(
        (ponta.0 - b.x).abs() < 1e-4 && (ponta.1 - b.y).abs() < 1e-4,
        "{ponta:?} ≠ {b:?}"
    );
}

/// ⭐ Cena 9: com a BÓIA em baixo o ponteiro aponta 225° e a lâmpada fica a 0,15; em cima, −45° e
/// 1 (os números que a mensagem ao dono promete). Sem relógio: sem mola, o valor final.
#[test]
fn the_dial_scene_points_the_needle_by_the_float_and_lights_the_lamp() {
    use ph2d_app_constraints::smoke_mostrador::{BOIA, PONTEIRO_GRAUS};
    use ph2d_ecs::canal::familia::OBJETO;
    use ph2d_ecs::canal_objeto::OPACIDADE;
    use ph2d_ecs::{CanaisEscritos, CanaisLidos, FieldKey};
    let (mut sim, mut drive) = cena(9);
    let (boia, ponteiro, lampada) = (
        chamada(&mut sim, "Bóia"),
        chamada(&mut sim, "Ponteiro"),
        chamada(&mut sim, "Lâmpada"),
    );
    let k = FieldKey::new(OBJETO, OPACIDADE, 0, 0);
    sim.world_mut()
        .get_resource_or_insert_with(CanaisLidos::default)
        .publicar(lampada.to_bits(), k, 1.0);
    for (y, graus, luz) in [
        (BOIA[0].1, PONTEIRO_GRAUS[0], 0.15),
        (BOIA[1].1, PONTEIRO_GRAUS[1], 1.0),
    ] {
        sim.world_mut()
            .get_mut::<Transform>(boia)
            .unwrap()
            .translation
            .y = y;
        ph2d_app_constraints::valor::valores(&mut sim, None);
        let escrito = sim
            .world()
            .resource::<CanaisEscritos>()
            .valor(lampada.to_bits(), k);
        ph2d_app_constraints::quadro(
            &mut sim,
            &mut ph2d_vec_scene::VecScene::new(),
            &mut drive,
            None,
        );
        let rot = f64::from(mundo(&sim, ponteiro).rotation);
        assert!(
            (rot - graus.to_radians()).abs() < 1e-5,
            "{rot} contra {graus}°"
        );
        assert!(
            escrito.is_some_and(|v| (v - luz).abs() < 1e-6),
            "{escrito:?}"
        );
        let _ = sim.world_mut().remove_resource::<CanaisEscritos>();
    }
}

/// ⭐ Cena 10 (os *behaviours*): longe do íman a moeda fica quieta e por cima dela treme; a lua roda o
/// planeta à mesma distância e vira; o balão sai do sítio; a limalha anda para o íman.
#[test]
fn the_playground_scene_moves_each_piece_by_its_behaviour() {
    use ph2d_app_constraints::relogio::Relogio;
    use ph2d_app_constraints::smoke_recreio::{IMA, IMA_Y, MOEDA};
    let (mut sim, mut drive) = cena(10);
    let ima = chamada(&mut sim, "Íman");
    let (moeda, balao) = (chamada(&mut sim, "Moeda"), chamada(&mut sim, "Balão"));
    let (lua, planeta) = (chamada(&mut sim, "Lua"), chamada(&mut sim, "Planeta"));
    let limalha = chamada(&mut sim, "Limalha");
    let doc = ph2d_timeline::TimelineDoc::new();
    let mut passo = |sim: &mut SimWorld, x: f32, t: f64| {
        sim.world_mut()
            .get_mut::<Transform>(ima)
            .unwrap()
            .translation
            .x = x;
        let r = Relogio {
            doc: &doc,
            t,
            laco: None,
        };
        ph2d_app_constraints::quadro(
            sim,
            &mut ph2d_vec_scene::VecScene::new(),
            &mut drive,
            Some(r),
        );
    };
    passo(&mut sim, IMA[0].1, 0.3);
    let m = mundo(&sim, moeda).translation;
    assert_eq!([m.x, m.y], MOEDA, "longe do íman a moeda não treme");
    passo(&mut sim, MOEDA[0], 0.3);
    let m = mundo(&sim, moeda).translation;
    assert_ne!([m.x, m.y], MOEDA, "o íman por baixo: treme");
    const { assert!(IMA_Y < MOEDA[1]) };
    let (l, p) = (mundo(&sim, lua), mundo(&sim, planeta));
    let r = (l.translation.x - p.translation.x).hypot(l.translation.y - p.translation.y);
    // O seno do `motion.orbit` é a parábola corrigida (~0,09 %): o raio oscila dentro disso.
    assert!((r - 1.5).abs() < 1.5 * 1e-3, "a lua fica à distância: {r}");
    assert!(l.rotation.abs() > 0.1, "e vira com a volta");
    assert_ne!(mundo(&sim, balao).translation.y, 1.5, "o balão flutua");
    let antes = mundo(&sim, limalha).translation.x;
    for f in 1..30 {
        passo(&mut sim, 3.0, f64::from(f) / doc.fps_display);
    }
    assert!(
        mundo(&sim, limalha).translation.x > antes + 0.5,
        "a limalha vai para o íman"
    );
}

/// ⭐ Cena 11 (W6): o relógio que conduz o placar dispara e NÃO pede o aviso «Signal: tique» — o
/// toast a cada 2 s era ruído de um relógio de jogo. O sinal publica-se na mesma (o placar ouve-o).
#[test]
fn the_hud_scene_timer_fires_without_a_notice() {
    use ph2d_app_constraints::smoke_painel::{TIQUE_US, vestir};
    let mut sim = SimWorld::new();
    assert!(smoke::encenar(sim.world_mut(), 11));
    let mut cena = ph2d_vec_scene::VecScene::new();
    ph2d_app_constraints::quadro(&mut sim, &mut cena, &mut PreviewDrive::default(), None);
    let mut map = ph2d_vec_entities::entities::VecEntityMap::new();
    ph2d_vec_entities::entities::sync(&mut sim, &mut cena, &mut map);
    vestir(&mut sim).expect("veste");
    ph2d_ecs::reconcile_timers(sim.world_mut());
    let disparos = ph2d_ecs::timer::fire_all(sim.world_mut(), TIQUE_US);
    assert_eq!(disparos.len(), 1, "o relógio da cena não disparou");
    assert_eq!(disparos[0].name, ph2d_app_constraints::smoke_painel::SINAL);
    assert!(
        !disparos[0].notice,
        "o relógio da cena 11 pediu o aviso a cada tique"
    );
}

/// ⭐ Cena 11 (o HUD que se arruma): montada de verdade (semear → as entidades nascem → vestir), a fiação está lá
/// e os números da cena contam a história — na janela LARGA a pontuação longa cabe; em `16:9` ela encolhe e a moeda
/// fica colada sem invadir o minimapa; com o «Score: 0» nada se mexe.
#[test]
fn the_hud_scene_keeps_the_coin_beside_and_shrinks_the_score_before_the_minimap() {
    use ph2d_app_constraints::smoke_painel::{
        FOLGA, LINHA_Y, MAPA, MARGEM, MOEDA_R, PONT_X, Painel, REF, relacoes, vestir,
    };
    use ph2d_ecs::hud_arranjo::UiArranjo;
    use ph2d_ecs::{GameCamera, StableId, UiLabel, VecAnchors, VecShape};
    use ph2d_hud_layout::{Caixa, Elemento, resolve};
    let mut sim = SimWorld::new();
    assert!(smoke::encenar(sim.world_mut(), 11));
    let mut cena = ph2d_vec_scene::VecScene::new();
    ph2d_app_constraints::quadro(&mut sim, &mut cena, &mut PreviewDrive::default(), None);
    let largura = sim
        .world()
        .get_resource::<Painel>()
        .expect("o documento semeado")
        .largura;
    assert!(vestir(&mut sim).is_none(), "sem entidades ainda não veste");
    let mut map = ph2d_vec_entities::entities::VecEntityMap::new();
    ph2d_vec_entities::entities::sync(&mut sim, &mut cena, &mut map);
    let canvas = vestir(&mut sim).expect("veste quando as formas têm entidade");
    let (pont, moeda, mapa) = (
        chamada(&mut sim, "Pontuação"),
        chamada(&mut sim, "Moeda"),
        chamada(&mut sim, "Minimapa"),
    );
    let w = sim.world();
    let id = |e| *w.get::<StableId>(e).unwrap();
    assert_eq!(
        w.get::<UiArranjo>(canvas).unwrap().lacos,
        relacoes(id(pont), id(moeda), id(mapa))
    );
    assert!(
        matches!(w.get::<VecShape>(pont), Some(VecShape::Text(_)))
            && w.get::<UiLabel>(pont).is_some()
    );
    assert_eq!(
        w.get::<VecAnchors>(mapa).unwrap().min,
        [1.0, 1.0],
        "o minimapa preso ao canto de cima à direita"
    );
    assert_eq!(w.get::<VecAnchors>(pont).unwrap().min, [0.0, 1.0]);
    let mut q = sim.world_mut().query::<&GameCamera>();
    assert_eq!(
        q.iter(sim.world()).count(),
        1,
        "a câmera de jogo conduz o HUD"
    );
    // As caixas LOCAIS do canvas, como as âncoras as deixam numa janela de largura `tela` (Expand: o minimapa anda
    // com a borda direita); a pontuação com o texto de largura `texto`.
    let (r, h, folga) = (MOEDA_R, 2.0, FOLGA);
    let caixas = |tela: f64, texto: f64| {
        let dx = (tela - f64::from(REF[0])) / 2.0;
        let (mw, mh) = (MAPA[0], MAPA[1]);
        let mx = f64::from(REF[0]) / 2.0 - f64::from(MARGEM) - mw + dx;
        let my = f64::from(REF[1]) / 2.0 - f64::from(MARGEM) - mh;
        let px = f64::from(PONT_X) - dx;
        let el = |x, y, w, h, c| Elemento {
            caixa: Caixa { x, y, w, h },
            conteudo: c,
        };
        vec![
            el(px, f64::from(LINHA_Y) - h / 2.0, texto, h, Some(texto)),
            el(
                px + largura + folga,
                f64::from(LINHA_Y) - r,
                2.0 * r,
                2.0 * r,
                None,
            ),
            el(mx, my, mw, mh, None),
        ]
    };
    let rel: Vec<_> = relacoes(StableId(1), StableId(2), StableId(3))
        .into_iter()
        .filter_map(|(r, f)| Some((r.mapeia(|s: StableId| usize::try_from(s.0 - 1).ok())?, f)))
        .collect();
    let corre = |tela: f64, texto: f64| {
        let t = Caixa {
            x: -tela / 2.0,
            y: -f64::from(REF[1]) / 2.0,
            w: tela,
            h: f64::from(REF[1]),
        };
        (caixas(tela, texto), resolve(t, &caixas(tela, texto), &rel))
    };
    // «Score: 0»: a moeda já está colada; nada se mexe.
    let (antes, depois) = corre(f64::from(REF[0]), largura);
    // Dentro da PARADA o passe não publica nada (`hud_arranjo::parada`).
    let d = ph2d_hud_layout::PARADA;
    for (a, b) in antes.iter().zip(&depois) {
        let a = a.caixa;
        assert!(
            [(a.x, b.x), (a.y, b.y), (a.w, b.w), (a.h, b.h)]
                .iter()
                .all(|(p, q)| (p - q).abs() <= d),
            "juntar não mexe no desenho: {a:?} → {b:?}"
        );
    }
    // Uma pontuação que cabe na janela larga e não em 16:9.
    let longa = f64::from(REF[0]) - 2.0 * f64::from(MARGEM) - MAPA[0] - 2.0 * folga - 2.0 * r + 3.0;
    let (_, larga) = corre(f64::from(REF[0]) + 10.0, longa);
    assert!(
        (larga[0].w - longa).abs() <= d,
        "na janela larga a pontuação cabe inteira"
    );
    let (els, estreita) = corre(f64::from(REF[0]), longa);
    assert!(
        estreita[0].w < longa - 1.0,
        "em 16:9 a pontuação encolhe: {estreita:?}"
    );
    assert!(
        (estreita[1].x - (estreita[0].x + estreita[0].w + folga)).abs() <= d,
        "a moeda colada"
    );
    assert!(
        estreita[1].x + estreita[1].w + folga <= estreita[2].x + d,
        "a moeda não invade o minimapa"
    );
    assert_eq!(estreita[2], els[2].caixa, "o minimapa não cede");
    assert!(
        (estreita[1].w - 2.0 * r).abs() <= d && (estreita[1].h - 2.0 * r).abs() <= d,
        "a moeda não muda de tamanho (só o texto encolhe): {estreita:?}"
    );
    // A janela estreita (Expand) sobe o que está preso ao topo: a moeda acompanha a pontuação na altura.
    let mut alta = caixas(f64::from(REF[0]), largura);
    alta[0].caixa.y += 2.0;
    let t = Caixa {
        x: -16.0,
        y: -11.0,
        w: 32.0,
        h: 22.0,
    };
    let r2 = resolve(t, &alta, &rel);
    let meio = |c: &Caixa| c.y + c.h / 2.0;
    assert!(
        (meio(&r2[1]) - meio(&r2[0])).abs() <= d,
        "a moeda ficou para trás na altura: {r2:?}"
    );
}

/// ⭐ Cena 12 (W6): a Bola tem o Copy Particle na partícula da Fonte; com a partícula viva ela fica onde a
/// partícula está NO MUNDO (o afim da Fonte), e sem ela volta à pose dela. (O nascer e morrer da partícula
/// pelo cozimento real é o gate irmão no Motion, `the_fountain_particle_the_ball_rides_…`.)
#[test]
fn the_particle_scene_pins_the_ball_to_the_fountain_particle() {
    use ph2d_app_constraints::smoke_particula::{BOLA_EM, FONTE, FONTE_EM, PARTICULA};
    use ph2d_constraints::{Kind, PontoDeSimulacao};
    use ph2d_constraints_ecs::Constraints;
    use ph2d_ecs::StableId;
    use ph2d_ecs::motion_generator::{Particulas, ParticulasPedidas};
    let (mut sim, mut drive) = cena(12);
    let fonte = chamada(&mut sim, FONTE);
    let bola = chamada(&mut sim, "Bola");
    assert!(
        sim.world()
            .get::<ph2d_ecs::MotionGenerator>(fonte)
            .is_some()
    );
    let s = *sim.world().get::<StableId>(fonte).expect("id da Fonte");
    let item = sim.world().get::<Constraints>(bola).expect("pilha").items[0];
    assert_eq!(
        item.kind,
        Kind::PontoDeSimulacao(PontoDeSimulacao { id: PARTICULA })
    );
    assert_eq!(item.target, Some(s));
    let mut vcena = ph2d_vec_scene::VecScene::new();
    ph2d_app_constraints::quadro(&mut sim, &mut vcena, &mut drive, None);
    assert_eq!(sim.world().resource::<ParticulasPedidas>().0, [s].into());
    let pose = |sim: &SimWorld| {
        let t = mundo(sim, bola);
        [t.translation.x, t.translation.y]
    };
    assert_eq!(pose(&sim), BOLA_EM, "sem partículas: a pose da Bola");
    for (k, q) in [[0.0, 0.5], [0.4, 1.6], [-0.7, 2.9]]
        .into_iter()
        .enumerate()
    {
        let outra = PARTICULA + 1 + u32::try_from(k).unwrap();
        sim.world_mut().insert_resource(Particulas(
            [(s, [(PARTICULA, q), (outra, [9.0, 9.0])].into())].into(),
        ));
        ph2d_app_constraints::quadro(&mut sim, &mut vcena, &mut drive, None);
        drive.settle();
        #[expect(clippy::cast_possible_truncation, reason = "o Transform é f32")]
        let certo = [FONTE_EM[0] + q[0] as f32, FONTE_EM[1] + q[1] as f32];
        let p = pose(&sim);
        assert!(
            (p[0] - certo[0]).abs() < 1e-5 && (p[1] - certo[1]).abs() < 1e-5,
            "quadro {k}: a Bola em {p:?}, a partícula em {certo:?}"
        );
    }
    sim.world_mut().insert_resource(Particulas(
        [(s, [(PARTICULA + 9, [1.0, 1.0])].into())].into(),
    ));
    ph2d_app_constraints::quadro(&mut sim, &mut vcena, &mut drive, None);
    drive.settle();
    assert_eq!(
        pose(&sim),
        BOLA_EM,
        "a partícula morreu: a Bola volta à pose dela"
    );
}

/// ⭐ Cena 12 (W6): a Ponta tem a mola, e com a timeline PARADA arrastar o Corpo faz a ponta ficar para trás
/// e assentar (a mola ao vivo).
#[test]
fn the_particle_scene_tip_springs_when_the_body_is_dragged_with_the_time_stopped() {
    use ph2d_app_constraints::relogio::{Parede, Relogio};
    use ph2d_constraints::Kind;
    use ph2d_constraints_ecs::Constraints;
    let (mut sim, mut drive) = cena(12);
    let corpo = chamada(&mut sim, "Corpo");
    let ponta = chamada(&mut sim, "Ponta");
    let pilha = sim
        .world()
        .get::<Constraints>(ponta)
        .expect("pilha da ponta");
    assert!(
        pilha
            .items
            .iter()
            .any(|it| matches!(it.kind, Kind::Spring(_))),
        "a ponta não tem mola"
    );
    let doc = ph2d_timeline::TimelineDoc::default();
    let mut vcena = ph2d_vec_scene::VecScene::new();
    let mut parede = 0.0;
    let mut passo = |sim: &mut SimWorld, drive: &mut PreviewDrive, parede: &mut f64| {
        sim.world_mut().insert_resource(Parede(*parede));
        let r = Relogio {
            doc: &doc,
            t: 0.0,
            laco: None,
        };
        ph2d_app_constraints::quadro(sim, &mut vcena, drive, Some(r));
        drive.settle();
        *parede += 1.0 / 60.0;
        mundo(sim, ponta).translation.x
    };
    let x0 = (0..3)
        .map(|_| passo(&mut sim, &mut drive, &mut parede))
        .last()
        .unwrap();
    sim.world_mut()
        .get_mut::<Transform>(corpo)
        .unwrap()
        .translation
        .x += 1.5;
    let xs: Vec<f32> = (0..600)
        .map(|_| passo(&mut sim, &mut drive, &mut parede))
        .collect();
    assert!(
        xs[0] - x0 < 1.4,
        "a ponta não ficou para trás: {x0} → {}",
        xs[0]
    );
    let fim = *xs.last().unwrap();
    assert!(
        (fim - (x0 + 1.5)).abs() < 1e-3,
        "a ponta não assentou: {x0} → {fim}"
    );
}
