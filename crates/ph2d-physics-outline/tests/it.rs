//! O produtor do «Outline» (W2c): argola com buraco, Animated Sprite pelas tags, imagem no CORPO,
//! a escrita pela fila (um passo de undo) e o tecto que sobe o Detail.

use ph2d_core::Vec2;
use ph2d_ecs::scene::{
    ComponentRegistry, EditorCommandQueue, apply_editor_commands, register_ecs_components,
};
use ph2d_ecs::{
    AnimationTag, ChildOf, ExtraSheet, SimWorld, SpriteAnimations, SpriteGrid, SpritePixels,
    SpriteSheets, Transform,
};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderOutline, ColliderShape, PhysicsBridge, RigidBody,
    register_physics_components,
};
use ph2d_physics_outline::{Picture, frames_mask, picture_owner, retrace, trace, traced_frames};
use ph2d_render::Sprite;

/// RGBA `w × h`, opaco onde `on(x, y)`.
fn img(w: u32, h: u32, on: impl Fn(u32, u32) -> bool) -> Vec<u8> {
    (0..h)
        .flat_map(|y| (0..w).map(move |x| (x, y)))
        .flat_map(|(x, y)| [255, 255, 255, if on(x, y) { 255 } else { 0 }])
        .collect()
}

/// Pixels por metro do projeto (o Pivot resolvido da folha `0`).
const PPM: f32 = 8.0;

fn registry() -> ComponentRegistry {
    let mut reg = ComponentRegistry::new();
    register_ecs_components(&mut reg);
    register_physics_components(&mut reg);
    reg
}

/// Argola 32×32 px (buraco 16×16 no meio) num corpo Static de 4×4 m; a forma é a FILHA sem imagem.
fn ring_scene(sim: &mut SimWorld) -> (ph2d_ecs::Entity, ph2d_ecs::Entity) {
    let body = sim
        .world_mut()
        .spawn((
            RigidBody {
                kind: BodyKind::Static,
            },
            Transform::default(),
            Sprite::atlas(0, [4.0, 4.0], [1.0; 4]),
        ))
        .id();
    let shape = sim
        .world_mut()
        .spawn((Collider::default(), Transform::default(), ChildOf(body)))
        .id();
    (body, shape)
}

fn ring_px() -> Vec<u8> {
    img(32, 32, |x, y| {
        !((8..24).contains(&x) && (8..24).contains(&y))
    })
}

/// (a)+(c)+(d) A argola dá anel de fora + buraco; a bola largada DENTRO do buraco cai até ao fundo dele
/// (y = −1 m + raio), não é cuspida pela caixa. A imagem é a do
/// CORPO. A escrita vai pela fila: nada muda antes de a aplicar, e os dois componentes vão juntos.
/// *Mutações: os buracos fora do traço ⇒ a bola é cuspida; `picture_owner` sem subir ao corpo ⇒ `None`.*
#[test]
fn a_argola_tem_buraco_e_a_bola_atravessa() {
    let reg = registry();
    let mut sim = SimWorld::new();
    let (body, shape) = ring_scene(&mut sim);
    assert_eq!(picture_owner(sim.world(), shape), Some(body));
    let px = ring_px();
    let q = EditorCommandQueue::new();
    let pic = |_, _| {
        Some(Picture {
            w: 32,
            h: 32,
            rgba: (&px[..]).into(),
        })
    };
    let t = retrace(sim.world(), shape, pic, PPM, &q, &reg).expect("traça");
    assert_eq!(t.outline.rings.len(), 2, "anel de fora + buraco");
    assert!(!t.raised);
    assert_eq!(q.len(), 2, "Collider + ColliderOutline numa só leva");
    assert!(
        sim.world().get::<ColliderOutline>(shape).is_none(),
        "nada escrito à mão"
    );
    apply_editor_commands(sim.world_mut(), &q, &reg).expect("aplica");
    let c = sim
        .world()
        .get::<Collider>(shape)
        .copied()
        .expect("collider");
    assert_eq!(
        c.shape,
        ColliderShape::Outline {
            half_x: 2.0,
            half_y: 2.0
        }
    );
    assert_eq!(sim.world().get::<ColliderOutline>(shape), Some(&t.outline));
    let ball = sim
        .world_mut()
        .spawn((
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            Collider {
                shape: ColliderShape::Ball { radius: 0.3 },
                density: 1.0,
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(0.0, 0.5)),
        ))
        .id();
    let mut bridge = PhysicsBridge::new();
    for tick in 1..=240 {
        bridge.dispatch(&mut sim, true, tick);
    }
    let y = bridge.body_pose(ball).expect("corpo").1;
    assert!(
        (y + 0.7).abs() < 0.05,
        "a bola parou em y = {y}; no fundo do buraco é ≈ −0,7"
    );
}

/// (b) Animated Sprite: as células das tags (bolha esquerda, bolha direita) dão UM contorno que
/// cobre as duas; a célula do meio, fora das tags, fica de fora — e entra quando não há tags.
/// *Mutação: `traced_frames` ignorar as tags ⇒ o meio entra, vermelho.*
#[test]
fn o_animated_sprite_traca_a_uniao_das_celulas_das_tags() {
    let mut sim = SimWorld::new();
    let px = img(64, 16, |x, y| {
        let (cell, cx) = (x / 16, x % 16);
        (4..12).contains(&y)
            && match cell {
                0 => cx < 6,
                1 => cx >= 10,
                2 => (6..10).contains(&cx),
                _ => false,
            }
    });
    let grid = SpriteGrid {
        hframes: 4,
        vframes: 1,
        frame: 0,
        sheet: 0,
    };
    let mut tags = SpriteAnimations::new();
    tags.insert(AnimationTag::new("idle", 0, 1)).expect("tag");
    let e = sim
        .world_mut()
        .spawn((Sprite::atlas(0, [1.0, 1.0], [1.0; 4]), grid, tags))
        .id();
    let asked = ColliderOutline {
        detail_q4: 4,
        ..ColliderOutline::default()
    };
    let traced_at = |w: &ph2d_ecs::World, frames: &[(u8, u32)]| {
        let pic = |_| {
            Some(Picture {
                w: 64,
                h: 16,
                rgba: (&px[..]).into(),
            })
        };
        let (m, _) = frames_mask(w, e, frames, pic, asked.alpha_cutoff, PPM).expect("máscara");
        let o = ph2d_outline_trace::Outline {
            rings: trace(&m, &asked).outline.rings.to_vec(),
        };
        let at = |x: i32| o.contains_q4([x * 16 + 8, 8 * 16 + 8]);
        (at(2), at(13), at(8))
    };
    let frames = traced_frames(sim.world(), e);
    assert_eq!(frames, vec![(0, 0), (0, 1)]);
    assert_eq!(
        traced_at(sim.world(), &frames),
        (true, true, false),
        "esquerda, direita, sem o meio"
    );
    sim.world_mut().entity_mut(e).remove::<SpriteAnimations>();
    let all = traced_frames(sim.world(), e);
    assert_eq!(all.len(), 4);
    assert_eq!(
        traced_at(sim.world(), &all),
        (true, true, true),
        "sem tags, todas as células"
    );
}

/// (c) A forma com imagem PRÓPRIA traça a dela; acima do corpo não se sobe.
#[test]
fn a_imagem_e_a_da_forma_ou_a_do_antepassado_ate_ao_corpo() {
    let mut sim = SimWorld::new();
    let w = sim.world_mut();
    let top = w.spawn(Sprite::atlas(1, [1.0, 1.0], [1.0; 4])).id();
    let body = w
        .spawn((
            RigidBody {
                kind: BodyKind::Static,
            },
            ChildOf(top),
        ))
        .id();
    let bare = w.spawn((Collider::default(), ChildOf(body))).id();
    let own = w
        .spawn((
            Collider::default(),
            ChildOf(body),
            Sprite::atlas(2, [1.0, 1.0], [1.0; 4]),
        ))
        .id();
    assert_eq!(picture_owner(sim.world(), own), Some(own));
    assert_eq!(
        picture_owner(sim.world(), bare),
        None,
        "o corpo sem imagem pára a subida"
    );
}

/// (e) Acima de `MAX_PIECES` o Detail sobe, e o `raised` vai para o cartão (W4).
/// *Mutação: `raised` sempre `false` ⇒ vermelho.*
#[test]
fn acima_do_tecto_o_detail_sobe_e_diz() {
    let px = img(64, 64, |x, y| (x + y) % 2 == 0);
    let m = ph2d_outline_trace::Mask::from_rgba8(64, 64, &px, 128);
    let asked = ColliderOutline {
        detail_q4: 1,
        ..ColliderOutline::default()
    };
    let t = trace(&m, &asked);
    assert!(t.raised, "o tecto tinha de subir o Detail");
    assert_eq!(t.outline.detail_q4, 1, "o Detail PEDIDO fica");
    assert!(t.outline.raised_q4 > 1 && t.outline.detail_used_q4() == t.outline.raised_q4);
}

/// A entrada da app: `Shape(3)` pedido no CORPO retraça a 1.ª forma pelos pixels do `AssetDb` (átlas);
/// «Fit to picture» numa caixa não traça. *Mutação: `retrace_edited` sem o `Shape(3)` ⇒ 0, vermelho.*
#[test]
fn a_escolha_do_outline_no_corpo_traca_pelo_asset_db() {
    use ph2d_editor_core::PhysicsFieldEdit as P;
    let reg = registry();
    let mut sim = SimWorld::new();
    let (body, shape) = ring_scene(&mut sim);
    let db = ph2d_asset::AssetDb::new();
    let atlas = std::collections::BTreeMap::from([(0, db.insert_image_rgba8(32, 32, ring_px()))]);
    let q = EditorCommandQueue::new();
    let fit = [(body.to_bits(), P::FitToPicture)];
    assert_eq!(
        ph2d_physics_outline::retrace_edited(sim.world(), &fit, &db, &atlas, PPM, &q, &reg),
        0
    );
    let c = Collider {
        shape: ColliderShape::Outline {
            half_x: 1.0,
            half_y: 1.0,
        },
        ..Collider::default()
    };
    sim.world_mut().entity_mut(shape).insert(c);
    let pick = [(body.to_bits(), P::Shape(3))];
    assert_eq!(
        ph2d_physics_outline::retrace_edited(sim.world(), &pick, &db, &atlas, PPM, &q, &reg),
        1
    );
    apply_editor_commands(sim.world_mut(), &q, &reg).expect("aplica");
    let o = sim.world().get::<ColliderOutline>(shape).expect("traçado");
    assert_eq!((o.rings.len(), o.picture_px), (2, [32, 32]));
}

/// (1) **Folha que não divide certo** (10×4 px, 3 colunas): a célula é a do motor (`SourceCells`,
/// canto `floor`, lado `ceil` = 4 px) e a última coluna de pixels (x = 9) entra no contorno.
/// *Mutação: o lado por baixo (3 px) ⇒ a coluna 9 fica de fora.*
#[test]
fn a_folha_que_nao_divide_certo_traca_a_ultima_coluna() {
    let mut sim = SimWorld::new();
    let px = img(10, 4, |x, _| x == 9);
    let grid = SpriteGrid {
        hframes: 3,
        vframes: 1,
        frame: 0,
        sheet: 0,
    };
    let e = sim
        .world_mut()
        .spawn((Sprite::atlas(0, [1.0, 1.0], [1.0; 4]), grid))
        .id();
    let pic = |_| {
        Some(Picture {
            w: 10,
            h: 4,
            rgba: (&px[..]).into(),
        })
    };
    let frames = traced_frames(sim.world(), e);
    let (m, _) = frames_mask(sim.world(), e, &frames, pic, 128, PPM).expect("máscara");
    assert_eq!([m.w, m.h], [4, 4], "a célula do motor: ceil(10/3) × 4");
    assert!((0..4).all(|y| m.get(3, y)), "a coluna 9 da folha");
    let asked = ColliderOutline {
        detail_q4: 4,
        ..ColliderOutline::default()
    };
    let o = ph2d_outline_trace::Outline {
        rings: trace(&m, &asked).outline.rings.to_vec(),
    };
    assert!(
        o.contains_q4([3 * 16 + 8, 2 * 16 + 8]),
        "o contorno cobre-a"
    );
}

/// Argola num corpo cujo contorno nasceu pelo «Outline» do cartão; a 1.ª passagem só anota.
fn followed_ring() -> (
    ComponentRegistry,
    SimWorld,
    ph2d_ecs::Entity,
    ph2d_ecs::Entity,
    ph2d_asset::AssetDb,
) {
    use ph2d_editor_core::PhysicsFieldEdit as P;
    let reg = registry();
    let mut sim = SimWorld::new();
    let (body, shape) = ring_scene(&mut sim);
    let db = ph2d_asset::AssetDb::new();
    let ring = db.insert_image_rgba8(32, 32, ring_px());
    sim.world_mut().entity_mut(body).insert(SpritePixels(ring));
    let c = Collider {
        shape: ColliderShape::Outline {
            half_x: 2.0,
            half_y: 2.0,
        },
        ..Collider::default()
    };
    sim.world_mut().entity_mut(shape).insert(c);
    let q = EditorCommandQueue::new();
    let none = std::collections::BTreeMap::new();
    let pick = [(body.to_bits(), P::Shape(3))];
    ph2d_physics_outline::retrace_edited(sim.world(), &pick, &db, &none, PPM, &q, &reg);
    apply_editor_commands(sim.world_mut(), &q, &reg).expect("aplica");
    assert_eq!(follow(&mut sim, &db, &q, &reg), 0, "a 1.ª vista só anota");
    assert!(q.is_empty());
    (reg, sim, body, shape, db)
}

fn follow(
    sim: &mut SimWorld,
    db: &ph2d_asset::AssetDb,
    q: &EditorCommandQueue,
    reg: &ComponentRegistry,
) -> usize {
    let none = std::collections::BTreeMap::new();
    ph2d_physics_outline::retrace_followed(sim.world_mut(), db, &none, PPM, q, reg)
}

fn rings(sim: &SimWorld, shape: ph2d_ecs::Entity) -> usize {
    sim.world()
        .get::<ColliderOutline>(shape)
        .map_or(0, |o| o.rings.len())
}

/// (2) **O contorno que SEGUE a imagem retraça quando ela muda**: trocar a imagem (argola → disco sem
/// buraco) e editar os pixels (o buraco volta) retraçam na MESMA passagem; o «Undo» (imagem e contorno
/// de antes, juntos) não põe nada na fila — nenhum passo novo. O «Custom size» não retraça.
/// *Mutações: `retrace_followed` sem retraçar ⇒ o disco fica com buraco; sem o filtro do «segue» ⇒ o
/// Custom retraça; sem o guarda «já é o gravado» ⇒ o Undo enche a fila.*
#[test]
fn o_contorno_que_segue_retraca_quando_a_imagem_muda() {
    let (reg, mut sim, body, shape, db) = followed_ring();
    let q = EditorCommandQueue::new();
    assert_eq!(rings(&sim, shape), 2, "argola: fora + buraco");
    let disc = db.insert_image_rgba8(32, 32, img(32, 32, |_, _| true));
    sim.world_mut().entity_mut(body).insert(SpritePixels(disc));
    assert_eq!(follow(&mut sim, &db, &q, &reg), 1, "trocou a imagem");
    apply_editor_commands(sim.world_mut(), &q, &reg).expect("aplica");
    assert_eq!(rings(&sim, shape), 1, "o disco não tem buraco");
    let w = sim.world();
    let disc_state = (
        w.get::<Collider>(shape).copied().expect("c"),
        w.get::<ColliderOutline>(shape).cloned().expect("o"),
    );
    let edited = db.insert_image_rgba8(32, 32, ring_px());
    sim.world_mut()
        .entity_mut(body)
        .insert(SpritePixels(edited));
    assert_eq!(follow(&mut sim, &db, &q, &reg), 1, "pixels editados");
    apply_editor_commands(sim.world_mut(), &q, &reg).expect("aplica");
    assert_eq!(rings(&sim, shape), 2, "o buraco voltou");
    // O Undo da edição: imagem e contorno de antes, num só passo; nada a retraçar de novo.
    sim.world_mut().entity_mut(body).insert(SpritePixels(disc));
    sim.world_mut().entity_mut(shape).insert(disc_state.clone());
    let _ = follow(&mut sim, &db, &q, &reg);
    assert!(
        q.is_empty(),
        "o Undo não enche a fila: o traço já é o gravado"
    );
    // Custom size: a forma deixa de medir a imagem e a troca não lhe toca.
    let custom = Collider {
        shape: ColliderShape::Outline {
            half_x: 1.5,
            half_y: 2.5,
        },
        ..disc_state.0
    };
    sim.world_mut().entity_mut(shape).insert(custom);
    sim.world_mut()
        .entity_mut(body)
        .insert(SpritePixels(edited));
    assert_eq!(follow(&mut sim, &db, &q, &reg), 0, "Custom não segue");
    assert!(q.is_empty());
    assert_eq!(rings(&sim, shape), 1, "o Custom guarda o traço que tinha");
}

/// (3) **Folhas com células de tamanhos diferentes**: folha 0 com 16×16 px num quad de 2×2 m (bolha à
/// esquerda), folha 1 com 8×8 px num quad de 1×1 m posto 2 m à direita (bolha cheia). UM contorno, nos
/// pixels da folha 0 (8 px/m), cobre as duas onde se DESENHAM; a caixa estende-se à arte da folha 1.
/// *Mutações: a folha extra no centro da 0 (sem o Pivot dela) ⇒ a bolha da direita fica de fora; os
/// limites só da folha 0 ⇒ idem.*
#[test]
fn folhas_de_celulas_diferentes_dao_um_contorno_onde_se_desenham() {
    let mut sim = SimWorld::new();
    let px0 = img(16, 16, |x, y| x < 6 && (4..12).contains(&y));
    let px1 = img(8, 8, |_, _| true);
    let grid = SpriteGrid::SINGLE;
    let mut tags = SpriteAnimations::new();
    tags.insert(AnimationTag::new("idle", 0, 0)).expect("tag");
    let mut run = AnimationTag::new("run", 0, 0);
    run.sheet = 1;
    tags.insert(run).expect("tag");
    let sheets = SpriteSheets(
        [ExtraSheet {
            asset: ph2d_asset::AssetId::from_bytes(b"run"),
            size: [1.0, 1.0],
            anchor: [2.0, 0.0],
            hframes: 1,
            vframes: 1,
            offsets: Default::default(),
        }]
        .into_iter()
        .collect(),
    );
    let e = sim
        .world_mut()
        .spawn((Sprite::atlas(0, [2.0, 2.0], [1.0; 4]), grid, tags, sheets))
        .id();
    let frames = traced_frames(sim.world(), e);
    assert_eq!(frames, vec![(0, 0), (1, 0)]);
    let pic = |k: u8| {
        let (w, px) = if k == 0 { (16, &px0) } else { (8, &px1) };
        Some(Picture {
            w,
            h: w,
            rgba: (&px[..]).into(),
        })
    };
    let (m, lay) = frames_mask(sim.world(), e, &frames, pic, 128, PPM).expect("máscara");
    assert_eq!(lay.half, [1.75, 1.0], "de −1 m a 2,5 m × de −1 m a 1 m");
    assert_eq!(lay.centre, [0.75, 0.0]);
    assert_eq!([m.w, m.h], [28, 16], "3,5 m × 2 m a 8 px/m");
    assert!(m.get(2, 8), "a bolha da folha 0");
    assert!(!m.get(10, 8), "o vazio da folha 0");
    assert!(
        (20..28).all(|x| (4..12).all(|y| m.get(x, y))),
        "a folha 1 em 2 m ± 0,5 m"
    );
    assert!(!m.get(18, 8) && !m.get(20, 2), "fora do quad da folha 1");
}

/// (W4 d) **«Follows the picture» das duas folhas**: a caixa do traço é a UNIÃO (1,75 × 1 m), não a
/// folha 0 (1 × 1 m) — `follows` lê o `OutlineSeen`. *Mutação: comparar à folha 0 ⇒ `false`.*
#[test]
fn o_outline_de_duas_folhas_segue_a_uniao() {
    let mut sim = SimWorld::new();
    let px0 = img(16, 16, |x, y| x < 6 && (4..12).contains(&y));
    let mut tags = SpriteAnimations::new();
    tags.insert(AnimationTag::new("idle", 0, 0)).expect("tag");
    let mut run = AnimationTag::new("run", 0, 0);
    run.sheet = 1;
    tags.insert(run).expect("tag");
    let extra = ExtraSheet {
        asset: ph2d_asset::AssetId::from_bytes(b"run"),
        size: [1.0, 1.0],
        anchor: [2.0, 0.0],
        hframes: 1,
        vframes: 1,
        offsets: Default::default(),
    };
    let sheets = SpriteSheets([extra].into_iter().collect());
    let sprite = Sprite::atlas(0, [2.0, 2.0], [1.0; 4]);
    let e = sim
        .world_mut()
        .spawn((sprite, SpriteGrid::SINGLE, tags, sheets))
        .id();
    let frames = traced_frames(sim.world(), e);
    let px1 = img(8, 8, |_, _| true);
    let pic = |k: u8| {
        let (w, px) = if k == 0 { (16, &px0) } else { (8, &px1) };
        Some(Picture {
            w,
            h: w,
            rgba: (&px[..]).into(),
        })
    };
    let (_, layout) = frames_mask(sim.world(), e, &frames, pic, 128, PPM).expect("máscara");
    let [half_x, half_y] = layout.half;
    let c = Collider {
        shape: ColliderShape::Outline { half_x, half_y },
        ..Collider::default()
    };
    let seen = ph2d_physics_outline::OutlineSeen {
        key: 0,
        layout,
        asked: [32, 128, 0],
    };
    sim.world_mut().entity_mut(e).insert((c, seen));
    assert!(
        !c.follows_picture([2.0, 2.0]),
        "a folha 0 sozinha diria que não"
    );
    assert_eq!(ph2d_physics_outline::follows(sim.world(), e), Some(true));
}

/// (W4 b, e) **A curva do Detail retraça**: o canal grava só o PEDIDO (`asked_with`, a escrita de
/// `apply_physics_edit`); a passagem seguinte retraça com MENOS pontos; o Undo (pedido e anéis de
/// antes) não enche a fila. *Mutação: `retrace_followed` sem o «pedido mudou» ⇒ 0, vermelho.*
#[test]
fn a_curva_do_detail_retraca_com_menos_pontos() {
    let (reg, mut sim, body, shape, db) = followed_ring();
    let q = EditorCommandQueue::new();
    // Um pente: dentes de 4 px de fundo, que o Detail de 2 px guarda e o de 6 px fecha.
    let comb = img(32, 32, |x, y| {
        (4..28).contains(&x) && (4..28).contains(&y) && (y >= 8 || (x / 3) % 2 == 0)
    });
    let comb = db.insert_image_rgba8(32, 32, comb);
    sim.world_mut().entity_mut(body).insert(SpritePixels(comb));
    assert_eq!(follow(&mut sim, &db, &q, &reg), 1, "o pente");
    apply_editor_commands(sim.world_mut(), &q, &reg).expect("aplica");
    let pontos = |sim: &SimWorld| {
        let o = sim.world().get::<ColliderOutline>(shape).expect("o");
        o.rings.iter().map(Vec::len).sum::<usize>()
    };
    let antes = sim
        .world()
        .get::<ColliderOutline>(shape)
        .cloned()
        .expect("o");
    let n0 = pontos(&sim);
    let pedido =
        ph2d_physics_outline::asked_with(&antes, ph2d_editor_core::OutlineEdit::Detail(6.0));
    assert_eq!(pedido.rings, antes.rings, "o canal só grava o pedido");
    sim.world_mut().entity_mut(shape).insert(pedido);
    assert_eq!(follow(&mut sim, &db, &q, &reg), 1, "o pedido mudou");
    apply_editor_commands(sim.world_mut(), &q, &reg).expect("aplica");
    let o = sim.world().get::<ColliderOutline>(shape).expect("o");
    assert_eq!(o.detail_q4, 6 * 16);
    assert!(
        pontos(&sim) < n0,
        "Detail maior, menos pontos: {} ≥ {n0}",
        pontos(&sim)
    );
    sim.world_mut().entity_mut(shape).insert(antes);
    let _ = follow(&mut sim, &db, &q, &reg);
    assert!(q.is_empty(), "o Undo já é o traço gravado");
}

/// (W4 1) **O «Custom size» guarda a caixa quando o pedido muda**: o Detail mudado (pela curva, em
/// `retrace_followed`, e pelo cartão, em `retrace_edited`) retraça, mas a meia-extensão e o desvio
/// ficam os do artista — os anéis esticam-se a ela. *Mutação: `keep` sempre `false` ⇒ a caixa volta à
/// da imagem (2 × 2 m), vermelho.*
#[test]
fn o_custom_size_retraca_e_guarda_a_caixa() {
    use ph2d_editor_core::{OutlineEdit, PhysicsFieldEdit as P};
    let (reg, mut sim, body, shape, db) = followed_ring();
    let q = EditorCommandQueue::new();
    let mut custom = sim.world().get::<Collider>(shape).copied().expect("c");
    custom.shape = ColliderShape::Outline {
        half_x: 1.5,
        half_y: 2.5,
    };
    custom.offset = [0.25, -0.5];
    sim.world_mut().entity_mut(shape).insert(custom);
    let antes = sim
        .world()
        .get::<ColliderOutline>(shape)
        .cloned()
        .expect("o");
    let pedido = ph2d_physics_outline::asked_with(&antes, OutlineEdit::Detail(6.0));
    sim.world_mut().entity_mut(shape).insert(pedido);
    assert_eq!(follow(&mut sim, &db, &q, &reg), 1, "o pedido mudou");
    apply_editor_commands(sim.world_mut(), &q, &reg).expect("aplica");
    let w = sim.world();
    assert_eq!(w.get::<ColliderOutline>(shape).expect("o").detail_q4, 96);
    assert_eq!(w.get::<Collider>(shape).copied(), Some(custom), "a curva");
    let none = std::collections::BTreeMap::new();
    let edit = [(body.to_bits(), P::Outline(OutlineEdit::Detail(2.0)))];
    let n = ph2d_physics_outline::retrace_edited(sim.world(), &edit, &db, &none, PPM, &q, &reg);
    assert_eq!(n, 1, "o cartão retraça");
    apply_editor_commands(sim.world_mut(), &q, &reg).expect("aplica");
    let w = sim.world();
    assert_eq!(w.get::<ColliderOutline>(shape).expect("o").detail_q4, 32);
    assert_eq!(w.get::<Collider>(shape).copied(), Some(custom), "o cartão");
}
