use super::*;

#[test]
fn o_nivel_le_se_pela_porta_e_fora_da_faixa_e_inerte() {
    let com = |v: &'static str| move |k: &str| (k == "PH2D_GAME_SMOKE").then(|| v.to_owned());
    assert_eq!(armed_level_in(com("1")), Some(1));
    assert_eq!(armed_level_in(com("2")), Some(2));
    assert_eq!(armed_level_in(com("3")), Some(3));
    assert_eq!(armed_level_in(com("4")), Some(4));
    assert_eq!(armed_level_in(com("5")), None);
    assert_eq!(armed_level_in(com("x")), None);
    assert_eq!(armed_level_in(|_| None), None);
}

#[test]
fn o_condutor_joga_corre_e_para_por_esta_ordem() {
    let com = |v: &'static str| move |k: &str| (k == "PH2D_GAME_SMOKE_DRIVE").then(|| v.to_owned());
    assert_eq!(conducao_in(com("window")), Some(GameStage::Window));
    assert_eq!(conducao_in(com("fullscreen")), Some(GameStage::Fullscreen));
    assert_eq!(conducao_in(com("editor")), Some(GameStage::Editor));
    assert_eq!(conducao_in(com("x")), None);
    let s = GameStage::Fullscreen;
    let g: Vec<_> = (0..1000).filter_map(|q| gesto_no_quadro(1, s, q)).collect();
    assert_eq!(
        g,
        [
            Gesto::Joga(s),
            Gesto::Direita(true),
            Gesto::Direita(false),
            Gesto::Para
        ]
    );
}

/// A cena tem o que o smoke manda olhar: o jogador, a câmara que o segue e as moedas com script.
#[test]
fn o_nivel_tem_jogador_camara_e_moedas_com_script() {
    let mut sim = SimWorld::new();
    let jogador = montar(&mut sim, &crate::registo_de_teste(), 1);
    let world = sim.world_mut();
    assert_eq!(world.get::<Name>(jogador).map(Name::as_str), Some("Player"));
    let mut q = world.query::<(&Name, Option<&LuauScript>, Option<&CameraFollow>)>();
    let (mut moedas, mut seguem) = (0, 0);
    for (n, s, f) in q.iter(world) {
        if n.as_str().starts_with("Coin") {
            assert!(s.is_some(), "{} sem script", n.as_str());
            moedas += 1;
        }
        if f.is_some_and(|f| f.target == "Player") {
            seguem += 1;
        }
    }
    assert_eq!((moedas, seguem), (MOEDAS.len(), 1));
    assert!(
        world
            .resource::<ProjectScripts>()
            .0
            .by_name(MOEDA_NAME)
            .is_some()
    );
}

/// ⭐ A moeda SOME quando o jogador lhe chega: a física paira-o e corre-o para a direita (o horário do
/// condutor), os scripts correm a cada passo, e as duas primeiras moedas saem. A corrida é sobre uma
/// cópia (R0a), por isso apagar um objecto que o artista pôs é do jogo — o Stop devolve-o.
#[test]
fn a_moeda_some_quando_o_jogador_lhe_chega() {
    use ph2d_physics_ecs::{PhysicsBridge, PlayerInput};
    let mut sim = ph2d_ecs::SimWorld::new();
    let jogador = montar(&mut sim, &crate::registo_de_teste(), 1);
    let moeda = |sim: &mut ph2d_ecs::SimWorld, nome: &str| {
        let w = sim.world_mut();
        let mut q = w.query::<&Name>();
        q.iter(w).any(|n| n.as_str() == nome)
    };
    crate::marca_a_copia(sim.world_mut(), true);
    let mut host = ph2d_script::ScriptHost::new().expect("vm");
    let mut bridge = PhysicsBridge::new();
    let (mut caiu, mut sumiram) = (Vec::new(), Vec::new());
    for tick in 1..=240_u64 {
        let drive = if tick > 60 { 1.0 } else { 0.0 };
        bridge.set_player_input(
            jogador,
            PlayerInput {
                drive,
                ..PlayerInput::default()
            },
        );
        bridge.dispatch(&mut sim, true, tick);
        host.scene_sync(sim.world_mut());
        let r = host.scene_tick(&mut sim, 1.0 / 60.0);
        for e in r.despawned {
            sim.world_mut().entity_mut(e).despawn();
            sumiram.push(tick);
        }
        if tick == 60 {
            caiu.push(
                sim.world()
                    .get::<Transform>(jogador)
                    .expect("pose")
                    .translation
                    .y,
            );
        }
    }
    let consola: Vec<String> = host.drain_console().into_iter().map(|l| l.text).collect();
    // A altura a que paira (o topo do chão é y = 0): o alcance de 0,6 m tem de a cobrir.
    assert!((0.7..1.1).contains(&caiu[0]), "paira a {caiu:?}");
    assert!(
        !moeda(&mut sim, "Coin 1") && !moeda(&mut sim, "Coin 2"),
        "{consola:?}"
    );
    assert!(moeda(&mut sim, "Coin 3"), "a da plataforma fica");
    assert_eq!(sumiram.len(), 2, "{consola:?}");
}

/// Sem cópia (o transporte da timeline a tocar), a moeda do artista fica — e a consola diz porquê.
#[test]
fn sem_copia_a_moeda_do_artista_fica_e_a_consola_diz_porque() {
    let mut sim = ph2d_ecs::SimWorld::new();
    let jogador = montar(&mut sim, &crate::registo_de_teste(), 1);
    crate::marca_a_copia(sim.world_mut(), false);
    let mut host = ph2d_script::ScriptHost::new().expect("vm");
    host.scene_sync(sim.world_mut());
    sim.world_mut()
        .get_mut::<Transform>(jogador)
        .expect("pose")
        .translation = Vec2::new(-4.0, 0.9);
    let r = host.scene_tick(&mut sim, 1.0 / 60.0);
    assert!(r.despawned.is_empty());
    assert!(
        host.drain_console()
            .iter()
            .any(|l| l.text.contains("ph2d.despawn: 'Coin 1'"))
    );
}

/// ⭐ **O nível 2 nasce SÓ pelo menu Add**: cada peça tem o conjunto do objecto que a fez, e a parede
/// do fim é uma forma dentro do chão (sem corpo próprio).
#[test]
fn o_nivel_2_nasce_so_pelo_menu() {
    use ph2d_physics_ecs::{BodyKind, CharacterAnimations, Collider, PlatformPlayer, RigidBody};
    let mut sim = SimWorld::new();
    let jogador = montar(&mut sim, &crate::registo_de_teste(), 2);
    let w = sim.world_mut();
    let mut por_nome = std::collections::BTreeMap::new();
    let mut q = w.query::<(Entity, &Name)>();
    for (e, n) in q.iter(w) {
        por_nome.insert(n.as_str().to_owned(), e);
    }
    let w = sim.world();
    for (nome, ..) in BLOCOS_2.iter() {
        let e = por_nome[*nome];
        assert_eq!(
            w.get::<RigidBody>(e).map(|b| b.kind),
            Some(BodyKind::Static),
            "{nome}"
        );
        assert!(
            w.get::<Sprite>(e).is_some() && w.get::<Collider>(e).is_some(),
            "{nome}"
        );
    }
    let parede = por_nome["Invisible Wall"];
    assert!(w.get::<Collider>(parede).is_some() && w.get::<RigidBody>(parede).is_none());
    assert!(w.get::<Sprite>(parede).is_none(), "a parede é invisível");
    assert_eq!(
        w.get::<ChildOf>(parede).map(|c| c.parent()),
        Some(por_nome["Ground Right"])
    );
    assert_eq!(por_nome["Player"], jogador);
    assert!(
        w.get::<PlatformPlayer>(jogador).is_some()
            && w.get::<CharacterAnimations>(jogador).is_some()
    );
    // A arte de fábrica: cada bloco repete o azulejo do seu papel ao tamanho dele; as moedas giram e a
    // bandeira ondula; o céu fica atrás de tudo.
    for (nome, _, h) in BLOCOS_2.iter() {
        let e = por_nome[*nome];
        let key = w.get::<Sprite>(e).map(|s| s.source);
        let uv = w.get::<ph2d_ecs::UvTransform>(e).expect(nome).scale;
        let (art, alto) = match *nome {
            "Platform Low" | "Platform High" => (ph2d_render::PLATFORM_TILE, 0.5),
            "Wall Left" => (ph2d_render::DIRT_TILE, 1.0),
            _ => (ph2d_render::GROUND_TILE, 1.0),
        };
        assert_eq!(
            key,
            Some(ph2d_render::SpriteSource::Atlas { key: art.key }),
            "{nome}"
        );
        assert_eq!(uv, [2.0 * h[0], 2.0 * h[1] / alto], "{nome}");
        // O azulejo é UMA imagem; a cópia do meio é do atlas (plano GEF 13 D1).
        assert!(w.get::<ph2d_ecs::SpriteGrid>(e).is_none(), "{nome} tem quadros");
        assert_eq!(
            w.get::<ph2d_ecs::TextureRepeat>(e).map(|r| r.0),
            Some(ph2d_ecs::RepeatMode::Enabled),
            "{nome}"
        );
    }
    for (nome, art, tag) in [
        ("Goal", ph2d_render::FLAG_SHEET, "wave"),
        ("Coin 1", ph2d_render::COIN_SHEET, "spin"),
        ("Coin 5", ph2d_render::COIN_SHEET, "spin"),
    ] {
        let e = por_nome[nome];
        assert_eq!(
            w.get::<Sprite>(e).map(|s| s.source),
            Some(ph2d_render::SpriteSource::Atlas { key: art.key }),
            "{nome}"
        );
        let a = w.get::<ph2d_ecs::SpriteAnimator>(e).expect(nome);
        assert_eq!((a.current.as_str(), a.autoplay), (tag, true), "{nome}");
        assert_eq!(
            w.get::<ph2d_ecs::SpriteGrid>(e).map(|g| g.hframes),
            Some(art.grid[0])
        );
    }
    let moedas = (1..=MOEDAS_2.len())
        .filter(|i| {
            w.get::<LuauScript>(por_nome[&format!("Coin {i}")])
                .is_some()
        })
        .count();
    assert_eq!(moedas, MOEDAS_2.len(), "cada moeda corre o moeda.luau");
    assert_eq!(
        w.get::<ph2d_ecs::ZIndexOverride>(por_nome["Sky"])
            .map(|z| z.0),
        Some(-3)
    );
    // O fundo (plano 11): o céu atrás de tudo, depois as nuvens, depois os montes; cada um com a sua fracção.
    for (nome, k, ordem) in [("Sky", 0.0, -3), ("Clouds", 0.15, -2), ("Hills", 0.4, -1)] {
        let e = por_nome[nome];
        assert_eq!(
            w.get::<ph2d_ecs::ScrollFactor>(e).map(|f| f.k),
            Some([k, k]),
            "{nome}"
        );
        assert_eq!(
            w.get::<ph2d_ecs::ZIndexOverride>(e).map(|z| z.0),
            Some(ordem),
            "{nome}"
        );
    }
    assert!(
        w.get::<CameraFollow>(por_nome["Camera"])
            .is_some_and(|f| f.target == "Player")
    );
}

/// Corre o nível 2 como o app: o horário do condutor vira o dedo do jogador, a física avança e a ponte
/// escreve o animador. Devolve as animações por ordem de troca e a pose final.
fn corre_o_nivel_2(dedo: impl Fn(u32) -> (f32, bool)) -> (Vec<String>, Vec2, bool) {
    use ph2d_physics_ecs::{PhysicsBridge, PlayerInput};
    let mut sim = SimWorld::new();
    let jogador = montar(&mut sim, &crate::registo_de_teste(), 2);
    let mut bridge = PhysicsBridge::new();
    let mut vistas: Vec<String> = Vec::new();
    for tick in 1..=420_u32 {
        let (drive, jump) = dedo(tick + 60);
        bridge.set_player_input(
            jogador,
            PlayerInput {
                drive,
                jump,
                ..PlayerInput::default()
            },
        );
        bridge.dispatch(&mut sim, true, u64::from(tick));
        crate::personagem::anima_personagens(
            sim.world_mut(),
            &bridge,
            &Default::default(),
            &ph2d_runtime::SignalOutbox::new(),
        );
        let agora = sim
            .world()
            .get::<ph2d_ecs::SpriteAnimator>(jogador)
            .expect("animador")
            .current
            .clone();
        if vistas.last() != Some(&agora) {
            vistas.push(agora);
        }
    }
    let w = sim.world();
    let pose = w.get::<Transform>(jogador).expect("pose").translation;
    (
        vistas,
        pose,
        w.get::<Sprite>(jogador).expect("sprite").flip_x,
    )
}

/// O dedo do horário do condutor no quadro `q` (a seta direita e a de saltar premidas?).
fn dedo_do_horario(q: u32) -> (f32, bool) {
    let premida = |liga: Gesto, desliga: Gesto| {
        let em = |g| {
            HORARIO_2
                .iter()
                .find(|(_, x)| *x == g)
                .map(|(q, _)| *q)
                .expect("no horário")
        };
        (em(liga)..em(desliga)).contains(&q)
    };
    let direita = premida(Gesto::Direita(true), Gesto::Direita(false));
    (
        if direita { 1.0 } else { 0.0 },
        premida(Gesto::Salta(true), Gesto::Salta(false)),
    )
}

/// ⭐⭐ **J2 + J3**: com o horário do condutor o boneco corre, salta, cai e pára — a animação troca por
/// esta ordem, pelo estado do personagem — e acaba de pé no chão da direita, depois do vão.
#[test]
fn o_nivel_2_anda_salta_cai_e_anima() {
    let (vistas, pose, flip) = corre_o_nivel_2(dedo_do_horario);
    let ordem = ["idle", "run", "jump", "fall", "idle"];
    let mut it = vistas.iter();
    for want in ordem {
        assert!(
            it.any(|v| v == want),
            "faltou «{want}» por ordem em {vistas:?}"
        );
    }
    assert!(pose.x > 5.0, "não passou o vão: x = {}", pose.x);
    assert!(
        (0.0..2.0).contains(&pose.y),
        "não acabou de pé no chão: y = {}",
        pose.y
    );
    assert!(!flip, "a olhar para a direita não espelha");
}

/// A andar para a ESQUERDA a imagem espelha (o boneco desenhado olha para a direita).
#[test]
fn a_andar_para_a_esquerda_o_boneco_espelha() {
    let (vistas, _, flip) =
        corre_o_nivel_2(|q| (if (120..200).contains(&q) { -1.0 } else { 0.0 }, false));
    assert!(vistas.iter().any(|v| v == "run"), "{vistas:?}");
    assert!(flip);
}

/// Plano 09: no nível 2 o boneco do menu Add agacha — segurando ↓ entra sob o «Low Ceiling» que de pé o
/// pára; soltando ↓ lá debaixo continua agachado; a andar para fora, levanta.
#[test]
fn no_nivel_2_agachado_passa_sob_o_tecto_baixo() {
    use ph2d_physics_ecs::{PhysicsBridge, PlayerInput};
    let corre = |passos: &[(u32, f32, bool)]| {
        let mut sim = SimWorld::new();
        let jogador = montar(&mut sim, &crate::registo_de_teste(), 2);
        let mut bridge = PhysicsBridge::new();
        let mut t = 0_u64;
        for &(n, drive, down) in passos {
            for _ in 0..n {
                bridge.set_player_input(
                    jogador,
                    PlayerInput {
                        drive,
                        down,
                        ..PlayerInput::default()
                    },
                );
                t += 1;
                bridge.dispatch(&mut sim, true, t);
            }
        }
        let x = sim
            .world()
            .get::<Transform>(jogador)
            .expect("pose")
            .translation
            .x;
        let agachado = bridge.player_view(jogador).is_some_and(|v| v.crouching);
        let topo = bridge.body_extent(jogador).expect("corpo").1[1];
        (x, agachado, topo)
    };
    let (_, c, h) = BLOCOS_2
        .iter()
        .find(|b| b.0 == "Low Ceiling")
        .expect("o tecto");
    let (fundo, saida) = (c[1] - h[1], c[0] + h[0]);
    let (x, _, _) = corre(&[(60, 0.0, false), (150, -1.0, false)]);
    assert!(x > saida, "de pé o tecto pára-o: x = {x}");
    let (x, agachado, topo) = corre(&[(60, 0.0, false), (150, -1.0, true), (60, 0.0, false)]);
    assert!(x < saida - 0.5, "agachado entrou: x = {x}");
    assert!(
        agachado && topo < fundo,
        "lá debaixo continua baixo: topo {topo}"
    );
    let (x, agachado, _) = corre(&[(60, 0.0, false), (150, -1.0, true), (120, 1.0, false)]);
    assert!(x > saida + 0.5 && !agachado, "fora, levanta: x = {x}");
}

/// ⭐ Nível 3 (planos 09–10), pelo horário do condutor e pelas regras DERIVADAS do Add (sem regra à mão):
/// a andar agachado toca «crouch_walk»; parado sob o tecto, «crouch» (soltar ↓ lá debaixo continua);
/// a sair para a direita levanta e volta ao de pé.
#[test]
fn o_nivel_3_agacha_sob_o_tecto_e_a_regra_toca() {
    use ph2d_physics_ecs::{PhysicsBridge, PlayerInput};
    let mut sim = SimWorld::new();
    let jogador = montar(&mut sim, &crate::registo_de_teste(), 3);
    let mut bridge = PhysicsBridge::new();
    let mut premidas = std::collections::BTreeSet::new();
    let mut leituras = Vec::new();
    for tick in 1..=420_u32 {
        if let Some((seta, desce)) =
            gesto_no_quadro(3, GameStage::Window, tick + 60).and_then(Gesto::seta)
        {
            if desce {
                premidas.insert(seta);
            } else {
                premidas.remove(&seta);
            }
        }
        let tem = |s| premidas.contains(&s);
        let drive =
            f32::from(u8::from(tem(Seta::Direita))) - f32::from(u8::from(tem(Seta::Esquerda)));
        bridge.set_player_input(
            jogador,
            PlayerInput {
                drive,
                down: tem(Seta::Baixo),
                ..PlayerInput::default()
            },
        );
        bridge.dispatch(&mut sim, true, u64::from(tick));
        crate::personagem::anima_personagens(
            sim.world_mut(),
            &bridge,
            &Default::default(),
            &ph2d_runtime::SignalOutbox::new(),
        );
        let a = sim
            .world()
            .get::<ph2d_ecs::SpriteAnimator>(jogador)
            .expect("animador");
        let x = sim
            .world()
            .get::<Transform>(jogador)
            .expect("pose")
            .translation
            .x;
        leituras.push((tick + 60, a.current.clone(), x));
    }
    let em = |q: u32| leituras.iter().find(|l| l.0 == q).expect("quadro").clone();
    let (_, a_andar, x) = em(200);
    assert_eq!(a_andar, "crouch_walk", "a andar agachado: x = {x}");
    let (_, debaixo, x) = em(320);
    assert_eq!(
        debaixo, "crouch",
        "solto ↓ debaixo do tecto continua agachado: x = {x}"
    );
    assert!(x < -9.5, "e está debaixo dele: x = {x}");
    let (_, fora, x) = em(470);
    assert_eq!(fora, "idle", "fora do tecto levantou e parou: x = {x}");
    assert!(x > -9.0, "saiu: x = {x}");
}

/// ⭐ **O topo do DESENHO agachado é o do CORPO agachado** (plano 10), medido na física do nível 2: o boneco do
/// Add, parado, agachado ou de pé; o topo do corpo (`body_extent`) contra a linha mais alta de cada célula
/// da tag, os dois acima do centro da sprite (1 × 1 m). CONTROLO na mesma régua: `idle` contra o corpo de pé.
#[test]
fn o_desenho_agachado_tem_o_topo_do_corpo() {
    use ph2d_physics_ecs::{PhysicsBridge, PlayerInput};
    use ph2d_render::{HERO_SHEET_CELL_PX, HERO_SHEET_COLS, HERO_SHEET_TAGS, hero_sheet_pixels};
    let n = HERO_SHEET_CELL_PX as usize;
    let w = n * HERO_SHEET_COLS as usize;
    let px = hero_sheet_pixels();
    let topo_da_celula = |i: u32| -> f32 {
        let (row, col) = (
            (i / HERO_SHEET_COLS) as usize,
            (i % HERO_SHEET_COLS) as usize,
        );
        let y = (0..n)
            .find(|y| (0..n).any(|x| px[(row * n + y) * w + col * n + x].a() > 64))
            .expect("desenho");
        0.5 - y as f32 / n as f32
    };
    let corpo = |down: bool| -> f32 {
        let mut sim = SimWorld::new();
        let jogador = montar(&mut sim, &crate::registo_de_teste(), 2);
        let mut bridge = PhysicsBridge::new();
        for t in 1..=120_u64 {
            bridge.set_player_input(
                jogador,
                PlayerInput {
                    down,
                    ..PlayerInput::default()
                },
            );
            bridge.dispatch(&mut sim, true, t);
        }
        let centro = sim
            .world()
            .get::<Transform>(jogador)
            .expect("pose")
            .translation
            .y;
        assert_eq!(
            bridge.player_view(jogador).is_some_and(|v| v.crouching),
            down
        );
        bridge.body_extent(jogador).expect("corpo").1[1] - centro
    };
    let (de_pe, agachado) = (corpo(false), corpo(true));
    assert!(
        de_pe - agachado > 0.25,
        "o corpo baixou: {de_pe} → {agachado}"
    );
    for (tag, alvo) in [
        ("idle", de_pe),
        ("crouch", agachado),
        ("crouch_walk", agachado),
    ] {
        let t = HERO_SHEET_TAGS
            .iter()
            .find(|t| t.name == tag)
            .expect("tag da folha");
        for i in t.from..=t.to {
            let topo = topo_da_celula(i);
            assert!(
                (topo - alvo).abs() <= 0.03,
                "{tag} célula {i}: o desenho chega a {topo:.3} m, o corpo a {alvo:.3} m"
            );
        }
    }
}

/// Plano 10: o Character do menu traz as tags do agachar, e as regras que valem nele (o cartão e o jogo
/// leem a mesma porta) são as 6 — sem nada gravado.
#[test]
fn o_personagem_do_menu_deriva_as_seis_regras() {
    use ph2d_physics_ecs::{AnimRules, AnimWhen, CharacterPose};
    let mut sim = SimWorld::new();
    let jogador = montar(&mut sim, &crate::registo_de_teste(), 2);
    assert!(
        sim.world().get::<AnimRules>(jogador).is_none(),
        "nada gravado"
    );
    let r = AnimRules::of_entity(sim.world(), jogador).expect("regras");
    let whens: Vec<_> =
        r.0.iter()
            .map(|r| (r.when.clone(), r.play.as_str()))
            .collect();
    assert_eq!(whens.len(), 6);
    assert_eq!(
        whens[..2],
        [
            (AnimWhen::Pose(CharacterPose::CrouchWalk), "crouch_walk"),
            (AnimWhen::Pose(CharacterPose::Crouch), "crouch"),
        ]
    );
}
