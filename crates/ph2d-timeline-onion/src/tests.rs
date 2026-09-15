//! O onion da timeline é **puro** (ADR-0142): dado o mundo, o doc, os alvos e o instante
//! vivo, os fantasmas são função disso — respondível headless. Estes gates pinam *quantos*
//! fantasmas, *onde* (a pose de `t±k`, não a viva), *de que cor* (frio atrás, quente à
//! frente), *quão fortes* (falloff) e *que forma* (silhueta plana).

use super::{GHOST_MIN_ALPHA, OnionMode, OnionSettings, build_ghosts};
use ph2d_anim::{AnimValue, Interp, RationalTime};
use ph2d_core::Vec2;
use ph2d_ecs::{SimWorld, Transform};
use ph2d_render::RenderInstance;
use ph2d_timeline::{PropKind, TimelineDoc, pose_at};

/// A régua do projecto nestes gates. ⚠️ Nenhum alvo aqui tem PELE, logo ela não chega a ser lida —
/// ela existe para a chamada ser a do produto.
const PPM: f32 = 100.0;

/// Um `RenderInstance` neutro (branco, basis identidade) — o "vivo" que os fantasmas
/// herdam os campos de sprite (aqui só a forma importa).
fn template() -> RenderInstance {
    let mut t: RenderInstance = bytemuck::Zeroable::zeroed();
    t.tint = [1.0, 1.0, 1.0, 1.0];
    t.per_corner_tint = [[1.0, 1.0, 1.0, 1.0]; 4];
    t.size = [1.0, 1.0];
    t.basis = RenderInstance::IDENTITY_BASIS;
    t.opacity = 1.0;
    t.texture_id = 1;
    t
}

/// Sim + doc: um objeto cujo X vai de 0 a 10 em 0..4 s (linear ⇒ `x = 2,5·t`).
fn rig() -> (SimWorld, u64, TimelineDoc) {
    let mut w = SimWorld::new();
    let e = w
        .world_mut()
        .spawn(Transform::from_translation(Vec2::ZERO))
        .id();
    let b = e.to_bits();
    let mut doc = TimelineDoc::new();
    doc.insert_key(
        b,
        PropKind::TranslationX,
        RationalTime::from_seconds(0.0),
        AnimValue::Float(0.0),
        Interp::Linear,
    );
    doc.insert_key(
        b,
        PropKind::TranslationX,
        RationalTime::from_seconds(4.0),
        AnimValue::Float(10.0),
        Interp::Linear,
    );
    (w, b, doc)
}

/// Modo FRAMES, `fps = 4` ⇒ `dt = 0,25 s` ⇒ passos de X de `0,625` bem separados. (O
/// default é `Keys`; estes gates fixam `Frames` de propósito — é o que eles testam.)
/// O alvo de sempre: a entidade desenha-se E tem as keys — o caso de toda cena sem rig.
fn alvo(e: u64) -> super::GhostTarget {
    super::GhostTarget {
        entity: e,
        template: template(),
        relogios: vec![e],
    }
}

fn settings() -> OnionSettings {
    OnionSettings {
        enabled: true,
        frames_before: 2,
        frames_after: 2,
        fps: 4.0,
        mode: OnionMode::Frames,
        ..OnionSettings::default()
    }
}

/// Um rig com keys em 0,1,2,3,4 s (`x = 2,5·t`) — o modo Keys ghosta as VIZINHAS.
fn rig_keys() -> (SimWorld, u64, TimelineDoc) {
    let mut w = SimWorld::new();
    let e = w
        .world_mut()
        .spawn(Transform::from_translation(Vec2::ZERO))
        .id();
    let b = e.to_bits();
    let mut doc = TimelineDoc::new();
    for t in [0.0, 1.0, 2.0, 3.0, 4.0] {
        doc.insert_key(
            b,
            PropKind::TranslationX,
            RationalTime::from_seconds(t),
            AnimValue::Float((2.5 * t) as f32),
            Interp::Linear,
        );
    }
    (w, b, doc)
}

#[test]
fn the_onion_ghosts_the_frames_before_and_after() {
    let (w, e, doc) = rig();
    let s = settings();
    let mut out = ph2d_render::LiftedInstances::default();
    build_ghosts(&s, &w, &doc, &[alvo(e)], 2.0, PPM, &mut out);
    assert_eq!(
        out.len(),
        (s.frames_before + s.frames_after) as usize,
        "um fantasma por quadro antes e depois"
    );
}

#[test]
fn past_ghosts_are_cool_and_future_ghosts_are_warm() {
    let (w, e, doc) = rig();
    let s = settings();
    let live_x = pose_at(w.world(), &doc, e, 2.0).unwrap().translation.x; // 5.0
    let mut out = ph2d_render::LiftedInstances::default();
    build_ghosts(&s, &w, &doc, &[alvo(e)], 2.0, PPM, &mut out);
    for g in out.instances() {
        let cool = g.world_pos[0] < live_x; // passado = X menor (objeto ia para a direita)
        let want = if cool { s.color_before } else { s.color_after };
        assert_eq!(
            [g.tint[0], g.tint[1], g.tint[2]],
            want,
            "fantasma em x={} usou a cor errada",
            g.world_pos[0]
        );
    }
    // E os dois lados de fato existem (senão "todos frios" passaria vazio).
    assert!(
        out.instances().iter().any(|g| g.world_pos[0] < live_x),
        "sem fantasma passado"
    );
    assert!(
        out.instances().iter().any(|g| g.world_pos[0] > live_x),
        "sem fantasma futuro"
    );
}

#[test]
fn a_farther_ghost_is_fainter() {
    let (w, e, doc) = rig();
    let s = settings();
    let live_x = pose_at(w.world(), &doc, e, 2.0).unwrap().translation.x;
    let mut out = ph2d_render::LiftedInstances::default();
    build_ghosts(&s, &w, &doc, &[alvo(e)], 2.0, PPM, &mut out);
    // Do lado futuro: o mais próximo (menor |x-live|) é mais forte que o mais distante.
    let mut future: Vec<_> = out
        .instances()
        .iter()
        .filter(|g| g.world_pos[0] > live_x)
        .collect();
    future.sort_by(|a, b| a.world_pos[0].partial_cmp(&b.world_pos[0]).unwrap());
    assert!(future.len() >= 2);
    assert!(
        future[0].tint[3] > future[1].tint[3],
        "o fantasma mais próximo ({}) não é mais forte que o distante ({})",
        future[0].tint[3],
        future[1].tint[3]
    );
    for g in out.instances() {
        assert!(g.tint[3] >= GHOST_MIN_ALPHA, "abaixo do piso de opacidade");
    }
}

#[test]
fn a_ghost_stands_where_the_object_would_be_not_where_it_is() {
    // A prova de que a pose vem de `pose_at(t±k)`, não do instante vivo: NENHUM fantasma
    // fica onde o objeto está agora. Mutação (amostrar em `live_clip_t`) faz todos
    // caírem em x=live ⇒ RED.
    let (w, e, doc) = rig();
    let s = settings();
    let live_x = pose_at(w.world(), &doc, e, 2.0).unwrap().translation.x;
    let mut out = ph2d_render::LiftedInstances::default();
    build_ghosts(&s, &w, &doc, &[alvo(e)], 2.0, PPM, &mut out);
    for g in out.instances() {
        assert!(
            (g.world_pos[0] - live_x).abs() > 1e-3,
            "um fantasma ({}) caiu sobre a pose viva ({live_x}) — a pose não veio de t±k",
            g.world_pos[0]
        );
    }
}

#[test]
fn a_ghost_is_a_flat_silhouette() {
    let (w, e, doc) = rig();
    let s = settings();
    let mut out = ph2d_render::LiftedInstances::default();
    build_ghosts(&s, &w, &doc, &[alvo(e)], 2.0, PPM, &mut out);
    assert!(!out.is_empty());
    for g in out.instances() {
        assert!(
            g.flip_uv & RenderInstance::TINT_FILL_BIT != 0,
            "o fantasma não está em modo silhueta (tint_fill)"
        );
        assert_eq!(
            g.per_corner_tint,
            [[1.0, 1.0, 1.0, 1.0]; 4],
            "per-corner não-neutro tingiria a silhueta"
        );
        assert_eq!(
            g.opacity, 1.0,
            "a opacity herdada apagaria o falloff da tinta"
        );
    }
}

#[test]
fn the_onion_is_off_by_default_and_when_disabled() {
    assert!(!OnionSettings::default().enabled, "o onion nasce desligado");
    let (w, e, doc) = rig();
    let s = OnionSettings {
        enabled: false,
        ..settings()
    };
    let mut out = ph2d_render::LiftedInstances::default();
    build_ghosts(&s, &w, &doc, &[alvo(e)], 2.0, PPM, &mut out);
    assert!(out.is_empty(), "desligado não desenha fantasma");
}

#[test]
fn no_targets_no_ghosts() {
    let (w, _e, doc) = rig();
    let mut out = ph2d_render::LiftedInstances::default();
    build_ghosts(&settings(), &w, &doc, &[], 2.0, PPM, &mut out);
    assert!(out.is_empty());
}

#[test]
fn keys_mode_ghosts_the_neighboring_keyframes() {
    // Keys em 0,1,2,3,4; playhead em 2 (sobre um key); antes=2/depois=2 ⇒ fantasmas nas
    // keys 1 e 0 (passado) e 3 e 4 (futuro) — as poses AUTORADAS vizinhas.
    let (w, e, doc) = rig_keys();
    let s = OnionSettings {
        enabled: true,
        frames_before: 2,
        frames_after: 2,
        mode: OnionMode::Keys,
        ..OnionSettings::default()
    };
    let mut out = ph2d_render::LiftedInstances::default();
    build_ghosts(&s, &w, &doc, &[alvo(e)], 2.0, PPM, &mut out);
    assert_eq!(out.len(), 4, "duas keys de cada lado do playhead");
    // As posições X dos fantasmas são as poses NAS keys (2,5·t): 2.5 e 0.0 (passado),
    // 7.5 e 10.0 (futuro). O live (5.0) NÃO está entre elas.
    let mut xs: Vec<f32> = out.instances().iter().map(|g| g.world_pos[0]).collect();
    xs.sort_by(|a, b| a.partial_cmp(b).unwrap());
    assert_eq!(
        xs,
        vec![0.0, 2.5, 7.5, 10.0],
        "os fantasmas caem NAS keyframes vizinhas"
    );
    assert!(
        !xs.contains(&5.0),
        "a pose viva (uma key) não vira fantasma"
    );
}

#[test]
fn keys_mode_ignores_the_frame_grid() {
    // O CONTROLE do modo Keys: os fantasmas caem nas KEYS, então mudar o fps (a grade de
    // quadros) não muda onde eles estão. Mutação (Keys usar `live ± k·dt`) ⇒ RED.
    let (w, e, doc) = rig_keys();
    let base = OnionSettings {
        enabled: true,
        frames_before: 2,
        frames_after: 2,
        mode: OnionMode::Keys,
        ..OnionSettings::default()
    };
    let x_of = |fps: f64| -> Vec<f32> {
        let s = OnionSettings { fps, ..base };
        let mut out = ph2d_render::LiftedInstances::default();
        build_ghosts(&s, &w, &doc, &[alvo(e)], 2.0, PPM, &mut out);
        let mut xs: Vec<f32> = out.instances().iter().map(|g| g.world_pos[0]).collect();
        xs.sort_by(|a, b| a.partial_cmp(b).unwrap());
        xs
    };
    assert_eq!(
        x_of(4.0),
        x_of(60.0),
        "o fps não pode mover um fantasma de modo Keys"
    );
    assert_eq!(
        x_of(4.0),
        vec![0.0, 2.5, 7.5, 10.0],
        "e eles estão NAS keys"
    );
}

/// Um rig mínimo: **um osso** e uma **imagem presa a ele** pela porta do produto, com o espelho da
/// imagem no presente — a fixtura do escopo E da malha do fantasma.
fn rig_com_pele() -> (SimWorld, ph2d_ecs::PresentWorld, u64, u64) {
    rig_com_pele_de([40, 20])
}

/// [`rig_com_pele`] com a imagem do tamanho pedido — a sonda de relógio quer uma malha do tamanho
/// que o produto desenha, e não a mínima que um gate de escopo precisa.
fn rig_com_pele_de(px: [u32; 2]) -> (SimWorld, ph2d_ecs::PresentWorld, u64, u64) {
    let mut sim = SimWorld::new();
    // ⚠️ Uma CORRENTE de três, e não um osso solto: a pose de um fantasma é `world_pose_at` de cada
    // ELO, e uma fixtura de um osso só esconderia esse preço da sonda de relógio lá em baixo.
    let mut pai: Option<ph2d_ecs::Entity> = None;
    let mut raiz: Option<ph2d_ecs::Entity> = None;
    for i in 0..3 {
        let x = f64::from(i) * 1.5 - 2.0;
        let bits = ph2d_skeleton_live::bone::create(&mut sim, pai, [x, 0.0], [x + 1.5, 0.0])
            .expect("osso");
        pai = Some(ph2d_ecs::Entity::from_bits(bits));
        raiz = raiz.or(pai);
    }
    let osso = raiz.expect("a corrente tem raiz");
    let arte = sim
        .world_mut()
        .spawn((
            Transform::from_translation(Vec2::ZERO),
            ph2d_render::Sprite::atlas(0, [4.0, 2.0], [1.0; 4]),
        ))
        .id();
    // ⚠️ **Prende pela PORTA do produto** (`bind_image`), e não com um `SkinBind` escrito à mão: é
    // ela que guarda a malha e as matrizes de repouso, e um bind fabricado mediria outra coisa.
    assert!(
        ph2d_skeleton_live::skin_live::bind_image(
            &mut sim,
            arte,
            &vec![255u8; (px[0] * px[1] * 4) as usize],
            px,
            PPM,
            ph2d_poly2d::GridOptions::default(),
            Some(osso),
        ),
        "a fixtura nao prendeu a imagem ao osso"
    );
    let mut present = ph2d_ecs::PresentWorld::new();
    present.world_mut().spawn((
        ph2d_ecs::SimRef(arte),
        ph2d_ecs::GlobalTransform::from_transform(Transform::from_translation(Vec2::ZERO)),
        template(),
    ));
    (sim, present, osso.to_bits(), arte.to_bits())
}

/// ⭐⭐⭐ **Um OSSO seleccionado ghosta a ARTE que ele deforma.**
///
/// ⛔⛔ **Até 2026-09-13 ele não ghostava NADA, e por duas guardas que se excluíam:** a imagem não
/// está animada (quem leva keys são os ossos) e o osso não tem instância de desenho. *Um recurso
/// cujas duas guardas se excluem uma à outra está desligado, não configurado.*
#[test]
fn a_selected_bone_ghosts_the_art_it_deforms() {
    let (sim, mut present, osso, arte) = rig_com_pele();
    let mut doc = TimelineDoc::new();
    for (t, v) in [(0.0, 0.0f32), (4.0, 1.0)] {
        doc.insert_key(
            osso,
            PropKind::Rotation,
            RationalTime::from_seconds(t),
            AnimValue::Float(v),
            Interp::Linear,
        );
    }
    let alvos = super::ghost_targets(&sim, &mut present, &doc, osso);
    assert_eq!(
        alvos.iter().map(|a| a.entity).collect::<Vec<_>>(),
        vec![arte],
        "o osso seleccionado tinha de ghostar a imagem presa ao esqueleto dele"
    );
}

/// ⛔ **Os dois CONTROLOS:** sem nada animado no esqueleto não há passado a mostrar, e um osso de
/// outro esqueleto não empresta a arte deste.
#[test]
fn a_bone_with_nothing_animated_ghosts_nothing() {
    let (mut sim, mut present, osso, _arte) = rig_com_pele();
    let vazio = TimelineDoc::new();
    assert!(
        super::ghost_targets(&sim, &mut present, &vazio, osso).is_empty(),
        "um esqueleto parado nao tem passado nem futuro a mostrar"
    );
    // Um SEGUNDO esqueleto, animado, não faz a arte do primeiro aparecer.
    let outro = sim
        .world_mut()
        .spawn((
            Transform::from_translation(Vec2::new(50.0, 0.0)),
            ph2d_skeleton_ecs::Bone {
                length: 1.0,
                strength: 1.0,
                ..Default::default()
            },
        ))
        .id();
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    let mut doc = TimelineDoc::new();
    for (t, v) in [(0.0, 0.0f32), (4.0, 1.0)] {
        doc.insert_key(
            outro.to_bits(),
            PropKind::Rotation,
            RationalTime::from_seconds(t),
            AnimValue::Float(v),
            Interp::Linear,
        );
    }
    assert!(
        super::ghost_targets(&sim, &mut present, &doc, outro.to_bits()).is_empty(),
        "o osso de OUTRO esqueleto ghostou arte que nao e' dele"
    );
}

/// ⭐⭐⭐ **O fantasma de uma imagem presa leva a MALHA, e ela é a arte dobrada NAQUELE instante.**
///
/// ⛔⛔ **O defeito que este gate não deixa voltar** (nomeado na W3 e curado na W7): o fantasma
/// herdava os campos de sprite do vivo e desenhava o **quad de repouso** — a cena a ensinar que a
/// arte não dobra, exactamente onde o animador olha para ver como ela dobrou.
///
/// ⚠️ **A régua é a DIFERENÇA entre dois fantasmas**, e não *«tem malha»*: uma malha calculada uma
/// vez e repetida em todos os instantes passaria numa asserção de presença e seria o mesmo defeito
/// com outra cara.
#[test]
fn the_ghost_of_a_bound_image_carries_the_mesh_bent_at_that_instant() {
    let (sim, mut present, osso, _arte) = rig_com_pele();
    let mut doc = TimelineDoc::new();
    for (t, v) in [(0.0, 0.0f32), (4.0, 1.2)] {
        doc.insert_key(
            osso,
            PropKind::Rotation,
            RationalTime::from_seconds(t),
            AnimValue::Float(v),
            Interp::Linear,
        );
    }
    let mut out = ph2d_render::LiftedInstances::default();
    super::collect_onion_ghosts(
        &settings(),
        &sim,
        &mut present,
        &doc,
        Some(osso),
        Some(2.0),
        PPM,
        &mut out,
    );
    assert_eq!(out.len(), 4, "dois fantasmas de cada lado do playhead");
    let malhas: Vec<&ph2d_render::SpriteMesh> = (0..out.len())
        .map(|i| {
            out.mesh_of(i)
                .expect("cada fantasma de uma imagem presa leva a malha dele")
        })
        .collect();
    let desvio = malhas[0]
        .local
        .iter()
        .zip(&malhas[3].local)
        .map(|(p, q)| f64::from(p[0] - q[0]).hypot(f64::from(p[1] - q[1])))
        .fold(0.0f64, f64::max);
    println!("fantasma mais antigo vs mais futuro: maior desvio {desvio}");
    assert!(
        desvio > 1e-3,
        "os fantasmas de instantes diferentes trazem a MESMA malha ({desvio}) — a pele nao esta' a \
         ser resolvida em `t`"
    );
}

/// ⏱️ **O QUE O ONION DE UM RIG CUSTA POR QUADRO** — a sonda, não um gate.
///
/// ⚠️ **Mede o MÍNIMO de N corridas com a mediana ao lado** (§5.0 + a lição da W4): a carga de FUNDO
/// desta workstation é `~7` sem ninguém compilar, e uma média sob contenção mede o vizinho. A
/// distância entre o mínimo e a mediana diz quanto a máquina estava a roubar.
///
/// `cargo nextest run -p ph2d-host-desktop -E 'test(measure_the_cost_of_a_rigged_onion)' \
///   --run-ignored only --no-capture`
#[test]
#[ignore = "sonda de relógio: corre à mão, com a máquina calma"]
fn measure_the_cost_of_a_rigged_onion() {
    let (sim, mut present, osso, arte) = rig_com_pele_de([480, 240]);
    let mut doc = TimelineDoc::new();
    for (t, v) in [(0.0, 0.0f32), (4.0, 1.2)] {
        doc.insert_key(
            osso,
            PropKind::Rotation,
            RationalTime::from_seconds(t),
            AnimValue::Float(v),
            Interp::Linear,
        );
    }
    let pecas = ph2d_skeleton_live::skin_image::mesh_of(&sim, ph2d_ecs::Entity::from_bits(arte))
        .map_or(0, |m| m.tris.len());
    let mut ms: Vec<f64> = Vec::new();
    for _ in 0..40 {
        let mut out = ph2d_render::LiftedInstances::default();
        let t0 = std::time::Instant::now();
        super::collect_onion_ghosts(
            &settings(),
            &sim,
            &mut present,
            &doc,
            Some(osso),
            Some(2.0),
            PPM,
            &mut out,
        );
        ms.push(t0.elapsed().as_secs_f64() * 1e3);
        assert_eq!(out.len(), 4);
    }
    ms.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let (min, mediana) = (ms[0], ms[ms.len() / 2]);
    println!(
        "/proc/loadavg: {}",
        std::fs::read_to_string("/proc/loadavg")
            .unwrap_or_default()
            .trim()
    );
    println!(
        "onion de um rig: 4 fantasmas x {pecas} pecas — min {min:.3} ms · mediana {mediana:.3} ms \
         ({:.2}% de um quadro de 16,67 ms)",
        min / 16.667 * 100.0
    );
}

/// ⭐⭐⭐ **No modo `Keys` — o de OMISSÃO — um rig tem fantasmas, e os instantes saem dos OSSOS.**
///
/// ⛔⛔ **O defeito que este gate não deixa voltar, achado ANTES do smoke:** o `ghost_times` lia as
/// keyframes do alvo DESENHADO, e numa personagem riggada a imagem não tem nenhuma. Com o escopo
/// curado e os instantes não, o recurso continuava mudo **na configuração de fábrica** — *o mesmo
/// defeito do escopo, um nível abaixo*.
#[test]
fn in_keys_mode_the_instants_of_a_rig_come_from_its_bones() {
    let (sim, mut present, osso, arte) = rig_com_pele();
    let mut doc = TimelineDoc::new();
    for (t, v) in [(0.0, 0.0f32), (2.0, 1.2)] {
        doc.insert_key(
            osso,
            PropKind::Rotation,
            RationalTime::from_seconds(t),
            AnimValue::Float(v),
            Interp::Linear,
        );
    }
    let keys = OnionSettings {
        enabled: true,
        mode: OnionMode::Keys,
        ..OnionSettings::default()
    };
    let mut out = ph2d_render::LiftedInstances::default();
    super::collect_onion_ghosts(
        &keys,
        &sim,
        &mut present,
        &doc,
        Some(osso),
        Some(1.0),
        PPM,
        &mut out,
    );
    assert_eq!(
        out.len(),
        2,
        "no modo Keys um rig tinha de ghostar a key de tras e a da frente"
    );
    // ⛔ O CONTROLO: as keyframes da ARTE são zero — é por isso que os relógios existem.
    assert!(
        ph2d_timeline::entity_key_times(&doc, arte).is_empty(),
        "a fixtura keyou a ARTE: o gate acima passaria sem os relogios"
    );
}

/// ⭐ O RELÓGIO do onion — irmão deste ficheiro pelo teto de 600 LOC, e sub-módulo de propósito:
/// ele herda as fixturas daqui (`rig_com_pele`, `settings`, `PPM`), que é o molde do
/// `skin_at_time_tests` da W7.
#[path = "clock_tests.rs"]
mod relogio;
