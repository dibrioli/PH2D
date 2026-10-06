//! ⛔⛔ **Censo: nenhuma cena de smoke desenha FORA do corpo.**
//!
//! Relato do dono (05/10): morcegos desenhados como quadrado de 0,45 m sobre uma `Ball` inscrita
//! — os cantos saíam 41 % do corpo e entravam na lava sem queimar. A família é a mesma em todo o
//! lado: `WHITE_TILE_KEY` (quadrado cheio) sobre `Ball`. A população é a DECLARAÇÃO da família
//! (`FAMILY.routers` × `1..=max_level`); um roteador sem construtor aqui reprova.

use crate::scene_ctx::SceneCtx;
use bevy_ecs::world::World;
use ph2d_ecs::{ChildOf, Name, SimWorld, Transform};
use ph2d_physics_ecs::{Collider, ColliderShape};
use ph2d_render::{DISC_TILE_KEY, Sprite, SpriteSource};

const TOL: f32 = 1e-4;

/// Cenas que não se montam sem a shell. Vazio: toda cena da família monta-se sem GPU.
const SEM_CORPOS: &[(&str, u32, &str)] = &[];

/// Os empréstimos que uma cena pede, montados à mão (o relógio e a árvore de tags não precisam de ecrã).
fn com_ctx(f: impl FnOnce(&mut SceneCtx)) -> SimWorld {
    let mut sim = SimWorld::new();
    let mut vec_scene = ph2d_vec_scene::VecScene::new();
    let mut vec_entities = ph2d_vec_entities::entities::VecEntityMap::new();
    let registry = crate::test_support::registo();
    let mut tags = ph2d_tags::TagTree::new();
    let mut playhead = ph2d_core::Playhead::default();
    let mut camera_preview = false;
    {
        let mut cx = SceneCtx {
            sim: &mut sim,
            vec_scene: &mut vec_scene,
            vec_entities: &mut vec_entities,
            registry: &registry,
            tags: &mut tags,
            hero_screen: None,
            playhead: &mut playhead,
            audio_ready: false,
            camera_preview: &mut camera_preview,
        };
        f(&mut cx);
    }
    sim
}

/// ⭐ A TABELA DOS CONSTRUTORES — um braço por roteador de `FAMILY`, o MESMO `montar` da shell.
fn monta(env: &str, nivel: u32) -> SimWorld {
    use crate as c;
    let mut tree = ph2d_tags::TagTree::new();
    let mut sim = SimWorld::new();
    match env {
        "PH2D_AUDIO_2D_SMOKE" => return com_ctx(|cx| c::audio_2d_smoke::audio_2d_smoke(cx)),
        "PH2D_GAME_CAMERA_SMOKE" => return com_ctx(|cx| c::camera_2d_smoke::game_camera_smoke(cx)),
        "PH2D_INSTANCE_SMOKE" => {
            return com_ctx(|cx| match nivel {
                1 => c::instance_smoke::instance_smoke_ragdoll(cx),
                2 => c::instance_smoke::instance_smoke_vector(cx),
                3 => c::instance_nested_smoke::instance_smoke_nested(cx),
                4 => c::instance_replace_smoke::instance_smoke_replace(cx),
                5 => c::instance_removed_smoke::instance_smoke_removed(cx),
                6 => c::instance_added_smoke::instance_smoke_added(cx),
                7 => c::instance_move_smoke::instance_smoke_move(cx),
                n => panic!("INSTANCE nivel {n} sem construtor no censo"),
            });
        }
        "PH2D_SIGNAL_ACTION_SMOKE" => {
            return com_ctx(|cx| c::signal_action_smoke::signal_action_smoke(cx));
        }
        "PH2D_FACTORY_SMOKE" => {
            return com_ctx(|cx| {
                c::factory_smoke::factory_smoke(cx, nivel);
            });
        }
        "PH2D_TAGS_SMOKE" => {
            return com_ctx(|cx| {
                c::tags_smoke::tags_smoke(cx, nivel);
            });
        }
        "PH2D_TIMER_SMOKE" => return com_ctx(|cx| c::timer_smoke::timer_smoke(cx)),
        "PH2D_PATHFOLLOW_SMOKE" => {
            return com_ctx(|cx| {
                c::path_follow_smoke::montar(cx, nivel).expect("nivel existe");
            });
        }
        "PH2D_NAV_SMOKE" => {
            c::nav_smoke::montar(sim.world_mut(), nivel);
        }
        "PH2D_RESTART_SMOKE" => {
            c::restart_smoke::montar(sim.world_mut(), nivel);
        }
        "PH2D_SHAKE_SMOKE" => {
            c::shake_smoke::montar(sim.world_mut(), nivel);
        }
        "PH2D_TOPDOWN_SMOKE" => {
            c::topdown_smoke::montar(sim.world_mut(), nivel);
        }
        "PH2D_PROJECTILE_SMOKE" => {
            c::projectile_smoke::montar(sim.world_mut(), nivel);
        }
        "PH2D_STATEMACHINE_SMOKE" => {
            c::statemachine_smoke::montar(sim.world_mut(), nivel);
        }
        "PH2D_SCRIPT_SMOKE" => {
            let dir = std::env::temp_dir().join("ph2d_censo_desenho_e_corpo");
            std::fs::create_dir_all(&dir).expect("pasta temporaria");
            c::script_smoke::montar(sim.world_mut(), nivel, &dir).expect("monta");
        }
        "PH2D_PARTICLES_SMOKE" => {
            c::particles_smoke::montar(sim.world_mut(), nivel);
        }
        "PH2D_TRIGGER_SMOKE" => {
            c::trigger_smoke::montar(sim.world_mut(), nivel);
        }
        "PH2D_DANO_SMOKE" => {
            c::dano_smoke::montar(sim.world_mut(), &mut tree, nivel);
        }
        "PH2D_RAY_SMOKE" => {
            c::ray_smoke::montar(sim.world_mut(), nivel);
        }
        "PH2D_TWEEN_SMOKE" => {
            c::tween_smoke::montar(sim.world_mut(), nivel);
        }
        "PH2D_PARALLAX_SMOKE" => {
            let _ = c::parallax_smoke::montar(sim.world_mut(), nivel);
        }
        "PH2D_VIDA_SMOKE" => {
            c::vida_smoke::montar(sim.world_mut(), nivel);
        }
        "PH2D_WEAPON_SMOKE" => {
            c::weapon_smoke::montar(sim.world_mut(), nivel);
        }
        other => panic!("router {other} sem construtor no censo — acrescente-o"),
    }
    sim
}

/// O que o desenho ocupa, no referencial local do corpo.
#[derive(Clone, Copy)]
enum Desenho {
    Retangulo { c: [f32; 2], w: f32, h: f32, rot: f32 },
    Disco { c: [f32; 2], r: f32 },
}

fn desenho(s: &Sprite, desloc: [f32; 2], rot: f32, escala: [f32; 2]) -> Desenho {
    let c = [
        desloc[0] + rodar(s.anchor, rot, escala)[0],
        desloc[1] + rodar(s.anchor, rot, escala)[1],
    ];
    let (w, h) = (s.size[0] * escala[0], s.size[1] * escala[1]);
    match s.source {
        SpriteSource::Atlas { key } if key == DISC_TILE_KEY && (w - h).abs() <= TOL => {
            Desenho::Disco { c, r: w / 2.0 }
        }
        _ => Desenho::Retangulo { c, w, h, rot },
    }
}

fn rodar(p: [f32; 2], rot: f32, escala: [f32; 2]) -> [f32; 2] {
    let (x, y) = (p[0] * escala[0], p[1] * escala[1]);
    let (s, c) = rot.sin_cos();
    [x * c - y * s, x * s + y * c]
}

fn cantos(c: [f32; 2], w: f32, h: f32, rot: f32) -> [[f32; 2]; 4] {
    [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)].map(|(sx, sy)| {
        let p = rodar([sx * w / 2.0, sy * h / 2.0], rot, [1.0, 1.0]);
        [c[0] + p[0], c[1] + p[1]]
    })
}

fn dist_segmento(p: [f32; 2], hh: f32) -> f32 {
    let y = p[1].clamp(-hh, hh);
    ((p[0]).powi(2) + (p[1] - y).powi(2)).sqrt()
}

/// Quanto o desenho sai do corpo (≤ 0 = dentro).
fn excesso(d: Desenho, corpo: ColliderShape) -> f32 {
    match (d, corpo) {
        (Desenho::Retangulo { c, w, h, rot }, ColliderShape::Ball { radius }) => cantos(c, w, h, rot)
            .iter()
            .map(|p| p[0].hypot(p[1]) - radius)
            .fold(f32::MIN, f32::max),
        (Desenho::Retangulo { c, w, h, rot }, ColliderShape::Cuboid { half_x, half_y }) => {
            let k = cantos(c, w, h, rot);
            let mx = k.iter().map(|p| p[0].abs()).fold(0.0, f32::max);
            let my = k.iter().map(|p| p[1].abs()).fold(0.0, f32::max);
            (mx - half_x).max(my - half_y)
        }
        (Desenho::Retangulo { c, w, h, rot }, ColliderShape::Capsule { half_height, radius }) => {
            cantos(c, w, h, rot)
                .iter()
                .map(|p| dist_segmento(*p, half_height) - radius)
                .fold(f32::MIN, f32::max)
        }
        (Desenho::Disco { c, r }, ColliderShape::Ball { radius }) => c[0].hypot(c[1]) + r - radius,
        (Desenho::Disco { c, r }, ColliderShape::Cuboid { half_x, half_y }) => {
            (c[0].abs() + r - half_x).max(c[1].abs() + r - half_y)
        }
        (Desenho::Disco { c, r }, ColliderShape::Capsule { half_height, radius }) => {
            dist_segmento(c, half_height) + r - radius
        }
    }
}

fn area_corpo(c: ColliderShape) -> f32 {
    match c {
        ColliderShape::Ball { radius } => std::f32::consts::PI * radius * radius,
        ColliderShape::Cuboid { half_x, half_y } => 4.0 * half_x * half_y,
        ColliderShape::Capsule { half_height, radius } => {
            4.0 * half_height * radius + std::f32::consts::PI * radius * radius
        }
    }
}

fn area_desenho(d: Desenho) -> f32 {
    match d {
        Desenho::Retangulo { w, h, .. } => w * h,
        Desenho::Disco { r, .. } => std::f32::consts::PI * r * r,
    }
}

fn txt_desenho(d: Desenho) -> String {
    match d {
        Desenho::Retangulo { c, w, h, .. } => format!("quadrado {w:.3}x{h:.3} em ({:.3},{:.3})", c[0], c[1]),
        Desenho::Disco { c, r } => format!("disco {:.3}x{:.3} em ({:.3},{:.3})", 2.0 * r, 2.0 * r, c[0], c[1]),
    }
}

fn txt_corpo(c: ColliderShape) -> String {
    match c {
        ColliderShape::Ball { radius } => format!("Ball r={radius:.3}"),
        ColliderShape::Cuboid { half_x, half_y } => format!("Cuboid {:.3}x{:.3}", 2.0 * half_x, 2.0 * half_y),
        ColliderShape::Capsule { half_height, radius } => {
            format!("Capsule hh={half_height:.3} r={radius:.3}")
        }
    }
}

/// O resultado de varrer um mundo.
#[derive(Default)]
struct Varredura {
    foras: Vec<String>,
    folgas: Vec<String>,
    corpos: usize,
}

fn varre(world: &World) -> Varredura {
    let mut v = Varredura::default();
    for er in world.iter_entities() {
        let Some(sprite) = er.get::<Sprite>() else { continue };
        let t = er.get::<Transform>().copied();
        let nome = er
            .get::<Name>()
            .map_or_else(|| format!("{:?}", er.id()), |n| n.0.clone());
        // O corpo e o referencial em que o desenho se confronta com ele.
        let (corpo, d) = if let Some(col) = er.get::<Collider>() {
            (col.shape, desenho(sprite, [0.0, 0.0], 0.0, [1.0, 1.0]))
        } else if let Some(pai) = er.get::<ChildOf>().and_then(|p| world.get::<Collider>(p.parent())) {
            let t = t.unwrap_or_default();
            let esc = [t.scale.x, t.scale.y];
            (pai.shape, desenho(sprite, [t.translation.x, t.translation.y], t.rotation, esc))
        } else {
            continue;
        };
        if let Some(t) = t
            && (t.scale.x - t.scale.y).abs() > TOL
        {
            v.foras.push(format!(
                "{nome} · escala NAO uniforme ({:.3},{:.3})",
                t.scale.x, t.scale.y
            ));
        }
        if let Desenho::Retangulo { w, h, .. } = d
            && matches!(sprite.source, SpriteSource::Atlas { key } if key == DISC_TILE_KEY)
        {
            v.foras.push(format!("{nome} · DISC_TILE_KEY elipse {w:.3}x{h:.3} (bounding rect conferido)"));
        }
        v.corpos += 1;
        let ex = excesso(d, corpo);
        let linha = format!(
            "{nome} · desenho {} · corpo {} · sai {ex:.4} m",
            txt_desenho(d),
            txt_corpo(corpo)
        );
        if ex > TOL {
            v.foras.push(linha);
        } else if area_desenho(d) < 0.5 * area_corpo(corpo) {
            v.folgas.push(linha);
        }
    }
    v
}

fn foras(world: &World) -> Vec<String> {
    varre(world).foras
}

#[test]
fn nenhuma_cena_de_smoke_desenha_fora_do_corpo() {
    let mut ofensores = Vec::new();
    let mut folgas = Vec::new();
    let (mut cenas, mut corpos, mut esperadas) = (0usize, 0usize, 0usize);
    for rt in crate::FAMILY.routers {
        for nivel in 1..=rt.max_level {
            esperadas += 1;
            if let Some((_, _, porque)) = SEM_CORPOS.iter().find(|(e, n, _)| *e == rt.env && *n == nivel) {
                eprintln!("[censo] {}={nivel} saltada de proposito: {porque}", rt.env);
                continue;
            }
            let sim = monta(rt.env, nivel);
            cenas += 1;
            let v = varre(sim.world());
            corpos += v.corpos;
            ofensores.extend(v.foras.iter().map(|l| format!("{}={nivel} · {l}", rt.env)));
            folgas.extend(v.folgas.iter().map(|l| format!("{}={nivel} · {l}", rt.env)));
        }
    }
    eprintln!("[censo] cenas={cenas} corpos={corpos}");
    eprintln!("[censo] corpo muito maior que o desenho ({}):\n{}", folgas.len(), folgas.join("\n"));
    assert_eq!(cenas, esperadas - SEM_CORPOS.len(), "cenas montadas != declaradas");
    // Chão MEDIDO em 05/10: um construtor que monta o vazio reprova aqui.
    assert!(corpos >= PISO_DE_CORPOS, "corpos={corpos} < piso {PISO_DE_CORPOS}");
    assert!(
        ofensores.is_empty(),
        "{} desenhos saem do corpo:\n{}",
        ofensores.len(),
        ofensores.join("\n")
    );
}

/// Medido em 05/10: 42 cenas (com a da arma), 139 corpos (contam os filhos «Rumo»).
const PISO_DE_CORPOS: usize = 139;

/// CONTROLO: a mesma função vê o quadrado sobre a bola e aceita o disco.
#[test]
fn o_censo_ve_um_quadrado_sobre_uma_bola_e_aceita_o_disco() {
    let mut sim = SimWorld::new();
    let w = sim.world_mut();
    let col = |shape| Collider { shape, ..Collider::default() };
    let branco = ph2d_render::WHITE_TILE_KEY;
    w.spawn((
        Name("quadrado-sobre-bola".into()),
        Sprite::atlas(branco, [0.7, 0.7], [1.0; 4]),
        col(ColliderShape::Ball { radius: 0.35 }),
    ));
    w.spawn((
        Name("disco-sobre-bola".into()),
        Sprite::atlas(DISC_TILE_KEY, [0.7, 0.7], [1.0; 4]),
        col(ColliderShape::Ball { radius: 0.35 }),
    ));
    w.spawn((
        Name("quadrado-sobre-caixa".into()),
        Sprite::atlas(branco, [0.45, 0.45], [1.0; 4]),
        col(ColliderShape::Cuboid { half_x: 0.225, half_y: 0.225 }),
    ));
    let pai = w
        .spawn((Name("pai".into()), col(ColliderShape::Ball { radius: 0.35 })))
        .id();
    w.spawn((
        Name("filho-sobressai".into()),
        Sprite::atlas(branco, [0.7, 0.7], [1.0; 4]),
        Transform::default(),
        ChildOf(pai),
    ));
    let f = foras(sim.world());
    assert_eq!(f.len(), 2, "{f:#?}");
    assert!(f.iter().any(|l| l.contains("quadrado-sobre-bola")), "{f:#?}");
    assert!(f.iter().any(|l| l.contains("filho-sobressai")), "{f:#?}");
}

/// ⛔ **A população que o censo varre é a que a SHELL lê.** `PH2D_WEAPON_SMOKE` era lido pela shell
/// e não estava em `FAMILY.routers`: o censo nunca a montou. Aqui a população é a das envs
/// `PH2D_*_SMOKE` passadas a `var`/`var_os` nos dois ficheiros de roteamento da shell.
#[test]
fn todo_roteador_que_a_shell_le_esta_na_familia() {
    // Medido em 05/10: 23 roteadores lidos (22 declarados + o da arma).
    const PISO_DE_ROTEADORES: usize = 23;
    let shell = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../shells/desktop/src");
    let mut lidas = std::collections::BTreeSet::new();
    for ficheiro in ["components_scenes.rs", "components_scenes_suplentes.rs"] {
        let src = std::fs::read_to_string(shell.join(ficheiro)).expect("ficheiro da shell");
        for abre in ["var(\"", "var_os(\""] {
            for (i, _) in src.match_indices(abre) {
                let resto = &src[i + abre.len()..];
                let nome = &resto[..resto.find('"').expect("literal fechado")];
                if nome.starts_with("PH2D_") && nome.ends_with("_SMOKE") {
                    lidas.insert(nome.to_owned());
                }
            }
        }
    }
    assert!(lidas.len() >= PISO_DE_ROTEADORES, "so {} roteadores lidos: {lidas:?}", lidas.len());
    let fora: Vec<_> = lidas
        .iter()
        .filter(|e| !crate::FAMILY.routers.iter().any(|r| r.env == e.as_str()))
        .collect();
    assert!(fora.is_empty(), "a shell le roteadores que FAMILY nao declara: {fora:?}");
}
