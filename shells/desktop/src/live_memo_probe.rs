//! SONDA (`--ignored`): **quanto custa mover uma forma que tem geometria VIVA?**
//!
//! # A pergunta
//!
//! Todo produtor de `LiveGeometry` memoiza, e a §11 do plano 25 afirma que **a chave é o
//! MUNDO — que é exatamente o que a animação move**. Se a afirmação valer, então arrastar (ou
//! animar) uma forma com Contour/Offset/Perfil/Simetria re-cozinha o efeito **em todo quadro**,
//! e o memo protege só a cena PARADA.
//!
//! ⚠️ **A sonda mede pela porta do PRODUTO** (`recook`, o que o frame chama), nunca por um laço
//! próprio sobre o kernel: esta casa já pagou três vezes por medir uma peça isolada e chamar o
//! número de produto (doc 28 §5.40 do Painter, e a decomposição do `build_flow_field`).
//!
//! # As duas colunas, e por que a comparação é honesta
//!
//! - **PARADO** — `recook` N vezes com a MESMA pose. É o que o memo promete proteger.
//! - **ANIMADO** — `recook` N vezes com a pose a TRANSLADAR. É o que a timeline faz a 60 Hz.
//!
//! Uma translação **não muda a forma do efeito**: o contorno de uma estrela deslocada é o
//! contorno da estrela, deslocado. Então toda diferença entre as duas colunas é trabalho que a
//! resposta não precisou.
//!
//! Rode: `cargo test -p ph2d-host-desktop --release live_memo -- --ignored --nocapture`

use crate::vec_entities::VecEntityMap;
use ph2d_ecs::{Entity, Name, SimWorld, Transform, VecContour, VecOffset, VecPathRef};
use ph2d_vec_scene::{VecPath, VecPathId, VecScene, VecVertex, VecXforms};

/// Quantos quadros cada coluna mede. Sessenta é um segundo de animação a 60 Hz.
const FRAMES: usize = 60;

/// Uma estrela de 5 pontas — a fixture CARA do `probe_contour_cost` (quinas reentrantes), e a
/// única em que o offset tem trabalho de verdade a fazer.
fn star() -> VecPath {
    let mut pts = Vec::new();
    for i in 0..10 {
        let a = std::f64::consts::PI * 2.0 * f64::from(i) / 10.0 - std::f64::consts::FRAC_PI_2;
        let r = if i % 2 == 0 { 1.0 } else { 0.42 };
        pts.push([a.cos() * r, a.sin() * r]);
    }
    VecPath {
        verts: pts.into_iter().map(VecVertex::corner).collect(),
        closed: true,
        ..VecPath::default()
    }
}

/// A cena mínima: uma estrela posada, com entidade e nome.
fn posed_star() -> (VecScene, SimWorld, VecEntityMap, VecPathId, Entity) {
    let mut sim = SimWorld::default();
    let mut map = VecEntityMap::new();
    let mut scene = VecScene::new();
    let id = scene.push_path(star());
    let e = sim
        .world_mut()
        .spawn((Transform::IDENTITY, Name::new("Star"), VecPathRef(id)))
        .id();
    map.insert(id, e.to_bits());
    (scene, sim, map, id, e)
}

/// Empurra a pose de `e` para `x` e devolve os `VecXforms` que o frame publicaria.
fn pose_at(sim: &mut SimWorld, map: &VecEntityMap, e: Entity, x: f32) -> VecXforms {
    if let Ok(mut em) = sim.world_mut().get_entity_mut(e)
        && let Some(mut t) = em.get_mut::<Transform>()
    {
        t.translation = ph2d_core::Vec2::new(x, 0.0);
    }
    crate::vec_transform::build(sim, map)
}

/// Roda `frames` quadros e devolve o custo MEDIANO por quadro, em ms.
///
/// ⚠️ **Mediana, não mínimo:** o primeiro quadro de cada coluna é sempre um miss (o memo nasce
/// vazio), e o mínimo seria exactamente a amostra SEM o fenômeno — a lição de fixture que o
/// Painter pagou no gate de razão do Wet Paint (doc 28 §5.12).
fn median_ms(frames: usize, mut f: impl FnMut(usize)) -> f64 {
    let mut ms = Vec::with_capacity(frames);
    for i in 0..frames {
        let t0 = std::time::Instant::now();
        f(i);
        ms.push(t0.elapsed().as_secs_f64() * 1e3);
    }
    ms.sort_by(f64::total_cmp);
    ms[ms.len() / 2]
}

/// A translação do quadro `i` — 0,01 unidade por quadro, o que um arrasto lento faz.
#[allow(clippy::cast_precision_loss)]
fn drift(i: usize) -> f32 {
    i as f32 * 0.01
}

#[test]
#[ignore = "sonda: rode com --release -- --ignored"]
fn live_memo_probe() {
    println!("[sonda] {FRAMES} quadros por coluna; mediana em ms/quadro");
    println!(
        "[sonda] {:<12} {:>10} {:>10} {:>8} {:>6}",
        "produtor", "PARADO", "ANIMADO", "razao", "saida"
    );

    probe_contour();
    probe_offset();
    probe_profile();
    probe_symmetry();
}

/// ⚠️ **O CONTROLE da sonda.** Um produtor que não produziu nada mede o custo de um `continue`,
/// e a coluna ANIMADO sairia tranquilizadoramente baixa — a mesma armadilha de fixture que o
/// Painter pagou no `build_flow_field` (a máscara `active` vazia fazia todo passe sair pela
/// porta de trás, e a soma casava com o passo *por coincidência*).
fn require(name: &str, produced: usize) -> usize {
    assert!(
        produced > 0,
        "a fixture de {name} nao produziu geometria viva — a sonda estaria a medir um `continue`"
    );
    produced
}

/// ⚠️ **O número sai com o ESCOPO ao lado.** Um custo sem a contagem que ele produziu é
/// inatribuível: `0,04 ms` sobre dezasseis anéis e `0,04 ms` sobre um caminho degenerado são
/// leituras opostas, e só a coluna `saída` as separa.
fn report(name: &str, still: f64, moving: f64, produced: usize) {
    let ratio = if still > 0.0 {
        moving / still
    } else {
        f64::INFINITY
    };
    let flag = if moving > 16.6 {
        "  <= NAO CABE NUM QUADRO DE 60 Hz"
    } else {
        ""
    };
    println!("[sonda] {name:<12} {still:>10.3} {moving:>10.3} {ratio:>7.1}x {produced:>6}{flag}");
}

fn probe_contour() {
    let (scene, mut sim, map, id, e) = posed_star();
    let ent = Entity::from_bits(map[&id]);
    let _ = ent;
    sim.world_mut().entity_mut(e).insert(VecContour {
        steps: 16,
        d: 0.30,
        join: 1, // Round: a quina cara
        to: [255, 255, 255, 255],
        ..VecContour::default()
    });

    let mut live = crate::contour_live::ContourLive::default();
    let xf = pose_at(&mut sim, &map, e, 0.0);
    let still = median_ms(FRAMES, |_| live.recook(&scene, &sim, &map, &xf));

    let mut live = crate::contour_live::ContourLive::default();
    let mut xfs = Vec::with_capacity(FRAMES);
    for i in 0..FRAMES {
        xfs.push(pose_at(&mut sim, &map, e, drift(i)));
    }
    let moving = median_ms(FRAMES, |i| live.recook(&scene, &sim, &map, &xfs[i]));
    let produced = require("contour", live.live().values().map(Vec::len).sum());
    report("contour", still, moving, produced);
}

fn probe_offset() {
    let (scene, mut sim, map, id, e) = posed_star();
    let _ = id;
    sim.world_mut().entity_mut(e).insert(VecOffset {
        d: 0.12,
        join: 1,
        side: 0,
    });

    let mut live = crate::offset_live::OffsetLive::default();
    let xf = pose_at(&mut sim, &map, e, 0.0);
    let still = median_ms(FRAMES, |_| live.recook(&scene, &sim, &map, &xf));

    let mut live = crate::offset_live::OffsetLive::default();
    let mut xfs = Vec::with_capacity(FRAMES);
    for i in 0..FRAMES {
        xfs.push(pose_at(&mut sim, &map, e, drift(i)));
    }
    let moving = median_ms(FRAMES, |i| live.recook(&scene, &sim, &map, &xfs[i]));
    let produced = require("offset", live.live().values().map(Vec::len).sum());
    report("offset", still, moving, produced);
}

fn probe_profile() {
    let (mut scene, mut sim, map, id, e) = posed_star();
    // O perfil só produz fita se a forma TIVER traço — a fixture tem de conter o fenômeno.
    if let Some(p) = scene.paths_mut().iter_mut().find(|p| p.id == id) {
        p.stroke = Some(ph2d_vec_scene::StrokeSpec::new(
            ph2d_vec_scene::Rgba8::new(0, 0, 0, 255),
            0.08,
        ));
    }
    // ⚠️ `VecStrokeProfile::default()` tem `stops` VAZIO — o neutro que a shell REMOVE em vez de
    // guardar. Armá-lo é armar a ausência do efeito; o perfil precisa de paradas de verdade.
    sim.world_mut()
        .entity_mut(e)
        .insert(ph2d_ecs::VecStrokeProfile {
            stops: ph2d_vec_scene::WidthProfile {
                start: 0.2,
                mid: 1.8,
                end: 0.2,
                position: 0.5,
            }
            .to_stops(),
        });

    let mut live = crate::profile_live::ProfileLive::default();
    let xf = pose_at(&mut sim, &map, e, 0.0);
    let still = median_ms(FRAMES, |_| live.recook(&scene, &sim, &map, &xf));

    let mut live = crate::profile_live::ProfileLive::default();
    let mut xfs = Vec::with_capacity(FRAMES);
    for i in 0..FRAMES {
        xfs.push(pose_at(&mut sim, &map, e, drift(i)));
    }
    let moving = median_ms(FRAMES, |i| live.recook(&scene, &sim, &map, &xfs[i]));
    let produced = require("profile", live.live().values().map(Vec::len).sum());
    report("profile", still, moving, produced);
}

fn probe_symmetry() {
    let (scene, mut sim, map, id, e) = posed_star();
    let _ = id;
    sim.world_mut()
        .entity_mut(e)
        .insert(ph2d_ecs::VecSymmetry::default());

    let mut live = crate::symmetry_live::SymmetryLive::default();
    let xf = pose_at(&mut sim, &map, e, 0.0);
    let still = median_ms(FRAMES, |_| live.recook(&scene, &sim, &map, &xf, true));

    let mut live = crate::symmetry_live::SymmetryLive::default();
    let mut xfs = Vec::with_capacity(FRAMES);
    for i in 0..FRAMES {
        xfs.push(pose_at(&mut sim, &map, e, drift(i)));
    }
    let moving = median_ms(FRAMES, |i| live.recook(&scene, &sim, &map, &xfs[i], true));
    let produced = require("symmetry", live.live().values().map(Vec::len).sum());
    report("symmetry", still, moving, produced);
}
