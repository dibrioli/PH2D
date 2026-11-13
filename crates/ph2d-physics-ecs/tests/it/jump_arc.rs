//! **A curva do canvas é o pulo do jogo** (`ph2d_platformer::arc`, ficha GEF 04): o arco que o
//! editor desenha contra o player REAL na ponte (rapier, mola, cápsula, sub-passos) num chão plano.

use crate::platform_scene::{pose, scene};

use ph2d_ecs::{Entity, SimWorld};
use ph2d_physics_ecs::{PhysicsBridge, PlatformPlayer, PlayerInput};
use ph2d_platformer::arc::{JumpArc, TouchKind};
use ph2d_platformer::feel::Feel;

/// O que se compara de um pulo: o ápice (acima do arranque, para o lado), o tique do pouso e o
/// alcance nele.
#[derive(Debug)]
struct Medido {
    apex_up: f32,
    apex_side: f32,
    air_ticks: usize,
    reach: f32,
}

fn apice(pts: &[[f32; 2]]) -> [f32; 2] {
    pts.iter()
        .copied()
        .max_by(|a, b| a[1].total_cmp(&b[1]))
        .unwrap()
}

/// O desenho cortado no terreno: o pouso é o fim do traço (o tique em que a perna agarra).
fn do_arco(a: &JumpArc) -> Medido {
    let top = apice(&a.points);
    let toque = a.touch.expect("a curva não pousou");
    assert_eq!(toque.kind, TouchKind::Land, "{toque:?}");
    Medido {
        apex_up: top[1],
        apex_side: top[0],
        air_ticks: a.points.len() - 1,
        reach: toque.at[0],
    }
}

/// Quanto o jogo pode afastar-se do voo livre da lei sem ter pousado: no ar o desenho é o jogo a
/// ~3e-7 m (12/11); no tique em que a perna agarra na ponta do alcance a mola desce-o só 0,4 mm
/// (Floaty 2,5 m), então 1 mm dava o pouso um tique tarde.
const VOO: f32 = 1.0e-4;

/// O pouso do JOGO: o primeiro tique em que o player real sai do voo livre (`voo`, a lei sem
/// corte) — a perna agarrou e a mola tomou conta. Não há outro relógio público do pouso: o
/// `footing` diz «chão» a subida inteira de um pulo baixo (o raio vê o chão de partida).
fn do_jogo_medido(pts: &[[f32; 2]], voo: &[[f32; 2]]) -> Medido {
    let top = apice(pts);
    let i = (1..pts.len().min(voo.len()))
        .find(|&i| (pts[i][1] - voo[i][1]).abs() > VOO)
        .unwrap_or(pts.len() - 1);
    Medido {
        apex_up: top[1],
        apex_side: top[0],
        air_ticks: i,
        reach: pts[i][0],
    }
}

#[derive(Copy, Clone)]
enum Como {
    Correndo,
    Parado,
    Curto,
}

/// O desenho (lei + corte no chão, lidos na pose assentada) e o pulo do player REAL.
fn caso(feel: Option<Feel>, como: Como, h: f32) -> (Medido, Medido) {
    let (mut sim, mut bridge, player) = scene(0.0, -20.0);
    {
        let mut e = sim.world_mut().entity_mut(player);
        let mut p = e.get_mut::<PlatformPlayer>().unwrap();
        if let Some(f) = feel {
            p.set_feel(f);
        }
        p.jump_height = h;
    }
    let drive = 1.0;
    let mut tick = 0_u64;
    // Assenta na perna; correndo, já chega à velocidade máxima antes do salto.
    for _ in 0..60 {
        let d = if matches!(como, Como::Parado) {
            0.0
        } else {
            drive
        };
        bridge.set_player_input(
            player,
            PlayerInput {
                drive: d,
                ..PlayerInput::default()
            },
        );
        tick += 1;
        bridge.dispatch(&mut sim, true, tick);
    }
    let (x0, y0) = pose(&sim);
    let settings = bridge.settings();
    let w = ph2d_physics_ecs::arc_world(&settings, 1.0);
    let (mut arcs, fh) = {
        let p = sim.world().get::<PlatformPlayer>(player).unwrap();
        (p.jump_arcs(w), p.float_height)
    };
    let pick = |a: &ph2d_platformer::arc::JumpArcs| match como {
        Como::Correndo => a.full.clone(),
        Como::Parado => a.standing.clone(),
        Como::Curto => a.short.clone(),
    };
    let voo = pick(&arcs).points;
    ph2d_physics_ecs::arc_terrain::cut_on_terrain(
        sim.world(),
        &settings,
        player,
        [x0, y0 - fh],
        [0.0, 1.0],
        &mut arcs,
    );
    let arco = pick(&arcs);
    let mut pts = vec![[0.0, 0.0]];
    for i in 0..200 {
        let jump = match como {
            Como::Curto => i == 0,
            _ => true,
        };
        bridge.set_player_input(
            player,
            PlayerInput {
                drive,
                jump,
                ..PlayerInput::default()
            },
        );
        tick += 1;
        bridge.dispatch(&mut sim, true, tick);
        let (x, y) = pose(&sim);
        pts.push([x - x0, y - y0]);
    }
    (do_arco(&arco), do_jogo_medido(&pts, &voo))
}

/// As alturas dos gates da Jump Height (m): a de partida e as pontas da faixa que se usa.
const ALTURAS: [f32; 4] = [0.5, 1.0, 2.0, 2.5];

/// Meio pixel a 100 px/m: o que o número do cartão e o da alça, em px inteiros, deixam de esconder.
const MEIO_PX: f32 = 0.005;

/// O ápice contra a Jump Height: a lei com os passos do mundo erra ≤ 0,3 mm medido (12/11); sem
/// eles, até 4,7 mm no jogo e 1,9 cm no desenho, que o meio pixel deixava passar no jogo.
const LEI: f32 = 0.001;

#[test]
fn o_arco_do_desenho_e_o_do_jogo() {
    let mut falhas = Vec::new();
    for feel in [
        None,
        Some(Feel::Floaty),
        Some(Feel::Snappy),
        Some(Feel::Heavy),
    ] {
        for h in ALTURAS {
            for (nome, como) in [
                ("correndo", Como::Correndo),
                ("parado", Como::Parado),
                ("curto", Como::Curto),
            ] {
                let (a, j) = caso(feel, como, h);
                eprintln!("{feel:?} h={h} {nome}: desenho {a:?}\n  jogo    {j:?}");
                // Lado do ápice: um tique de corrida (6 m/s · 1/60), porque num topo achatado dois
                // tiques empatam. O pulo inteiro (não o curto) sobe a Jump Height, ao meio pixel.
                let cheio = !matches!(como, Como::Curto);
                let apice = (a.apex_up - j.apex_up).abs() < MEIO_PX
                    && (!cheio || ((a.apex_up - h).abs() <= LEI && (j.apex_up - h).abs() <= LEI))
                    && (a.apex_side - j.apex_side).abs() < 0.11;
                // O pouso em TODAS as alturas: o MESMO tique em que a perna agarra (medido 12/11:
                // 48/48 iguais, alcance a ≤ 1 cm). ±1 escondia o sinal da velocidade no ápice (a
                // perna lida pela média do tique pousa um tique tarde o pulo curto de 0,5 m).
                let pouso = a.air_ticks == j.air_ticks && (a.reach - j.reach).abs() < 0.11;
                let ok = apice && pouso;
                if !ok {
                    falhas.push(format!("{feel:?} h={h} {nome}: desenho {a:?} jogo {j:?}"));
                }
            }
        }
    }
    assert!(falhas.is_empty(), "o desenho mente em: {falhas:?}");
}

/// O player REAL na ponte: segura o pulo, e no ápice do do chão solta um tique e volta a premir
/// (o pulo do ar). Devolve a subida de cada um, medida do próprio arranque.
fn subidas_do_jogo(feel: Option<Feel>, h: f32, air_h: f32) -> (f32, f32, f32, f32) {
    let (mut sim, mut bridge, player) = scene(0.0, -20.0);
    let (h, ah) = {
        let mut e = sim.world_mut().entity_mut(player);
        let mut p = e.get_mut::<PlatformPlayer>().unwrap();
        if let Some(f) = feel {
            p.set_feel(f);
        }
        p.jump_height = h;
        p.air_jumps = 1;
        p.air_jump_height = air_h;
        (p.jump_height, p.air_jump_height)
    };
    let mut tick = 0_u64;
    let mut passo = |sim: &mut SimWorld, bridge: &mut PhysicsBridge, jump: bool| {
        bridge.set_player_input(
            player,
            PlayerInput {
                jump,
                ..PlayerInput::default()
            },
        );
        tick += 1;
        bridge.dispatch(sim, true, tick);
        pose(sim).1
    };
    for _ in 0..60 {
        passo(&mut sim, &mut bridge, false);
    }
    let y0 = pose(&sim).1;
    let (mut topo, mut antes) = (y0, y0);
    loop {
        let y = passo(&mut sim, &mut bridge, true);
        if y < antes {
            break;
        }
        topo = topo.max(y);
        antes = y;
    }
    let y1 = passo(&mut sim, &mut bridge, false);
    let mut topo_ar = y1;
    for _ in 0..200 {
        topo_ar = topo_ar.max(passo(&mut sim, &mut bridge, true));
    }
    (topo - y0, h, topo_ar - y1, ah)
}

/// A alça escreve o ápice pedido, ao centímetro, em qualquer sensação: cima = `jump_height`
/// (é o ápice), lado = `speed`. O desenho é o jogo (gate acima), então o desenho basta.
#[test]
fn a_alca_poe_o_apice_onde_foi_pedido() {
    let mut falhas = Vec::new();
    for feel in [
        None,
        Some(Feel::Floaty),
        Some(Feel::Snappy),
        Some(Feel::Heavy),
    ] {
        let (mut sim, bridge, player) = scene(0.0, -20.0);
        let w = ph2d_physics_ecs::arc_world(&bridge.settings(), 1.0);
        let mut e = sim.world_mut().entity_mut(player);
        let mut p = e.get_mut::<PlatformPlayer>().unwrap();
        if let Some(f) = feel {
            p.set_feel(f);
        }
        p.pull_apex(2.0, 2.6, w);
        let a = p.jump_arcs(w).full.apex;
        if (a[1] - 2.6).abs() >= 0.02 || (a[0] - 2.0).abs() >= 0.02 {
            falhas.push(format!("{feel:?}: ápice {a:?}"));
        }
    }
    assert!(falhas.is_empty(), "a alça erra o alvo [2, 2.6]: {falhas:?}");
}

/// ⭐ GEF 04, «altura que mente»: o campo Jump Height (e o Air Jump Height) é o ápice REAL do
/// jogo, em qualquer sensação — a lei resolve o arranque pelas fases de gravidade.
#[test]
fn o_pulo_do_jogo_sobe_a_jump_height() {
    let mut falhas = Vec::new();
    for feel in [
        None,
        Some(Feel::Floaty),
        Some(Feel::Snappy),
        Some(Feel::Heavy),
    ] {
        for h in ALTURAS {
            let (sobe, h, sobe_ar, ah) = subidas_do_jogo(feel, h, 0.75 * h);
            eprintln!("{feel:?}: chão {sobe:.4} (Jump Height {h}) · ar {sobe_ar:.4} (Air {ah})");
            if (sobe - h).abs() > LEI || (sobe_ar - ah).abs() > LEI {
                falhas.push(format!("{feel:?}: chão {sobe:.4}/{h} ar {sobe_ar:.4}/{ah}"));
            }
        }
    }
    assert!(falhas.is_empty(), "o campo mente: {falhas:?}");
}

/// A lei modela os passos do mundo de fábrica ([`ph2d_platformer::JumpConfig::world_steps`]): se
/// os sub-passos ou as iterações de fábrica mudarem, o ápice do jogo sai da Jump Height.
#[test]
fn os_passos_do_mundo_de_fabrica_sao_os_da_lei() {
    let s = ph2d_physics_ecs::PhysicsSettings::default();
    assert_eq!(
        ph2d_physics_ecs::world_steps(&s),
        ph2d_platformer::JumpConfig::DEFAULT_WORLD_STEPS
    );
    let (_, bridge, _) = scene(0.0, -20.0);
    assert_eq!(
        ph2d_physics_ecs::world_steps(&bridge.settings()),
        ph2d_platformer::JumpConfig::DEFAULT_WORLD_STEPS,
        "a cena dos gates corre no mundo de fábrica"
    );
}

// ── GEF 04: a curva pousa no TERRENO ─────────────────────────────────────────────────────────────

/// Uma caixa estática de topo em `top`, de `x0` a `x1`, com `h` de altura.
fn caixa(sim: &mut SimWorld, x0: f32, x1: f32, top: f32, h: f32) -> Entity {
    use ph2d_core::Vec2;
    use ph2d_ecs::{Name, Transform};
    use ph2d_physics_ecs::{BodyKind, Collider, ColliderShape, RigidBody};
    sim.world_mut()
        .spawn((
            Name::new("Terreno"),
            RigidBody {
                kind: BodyKind::Static,
            },
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x: (x1 - x0) * 0.5,
                    half_y: h * 0.5,
                },
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new((x0 + x1) * 0.5, top - h * 0.5)),
        ))
        .id()
}

/// Uma caixa-sensor (gatilho): o corpo atravessa-a.
fn sensor(sim: &mut SimWorld, x0: f32, x1: f32, top: f32, h: f32) {
    let e = caixa(sim, x0, x1, top, h);
    let mut c = sim
        .world_mut()
        .get_mut::<ph2d_physics_ecs::Collider>(e)
        .unwrap();
    c.is_sensor = true;
}

/// Assenta o player 60 tiques (`correndo` ou parado). Devolve o tique corrente.
fn assenta(sim: &mut SimWorld, bridge: &mut PhysicsBridge, player: Entity, correndo: bool) -> u64 {
    for tick in 1..=60 {
        bridge.set_player_input(
            player,
            PlayerInput {
                drive: if correndo { 1.0 } else { 0.0 },
                ..PlayerInput::default()
            },
        );
        bridge.dispatch(sim, true, tick);
    }
    60
}

/// A curva desenhada (cortada no terreno) e o pulo real: pontos relativos ao centro do arranque, e
/// o voo livre da lei (a curva sem corte).
fn terreno(
    extra: impl Fn(&mut SimWorld, f32),
    correndo: bool,
) -> (JumpArc, Vec<[f32; 2]>, Vec<[f32; 2]>) {
    terreno_com(extra, correndo, None, None)
}

/// [`terreno`] com uma sensação e uma Jump Height (`ajuste`).
fn terreno_com(
    extra: impl Fn(&mut SimWorld, f32),
    correndo: bool,
    ajuste: Option<(Option<Feel>, f32)>,
    matriz: Option<ph2d_physics_ecs::LayerMatrix>,
) -> (JumpArc, Vec<[f32; 2]>, Vec<[f32; 2]>) {
    let ajusta = |sim: &mut SimWorld, player: Entity| {
        if let Some((feel, h)) = ajuste {
            let mut e = sim.world_mut().entity_mut(player);
            let mut p = e.get_mut::<PlatformPlayer>().unwrap();
            if let Some(f) = feel {
                p.set_feel(f);
            }
            p.jump_height = h;
        }
    };
    // O x em que o player fica depois de assentar: corrida de ensaio sem terreno extra.
    let x_assentado = {
        let (mut sim, mut bridge, player) = scene(0.0, 0.0);
        ajusta(&mut sim, player);
        assenta(&mut sim, &mut bridge, player, correndo);
        pose(&sim).0
    };
    let (mut sim, mut bridge, player) = scene(0.0, 0.0);
    ajusta(&mut sim, player);
    extra(&mut sim, x_assentado);
    if let Some(m) = matriz {
        let mut s = bridge.settings();
        s.layer_matrix = m.rows();
        bridge.set_settings(s);
    }
    let mut tick = assenta(&mut sim, &mut bridge, player, correndo);
    let (x0, y0) = pose(&sim);
    let settings = bridge.settings();
    let w = ph2d_physics_ecs::arc_world(&settings, 1.0);
    let (mut arcs, fh) = {
        let p = sim.world().get::<PlatformPlayer>(player).unwrap();
        (p.jump_arcs(w), p.float_height)
    };
    let chao = [x0, y0 - fh];
    let voo = if correndo { &arcs.full } else { &arcs.standing }
        .points
        .clone();
    ph2d_physics_ecs::arc_terrain::cut_on_terrain(
        sim.world(),
        &settings,
        player,
        chao,
        [0.0, 1.0],
        &mut arcs,
    );
    let arco = if correndo { arcs.full } else { arcs.standing };
    let mut pts = vec![[0.0, 0.0]];
    for _ in 0..240 {
        bridge.set_player_input(
            player,
            PlayerInput {
                drive: 1.0,
                jump: true,
                ..PlayerInput::default()
            },
        );
        tick += 1;
        bridge.dispatch(&mut sim, true, tick);
        let (x, y) = pose(&sim);
        pts.push([x - x0, y - y0]);
    }
    (arco, pts, voo)
}

/// O pouso do jogo ([`do_jogo_medido`]), o x nele, e a altura em que o corpo ASSENTA logo depois
/// (a da superfície que o pegou).
fn pouso_real(pts: &[[f32; 2]], voo: &[[f32; 2]]) -> (usize, f32, f32) {
    let m = do_jogo_medido(pts, voo);
    let i = m.air_ticks;
    // Assenta: 20 tiques depois (a mola acaba de apanhar o corpo; ele continua a correr).
    (i, m.reach, pts[(i + 20).min(pts.len() - 1)][1])
}

fn confere_pouso(
    nome: &str,
    arco: &JumpArc,
    (pts, voo): (&[[f32; 2]], &[[f32; 2]]),
    h_esperada: f32,
) {
    let toque = arco
        .touch
        .unwrap_or_else(|| panic!("{nome}: a curva não tocou o terreno"));
    assert_eq!(toque.kind, TouchKind::Land, "{nome}: o pé pousa, não bate");
    assert!(
        (toque.contact[1] - h_esperada).abs() < 0.02,
        "{nome}: o contacto não está na superfície {:?}",
        toque.contact
    );
    let t = toque.at;
    let tique = arco.points.len() - 1;
    let (i, x, assenta) = pouso_real(pts, voo);
    eprintln!(
        "{nome}: desenho toca {t:?} no tique {tique} · jogo {i} x {x:.3} assenta {assenta:.3}"
    );
    assert!(
        (t[1] - h_esperada).abs() < 0.02,
        "{nome}: tocou a altura errada {t:?}"
    );
    assert!(
        (assenta - t[1]).abs() < 0.03,
        "{nome}: o jogo assenta em {assenta}, a curva em {}",
        t[1]
    );
    assert_eq!(tique, i, "{nome}: tique {tique} × jogo {i}");
    assert!((t[0] - x).abs() < 0.11, "{nome}: x {} × jogo {x}", t[0]);
}

/// Põe o player `dy` metros acima de onde a cena o pôs (antes do primeiro tique).
fn ergue(sim: &mut SimWorld, dy: f32) {
    let mut q = sim
        .world_mut()
        .query::<(&ph2d_ecs::Name, &mut ph2d_ecs::Transform)>();
    for (n, mut t) in q.iter_mut(sim.world_mut()) {
        if n.as_str() == "Player" {
            t.translation.y += dy;
        }
    }
}

/// As quatro sensações dos gates.
const FEELS: [Option<Feel>; 4] = [
    None,
    Some(Feel::Floaty),
    Some(Feel::Snappy),
    Some(Feel::Heavy),
];

/// (a) Arranque numa plataforma 1 m acima do chão, que acaba à frente: a curva DESCE ao chão de
/// baixo e pousa onde o player real pousa. Corrida e parado, em toda Jump Height e sensação.
#[test]
fn a_curva_desce_ao_chao_de_baixo_e_pousa_onde_o_jogo_pousa() {
    for feel in FEELS {
        for h in ALTURAS {
            for correndo in [true, false] {
                let (arco, pts, voo) = terreno_com(
                    |sim, x| {
                        caixa(sim, x - 8.0, x + 0.6, 1.5, 1.0);
                        ergue(sim, 1.0);
                        // Uma zona-sensor atravessada pela curva: não a pára (o jogo atravessa-a).
                        sensor(sim, x + 2.0, x + 5.0, 4.0, 3.5);
                    },
                    correndo,
                    Some((feel, h)),
                    None,
                );
                confere_pouso(
                    &format!("desce {feel:?} h={h} correndo={correndo}"),
                    &arco,
                    (&pts, &voo),
                    -1.0,
                );
            }
        }
    }
}

/// (b) Uma plataforma 1 m mais alta à frente, que o pulo alcança: a curva pousa em cima dela —
/// nas Jump Heights que passam de 1 m, em toda sensação.
#[test]
fn a_curva_pousa_na_plataforma_mais_alta() {
    for feel in FEELS {
        for h in [1.25, 2.0, 2.5] {
            for correndo in [true, false] {
                let ahead = if correndo { 3.0 } else { 1.5 };
                let (arco, pts, voo) = terreno_com(
                    |sim, x| {
                        caixa(sim, x + ahead, x + 20.0, 1.5, 1.0);
                    },
                    correndo,
                    Some((feel, h)),
                    None,
                );
                confere_pouso(
                    &format!("sobe {feel:?} h={h} correndo={correndo}"),
                    &arco,
                    (&pts, &voo),
                    1.0,
                );
            }
        }
    }
}

/// (c) Uma parede à frente: a curva pára NA parede (o corpo encosta nela), onde o jogo encosta.
#[test]
fn a_curva_para_na_parede() {
    let face = 2.0;
    let (arco, pts, _) = terreno(
        |sim, x| {
            caixa(sim, x + face, x + face + 1.0, 10.0, 9.5);
        },
        true,
    );
    let toque = arco.touch.expect("a curva não tocou a parede");
    assert_eq!(
        toque.kind,
        TouchKind::Bump,
        "a parede bate no lado: {toque:?}"
    );
    assert!(
        (toque.contact[0] - face).abs() < 0.02,
        "o contacto está na face da parede: {toque:?}"
    );
    let t = toque.at;
    let real = pts.iter().map(|p| p[0]).fold(f32::MIN, f32::max);
    eprintln!("parede: desenho {t:?} · jogo encosta em x {real:.3}");
    // A cápsula (raio 0,2) encosta na face: o centro fica a 0,2 dela.
    assert!(
        (t[0] - (face - 0.2)).abs() < 0.02,
        "a curva atravessa a parede: {t:?}"
    );
    assert!((t[0] - real).abs() < 0.11, "desenho {} × jogo {real}", t[0]);
    assert!(t[1] > 0.0, "a parede pega-o no ar: {t:?}");
}

/// (d) Um teto baixo por cima: a cabeça bate nele e a curva pára por CHOQUE, com o contacto NO teto.
#[test]
fn o_teto_baixo_para_a_curva_por_choque_com_o_contacto_no_teto() {
    // O fundo do teto, acima do pé (o chão de `platform_scene` tem o topo em 0,5).
    let fundo = 1.6;
    let (arco, _, _) = terreno(
        |sim, x| {
            caixa(sim, x - 3.0, x + 6.0, 0.5 + fundo + 0.4, 0.4);
        },
        true,
    );
    let toque = arco.touch.expect("a curva não tocou o teto");
    assert_eq!(toque.kind, TouchKind::Bump, "a cabeça bate: {toque:?}");
    assert!(
        (toque.contact[1] - fundo).abs() < 0.02,
        "o contacto está no fundo do teto: {toque:?}"
    );
    assert!(toque.at[1] < fundo, "o pé fica abaixo do teto: {toque:?}");
}

/// Sonda: o custo de UMA chamada do desenho (lei + corte no terreno) na cena (a), com Air Jumps 1.
/// `PH2D_RELEASE="mede velocidade" bash scripts/ph2d-run.sh cargo test --release -p ph2d-physics-ecs custo_das_curvas_no_terreno -- --ignored --nocapture`
#[test]
#[ignore = "sonda: imprime o custo"]
fn custo_das_curvas_no_terreno() {
    let (mut sim, bridge, player) = scene(0.0, 0.0);
    caixa(&mut sim, -8.0, 0.6, 1.5, 1.0);
    sensor(&mut sim, 2.0, 5.0, 4.0, 3.5);
    ergue(&mut sim, 1.0);
    let settings = bridge.settings();
    let w = ph2d_physics_ecs::arc_world(&settings, 1.0);
    sim.world_mut()
        .get_mut::<PlatformPlayer>(player)
        .unwrap()
        .air_jumps = 1;
    let (x0, y0) = pose(&sim);
    let chao = [x0, y0 - crate::platform_scene::FLOAT_HEIGHT];
    let mut best = f64::MAX;
    for _ in 0..5 {
        let t = std::time::Instant::now();
        for _ in 0..200 {
            let p = sim.world().get::<PlatformPlayer>(player).unwrap();
            let mut arcs = p.jump_arcs(w);
            ph2d_physics_ecs::arc_terrain::cut_on_terrain(
                sim.world(),
                &settings,
                player,
                chao,
                [0.0, 1.0],
                &mut arcs,
            );
            std::hint::black_box(arcs);
        }
        best = best.min(t.elapsed().as_secs_f64() / 200.0);
    }
    eprintln!(
        "lei + corte (3 formas, Air Jumps 1): {:.1} µs por chamada (mínimo de 5 blocos)",
        best * 1e6
    );
}

// ── GEF 04: o corte segue a exceção de camadas de CADA forma (`layer_override`) ───────────────────

/// A camada da plataforma dos casos de exceção (o player e o chão moram na 0).
const OUTRA: u8 = 3;

/// O player da cena.
fn o_player(sim: &mut SimWorld) -> Entity {
    let mut q = sim.world_mut().query::<(Entity, &ph2d_ecs::Name)>();
    q.iter(sim.world())
        .find(|(_, n)| n.as_str() == "Player")
        .map(|(e, _)| e)
        .unwrap()
}

fn excecao(sim: &mut SimWorld, e: Entity, include: u32, exclude: u32) {
    sim.world_mut()
        .entity_mut(e)
        .insert(ph2d_physics_ecs::CollisionOverride { include, exclude });
}

/// A plataforma de (b), 1 m acima, na camada [`OUTRA`], com a exceção nela ou (`no_player`) no
/// player.
fn plataforma_com_excecao(
    nome: &str,
    no_player: bool,
    include: u32,
    exclude: u32,
    matriz: Option<ph2d_physics_ecs::LayerMatrix>,
    h_esperada: f32,
) {
    let (arco, pts, voo) = terreno_com(
        |sim, x| {
            let p = caixa(sim, x + 3.0, x + 20.0, 1.5, 1.0);
            sim.world_mut()
                .get_mut::<ph2d_physics_ecs::Collider>(p)
                .unwrap()
                .layer = OUTRA;
            let quem = if no_player { o_player(sim) } else { p };
            excecao(sim, quem, include, exclude);
        },
        true,
        None,
        matriz,
    );
    confere_pouso(nome, &arco, (&pts, &voo), h_esperada);
}

/// A matriz em que a camada do player e a [`OUTRA`] NÃO colidem.
fn matriz_separada() -> ph2d_physics_ecs::LayerMatrix {
    let mut m = ph2d_physics_ecs::LayerMatrix::all();
    m.set(0, usize::from(OUTRA), false);
    m
}

/// A exceção que SOLTA: a plataforma colide pela matriz, mas uma exceção (dela ou do player)
/// exclui o par — o player real atravessa-a e pousa no chão de baixo, e a curva também.
#[test]
fn a_excecao_que_solta_deixa_a_curva_atravessar_como_o_jogo() {
    plataforma_com_excecao("cena exclui o player", false, 0, 1, None, 0.0);
    plataforma_com_excecao("player exclui a cena", true, 0, 1 << OUTRA, None, 0.0);
}

/// A exceção que PRENDE: a matriz separa as duas camadas, mas um `include` (dela ou do player)
/// junta-as — o player real pousa na plataforma, e a curva também.
#[test]
fn a_excecao_que_prende_para_a_curva_como_o_jogo() {
    let m = Some(matriz_separada());
    plataforma_com_excecao("cena inclui o player", false, 1, 0, m, 1.0);
    plataforma_com_excecao("player inclui a cena", true, 1 << OUTRA, 0, m, 1.0);
    // O controlo: sem exceção a matriz separa, e os dois atravessam.
    plataforma_com_excecao("só a matriz", false, 0, 0, m, 0.0);
}

/// (e) **A perna agarra a borda que o pé não alcança**: uma plataforma 1 m acima que acaba onde o
/// pé, a descer, ainda vai 12 cm acima do topo dela (dentro do `cling_distance`). O pé nunca cruza
/// o topo, mas o player real pousa na borda — e a curva também (o critério do pé passava por cima).
#[test]
fn a_perna_agarra_a_borda_que_o_pe_nao_alcanca() {
    let voo = {
        let (mut sim, mut bridge, player) = scene(0.0, 0.0);
        assenta(&mut sim, &mut bridge, player, true);
        let w = ph2d_physics_ecs::arc_world(&bridge.settings(), 1.0);
        let p = sim.world().get::<PlatformPlayer>(player).unwrap();
        p.jump_arcs(w).full.points
    };
    let top = voo
        .iter()
        .enumerate()
        .max_by(|a, b| a.1[1].total_cmp(&b.1[1]))
        .map_or(0, |(i, _)| i);
    let borda = voo[(top..voo.len()).find(|&i| voo[i][1] < 1.12).unwrap()][0] + 0.05;
    let (arco, pts, voo) = terreno(
        |sim, x| {
            caixa(sim, x + 3.0, x + borda, 1.5, 1.0);
        },
        true,
    );
    // Pousa na borda e, a correr, sai dela logo depois: o assentar dos outros casos não vale aqui.
    let toque = arco.touch.expect("a curva não tocou a borda");
    assert_eq!(toque.kind, TouchKind::Land, "{toque:?}");
    assert!(
        (toque.at[1] - 1.0).abs() < 0.02,
        "a curva passou a borda: {toque:?}"
    );
    let (i, x, _) = pouso_real(&pts, &voo);
    let tique = arco.points.len() - 1;
    eprintln!(
        "borda: desenho {:?} no tique {tique} · jogo {i} x {x:.3} · y a seguir {:?}",
        toque.at,
        &pts[i..i + 3]
    );
    assert_eq!(tique, i, "tique {tique} × jogo {i}");
    assert!(
        (toque.at[0] - x).abs() < 0.11,
        "x {} × jogo {x}",
        toque.at[0]
    );
    assert!(
        pts[i + 2][1] > 0.9,
        "o jogo não ficou na borda: {:?}",
        &pts[i..i + 3]
    );
}
